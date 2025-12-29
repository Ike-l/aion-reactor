#[derive(Debug, Default)]
pub struct TickAccumulator(usize);

impl TickAccumulator {
    pub fn inner_mut(&mut self) -> &mut usize {
        &mut self.0
    }

    pub fn inner(&self) -> &usize {
        &self.0
    }
}
