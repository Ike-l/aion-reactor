use std::{pin::Pin, sync::Arc};

use crate::{memory::Memory, state_machine::transition_phases::TransitionPhase};

pub mod processor;
pub mod event_manager;
pub mod blocker_manager;
pub mod background_processor;

pub trait KernelSystem {
    /// Cancel *Unsafe*
    fn tick(&mut self, memory: &Arc<Memory>, phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
}

pub type StoredKernelSystem = Box<dyn KernelSystem>;