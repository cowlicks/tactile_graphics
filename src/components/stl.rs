use crate::{stl::triangles_to_stl_writer, triangle::image_to_stl};
use log::info;
use std::io::BufWriter;
use std::rc::Rc;
use yew::{function_component, html, use_effect, Properties};

use yewdux::prelude::*;
use yewdux_functional::use_store;


use super::{
    external::STLViewer,
    image::img_from_bytes, store::GlobalState,
};

#[derive(Properties, PartialEq)]
pub struct StlViewerProps {
    pub bytes: Rc<Vec<u8>>,
    pub threshold_value: u8,
}

#[function_component(StlViewer)]
pub fn stl_viewer(props: &StlViewerProps) -> Html {

    let store = use_store::<BasicStore<GlobalState>>();

    let state = if let Some(s) = store.state() {
        s
    } else {
        return html! { <div> { "Could not get state" } </div> };
    };

    let image = img_from_bytes(&props.bytes).unwrap();

    let triangles = image_to_stl(
        image,
        props.threshold_value,
        state.stl_height,
        state.stl_scale_factor,
    )
    .unwrap()
    .collect();

    let b: Vec<u8> = vec![];
    let mut writer = BufWriter::new(b);
    triangles_to_stl_writer(&mut writer, triangles).unwrap();

    let inner = writer.into_inner().unwrap();

    let g_file = gloo_file::File::new("test.stl", inner.as_slice());
    use_effect(move || {
        STLViewer(g_file.as_ref(), "stl-cont");
        || info!("from destructor")
    });

    html! {
        <p>
            <b>{ "STL view" }</b>
            <p> { format!("threshold value: {}", &props.threshold_value) } </p>
            <p> { format!("num bytes: {}", &props.bytes.len()) } </p>
            <p> { format!("buffer size: {}", inner.len()) } </p>
            <a id={ "download-button" }
                target={ "_blank" }
                download={ "test.stl" }
                href={ "" }
            >
                <button>{ "Download STL" }</button>
            </a>
            <div
                id={"stl-cont"}
                style="width: 500px; height: 500px"
            >
             <h1> { "loading..." }</h1>
            </div>
        </p>
    }
}
