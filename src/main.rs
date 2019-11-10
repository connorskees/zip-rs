#![allow(dead_code, unused_imports)]
#![deny(missing_debug_implementations)]

extern crate bitreader;

use std::default::Default;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

use bitreader::BitReader;

const FILE_PATH: &str = "test.zip";
/// ZIP file magic bytes
const LOCAL_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
// const CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
// const END_CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];

#[derive(Debug)]
pub enum Version {
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

#[derive(Debug)]
pub enum InternalAttributes {
    ASCIIFile,
    Reserved,
    // TODO
    ControlFieldRecordsPrecedeLogicalRecords,
    Unused,
}

#[derive(Debug)]
pub enum ExternalAttributes {}

#[derive(Debug)]
pub struct CentralDirectory<'a> {
    version: Version,
    metadata: ZippedFileMetadata<'a>,
    comment: Option<&'a str>,
    internal_attributes: InternalAttributes,
    external_attributes: ExternalAttributes,
}

#[derive(Debug)]
pub struct DataDescriptor {
    uncompressed_size: u64,
    compressed_size: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct DateTimeModified {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    // years since 1980
    year: u16,
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

#[derive(Debug)]
pub struct ZippedFileMetadata<'a> {
    pub compression_method: CompressionMethod,
    pub date_time_modified: DateTimeModified,
    pub flags: ZipFlags,
    pub name: &'a str,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
}

#[derive(Debug, Copy, Clone)]
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
    metadata: ZippedFileMetadata<'a>,
    data: &'a [u8],
}

#[derive(Debug)]
pub struct ZippedArchive<'a> {
    files: Vec<ZippedFile<'a>>,
    central_directory: CentralDirectory<'a>,
}

macro_rules! read_bytes_to_buffer {
    ($reader:ident, $bytes:literal) => {
        if let Some(mut buffer) = Some([0u8; $bytes]) {
            $reader.read_exact(&mut buffer)?;
            // u32::from_le_bytes(buffer).to_be_bytes()
            buffer
        } else {
            unreachable!()
        }
    };
}

macro_rules! read_u8 {
    ($reader:ident) => {
        if let Some(mut buffer) = Some([0u8]) {
            $reader.read_exact(&mut buffer)?;
            u8::from_le_bytes(buffer)
        } else {
            unreachable!()
        }
    };
}

macro_rules! read_u16 {
    ($reader:ident) => {
        if let Some(mut buffer) = Some([0u8; 2]) {
            $reader.read_exact(&mut buffer)?;
            u16::from_le_bytes(buffer)
        } else {
            unreachable!()
        }
    };
}

macro_rules! read_u32 {
    ($reader:ident) => {
        if let Some(mut buffer) = Some([0u8; 4]) {
            $reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        } else {
            unreachable!()
        }
    };
}

fn main() -> io::Result<()> {
    let mut reader = BufReader::new(File::open(FILE_PATH)?);

    let mut buffer: [u8; 4] = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    assert_eq!(buffer, LOCAL_FILE_SIGNATURE);

    let version = read_u16!(reader);
    let bit_flags = ZipFlags::from_bytes(read_bytes_to_buffer!(reader, 2));
    let compression_method = CompressionMethod::from_u16(read_u16!(reader));
    let last_mod_date_time = DateTimeModified::from_bytes(read_bytes_to_buffer!(reader, 4));
    let crc = read_u32!(reader);
    let uncompressed_size = u64::from(read_u32!(reader));
    let compressed_size = u64::from(read_u32!(reader));
    let file_name_len = read_u16!(reader);
    let extra_field_len = read_u16!(reader);

    // dbg!(
    //     version,
    //     last_mod_date_time,
    //     compression_method,
    //     uncompressed_size,
    //     compressed_size,
    //     file_name_len
    // );

    let mut file_name_buffer = vec![0u8; file_name_len as usize];
    reader.read_exact(&mut file_name_buffer)?;

    let file_name = std::str::from_utf8(&file_name_buffer).unwrap();

    dbg!(file_name);

    let metadata = ZippedFileMetadata {
        compression_method: compression_method,
        date_time_modified: last_mod_date_time,
        flags: bit_flags,
        name: file_name,
        compressed_size: compressed_size,
        uncompressed_size: uncompressed_size,
    };

    dbg!(metadata);

    // dbg!(bit_reader.read_u8(2).unwrap());
    Ok(())
}
