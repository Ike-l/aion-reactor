use std::hash::Hash;

pub trait ResourceKey: Hash + PartialEq + Eq {}