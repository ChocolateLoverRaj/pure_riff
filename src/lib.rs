//! # Flow of parsing all chunks in a RIFF file
//! Keep track of a position.
//! Read [`BUFFER_LEN`] bytes at the position.
//! Give [`parse_chunk`] the data you read.
//! Now you have the id and len of the chunk.
//! You can then read the chunk data and do stuff with it.
//! You can recursively go through all containers by checking if the chunk is a container chunk with [`RiffChunkHeader::container_info`].
//! You can parse the sub chunks of a container chunk.
//! The end of the sub chunks is the start of the next sibling chunk of the container chunk (if present).
//!
//! # Handling possibly invalid data
//! This library does very minimal checking for invalid data, such as length fields that imply that data is out of bounds of the container it's in.
//! You have to do the checking yourself. If you feel like this library be better in terms of
//! handling potentially invalid data, create an issue.
//!
//! # RIFF Format
//! There are chunks. Each chunk has a field with a 4-byte id, a field for the size of its data
//! field, followed by a variable length data field. Each chunk is aligned by 2 bytes, so there is
//! a padding byte after the data field if needed.
//!
//! The type of a size in RIFF is a `u32` and is encoded in little-endian. This limits the size of
//! the RIFF file. There are extensions to RIFF that allow for larger sizes, but this library
//! currently does not implement them.
//!
//! A chunk can contain chunks inside of it.
//! There are special chunks which contain chunks inside of their data field.
//!
//! Some file extensions / formats, such as `.wav`, use RIFF as a container format. They contain
//! a single chunk with id `RIFF` which contains the file's data.
//!
//! See <https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#Explanation> for more information.
#![no_std]
pub use zerocopy;
use zerocopy::{FromBytes, Immutable, KnownLayout, little_endian::U32, transmute};

/// Supposed to be ASCII (and contains spaces for strings that are smaller than 4 characters).
/// Use [`str::from_utf8`] to parse it.
pub type Id = [u8; 4];

/// A RIFF header.
/// You can use `zerocopy` to "parse" this.
#[derive(Debug, Clone, Copy, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct RiffChunkHeader {
    pub chunk_id: Id,
    pub chunk_len: U32,
}

/// The length is too small for even the id
#[derive(Debug)]
pub struct ContainerInfoError;

/// To read the container id, see [`CONTAINER_ID_OFFSET`].
/// To read the sub chunks, see [`SUB_CHUNKS_OFFSET`].
#[derive(Debug)]
pub struct ContainerInfo {
    pub sub_chunks_len: u32,
}

/// The offset of the container id relative to the position of the container chunk.
pub const CONTAINER_ID_OFFSET: u32 = size_of::<RiffChunkHeader>() as u32;
/// The offset of the sub chunks relative to the position of the container chunk.
pub const SUB_CHUNKS_OFFSET: u32 = CONTAINER_ID_OFFSET + size_of::<Id>() as u32;

impl RiffChunkHeader {
    /// Checks the chunk id to see if this is a container, and if it, returns container info.
    pub fn container_info(&self) -> Option<Result<ContainerInfo, ContainerInfoError>> {
        match &self.chunk_id {
            b"RIFF" | b"LIST" => Some({
                let len_usize = usize::try_from(self.chunk_len.get()).unwrap();
                if len_usize >= size_of::<Id>() {
                    Ok(ContainerInfo {
                        sub_chunks_len: self.chunk_len.get()
                            - u32::try_from(size_of::<Id>()).unwrap(),
                    })
                } else {
                    Err(ContainerInfoError)
                }
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ParseChunkOutput {
    pub parsed_chunk: RiffChunkHeader,
    pub next_chunk_relative_position: u32,
}

pub const BUFFER_LEN: usize = size_of::<RiffChunkHeader>();

/// Parses the chunk header and returns it, along with the relative position of the next chunk header.
/// You can then read [`BUFFER_LEN`] at the new position to iterate through all chunks.
///
/// If you want to use untrusted / possibly invalid data, check for out of bounds access before reading the data.
pub fn parse_chunk(data: [u8; BUFFER_LEN]) -> ParseChunkOutput {
    let chunk_header: RiffChunkHeader = transmute!(data);
    let data_len = chunk_header.chunk_len.get();
    let header_len = u32::try_from(size_of::<RiffChunkHeader>()).unwrap();
    ParseChunkOutput {
        parsed_chunk: chunk_header,
        next_chunk_relative_position: header_len + data_len.next_multiple_of(2),
    }
}
