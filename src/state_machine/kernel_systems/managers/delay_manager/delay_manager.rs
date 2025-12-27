use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{CurrentEvents, DelayBuffer, DelayRegistry, KernelSystem, Memory, ProgramId, ProgramKey, ResourceId, Shared, SystemId, Unique};

pub struct DelayManager;

impl KernelSystem for DelayManager {
    fn system_id(&self) -> SystemId {
        SystemId::from("Delay Manager")
    }

    fn init(&mut self, memory: &Memory, _kernel_program_id: &ProgramId, _kernel_program_key: &ProgramKey) {
        event!(Level::DEBUG, status="Initialising", kernel_system_id = ?self.system_id());
        
        assert!(matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<CurrentEvents>(), None), Some(true)));
        
        assert!(memory.insert(None, None, None, DelayRegistry::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, DelayBuffer::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, status="Initialised", kernel_system_id = ?self.system_id());
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut buffer = memory.resolve::<Unique<DelayBuffer>>(None, None, None, None).unwrap().unwrap();
            let registry = memory.resolve::<Shared<DelayRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            
            let old_event_count = current_events.len();

            buffer.tick(&registry, &mut current_events);

            event!(Level::DEBUG, new_events = current_events.len() - old_event_count);
            event!(Level::DEBUG, some_current_events = ?current_events.get_range(0..5).collect::<Vec<_>>());
        })
    }
}