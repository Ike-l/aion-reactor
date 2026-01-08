pub mod reservation_map;
pub mod access_map;
pub mod host_permission;

use tracing::{Level, span};

use crate::prelude::{AccessKey, AccessMap, AccessPermission, AccessRemovalResult, Accessor, HostAccessPermission, HostDeAccessResult, HostReservationPermission, HostUnReserveResult, ReservationMap, ReservationMapPermission, ReserverKey};

pub struct Host<
    ReserverId,
    AccessId,
    Access, 
> {
    reservation_map: ReservationMap<ReserverId, AccessId, Access>,
    access_map: AccessMap<AccessId, Access>,
}

impl<
    ReserverId: ReserverKey,
    AccessId: AccessKey,
    Access: Accessor, 
> Host<ReserverId, AccessId, Access> {
    pub fn permits_access(
        &self,
        reserver_id: Option<&ReserverId>,
        access_id: &AccessId,
        access: &Access,
    ) -> HostAccessPermission {
        let span = span!(Level::DEBUG, "Host Permits Access");
        let _enter = span.enter();

        match self.reservation_map.permits_access(&reserver_id, &access_id, access) {
            ReservationMapPermission::ReservationConflict(conflicts) => {
                if conflicts {
                    HostAccessPermission::ReservationConflict
                } else {
                    HostAccessPermission::AccessMap(self.access_map.permits_access(&access_id, access))
                }
            },
        }
    }

    pub fn record_access(
        &self,
        access_id: AccessId,
        access: Access,
        reserver_id: Option<&ReserverId>
    ) {
        let span = span!(Level::DEBUG, "Host Record Access");
        let _enter = span.enter();

        if let Some(reserver_id) = reserver_id {
            self.reservation_map.unreserve(reserver_id, &access_id, &access);
        }

        self.access_map.record_access(access_id, access);
    }

    pub fn deaccess(
        &self,
        access_id: &AccessId,
        access: &Access
    ) -> HostDeAccessResult {
        match self.access_map.remove_access(access_id, access) {
            AccessRemovalResult::Split => HostDeAccessResult::Ok,
            AccessRemovalResult::UnknownAccessId => HostDeAccessResult::UnknownAccessId,
        }
    }

    pub fn unreserve(
        &self,
        reserver_id: &ReserverId,
        access_id: &AccessId,
        access: &Access
    ) -> HostUnReserveResult {
        HostUnReserveResult::ReservationMap(self.reservation_map.unreserve(reserver_id, access_id, access))
    }

    pub fn reserve(
        &self,
        reserver_id: ReserverId,
        access_id: AccessId,
        access: Access
    ) -> HostReservationPermission {
        match self.access_map.permits_access(&access_id, &access) {
            AccessPermission::Access(false) => HostReservationPermission::CurrentAccessConflict,
            AccessPermission::Access(true) |
            AccessPermission::UnknownAccessId => {
                match self.reservation_map.permits_access(&Some(&reserver_id), &access_id, &access) {
                    ReservationMapPermission::ReservationConflict(true) => HostReservationPermission::ReservationConflict,
                    ReservationMapPermission::ReservationConflict(false) => {
                        self.reservation_map.reserve(reserver_id, access_id, access);
                        HostReservationPermission::Ok
                    },
                }
            }
        }
    }

    pub fn clear_accesses(&self) {
        self.access_map.clear_accesses()
    }
}

impl<
    AccessId,
    ReserverId,
    Access: Accessor,
> Host<ReserverId, AccessId, Access> {
    pub fn is_active(&self) -> bool {
        self.access_map.is_active()
    }
}

impl<
    ReserverId,
    AccessId,
    Access, 
> Default for Host<ReserverId, AccessId, Access> {
    fn default() -> Self {
        Self { 
            reservation_map: ReservationMap::default(), 
            access_map: AccessMap::default() 
        }
    }
}