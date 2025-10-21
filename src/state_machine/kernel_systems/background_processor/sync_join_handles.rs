use std::{any::Any, thread::JoinHandle};

use crate::{id::Id, system::System};

pub struct SyncJoinHandles(Vec<(Id, JoinHandle<System>)>);

impl SyncJoinHandles {
    pub fn push(&mut self, id: Id, join_handle: JoinHandle<System>) {
        self.0.push((id, join_handle));
    }

    pub fn get_finished(&mut self) -> Vec<(Id, Result<System, Box<dyn Any + Send + 'static>>)> {
        let mut not_finished = Vec::new();
        let mut finished = Vec::new();
        for (id, handle) in self.0.drain(..) {
            if handle.is_finished() {
                let result = handle.join();
                finished.push((id, result));
            } else {
                not_finished.push((id, handle));
            }
        }

        self.0.extend(not_finished);

        return finished
    }
}