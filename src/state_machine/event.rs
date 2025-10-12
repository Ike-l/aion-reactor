use crate::id::Id;

// when finding a system
pub struct Event {
    id: Id
}

impl Event {
    pub fn id(&self) -> &Id {
        &self.id
    }
}

pub struct NextEvents {

}

pub struct CurrentEvents {

}