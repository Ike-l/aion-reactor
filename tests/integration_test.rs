use aion_reactor::{id::Id, injection::injection_primitives::{unique::Unique}, memory::{ResourceId, access_checked_heap::heap::HeapId}, state_machine::{StateMachine, kernel_systems::{processor::{processor_system_registry::ProcessorSystemRegistry, scheduler::ordering::SchedulerOrdering}}}, system::{System, stored_system::StoredSystem, sync_system::{into_sync_system::IntoSyncSystem}, system_metadata::{SystemMetadata, criteria::Criteria}, system_result::SystemResult}};

fn foo(mut bar: Unique<i32>) -> Option<SystemResult> {
    println!("Foo: {}", bar);
    **bar += 1;
    None
    // std::thread::sleep(Duration::from_secs(1));
    // panic!("yurp")
}

#[test]
fn sync_system() {
    let state_machine = StateMachine::new(1);
    state_machine.load_default(16);

    {
        let program_id = None;
        let program_key = None;
    
        let system_id = Id("FooSystem".to_string());
        let resource_id = ResourceId::Heap(HeapId::Label(system_id.clone()));
    
        let mut system_registry = state_machine.resolve::<Unique<ProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
    
        let system_metadata = SystemMetadata::new(
            resource_id.clone(), 
            program_id.clone(), 
            program_key, 
            Criteria::new(|_events| -> bool {
                // events.contains(&Event::from(Id("StartFoo".to_string())))
                true
            }), 
            SchedulerOrdering::default()
        );
    
        system_registry.0.insert(system_id, system_metadata);

        let stored_system = StoredSystem::new(System::Sync(Box::new(foo.into_system())));
    
        assert!(state_machine.insert(program_id.as_ref(), Some(resource_id), program_key.as_ref(), stored_system).unwrap().is_ok());
        assert!(state_machine.insert(program_id.as_ref(), None, program_key.as_ref(), 0 as i32).unwrap().is_ok());
    }

    let _r = pollster::block_on(state_machine.transition());
    let _r = pollster::block_on(state_machine.transition());
    let _r = pollster::block_on(state_machine.transition());
}