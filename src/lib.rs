pub mod registry;

pub mod prelude {
    pub use super::registry::{
        Registry,
        registry_results::{
            RegistryAccessResult, RegistryAccessPermission, RegistryReplacementResult, RegistryReservationResult, RegistryUnReserveResult
        },
        managed_registry::{
            ManagedRegistry, registry_results::ManagedRegistryAccessResult,
            operated_registry::{
                OperatedRegistry, registry_results::OperatedRegistryAccessResult, resource_key::ResourceKey
            }
        },
        reception::{
            Reception, 
            reception_permission::{
                ReceptionAccessPermission, ReceptionReservationPermission, ReceptionUnReserve
            },
            gate::{
                Gate, key::Key,
                gate_permission::{
                    GateAccessPermission,
                }
            },
            host::{
                Host,
                host_permission::{
                    HostAccessPermission, HostReservationPermission, HostUnReserve
                },
                access_map::{
                    AccessMap, access_key::AccessKey, 
                    accessor::Accessor,
                    access_map_permission::{
                        AccessPermission, AccessRemoval
                    }
                },
                reservation_map::{
                    ReservationMap, reserver_key::ReserverKey, 
                    reservation_map_permission::{
                        ReservationMapPermission, ReservationMapUnReserve
                    }
                }
            },
        },
    };
}