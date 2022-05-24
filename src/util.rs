use std::{error::Error, fs::File};

use serde_json::Value;

pub fn write_file(data: String, filename: &str) -> Result<(), Box<dyn Error>> {
    use std::io::Write;

    let mut outfile = File::create(filename)?;
    writeln!(outfile, "{}", data)?;
    Ok(())
}

pub fn read_file_to_value(filename: &str) -> Result<Value, Box<dyn Error>> {
    let f = File::open(filename)?;
    Ok(serde_json::from_reader(f)?)
}

pub mod testing {
    use crate::edge::Edge;
    use crate::vert::Vert;

    use image::io::Reader as ImageReader;
    use image::{DynamicImage, ImageError};

    pub fn image_from_file(filename: &str) -> Result<DynamicImage, ImageError> {
        ImageReader::open(filename)?.with_guessed_format()?.decode()
    }

    pub fn closed_edge(side: f32, x: f32, y: f32, hole: bool) -> Edge {
        let mut verts = vec![
            Vert::new(x, y),
            Vert::new(side + x, y),
            Vert::new(side + x, side + y),
            Vert::new(x, side + y),
            Vert::new(x, y),
        ];

        if hole {
            verts.reverse();
        }

        Edge::from(verts)
    }

    pub fn donut(side: f32, thickness: f32, x: f32, y: f32) -> Vec<Edge> {
        return vec![
            closed_edge(side - thickness * 2., x + thickness, y + thickness, true),
            closed_edge(side, x, y, false),
        ];
    }
}
