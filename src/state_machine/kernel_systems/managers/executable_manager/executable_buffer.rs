use crate::prelude::{ExecutableLabel, ExecutableMessage};

#[derive(Debug)]
pub struct BufferedExecutable {
    pub label: ExecutableLabel,
    pub source: ExecutableMessage,
    pub target: ExecutableMessage
}

impl BufferedExecutable {
    pub fn new(label: ExecutableLabel, source: ExecutableMessage, target: ExecutableMessage) -> Self {
        Self { label, source, target }
    }
}

#[derive(Default)]
pub struct ExecutableBuffer(Vec<BufferedExecutable>);

impl ExecutableBuffer {
    pub fn push(&mut self, buffered_executable: BufferedExecutable) {
        self.0.push(buffered_executable);
    }
}
