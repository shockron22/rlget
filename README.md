# RLget 

![License](https://img.shields.io/crates/l/cloak.svg)

## Rust Parallel Download client
  A download client that supports parallel downloading and custom memory settings.


## Features 

   - **You** are in control of your resource usage and can balance that against threads/speed of download.
   - Writes file to disk **only** once in parallel. Each thread only writes its part of the file.
   - Custom memory chunking settings. You control how much memory is used per thread.

## Building

  In project root run "cargo build --release"
  Then add ./target/release/rlget to your /usr/local/bin or other path.

## Usage
  <img src="https://shockron22.github.io/rlgetv0.0.1.svg">

## Speed test rlget VS wget

  time rlget -t 50 http://il.us.mirror.archlinux-br.org/iso/2019.09.01/archlinux-2019.09.01-x86_64.iso 

  rlget -t 50  5.02s user 7.56s system 32% cpu 38.617 total

  time wget http://il.us.mirror.archlinux-br.org/iso/2019.09.01/archlinux-2019.09.01-x86_64.iso 

  wget   0.64s user 5.80s system 7% cpu 1:22.51 total

  As you can see rlget can download files faster than wget at a higher CPU cost.

  Where rlget really shines is in downloading files that are rate limited per request.
  Your download speed increases exponentially in relation to your thread settings. 

  You can also get close to wget memory and cpu usage depending on your thread/memory settings.
  Most of the resource usage is because of the bloat in the Reqwest library.


## - Current TODO 
  - refactor downloading/thread spawning to be better organized.
  - add better error handling 
  - add unit tests
  - replace Reqwest library with something less bloated
  - Fix some progress bar rendering issues when you have many threads