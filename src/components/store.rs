use super::constants::{DEFAULT_STL_HEIGHT, DEFAULT_THRESHOLD_VALUE, DEFAULT_SCALE_FACTOR};

#[derive(Clone)]
pub struct GlobalState {
    pub threshold_value: u8,
    pub stl_height: f64,
    pub stl_scale_factor: f64,
    pub display_stl: bool,
    pub file_name: Option<String>,
    // TODO
    // unit is mm
    // pixels per unit length?
    //pub pixels_per_unit_length: f64,
    //pub pixels_per_unit_width: f64,
    // pixels per unit width?
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            threshold_value: DEFAULT_THRESHOLD_VALUE,
            stl_height: DEFAULT_STL_HEIGHT,
            stl_scale_factor: DEFAULT_SCALE_FACTOR,
            display_stl: false,
            file_name: None,
        }
    }
}
