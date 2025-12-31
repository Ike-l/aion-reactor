use std::collections::HashMap;

use crate::registry::owned_registry::reception::host::{AccessMap, Accessor, access_map::access_map_permission::AccessMapPermission, reservation_map::reservation_map_permission::ReservationMapPermission};

pub mod reservation_map_permission;

pub trait Reserver {
    type ReserverId: PartialEq;
}

pub struct ReservationMap<R: Reserver, A: Accessor> {
    reservations: HashMap<R::ReserverId, AccessMap<A>>
}

impl<R: Reserver, A: Accessor> ReservationMap<R, A> {
    pub fn permits_access(
        &self,
        reserver_id: &Option<&R::ReserverId>,
        access_id: &<A as Accessor>::AccessId,
        access: &<A as Accessor>::Access,
    ) -> ReservationMapPermission {
        ReservationMapPermission::ReservationConflict(self.reservations
            .iter()
            .any(|(reserver, reservation_map)| {
                let is_reservers_reservations = reserver_id.is_some_and(|reserver_id| reserver_id == reserver);
                if !is_reservers_reservations {
                    match reservation_map.permits_access(access_id, access) {
                        AccessMapPermission::Coexistence(can_coexit) => !can_coexit,
                        AccessMapPermission::UnknownAccessId => false,
                    }
                } else {
                    false
                }
            }))
    }
}

impl<R: Reserver, A: Accessor> Default for ReservationMap<R, A> {
    fn default() -> Self {
        Self {
            reservations: HashMap::new()
        }
    }
}