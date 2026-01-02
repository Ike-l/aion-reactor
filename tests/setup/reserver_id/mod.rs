use aion_reactor::prelude::ReserverKey;

#[derive(Hash, PartialEq, Eq)]
pub struct ReserverId;

impl ReserverKey for ReserverId {}