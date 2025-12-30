
use crate::{Memory, prelude::{RawSegmentMap}};

pub mod raw_segment_map;

#[derive(Default)]
pub struct SegmentMap<M: Memory> {
    guard: parking_lot::RwLock<()>,
    raw_segment_map: RawSegmentMap<M>,
}
