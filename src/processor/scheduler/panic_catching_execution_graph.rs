use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc};

use crate::processor::scheduler::execution_graph::ExecutionGraph;

pub struct PanicCatchingExecutionGraphs<T> {
    pub drop_signals: Arc<AtomicUsize>,
    pub panicked_signal: Arc<AtomicBool>,
    pub graphs: Arc<Vec<tokio::sync::RwLock<ExecutionGraph<T>>>>
}

impl<T> PanicCatchingExecutionGraphs<T> {
    pub fn new(graphs: Arc<Vec<tokio::sync::RwLock<ExecutionGraph<T>>>>) -> Self {
        Self {
            drop_signals: Arc::new(AtomicUsize::new(graphs.len())),
            panicked_signal: Arc::new(AtomicBool::new(false)),
            graphs
        }
    }

    pub fn arc_clone(&self) -> Self {
        Self {
            drop_signals: Arc::clone(&self.drop_signals),
            panicked_signal: Arc::clone(&self.panicked_signal),
            graphs: Arc::clone(&self.graphs)
        }
    }
}

impl<T> Drop for PanicCatchingExecutionGraphs<T> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // probs dont need SeqCst
            self.panicked_signal.store(true, Ordering::SeqCst);
        } else {
            self.drop_signals.fetch_sub(1, Ordering::SeqCst);
        }
    }
}