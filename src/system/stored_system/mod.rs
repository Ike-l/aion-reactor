use crate::{id::Id, memory::access_checked_resource_map::resource::ResourceId, state_machine::scheduler::ordering::ScheduleOrdering, system::stored_system::{criteria::Criteria, system_kind::SystemKind}};


pub mod system_kind;
pub mod criteria;

#[derive(Debug)]
pub struct StoredSystem {
    system_kind: SystemKind,
    resource_id: ResourceId,
    program_id: Id,
    criteria: Criteria,
    ordering: ScheduleOrdering,
}