use aion_reactor::prelude::{Criteria, KernelBuilder, StateMachine, System, SystemResult};
use aion_utilities::builders::systems::SystemBuilder;
use lazy_static::lazy_static;
use tracing::{Level, event};

use std::sync::Mutex;

use crate::init_tracing;

lazy_static! {
    static ref OUTPUT: Mutex<u32> = Mutex::new(1);
}

// #[instrument]
async fn no_input() -> Option<SystemResult> {
    tokio::time::sleep(std::time::Duration::from_secs_f32(0.2)).await;
    event!(Level::INFO, "Passed First Stage");
    {
        let mut guard = OUTPUT.lock().unwrap();
        *guard += 1;
    }
    event!(Level::INFO, "Passed Second Stage");
    tokio::time::sleep(std::time::Duration::from_secs_f32(0.2)).await;
    event!(Level::INFO, "Passed Third Stage");
    None
}

#[test]
fn enters_function_body() {
    init_tracing();
    
    let state_machine = StateMachine::new();
    KernelBuilder::full(4).init(&state_machine);

    state_machine.insert(None, None, None, 1);

    let _ = SystemBuilder::new("Foo", System::new_async(no_input))
        .replace_criteria(Criteria::new(|_| true))
        .build_blocking(&state_machine)
        .unwrap();

    let _ = SystemBuilder::new("Bar", System::new_async(no_input))
        .replace_criteria(Criteria::new(|_| true))
        .build_blocking(&state_machine)
        .unwrap();

    let _ = SystemBuilder::new("Baz", System::new_async(no_input))
        .replace_criteria(Criteria::new(|_| true))
        .build_blocking(&state_machine)
        .unwrap();
    
    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, 1);
    }

    let _r = state_machine.tick();

    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, 4);
    }

    let _r = state_machine.tick();

    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, 7);
    }
}