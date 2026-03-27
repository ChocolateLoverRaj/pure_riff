use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use pure_riff::{Id, ProcessDataOutput, ReadInstructions, RiffChunkHeader, RiffChunksParser};
use zerocopy::transmute;

fn main() {
    let mut file =
        File::open("Warriyo, Laura Brehm - Mortals (feat. Laura Brehm) [NCS Release].wav").unwrap();

    // The entire WAVE file is a RIFF chunk
    let mut header_buffer = [Default::default(); size_of::<RiffChunkHeader>()];
    file.read_exact(&mut header_buffer).unwrap();
    let header: RiffChunkHeader = transmute!(header_buffer);

    let header_id = str::from_utf8(&header.chunk_id);
    println!("{header:#?} {header_id:?}");

    let root_container_info = header.container_info().unwrap().unwrap();
    // Read the container id
    let mut id_buffer = [Default::default(); size_of::<Id>()];
    file.seek(SeekFrom::Start(root_container_info.id.position.into()))
        .unwrap();
    file.read_exact(&mut id_buffer).unwrap();

    let container_id = str::from_utf8(&id_buffer);
    println!("container id: {container_id:?}");

    let mut parser = RiffChunksParser::new(root_container_info.sub_chunks.len);
    loop {
        let ReadInstructions {
            position,
            prefetch_len: _prefetch_len,
        } = parser.read_instructions();
        file.seek(SeekFrom::Start(
            (root_container_info.sub_chunks.position + position).into(),
        ))
        .unwrap();
        let mut buffer = [Default::default(); RiffChunksParser::MIN_READ_BUFFER_LEN];
        file.read_exact(&mut buffer).unwrap();
        let ProcessDataOutput {
            chunk_info,
            continue_parsing,
        } = parser.process_data(buffer);

        let chunk_id = str::from_utf8(&chunk_info.header.chunk_id);
        println!("{chunk_info:#?} {chunk_id:?}");
        if chunk_info.valid_len
            && let Some(container_info) = chunk_info.header.container_info()
        {
            let container_info = container_info.unwrap();
            // Read the container id
            let mut id_buffer = [Default::default(); size_of::<Id>()];
            file.seek(SeekFrom::Start(
                (root_container_info.sub_chunks.position
                    + chunk_info.position
                    + container_info.id.position)
                    .into(),
            ))
            .unwrap();
            file.read_exact(&mut id_buffer).unwrap();

            let container_id = str::from_utf8(&id_buffer);
            println!("container id: {container_id:?}");
        }

        match continue_parsing.unwrap() {
            Some(next_parser) => {
                parser = next_parser;
            }
            None => {
                break;
            }
        }
    }
}
