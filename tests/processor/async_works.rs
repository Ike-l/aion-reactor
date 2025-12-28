use aion_reactor::prelude::{Criteria, StateMachine, System, SystemResult};
use aion_utilities::builders::systems::SystemBuilder;
use lazy_static::lazy_static;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use std::sync::Mutex;

lazy_static! {
    static ref OUTPUT: Mutex<u32> = Mutex::new(1);
}

async fn no_input() -> Option<SystemResult> {
    tokio::time::sleep(std::time::Duration::from_secs_f32(0.2)).await;
    {
        let mut guard = OUTPUT.lock().unwrap();
        *guard += 1;
    }
    tokio::time::sleep(std::time::Duration::from_secs_f32(0.2)).await;
    None
}

#[test]
fn enters_function_body() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        // .compact()
        // .pretty()
        // .with_env_filter(EnvFilter::new("info,aion_reactor=debug"))
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .with_target(false)
        .with_test_writer()
        .init();
    
    let state_machine = StateMachine::new();
    state_machine.load_default(4);

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

    let _r = state_machine.transition();

    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, 4);
    }

    let _r = state_machine.transition();

    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, 7);
    }
}