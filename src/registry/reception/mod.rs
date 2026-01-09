use tracing::{Level, span};

use crate::prelude::{AccessKey, Accessor, Gate, GateAccessPermission, Host, HostDeAccessResult, Key, ReceptionAccessPermission, ReceptionDeAccessResult, ReceptionRecordAccessResult, ReceptionReservationPermission, ReceptionUnReserveResult, ReserverKey, ResourceKey};

pub mod host;
pub mod gate;
pub mod reception_permission;

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
        access: &Access,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
    ) -> ReceptionAccessPermission {
        let span = span!(Level::DEBUG, "Reception Permits Access");
        let _enter = span.enter();

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
        key: Option<&KeyId>
    ) -> ReceptionRecordAccessResult {
        let span = span!(Level::DEBUG, "Reception Record Access");
        let _enter = span.enter();
        match self.gate.allows_passage(&access_id, key) {
            GateAccessPermission::Denied => ReceptionRecordAccessResult::NoEntry,
            GateAccessPermission::Allowed |
            GateAccessPermission::Unlocked => {
                self.host.record_access(access_id, access, reserver_id);
                ReceptionRecordAccessResult::Ok
            },
        }
    }

    pub fn deaccess(
        &self,
        access_id: &AccessId,
        access: &Access,
        key: Option<&KeyId>
    ) -> ReceptionDeAccessResult {
        match self.gate.allows_passage(access_id, key) {
            GateAccessPermission::Denied => ReceptionDeAccessResult::NoEntry,
            GateAccessPermission::Allowed |
            GateAccessPermission::Unlocked => {
                match self.host.deaccess(access_id, access) {
                    HostDeAccessResult::Ok => ReceptionDeAccessResult::Ok,
                    HostDeAccessResult::UnknownAccessId => ReceptionDeAccessResult::UnknownAccessId,
                }
            },
        }
    }

    pub fn unreserve(
        &self,
        reserver_id: &ReserverId,
        access_id: &AccessId,
        access: &Access,
        key: Option<&KeyId>
    ) -> ReceptionUnReserveResult {
        match self.gate.allows_passage(access_id, key) {
            GateAccessPermission::Denied => ReceptionUnReserveResult::NoEntry,
            GateAccessPermission::Allowed |
            GateAccessPermission::Unlocked => ReceptionUnReserveResult::Host(self.host.unreserve(reserver_id, access_id, access))
        }
    }

    pub fn reserve(
        &self,
        reserver_id: ReserverId,
        access_id: AccessId,
        access: Access,
        key: Option<&KeyId>
    ) -> ReceptionReservationPermission {
        match self.gate.allows_passage(&access_id, key) {
            GateAccessPermission::Denied => ReceptionReservationPermission::NoEntry,
            GateAccessPermission::Allowed |
            GateAccessPermission::Unlocked => ReceptionReservationPermission::Host(self.host.reserve(reserver_id, access_id, access)),
        }
    }

    pub fn clear_accesses(&self) {
        self.host.clear_accesses()
    }
}

impl<
    AccessId,
    ReserverId,
    Access: Accessor,
    ResourceId,
    KeyId,
> Reception<AccessId, ReserverId, Access, ResourceId, KeyId> {
    pub fn is_active(&self) -> bool {
        self.host.is_active()
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