use crate::setup::{KeyId, ReserverId, ResourceId, StoredResource};

pub struct Generator;

impl Generator {
    pub fn bert() -> (Option<ReserverId>, Option<KeyId>, ResourceId) {
        (None, None, ResourceId::Labelled("bert".to_string()))
    }

    pub fn barry(number: i32) -> (Option<ReserverId>, Option<KeyId>, ResourceId, StoredResource) {
        (None, None, ResourceId::Labelled("barry".to_string()), StoredResource(number))
    }
}