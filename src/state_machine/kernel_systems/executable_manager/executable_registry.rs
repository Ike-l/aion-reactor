use std::collections::HashMap;

use crate::state_machine::kernel_systems::executable_manager::executable::Executable;

// String is what is first mapped
pub struct ExecutableRegistry {
    skip_message: String,
    registry: HashMap<String, Executable>
}

impl Default for ExecutableRegistry {
    fn default() -> Self {
        let skip_message =  rand::random::<u64>().to_string();

        Self {
            skip_message,
            registry: HashMap::new()
        }
    }
}

impl ExecutableRegistry {
    pub fn get_skip(&self) -> &str {
        &self.skip_message
    }

    pub fn parse_mapping<'a>(&self, sequence: &'a str) -> (Result<Executable, String>, Option<&'a str>) {
        let (current, then) = if let Some((current, then)) = sequence.split_once(">") {
            (current, Some(then))
        } else {
            (sequence, None)
        };

        (self.registry.get(current).cloned().ok_or(format!("No Executable Found: {current}")), then)
    }

    pub fn insert(&mut self, label: String, executable: Executable) -> Option<Executable> {
        self.registry.insert(label, executable)
    }
}