pub mod background_processor_system_registry;
pub mod join_handles;
pub mod processors;

pub mod prelude {
    pub use super::{
        processors::{
            finish_background_processor::FinishBackgroundProcessor,
            start_background_processor::StartBackgroundProcessor
        },
        join_handles::{
            async_join_handles::AsyncJoinHandles,
            sync_join_handles::SyncJoinHandles,
        },
        background_processor_system_registry::BackgroundProcessorSystemRegistry
    };
}