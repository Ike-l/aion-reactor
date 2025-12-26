use std::{pin::Pin, sync::Arc};

use crate::prelude::{CurrentEvents, DelayBuffer, DelayRegistry, KernelSystem, Memory, ResourceId, Shared, Unique};

pub struct DelayManager;

impl KernelSystem for DelayManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<CurrentEvents>(), None), Some(true));

        assert!(memory.insert(None, None, None, DelayRegistry::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, DelayBuffer::default()).unwrap().is_ok());

        ResourceId::from_labelled_heap("KernelDelayManager")
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut buffer = memory.resolve::<Unique<DelayBuffer>>(None, None, None, None).unwrap().unwrap();
            let registry = memory.resolve::<Shared<DelayRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            
            buffer.tick(&registry, &mut current_events);
        })
    }
}