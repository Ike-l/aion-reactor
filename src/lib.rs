pub mod state_machine;
pub mod id;
pub mod memory;
pub mod system;
pub mod injection;
pub mod processor;

// #[cfg(test)]
// mod tests {
//     use crate::{id::Id, injection::injection_primitives::unique::Unique, memory::{access_checked_resource_map::resource::ResourceId, Memory}};
//     trait SyncSystem {
//         fn r(&mut self, memory: &Memory) { println!("Success") }
//     }
    
//     struct Bar;
    
//     impl SyncSystem for Bar {}
    
//     #[test]
//     fn foo() {
//         let mut memory = Memory::new();
//         memory.insert(None, Some(ResourceId::Id(Id("".to_string()))), Box::new(Bar) as Box<dyn SyncSystem>);
//         let f = memory.resolve::<Unique<Box<dyn SyncSystem>>>(None, Some(ResourceId::Id(Id("".to_string())))).unwrap().unwrap();
//         f.value.r(&memory);
//     }
// }

