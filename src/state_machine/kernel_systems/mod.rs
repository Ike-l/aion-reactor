use std::{pin::Pin, sync::Arc};

use crate::prelude::{Memory, ProgramId, ProgramKey, SystemId};

pub mod processors;
pub mod managers;

// Could have init resources
// and check_resources -> bool;
// so 2 phase initialisation

pub trait KernelSystem {
    fn system_id(&self) -> SystemId;

    fn init(&mut self, memory: &Memory, kernel_program_id: &ProgramId, kernel_program_key: &ProgramKey);
    /// Cancel *Unsafe*
    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
}

pub type StoredKernelSystem = Box<dyn KernelSystem>;

    // todo!("then start/end non blocking can use that");
    // todo!("then read only can get blocking for threadpool & runtime (& init)");