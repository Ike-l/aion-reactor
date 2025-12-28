mod runs_one;
mod fixed_ordering;
mod async_works;

/*

tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        // .compact()
        // .pretty()
        // .with_env_filter(EnvFilter::new("info,aion_reactor=debug"))
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .with_target(false)
        .with_test_writer()
        .init();

*/

// metadata:
// -- ordering (fixed_ordering, &)
// -- criteria
// working
// -- none accesses (runs_one)
// -- some accesses (runs_one)
// async
// -- working
// results
// -- enum SystemResult
// requirements
// -- resources
// -- accesses
// other
// -- blockers