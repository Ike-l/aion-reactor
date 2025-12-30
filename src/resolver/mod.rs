use std::sync::Arc;

use crate::{Memory, prelude::AccessDrop};

pub mod access_drop;

pub trait Resolver {
    type Item<'new>: AccessDrop;
    type Memory: Memory;
    
    fn retrieve<'a>(
        memory: &'a Arc<Self::Memory>, 
        resource_id: Option<&<Self::Memory as Memory>::ResourceId>,
        owner: Option<&<Self::Memory as Memory>::Owner>,
    ) -> Result<Self::Item<'a>, <Self::Memory as Memory>::Error>;
}