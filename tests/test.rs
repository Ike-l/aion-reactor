// struct Key;
// struct ResourceId;
// struct ReservationMap;
// struct ReserverId;
// struct Access;
// struct AccessId;
// struct Resource;

// struct RegistryOperator;

// impl aion_reactor::registry::owned_registry::registry::managed_registry::RegistryOperator for RegistryOperator {
//     type Resource = Resource;
//     type ResourceId = ResourceId;
// }

// struct RegistryManager;

// impl aion_reactor::registry::owned_registry::registry::RegistryManager for RegistryManager {
//     type RegistryOperator = RegistryOperator;
// }

// struct Accessor;

// impl aion_reactor::registry::owned_registry::reception::host::Accessor for Accessor {
//     type Access = Access;
//     type AccessId = AccessId;
// }

// struct Reserver;

// impl aion_reactor::registry::owned_registry::reception::host::Reserver for Reserver {
//     type ReservationMap = ReservationMap;
//     type ReserverId = ReserverId;
// }

// struct GateKeeper;

// impl aion_reactor::registry::owned_registry::reception::GateKeeper for GateKeeper {
//     type Key = Key;
//     type ResourceId = ResourceId;
// }

// struct Hoster;

// impl aion_reactor::registry::owned_registry::reception::Hoster for Hoster {
//     type Accessor = Accessor;
//     type Reserver = Reserver;
// }

// struct Receptionist;

// impl aion_reactor::registry::owned_registry::Receptionist for Receptionist {
//     type GateKeeper = GateKeeper;
//     type Hoster = Hoster;
// }

// struct Administrator;

// impl aion_reactor::registry::owned_registry::Administrator for Administrator {
//     type RegistryManager = RegistryManager;
// }

// struct Owner;

// impl aion_reactor::registry::owned_registry::Owner for Owner {
//     type Receptionist = Receptionist;

//     type Administrator = Administrator;
// }

// fn foo() {
//     let owned_registry = aion_reactor::registry::owned_registry::OwnedRegistry::<Owner>::default();
//     let a = owned_registry.get::<i32>(ResourceId, ReserverId);
// }