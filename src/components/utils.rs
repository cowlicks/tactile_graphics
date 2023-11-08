use image::Luma;
use image::imageops::ColorMap;
use photon_rs::transform::resize;
use photon_rs::monochrome::threshold;
use photon_rs::PhotonImage;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use crate::components::constants::VIEW_WIDTH_PX;
use super::external::canvas_from_image;
use super::image::data_url_from_img_bytes;

use log::info;

use wasm_bindgen::JsCast;


/// Faster than reading bytes to with photon_rs::native::open_image. but async
pub async fn photon_image_from(data: &Vec<u8>) -> PhotonImage {
    let durl = data_url_from_img_bytes(data).unwrap();
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

pub fn context_from_canvas(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
        canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap()
}
pub fn threshold_canvas(
    canvas: HtmlCanvasElement,
    value: u32,
    ) {
    wasm_bindgen_futures::spawn_local(async move {
        info!("from draw_data_to_canvas");
        let ctx = context_from_canvas(&canvas);

        let mut new_image = timeit!(
            "reading canvas into photon image took {}",
            photon_rs::open_image(canvas.clone(), ctx.clone())
        );
        timeit!(
            "threshold took {}",
            threshold(
                &mut new_image,
                value,
            )
        );

        timeit!(
            "photon_rs::putImageData threshold took {}",
            photon_rs::putImageData(canvas, ctx, new_image)
            );
    });
}

pub fn maybe_resize_photon_image(img: PhotonImage) -> PhotonImage {
        if img.get_width() > VIEW_WIDTH_PX {
            let scale = (VIEW_WIDTH_PX as f64) / (img.get_width() as f64);

            let new_width = (scale * img.get_width() as f64) as u32;
            let new_height = (scale * img.get_height() as f64) as u32;

            let img = timeit!(
                "resize took {}",
                resize(
                    &img,
                    new_width,
                    new_height,
                    photon_rs::transform::SamplingFilter::Nearest,
                ));
            img
        } else {
            img
        }
}

pub fn draw_data_to_canvas(
    canvas: HtmlCanvasElement,
    data: Vec<u8>,
    on_complete: (impl FnOnce(PhotonImage) + 'static)
) {
    canvas.set_width(500);
    wasm_bindgen_futures::spawn_local(async move {
        info!("from draw_data_to_canvas");
        let new_image = timeit!(
            "photon_image_from took {}", {
            photon_image_from(&data.to_vec()).await
            }
        );

        //let new_image = if new_image.get_width() > VIEW_WIDTH_PX {

        //let scale = (VIEW_WIDTH_PX as f64) / (new_image.get_width() as f64);

        //let new_width = (scale * new_image.get_width() as f64) as u32;
        //let new_height = (scale * new_image.get_height() as f64) as u32;

        //let new_image = timeit!(
        //    "resize took {}",
        //    resize(
        //        &new_image,
        //        new_width,
        //        new_height,
        //        photon_rs::transform::SamplingFilter::Nearest,
        //    ));
        //    new_image
        //} else {
        //    new_image
        //}

        canvas.set_width(new_image.get_width());
        canvas.set_height(new_image.get_height());

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();


        timeit!(
            "photon_rs::putImageData took {}",
            photon_rs::putImageData(canvas, ctx, new_image.clone())
            );
        on_complete(new_image);
    });

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


