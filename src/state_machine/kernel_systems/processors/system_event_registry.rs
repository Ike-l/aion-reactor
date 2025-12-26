use std::collections::HashMap;

use crate::{ids::system_id::SystemId, prelude::EventId};

#[derive(Default)]
pub struct SystemEventRegistry(HashMap<SystemId, Vec<EventId>>);

impl SystemEventRegistry {
    pub fn get(&self, system_id: &SystemId) -> Option<&Vec<EventId>> {
        self.0.get(system_id)
    }
}