use aion_reactor::registry::owned_registry::{OwnedRegistry, registry_results::OwnedRegistryAccessResult};

pub mod reserver_id;
pub mod access;
pub mod resource_id;
pub mod key_id;
pub mod resource;

pub use reserver_id::ReserverId;
pub use access::Access;
pub use resource_id::ResourceId;
pub use key_id::KeyId;
pub use resource::Resource;

use crate::setup::access::access_result::AccessResult;

pub fn setup() -> OwnedRegistry<ResourceId, ReserverId, Access, ResourceId, KeyId, Resource> {
    OwnedRegistry::default()
}

#[test]
fn get() {
    let registry = setup();

    let resource_id = ResourceId::Labelled("foo".to_string());
    let access = Access::Unique;

    registry.replace(
        resource_id.clone(),
        None,
        Resource(1)
    );

    let resource = registry.access(
        &resource_id, 
        &access, 
        None, 
        None
    );

    assert_eq!(resource, OwnedRegistryAccessResult::Found(AccessResult::Unique(&mut 0)));
}