extern crate reqwest;
extern crate docopt;
extern crate serde;

use docopt::Docopt;
use serde::Deserialize;
use reqwest::header::RANGE;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::sync::Arc;
use std::thread;

static ONE_MB: u64 = (1024 * 1024);

const USAGE: &'static str = "
Rget

Usage:
  rget [options] <url>
  rget (-h | --help)

Options:
  -t THREADS, --threads=THREADS     Number of download threads. (default: 1)
  -m MEMORY, --memory=MEMORY        The amount of memory for each thread. (default: 256mb)
  -f FILENAME, --filename=FILENAME  The output file name (default: value at end of url after /)
  -h, --help                        Show this screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_threads: u64,
    flag_t: u64,
    flag_memory: u64,
    flag_m: u64,
    flag_filename: String,
    flag_f: String,
    arg_url: String,
}

fn main() -> std::io::Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let url: String = args.arg_url;
    let threads = match args.flag_threads {
        0 => 1,
        _ => args.flag_threads,
    };
    let filename = match args.flag_filename.len() {
        0 => url.split("/").last().expect("failed to parse filename from url").to_string(),
        _ => args.flag_filename,
    };
    let memory = match args.flag_memory {
        0 => 256,
        _ => args.flag_memory
    };
    println!("threads: {}", threads);
    println!("url: {}", url);
    println!("memory: {}", memory);
    println!("filename: {}", filename);

    let client = Arc::new(reqwest::Client::new());
    let content_length_resp = client
        .get(&url)
        .send()
        .expect("error in content-length request");

    match content_length_resp.content_length() {
        Some(content_length) => {
            let children = download_parts(client, url, filename, memory, threads, content_length);
            for child in children {
                let _ = child.join();
            }
        }
        None => (),
    }

    Ok(())
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

    for x in 0..threads {
        let mut range_end = chunk_size + range_start;
        if x == (threads - 1) {
            range_end = content_length
        }
        let mut range: String = format!("bytes={}-{}", range_start, range_end);

        let client_ref = client.clone();
        let filename_ref = filename.clone();
        let url_ref = url.clone();
        children.push(
            thread::spawn(move || {
                let range_to_process: u64 = range_end - range_start;
                let buffer_chunks: u64 = range_to_process / (ONE_MB * memory);
                let chunk_remainder: u64 = range_to_process % (ONE_MB * memory);

                let mut file_handle = File::create(filename_ref).unwrap();
                file_handle.seek(SeekFrom::Start(range_start)).unwrap();

                let mut file_range_resp = client_ref
                    .get(&url_ref)
                    .header(RANGE, range)
                    .send()
                    .unwrap();

                for _x in 0..buffer_chunks {
                    let mut vector_buffer = vec![0u8; (ONE_MB * memory) as usize];
                    let mut file_range_ref = file_range_resp.by_ref();
                    file_range_ref.read_exact(&mut vector_buffer).unwrap();
                    file_handle.write(&vector_buffer).unwrap();
                    file_handle.flush().unwrap();
                }

                if chunk_remainder != 0 {
                    file_range_resp.copy_to(&mut file_handle).unwrap();
                }
            })
        );

        range_start = range_start + chunk_size + 1;
    }
    return children;
}
