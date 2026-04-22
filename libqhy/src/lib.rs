pub mod raw;
pub mod types;

pub struct QhyCcd {
    id: String,
}

impl QhyCcd {
    fn new(id: String) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub fn init_sdk() -> Vec<QhyCcd> {
    vec![]
}
