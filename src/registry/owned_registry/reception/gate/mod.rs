use std::collections::HashMap;

use crate::registry::owned_registry::reception::gate::gate_permission::GatePermission;

pub mod gate_permission;

pub trait GateKeeper {
    type ResourceId;
    type Key;
}

pub struct Gate<G: GateKeeper> {
    keys: HashMap<G::ResourceId, G::Key>
}

impl<G: GateKeeper> Gate<G> {
    pub fn allows_passage(&self) -> GatePermission {
        todo!("return GatePermission")
    }
}

impl<G: GateKeeper> Default for Gate<G> {
    fn default() -> Self {
        Self {
            keys: HashMap::new()
        }
    }
}