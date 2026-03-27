#![no_std]
pub use zerocopy;
use zerocopy::{FromBytes, Immutable, KnownLayout, little_endian::U32, transmute};

/// Supposed to be ASCII (and contains spaces for strings that are smaller than 4 characters).
pub type Id = [u8; 4];

/// A RIFF header.
/// You can use `zerocopy` to "parse" this.
///
/// For information about the format, see <https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#Explanation>.
#[derive(Debug, Clone, Copy, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct RiffChunkHeader {
    pub chunk_id: Id,
    pub chunk_len: U32,
}

/// The length is too small for even the id
#[derive(Debug)]
pub struct ContainerInfoError;

/// There would be negative len for sub chunks.
#[derive(Debug)]
pub struct SubChunksError;

#[derive(Debug)]
pub struct ContainerInfo {
    /// See the [`Id`] type for the len of the id.
    /// The id is different from the container chunk's id.
    /// Sample values for the id itself: "AVI ", "WAVE".
    pub id_position: u32,
    /// Position relative to position of the container chunk
    pub sub_chunks: SliceU32,
}

impl RiffChunkHeader {
    pub fn container_info(&self) -> Option<Result<ContainerInfo, ContainerInfoError>> {
        match &self.chunk_id {
            b"RIFF" | b"LIST" => Some({
                let len_usize = usize::try_from(self.chunk_len.get()).unwrap();
                if len_usize >= size_of::<Id>() {
                    let chunk_header_len_u32 = u32::try_from(size_of::<RiffChunkHeader>()).unwrap();
                    let id_len_u32 = u32::try_from(size_of::<Id>()).unwrap();
                    Ok(ContainerInfo {
                        id_position: chunk_header_len_u32,
                        sub_chunks: SliceU32 {
                            position: chunk_header_len_u32 + id_len_u32,
                            len: self.chunk_len.get() - id_len_u32,
                        },
                    })
                } else {
                    Err(ContainerInfoError)
                }
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SliceU32 {
    pub position: u32,
    pub len: u32,
}

#[derive(Debug)]
pub struct ParsedRiffChunkInfo {
    pub header: RiffChunkHeader,
    /// Position of this header relative to the start of the list of chunks.
    pub position: u32,
}

/// For the required len, use [`RiffChunksParser::MIN_READ_BUFFER_LEN`].
pub struct ReadInstructions {
    /// The position (where `0` is the start of the first RIFF chunk) to start reading from.
    pub position: u32,
    /// The number of bytes that it would be useful to read, including bytes that are not immediately needed but may be used to parse future chunks.
    /// This might speed up performance.
    pub prefetch_len: u32,
}

/// The remaining data is too small to contain another chunk.
#[derive(Debug)]
pub struct ParseChunksError;

pub struct ProcessDataOutput {
    pub chunk_info: ParsedRiffChunkInfo,
    pub continue_parsing: Result<Option<RiffChunksParser>, ParseChunksError>,
}

pub struct RiffChunksParser {
    position: u32,
}

impl RiffChunksParser {
    /// If you want to read from the beginning of the list of chunks, use position `0`.
    pub fn new(position: u32) -> Self {
        Self { position }
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub const MIN_READ_BUFFER_LEN: usize = size_of::<RiffChunkHeader>();

    pub fn process_data(&mut self, data: [u8; Self::MIN_READ_BUFFER_LEN]) -> ParsedRiffChunkInfo {
        let chunk_header: RiffChunkHeader = transmute!(data);
        let data_len = chunk_header.chunk_len.get();
        let header_len = u32::try_from(size_of::<RiffChunkHeader>()).unwrap();
        let chunk_position = self.position;
        // The specification says that there is a padding byte if the data len is odd
        self.position += header_len + data_len.next_multiple_of(2);
        ParsedRiffChunkInfo {
            header: chunk_header,
            position: chunk_position,
        }
    }
}
