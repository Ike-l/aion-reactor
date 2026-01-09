#[derive(Debug, PartialEq, Clone)]
pub struct StoredResource(Resource);

impl StoredResource {
    pub fn new(resource: Resource) -> Self {
        Self(resource)
    }

    pub fn get(&self) -> &Resource {
        &self.0
    }

    pub fn consume(self) -> Resource {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Resource(i32);

impl Resource {
    pub fn new(resource: i32) -> Self {
        Self(resource)
    }

    pub fn get(&self) -> &i32 {
        &self.0
    }
}