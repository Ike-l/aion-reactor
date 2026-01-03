use crate::setup::{Access, ResourceId, StoredResource, setup};

pub mod insert;

#[test]
pub fn dangling() {
    // let ptr: *const i32;
    // {
    //     let x = 42;
    //     ptr = &x;
    // }
    // unsafe {
    //     println!("{}", *ptr);
    // }
    
    let registry = setup();

    let resource = StoredResource(1);

    let resource_id = ResourceId::Labelled("1".to_string());
    registry.accessed_replacement(resource_id.clone(), Access::Replace, None, None, Some(resource.clone()));
    let access = registry.access(resource_id, Access::Unique, None, None);

    let resource_id = ResourceId::Labelled("2".to_string());
    registry.accessed_replacement(resource_id, Access::Replace, None, None, Some(resource.clone()));
    let resource_id = ResourceId::Labelled("3".to_string());
    registry.accessed_replacement(resource_id, Access::Replace, None, None, Some(resource.clone()));
    let resource_id = ResourceId::Labelled("4".to_string());
    registry.accessed_replacement(resource_id, Access::Replace, None, None, Some(resource.clone()));

    println!("access: {access:?}");

    // strategies if fail:
    // smart pointers
    // fixed capacity hashmapa
    // explicity resize: checks accesses
    // ^- use if current_access.is_held()
    // is_held means. Does this access imply a reference to the resource
}