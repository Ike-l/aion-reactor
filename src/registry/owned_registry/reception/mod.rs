pub mod host;
pub mod gate;
pub mod reception_permission;

pub use gate::{
    Gate,
};

pub use host::{
    Host, 
};

use crate::registry::owned_registry::{reception::{gate::{Key, gate_permission::GateAccessPermission}, host::{Accessor, access_map::AccessKey, reservation_map::ReserverKey}, reception_permission::{ReceptionAccessPermission}}, managed_registry::operated_registry::ResourceKey};

pub struct Reception<
    AccessId, 
    ReserverId,
    Access, 
    ResourceId,
    KeyId,
> {
    gate: Gate<ResourceId, KeyId>,
    host: Host<ReserverId, AccessId, Access>
}

impl<
    AccessId: ResourceKey + AccessKey, 
    ReserverId: ReserverKey,
    Access: Accessor, 
    KeyId: Key,
> Reception<AccessId, ReserverId, Access, AccessId, KeyId> {
    pub fn permits_access(
        &self,
        access_id: &AccessId,
        access: Option<&Access>,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
    ) -> ReceptionAccessPermission {
        match self.gate.allows_passage(access_id, key) {
            GateAccessPermission::Denied => ReceptionAccessPermission::NoEntry,
            GateAccessPermission::Allowed | GateAccessPermission::Unlocked => {
                ReceptionAccessPermission::Host(self.host.permits_access(reserver_id, access_id, access))
            },
        }
    }

    pub fn record_access(
        &self,
        access_id: AccessId,
        access: Access,
        reserver_id: Option<&ReserverId>,
    ) {
        self.host.record_access(access_id, access, reserver_id)
    }
}

impl<
    AccessId, 
    ReserverId,
    Access, 
    ResourceId, 
    Key,
> Default for Reception<AccessId, ReserverId, Access, ResourceId, Key> {
    fn default() -> Self {
        Self {
            gate: Gate::default(),
            host: Host::default()
        }
    }
}