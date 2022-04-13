use crate::{
    components::{
        image::{img_from_bytes, img_html_from_bytes, img_to_formatted_bytes},
        stl::StlViewer,
    },
    threshold_img_buffer,
};
use image::Luma;
use image::{imageops::resize, DynamicImage, ImageFormat};
use image::ImageBuffer;
use log::info;
use std::rc::Rc;
use yew::{html, Component, Context, Html, Properties};
use yewdux::prelude::*;

use crate::components::slider::Slider;

use super::{utils::SplitColor, store::GlobalState};

use crate::components::constants::IMAGE_WIDTH_PX;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub bytes: Rc<Vec<u8>>,
}

fn scale_proportionate(width_constraint: u32, image: DynamicImage) -> DynamicImage {
    let width = image.width();
    if width <= width_constraint {
        return image;
    }

    let factor = (width_constraint as f64) / (width as f64);
    let new_width = (factor * (image.height() as f64)) as u32;
    resize(
        &image,
        width_constraint,
        new_width,
        image::imageops::FilterType::Nearest,
    )
    .into()
}

pub enum Msg {
    ToggleDisplayStl,
    State(Rc<GlobalState>),
}

pub struct ThresholdImage {
    resized_greyed_image: ImageBuffer<Luma<u8>, Vec<u8>>,
    display_stl: bool,
    dispatch: Dispatch<BasicStore<GlobalState>>,
    state: Option<Rc<GlobalState>>,
}

macro_rules! _timeit {
    ($format_str:expr, $code:expr) => {
        {
            let start = Utc::now();
            let out = $code;
            info!(
                $format_str,
                (Utc::now() - start).num_milliseconds()
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
        let props = ctx.props();
        let full_img =
            img_from_bytes(&props.bytes).expect("this came from an image so this shouldn't fail");
        info!("created image from bytes");
        let resized_greyed_image = scale_proportionate(IMAGE_WIDTH_PX, full_img).into_luma8();
        info!("created resized greyed image");

        let dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            resized_greyed_image,
            display_stl: false,
            dispatch,
            state: Default::default()
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        //info!("Threshold component update");
        match msg {
            Msg::State(state) => {
                self.state = Some(state);
                true
            }
            Msg::ToggleDisplayStl => {
                self.display_stl = !self.display_stl;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //info!("Threshold component view");
                match &self.state {
                    None => html! {
                        <div> { "loading..." } </div>
                    },
                    Some(state) => {
                        let cmap = SplitColor::new(state.threshold_value);
                        let out_img = threshold_img_buffer(self.resized_greyed_image.clone(), cmap);

                        let onchange = self.dispatch.reduce_callback_with(|state: &mut GlobalState, v: f64| state.threshold_value = v as u8);

                        let onclick = ctx.link().callback(|_v| Msg::ToggleDisplayStl);


                        let img_html = img_html_from_bytes(&img_to_formatted_bytes(out_img, ImageFormat::Jpeg));

                        html! {
                            <div>
                                {
                                    img_html
                                }
                                <Slider label="Threshold Value"
                                    min=1.0
                                    max=255.0
                                    step=5.0
                                    onchange={onchange}
                                    value={ state.threshold_value as f64 }

                                />
                                <p> { "Choose a good threshold value" } </p>
                                <button
                                    onclick={onclick}
                                > { "Convert to STL" } </button>
                                if self.display_stl {
                                    <StlViewer
                                        bytes={ Rc::clone(&ctx.props().bytes) }
                                        threshold_value={ state.threshold_value }
                                    />
                                }
                            </div>
                        }
                    }
                }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let full_img = img_from_bytes(&ctx.props().bytes)
            .expect("this came from an image so this shouldn't fail");
        let resized_greyed_image = scale_proportionate(500, full_img).into_luma8();
        self.resized_greyed_image = resized_greyed_image;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::testing::image_from_file;

    #[test]
    fn test_scale_prop() -> Result<(), Box<dyn std::error::Error>> {
        let image = image_from_file("./images/small-wolf.png")?;
        let out = scale_proportionate(100, image);
        assert_eq!(out.width(), 100);
        Ok(())
    }
}
