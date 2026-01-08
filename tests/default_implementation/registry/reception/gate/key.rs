use aion_reactor::prelude::Key;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KeyId(u64);

impl Key for KeyId {}