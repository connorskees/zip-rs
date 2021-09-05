#![deny(missing_debug_implementations)]

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    str::Utf8Error,
};

use common::*;
use flate2::read::DeflateDecoder;
use parse::Parser;

mod common;
mod parse;

#[derive(Debug, Clone)]
pub(crate) struct CentralDirectoryFileHeader<'a> {
    pub(crate) os: OS,
    pub(crate) metadata: Metadata<'a>,
    pub(crate) disk_num_start: u16,
    pub(crate) internal_attributes: InternalAttributes,
    pub(crate) external_attributes: ExternalAttributes,
    pub(crate) zip_specification_version: u8,
    pub(crate) local_header_offset: u32,
}

#[derive(Debug)]
pub(crate) struct EndCentralDirectory {
    pub(crate) disk_num: u16,
    pub(crate) disk_central_dir_num: u16,
    pub(crate) disk_entires: u16,
    pub(crate) total_entires: u16,
    pub(crate) central_dir_size: u32,
    pub(crate) central_dir_offset: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct Metadata<'a> {
    pub(crate) version_needed: u16,
    pub(crate) compression_method: CompressionMethod,
    pub(crate) date_time_modified: DateTimeModified,
    pub(crate) flags: ZipFlags,
    pub(crate) name: &'a [u8],
    pub(crate) compressed_size: u64,
    pub(crate) uncompressed_size: u64,
    pub(crate) crc: u32,
}

#[derive(Debug)]
pub struct CompressedZipFile<'a> {
    metadata: Metadata<'a>,
    contents: &'a [u8],
}

impl<'a> CompressedZipFile<'a> {
    pub fn decompressed_contents(&self) -> io::Result<Vec<u8>> {
        // disallow decompressing files over 1gb to avoid zip bombs
        assert!(
            self.metadata.uncompressed_size < 1_000_000,
            "decompressing files larger than 5gb is not supported"
        );

        let mut out = vec![0; self.metadata.uncompressed_size as usize];

        match self.metadata.compression_method {
            CompressionMethod::None => return Ok(self.contents.to_vec()),
            CompressionMethod::Deflate => {
                DeflateDecoder::new(self.contents).read_exact(&mut out)?;
            }
            method => println!("unimplemented compression method {:?}", method),
        }

        Ok(out)
    }

    pub fn file_path(&self) -> Result<&Path, Utf8Error> {
        let str_name = std::str::from_utf8(self.metadata.name)?;

        Ok(str_name.as_ref())
    }

    pub fn file_path_bytes(&self) -> &'a [u8] {
        self.metadata.name
    }
}

/// An entire ZIP archive
#[derive(Debug)]
pub struct ZipArchive<'a> {
    central_directory: CentralDirectory<'a>,
    parser: Parser,
}

#[derive(Debug)]
struct CentralDirectory<'a> {
    files: Vec<CentralDirectoryFileHeader<'a>>,
    end: EndCentralDirectory,
}

impl<'a> ZipArchive<'a> {
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let buffer = unsafe { memmap::Mmap::map(&file) }?;

        let mut parser = Parser::new(buffer);

        let central_directory = parser.parse_central_directory().unwrap();

        Ok(ZipArchive {
            central_directory,
            parser,
        })
    }

    pub fn files<'b>(&'b mut self) -> impl Iterator<Item = Option<CompressedZipFile<'a>>> + 'b {
        let files = self.central_directory.files.clone();

        files
            .into_iter()
            .map(move |file_header| self.parser.read_file(&file_header))
    }
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

            dbg!(file.file_path());

            dbg!(file.decompressed_contents());
        }
    }
}
