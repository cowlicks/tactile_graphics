#[derive(Clone)]
pub struct GlobalState {
    pub threshold_value: u8,
    pub stl_height: f64,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            threshold_value: 128,
            stl_height: 2.0,
        }
    }
}
