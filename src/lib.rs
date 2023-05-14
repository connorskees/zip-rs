/*!
 * Zip file parsing library
 *
 * ```no_run
 * let file = ZipArchive::from_path("./foo.zip").unwrap();
 *
 * for file in files {
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

/// An entire ZIP archive file
#[derive(Debug)]
pub struct ZipArchive<'a, B: Deref<Target = [u8]>> {
    pub central_directory: CentralDirectory<'a>,
    parser: Parser<B>,
}

impl<'a> ZipArchive<'a, memmap::Mmap> {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ZipParseError> {
        let file = File::open(path)?;
        let buffer = unsafe { memmap::Mmap::map(&file) }?;

        Self::from_buffer(buffer)
    }
}

impl<'a, B: Deref<Target = [u8]>> ZipArchive<'a, B> {
    pub fn from_buffer(buffer: B) -> Result<Self, ZipParseError> {
        let mut parser = Parser::new(buffer);

        let central_directory = match parser.parse_central_directory() {
            Some(cd) => cd,
            None => return Err(ZipParseError::Generic("malformed zip file")),
        };

        Ok(ZipArchive {
            central_directory,
            parser,
        })
    }

    pub fn files<'b>(
        &'b mut self,
    ) -> impl Iterator<Item = Result<CompressedZipFile<'a>, ZipParseError>> + 'b {
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
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub crc: u32,
}

/// A single compressed zip file
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
    pub fn write(&self, w: &mut dyn Write) -> Result<(), ZipParseError> {
        // disallow decompressing files over 5gb to avoid zip bombs
        if self.metadata.uncompressed_size >= 5_000_000 {
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

    /// Decompress contents in one go into memory
    pub fn decompressed_contents(&self) -> Result<Cow<[u8]>, ZipParseError> {
        // disallow decompressing files over 5gb to avoid zip bombs
        if self.metadata.uncompressed_size >= 5_000_000 {
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

    pub fn file_path(&self) -> &Path {
        &Path::new(OsStr::from_bytes(self.metadata.name))
    }

    pub fn file_path_bytes(&self) -> &'a [u8] {
        self.metadata.name
    }

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
    use crate::ZipArchive;

    #[test]
    #[should_panic]
    fn zip_bomb() {
        let mut bomb = ZipArchive::from_path("files/bomb.zip").unwrap();

        for file in bomb.files() {
            let file = file.unwrap();

            file.decompressed_contents().unwrap();
        }
    }
}
