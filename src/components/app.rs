use photon_rs::transform::resize;
use photon_rs::PhotonImage;
use std::rc::Rc;
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlInputElement};
use yew::NodeRef;
use yew::{html, html::TargetCast, Component, Context, Html};

use gloo_file::callbacks::FileReader;
use gloo_file::File;
use log::info;

use crate::components::constants::VIEW_WIDTH_PX;
use crate::components::image::img_html_from_bytes;
use crate::components::threshold::ThresholdImage;

use super::external::canvas_from_image;
use super::image::data_url_from_img_bytes;

use wasm_bindgen::JsCast;

/// Faster than reading bytes to with photon_rs::native::open_image. but async
async fn photon_image_from(data: Vec<u8>) -> PhotonImage {
    let durl = data_url_from_img_bytes(&data).unwrap();
    let canvas = canvas_from_image(&durl)
        .await
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    info!("unwraped the context");

    photon_rs::open_image(canvas, ctx)
}

pub fn draw_data_to_canvas(
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    data: Vec<u8>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let new_image = photon_image_from(data.to_vec()).await;

        let scale = (VIEW_WIDTH_PX as f64) / (new_image.get_width() as f64);

        let new_width = (scale * new_image.get_width() as f64) as u32;
        let new_height = (scale * new_image.get_height() as f64) as u32;

        let new_image = resize(
            &new_image,
            new_width,
            new_height,
            photon_rs::transform::SamplingFilter::Nearest,
        );

        canvas.set_width(new_image.get_width());
        canvas.set_height(new_image.get_height());

        photon_rs::putImageData(canvas, ctx, new_image)
    });
}

pub enum Msg {
    LoadedBytes(String, Vec<u8>),
    Files(Vec<File>),
}

pub struct App {
    reader: Option<FileReader>,
    file_name: Option<String>,
    file_bytes: Option<Rc<Vec<u8>>>,
    node_ref: NodeRef,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            reader: None,
            file_name: None,
            file_bytes: None,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadedBytes(file_name, data) => {
                info!("Loaded bytes");
                self.file_bytes = Some(Rc::from(data));
                self.file_name = Some(file_name);
                self.reader = None;
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let link = ctx.link().clone();

                        info!("Got filename = {}", file_name);
                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::LoadedBytes(
                                file_name,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.reader = Some(task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div>
                    <input type="file"
                        accept="image/png, image/jpeg"
                        multiple=false
                        onchange={ctx.link().callback(move |e: Event| {
                            let mut result = Vec::new();
                            let input: HtmlInputElement = e.target_unchecked_into();

                            if let Some(files) = input.files() {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .map(|v| web_sys::File::from(v.unwrap()))
                                    .map(File::from);
                                result.extend(files);
                            }
                            Msg::Files(result)
                        })}
                    />
                </div>
                { self.view_file() }
                <canvas
                    ref={ self.node_ref.clone() } />
            </div>
        }
    }
}

impl App {
    fn view_file(&self) -> Html {
        if let Some(data) = &self.file_bytes {
            html! {
                <div class="image-container">
                    <div class="images">
                        <div class="cell">
                            { img_html_from_bytes(data) }
                        </div>
                        <div class="cell">
                            <ThresholdImage
                                bytes={data}
                                />
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
                <p>{ "Choose a file to convert to stl" }</p>
            }
        }
    }
}
