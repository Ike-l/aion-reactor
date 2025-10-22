use crate::{id::Id, system::System};

#[derive(Default)]
pub struct AsyncJoinHandles(Vec<(Id, tokio::task::JoinHandle<System>)>);

impl AsyncJoinHandles {
    pub fn push(&mut self, id: Id, join_handle: tokio::task::JoinHandle<System>) {
        self.0.push((id, join_handle));
    }

    pub async fn get_finished(&mut self) -> Vec<(Id, Result<System, tokio::task::JoinError>)> {
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