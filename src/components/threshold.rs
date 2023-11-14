use crate::components::{
    number_input::NumberInput, slider::Slider, utils::maybe_resize_photon_image,
};
use log::info;
use photon_rs::{monochrome::threshold, PhotonImage};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use yew::{html, Component, Context, Html, NodeRef, Properties};
use yewdux::prelude::*;

use super::{
    store::GlobalState,
    utils::{context_from_canvas, photon_image_from},
};

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
    ($format_str:expr, $code:expr) => {{
        let start = chrono::Utc::now();
        let out = $code;
        info!($format_str, (chrono::Utc::now() - start).num_milliseconds());
        out
    }};
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
        let bytes = ctx.props().bytes.clone();
        let link = ctx.link().clone();
        spawn_local(async move {
            let photon_image = photon_image_from(&bytes).await;

            // TODO should i remove this?
            let photon_image = maybe_resize_photon_image(photon_image);
            link.send_message(Msg::PhotonImageReady(photon_image));
        });
        self.canvas_loaded = false;
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        //info!("Threshold component update");
        match msg {
            Msg::State(state) => {
                if self.last_threshold_value != Some(state.threshold_value) {
                    info!(
                        "Threshold value changed from {:?} to {:?}",
                        self.last_threshold_value, state.threshold_value
                    );
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

    // yew 0.19's `html!` expands `<Slider label="..." />` in a way that trips this lint.
    #[allow(clippy::unnecessary_operation)]
    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.state {
            None => html! {
                <div> { "loading..." } </div>
            },
            Some(state) => {
                let onchange =
                    self.dispatch
                        .reduce_callback_with(move |state: &mut GlobalState, v: f64| {
                            state.threshold_value = v as u8;
                        });

                let height_onchange = self
                    .dispatch
                    .reduce_callback_with(|state: &mut GlobalState, v: f64| state.stl_height = v);
                let onclick = self
                    .dispatch
                    .reduce_callback_with(|state: &mut GlobalState, _v| {
                        state.display_stl = !state.display_stl
                    });
                html! {
                    <div>
                        <canvas ref={ self.canvas_ref.clone() } />
                        <Slider label="Threshold Value"
                            min=1.0
                            max=255.0
                            step=1.0
                            onchange={onchange}
                            value={ state.threshold_value as f64 }

                        />
                        <NumberInput label="Stl height"
                            min=0.0
                            max=200.0
                            onchange={height_onchange}
                            value={ state.stl_height }
                        />
                        <p> { "Choose a good threshold value" } </p>
                        <button
                            class={ "convert-to-stl" }
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
