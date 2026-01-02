use std::hash::Hash;

pub trait AccessKey: Hash + PartialEq + Eq + Clone {}