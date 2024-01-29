#[derive(Debug)]
pub struct Monitor {
    pub index: u32,
}

impl Monitor {
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}
