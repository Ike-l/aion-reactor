pub mod registry;

pub mod prelude {
    pub use super::registry::{
        Registry,
        registry_results::{
            RegistryAccessResult, RegistryAccessPermission, RegistryReplacementResult, RegistryReservationResult, RegistryUnReserveResult, RegistryDeAccessResult
        },
        managed_registry::{
            ManagedRegistry, 
            registry_results::{
                ManagedRegistryAccessResult, ManagedRegistryReplacementResult 
            },
            operated_registry::{
                OperatedRegistry, 
                registry_results::{
                    OperatedRegistryAccessResult, OperatedRegistryReplacementResult
                }, 
                resource_key::ResourceKey
            }
        },
        reception::{
            Reception, 
            reception_permission::{
                ReceptionAccessPermission, ReceptionReservationPermission, ReceptionUnReserveResult, ReceptionDeAccessResult, ReceptionRecordAccessResult
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
                    HostAccessPermission, HostReservationPermission, HostUnReserveResult, HostDeAccessResult
                },
                access_map::{
                    AccessMap, access_key::AccessKey, 
                    accessor::Accessor,
                    access_map_permission::{
                        AccessPermission, AccessRemovalResult
                    }
                },
                reservation_map::{
                    ReservationMap, reserver_key::ReserverKey, 
                    reservation_map_permission::{
                        ReservationMapPermission, ReservationMapUnReserveResult
                    }
                }
            },
        },
    };
}