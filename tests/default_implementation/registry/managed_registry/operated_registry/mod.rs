use aion_reactor::prelude::{Accessor, OperatedRegistry, OperatedRegistryAccessResult, OperatedRegistryReplacementResult};
use tracing::{Level, span};

use crate::default_implementation::{init_tracing, prelude::{Access, AccessResult, Resource, ResourceId, StoredResource}};

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

#[test]
fn noop_replace() {
    let mut registry = setup_operated_registry();
    let resource_id = ResourceId::labelled("foo");
    
    for access in Access::all() {
        let result = registry.accessed_replace(resource_id.clone(), &access, None);
        assert_eq!(result, OperatedRegistryReplacementResult::NoOp);
        
        assert!(!registry.contains(&resource_id));
    }
}

#[test]
fn insert_something_replace() {
    let resource_id = ResourceId::labelled("foo");
    let resource = StoredResource::new(Resource::new(1));

    for access in Access::all() {
        let mut registry = setup_operated_registry();
        let result = registry.accessed_replace(resource_id.clone(), &access, Some(resource.clone()));
        if access == Access::Replace {
            assert_eq!(result, OperatedRegistryReplacementResult::ResourceNotFound);

            assert!(registry.contains(&resource_id));
        } else {
            assert_eq!(result, OperatedRegistryReplacementResult::AccessFailure);

            assert!(!registry.contains(&resource_id));
        }
    }
}

#[test]
fn access_respected_replace() {
    let resource_id = ResourceId::labelled("foo");
    let resource = StoredResource::new(Resource::new(1));

    for access in Access::all() {
        let mut registry = setup_operated_registry();

        let span = span!(Level::DEBUG, "Access", access=?access);
        let _enter = span.enter();
        
        // insert
        let result = registry.accessed_replace(resource_id.clone(), &access, Some(resource.clone()));
        if access.can_insert() {
            assert_eq!(result, OperatedRegistryReplacementResult::ResourceNotFound);
            
            assert!(registry.contains(&resource_id));
        } else {
            assert_eq!(result, OperatedRegistryReplacementResult::AccessFailure);
            
            assert!(!registry.contains(&resource_id));
        }

        // replace
        let result = registry.accessed_replace(resource_id.clone(), &access, Some(resource.clone()));
        if access.can_remove() && access.can_insert() {
            assert_eq!(result, OperatedRegistryReplacementResult::Found(AccessResult::Owned(resource.clone())));
            
            assert!(registry.contains(&resource_id));
        } else {
            assert_eq!(result, OperatedRegistryReplacementResult::AccessFailure);
            
            assert!(!registry.contains(&resource_id));
        }

        // remove
        let result = registry.accessed_replace(resource_id.clone(), &access, None);
        if access.can_remove() {
            assert_eq!(result, OperatedRegistryReplacementResult::Found(AccessResult::Owned(resource.clone())));
            
            assert!(!registry.contains(&resource_id));
        } else {
            assert_eq!(result, OperatedRegistryReplacementResult::NoOp);

            assert!(!registry.contains(&resource_id));
        }
    }
}


#[test]
fn insert_respects_id() {
    let resource_id = ResourceId::labelled("foo");
    let other_resource_id = ResourceId::labelled("bar");

    let resource = StoredResource::new(Resource::new(1));
    let other_resource = StoredResource::new(Resource::new(2));

    let mut registry = setup_operated_registry();

    assert!(!registry.contains(&resource_id));
    assert!(!registry.contains(&other_resource_id));

    let result = registry.accessed_replace(resource_id.clone(), &Access::Replace, Some(resource.clone()));
    assert_eq!(result, OperatedRegistryReplacementResult::ResourceNotFound);

    assert!(registry.contains(&resource_id));
    assert!(!registry.contains(&other_resource_id));

    let result = registry.accessed_replace(resource_id.clone(), &Access::Replace, Some(resource.clone()));
    assert_eq!(result, OperatedRegistryReplacementResult::Found(AccessResult::Owned(resource)));

    assert!(registry.contains(&resource_id));
    assert!(!registry.contains(&other_resource_id));

    let result = registry.accessed_replace(other_resource_id.clone(), &Access::Replace, Some(other_resource.clone()));
    assert_eq!(result, OperatedRegistryReplacementResult::ResourceNotFound);

    assert!(registry.contains(&resource_id));
    assert!(registry.contains(&other_resource_id));

    let result = registry.accessed_replace(other_resource_id.clone(), &Access::Replace, Some(other_resource.clone()));
    assert_eq!(result, OperatedRegistryReplacementResult::Found(AccessResult::Owned(other_resource)));
}
