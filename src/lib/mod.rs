pub mod environment;
pub mod error;
pub mod identity;
pub mod nns_types;
pub mod sign;

#[derive(Clone, Debug)]
pub struct NetworkDescriptor {
    pub name: String,
    pub providers: Vec<String>,
    pub is_ic: bool,
}
