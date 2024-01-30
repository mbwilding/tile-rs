use crate::csharp::screen::Screen;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Monitor {
    pub index: usize,
    pub screen: Screen,
}

// TODO: Reduce calls for `working_area`
impl Monitor {
    pub fn new(index: usize, screen: Screen) -> Self {
        Self { index, screen }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.screen.device_name
    }

    #[allow(dead_code)]
    pub fn width(&self) -> i32 {
        self.screen.working_area().width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> i32 {
        self.screen.working_area().height
    }

    #[allow(dead_code)]
    pub fn x(&self) -> i32 {
        self.screen.working_area().x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> i32 {
        self.screen.working_area().y
    }
}
