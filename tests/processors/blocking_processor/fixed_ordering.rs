use aion_reactor::{injection::injection_primitives::{shared::Shared, unique::Unique}, prelude::KernelBuilder, state_machine::{StateMachine, kernel_systems::processors::{blocking_processor::scheduler::ordering::SchedulerOrdering, system::{System, system_metadata::criteria::Criteria, system_result::SystemResult}}}};
use aion_utilities::builders::{resolver::ResolverBuilder, resources::ResourceBuilder, systems::SystemBuilder};

fn foo(mut number: Unique<i32>) -> Option<SystemResult> {
    assert_eq!(**number, 0);
    **number += 1;

    None
}

fn bar(mut number: Unique<i32>) -> Option<SystemResult> {
    assert_eq!(**number, 1);
    **number += 1;

    None
}

#[test]
fn state_conserved() {
    let state_machine = StateMachine::new();
    KernelBuilder::full(16).init(&state_machine);

    let foo_builder_result = SystemBuilder::new("Foo", System::new_sync(foo))
        .replace_criteria(Criteria::new(|_| true))
        .build_blocking(&state_machine)
        .unwrap();

    let foo_system_id = foo_builder_result.system_id;

    let _ = SystemBuilder::new("Bar", System::new_sync(bar))
        .replace_criteria(Criteria::new(|_| true))
        .insert_ordering(SchedulerOrdering::default().insert_after(foo_system_id))
        .build_blocking(&state_machine)
        .unwrap();

    let resource_builder = ResourceBuilder::new();
    let resolver_builder = ResolverBuilder::new();
    
    let now = std::time::Instant::now();
    while now.elapsed() < std::time::Duration::from_secs(1) {
        resource_builder.build(&state_machine, 0 as i32);
        let _r = state_machine.tick();
        assert_eq!(**resolver_builder.resolve::<Shared<i32>>(&state_machine).unwrap(), 2);
    }
}