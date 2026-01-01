use aion_reactor::registry::owned_registry::reception::host::reservation_map::ReserverKey;

#[derive(Hash, PartialEq, Eq)]
pub struct ReserverId;

impl ReserverKey for ReserverId {}