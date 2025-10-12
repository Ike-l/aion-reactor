pub mod state_machine;
pub mod id;
pub mod memory;
pub mod system;
pub mod injection;
pub mod processor;

#[cfg(test)]
mod tests {
    use crate::{id::Id, injection::injection_primitives::unique::Unique, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}};

    trait SyncSystem {
        fn r(&mut self, memory: &Memory) { println!("Success") }
    }
    
    struct Bar;
    
    impl SyncSystem for Bar {}
    
    #[test]
    fn foo() {
        let mut memory = Memory::new();
        let heap_label = ResourceId::Heap(HeapId::Label(Id("()".to_string())));
        memory.insert(None, Some(heap_label.clone()), Box::new(Bar) as Box<dyn SyncSystem>);
        let f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(heap_label)).unwrap().unwrap();
        f.value.r(&memory);
    }
}

