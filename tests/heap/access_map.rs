use aion_reactor::prelude::AccessMap as AccessMapTrait;

#[derive(Default)]
pub struct AccessMap;

pub struct Access;

impl AccessMapTrait for AccessMap {
    type Access = Access;
}