use crate::prelude::{ProgramId, ProgramKey, ResourceId};

// Since criteria cant be cloned
#[derive(Debug, Clone)]
pub struct StoredSystemMetadata {
    resource_id: ResourceId,
    program_id: Option<ProgramId>,
    key: Option<ProgramKey>,
}

impl StoredSystemMetadata {
    pub fn new(resource_id: ResourceId, program_id: Option<ProgramId>, key: Option<ProgramKey>) -> Self {
        Self {
            resource_id, program_id, key
        }
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn program_id(&self) -> &Option<ProgramId> {
        &self.program_id
    }

    pub fn key(&self) -> &Option<ProgramKey> {
        &self.key
    }
}
