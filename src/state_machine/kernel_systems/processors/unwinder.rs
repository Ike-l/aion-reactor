use std::sync::mpsc::Sender;

pub struct Unwinder {
    results: Sender<bool>
}

impl Unwinder {
    pub fn new(results: Sender<bool>) -> Self {
        Self { results }
    }
}

impl Drop for Unwinder {
    fn drop(&mut self) {
        let _ = self.results.send(std::thread::panicking());
    }
}