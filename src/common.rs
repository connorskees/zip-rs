pub const LOCAL_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
pub const CENTRAL_DIRECTORY_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
pub const DATA_DESCRIPTOR_SIGNATURE: [u8; 4] = [0x08, 0x07, 0x4b, 0x50];
pub const END_CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];

#[derive(Debug, Clone, Copy)]
pub enum OS {
    Dos = 0,
    Amiga = 1,
    OpenVMS = 2,
    Unix = 3,
    VM = 4,
    AtariST = 5,
    OS2HPFS = 6,
    Macintosh = 7,
    ZSystem = 8,
    Cpm = 9,
    WindowsNtfs = 10,
    Mvs = 11,
    Vse = 12,
    AcornRisc = 13,
    Vfat = 14,
    AlternateMVS = 15,
    BeOS = 16,
    Tandem = 17,
    OS400 = 18,
    Darwin = 19,
    Unused,
}

impl OS {
    pub fn from_u8(n: u8) -> OS {
        match n {
            0 => OS::Dos,
            1 => OS::Amiga,
            2 => OS::OpenVMS,
            3 => OS::Unix,
            4 => OS::VM,
            5 => OS::AtariST,
            6 => OS::OS2HPFS,
            7 => OS::Macintosh,
            8 => OS::ZSystem,
            9 => OS::Cpm,
            10 => OS::WindowsNtfs,
            11 => OS::Mvs,
            12 => OS::Vse,
            13 => OS::AcornRisc,
            14 => OS::Vfat,
            15 => OS::AlternateMVS,
            16 => OS::BeOS,
            17 => OS::Tandem,
            18 => OS::OS400,
            19 => OS::Darwin,
            20..=255 => OS::Unused,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DateTimeModified(u32);

impl DateTimeModified {
    pub fn from_u32(b: u32) -> DateTimeModified {
        Self(b)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CompressionMethod {
    None = 0,
    Shrink = 1,
    Factor1 = 2,
    Factor2 = 3,
    Factor3 = 4,
    Factor4 = 5,
    Implode = 6,
    Reserved = 7,
    Deflate = 8,
    EnhancedDeflate = 9,
    PKWareDclImplode = 10,
    Bzip2 = 12,
    Lzma = 14,
    IbmTerse = 18,
    IbmLZ77z = 19,
    PPMd = 98,
}

impl CompressionMethod {
    pub fn from_i64(n: i64) -> CompressionMethod {
        match n {
            0 => CompressionMethod::None,
            1 => CompressionMethod::Shrink,
            2 => CompressionMethod::Factor1,
            3 => CompressionMethod::Factor2,
            4 => CompressionMethod::Factor3,
            5 => CompressionMethod::Factor4,
            6 => CompressionMethod::Implode,
            7 | 11 | 13 | 15..=17 => CompressionMethod::Reserved,
            8 => CompressionMethod::Deflate,
            9 => CompressionMethod::EnhancedDeflate,
            10 => CompressionMethod::PKWareDclImplode,
            12 => CompressionMethod::Bzip2,
            14 => CompressionMethod::Lzma,
            18 => CompressionMethod::IbmTerse,
            19 => CompressionMethod::IbmLZ77z,
            98 => CompressionMethod::PPMd,
            _ => unimplemented!(),
        }
    }
}

impl From<u16> for CompressionMethod {
    fn from(n: u16) -> Self {
        CompressionMethod::from_i64(i64::from(n))
    }
}

/// General purpose bit flags related to encoding
#[derive(Debug, Copy, Clone)]
pub struct ZipFlags(pub u16);

#[allow(dead_code)]
impl ZipFlags {
    const ENCRYPTED_FILE: u16 = 1 << 0;
    const COMPRESSION_OPTION1: u16 = 1 << 1;
    const COMPRESSION_OPTION2: u16 = 1 << 2;
    const DATA_DESCRIPTOR: u16 = 1 << 3;
    const ENHANCED_DEFLATION: u16 = 1 << 4;
    const COMPRESSED_PATCHED_DATA: u16 = 1 << 5;
    const STRONG_ENCRYPTION: u16 = 1 << 6;
    const LANGUAGE_ENCODING: u16 = 1 << 11;
    const RESERVED: u16 = 1 << 12;
    const MASK_HEADER_VALUES: u16 = 1 << 13;

    pub fn has_data_descriptor(&self) -> bool {
        (self.0 & Self::DATA_DESCRIPTOR) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExternalAttributes {
    TODO,
}

#[derive(Debug, Clone, Copy)]
pub struct InternalAttributes(pub u16);
