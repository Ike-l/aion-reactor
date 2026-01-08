use std::collections::HashMap;

use tracing::{Level, event, span};

use crate::prelude::{AccessKey, AccessMap, AccessPermission, Accessor, ReservationMapPermission, ReservationMapUnReserveResult, ReserverKey};

pub mod reservation_map_permission;
pub mod reserver_key;


pub struct ReservationMap<
    ReserverId, 
    AccessId, 
    Access, 
> {
    reservations: parking_lot::RwLock<HashMap<ReserverId, AccessMap<AccessId, Access>>>
}

impl<
    ReserverId: ReserverKey, 
    AccessId: AccessKey, 
    Access: Accessor, 
> ReservationMap<ReserverId, AccessId, Access> {
    pub fn permits_access(
        &self,
        reserver_id: &Option<&ReserverId>,
        access_id: &AccessId,
        access: &Access,
    ) -> ReservationMapPermission {
        let span = span!(Level::DEBUG, "ReservationMap Permits Access");
        let _enter = span.enter();

        ReservationMapPermission::ReservationConflict(self.reservations.read()
            .iter()
            .any(|(reserver, reservation_map)| {
                let is_reservers_reservations = reserver_id.is_some_and(|reserver_id| reserver_id == reserver);
                if !is_reservers_reservations {
                    match reservation_map.permits_access(access_id, access) {
                        AccessPermission::Access(can_coexit) => {
                            if can_coexit {
                                false
                            } else {
                                event!(Level::WARN, conflicting_reserver =? reserver, "Reservation Conflict");
                                true
                            }
                        },
                        AccessPermission::UnknownAccessId => false,
                    }
                } else {
                    false
                }
            }))
    }

    pub fn unreserve(
        &self,
        reserver_id: &ReserverId,
        access_id: &AccessId,
        access: &Access
    ) -> ReservationMapUnReserveResult {   
        let span = span!(Level::DEBUG, "ReservationMap Unreserve");
        let _enter = span.enter();

        if let Some(reserver) = self.reservations.read().get(reserver_id) {
            ReservationMapUnReserveResult::AccessMap(reserver.remove_access(access_id, access))
        } else {
            ReservationMapUnReserveResult::NoReservation
        }
    }

    pub fn reserve(
        &self,
        reserver_id: ReserverId,
        access_id: AccessId,
        access: Access
    ) {
        self.reservations.write().entry(reserver_id).or_default().record_access(access_id, access);
    }
}

impl<
    ReserverId, 
    AccessId, 
    Access, 
> Default for ReservationMap<ReserverId, AccessId, Access> {
    fn default() -> Self {
        Self {
            reservations: parking_lot::RwLock::new(HashMap::new())
        }
    }
}