#[derive(Default)]
pub struct World(hecs::World);

impl World {
    pub fn new_hecs(world: hecs::World) -> Self {
        Self(world)
    }

    pub fn get_hecs(&self) -> Option<&hecs::World> {
        Some(&self.0)
    }

    pub fn get_mut_hecs(&mut self) -> Option<&mut hecs::World> {
        Some(&mut self.0)
    }
}
