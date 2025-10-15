use crate::memory::Memory;

pub struct Blacklist {
    blocks: Vec<Box<dyn Fn(&Memory) + Send>>,
    unblocks: Vec<Box<dyn Fn(&Memory) + Send>>
}

impl Blacklist {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            unblocks: Vec::new()
        }
    }

    pub fn insert_block<T, S>(&mut self, block: T, unblock: S) 
        where 
            T: Fn(&Memory) + Send + 'static, 
            S: Fn(&Memory) + Send + 'static
    {
        self.blocks.push(Box::new(block));
        self.unblocks.push(Box::new(unblock))
    }

    pub fn block(&self, memory: &Memory) {
        self.blocks.iter().for_each(|b| b(memory));
    }

    pub fn unblock(&self, memory: &Memory) {
        self.unblocks.iter().for_each(|ub| ub(memory));
    }
}