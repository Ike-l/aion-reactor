use crate::prelude::ResourceId;

#[derive(Default)]
pub struct KernelSystemRegistry {
    graph: Vec<Vec<ResourceId>>
}

impl KernelSystemRegistry {
    pub fn iter(&mut self) -> impl Iterator<Item = &Vec<ResourceId>> {
        self.graph.iter()
    }

    pub fn insert(&mut self, index: usize, resource_id: ResourceId) {
        while index + 1 > self.graph.len() {
            self.graph.push(vec![]);
        }

        self.graph[index].push(resource_id);
    }
}