use std::collections::HashMap;

pub use crate::kernel_prelude::*;
use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::access_checked_heap::heap::HeapId, state_machine::kernel_systems::event_manager::event::{CurrentEvents, Event}};

pub struct ExecutableManager;

// String is what is first mapped
pub struct ExecutableRegistry(pub HashMap<String, Executable>);

impl ExecutableRegistry {
    pub fn parse_mapping<'a>(&self, sequence: &'a str) -> (Result<Executable, String>, Option<&'a str>) {
        let (current, then) = if let Some((current, then)) = sequence.split_once(">") {
            (current, Some(then))
        } else {
            (sequence, None)
        };

        (self.0.get(current).cloned().ok_or(format!("No Executable Found: {current}")), then)
    }
}

#[derive(Clone)]
pub struct Executable {
    // used as the ExecutableLabelComponent
    label: String,
    trigger: Event,
}

#[derive(Clone)]
pub enum ExecutableMessage {
    ResourceId(ResourceId),

    #[cfg(feature = "ecs")]
    ECS(EntityId)
}

#[cfg(feature = "ecs")]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EntityId(hecs::Entity);

#[cfg(feature = "ecs")]
pub struct World(pub hecs::World);

// Label (which process handler), From resource, To resource
pub struct ExecutableBuffer(Vec<(ExecutableLabelComponent, ExecutableDataComponent, ExecutableDataComponent)>);

// "Foo|FooBarAdapter|Bar|BarBazAdapter|Baz", FooInput
// "FooBarAdapter|Bar|BarBazAdapter|Baz", FooOutput
// "Bar|BarBazAdapter|Baz", BarInput 
// "BarBazAdapter|Baz", BarOutput 
// "Baz", BazInput
// Complete
pub struct ExecutableQueue(pub Vec<(String, ExecutableMessage)>);

pub struct ExecutableLabelComponent(pub String);
pub struct ExecutableDataComponent(pub ExecutableMessage);

impl KernelSystem for ExecutableManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, ExecutableQueue(Vec::new())).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableBuffer(Vec::new())).unwrap().is_ok());
        assert!(memory.insert(None, None, None, ExecutableRegistry(HashMap::new())).unwrap().is_ok());
        // World?
        ResourceId::Heap(HeapId::Label(Id("KernelExecutableManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);

        Box::pin(async move {
            let mut executable_queue = memory.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            let executable_registry = memory.resolve::<Shared<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();

            let mut new_executable_queue = ExecutableQueue(Vec::new());
            
            for (mapping, message) in executable_queue.0.drain(..) {
                // give mapping and executable registry (fn on registry)
                let (executable, remaining) = executable_registry.parse_mapping(&mapping);

                let executable = match executable {
                    Ok(executable) => executable,
                    Err(err) => { println!("Warn: {err}. Skipping"); break; },
                };

                let event = executable.trigger;
                current_events.insert(event);

                let new_message = match message {
                    // if resource id, supply the resource_id of both the origin/source (so requires the resource to downcast to the same type (since i cant create a new resource and the user cant replace the resource))
                    // maybe later can accept an event which will map the new_message to a different resource_id than the one from before, in those cases, the latter data component will be different
                    ExecutableMessage::ResourceId(resource_id) => {
                        let mut buffer = memory.resolve::<Unique<ExecutableBuffer>>(None, None, None, None).unwrap().unwrap();
                        let new_message = ExecutableMessage::ResourceId(resource_id);
                        buffer.0.push(
                            (ExecutableLabelComponent(executable.label), ExecutableDataComponent(new_message.clone()), ExecutableDataComponent(new_message.clone()))
                        );

                        new_message
                    },
                    #[cfg(feature = "ecs")]
                    // if ecs, create a new entity with the label of the process handler, and the id of where to get the old data
                    ExecutableMessage::ECS(entity_id) => {
                        let mut buffer = memory.resolve::<Unique<World>>(None, None, None, None).unwrap().unwrap();
                        ExecutableMessage::ECS(EntityId(buffer.0.spawn(
                            (ExecutableLabelComponent(executable.label), ExecutableDataComponent(ExecutableMessage::ECS(entity_id)))
                        )))
                    },
                };

                if let Some(remaining) = remaining {
                    new_executable_queue.0.push((remaining.to_string(), new_message));
                }
            }

            executable_queue.0.extend(new_executable_queue.0);
        })
    }
}