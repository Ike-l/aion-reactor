#![allow(dead_code)]

pub mod state;
pub mod memory;
pub mod resolver;

pub mod prelude {
    pub use super::{
        state::{
            State,
            safe_segment_map::{
                SafeSegmentMap,
                reception::{
                    Reception,
                    guard::{
                        Guard
                    },
                    host::{
                        Host
                    }
                },
                segment_map::{
                    SegmentMap,
                    raw_segment_map::{
                        RawSegmentMap,
                        inner_segment_map::{
                            InnerSegmentMap
                        }
                    }
                }
            }
        },
        memory::{
            Memory,
            access_map::{
                AccessMap
            }
        },
        resolver::{
            Resolver,
            access_drop::{
                AccessDrop
            }
        }
    };
}

pub use prelude::{
    Memory, State, Resolver
};