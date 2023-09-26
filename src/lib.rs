/*!
 * Zip file parsing library
 *
 * ```no_run
 * # use zip_rs::ZipArchive;
 * let mut archive = ZipArchive::from_path("./foo.zip").unwrap();
 *
 * for file in archive.files() {
 *   // ...
 * }
 * ```
 */

#![deny(missing_debug_implementations)]

use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    ops::Deref,
    path::Path,
};

#[cfg(target_family = "unix")]
use std::os::unix::ffi::OsStrExt;

#[cfg(target_family = "windows")]
use std::os::unix::ffi::OsStrExt;

pub use common::*;
pub use error::ZipParseError;
use flate2::read::DeflateDecoder;
use parse::Parser;

mod common;
mod error;
mod parse;

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const GB: usize = 1024 * MB;

/// An entire ZIP archive file
#[derive(Debug)]
pub struct ZipArchive<'a, B: Deref<Target = [u8]>> {
    pub central_directory: CentralDirectory<'a>,
    parser: Parser<B>,
}

impl<'a> ZipArchive<'a, memmap::Mmap> {
    /// Parse a [`ZipArchive`] from a file path. The contents are memory mapped.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ZipParseError> {
        let file = File::open(path)?;
        let buffer = unsafe { memmap::Mmap::map(&file) }?;

        Self::from_buffer(buffer)
    }
}

impl<'a, B: Deref<Target = [u8]>> ZipArchive<'a, B> {
    /// Parse a [`ZipArchive`] from an existing buffer in memory
    pub fn from_buffer(buffer: B) -> Result<Self, ZipParseError> {
        let mut parser = Parser::new(buffer);

        let central_directory = parser.parse_central_directory()?;

        Ok(ZipArchive {
            central_directory,
            parser,
        })
    }

    /// Lazily iterate over the files in this archive
    pub fn files<'b>(
        &'b mut self,
    ) -> impl Iterator<Item = Result<CompressedZipFile<'a>, ZipParseError>> + 'b {
        // todo: can we avoid this clone by wrapping self.parser in RefCell?
        let files = self.central_directory.files.clone();

        files
            .into_iter()
            .map(move |file_header| self.parser.read_file(&file_header))
    }
}

#[derive(Debug, Clone)]
pub struct CentralDirectoryFileHeader<'a> {
    pub os: Os,
    pub metadata: Metadata<'a>,
    pub disk_num_start: u16,
    pub internal_attributes: InternalAttributes,
    pub external_attributes: ExternalAttributes,
    pub zip_specification_version: u8,
    pub local_header_offset: u32,
    pub comment: &'a [u8],
}

#[derive(Debug)]
pub struct EndCentralDirectory {
    pub disk_num: u16,
    pub disk_central_dir_num: u16,
    pub disk_entires: u16,
    pub total_entires: u16,
    pub central_dir_size: u32,
    pub central_dir_offset: u32,
}

#[derive(Debug, Clone)]
pub struct Metadata<'a> {
    pub version_needed: u16,
    pub compression_method: CompressionMethod,
    pub date_time_modified: DateTimeModified,
    pub flags: ZipFlags,
    pub name: &'a [u8],
    pub extra_field: &'a [u8],
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub crc: u32,
}

/// A single compressed ZIP file
#[derive(Debug)]
pub struct CompressedZipFile<'a> {
    pub metadata: Metadata<'a>,
    contents: &'a [u8],
}

impl<'a> CompressedZipFile<'a> {
    pub fn compressed_contents(&self) -> &[u8] {
        self.contents
    }

    /// Efficiently writes decompressed contents to sink without loading full
    /// decompressed contents into memory
    ///
    /// `limit` controls the max uncompressed file size that will be accepted. A
    /// `limit` of `None` implies no limit. Note that setting too high of a limit
    /// can make decoders susceptible to DoS through ZIP bombs or other means.
    pub fn write_with_limit(
        &self,
        w: &mut dyn Write,
        limit: Option<usize>,
    ) -> Result<(), ZipParseError> {
        if Some(self.metadata.uncompressed_size as usize) >= limit {
            return Err(ZipParseError::FileTooLarge(self.metadata.uncompressed_size));
        }

        match self.metadata.compression_method.name() {
            CompressionMethodName::None => {
                w.write_all(self.contents)?;
            }
            CompressionMethodName::Deflate => {
                let mut decoder = DeflateDecoder::new(self.contents);

                let amt_read = std::io::copy(&mut decoder, w)?;

                if amt_read != self.metadata.uncompressed_size {
                    return Err(ZipParseError::Generic("failed to write full buffer"));
                }
            }
            method => todo!("unimplemented compression method {:?}", method),
        }

        Ok(())
    }

    /// Efficiently writes decompressed contents to sink without loading full
    /// decompressed contents into memory.
    ///
    /// This method uses the default limit of 8 gigabytes. See
    /// [CompressedZipFile::write_with_limit] to configure this limit.
    pub fn write(&self, w: &mut dyn Write) -> Result<(), ZipParseError> {
        self.write_with_limit(w, Some(8 * GB))
    }

    /// Decompress full contents into memory
    ///
    /// `limit` controls the max uncompressed file size that will be accepted. A
    /// `limit` of `None` implies no limit. Note that setting too high of a limit
    /// can make decoders susceptible to DoS through ZIP bombs or other means.
    pub fn decompressed_contents_with_limit(
        &self,
        limit: Option<usize>,
    ) -> Result<Cow<[u8]>, ZipParseError> {
        if Some(self.metadata.uncompressed_size as usize) >= limit {
            return Err(ZipParseError::FileTooLarge(self.metadata.uncompressed_size));
        }

        match self.metadata.compression_method.name() {
            CompressionMethodName::None => return Ok(Cow::Borrowed(self.contents)),
            CompressionMethodName::Deflate => {
                let mut out = vec![0; self.metadata.uncompressed_size as usize];

                DeflateDecoder::new(self.contents).read_exact(&mut out)?;

                Ok(Cow::Owned(out))
            }
            method => todo!("unimplemented compression method {:?}", method),
        }
    }

    /// Decompress full contents into memory
    ///
    /// This method uses the default limit of 8 gigabytes. See
    /// [CompressedZipFile::decompressed_contents_with_limit] to configure this
    /// limit.
    pub fn decompressed_contents(&self) -> Result<Cow<[u8]>, ZipParseError> {
        self.decompressed_contents_with_limit(Some(8 * GB))
    }

    /// This file's `Path` inside the ZIP archive.
    ///
    /// Note that this path may reference file paths outside the archive through
    /// the use of absolute paths or the parent directory (`..`). The full file path
    /// should not be used when interacting with the host file system if the ZIP
    /// file is untrusted.
    pub fn file_path(&self) -> &Path {
        &Path::new(OsStr::from_bytes(self.metadata.name))
    }

    /// The raw bytes of this file's path inside the ZIP archive.
    ///
    /// Note that this path may reference file paths outside the archive through
    /// the use of absolute paths or the parent directory (`..`). The full file path
    /// should not be used when interacting with the host file system if the ZIP
    /// file is untrusted.
    pub fn file_path_bytes(&self) -> &'a [u8] {
        self.metadata.name
    }

    /// The algorithm used to compress this file.
    ///
    /// This is typically [`CompressionMethodName::None`] or
    /// [`CompressionMethodName::Deflate`].
    pub fn compression_method(&self) -> CompressionMethod {
        self.metadata.compression_method
    }
}

#[derive(Debug)]
pub struct CentralDirectory<'a> {
    pub files: Vec<CentralDirectoryFileHeader<'a>>,
    pub end: EndCentralDirectory,
}

#[cfg(test)]
mod test {
    use crate::{ZipArchive, MB};

    #[test]
    #[should_panic]
    fn zip_bomb() {
        let mut bomb = ZipArchive::from_path("files/bomb.zip").unwrap();

        for file in bomb.files() {
            let file = file.unwrap();

            file.decompressed_contents_with_limit(Some(20 * MB))
                .unwrap();
        }
    }
}
