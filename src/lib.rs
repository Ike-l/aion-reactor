pub mod registry;

pub mod prelude {
    pub use super::registry::{
        Registry,
        registry_results::{
            RegistryAccessResult, RegistryAccessPermission, RegistryReplacementResult
        },
        managed_registry::{
            ManagedRegistry, registry_results::ManagedRegistryAccessResult,
            operated_registry::{
                OperatedRegistry, registry_results::OperatedRegistryAccessResult, resource_key::ResourceKey
            }
        },
        reception::{
            Reception, reception_permission::ReceptionAccessPermission,
            gate::{
                Gate, key::Key,
                gate_permission::{
                    GateAccessPermission,
                }
            },
            host::{
                Host,
                host_permission::{
                    HostAccessPermission
                },
                access_map::{
                    AccessMap, access_key::AccessKey, access_map_permission::AccessPermission, accessor::Accessor
                },
                reservation_map::{
                    ReservationMap, reserver_key::ReserverKey, reservation_map_permission::ReservationMapPermission
                }
            },
        },
    };
}