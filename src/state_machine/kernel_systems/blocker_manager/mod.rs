use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::unique::Unique, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}, state_machine::{kernel_systems::{blocker_manager::blocker::{CurrentBlockers, NextBlockers}, KernelSystem, StoredKernelSystem}, transition_phases::TransitionPhase}};

pub mod blocker;

pub struct BlockerManager;

impl KernelSystem for BlockerManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        memory.insert(None, None, NextBlockers::default());
        memory.insert(None, None, CurrentBlockers::default());

        let blocker_manager_id = ResourceId::Heap(HeapId::Label(Id("KernelBlockerManager".to_string())));
        memory.insert(None, Some(blocker_manager_id.clone()), Box::new(Self) as StoredKernelSystem);
        blocker_manager_id
    }

    fn tick(&mut self, memory: &Arc<Memory>, _phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_blockers = memory.resolve::<Unique<NextBlockers>>(None, None, None).unwrap().unwrap();
            let mut current_blockers = memory.resolve::<Unique<CurrentBlockers>>(None, None, None).unwrap().unwrap();

            current_blockers.tick(&mut next_blockers);
        })
    }
}