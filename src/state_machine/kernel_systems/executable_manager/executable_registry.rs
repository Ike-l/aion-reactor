use std::collections::HashMap;

use crate::state_machine::kernel_systems::executable_manager::executable::Executable;

// String is what is first mapped
pub struct ExecutableRegistry {
    delimiter: char,
    skip_message: String,
    registry: HashMap<String, Executable>
}

impl Default for ExecutableRegistry {
    fn default() -> Self {
        let skip_message =  rand::random::<u64>().to_string();

        let delimiter = '>';

        Self {
            delimiter,
            skip_message,
            registry: HashMap::new()
        }
    }
}

// #[derive(PartialEq)]
pub enum ParseResult {
    Skip,
    NotFound(String)
}

impl ExecutableRegistry {
    pub fn get_skip(&self) -> &str {
        &self.skip_message
    }

    pub fn get_delim(&self) -> &char {
        &self.delimiter
    }

    pub fn parse_mapping<'a>(&self, sequence: &'a str) -> (Result<Executable, ParseResult>, Option<&'a str>) {
        let (current, then) = if let Some((current, then)) = sequence.split_once(*self.get_delim()) {
            (current, Some(then))
        } else {
            (sequence, None)
        };

        if current == self.get_skip() {
            return (Err(ParseResult::Skip), then);
        }

        (self.registry.get(current).cloned().ok_or(ParseResult::NotFound(current.to_string())), then)
    }

    pub fn insert(&mut self, label: String, executable: Executable) -> Option<Executable> {
        self.registry.insert(label, executable)
    }
}