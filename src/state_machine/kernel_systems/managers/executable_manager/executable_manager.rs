use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{BufferedExecutable, CurrentEvents, EntityId, EventId, Executable, ExecutableBuffer, ExecutableLabel, ExecutableMessage, ExecutableQueue, ExecutableRegistry, KernelSystem, Memory, ParseResult, ProgramId, ProgramKey, QueuedExecutable, Shared, StateMachine, SystemId, Unique, World};

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
        // matches!(memory.contains_resource(None, &ResourceId::raw_heap::<World>(), None), Some(true));
        event!(Level::DEBUG, status="Initialising", kernel_system_id = ?self.system_id());
        
        assert!(memory.insert(None, None, None, ExecutableQueue::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableBuffer::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableRegistry::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, status="Initialised", kernel_system_id = ?self.system_id());
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);

        Box::pin(async move {
            let mut executable_queue = memory.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            let executable_registry = memory.resolve::<Shared<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();

            let mut new_executable_queue = ExecutableQueue::default();
            
            event!(Level::DEBUG, executable_queue_count = executable_queue.len());
            event!(Level::DEBUG, some_executables_queued = ?executable_queue.get_range(0..5).collect::<Vec<_>>());

            for queued_executable in executable_queue.drain() {
                let (executable, remaining) = executable_registry.parse_mapping(&queued_executable.label);

                match &executable {
                    Err(ParseResult::NotFound(key)) => event!(Level::WARN, key=key, "Executable Not Found (Skipping)"),
                    _ => ()
                };

                let new_source_message = if let Ok(executable) = executable {
                    let event = executable.trigger;
                    event!(Level::TRACE, event=?event, "New Event");
                    
                    // NextEvent ? (if put executable before EventManager)
                    current_events.insert(event);

                    let label = ExecutableLabel::new(executable.label);

                    let target_message = match &queued_executable.message {
                        ExecutableMessage::ResourceId(source_id) => {
                            // TODO use an event to change the resource id
                            let target_id = source_id.clone();
                            ExecutableMessage::ResourceId(target_id)
                        },
                        ExecutableMessage::ECS(_) => {
                            let mut world = memory.resolve::<Unique<World>>(None, None, None, None).unwrap().unwrap();
                            let world = world.get_mut_hecs().expect("hecs::World in World");

                            let target_id = EntityId::new_hecs(world.reserve_entity());
                            ExecutableMessage::ECS(target_id)
                        },
                    };  

                    let source = queued_executable.message;
                    let target = target_message.clone();

                    let mut buffer = memory.resolve::<Unique<ExecutableBuffer>>(None, None, None, None).unwrap().unwrap();
                    let buffered_executable = BufferedExecutable::new(label, source, target);

                    event!(Level::TRACE, buffered_executable=?buffered_executable, "New Buffered Executable");

                    buffer.push(buffered_executable);

                    target_message
                } else {
                    queued_executable.message
                };
                
                if let Some(remaining) = remaining {
                    event!(Level::TRACE, remaining=remaining, new_source_message=?new_source_message, "Queuing Executable");
                    new_executable_queue.queue(QueuedExecutable::new(remaining.to_string(), new_source_message));
                }
            }

            executable_queue.extend(new_executable_queue.drain());
        })
    }
}