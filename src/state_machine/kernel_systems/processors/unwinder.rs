use std::sync::mpsc::Sender;

pub struct Unwinder {
    results: Sender<(bool, usize)>,
    current_thread: usize
}

impl Unwinder {
    pub fn new(results: Sender<(bool, usize)>, current_thread: usize) -> Self {
        Self { results, current_thread }
    }
}

impl Drop for Unwinder {
    fn drop(&mut self) {
        let _ = self.results.send((std::thread::panicking(), self.current_thread));
    }
}