use std::collections::HashMap;

use crate::Memory;

pub struct InnerSegmentMap<M: Memory> {
    segments: HashMap<M::ResourceId, M::Resource>
}

impl<M: Memory> Default for InnerSegmentMap<M> {
    fn default() -> Self {
        Self {
            segments: HashMap::new()
        }
    }
}