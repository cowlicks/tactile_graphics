use log::info;
use yew::{html, Html};

use base64::encode;
use image::{
    guess_format, load_from_memory_with_format, DynamicImage,
    ImageError, ImageFormat,
};
use std::io::Cursor;

fn format_to_dataurl_media_type(fmt: ImageFormat) -> String {
    match fmt {
        ImageFormat::Png => "image/png".to_string(),
        ImageFormat::Jpeg => "image/jpeg".to_string(),
        _ => panic!("no data url media type available for ImageFormat {:?}", fmt),
    }
}

pub fn data_url_from_img_bytes(bytes: &Vec<u8>) -> Result<String, ImageError> {
    let format = guess_format(bytes).expect("could not guess format");
    let mediatype = format_to_dataurl_media_type(format);
    let b64 = encode(bytes);
    Ok(format!("data:{};base64,{}", mediatype, b64))
}

pub fn img_from_bytes(bytes: &Vec<u8>) -> Result<DynamicImage, ImageError> {
    let format = guess_format(bytes)?;
    info!("image format is {:?}", format);
    load_from_memory_with_format(bytes, format)
}

pub fn img_html_from_bytes(bytes: &Vec<u8>) -> Html {
    log::info!("from img_html_from_bytes!!");
    let durl = data_url_from_img_bytes(bytes).unwrap();
    html! {
        <img src={ durl } alt={ "image from bytes" }/>
    }
}

pub fn img_to_formatted_bytes(img: DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut c = Cursor::new(vec![]);
    img.write_to(&mut c, format)
        .expect("Write to buffer should always work");
    c.into_inner()
}
