use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::sync::Arc;
use std::thread;

mod network;
mod progress;

static ONE_KB: u64 = 1024;

pub struct Download {
    pub url: String,
    pub filename: String,
    pub memory: u64,
    pub threads: u64,
    pub network: network::Network,
    pub progress: progress::Progress,
}

impl Default for Download {
    fn default() -> Download {
        Download {
            url: "".to_string(),
            filename: "".to_string(),
            memory: 256,
            threads: 1,
            network: network::Network {
                ..Default::default()
            },
            progress: progress::Progress {
                ..Default::default()
            },
        }
    }
}

impl Download {
    pub fn get(self) {
        let content_length_resp: Option<u64> = self.network.get_content_length(&self.url);

        match content_length_resp {
            Some(content_length) => {
                let children = Download::spawn_threads(self, content_length);
                for child in children {
                    let _ = child.join();
                }
            }
            None => println!("Content length is not present for this URL. Support for this type of hosted file will be added in the future."),
        }

    }

    fn calculate_ranges(
        threads: u64, 
        content_length: u64, 
        max_buffer_size: u64,
        mut progress: progress::Progress
    ) -> (progress::Progress, Vec<(String, u64, u64, u64, u64)>) {
        let mut range_start = 0;
        let mut ranges = vec![];
        let chunk_size = content_length / threads - 1;
    
        println!("Processing ranges: ");
        for thread in 0..threads {
            let mut range_end = chunk_size + range_start;
            if thread == (threads - 1) {
                range_end = content_length
            }

            let thread_number = thread + 1;
            let range: String = format!("bytes={}-{}", range_start, range_end);
            let range_to_process: u64 = range_end - range_start;
            let buffer_chunks: u64 = range_to_process / max_buffer_size;
            let chunk_remainder: u64 = range_to_process % max_buffer_size;
    
            println!("   Thread: {}, range: {}, chunks: {}, chunk_remainder: {}", thread_number, range, buffer_chunks, chunk_remainder);
            ranges.push((range, range_start, thread_number, buffer_chunks, chunk_remainder));
            progress.add(range_to_process, &thread_number);
    
            range_start = range_start + chunk_size + 1;
        }
        return (progress, ranges);
    }

    fn spawn_threads(self, content_length: u64) -> Vec<std::thread::JoinHandle<()>> {
        let mut children = vec![];
        let max_buffer_size = ONE_KB * self.memory;
        let (progress, ranges) = Download::calculate_ranges(self.threads, content_length, max_buffer_size, self.progress);

        let progress_arc = Arc::new(progress);
        let network_arc = Arc::new(self.network);

        println!("Spawning Threads...");
        for (range, range_start, thread_number, buffer_chunks, chunk_remainder) in ranges {

            let network_ref = network_arc.clone();
            let progress_ref = progress_arc.clone();
            let filename_ref = self.filename.clone();
            let url_ref = self.url.clone();

            children.push(
                thread::spawn(move || {

                    let mut file_handle = File::create(filename_ref).unwrap();
                    file_handle.seek(SeekFrom::Start(range_start)).unwrap();

                    let mut file_range_resp = network_ref.make_request(&url_ref, range);

                    for _x in 0..buffer_chunks {
                        let mut vector_buffer = vec![0u8; max_buffer_size as usize];
                        let file_range_ref = file_range_resp.by_ref();
                        file_range_ref.read_exact(&mut vector_buffer).unwrap();
                        file_handle.write(&vector_buffer).unwrap();
                        file_handle.flush().unwrap();
                        progress_ref.inc(max_buffer_size, &thread_number);
                    }

                    if chunk_remainder != 0 {
                        file_range_resp.copy_to(&mut file_handle).unwrap();
                        progress_ref.set_position(chunk_remainder, &thread_number);
                    }
                    progress_ref.finish(&thread_number);
                })
            );
        }
        progress_arc.clone().join_and_clear();
        return children;
    }
}
