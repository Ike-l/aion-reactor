#[derive(Debug)]
pub enum SystemKind {
    Sync,
    Async
}

impl Default for SystemKind {
    fn default() -> Self {
        Self::Sync
    }
}