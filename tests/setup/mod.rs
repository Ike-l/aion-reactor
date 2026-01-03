pub mod reserver_id;
pub mod access;
pub mod resource_id;
pub mod key_id;
pub mod resource;

use aion_reactor::prelude::Registry;

pub use reserver_id::ReserverId;
pub use access::Access;
pub use resource_id::ResourceId;
pub use key_id::KeyId;
pub use resource::StoredResource;

use crate::init_tracing;

pub fn setup() -> Registry<ResourceId, ReserverId, Access, ResourceId, KeyId, StoredResource> {
    init_tracing();
    Registry::default()
}

// #[test]
// fn foo() {
//     let registry = setup();

//     let resource_id = ResourceId::Labelled("foo".to_string());

//     assert_eq!(registry.accessed_replacement(
//         resource_id.clone(),
//         Access::Replace,
//         None,
//         None,
//         Some(StoredResource(1))
//     ), RegistryReplacementResult::ResourceNotFound);

//     assert_eq!(registry.accessed_replacement(
//         resource_id.clone(),
//         Access::Replace,
//         None,
//         None,
//         Some(StoredResource(2))
//     ), RegistryReplacementResult::Found(AccessResult::Owned(StoredResource(1))));

    // let r_2 = ResourceId::Labelled("f".to_string());
    // assert_eq!(registry.replace(
    //     r_2,
    //     None,
    //     StoredResource(2)
    // ), RegistryReplacementResult::Ok(None));

    // let resource_id_bar = ResourceId::Labelled("foo".to_string());
    // let access_bar = Access::Owned;

    // let resource_bar = registry.access(
    //     resource_id_bar, 
    //     access_bar, 
    //     None, 
    //     None
    // );

    // assert_eq!(resource_bar, RegistryAccessResult::Found(AccessResult::Owned(1)));

    // assert_eq!(registry.replace(
    //     resource_id.clone(),
    //     None,
    //     StoredResource(2)
    // ), RegistryReplacementResult::Ok(Some(StoredResource(1))));

    // let resource = registry.access(
    //     resource_id.clone(), 
    //     access, 
    //     None, 
    //     None
    // );


    // assert_eq!(resource, RegistryAccessResult::Found(AccessResult::Shared(&2)));

    // assert_eq!(registry.replace(
    //     resource_id.clone(),
    //     None,
    //     StoredResource(2)
    // ), RegistryReplacementResult::Denied);

// }