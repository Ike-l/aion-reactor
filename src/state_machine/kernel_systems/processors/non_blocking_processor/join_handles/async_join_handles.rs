use crate::prelude::{System, SystemId, SystemResult};

#[derive(Default)]
pub struct AsyncJoinHandles(Vec<(SystemId, tokio::task::JoinHandle<(System, Option<SystemResult>)>)>);

impl AsyncJoinHandles {
    pub fn push(&mut self, id: SystemId, join_handle: tokio::task::JoinHandle<(System, Option<SystemResult>)>) {
        self.0.push((id, join_handle));
    }

    pub async fn get_finished(&mut self) -> Vec<(SystemId, Result<(System, Option<SystemResult>), tokio::task::JoinError>)> {
        let mut not_finished = Vec::new();
        let mut finished = Vec::new();
        for (id, handle) in self.0.drain(..) {
            if handle.is_finished() {
                let result = handle.await;
                finished.push((id, result));
            } else {
                not_finished.push((id, handle));
            }
        }

        self.0.extend(not_finished);

        return finished
    }
}