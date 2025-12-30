use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{EventId, Executable, ExecutableBuffer, ExecutableMessage, ExecutableQueue, ExecutableRegistry, KernelSystem, Memory, NextEvents, ProgramId, ProgramKey, QueuedExecutable, ResourceId, Shared, StateMachine, SystemId, Unique, World};

pub struct ExecutableManager;

impl ExecutableManager {
    pub fn insert_executable(state_machine: &StateMachine, executable_name: &str, trigger_event: EventId) -> String {
        let mut executable_registry = state_machine.resolve::<Unique<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();
        let executable_label = format!("{executable_name}-Executable");
        executable_registry.insert(executable_label.clone(), Executable::new(executable_label.clone(), trigger_event));
        executable_label
    }

    pub fn queue_executable(state_machine: &StateMachine, executable_label: String, executable_message: ExecutableMessage) {
        let mut executable_queue = state_machine.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
        executable_queue.queue(QueuedExecutable::new(executable_label, executable_message));
    }
}

impl KernelSystem for ExecutableManager {
    fn system_id(&self) -> SystemId {
        SystemId::from("Executable Manager")
    }

    fn init(&mut self, memory: &Memory, _kernel_program_id: &ProgramId, _kernel_program_key: &ProgramKey) {
        event!(Level::DEBUG, "Inserting ExecutableQueue");
        assert!(memory.insert(None, None, None, ExecutableQueue::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Inserting ExecutableBuffer");
        assert!(memory.insert(None, None, None, ExecutableBuffer::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Inserting ExecutableRegistry");
        assert!(memory.insert(None, None, None, ExecutableRegistry::default()).unwrap().is_ok());

        event!(Level::DEBUG, "Checking NextEvents");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextEvents>(), None), Some(true)) {
            event!(Level::WARN, "NextEvents Not Found");   
        }

        event!(Level::DEBUG, "Checking World");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<World>(), None), Some(true)) {
            // Only warn because path may never trigger a panic
            // (also the reason the checks are after inserts is because they may be recoverable in the future)
            event!(Level::WARN, "World Not Found");   
        }
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);

        Box::pin(async move {
            let mut executable_queue = memory.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let executable_registry = memory.resolve::<Shared<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();

            event!(Level::DEBUG, old_executable_queue_count = executable_queue.len());
            event!(Level::DEBUG, old_next_event_count = next_events.len());

            executable_queue.tick(&memory, &executable_registry, &mut next_events);
            event!(Level::DEBUG, new_executable_queue_count = executable_queue.len());
            event!(Level::TRACE, executables_queued = ?executable_queue);

            event!(Level::DEBUG, new_current_event_count = next_events.len());
            event!(Level::TRACE, next_events = ?next_events);
        })
    }
}