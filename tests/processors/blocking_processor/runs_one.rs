use aion_reactor::{injection::injection_primitives::{shared::Shared, unique::Unique}, prelude::KernelBuilder, state_machine::{StateMachine, kernel_systems::processors::system::{System, system_metadata::criteria::Criteria, system_result::SystemResult}}};
use aion_utilities::builders::{resolver::ResolverBuilder, resources::ResourceBuilder, systems::SystemBuilder};

use lazy_static::lazy_static;

use std::sync::Mutex;

use crate::init_tracing;

lazy_static! {
    static ref OUTPUT: Mutex<bool> = Mutex::new(false);
}

fn no_input() -> Option<SystemResult> {
    tracing::event!(tracing::Level::DEBUG, "Hello");
    let mut guard = OUTPUT.lock().unwrap();
    *guard = !*guard;
    tracing::event!(tracing::Level::DEBUG, "World");
    None
}

#[test]
fn enters_function_body() {
    init_tracing();

    let state_machine = StateMachine::new();
    KernelBuilder::full(1).init(&state_machine);

    let _ = SystemBuilder::new("Foo", System::new_sync(no_input))
        .replace_criteria(Criteria::new(|_| true))
        .build_blocking(&state_machine)
        .unwrap();
    
    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, false);
    }

    let _r = state_machine.tick();

    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, true);
    }

    let _r = state_machine.tick();

    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, false);
    }
}

fn has_input(mut number: Unique<i32>) -> Option<SystemResult> {
    **number += 1;

    // Some(SystemResult::Error(anyhow::anyhow!("has input has errored!")))
    Some(SystemResult::Conditional(true))
    // Some(SystemResult::Event(SystemEvent::NoEvent))
}

#[test]
fn state_changes() {
    let state_machine = StateMachine::new();
    KernelBuilder::full(2).init(&state_machine);

    let _ = SystemBuilder::new("Foo", System::new_sync(has_input))
        .replace_criteria(Criteria::new(|_| true))
        .build_blocking(&state_machine)
        .unwrap();
    
    let resource_builder = ResourceBuilder::new();
    resource_builder.build(&state_machine, 0 as i32);
    
    let resolver_builder = ResolverBuilder::new();
    
    {
        let number = resolver_builder.resolve::<Shared<i32>>(&state_machine).unwrap();
        assert_eq!(**number, 0);
    }
    
    let _r = state_machine.tick();
    
    {
        let number = resolver_builder.resolve::<Shared<i32>>(&state_machine).unwrap();
        assert_eq!(**number, 1);
    }

    let _r = state_machine.tick();
    
    {
        let number = resolver_builder.resolve::<Shared<i32>>(&state_machine).unwrap();
        assert_eq!(**number, 2);
    }
}