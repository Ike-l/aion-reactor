pub mod state_machine;
pub mod ids;
pub mod memory;
pub mod injection;
pub mod ecs;

// pub mod kernel_prelude {
//     pub use std::{pin::Pin, sync::Arc};

//     pub use crate::{memory::{Memory, ResourceId}, state_machine::kernel_systems::KernelSystem};
// }
#[allow(unused_imports)]
pub(crate) mod prelude {
    pub use super::{
        ecs::{
            entity::EntityId, world::World
        },
        ids::{
            Id, system_id::SystemId, program_id::ProgramId, event_id::EventId
        },
        injection::{
            AccessDropper, DeAccessResolver, 
            injection_advanced::{
                global::Global, resulting::Resulting
            },
            injection_primitives::{
                cloned::Cloned, shared::Shared, unique::Unique
            },
            injection_trait::{
                Injection, MemoryTarget
            }
        },
        memory::{
            Memory, 
            access_checked_heap::{
                AccessCheckedHeap,
                heap::{
                    HeapId, HeapObject, heap::Heap, inner_heap::InnerHeap, raw_heap::RawHeap, raw_heap_object::RawHeapObject
                },
                raw_access_map::RawAccessMap, reservation_access_map::ReservationAccessMap, reserve_access_map::ReserveAccessMap
            },
            access_map::{
                Access, AccessMap
            },
            errors::{
                DeResolveError, InsertError, ReservationError, ResolveError
            },
            memory_domain::MemoryDomain,
            program_memory_map::{
                ProgramMemoryMap,
                inner_program_memory_map::{
                    InnerProgramMemoryMap, ProgramKey
                },
                raw_program_memory_map::RawProgramMemoryMap
            },
            resource_id::{
                Resource, ResourceId
            },
        },
        state_machine::{
            StateMachine, kernel_registry::KernelSystemRegistry,
            kernel_systems::{
                KernelSystem, StoredKernelSystem, 
                managers::{
                    blocker_manager::{
                        blocker::Blocker, blocker_manager::BlockerManager, current_blockers::CurrentBlockers, next_blockers::NextBlockers,
                    },
                    delay_manager::{
                        delay::{
                            Delay, registered_delay::RegisteredDelay
                        },
                        delay_buffer::DelayBuffer, delay_manager::DelayManager, delay_registry::DelayRegistry
                    },
                    event_manager::{
                        event_manager::EventManager, event_mapper::EventMapper, next_events::NextEvents, current_events::CurrentEvents
                    },
                    executable_manager::{
                        executable_buffer::{
                            BufferedExecutable, ExecutableBuffer
                        },
                        executable_label::ExecutableLabel,
                        executable_manager::ExecutableManager,
                        executable_message::ExecutableMessage,
                        executable_queue::{
                            QueuedExecutable, ExecutableQueue
                        },
                        executable::Executable,
                        executable_registry::{
                            ExecutableRegistry, ParseResult
                        }
                    },
                },
                processors::{
                    system_event_registry::SystemEventRegistry,
                    background_processor::{
                        processors::{
                            finish_background_processor::FinishBackgroundProcessor,
                            start_background_processor::StartBackgroundProcessor
                        },
                        join_handles::{
                            async_join_handles::AsyncJoinHandles,
                            sync_join_handles::SyncJoinHandles,
                        },
                        background_processor_system_registry::BackgroundProcessorSystemRegistry
                    },
                    system::{
                        FunctionSystem, System, system_cell::SystemCell, system_status::SystemStatus,
                        async_system::{
                            AsyncSystem, StoredAsyncSystem, into_async_system::IntoAsyncSystem
                        },
                        stored_system::{
                            StoredSystem, StoredSystemError
                        },
                        sync_system::{
                            StoredSyncSystem, SyncSystem, into_sync_system::IntoSyncSystem   
                        },
                        system_metadata::{
                            SystemMetadata, SystemRegistry, criteria::Criteria
                        },
                        system_result::{
                            SystemEvent, SystemResult
                        }
                    },
                    processor::{
                        Processor, Trace, Unwinder, processor_system_registry::ProcessorSystemRegistry, tasks::DummyWaker,
                        scheduler::{
                            execution_graph::ExecutionGraph, node::Node, 
                            ordering::{
                                ExecutionOrdering, SchedulerOrdering
                            }
                        }
                    },
                    read_only_processor::{
                        ReadOnlyInjection, ReadOnlyProcessor, ReadOnlySystem, ReadOnlySystemRegistry
                    }
                }
            }
        }
    };
}

// #[cfg(test)]
// mod tests {
//     use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}};

//     trait SyncSystem {
//         fn r(&mut self, _memory: &Memory) { println!("Success 1") }

//         fn s(&self, _memory: &Memory) { println!("Success 2") }
//     }
    
//     struct Bar;
    
//     impl SyncSystem for Bar {}
    
//     #[test]
//     fn foo() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
//         let mut f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.r(&memory);
//     }

//     #[test]
//     fn bar() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
//         let mut f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.r(&memory);
//         drop(f);
//         let mut f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.r(&memory);
//     }

//     #[test]
//     fn baz() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
//         let f = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.s(&memory);
//         let fa = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         fa.s(&memory);
//     }

//     #[test]
//     #[should_panic]
//     fn fizz() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));

//         let b = memory.reserve_accesses::<Unique<Box<dyn SyncSystem>>>(None, Some(heap_label.clone()), ResourceId::Heap(HeapId::Label(Id("yuurp".to_string()))));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);

//         let f = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.s(&memory);

//         let fa = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         fa.s(&memory);
//     }
// }

