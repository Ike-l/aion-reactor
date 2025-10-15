#[derive(Debug)]
pub enum SystemEvent {
    NoEvent,
}

#[derive(Debug)]
pub enum SystemResult {
    Event(SystemEvent),
    Error(anyhow::Error)
}