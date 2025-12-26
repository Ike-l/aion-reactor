use std::{pin::Pin, sync::Arc};

use crate::prelude::{BufferedExecutable, CurrentEvents, EntityId, EventId, Executable, ExecutableBuffer, ExecutableLabel, ExecutableMessage, ExecutableQueue, ExecutableRegistry, KernelSystem, Memory, ParseResult, QueuedExecutable, ResourceId, Shared, StateMachine, Unique, World};

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
    fn init(&mut self, memory: &Memory) -> ResourceId {
        // matches!(memory.contains_resource(None, &ResourceId::raw_heap::<World>(), None), Some(true));

        assert!(memory.insert(None, None, None, ExecutableQueue::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableBuffer::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableRegistry::default()).unwrap().is_ok());

        ResourceId::from_labelled_heap("KernelExecutableManager")
    }

    fn tick(&mut self, memory: &Arc<Memory>, ) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);

        Box::pin(async move {
            let mut executable_queue = memory.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            let executable_registry = memory.resolve::<Shared<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();

            let mut new_executable_queue = ExecutableQueue::default();
            
            for queued_executable in executable_queue.drain() {
                let (executable, remaining) = executable_registry.parse_mapping(&queued_executable.label);

                match &executable {
                    Err(ParseResult::NotFound(key)) => println!("Warn: Executable `{key}` Not Found. Skipping"),
                    _ => ()
                };

                let new_source_message = if let Ok(executable) = executable {
                    let event = executable.trigger;
                    // NextEvent ? 
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

                    buffer.push(buffered_executable);

                    target_message
                } else {
                    queued_executable.message
                };
                
                if let Some(remaining) = remaining {
                    new_executable_queue.queue(QueuedExecutable::new(remaining.to_string(), new_source_message));
                }
            }

            executable_queue.extend(new_executable_queue.drain());
        })
    }
}