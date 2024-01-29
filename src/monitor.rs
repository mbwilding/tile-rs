use crate::screen::Screen;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Monitor {
    pub index: u32,
    pub screen: Screen,
}

// TODO: Reduce calls for `working_area`
impl Monitor {
    pub fn new(index: u32, screen: Screen) -> Self {
        Self { index, screen }
    }

    pub fn name(&self) -> &str {
        &self.screen.device_name
    }

    pub fn width(&self) -> i32 {
        self.screen.working_area().width
    }

    pub fn height(&self) -> i32 {
        self.screen.working_area().height
    }

    pub fn x(&self) -> i32 {
        self.screen.working_area().x
    }

    pub fn y(&self) -> i32 {
        self.screen.working_area().y
    }
}
