pub mod state_machine;
pub mod ids;
pub mod memory;
pub mod injection;
pub mod ecs;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{
        ecs::{
            entity::EntityId, world::World
        },
        ids::{
            Id, system_id::SystemId, program_id::ProgramId, event_id::EventId, blocker_id::BlockerId
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
                        blocker_manager::BlockerManager, current_blockers::CurrentBlockers, next_blockers::NextBlockers,
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
                    system_event_registry::SystemEventRegistry, Processor, tasks::DummyWaker, unwinder::Unwinder,
                    non_blocking_processor::{
                        processors::{
                            finish_non_blocking_processor::FinishNonBlockingProcessor,
                            start_non_blocking_processor::StartNonBlockingProcessor
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
                            SystemMetadata, SystemRegistry, criteria::Criteria, stored_system_metadata::StoredSystemMetadata
                        },
                        system_result::{
                            SystemEvent, SystemResult
                        }
                    },
                    blocking_processor::{
                        processor::BlockingProcessor, processor_system_registry::ProcessorSystemRegistry, 
                        scheduler::{
                            execution_graph::ExecutionGraph, node::Node, 
                            ordering::{
                                ExecutionOrdering, SchedulerOrdering
                            }
                        }
                    },
                    read_only_processor::{
                        read_only_system_registry::ReadOnlySystemRegistry, processor::ReadOnlyProcessor,
                        read_only_system::{
                            ReadOnlyInjection, ReadOnlySystem
                        }
                    }
                }
            }
        }
    };
}
