use std::{fmt::Debug, hash::Hash};

pub trait ReserverKey: Debug + Hash + PartialEq + Eq {}