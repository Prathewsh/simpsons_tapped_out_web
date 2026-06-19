pub struct InputState {
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub last_click: Option<(f64, f64)>,
    pub keys_held: std::collections::HashSet<String>,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            mouse_x: 0.0,
            mouse_y: 0.0,
            last_click: None,
            keys_held: std::collections::HashSet::new(),
        }
    }

    pub fn consume_click(&mut self) -> Option<(f64, f64)> {
        self.last_click.take()
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
