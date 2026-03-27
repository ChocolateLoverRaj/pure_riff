use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use pure_riff::{
    BUFFER_LEN, CONTAINER_ID_OFFSET, Id, ParseChunkOutput, SUB_CHUNKS_OFFSET, parse_chunk,
};

fn main() {
    let mut file =
        File::open("Warriyo, Laura Brehm - Mortals (feat. Laura Brehm) [NCS Release].wav").unwrap();
    let file_len = u32::try_from(file.metadata().unwrap().len()).unwrap();

    let mut depth = 0;
    let mut container_end_stack = vec![file_len];
    let mut position = 0;
    while let Some(current_container_end) = container_end_stack.last() {
        if position == *current_container_end {
            if container_end_stack.pop().is_some() {
                depth -= 1;
                continue;
            } else {
                break;
            }
        }
        let mut buffer = [Default::default(); BUFFER_LEN];
        file.seek(SeekFrom::Start(position.into())).unwrap();
        file.read_exact(&mut buffer).unwrap();
        let ParseChunkOutput {
            parsed_chunk,
            next_chunk_relative_position,
        } = parse_chunk(buffer);

        let chunk_id = str::from_utf8(&parsed_chunk.chunk_id).unwrap();
        let chunk_len = parsed_chunk.chunk_len.get();
        for _ in 0..depth {
            print!("  ");
        }
        println!("{chunk_id} ({chunk_len} B)");
        if let Some(container_info) = parsed_chunk.container_info() {
            let container_info = container_info.unwrap();
            // Read the container id
            let mut id_buffer = [Default::default(); size_of::<Id>()];
            depth += 1;

            file.seek(SeekFrom::Start((position + CONTAINER_ID_OFFSET).into()))
                .unwrap();
            file.read_exact(&mut id_buffer).unwrap();
            let id = str::from_utf8(&id_buffer).unwrap();
            for _ in 0..depth {
                print!("  ");
            }
            println!("{id}");

            container_end_stack.push(position + SUB_CHUNKS_OFFSET + container_info.sub_chunks_len);
            position += SUB_CHUNKS_OFFSET;
        } else {
            position += next_chunk_relative_position;
        }
    }
}
