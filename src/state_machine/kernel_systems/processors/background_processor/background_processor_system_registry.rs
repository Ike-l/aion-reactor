use crate::prelude::{SystemId, SystemMetadata, SystemRegistry};

#[derive(Default)]
pub struct BackgroundProcessorSystemRegistry(SystemRegistry);

impl BackgroundProcessorSystemRegistry {
    pub fn get(&self, system_id: &SystemId) -> Option<&SystemMetadata> {
        self.0.get(system_id)
    }

    pub fn ref_generic(&self) -> &SystemRegistry {
        &self.0
    }

    pub fn ref_mut_generic(&mut self) -> &mut SystemRegistry {
        &mut self.0
    }
}