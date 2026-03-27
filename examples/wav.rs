use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use pure_riff::{BUFFER_LEN, Id, ParseChunkOutput, parse_chunk};

fn main() {
    let mut file =
        File::open("Warriyo, Laura Brehm - Mortals (feat. Laura Brehm) [NCS Release].wav").unwrap();
    let file_len = u32::try_from(file.metadata().unwrap().len()).unwrap();

    let mut depth = 0;
    let mut container_end_stack = vec![file_len];
    let mut position = 0;
    // let mut parser = RiffChunksParser::new(0);
    while let Some(current_container_end) = container_end_stack.last() {
        // println!("position: {position}. stack: {container_end_stack:?}");
        if position == *current_container_end {
            if container_end_stack.pop().is_some() {
                depth -= 1;
                continue;
            } else {
                break;
            }
        }
        let mut buffer = [Default::default(); BUFFER_LEN];
        // Seek to position, but it's already at the position so no need to seek
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

            file.seek(SeekFrom::Start(
                (position + container_info.id_position).into(),
            ))
            .unwrap();
            file.read_exact(&mut id_buffer).unwrap();
            let id = str::from_utf8(&id_buffer).unwrap();
            for _ in 0..depth {
                print!("  ");
            }
            println!("{id}");

            // parser =
            //     RiffChunksParser::new(chunk_info.position + container_info.sub_chunks.position);
            container_end_stack.push(
                position + container_info.sub_chunks.position + container_info.sub_chunks.len,
            );
            position += container_info.sub_chunks.position;
        } else {
            position += next_chunk_relative_position;
        }
    }
}
