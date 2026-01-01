use aion_reactor::registry::owned_registry::reception::gate::Key;

#[derive(Hash, PartialEq, Eq)]
pub struct KeyId;

impl Key for KeyId {}