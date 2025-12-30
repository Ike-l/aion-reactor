use std::{pin::Pin, sync::Arc};

use crate::prelude::{CurrentBlockers, KernelSystem, Memory, NextBlockers, ProgramId, ProgramKey, SystemId, Unique};

use tracing::{event, Level};

pub struct BlockerManager;

impl KernelSystem for BlockerManager {
    fn system_id(&self) -> SystemId {
        SystemId::from("Blocker Manager")
    }

    fn init(&mut self, memory: &Memory, _kernel_program_id: &ProgramId, _kernel_program_key: &ProgramKey) {
        event!(Level::DEBUG, "Inserting NextBlockers");
        assert!(memory.insert(None, None, None, NextBlockers::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Inserting CurrentBlockers");
        assert!(memory.insert(None, None, None, CurrentBlockers::default()).unwrap().is_ok());
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_blockers = memory.resolve::<Unique<NextBlockers>>(None, None, None, None).unwrap().unwrap();
            let mut current_blockers = memory.resolve::<Unique<CurrentBlockers>>(None, None, None, None).unwrap().unwrap();

            event!(Level::DEBUG, old_current_blockers_count = current_blockers.len());

            current_blockers.tick(next_blockers.drain());
            event!(Level::DEBUG, new_current_blockers_count = current_blockers.len());
            event!(Level::TRACE, current_blockers = ?current_blockers);
        })
    }
}
