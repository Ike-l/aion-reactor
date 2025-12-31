pub mod host;
pub mod gate;
pub mod reception_permission;

pub use gate::{
    Gate,
    GateKeeper
};

pub use host::{
    Host, 
    Hoster
};

use crate::registry::owned_registry::reception::{host::{Accessor, Reserver}, reception_permission::{ReceptionPermission}};

pub trait Receptionist {
    type Hoster: Hoster;
    type GateKeeper: GateKeeper;
}

pub struct Reception<R: Receptionist> {
    gate: Gate<R::GateKeeper>,
    host: Host<R::Hoster>
}

impl<R: Receptionist> Reception<R> {
    pub fn permits_access(
        &self,
        reserver_id: Option<&<<<R as Receptionist>::Hoster as Hoster>::Reserver as Reserver>::ReserverId>,
        access_id: &<<<R as Receptionist>::Hoster as Hoster>::Accessor as Accessor>::AccessId,
        access: &<<<R as Receptionist>::Hoster as Hoster>::Accessor as Accessor>::Access,
    ) -> ReceptionPermission {
        todo!();
        // match self.gate.allows_passage() {

        // }
        // if self.gate.allows_passage() {
        //     ReceptionPermission::Host(self.host.permits_access(reserver_id, access_id, access))
        // } else {
        //     ReceptionPermission::NoEntry
        // }
    }
}

impl<R: Receptionist> Default for Reception<R> {
    fn default() -> Self {
        Self {
            gate: Gate::default(),
            host: Host::default()
        }
    }
}