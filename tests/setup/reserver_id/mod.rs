use aion_reactor::prelude::ReserverKey;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ReserverId;

impl ReserverKey for ReserverId {}