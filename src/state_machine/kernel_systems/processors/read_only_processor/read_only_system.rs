use crate::prelude::{Cloned, Injection, Shared, SyncSystem, SystemId};

pub trait ReadOnlyInjection: Injection {}

impl<T: 'static> ReadOnlyInjection for Shared<'_, T> {}
impl<T: Clone + 'static> ReadOnlyInjection for Cloned<T> {}

pub trait ReadOnlySystem {
    fn check_read_only(&self, source: Option<&SystemId>) -> bool;    
}

// do AsyncSystems by abstracting SyncSystem & AsyncSystem traits
impl<T: SyncSystem> ReadOnlySystem for T {
    fn check_read_only(&self, source: Option<&SystemId>) -> bool {
        SyncSystem::check_read_only(self, source)
    }
}
