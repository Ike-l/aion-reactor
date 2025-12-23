use std::{pin::Pin, sync::Arc};

use crate::{ecs::{entity::EntityId, world::World}, id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}, state_machine::{StateMachine, kernel_systems::{KernelSystem, event_manager::{event::Event, prelude::CurrentEvents}, executable_manager::{components::{ExecutableDataComponent, ExecutableLabelComponent}, executable::Executable, executable_buffer::{BufferedExecutable, ExecutableBuffer}, executable_message::ExecutableMessage, executable_queue::{ExecutableQueue, QueuedExecutable}, executable_registry::ExecutableRegistry}}}};

pub struct ExecutableManager;

impl ExecutableManager {
    pub fn insert_executable(state_machine: &StateMachine, executable_name: &str, trigger_event: Event) -> String {
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
        todo!("Assert CurrentEvents");
        todo!("Assert World");
        assert!(memory.insert(None, None, None, ExecutableQueue::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableBuffer::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableRegistry::default()).unwrap().is_ok());
        ResourceId::Heap(HeapId::Label(Id("KernelExecutableManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, ) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);

        Box::pin(async move {
            let mut executable_queue = memory.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            let executable_registry = memory.resolve::<Shared<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();

            let mut new_executable_queue = ExecutableQueue::default();
            
            for queued_executable in executable_queue.drain() {
                // give mapping and executable registry (fn on registry)

                // skip 
                let (executable, remaining) = executable_registry.parse_mapping(&queued_executable.label);

                let executable = match executable {
                    Ok(executable) => executable,
                    Err(err) => { 
                        if err != executable_registry.get_skip() { 
                            println!("Warn: {err}. Skipping"); 
                        } 

                        break; 
                    },
                };

                let event = executable.trigger;
                current_events.insert(event);

                let new_message = match queued_executable.message {
                    // if resource id, supply the resource_id of both the origin/source 
                    // (so requires the resource to downcast to the same type 
                    // (since i cant create a new resource and the user cant replace the resource))
                    // maybe later can accept an event which will map the new_message to a different resource_id than the one from before, 
                    // in those cases, the latter data component will be different
                    ExecutableMessage::ResourceId(resource_id) => {
                        let mut buffer = memory.resolve::<Unique<ExecutableBuffer>>(None, None, None, None).unwrap().unwrap();
                        let new_message = ExecutableMessage::ResourceId(resource_id);
                        
                        buffer.push(BufferedExecutable::new(
                            ExecutableLabelComponent::new(executable.label),
                            ExecutableDataComponent::new(new_message.clone()),
                            ExecutableDataComponent::new(new_message.clone())
                        ));

                        new_message
                    },
                    ExecutableMessage::ECS(entity_id) => {
                        let mut buffer = memory.resolve::<Unique<World>>(None, None, None, None).unwrap().unwrap();
                           
                        ExecutableMessage::ECS(
                            EntityId::new_hecs(
                                buffer.get_mut_hecs().unwrap().spawn((
                                    ExecutableLabelComponent::new(executable.label), 
                                    ExecutableDataComponent::new(ExecutableMessage::ECS(entity_id))
                                ))
                            )
                        )
                    },
                };

                if let Some(remaining) = remaining {
                    new_executable_queue.queue(QueuedExecutable::new(remaining.to_string(), new_message));
                }
            }

            executable_queue.extend(new_executable_queue.drain());
        })
    }
}