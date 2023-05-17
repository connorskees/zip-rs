pub const LOCAL_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
pub const CENTRAL_DIRECTORY_FILE_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
pub const DATA_DESCRIPTOR_SIGNATURE: [u8; 4] = [0x08, 0x07, 0x4b, 0x50];
pub const END_CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Os(pub u8);

impl Os {
    pub fn name(self) -> OsName {
        OsName::from_u8(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OsName {
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

impl OsName {
    pub fn from_u8(n: u8) -> OsName {
        match n {
            0 => OsName::Dos,
            1 => OsName::Amiga,
            2 => OsName::OpenVMS,
            3 => OsName::Unix,
            4 => OsName::VM,
            5 => OsName::AtariST,
            6 => OsName::OS2HPFS,
            7 => OsName::Macintosh,
            8 => OsName::ZSystem,
            9 => OsName::Cpm,
            10 => OsName::WindowsNtfs,
            11 => OsName::Mvs,
            12 => OsName::Vse,
            13 => OsName::AcornRisc,
            14 => OsName::Vfat,
            15 => OsName::AlternateMVS,
            16 => OsName::BeOS,
            17 => OsName::Tandem,
            18 => OsName::OS400,
            19 => OsName::Darwin,
            20..=255 => OsName::Unused,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DateTimeModified(u32);

impl DateTimeModified {
    pub fn from_u32(b: u32) -> DateTimeModified {
        Self(b)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompressionMethod(pub u16);

impl CompressionMethod {
    pub fn name(self) -> CompressionMethodName {
        CompressionMethodName::from_u16(self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompressionMethodName {
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
    Unknown,
}

impl CompressionMethodName {
    pub fn from_u16(n: u16) -> CompressionMethodName {
        match n {
            0 => CompressionMethodName::None,
            1 => CompressionMethodName::Shrink,
            2 => CompressionMethodName::Factor1,
            3 => CompressionMethodName::Factor2,
            4 => CompressionMethodName::Factor3,
            5 => CompressionMethodName::Factor4,
            6 => CompressionMethodName::Implode,
            7 | 11 | 13 | 15..=17 => CompressionMethodName::Reserved,
            8 => CompressionMethodName::Deflate,
            9 => CompressionMethodName::EnhancedDeflate,
            10 => CompressionMethodName::PKWareDclImplode,
            12 => CompressionMethodName::Bzip2,
            14 => CompressionMethodName::Lzma,
            18 => CompressionMethodName::IbmTerse,
            19 => CompressionMethodName::IbmLZ77z,
            98 => CompressionMethodName::PPMd,
            _ => CompressionMethodName::Unknown,
        }
    }
}

/// General purpose bit flags related to encoding
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ZipFlags(pub u16);

impl ZipFlags {
    pub const ENCRYPTED_FILE: u16 = 1 << 0;
    pub const COMPRESSION_OPTION1: u16 = 1 << 1;
    pub const COMPRESSION_OPTION2: u16 = 1 << 2;
    pub const DATA_DESCRIPTOR: u16 = 1 << 3;
    pub const ENHANCED_DEFLATION: u16 = 1 << 4;
    pub const COMPRESSED_PATCHED_DATA: u16 = 1 << 5;
    pub const STRONG_ENCRYPTION: u16 = 1 << 6;
    pub const LANGUAGE_ENCODING: u16 = 1 << 11;
    pub const RESERVED: u16 = 1 << 12;
    pub const MASK_HEADER_VALUES: u16 = 1 << 13;

    pub fn has_data_descriptor(&self) -> bool {
        (self.0 & Self::DATA_DESCRIPTOR) != 0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalAttributes(pub u32);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternalAttributes(pub u16);

impl InternalAttributes {
    pub const IS_ASCII: u16 = 1 << 0;
    pub const RESERVED: u16 = 1 << 1;
    pub const CONTROL_FIELD_RECORDS_PRECEDE_LOGICAL_RECORDS: u16 = 1 << 2;
}
