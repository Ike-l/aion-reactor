use crate::Memory;

#[derive(Default)]
pub struct Guard<M: Memory> {
    access_map: parking_lot::Mutex<M::AccessMap>
}