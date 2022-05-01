use crate::components::{
        number_input::NumberInput,
        slider::Slider, utils::maybe_resize_photon_image,
    };
use log::info;
use web_sys::HtmlCanvasElement;
use std::rc::Rc;
use yew::{html, Component, Context, Html, Properties, NodeRef};
use yewdux::prelude::*;
use wasm_bindgen_futures::spawn_local;
use photon_rs::{PhotonImage, monochrome::threshold};

use super::{utils::{photon_image_from, context_from_canvas}, store::GlobalState};


#[derive(Properties, PartialEq)]
pub struct Props {
    pub bytes: Rc<Vec<u8>>,
}

pub enum Msg {
    State(Rc<GlobalState>),
    PhotonImageReady(PhotonImage),
    UpdateImage,
}

pub struct ThresholdImage {
    canvas_ref: NodeRef,
    canvas_loaded: bool,
    dispatch: Dispatch<BasicStore<GlobalState>>,
    state: Option<Rc<GlobalState>>,
    last_threshold_value: Option<u8>,
    photon_image: Option<PhotonImage>,
}

macro_rules! timeit {
    ($format_str:expr, $code:expr) => {
        {
            let start = chrono::Utc::now();
            let out = $code;
            info!(
                $format_str,
                (chrono::Utc::now() - start).num_milliseconds()
            );
            out
        }
    };
}

impl Component for ThresholdImage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        info!("creating thresholded image");
        let dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        let bytes = ctx.props().bytes.clone();
        let link = ctx.link().clone();
        spawn_local(async move {
            let photon_image = photon_image_from(&bytes).await;

            // TODO should i remove this?
            let photon_image = maybe_resize_photon_image(photon_image);
            link.send_message(Msg::PhotonImageReady(photon_image));
        });
        Self {
            canvas_loaded: false,
            dispatch,
            state: Default::default(),
            canvas_ref: NodeRef::default(),
            last_threshold_value: None,
            photon_image: None,
        }
    }
    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        info!(" from threshold changed !!!!!!!!!!!!!!!!!!!!!!!");
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        //info!("Threshold component update");
        match msg {
            Msg::State(state) => {
                    if self.last_threshold_value != Some(state.threshold_value) {
                        info!("Threshold value changed from {:?} to {:?}", self.last_threshold_value, state.threshold_value);
                        self.last_threshold_value = Some(state.threshold_value);
                        ctx.link().send_message(Msg::UpdateImage);
                    }
                self.state = Some(state);
                true
            }
            Msg::PhotonImageReady(value) => {
                info!("Photon image ready");
                self.photon_image = Some(value);
                ctx.link().send_message(Msg::UpdateImage);
                false
            }
            Msg::UpdateImage => {
                if let (Some(state), Some(photon_image)) = (&self.state, &self.photon_image) {
                    if self.canvas_loaded {
                        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
                        canvas.set_width(photon_image.get_width());
                        canvas.set_height(photon_image.get_height());
                        let ctx = context_from_canvas(&canvas);

                        let mut img = photon_image.clone();
                        info!("threshold value = {}", state.threshold_value);
                        timeit!(
                        "thresholding image took {}",
                        threshold(&mut img, state.threshold_value as u32)
                        );

                        photon_rs::putImageData(canvas, ctx, img);
                    }
                }
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.state {
            None => html! {
                <div> { "loading..." } </div>
            },
            Some(state) => {
                let onchange = self.dispatch.reduce_callback_with(move |state: &mut GlobalState, v: f64| {
                    state.threshold_value = v as u8;
                });

                let height_onchange = self.dispatch.reduce_callback_with(|state: &mut GlobalState, v: f64| state.stl_height = v);
                let onclick = self.dispatch.reduce_callback_with(|state: &mut GlobalState, _v| state.display_stl = !state.display_stl);
                html! {
                    <div>
                        <canvas ref={ self.canvas_ref.clone() } />
                        <Slider label="Threshold Value"
                            min=1.0
                            max=255.0
                            step=5.0
                            onchange={onchange}
                            value={ state.threshold_value as f64 }

                        />
                        <NumberInput label="Stl height"
                            min=0.0
                            max=200.0
                            onchange={height_onchange}
                            value={ state.stl_height as f64 }
                        />
                        <p> { "Choose a good threshold value" } </p>
                        <button
                            onclick={onclick}
                        > { "Convert to STL" } </button>
                    </div>
                }
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if !self.canvas_loaded {
            ctx.link().send_message(Msg::UpdateImage);
            self.canvas_loaded = true;
        }
    }

}
