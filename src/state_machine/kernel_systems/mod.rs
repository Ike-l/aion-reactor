use std::{pin::Pin, sync::Arc};

use crate::{memory::{Memory, ResourceId}, state_machine::transition_phases::TransitionPhase};

pub mod processor;
pub mod event_manager;
pub mod blocker_manager;
pub mod background_processor;

pub trait KernelSystem {
    fn init(&mut self, memory: &Memory) -> ResourceId;
    /// Cancel *Unsafe*
    fn tick(&mut self, memory: &Arc<Memory>, phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
}

pub type StoredKernelSystem = Box<dyn KernelSystem>;