pub mod registry;

pub mod prelude;
pub mod resource;
pub mod access_result;

use tracing_subscriber::fmt;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        fmt()
            // .with_ansi(false)
            // .compact()
            // .pretty()
            // .with_env_filter(EnvFilter::new("info,aion_reactor=debug"))
            .with_max_level(tracing::Level::TRACE)
            // .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT)
            // .with_c
            .with_target(false)
            .with_test_writer()           
            .init();
    });
}