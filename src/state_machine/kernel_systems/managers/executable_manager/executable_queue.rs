// "Foo|FooBarAdapter|Bar|BarBazAdapter|Baz", FooInput
// "FooBarAdapter|Bar|BarBazAdapter|Baz", FooOutput
// "Bar|BarBazAdapter|Baz", BarInput 
// "BarBazAdapter|Baz", BarOutput 
// "Baz", BazInput
// Complete

use std::ops::Range;

use tracing::{Level, event};

use crate::{memory::Memory, prelude::{BufferedExecutable, EntityId, ExecutableBuffer, ExecutableLabel, ExecutableMessage, ExecutableRegistry, NextEvents, ParseResult, Unique, World}};

#[derive(Debug)]
pub struct QueuedExecutable {
    pub label: String,
    pub message: ExecutableMessage
}

impl QueuedExecutable {
    pub fn new(label: String, message: ExecutableMessage) -> Self {
        Self { label, message }
    }
}

#[derive(Default, Debug)]
pub struct ExecutableQueue(Vec<QueuedExecutable>);

impl ExecutableQueue {
    pub fn queue(&mut self, executable: QueuedExecutable) {
        self.0.push(executable);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = QueuedExecutable> {
        self.0.drain(..)
    }

    pub fn extend<T>(&mut self, iter: T) 
        where T: IntoIterator<Item = QueuedExecutable> 
    {
        self.0.extend(iter);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// no guarantees about the ordering
    pub fn get_range(&self, amount: Range<usize>) -> impl Iterator<Item = &QueuedExecutable> {
        self.0.iter().take(amount.end)
    }

    pub fn tick(
        &mut self,
        memory: &Memory,
        executable_registry: &ExecutableRegistry,
        next_events: &mut NextEvents,
    ) {
        let mut new_executable_queue = ExecutableQueue::default();
        for queued_executable in self.drain() {
            let (executable, remaining) = executable_registry.parse_mapping(&queued_executable.label);

            match &executable {
                Err(ParseResult::NotFound(key)) => event!(Level::WARN, key=key, "Executable Not Found (Skipping)"),
                _ => ()
            };

            let new_source_message = if let Ok(executable) = executable {
                let event = executable.trigger;
                event!(Level::TRACE, event=?event, "New Event");
                
                // NextEvent ? (if put executable before EventManager)
                next_events.insert(event);

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

        self.extend(new_executable_queue.drain());
    }
}
