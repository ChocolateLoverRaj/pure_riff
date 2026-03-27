use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use pure_riff::{Id, RiffChunksParser};

fn main() {
    let mut file =
        File::open("Warriyo, Laura Brehm - Mortals (feat. Laura Brehm) [NCS Release].wav").unwrap();
    let file_len = u32::try_from(file.metadata().unwrap().len()).unwrap();

    let mut depth = 0;
    let mut container_end_stack = vec![file_len];
    let mut parser = RiffChunksParser::new(0);
    while let Some(current_container_end) = container_end_stack.last() {
        if parser.position() == *current_container_end {
            if container_end_stack.pop().is_some() {
                depth -= 1;
                continue;
            } else {
                break;
            }
        }
        file.seek(SeekFrom::Start((parser.position()).into()))
            .unwrap();
        let mut buffer = [Default::default(); RiffChunksParser::MIN_READ_BUFFER_LEN];
        file.read_exact(&mut buffer).unwrap();
        let chunk_info = parser.process_data(buffer);

        let chunk_id = str::from_utf8(&chunk_info.header.chunk_id).unwrap();
        let chunk_len = chunk_info.header.chunk_len.get();
        for _ in 0..depth {
            print!("  ");
        }
        println!("{chunk_id} ({chunk_len} B)");
        if let Some(container_info) = chunk_info.header.container_info() {
            let container_info = container_info.unwrap();
            // Read the container id
            let mut id_buffer = [Default::default(); size_of::<Id>()];
            depth += 1;

            file.seek(SeekFrom::Start(
                (chunk_info.position + container_info.id_position).into(),
            ))
            .unwrap();
            file.read_exact(&mut id_buffer).unwrap();
            let id = str::from_utf8(&id_buffer).unwrap();
            for _ in 0..depth {
                print!("  ");
            }
            println!("{id}");

            parser =
                RiffChunksParser::new(chunk_info.position + container_info.sub_chunks.position);
            container_end_stack.push(
                chunk_info.position
                    + container_info.sub_chunks.position
                    + container_info.sub_chunks.len,
            );
        }
    }
}
