pub mod event;
pub mod next_events;
pub mod current_events;
pub mod event_manager;
pub mod event_mapper;

pub mod prelude {
    pub use super::{
        event::Event, next_events::NextEvents, current_events::CurrentEvents, event_manager::EventManager, event_mapper::EventMapper
    };
}

