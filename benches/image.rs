use bit_to_stl::components::constants::DEFAULT_THRESHOLD_VALUE;
use bit_to_stl::components::utils::SplitColor;
use bit_to_stl::{edge_img, ret_thresholded_img, threshold_img};
use image::io::Reader as ImageReader;
use image::DynamicImage;

fn main() {
    divan::main();
}

fn open(path: &str) -> DynamicImage {
    ImageReader::open(path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
}

#[divan::bench]
fn grey_then_color_map_in_place_threshold(bencher: divan::Bencher) {
    let img = open("./images/doggy.png");
    bencher
        .with_inputs(|| img.clone())
        .bench_local_values(|img| ret_thresholded_img(img, SplitColor::new(127)));
}

#[divan::bench]
fn inplace_custom_threshold(bencher: divan::Bencher) {
    let img = open("./images/doggy.png");
    bencher
        .with_inputs(|| img.clone())
        .bench_local_refs(|img| threshold_img(img, 128));
}

#[divan::bench(args = ["./images/doggy.png", "./images/eagle.png"])]
fn edge_image(bencher: divan::Bencher, path: &str) {
    let mut img = open(path);
    threshold_img(&mut img, DEFAULT_THRESHOLD_VALUE);
    bencher.bench_local(|| edge_img(&img).unwrap());
}

#[divan::bench(args = ["./images/small-wolf.png", "./images/doggy.png"])]
fn image_to_stl(bencher: divan::Bencher, path: &str) {
    const HEIGHT: f64 = 5.0;
    const SCALE_FACTOR: f64 = 3.0;
    let img = open(path);
    bencher
        .with_inputs(|| img.clone())
        .bench_local_values(|img| {
            bit_to_stl::triangle::image_to_stl(img, DEFAULT_THRESHOLD_VALUE, HEIGHT, SCALE_FACTOR)
                .unwrap()
                .collect::<Vec<f64>>()
        });
}
