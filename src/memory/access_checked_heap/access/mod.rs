pub mod access_map;

#[derive(Debug)]
pub enum Access {
    Unique,
    Shared(usize)
}