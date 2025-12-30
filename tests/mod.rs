use aion_reactor::State;

use crate::heap::Heap;

pub mod heap;

#[test]
fn foo() {
    let state = State::<Heap>::default();

    // state.
}