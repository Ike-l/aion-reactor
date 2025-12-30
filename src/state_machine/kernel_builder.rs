use std::sync::Arc;

use tracing::{Level, event, span};

use crate::prelude::{BlockerManager, BlockingProcessor, DelayManager, EventManager, ExecutableManager, FinishNonBlockingProcessor, KernelSystem, ReadOnlyProcessor, StartNonBlockingProcessor, StateMachine, StoredKernelSystem};

fn load_default(kernel_builder: KernelBuilder) -> KernelBuilder {
    // FinishNonBlockingProcessor: 0. Join handles asap

    // Managers?

    // BlockingProcessor: 5. Main Systems
    // ReadOnlyProcessor: 6. Observers over BlockingProcessor
    // StartNonBlockingProcessor: 7. Kick off background tasks

    // Finish is before Start because if you imagine the lifetime of a "tick"
    // the time before and after are undetermined and therefore could be treated as long enough
    // for the body of a background task to complete
    // especially since the time "gained" from having the Start at the start of a tick is "known" to be in ms
    // there is no practical advantage
    // it also means accesses wont be blocking i.e for BlockingProcessor

    kernel_builder
        .with_system(FinishNonBlockingProcessor)
        .with_system(ExecutableManager)
        .with_system(DelayManager)
        .with_system(BlockerManager)
        .with_system(EventManager)
        .with_system(BlockingProcessor)
        .with_system(ReadOnlyProcessor)
        .with_system(StartNonBlockingProcessor)
}

pub struct KernelBuilder {
    threads: usize,
    kernel_systems: Vec<(StoredKernelSystem, bool)>,
}

impl KernelBuilder {
    fn empty() -> Self {
        Self {
            threads: 0,
            kernel_systems: Vec::new()
        }
    }

    fn with_system<K: KernelSystem + 'static>(mut self, kernel_system: K) -> Self {
        self.kernel_systems.push((Box::new(kernel_system) as StoredKernelSystem, true));
        self
    }

    fn with_all(self) -> Self {
        load_default(self)
    }
    
    pub fn full(threads: usize) -> Self {
        Self::empty()
            .with_threads(threads)
            .with_all()
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    pub fn systems_count(&self) -> usize {
        self.kernel_systems.len()
    }
    
    pub fn toggle(mut self, ordering_index: usize, enable: bool) -> Self {
        if let Some((_, toggle)) = self.kernel_systems.get_mut(ordering_index) {
            *toggle = enable
        }

        self
    }

    pub fn init(self, state_machine: &StateMachine) {
        let span = span!(Level::DEBUG, "Loading Kernel Systems");
        let _enter = span.enter();

        event!(Level::DEBUG, "Started");
        
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let threadpool = threadpool::ThreadPool::new(self.threads);

        assert!(state_machine.memory.insert(
            Some(&state_machine.program_id), 
            None, 
            Some(&state_machine.kernel_key), 
            Arc::new(rt)
        ).unwrap().is_ok());

        assert!(state_machine.memory.insert(
            Some(&state_machine.program_id), 
            None, 
            Some(&state_machine.kernel_key), 
            threadpool
        ).unwrap().is_ok());

        for (ordering_index, (mut kernel_system, _)) 
            in self.kernel_systems
                .into_iter()
                .enumerate()
                .filter(|(_, (_, toggled))| *toggled) 
        {
            let system_id = kernel_system.system_id();
            
            let kernel_system_span = span!(Level::DEBUG, "Kernel System Init", kernel_system_id=?system_id);
            let _enter = kernel_system_span.enter();

            event!(Level::DEBUG, "Started");
            kernel_system.init(&state_machine.memory, &state_machine.program_id, &state_machine.kernel_key);
            event!(Level::DEBUG, "Finished");

            state_machine.insert_stored_system(system_id, kernel_system, ordering_index);
        }

        event!(Level::DEBUG, "Finished")
    }
}