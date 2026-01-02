use std::hash::Hash;

pub trait Key: Hash + PartialEq + Eq {}