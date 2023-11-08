#![feature(extract_if, const_float_bits_conv, test)]

pub mod components;
pub mod earcut;
pub mod edge;
pub mod edge_collection;
pub mod json;
pub mod ribbon;
pub mod stl;
pub mod tree;
pub mod triangle;
pub mod util;
pub mod vert;

use std::error::Error;

use components::utils::SplitColor;
use image::error::{LimitError, LimitErrorKind};
use image::imageops::ColorMap;
use image::io::Reader as ImageReader;
use image::{
    DynamicImage, GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageError, Luma, Rgba,
};

use vert::{get_quad_edge, QuadCase};
use edge_collection::Edges;

pub fn rgb_to_greyscale(p: Rgba<u8>) -> Rgba<u8> {
    let x: u8 = (0.2126 * (p[0] as f64) + 0.7152 * (p[1] as f64) + 0.0722 * (p[2] as f64)) as u8;
    Rgba([x, x, x, 255])
}

// TODO add option to invert the threshold
pub fn rgb_to_threshold(p: Rgba<u8>, threshold: u8) -> Rgba<u8> {
    let x: u8 = (0.2126 * (p[0] as f64) + 0.7152 * (p[1] as f64) + 0.0722 * (p[2] as f64)) as u8;
    if x > threshold {
        Rgba([255, 255, 255, 0])
    } else {
        Rgba([0, 0, 0, 255])
    }
}

pub fn greyscale_img(img: &mut DynamicImage) {
    let width = img.width();
    let height = img.height();
    for x in 0..width {
        for y in 0..height {
            img.put_pixel(x, y, rgb_to_greyscale(img.get_pixel(x, y)));
        }
    }
}

pub fn threshold_img(img: &mut DynamicImage, threshold: u8) {
    let width = img.width();
    let height = img.height();
    for x in 0..width {
        for y in 0..height {
            img.put_pixel(x, y, rgb_to_threshold(img.get_pixel(x, y), threshold));
        }
    }
}

pub fn gray_threshold(p: &mut Luma<u8>, threshold: u8) {
    if p.0[0] > threshold {
        p.0[0] = 255;
    } else {
        p.0[0] = 0;
    }
}

pub fn threshold_grey_image(img: &mut GrayImage, threshold: u8) {
    let width = img.width();
    let height = img.height();
    for x in 0..width {
        for y in 0..height {
            let p = img.get_pixel_mut(x, y);
            gray_threshold(p, threshold);
        }
    }
}

pub fn threshold_img_buffer(
    mut image_buffer: ImageBuffer<Luma<u8>, Vec<u8>>,
    cmap: SplitColor,
) -> DynamicImage {
    for (_x, _y, p) in image_buffer.enumerate_pixels_mut() {
        cmap.map_color(p);
    }
    image_buffer.into()
}

pub fn ret_thresholded_img(image: DynamicImage, cmap: SplitColor) -> DynamicImage {
    let greyed = image.into_luma8();
    threshold_img_buffer(greyed, cmap)
}

fn get_cell_case(image: &DynamicImage, x: u32, y: u32) -> Result<QuadCase, ImageError> {
    if x >= image.width() - 1 {
        return Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )));
    }
    if y >= image.height() - 1 {
        return Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )));
    }

    let top_left = image.get_pixel(x, y)[0] == 0;
    let top_right = image.get_pixel(x + 1, y)[0] == 0;
    let bottom_right = image.get_pixel(x + 1, y + 1)[0] == 0;
    let bottom_left = image.get_pixel(x, y + 1)[0] == 0;
    Ok((top_left, top_right, bottom_right, bottom_left))
}

pub fn edge_img(img: &DynamicImage) -> Result<Edges, Box<dyn Error>> {
    let mut out = Edges::new();

    let width = img.width() - 1;
    let height = img.height() - 1;
    for x in 0..width {
        for y in 0..height {
            let border_case = (y == 0, x == width - 1, y == height - 1, x == 0);
            let quad = get_cell_case(img, x, y)?;
            let edges = get_quad_edge(border_case, quad, x as f64, y as f64, height as f64);
            for e in edges {
                out.add_edge(e);
            }
        }
    }
    Ok(out)
}

pub fn edge_file(threshold: u8, filename: &str) -> Result<Edges, Box<dyn Error>> {
    let mut img = ImageReader::open(filename)?
        .with_guessed_format()?
        .decode()?;
    threshold_img(&mut img, threshold);
    edge_img(&img)
}

pub fn threshold_png(in_filename: &str, out_filename: &str) -> Result<(), Box<dyn Error>> {
    let mut img = ImageReader::open(in_filename)?
        .with_guessed_format()?
        .decode()?;
    threshold_img(&mut img, 128);
    img.save(out_filename)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::json::save_vec_edge_as_geojson;
    use crate::components::constants::DEFAULT_THRESHOLD_VALUE;
    use super::*;

    extern crate test;
    use test::Bencher;

    #[bench]
    fn grey_then_color_map_in_place_threshold(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
        let img = ImageReader::open("./images/doggy.png")?
            .with_guessed_format()?
            .decode()?;
        b.iter(|| {
            ret_thresholded_img(img.clone(), SplitColor::new(127));
        });
        Ok(())
    }

    #[bench]
    fn inplace_custom_threshold(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
        let img = ImageReader::open("./images/doggy.png")?
            .with_guessed_format()?
            .decode()?;
        b.iter(|| {
            threshold_img(&mut img.clone(), 128);
        });
        Ok(())
    }

    #[test]
    fn edge_file_stick_figure() -> Result<(), Box<dyn Error>> {
        let edges = edge_file(DEFAULT_THRESHOLD_VALUE, "./images/stick-figure.png")?;
        assert_eq!(edges.closed_edges.len(), 9);
        Ok(())
    }
    #[test]
    fn edge_file_small_wolf() -> Result<(), Box<dyn Error>> {
        let edges = edge_file(DEFAULT_THRESHOLD_VALUE, "./images/small-wolf.png")?;
        save_vec_edge_as_geojson(edges.closed_edges, "small-wolf.json")?;
        //assert_eq!(edges.closed_edges.len(), 7);
        Ok(())
    }

    #[bench]
    fn edge_image_doggy(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
        let mut img = ImageReader::open("./images/doggy.png")?
            .with_guessed_format()?
            .decode()?;
        threshold_img(&mut img, DEFAULT_THRESHOLD_VALUE);

        b.iter(|| {
            edge_img(&img).unwrap();
        });
        return Ok(());
    }

    #[bench]
    fn edge_image_eagle_png(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
        let mut img = ImageReader::open("./images/eagle.png")?
            .with_guessed_format()?
            .decode()?;
        threshold_img(&mut img, DEFAULT_THRESHOLD_VALUE);

        b.iter(|| {
            edge_img(&img).unwrap();
        });
        return Ok(());
    }
}
