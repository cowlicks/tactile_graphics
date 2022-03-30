use crate::{
    components::{
        image::{img_from_bytes, img_html_from_bytes, img_to_formatted_bytes},
        stl::StlViewer,
    },
    threshold_img_buffer,
};
use image::Luma;
use image::{imageops::resize, DynamicImage, ImageFormat};
use image::{imageops::ColorMap, ImageBuffer};
use log::info;
use std::rc::Rc;
use yew::{html, Component, Context, Html, Properties};

use crate::components::slider::Slider;

use chrono::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub bytes: Rc<Vec<u8>>,
}

#[derive(Clone, Copy)]
pub struct SplitColor {
    value: u8,
}

// copied from the definition of image::imageops::colorops::BiLevel
// https://docs.rs/image/0.24.1/src/image/imageops/colorops.rs.html#412
impl SplitColor {
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}
impl ColorMap for SplitColor {
    type Color = Luma<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Luma<u8>) -> usize {
        let luma = color.0;
        if luma[0] > self.value {
            1
        } else {
            0
        }
    }

    #[inline(always)]
    fn lookup(&self, idx: usize) -> Option<Self::Color> {
        match idx {
            0 => Some([0].into()),
            1 => Some([255].into()),
            _ => None,
        }
    }

    /// Indicate NeuQuant implements `lookup`.
    fn has_lookup(&self) -> bool {
        true
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Luma<u8>) {
        let new_color = 0xFF * self.index_of(color) as u8;
        let luma = &mut color.0;
        luma[0] = new_color;
    }
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
    NewThresholdValu(u8),
    ToggleDisplayStl,
}

pub struct ThresholdImage {
    resized_greyed_image: ImageBuffer<Luma<u8>, Vec<u8>>,
    threshold_value: u8,
    display_stl: bool,
}

use crate::components::constants::IMAGE_WIDTH_PX;

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

        Self {
            resized_greyed_image,
            threshold_value: 128,
            display_stl: false,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        //info!("Threshold component update");
        match msg {
            Msg::NewThresholdValu(threshold_value) => {
                self.threshold_value = threshold_value;
                true
            }
            Msg::ToggleDisplayStl => {
                self.display_stl = !self.display_stl;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        info!("Threshold component view");
        let cmap = SplitColor::new(self.threshold_value);
        let out_img = threshold_img_buffer(self.resized_greyed_image.clone(), cmap);
        let onchange = ctx.link().callback(|v| Msg::NewThresholdValu(v as u8));

        let onclick = ctx.link().callback(|_v| Msg::ToggleDisplayStl);

        let start = Utc::now();

        let img_html = img_html_from_bytes(&img_to_formatted_bytes(out_img, ImageFormat::Jpeg));

        info!(
            "threshold view took {:?}",
            (Utc::now() - start).num_milliseconds()
        );

        let out = html! {
            <div>
                {
                    img_html
                }
                <Slider label="Threshold Value"
                    min=1.0
                    max=255.0
                    step=5.0
                    onchange={onchange}
                    value={ self.threshold_value as f64 }
                />
                <p> { "Choose a good threshold value" } </p>
                <button
                    onclick={onclick}
                > { "Convert to STL" } </button>
                if self.display_stl {
                    <StlViewer
                        bytes={ Rc::clone(&ctx.props().bytes) }
                        threshold_value={ self.threshold_value as u8 }
                    />
                }
            </div>
        };
        out
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        //info!(" from changed!");
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
