use std::{ time::Duration };

pub struct Crawler {
    delay: Duration,
    concurrent_number: usize,
    concurrent_process: usize,
}

impl Crawler {
    pub fn new(delay: Duration, concurrent_number: usize, concurrent_process: usize) -> Self {
        Crawler {
            delay,
            concurrent_number,
            concurrent_process,
        }
    }

    pub fn crawler(&self, url: String) {}
}
