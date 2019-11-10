#![allow(dead_code, unused_imports)]
#![deny(missing_debug_implementations)]

extern crate bitreader;

use std::default::Default;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

use bitreader::BitReader;

// TODO // u32::from_le_bytes(buffer).to_be_bytes()

const FILE_PATH: &str = "test.zip";
/// ZIP file magic bytes
const LOCAL_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const DATA_DESCRIPTOR_SIGNATURE: [u8; 4] = [0x08, 0x07, 0x4b, 0x50];
const CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const END_CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];

macro_rules! read_bytes_to_buffer {
    ($reader:expr, $bytes:literal) => {
        if let Some(mut buffer) = Some([0u8; $bytes]) {
            $reader.read_exact(&mut buffer)?;
            buffer
        } else {
            unreachable!()
        }
    };
}

macro_rules! read_u8 {
    ($reader:expr) => {
        if let Some(mut buffer) = Some([0u8]) {
            $reader.read_exact(&mut buffer)?;
            u8::from_le_bytes(buffer)
        } else {
            unreachable!()
        }
    };
}

macro_rules! read_u16 {
    ($reader:expr) => {
        if let Some(mut buffer) = Some([0u8; 2]) {
            $reader.read_exact(&mut buffer)?;
            u16::from_le_bytes(buffer)
        } else {
            unreachable!()
        }
    };
}

macro_rules! read_u32 {
    ($reader:expr) => {
        if let Some(mut buffer) = Some([0u8; 4]) {
            $reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        } else {
            unreachable!()
        }
    };
}

#[derive(Debug)]
pub enum OS {
    DOS = 0,
    Amiga = 1,
    OpenVMS = 2,
    UNIX = 3,
    VM = 4,
    AtariST = 5,
    OS2HPFS = 6,
    Macintosh = 7,
    ZSystem = 8,
    CPM = 9,
    WindowsNTFS = 10,
    MVS = 11,
    VSE = 12,
    AcornRisc = 13,
    VFAT = 14,
    AlternateMVS = 15,
    BeOS = 16,
    Tandem = 17,
    OS400 = 18,
    Darwin = 19,
    Unused,
}

impl Default for OS {
    fn default() -> OS {
        OS::Unused
    }
}

impl OS {
    pub fn from_u8(n: u8) -> OS {
        match n {
            0 => OS::DOS,
            1 => OS::Amiga,
            2 => OS::OpenVMS,
            3 => OS::UNIX,
            4 => OS::VM,
            5 => OS::AtariST,
            6 => OS::OS2HPFS,
            7 => OS::Macintosh,
            8 => OS::ZSystem,
            9 => OS::CPM,
            10 => OS::WindowsNTFS,
            11 => OS::MVS,
            12 => OS::VSE,
            13 => OS::AcornRisc,
            14 => OS::VFAT,
            15 => OS::AlternateMVS,
            16 => OS::BeOS,
            17 => OS::Tandem,
            18 => OS::OS400,
            19 => OS::Darwin,
            20..=255 => OS::Unused,
        }
    }
}

#[derive(Debug, Default)]
pub struct InternalAttributes {
    is_ascii: bool,
    control_field_records_precede_logical_records: bool,
}

impl InternalAttributes {
    pub fn from_bytes(b: [u8; 2]) -> InternalAttributes {
        let mut bit_reader = BitReader::new(&b);
        let is_ascii = bit_reader.read_u8(1).unwrap() == 1u8;
        bit_reader.skip(1).unwrap();
        let control_field_records_precede_logical_records = bit_reader.read_u8(1).unwrap() == 1u8;
        InternalAttributes {
            is_ascii,
            control_field_records_precede_logical_records,
        }
    }
}

#[derive(Debug)]
pub enum ExternalAttributes {
    TODO,
}

impl Default for ExternalAttributes {
    fn default() -> ExternalAttributes {
        ExternalAttributes::TODO
    }
}

#[derive(Debug, Default)]
pub struct CentralDirectory {
    pub os: OS,
    pub metadata: ZippedFileMetadata,
    pub comment: Option<String>,
    pub disk_num_start: u16,
    pub internal_attributes: InternalAttributes,
    pub external_attributes: ExternalAttributes,
    pub zip_specification_version: u8,
    pub local_header_offset: u32,
}

#[derive(Debug)]
pub struct DataDescriptor {
    pub uncompressed_size: u64,
    pub compressed_size: u64,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTimeModified {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    // years since 1980
    year: u16,
}

impl fmt::Display for DateTimeModified {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:0>4}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl DateTimeModified {
    pub fn from_bytes(b: [u8; 4]) -> DateTimeModified {
        let mut bit_reader = BitReader::new(&b);
        let second = 2 * bit_reader.read_u8(5).unwrap();
        let minute = bit_reader.read_u8(6).unwrap();
        let hour = bit_reader.read_u8(5).unwrap();

        let day = bit_reader.read_u8(5).unwrap();
        let month = bit_reader.read_u8(4).unwrap();
        let year = 1980 + bit_reader.read_u16(7).unwrap();

        DateTimeModified {
            second,
            minute,
            hour,
            day,
            month,
            year,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CompressionMethod {
    None = 0,
    Shrunk = 1,
    Factor1 = 2,
    Factor2 = 3,
    Factor3 = 4,
    Factor4 = 5,
    Imploded = 6,
    Reserved = 7,
    Deflated = 8,
    EnhancedDeflated = 9,
    PKWareDclImploded = 10,
    BZIP2 = 12,
    LZMA = 14,
    IbmTerse = 18,
    IbmLZ77z = 19,
    PPMd = 98,
}

impl CompressionMethod {
    pub fn from_i64(n: i64) -> CompressionMethod {
        match n {
            0 => CompressionMethod::None,
            1 => CompressionMethod::Shrunk,
            2 => CompressionMethod::Factor1,
            3 => CompressionMethod::Factor2,
            4 => CompressionMethod::Factor3,
            5 => CompressionMethod::Factor4,
            6 => CompressionMethod::Imploded,
            7 | 11 | 13 | 15..=17 => CompressionMethod::Reserved,
            8 => CompressionMethod::Deflated,
            9 => CompressionMethod::EnhancedDeflated,
            10 => CompressionMethod::PKWareDclImploded,
            12 => CompressionMethod::BZIP2,
            14 => CompressionMethod::LZMA,
            18 => CompressionMethod::IbmTerse,
            19 => CompressionMethod::IbmLZ77z,
            98 => CompressionMethod::PPMd,
            _ => unimplemented!(),
        }
    }

    pub fn from_u8(n: u8) -> CompressionMethod {
        CompressionMethod::from_i64(i64::from(n))
    }

    pub fn from_u16(n: u16) -> CompressionMethod {
        CompressionMethod::from_i64(i64::from(n))
    }
}

impl Default for CompressionMethod {
    fn default() -> CompressionMethod {
        CompressionMethod::None
    }
}

#[derive(Debug, Default)]
pub struct ZippedFileMetadata {
    pub version_needed: u16,
    pub compression_method: CompressionMethod,
    pub date_time_modified: DateTimeModified,
    pub flags: ZipFlags,
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub crc: [u8; 4],
    pub extra_fields: Vec<u8>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ZipFlags {
    is_encrypted: bool,
    /*
        (For Method 6 - Imploding)
        Bit 1: If the compression method used was type 6,
               Imploding, then this bit, if set, indicates
               an 8K sliding dictionary was used.  If clear,
               then a 4K sliding dictionary was used.

        Bit 2: If the compression method used was type 6,
               Imploding, then this bit, if set, indicates
               3 Shannon-Fano trees were used to encode the
               sliding dictionary output.  If clear, then 2
               Shannon-Fano trees were used.

        (For Methods 8 and 9 - Deflating)
        Bit 2  Bit 1
          0      0    Normal (-en) compression option was used.
          0      1    Maximum (-exx/-ex) compression option was used.
          1      0    Fast (-ef) compression option was used.
          1      1    Super Fast (-es) compression option was used.

        (For Method 14 - LZMA)
        Bit 1: If the compression method used was type 14,
               LZMA, then this bit, if set, indicates
               an end-of-stream (EOS) marker is used to
               mark the end of the compressed data stream.
               If clear, then an EOS marker is not present
               and the compressed data size must be known
               to extract.

        Note:  Bits 1 and 2 are undefined if the compression
               method is any other.

    // compression option
    // compression option
    */
    has_data_descriptor: bool,
    enhanced_deflation: bool,
    compressed_patched_data: bool,
    /// Strong encryption.  If this bit is set, you MUST
    /// set the version needed to extract value to at least
    /// 50 and you MUST also set bit 0.  If AES encryption
    /// is used, the version needed to extract value MUST
    /// be at least 51. See the section describing the Strong
    /// Encryption Specification for details.  Refer to the
    /// section in this document entitled "Incorporating PKWARE
    /// Proprietary Technology into Your Product" for more
    /// information.
    strong_encryption: bool,
    is_utf8: bool,
    /// Set when encrypting the Central Directory to indicate
    /// selected data values in the Local Header are masked to
    /// hide their actual values.  See the section describing
    /// the Strong Encryption Specification for details.  Refer
    /// to the section in this document entitled "Incorporating
    /// PKWARE Proprietary Technology into Your Product" for
    /// more information.
    mask_header_values: bool,
}

impl ZipFlags {
    pub fn from_bytes(b: [u8; 2]) -> ZipFlags {
        let mut bit_reader = BitReader::new(&b);
        let is_encrypted = bit_reader.read_u8(1).unwrap() == 1u8;
        // TODO: bits 1 and 2
        let _ = bit_reader.read_u8(1).unwrap() == 1u8;
        let _ = bit_reader.read_u8(1).unwrap() == 1u8;
        let has_data_descriptor = bit_reader.read_u8(1).unwrap() == 1u8;
        let enhanced_deflation = bit_reader.read_u8(1).unwrap() == 1u8;
        let compressed_patched_data = bit_reader.read_u8(1).unwrap() == 1u8;
        let strong_encryption = bit_reader.read_u8(1).unwrap() == 1u8;
        bit_reader.skip(3).unwrap();
        let is_utf8 = bit_reader.read_u8(1).unwrap() == 1u8;
        let mask_header_values = bit_reader.read_u8(1).unwrap() == 1u8;
        bit_reader.skip(2).unwrap();

        ZipFlags {
            is_encrypted,
            has_data_descriptor,
            enhanced_deflation,
            compressed_patched_data,
            strong_encryption,
            is_utf8,
            mask_header_values,
        }
    }
}

/// A single file within a ZIP archive
#[derive(Debug)]
pub struct ZippedFile<'a> {
    metadata: ZippedFileMetadata,
    data: &'a [u8],
}

#[derive(Debug)]
pub struct ZippedArchive<'a, R: Read + BufRead> {
    files: Vec<ZippedFile<'a>>,
    central_directory: CentralDirectory,
    reader: R,
}

impl<'a, R: Read + BufRead> ZippedArchive<'a, R> {
    pub fn from_buffer(r: R) -> ZippedArchive<'a, R> {
        ZippedArchive {
            files: Vec::new(),
            central_directory: Default::default(),
            reader: r,
        }
    }

    pub fn unzip(&mut self) -> io::Result<()> {
        // Check file magic bytes
        assert_eq!(read_bytes_to_buffer!(self.reader, 4), LOCAL_FILE_SIGNATURE);

        loop {
            // Match on header using magic bytes
            match read_bytes_to_buffer!(self.reader, 4) {
                LOCAL_FILE_SIGNATURE => self.read_file()?,
                CENTRAL_DIRECTORY_SIGNATURE => self.read_central_directory()?,
                _ => unimplemented!(),
            };
        }

        Ok(())
    }

    pub fn read_metadata(&mut self) -> Result<ZippedFileMetadata, io::Error> {
        let version_needed = read_u16!(self.reader);
        let bit_flags = ZipFlags::from_bytes(read_bytes_to_buffer!(self.reader, 2));
        let compression_method = CompressionMethod::from_u16(read_u16!(self.reader));
        let last_mod_date_time =
            DateTimeModified::from_bytes(read_bytes_to_buffer!(self.reader, 4));
        let crc: [u8; 4] = read_bytes_to_buffer!(self.reader, 4);
        let uncompressed_size = u64::from(read_u32!(self.reader));
        let compressed_size = u64::from(read_u32!(self.reader));
        let file_name_len = read_u16!(self.reader);
        let extra_field_len = read_u16!(self.reader);

        let mut file_name_buffer = vec![0u8; file_name_len as usize];
        self.reader.read_exact(&mut file_name_buffer)?;

        let mut extra_field_buffer = vec![0u8; extra_field_len as usize];
        self.reader.read_exact(&mut extra_field_buffer)?;

        let file_name = std::str::from_utf8(&file_name_buffer).unwrap().to_string();

        Ok(ZippedFileMetadata {
            version_needed,
            compression_method,
            date_time_modified: last_mod_date_time,
            flags: bit_flags,
            name: file_name,
            crc,
            compressed_size: compressed_size,
            uncompressed_size: uncompressed_size,
            extra_fields: Vec::from(extra_field_buffer),
        })
    }

    pub fn read_file(&mut self) -> Result<(), io::Error> {
        let mut metadata = self.read_metadata()?;

        if metadata.flags.has_data_descriptor {
            let optional_signature: [u8; 4] = read_bytes_to_buffer!(self.reader, 4);
            metadata.crc = if optional_signature == DATA_DESCRIPTOR_SIGNATURE {
                read_bytes_to_buffer!(self.reader, 4)
            } else {
                optional_signature
            };
            metadata.compressed_size = u64::from(read_u32!(self.reader));
            metadata.uncompressed_size = u64::from(read_u32!(self.reader));
        }

        dbg!(&metadata);

        self.files.push(ZippedFile {
            metadata,
            data: &[0u8],
        });
        Ok(())
    }

    pub fn read_central_directory(&mut self) -> Result<(), io::Error> {
        let os = OS::from_u8(read_u8!(self.reader));
        let zip_specification_version = read_u8!(self.reader);
        let version_needed = read_u16!(self.reader);
        let bit_flags = ZipFlags::from_bytes(read_bytes_to_buffer!(self.reader, 2));
        let compression_method = CompressionMethod::from_u16(read_u16!(self.reader));
        let date_time_modified =
            DateTimeModified::from_bytes(read_bytes_to_buffer!(self.reader, 4));
        let crc: [u8; 4] = read_bytes_to_buffer!(self.reader, 4);
        let uncompressed_size = u64::from(read_u32!(self.reader));
        let compressed_size = u64::from(read_u32!(self.reader));
        let file_name_len = read_u16!(self.reader);
        let extra_field_len = read_u16!(self.reader);
        let comment_len = read_u16!(self.reader);
        let disk_num_start = read_u16!(self.reader);
        let internal_attributes =
            InternalAttributes::from_bytes(read_bytes_to_buffer!(self.reader, 2));
        let external_attributes: [u8; 4] = read_bytes_to_buffer!(self.reader, 4);
        let local_header_offset = read_u32!(self.reader);

        let mut file_name_buffer = vec![0u8; file_name_len as usize];
        self.reader.read_exact(&mut file_name_buffer)?;

        let file_name = std::str::from_utf8(&file_name_buffer).unwrap().to_string();

        let mut extra_field_buffer = vec![0u8; extra_field_len as usize];
        self.reader.read_exact(&mut extra_field_buffer)?;

        let comment = if comment_len > 0 {
            let mut comment_buffer = vec![0u8; comment_len as usize];
            self.reader.read_exact(&mut comment_buffer)?;

            Some(std::str::from_utf8(&comment_buffer).unwrap().to_string())
        } else {
            None
        };

        let metadata = ZippedFileMetadata {
            version_needed,
            compression_method,
            date_time_modified,
            flags: bit_flags,
            name: file_name,
            crc,
            compressed_size: compressed_size,
            uncompressed_size: uncompressed_size,
            extra_fields: Vec::from(extra_field_buffer),
        };

        self.central_directory = CentralDirectory {
            os,
            comment,
            metadata,
            internal_attributes,
            external_attributes: ExternalAttributes::TODO,
            disk_num_start,
            zip_specification_version,
            local_header_offset,
        };
        Ok(())
    }
}

impl<'a> ZippedArchive<'a, BufReader<File>> {
    pub fn from_path<P: AsRef<std::path::Path>>(p: P) -> ZippedArchive<'a, BufReader<File>> {
        let buffer = BufReader::new(File::open(FILE_PATH).unwrap());
        ZippedArchive::from_buffer(buffer)
    }
}

fn main() -> io::Result<()> {
    let zip = ZippedArchive::from_path(FILE_PATH).unzip()?;

    // dbg!(bit_reader.read_u8(2).unwrap());
    Ok(())
}
