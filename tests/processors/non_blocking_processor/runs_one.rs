use aion_reactor::prelude::{Criteria, StateMachine, System, SystemResult};
use aion_utilities::builders::systems::SystemBuilder;
use lazy_static::lazy_static;

use std::sync::Mutex;

use crate::init_tracing;

lazy_static! {
    static ref OUTPUT: Mutex<bool> = Mutex::new(false);
}

fn no_input() -> Option<SystemResult> {
    let mut guard = OUTPUT.lock().unwrap();
    *guard = !*guard;
    std::thread::sleep(std::time::Duration::from_secs_f32(0.4));
    None
}

#[test]
fn enters_function_body() {
    init_tracing();
    
    let state_machine = StateMachine::new();
    state_machine.load_default(1);

    let _ = SystemBuilder::new("Foo", System::new_sync(no_input))
        .replace_criteria(Criteria::new(|_| true))
        .build_non_blocking(&state_machine)
        .unwrap();
    
    {
        let guard = OUTPUT.lock().unwrap();
        assert_eq!(*guard, false);
    }

    let _r = state_machine.transition();

    std::thread::sleep(std::time::Duration::from_secs_f32(0.2));

    let _r = state_machine.transition();

    std::thread::sleep(std::time::Duration::from_secs_f32(0.1));

    let _r = state_machine.transition();

    std::thread::sleep(std::time::Duration::from_secs_f32(0.2));
    // {
    //     let guard = OUTPUT.lock().unwrap();
    //     assert_eq!(*guard, true);
    // }

    // let _r = state_machine.transition();

    // {
    //     let guard = OUTPUT.lock().unwrap();
    //     assert_eq!(*guard, false);
    // }
}