#[derive(Debug, Clone, Copy)]
pub struct PointerState {
    pub pos: Option<glam::Vec2>,
    pub secondary_down: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyboardState {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub q: bool,
    pub e: bool,
}
