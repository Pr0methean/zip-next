use std::io;
use std::io::{IoResult, IoError};
use time::Tm;
use util;

static LOCAL_FILE_HEADER_SIGNATURE : u32 = 0x04034b50;
static DATA_DESCRIPTOR_SIGNATURE : u32 = 0x08074b50;
static CENTRAL_DIRECTORY_HEADER_SIGNATURE : u32 = 0x02014b50;
static DIGITAL_SIGNATURE_SIGNATURE : u32 = 0x05054b50;

#[deriving(FromPrimitive, Show)]
pub enum CompressionMethod
{
    Stored = 0,
    Shrunk = 1,
    Reduced1 = 2,
    Reduced2 = 3,
    Reduced3 = 4,
    Reduced4 = 5,
    Imploded = 6,
    Deflated = 8,
    Deflate64 = 9,
    PkwareImploding = 10,
    Bzip2 = 12,
    LZMA = 14,
    IBMTerse = 18,
    LZ77 = 19,
    WavPack = 97,
    PPMdI1 = 98,
    Unknown = 100000,
}

#[deriving(Show)]
pub struct LocalFileHeader
{
    extract_version: u16,

    // general purpose flags
    encrypted: bool, // bit 0
    // bit 1 & 2 unused
    has_descriptor: bool, // bit 3
    // bit 4 unused
    is_compressed_patch: bool, // bit 5
    strong_encryption: bool, // bit 6
    // bit 7 - 10 unused
    is_utf8: bool, // bit 11
    // bit 12 unused
    is_masked: bool, // bit 13
    // bit 14 & 15 unused
    
    compression_method: CompressionMethod,
    last_modified: Tm,
    pub crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    pub file_name: Vec<u8>,
    extra_field: Vec<u8>,
    header_end: u64,
}


impl LocalFileHeader
{
    pub fn parse<T: Reader+Seek>(reader: &mut T) -> IoResult<LocalFileHeader>
    {
        let signature = try!(reader.read_le_u32());
        if signature != LOCAL_FILE_HEADER_SIGNATURE
        {
            return Err(IoError {
                kind: io::MismatchedFileTypeForOperation,
                desc: "Invalid local file header",
                detail: None })
        }
        let version = try!(reader.read_le_u16());
        let flags = try!(reader.read_le_u16());
        let compression_method = try!(reader.read_le_u16());
        let last_mod_time = try!(reader.read_le_u16());
        let last_mod_date = try!(reader.read_le_u16());
        let crc = try!(reader.read_le_u32());
        let compressed_size = try!(reader.read_le_u32());
        let uncompressed_size = try!(reader.read_le_u32());
        let file_name_length = try!(reader.read_le_u16());
        let extra_field_length = try!(reader.read_le_u16());
        let file_name = try!(reader.read_exact(file_name_length as uint));
        let extra_field = try!(reader.read_exact(extra_field_length as uint));
        let header_end = try!(reader.tell());

        Ok(LocalFileHeader
           {
               extract_version: version,
               encrypted: (flags & (1 << 0)) != 0,
               has_descriptor: (flags & (1 << 3)) != 0,
               is_compressed_patch: (flags & (1 << 5)) != 0,
               strong_encryption: (flags & (1 << 6)) != 0,
               is_utf8: (flags & (1 << 11)) != 0,
               is_masked: (flags & (1 << 13)) != 0,
               compression_method: FromPrimitive::from_u16(compression_method).unwrap_or(Unknown),
               last_modified: util::msdos_datetime_to_tm(last_mod_time, last_mod_date),
               crc32: crc,
               compressed_size: compressed_size,
               uncompressed_size: uncompressed_size,
               file_name: file_name,
               extra_field: extra_field,
               header_end: header_end,
           })
    }
}

struct DataDescriptor
{
    compressed_size: u32,
    uncompressed_size: u32,
    crc32: u32,
}

impl DataDescriptor
{
    pub fn parse<T: Reader>(reader: &mut T) -> IoResult<DataDescriptor>
    {
        let first = try!(reader.read_le_u32());
        let compressed = if first == DATA_DESCRIPTOR_SIGNATURE
        {
            try!(reader.read_le_u32())
        }
        else
        {
            first
        };

        let uncompressed = try!(reader.read_le_u32());
        let crc = try!(reader.read_le_u32());

        Ok(DataDescriptor
           {
               compressed_size: compressed,
               uncompressed_size: uncompressed,
               crc32: crc,
           })
    }
}

struct CentralDirectoryHeader
{
    made_by: u16,
    version_needed: u16,

    // general purpose flags
    encrypted: bool, // bit 0
    // bit 1 & 2 unused
    has_descriptor: bool, // bit 3
    // bit 4 unused
    is_compressed_patch: bool, // bit 5
    strong_encryption: bool, // bit 6
    // bit 7 - 10 unused
    is_utf8: bool, // bit 11
    // bit 12 unused
    is_masked: bool, // bit 13
    // bit 14 & 15 unused

    compression_method: CompressionMethod,
    last_modified_time: Tm,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name: Vec<u8>,
    extra_field: Vec<u8>,
    file_comment: Vec<u8>,
    disk_number: u16,
    file_offset: u32,
}

impl CentralDirectoryHeader
{
    pub fn parse<T: Reader>(reader: &mut T) -> IoResult<CentralDirectoryHeader>
    {
        let signature = try!(reader.read_le_u32());
        if signature != CENTRAL_DIRECTORY_HEADER_SIGNATURE
        {
            return Err(IoError {
                kind: io::MismatchedFileTypeForOperation,
                desc: "Invalid central directory header",
                detail: None })
        }

        let made_by = try!(reader.read_le_u16());
        let version_needed = try!(reader.read_le_u16());
        let flags = try!(reader.read_le_u16());
        let compression = try!(reader.read_le_u16());
        let last_mod_time = try!(reader.read_le_u16());
        let last_mod_date = try!(reader.read_le_u16());
        let crc = try!(reader.read_le_u32());
        let compressed_size = try!(reader.read_le_u32());
        let uncompressed_size = try!(reader.read_le_u32());
        let file_name_length = try!(reader.read_le_u16()) as uint;
        let extra_field_length = try!(reader.read_le_u16()) as uint;
        let file_comment_length = try!(reader.read_le_u16()) as uint;
        let disk_number = try!(reader.read_le_u16());
        try!(reader.read_le_u16()); // internal file attribute
        try!(reader.read_le_u32()); // external file attribute
        let offset = try!(reader.read_le_u32());
        let file_name = try!(reader.read_exact(file_name_length));
        let extra_field = try!(reader.read_exact(extra_field_length));
        let file_comment  = try!(reader.read_exact(file_comment_length));

        Ok(CentralDirectoryHeader
           {
               made_by: made_by,
               version_needed: version_needed,
               encrypted: flags & (1 << 0) != 0,
               has_descriptor: flags & (1 << 3) != 0,
               is_compressed_patch: flags & (1 << 5) != 0,
               strong_encryption: flags & (1 << 6) != 0,
               is_utf8: flags & (1 << 11) != 0,
               is_masked: flags & (1 << 13) != 0,
               compression_method: FromPrimitive::from_u16(compression).unwrap_or(Unknown),
               last_modified_time: util::msdos_datetime_to_tm(last_mod_time, last_mod_date),
               crc32: crc,
               compressed_size: compressed_size,
               uncompressed_size: uncompressed_size,
               file_name: file_name,
               extra_field: extra_field,
               file_comment: file_comment,
               disk_number: disk_number,
               file_offset: offset,
            })
    }
}

struct DigitalSignature
{
    data: Vec<u8>,
}

impl DigitalSignature
{
    pub fn parse<T: Reader>(reader: &mut T) -> IoResult<DigitalSignature>
    {
        let magic = try!(reader.read_le_u32());
        if magic != DIGITAL_SIGNATURE_SIGNATURE
        {
            return Err(IoError {
                kind: io::MismatchedFileTypeForOperation,
                desc: "Invalid digital signature header",
                detail: None })
        }
        let size = try!(reader.read_le_u16()) as uint;
        let data = try!(reader.read_exact(size));
        Ok(DigitalSignature { data: data })
    }
}
