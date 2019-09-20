extern crate reqwest;
extern crate indicatif;

use self::indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use download::reqwest::header::RANGE;

use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::sync::Arc;
use std::thread;

static ONE_KB: u64 = 1024;

pub struct Download {
    pub url: String,
    pub filename: String,
    pub memory: u64,
    pub threads: u64,
    pub client: Arc<reqwest::Client>,
}

impl Default for Download {
    fn default() -> Download {
        Download {
            url: "".to_string(),
            filename: "".to_string(),
            memory: 256,
            threads: 1,
            client: Arc::new(reqwest::Client::new()),
        }
    }
}

impl Download {
    pub fn get(self) {
        let content_length_resp = &self.client
            .get(&self.url)
            .send()
            .expect("error in content-length request");

        match content_length_resp.content_length() {
            Some(content_length) => {
                let children = download_parts(self.client, self.url, self.filename, self.memory, self.threads, content_length);
                for child in children {
                    let _ = child.join();
                }
            }
            None => (),
        }

    }
}

fn download_parts(
    client: Arc<reqwest::Client>,
    url: String,
    filename: String,
    memory: u64,
    threads: u64,
    content_length: u64,
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

        let client_ref = client.clone();
        let filename_ref = filename.clone();
        let url_ref = url.clone();
        children.push(
            thread::spawn(move || {
                let max_buffer_size = ONE_KB * memory;
                let buffer_chunks: u64 = range_to_process / max_buffer_size;
                let chunk_remainder: u64 = range_to_process % max_buffer_size;

                let mut file_handle = File::create(filename_ref).unwrap();
                file_handle.seek(SeekFrom::Start(range_start)).unwrap();

                let mut file_range_resp = client_ref
                    .get(&url_ref)
                    .header(RANGE, range)
                    .send()
                    .unwrap();

                for _x in 0..buffer_chunks {
                    let mut vector_buffer = vec![0u8; max_buffer_size as usize];
                    let file_range_ref = file_range_resp.by_ref();
                    file_range_ref.read_exact(&mut vector_buffer).unwrap();
                    file_handle.write(&vector_buffer).unwrap();
                    file_handle.flush().unwrap();
                    pb.inc(max_buffer_size);
                }

                if chunk_remainder != 0 {
                    file_range_resp.copy_to(&mut file_handle).unwrap();
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