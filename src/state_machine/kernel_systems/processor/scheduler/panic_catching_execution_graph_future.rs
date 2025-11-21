use std::{sync::atomic::Ordering, task::Poll};

use threadpool::ThreadPool;

use crate::{id::Id, state_machine::kernel_systems::processor::scheduler::panic_catching_execution_graph::PanicCatchingExecutionGraphs};

pub struct PanicCatchingExecutionGraphsFuture<'a> {
    threadpool: &'a ThreadPool,
    execution_graphs: &'a PanicCatchingExecutionGraphs<Id>
}

impl<'a> PanicCatchingExecutionGraphsFuture<'a> {
    pub fn new(threadpool: &'a ThreadPool, execution_graphs: &'a PanicCatchingExecutionGraphs<Id>) -> Self {
        Self {
            threadpool,
            execution_graphs
        }
    }
}

impl<'a> Future for PanicCatchingExecutionGraphsFuture<'a> {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.execution_graphs.panicked_signal.load(Ordering::Relaxed) {
            panic!("A Scheduler thread has panicked. Choosing to panic the main thread");
        } else {
            if self.execution_graphs.drop_signals.load(Ordering::Relaxed) == 0 {
                self.threadpool.join();      
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }          
    }
}