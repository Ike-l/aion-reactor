use std::{fmt::Debug, hash::Hash};

pub trait AccessKey: Debug + Hash + PartialEq + Eq + Clone {}