use std::collections::HashMap;

use crate::prelude::{AccessKey, AccessMap, AccessPermission, Accessor, ReservationMapPermission, ReserverKey};

pub mod reservation_map_permission;
pub mod reserver_key;


pub struct ReservationMap<
    ReserverId, 
    AccessId, 
    Access, 
> {
    reservations: HashMap<ReserverId, AccessMap<AccessId, Access>>
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
        ReservationMapPermission::ReservationConflict(self.reservations
            .iter()
            .any(|(reserver, reservation_map)| {
                let is_reservers_reservations = reserver_id.is_some_and(|reserver_id| reserver_id == reserver);
                if !is_reservers_reservations {
                    match reservation_map.permits_access(access_id, Some(access)) {
                        AccessPermission::Access(can_coexit) => !can_coexit,
                        AccessPermission::UnknownAccessId => false,
                        AccessPermission::Insert(_) => unreachable!("Access is Some"),
                    }
                } else {
                    false
                }
            }))
    }

    pub fn record_access(
        &self,
        reserver_id: &ReserverId,
        access_id: &AccessId,
        access: &Access
    ) {   
        if let Some(reserver) = self.reservations.get(reserver_id) {
            reserver.remove_access(access_id, access)
        }
    }
}

impl<
    ReserverId, 
    AccessId, 
    Access, 
> Default for ReservationMap<ReserverId, AccessId, Access> {
    fn default() -> Self {
        Self {
            reservations: HashMap::new()
        }
    }
}