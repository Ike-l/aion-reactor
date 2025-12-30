use std::collections::HashMap;

use crate::Memory;

pub struct Host<M: Memory> {
    reservation_map: HashMap<M::Owner, M::AccessMap>,
}

impl<M: Memory> Default for Host<M> {
    fn default() -> Self {
        Self {
            reservation_map: HashMap::new()
        }
    }
}