use aion_reactor::prelude::{RegistryAccessResult, RegistryReplacementResult};
use tracing::{Level, event, span};

use crate::{Generator, setup::{Access, access::access_result::AccessResult, setup}};

#[test]
fn insert_nothing() {
    let registry = setup();

    let (reserver_id, key, resource_id) = Generator::bert();

    let result = registry.accessed_replacement(
        resource_id, 
        Access::Replace, 
        reserver_id.as_ref(), 
        key.as_ref(), 
        None
    );

    assert_eq!(result, RegistryReplacementResult::ResourceNotFound);


    let (reserver_id, key, resource_id) = Generator::bert();

    let result = registry.accessed_replacement(
        resource_id, 
        Access::Replace, 
        reserver_id.as_ref(), 
        key.as_ref(), 
        None
    );

    assert_eq!(result, RegistryReplacementResult::ResourceNotFound);
}

#[test]
fn insert_something() {
    let number = 1;
    let registry = setup();
    
    let (reserver_id, key, resource_id, stored_resource) = Generator::barry(number);
    
    let span = span!(Level::DEBUG, "Insert Something Test", reserver_id =? reserver_id, some_key =? key.is_some(), resource_id =? resource_id, stored_resource =? stored_resource);
    let _enter = span.enter();

    let result = registry.accessed_replacement(
        resource_id.clone(), 
        Access::Replace, 
        reserver_id.as_ref(), 
        key.as_ref(), 
        Some(stored_resource)
    );

    // check other Access

    event!(Level::DEBUG, result =? result, "Replacement");

    assert_eq!(result, RegistryReplacementResult::ResourceNotFound);

    let result = registry.access(resource_id, Access::Shared(1), reserver_id.as_ref(), key.as_ref());
    
    event!(Level::DEBUG, result =? result, "Access");

    assert_eq!(result, RegistryAccessResult::Found(AccessResult::Shared(&number)));
}

// make sure 

// does remove/replace check for reservations? (should do but if not maybe something wrong with how Access is? if a reservation is saying)

// if make a "remove" access will need to remove accesses as well :/