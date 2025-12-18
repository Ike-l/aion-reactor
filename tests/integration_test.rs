use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use aion_reactor::{id::Id, injection::injection_primitives::{cloned::Cloned, unique::Unique}, memory::{ResourceId, access_checked_heap::heap::HeapId}, state_machine::{StateMachine, kernel_systems::{event_manager::event::Event, executable_manager::{EntityId, Executable, ExecutableDataComponent, ExecutableLabelComponent, ExecutableMessage, ExecutableQueue, ExecutableRegistry, World}, processor::{processor_system_registry::ProcessorSystemRegistry, scheduler::ordering::SchedulerOrdering}}}, system::{System, async_system::into_async_system::IntoAsyncSystem, stored_system::StoredSystem, sync_system::into_sync_system::IntoSyncSystem, system_metadata::{SystemMetadata, criteria::Criteria}, system_result::SystemResult}};

fn foo(mut bar: Unique<i32>, world: Unique<World>/*, data: ExecutableBuffer*/) -> Option<SystemResult> {
    // FooExec

    let mut sources = Vec::new();
    for (_, (label, data)) in &mut world.0.query::<(&ExecutableLabelComponent, &ExecutableDataComponent)>() {
        println!("label: {label:?}");
        match &data.0 {
            ExecutableMessage::ResourceId(_) => todo!(),
            ExecutableMessage::ECS(entity_id) => sources.push(entity_id.clone()),
        }
    }
    
    let mut thingy = world.0.get::<&mut f32>(sources.remove(0).get_hecs().clone()).unwrap();
    println!("thingy: {thingy:?}");
    *thingy += 1.0;
    println!("Foo: {}", bar);
    **bar += 1;
    None
    // std::thread::sleep(Duration::from_secs(1));
    // panic!("yurp")
}

async fn dummy2() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

async fn dummy() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    dummy2().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
}

async fn async_foo(bar: Cloned<Arc<Mutex<i32>>>/*mut bar: Unique<i32>, world: Unique<World>/*, data: ExecutableBuffer*/*/) -> Option<SystemResult> {
    panic!("yurp");
    let mut bar = bar.lock().await;
    println!("Bar: {}", bar);
    *bar += 1;
    println!("Before Dummy Foo");
    dummy().await;
    println!("After Dummy Foo");
    None
    // std::thread::sleep(Duration::from_secs(1));
}


async fn async_bar(/*mut bar: Unique<i32>, world: Unique<World>/*, data: ExecutableBuffer*/*/) -> Option<SystemResult> {
    async fn inner() {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    let handle = tokio::task::spawn(async {
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(1)).await;
        7u8
    });

    assert_eq!(match handle.await {
        Ok(7) => "spawn-ok",
        Ok(_) => "spawn-weird",
        Err(_) => "spawn-panicked-or-cancelled"
    }, "spawn-ok");

    let (tx, rx) = tokio::sync::oneshot::channel::<u8>();

    tokio::spawn(async move {
        tx.send(10).ok();
    });

    let val = rx.await.unwrap();

    assert_eq!(if val == 10 {
        "ok"
    } else {
        "wrong"
    }, "ok");

    println!("Before Dummy Bar");
    dummy().await;
    println!("After Dummy Bar");
    inner().await;
    None
    // std::thread::sleep(Duration::from_secs(1));
    // panic!("yurp")
}

fn _bar(world: Unique<World>/*, data: ExecutableBuffer*/) -> Option<SystemResult> {
    // FooExec

    let mut sources = Vec::new();
    for (_, (label, data)) in &mut world.0.query::<(&ExecutableLabelComponent, &ExecutableDataComponent)>() {
        println!("label: {label:?}");
        match &data.0 {
            ExecutableMessage::ResourceId(_) => todo!(),
            ExecutableMessage::ECS(entity_id) => sources.push(entity_id.clone()),
        }
    }
    
    let thingy = world.0.get::<&f32>(sources.remove(0).get_hecs().clone()).unwrap();
    println!("thingy: {thingy:?}");
    None
    // std::thread::sleep(Duration::from_secs(1));
    // panic!("yurp")
}

#[test]
fn sync_system() {
    let state_machine = StateMachine::new();
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

    let _r = state_machine.transition();
    let _r = state_machine.transition();
    let _r = state_machine.transition();
}

#[test]
fn foobar() {
    let state_machine = StateMachine::new();
    state_machine.load_default(16);
    let trigger_event = Event::new("trigger".to_string());

    let mut world = hecs::World::new();
    let current_entity = world.spawn((1.0 as f32,));

    {
        state_machine.insert(None, None, None, World(world));
        let program_id = None;
        let program_key = None;
    
        let system_id = Id("FooSystem".to_string());
        let resource_id = ResourceId::Heap(HeapId::Label(system_id.clone()));
    
        let mut system_registry = state_machine.resolve::<Unique<ProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
    
        let criteria_trigger_event = trigger_event.clone();
        let system_metadata = SystemMetadata::new(
            resource_id.clone(), 
            program_id.clone(), 
            program_key, 
            Criteria::new(move |events| -> bool {
                events.contains(&criteria_trigger_event)
            }), 
            SchedulerOrdering::default()
        );
    
        system_registry.0.insert(system_id, system_metadata);

        let stored_system = StoredSystem::new(System::Sync(Box::new(foo.into_system())));
    
        assert!(state_machine.insert(program_id.as_ref(), Some(resource_id), program_key.as_ref(), stored_system).unwrap().is_ok());
        assert!(state_machine.insert(program_id.as_ref(), None, program_key.as_ref(), 0 as i32).unwrap().is_ok());
    }

    let _r = state_machine.transition();

    {
        let mut executable_registry = state_machine.resolve::<Unique<ExecutableRegistry>>(None, None, None, None).unwrap().unwrap();

        let executable_label = "fooExec".to_string();
        executable_registry.insert(executable_label.clone(), Executable::new(executable_label.clone(), trigger_event.clone()));

        // let mut next_events = state_machine.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
        // next_events.insert(trigger_event);

        let mut executable_queue = state_machine.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
        executable_queue.0.push((executable_label, ExecutableMessage::ECS(EntityId::new(current_entity))));
    }   
    
    let _r = state_machine.transition();

    {
        let executable_label = "fooExec".to_string();
        let mut executable_queue = state_machine.resolve::<Unique<ExecutableQueue>>(None, None, None, None).unwrap().unwrap();
        executable_queue.0.push((executable_label, ExecutableMessage::ECS(EntityId::new(current_entity))));
    }

    let _r = state_machine.transition();
}

#[test]
fn async_system() {
    let state_machine = StateMachine::new();
    state_machine.load_default(3);

    {
        let program_id = None;
        let program_key = None;
    
        let system_id = Id("FooASystem".to_string());
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

        let stored_system = StoredSystem::new(System::Async(Box::new(async_foo.into_system())));
    
        assert!(state_machine.insert(program_id.as_ref(), Some(resource_id), program_key.as_ref(), stored_system).unwrap().is_ok());
        assert!(state_machine.insert(program_id.as_ref(), None, program_key.as_ref(), Arc::new(Mutex::new(1 as i32))).unwrap().is_ok());

        let system_id = Id("BarASystem".to_string());
        let resource_id = ResourceId::Heap(HeapId::Label(system_id.clone()));

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

        let stored_system = StoredSystem::new(System::Async(Box::new(async_bar.into_system())));
        assert!(state_machine.insert(program_id.as_ref(), Some(resource_id), program_key.as_ref(), stored_system).unwrap().is_ok());
    }

    state_machine.transition();
    state_machine.transition();
    state_machine.transition();
}