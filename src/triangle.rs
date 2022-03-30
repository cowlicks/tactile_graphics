use log::info;
use std::error::Error;

use crate::{
    earcut::{earcut, flatten},
    edge_img,
    ribbon::edge_to_ribbon,
    stl::write_stl_3d,
    threshold_img,
    tree::sort_edges,
    DEFAULT_THRESHOLD,
};

use image::{io::Reader as ImageReader, DynamicImage};

fn to_3d(tris: &Vec<f64>, z: f64) -> Vec<f64> {
    let d2 = 2;
    assert!(tris.len() % d2 == 0);
    let n_verts = tris.len() / d2;
    (0..n_verts)
        .flat_map(|c_i| {
            let i = c_i * d2;
            vec![tris[i], tris[i + 1], z]
        })
        .collect()
}

fn scale(v: Vec<f64>, factor: f64) -> Vec<f64> {
    v.iter().map(|x| x * factor).collect()
}

/// copy triangles, but flipped over and moved to z
fn flip_triangles(tris: &[f64], new_z: f64) -> Vec<f64> {
    let dims = 3;
    let n_corners = 3;
    let n_verts_in_polygon = dims * n_corners;

    assert!(tris.len() % (n_verts_in_polygon) == 0);
    info!("flipping triangles n = {:?}", tris.len() / n_verts_in_polygon);

    let n_tris = tris.len() / n_verts_in_polygon;
    (0..n_tris)
        .flat_map(|tri_i| {
            let i = tri_i * n_verts_in_polygon;
            vec![
                tris[i],
                tris[i + 1],
                new_z,
                tris[i + 6],
                tris[i + 7],
                new_z,
                tris[i + 3],
                tris[i + 4],
                new_z,
            ]
        })
        .collect()
}

/*
 * TODO
 * how to sort edges in a streaming way:
 * once an exterior edge is closed, the holes will
 * already have been completed. So we can construct its "polygon array"
 * each time an edge is closed insert into sort tree
 * if the edge is exterior, insert it, then remove it with its children
 * construct this polygon arr with this
 * then proceed to flatten and earcut it
 */

/// Given an image create an array of triangles representing an stl
/// The edges of the shape are determined by thresholding with the given value.
/// The provided height of the shape, , in stl units (usually millimeters)
/// The length & width of resulting stl are the pixel-heght & pixel-width
/// of the input image, multiplied by the provided scale.
pub fn image_to_stl(
    mut img: DynamicImage,
    threshold: u8,
    height: f64,
    scale_factor: f64,
) -> Result<Vec<f64>, Box<dyn Error>> {
    //let mut img = img.to_luma8();

    //threshold_grey_image(&mut img, threshold);
    threshold_img(&mut img, threshold);
    info!("thresholded image for stl");

    let edges = edge_img(&img)?;
    info!("created edges for stl");

    let ribbon_tris: Vec<f64> = edges
        .closed_edges
        .iter()
        .flat_map(|edge| edge_to_ribbon(edge, height))
        .collect();

    let nodes = sort_edges(edges.closed_edges);

    info!("sorted edges for stl");

    // simplify this step
    let polygon_arr: Vec<Vec<Vec<Vec<f64>>>> = nodes
        .into_iter()
        .map(|node| node.to_geojson_polygon_arr())
        .map(|polygon| {
            polygon
                .iter()
                .map(|line| line.iter().map(|coord| vec![coord[0], coord[1]]).collect())
                .collect()
        })
        .collect();

    info!("created polygon from sorted edges for stl");
    let earcut_args_arr: Vec<(Vec<f64>, Vec<usize>, usize)> =
        polygon_arr.iter().map(flatten).collect();

    info!("got earcut args for stl");

    let triangle_indexes: Vec<Vec<usize>> = earcut_args_arr
        .iter()
        .map(|(verts, holes, _dims)| {
            println!("shape num holes: {}", holes.len());
            earcut(verts, holes, 2)
        })
        .collect();

    info!("earcut into triangles for stl");

    let triangles: Vec<f64> = triangle_indexes
        .iter()
        .zip(earcut_args_arr)
        .flat_map(|(indexes, ear_args)| {
            let verts = ear_args.0;
            println!(
                "
                verts.len = {}
                indexes.len = {}
                ",
                verts.len(),
                indexes.len()
            );
            indexes
                .iter()
                .map(|i| {
                    let start = i * 2;
                    verts[start..(start + 2)].to_vec()
                })
                .collect::<Vec<Vec<f64>>>()
        })
        .flatten()
        .collect();

    info!("created flat tris for stl");

    let tri_3d = to_3d(&triangles, height);

    info!("made tris 3D");
    let bottom_triangles: Vec<f64> = flip_triangles(&tri_3d, 0.);
    info!("made bottom tris ");

    let all_tris = ribbon_tris
        .into_iter()
        .chain(tri_3d.into_iter())
        .chain(bottom_triangles.into_iter())
        .collect();
    info!("combined all triangles");

    Ok(scale(all_tris, scale_factor))
}

// add height, scale, and threshold arguments
pub fn image_file_to_stl(
    input_filename: &str,
    height: f64,
    scale_factor: f64,
    output_filename: &str,
) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::open(input_filename)?
        .with_guessed_format()?
        .decode()?;

    let tris = image_to_stl(img, DEFAULT_THRESHOLD, height, scale_factor)?;
    write_stl_3d(output_filename, tris)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::*;
    use std::error::Error;
    extern crate test;

    static SOME_HEIGHT: f64 = 5.0;
    static SOME_SCALE_FACTOR: f64 = 3.0;

    #[bench]
    fn bench_img_to_stl_small_wolf(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
        let img = ImageReader::open("./images/small-wolf.png")?
            .with_guessed_format()?
            .decode()?;

        b.iter(|| {
            image_to_stl(
                img.clone(),
                DEFAULT_THRESHOLD,
                SOME_HEIGHT,
                SOME_SCALE_FACTOR,
            )
            .unwrap();
        });
        Ok(())
    }

    #[bench]
    fn bench_img_to_stl_middle_finger(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
        let img = ImageReader::open("./images/doggy.png")?
            .with_guessed_format()?
            .decode()?;

        b.iter(|| {
            image_to_stl(
                img.clone(),
                DEFAULT_THRESHOLD,
                SOME_HEIGHT,
                SOME_SCALE_FACTOR,
            )
            .unwrap();
        });
        Ok(())
    }

    #[test]
    fn test_earcut_steps_3d() -> Result<(), Box<dyn Error>> {
        let input_filename = "./images/stick-figure.png";
        let height = 50.0;
        let scale_factor = 0.05;
        let output_filename = "test.stl";

        image_file_to_stl(input_filename, height, scale_factor, output_filename)
    }
}
