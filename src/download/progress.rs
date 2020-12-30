extern crate indicatif;

use self::indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use std::collections::HashMap;

pub struct Progress {
    pub multi_progress: MultiProgress,
    pub progress_bars: HashMap<u64, ProgressBar>,
}

impl Default for Progress {
    fn default() -> Progress {
        Progress {
            multi_progress: MultiProgress::new(),
            progress_bars: HashMap::new(),
        }
    }
}

impl Progress {
    pub fn add(&mut self, range: u64, thread_number: &u64) {
        let pb = self.multi_progress.add(ProgressBar::new(range));
        let style: ProgressStyle = ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta} remaining)  {msg}")
            .progress_chars("##-");
        pb.set_style(style);
        pb.set_message(&format!("thread #{}", &thread_number));
        self.progress_bars.insert(thread_number.to_owned(), pb);
    }

    pub fn inc(&self, amount: u64, thread_number: &u64) {
        let pb = match self.progress_bars.get(thread_number) {
            Some(x) => x,
            None => return,
        };
        pb.inc(amount);
    }

    pub fn set_position(&self, amount: u64, thread_number: &u64) {
        let pb = match self.progress_bars.get(thread_number) {
            Some(x) => x,
            None => return,
        };
        pb.set_position(amount);
    }

    pub fn finish(&self, thread_number: &u64) {
        let pb = match self.progress_bars.get(thread_number) {
            Some(x) => x,
            None => return,
        };
        pb.finish_with_message("--done--");
    }

    pub fn join_and_clear(&self) {
        self.multi_progress.join_and_clear().unwrap();
    }
}



