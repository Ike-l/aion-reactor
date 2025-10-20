pub mod background_processor_system_registry;
pub mod start_background_processor;
pub mod finish_background_processor;

// pub struct BackgroundProcessor;

// impl BackgroundProcessor {
//     pub fn new() -> Self {
//         Self
//     }

//     pub fn insert_system(state_machine: &StateMachine, id: Id, system_metadata: SystemMetadata, system: StoredSystem) -> Option<SystemMetadata> {
//         let mut system_registry = state_machine.state.quick_resolve::<Unique<BackgroundProcessorSystemRegistry>>();
//         Processor::insert_system(state_machine, &mut system_registry.0, id, system_metadata, system)
//     }
// }

// // Split into 2: Start/Finish. Finish is before Processor, Start is after Processor

// impl KernelSystem for BackgroundProcessor {
//     fn tick(&mut self, memory: &Arc<Memory>, _phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
//         // let memory = Arc::clone(&memory);
//         // Box::pin(async move {
//         //     let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None).unwrap().unwrap();
//         //     let systems = Processor::get_systems(&memory, &system_registry.0);
//         //     // Start threads
//         //     // Collect threads

//         // })
//         todo!()
//     }
// }