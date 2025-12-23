use crate::state_machine::kernel_systems::executable_manager::executable_message::ExecutableMessage;

#[derive(Debug)]
pub struct ExecutableLabelComponent(String);

impl ExecutableLabelComponent {
    pub fn new(executable_label: String) -> Self {
        Self(executable_label)
    }
}

pub struct ExecutableDataComponent(ExecutableMessage);

impl ExecutableDataComponent {
    pub fn new(executable_message: ExecutableMessage) -> Self {
        Self(executable_message)
    }
}