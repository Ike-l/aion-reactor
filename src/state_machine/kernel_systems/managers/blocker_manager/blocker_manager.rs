use std::{pin::Pin, sync::Arc};

use crate::prelude::{CurrentBlockers, KernelSystem, Memory, NextBlockers, ResourceId, Unique};

pub struct BlockerManager;

impl KernelSystem for BlockerManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        memory.insert(None, None, None, NextBlockers::default());
        memory.insert(None, None, None, CurrentBlockers::default());

        ResourceId::from_labelled_heap("KernelBlockerManager")
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_blockers = memory.resolve::<Unique<NextBlockers>>(None, None, None, None).unwrap().unwrap();
            let mut current_blockers = memory.resolve::<Unique<CurrentBlockers>>(None, None, None, None).unwrap().unwrap();

            current_blockers.tick(&mut next_blockers);
        })
    }
}
