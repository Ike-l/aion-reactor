use std::collections::HashMap;

use crate::memory::ResourceId;

#[derive(Default)]
pub struct KernelSystemRegistry {
    graph: HashMap<usize, Vec<ResourceId>>
}

impl KernelSystemRegistry {
    pub fn iter(&mut self) -> impl Iterator<Item = &Vec<ResourceId>> {
        self.graph.values()
    }

    pub fn insert(&mut self, index: usize, resource_id: ResourceId) {
        self.graph.entry(index).or_default().push(resource_id);
    }
}