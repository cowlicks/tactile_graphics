use super::constants::{DEFAULT_STL_HEIGHT, DEFAULT_THRESHOLD_VALUE, DEFAULT_SCALE_FACTOR};

#[derive(Clone)]
pub struct GlobalState {
    pub threshold_value: u8,
    pub stl_height: f64,
    pub stl_scale_factor: f64,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            threshold_value: DEFAULT_THRESHOLD_VALUE,
            stl_height: DEFAULT_STL_HEIGHT,
            stl_scale_factor: DEFAULT_SCALE_FACTOR,
        }
    }
}
