use crate::state_machine::kernel_systems::executable_manager::components::{ExecutableDataComponent, ExecutableLabelComponent};

pub struct BufferedExecutable {
    pub label: ExecutableLabelComponent,
    pub source: ExecutableDataComponent,
    pub target: ExecutableDataComponent
}

impl BufferedExecutable {
    pub fn new(label: ExecutableLabelComponent, source: ExecutableDataComponent, target: ExecutableDataComponent) -> Self {
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
