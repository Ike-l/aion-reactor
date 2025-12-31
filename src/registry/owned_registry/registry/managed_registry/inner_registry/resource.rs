use crate::registry::owned_registry::{reception::host::access_map::access::Access, registry_result::GetResult};

pub trait Resource {
    fn get<T: 'static, A: Access>(&self, access: &A) -> Option<GetResult<'_, T>>;
}