use aion_reactor::prelude::{OperatedRegistry, OperatedRegistryAccessResult, OperatedRegistryReplacementResult};

use crate::default_implementation::{init_tracing, prelude::{Access, Resource, ResourceId, StoredResource}};

pub mod resource_key;

fn setup_operated_registry() -> OperatedRegistry<ResourceId, Box<StoredResource>> {
    init_tracing();
    OperatedRegistry::default()
}

#[test]
fn empty_registry_access() {
    let registry = setup_operated_registry();

    let resource_id = ResourceId::labelled("foo");
    for access in Access::all() {
        let stored_resource = registry.access(&resource_id, &access);
        assert_eq!(stored_resource, OperatedRegistryAccessResult::ResourceNotFound);
    }
}

#[test]
fn wrong_resource_id_access() {
    let mut registry = setup_operated_registry();

    let in_resource_id = ResourceId::labelled("foo");

    let stored_resource = StoredResource::new(Resource::new(1));
    let replacement_result = registry.accessed_replace(in_resource_id, &Access::Replace, Some(stored_resource));
    assert_eq!(replacement_result, OperatedRegistryReplacementResult::ResourceNotFound);

    let wrong_resource_id = ResourceId::labelled("bar");
    for access in Access::all() {
        let stored_resource = registry.access(&wrong_resource_id, &access);
        assert_eq!(stored_resource, OperatedRegistryAccessResult::ResourceNotFound);
    }
}

#[test]
fn found_resource_access() {
    let mut registry = setup_operated_registry();

    let resource_id = ResourceId::labelled("foo");
    let stored_resource = StoredResource::new(Resource::new(1));
    let replacement_result = registry.accessed_replace(resource_id.clone(), &Access::Replace, Some(stored_resource));
    assert_eq!(replacement_result, OperatedRegistryReplacementResult::ResourceNotFound);

    for access in Access::all() {
        // will panic
        if access == Access::Replace {
            continue;
        }

        let stored_resource = registry.access(&resource_id, &access);
        assert!(matches!(stored_resource, OperatedRegistryAccessResult::Found(_)))
    }
}

#[should_panic]
#[test]
fn found_resource_bad_access() {
    let mut registry = setup_operated_registry();

    let resource_id = ResourceId::labelled("foo");
    let stored_resource = StoredResource::new(Resource::new(1));
    let replacement_result = registry.accessed_replace(resource_id.clone(), &Access::Replace, Some(stored_resource));
    assert_eq!(replacement_result, OperatedRegistryReplacementResult::ResourceNotFound);

    let access = Access::Replace;

    let stored_resource = registry.access(&resource_id, &access);

    assert!(!matches!(stored_resource, OperatedRegistryAccessResult::Found(_)));
}

// Replace:
// ResourceNotFound
//  insert
//  insert again does not give
//  insert different
// AccessFailure
//   if removes and cant remove
//   if inserting and cant insert
// Found
//   returns owned