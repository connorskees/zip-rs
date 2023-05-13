use std::ops::Deref;

use crate::{
    common::*, CentralDirectory, CentralDirectoryFileHeader, CompressedZipFile,
    EndCentralDirectory, Metadata, ZipParseError,
};
use memchr::memmem;

#[derive(Debug)]
pub(super) struct Parser<B: Deref<Target = [u8]>> {
    buffer: B,
    cursor: usize,
}

impl<'a, B: Deref<Target = [u8]>> Parser<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn read_byte(&mut self) -> Option<u8> {
        self.buffer.get(self.cursor).copied().map(|b| {
            self.cursor += 1;
            b
        })
    }

    fn read_u16(&mut self) -> Option<u16> {
        let b1 = self.read_byte()?;
        let b2 = self.read_byte()?;

        Some(u16::from_le_bytes([b1, b2]))
    }

    fn read_u32(&mut self) -> Option<u32> {
        let b1 = self.read_byte()?;
        let b2 = self.read_byte()?;
        let b3 = self.read_byte()?;
        let b4 = self.read_byte()?;

        Some(u32::from_le_bytes([b1, b2, b3, b4]))
    }

    fn read_signature(&mut self, signature: [u8; 4]) -> bool {
        if self.buffer.len() <= self.cursor + 3 {
            return false;
        }

        let b1 = self.buffer[self.cursor];
        let b2 = self.buffer[self.cursor + 1];
        let b3 = self.buffer[self.cursor + 2];
        let b4 = self.buffer[self.cursor + 3];

        if [b1, b2, b3, b4] == signature {
            self.read_u32();
            true
        } else {
            false
        }
    }

    fn get_byte_range(&mut self, len: usize) -> Option<&'a [u8]> {
        let start = self.cursor;

        self.cursor += len;

        unsafe { self.lengthen_buffer_lifetime() }.get(start..self.cursor)
    }

    unsafe fn lengthen_buffer_lifetime(&self) -> &'a [u8] {
        &*(&*self.buffer as *const _)
    }

    fn read_metadata(&mut self) -> Option<Metadata<'a>> {
        let version_needed = self.read_u16()?;
        let flags = ZipFlags(self.read_u16()?);
        let compression_method = CompressionMethod::from(self.read_u16()?);
        let last_mod_date_time = DateTimeModified::from_u32(self.read_u32()?);
        let mut crc = self.read_u32()?;
        let mut compressed_size = u64::from(self.read_u32()?);
        let mut uncompressed_size = u64::from(self.read_u32()?);
        let file_name_len = usize::from(self.read_u16()?);
        let extra_field_len = self.read_u16()?;

        let file_name = self.get_byte_range(file_name_len)?;

        // skip extra fields
        self.cursor += usize::from(extra_field_len);

        if flags.has_data_descriptor() {
            let optional_signature = self.read_u32()?;
            crc = if optional_signature.to_be_bytes() == DATA_DESCRIPTOR_SIGNATURE {
                self.read_u32()?
            } else {
                optional_signature
            };

            compressed_size = u64::from(self.read_u32()?);
            uncompressed_size = u64::from(self.read_u32()?);
        }

        Some(Metadata {
            version_needed,
            compression_method,
            date_time_modified: last_mod_date_time,
            flags,
            name: file_name,
            crc,
            compressed_size,
            uncompressed_size,
        })
    }

    fn read_central_directory_file_headers(
        &mut self,
        offset: usize,
    ) -> Option<Vec<CentralDirectoryFileHeader<'a>>> {
        self.cursor = offset;

        let mut headers = Vec::new();

        while self.read_signature(CENTRAL_DIRECTORY_FILE_SIGNATURE) {
            let os = OS::from_u8(self.read_byte()?);
            let zip_specification_version = self.read_byte()?;
            let version_needed = self.read_u16()?;
            let bit_flags = ZipFlags(self.read_u16()?);
            let compression_method = CompressionMethod::from(self.read_u16()?);
            let date_time_modified = DateTimeModified::from_u32(self.read_u32()?);
            let crc = self.read_u32()?;
            let compressed_size = u64::from(self.read_u32()?);
            let uncompressed_size = u64::from(self.read_u32()?);
            let file_name_len = usize::from(self.read_u16()?);
            let extra_field_len = usize::from(self.read_u16()?);
            let comment_len = usize::from(self.read_u16()?);
            let disk_num_start = self.read_u16()?;
            let internal_attributes = InternalAttributes(self.read_u16()?);
            let _external_attributes = self.read_u32()?;
            let local_header_offset = self.read_u32()?;

            let file_name = self.get_byte_range(file_name_len)?;

            // skip extra fields
            self.cursor += extra_field_len;

            // skip comment
            self.cursor += comment_len;

            let metadata = Metadata {
                version_needed,
                compression_method,
                date_time_modified,
                flags: bit_flags,
                name: file_name,
                crc,
                compressed_size,
                uncompressed_size,
            };

            headers.push(CentralDirectoryFileHeader {
                os,
                metadata,
                internal_attributes,
                external_attributes: ExternalAttributes::TODO,
                disk_num_start,
                zip_specification_version,
                local_header_offset,
            })
        }

        Some(headers)
    }

    fn read_end_central_directory(&mut self, offset: usize) -> Option<EndCentralDirectory> {
        self.cursor = offset;

        assert!(self.read_signature(END_CENTRAL_DIRECTORY_SIGNATURE));

        let disk_num = self.read_u16()?;
        let disk_central_dir_num = self.read_u16()?;
        let disk_entires = self.read_u16()?;
        let total_entires = self.read_u16()?;
        let central_dir_size = self.read_u32()?;
        let central_dir_offset = self.read_u32()?;
        let comment_len = self.read_u16()?;

        // skip comment
        self.cursor += usize::from(comment_len);

        Some(EndCentralDirectory {
            disk_num,
            disk_central_dir_num,
            disk_entires,
            total_entires,
            central_dir_size,
            central_dir_offset,
        })
    }

    pub(super) fn parse_central_directory(&mut self) -> Option<CentralDirectory<'a>> {
        for offset in memmem::rfind_iter(&self.buffer, &END_CENTRAL_DIRECTORY_SIGNATURE) {
            let end = self.read_end_central_directory(offset)?;
            let file_headers =
                self.read_central_directory_file_headers(end.central_dir_offset as usize)?;

            return Some(CentralDirectory {
                files: file_headers,
                end,
            });
        }

        None
    }

    pub(super) fn read_file(
        &mut self,
        central_directory_header: &CentralDirectoryFileHeader,
    ) -> Result<CompressedZipFile<'a>, ZipParseError> {
        self.cursor = central_directory_header.local_header_offset as usize;

        assert!(self.read_signature(LOCAL_FILE_SIGNATURE));

        let metadata = self.read_metadata().unwrap();
        let contents = self
            .get_byte_range(metadata.compressed_size as usize)
            .unwrap();

        Ok(CompressedZipFile { metadata, contents })
    }
}
