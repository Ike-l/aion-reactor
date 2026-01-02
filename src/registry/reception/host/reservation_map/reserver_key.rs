use std::hash::Hash;

pub trait ReserverKey: Hash + PartialEq + Eq {}