use std::cell::UnsafeCell;

use crate::{Memory, prelude::InnerSegmentMap};

pub mod inner_segment_map;

#[derive(Default)]
pub struct RawSegmentMap<M: Memory> {
    segments: UnsafeCell<InnerSegmentMap<M>>
}
