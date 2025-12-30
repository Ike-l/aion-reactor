use crate::{Memory, prelude::{Guard, Host}};

pub mod guard;
pub mod host;

#[derive(Default)]
pub struct Reception<M: Memory> {
    guard: Guard<M>,
    host: Host<M>
}