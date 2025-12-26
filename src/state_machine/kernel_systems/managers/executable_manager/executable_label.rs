#[derive(Debug)]
pub struct ExecutableLabel {
    label: String   
}

impl ExecutableLabel{
    pub fn new(label: String) -> Self {
        Self { label }
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }
}