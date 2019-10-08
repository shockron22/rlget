extern crate http_req;
extern crate indicatif;

use self::indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use self::http_req::request;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::thread;

static ONE_KB: u64 = 1024;

pub struct Download {
    pub url: String,
    pub filename: String,
    pub memory: u64,
    pub threads: u64,
}

impl Download {
    pub fn get(self) {
        let mut writer = vec![0u8; 0];
        let content_length_resp = request::get(&self.url, &mut writer).unwrap();

        let content_length = content_length_resp.content_len().unwrap();
        println!("CONTENT LENGTH : {}", content_length);
        /* let content_length_u64 = content_length.parse::<u64>().expect("could not parse content length");
        let children = download_parts(self.filename, self.memory, self.threads, content_length_u64, request);
        for child in children {
            let _ = child.join();
        }*/
    }
}

fn download_parts(
    filename: String,
    memory: u64,
    threads: u64,
    content_length: u64,
    request: ureq::Request,
) -> Vec<std::thread::JoinHandle<()>> {
    let mut range_start = 0;
    let mut children = vec![];
    let chunk_size = content_length / threads - 1;

    let m = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta} remaining)  {msg}")
        .progress_chars("##-");

    println!("Spawning Threads...");
    for x in 0..threads {
        let mut range_end = chunk_size + range_start;
        if x == (threads - 1) {
            range_end = content_length
        }
        let range: String = format!("bytes={}-{}", range_start, range_end);
        let range_to_process: u64 = range_end - range_start;
        let pb = m.add(ProgressBar::new(range_to_process));
        pb.set_style(sty.clone());
        pb.set_message(&format!("thread #{}", x + 1));

        let mut request_ref = request.clone();
        let filename_ref = filename.clone();
        children.push(
            thread::spawn(move || {
                let max_buffer_size = ONE_KB * memory;
                let buffer_chunks: u64 = range_to_process / max_buffer_size;
                let chunk_remainder: u64 = range_to_process % max_buffer_size;

                let mut file_handle = File::create(filename_ref).unwrap();
                file_handle.seek(SeekFrom::Start(range_start)).unwrap();

                let file_range_resp = request_ref
                    .set("Range", &range)
                    .call();

                let mut file_range_reader = file_range_resp.into_reader();

                for _x in 0..buffer_chunks {
                    let mut vector_buffer = vec![0u8; max_buffer_size as usize];
                    file_range_reader.read_exact(&mut vector_buffer).unwrap();
                    file_handle.write(&vector_buffer).unwrap();
                    file_handle.flush().unwrap();
                    pb.inc(max_buffer_size);
                }

                if chunk_remainder != 0 {
                    let mut remainder_buffer = vec![0u8; max_buffer_size as usize];
                    file_range_reader.read_to_end(&mut remainder_buffer).unwrap();
                    file_handle.write(&remainder_buffer).unwrap();
                    file_handle.flush().unwrap();
                    pb.set_position(chunk_remainder);
                    pb.finish_with_message("--done--");
                }
            })
        );

        range_start = range_start + chunk_size + 1;
    }
    m.join_and_clear().unwrap();
    return children;
}