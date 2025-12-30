use crate::{Memory, prelude::{Reception, SegmentMap}};

pub mod reception;
pub mod segment_map;

#[derive(Default)]
pub struct SafeSegmentMap<M: Memory> {
    segment_map: SegmentMap<M>,
    reception: Reception<M>
}