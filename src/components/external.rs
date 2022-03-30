use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/wrappers.js")]
extern "C" {
    pub fn STLViewer(file: &web_sys::File, id: &str);
    pub fn insert_canvas(durl: &str, canvas_id: &str);
    pub async fn canvas_from_image(durl: &str) -> JsValue;
}
