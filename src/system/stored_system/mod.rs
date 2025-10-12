use crate::{id::Id, memory::ResourceId, processor::scheduler::ordering::ScheduleOrdering, system::stored_system::{criteria::Criteria, system_kind::SystemKind}};


pub mod system_kind;
pub mod criteria;

#[derive(Debug)]
pub struct StoredSystem {
    system_kind: SystemKind,
    resource_id: ResourceId,
    program_id: Option<Id>,
    criteria: Criteria,
    ordering: ScheduleOrdering,
}