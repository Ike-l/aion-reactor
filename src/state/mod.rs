use crate::{Memory, prelude::SafeSegmentMap};

pub mod safe_segment_map;

#[derive(Default)]
pub struct State<M: Memory> {
    segment_map: SafeSegmentMap<M>
}
