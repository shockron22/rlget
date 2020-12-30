extern crate docopt;
extern crate serde;

use docopt::Docopt;
use serde::Deserialize;

mod download;

const USAGE: &'static str = "
RLget

Usage:
  rlget [options] <url>
  rlget (-h | --help)

Options:
    -t THREADS, --threads=THREADS     Number of download threads. (default: 1)
    -m MEMORY, --memory=MEMORY        The amount of memory for each thread to chunk request by in KB (default: 256kb)
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

    let (url, threads, filename, memory) = parse_args(args);

    println!("threads: {}", threads);
    println!("url: {}", url);
    println!("memory: {}", memory);
    println!("filename: {}\n", filename);

    let download = download::Download { 
        threads: threads,
        url: url,
        memory: memory,
        filename: filename, 
        ..Default::default() 
    };

    download.get();

    Ok(())
}

fn parse_args(args: Args) -> (std::string::String, u64, std::string::String, u64){
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
    return (url, threads, filename, memory);
}