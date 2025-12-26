use std::collections::HashMap;

use crate::{ids::system_id::SystemId, prelude::EventId};

#[derive(Default)]
pub struct SystemEventRegistry(pub HashMap<SystemId, Vec<EventId>>);