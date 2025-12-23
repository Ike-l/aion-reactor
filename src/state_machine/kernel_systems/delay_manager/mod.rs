pub mod delay_registry;
pub mod delay_buffer;
pub mod delay;
pub mod delay_manager;

// Regular use-case:
// RegisteredDelay { activated_by: Start, then_inserts: End, delayed_by: During }
// Have Event `Start` sent Once
// Have Event `During` sent Continuously 
// When `During` finishes, insert `End`

// Edge use-cases:
// If no `During` or `delayed_by` is none: Acts as Map(`Start` -> `End`)
// If `activated_by` == `delayed_by`: If Start only once then: Acts as Wait(1), Map(`Start` -> `End`)
// If `activated_by` == `then_inserts`: Acts as continuous chain. `delayed_by` breaks the chain
// If `then_inserts` == `delayed_by`: Can be used to prevent 2 `then_inserts` in a row, if there are 2 `activated_by`

pub mod prelude {
    pub use super::{
        delay_registry::DelayRegistry,
        delay_buffer::DelayBuffer,
        delay_manager::DelayManager,
        delay::{
            Delay, 
            registered_delay::RegisteredDelay
        }
    };
}
