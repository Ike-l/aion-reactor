pub mod state_machine;
pub mod id;
pub mod memory;
pub mod system;
pub mod injection;

pub mod kernel_prelude {
    pub use std::{pin::Pin, sync::Arc};

    pub use crate::{memory::{Memory, ResourceId}, state_machine::{kernel_systems::KernelSystem, transition_phases::TransitionPhase}};
}


// #[cfg(test)]
// mod tests {
//     use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}};

//     trait SyncSystem {
//         fn r(&mut self, _memory: &Memory) { println!("Success 1") }

//         fn s(&self, _memory: &Memory) { println!("Success 2") }
//     }
    
//     struct Bar;
    
//     impl SyncSystem for Bar {}
    
//     #[test]
//     fn foo() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
//         let mut f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.r(&memory);
//     }

//     #[test]
//     fn bar() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
//         let mut f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.r(&memory);
//         drop(f);
//         let mut f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.r(&memory);
//     }

//     #[test]
//     fn baz() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
//         let f = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.s(&memory);
//         let fa = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         fa.s(&memory);
//     }

//     #[test]
//     #[should_panic]
//     fn fizz() {
//         let memory = Memory::new();
//         let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));

//         let b = memory.reserve_accesses::<Unique<Box<dyn SyncSystem>>>(None, Some(heap_label.clone()), ResourceId::Heap(HeapId::Label(Id("yuurp".to_string()))));
//         memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);

//         let f = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         f.s(&memory);

//         let fa = memory.resolve::<Shared<Box<dyn SyncSystem>>>(None, Some(&heap_label), None).unwrap().unwrap();
//         fa.s(&memory);
//     }
// }

