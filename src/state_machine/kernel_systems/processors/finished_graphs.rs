use std::sync::atomic::{AtomicBool, Ordering};

pub struct FinishedGraphTracker {
    graphs: Vec<AtomicBool>
}

impl FinishedGraphTracker {
    pub fn new(graph_count: usize) -> Self {
        Self {
            graphs: (0..graph_count).map(|_| AtomicBool::new(false)).collect::<Vec<_>>()
        }
    }

    /// 0 if all graphs are `true`
    pub fn load(&self, ordering: Ordering) -> usize {
        self.graphs.iter().fold(0, |acc, graph_status| {
            if !graph_status.load(ordering) {
                acc + 1
            } else {
                acc
            }
        })
    }

    // returns the previous value (& false if graph_index doesnt exist)
    pub fn complete(&self, graph_index: usize) -> bool {
        if let Some(graph_status) = self.graphs.get(graph_index) {
            return graph_status.fetch_or(true, Ordering::Release)
        }

        false
    }
}