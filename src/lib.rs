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
    /// Checks the chunk id to see if this is a container, and if it, returns container info.
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

#[derive(Debug)]
pub struct ParseChunkOutput {
    pub parsed_chunk: RiffChunkHeader,
    pub next_chunk_relative_position: u32,
}

pub const BUFFER_LEN: usize = size_of::<RiffChunkHeader>();

pub fn parse_chunk(data: [u8; BUFFER_LEN]) -> ParseChunkOutput {
    let chunk_header: RiffChunkHeader = transmute!(data);
    let data_len = chunk_header.chunk_len.get();
    let header_len = u32::try_from(size_of::<RiffChunkHeader>()).unwrap();
    ParseChunkOutput {
        parsed_chunk: chunk_header,
        next_chunk_relative_position: header_len + data_len.next_multiple_of(2),
    }
}
