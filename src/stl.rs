use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
};

static HEADER: [u8; 80] = [0; 80];
static NORM: [u8; 12] = [0; 12];
static ATTRIBUTES: [u8; 2] = [0; 2];

static _Z_ZERO: [u8; 4] = (0.0f32).to_le_bytes();

pub fn write_triangle<T>(w: &mut T, tri: &[f32]) -> Result<(), Box<dyn Error>>
where
    T: std::io::Write,
{
    assert!(tri.len() == 9);

    w.write_all(&NORM)?;

    w.write_all(&tri[0].to_le_bytes())?;
    w.write_all(&tri[1].to_le_bytes())?;
    w.write_all(&tri[2].to_le_bytes())?;

    w.write_all(&tri[3].to_le_bytes())?;
    w.write_all(&tri[4].to_le_bytes())?;
    w.write_all(&tri[5].to_le_bytes())?;

    w.write_all(&tri[6].to_le_bytes())?;
    w.write_all(&tri[7].to_le_bytes())?;
    w.write_all(&tri[8].to_le_bytes())?;

    w.write_all(&ATTRIBUTES)?;

    Ok(())
}

pub fn write_triangle_f64<T>(w: &mut T, tri: &[f64]) -> Result<(), Box<dyn Error>>
where
    T: std::io::Write,
{
    assert!(tri.len() == 9);

    w.write_all(&NORM)?;

    w.write_all(&(tri[0] as f32).to_le_bytes())?;
    w.write_all(&(tri[1] as f32).to_le_bytes())?;
    w.write_all(&(tri[2] as f32).to_le_bytes())?;

    w.write_all(&(tri[3] as f32).to_le_bytes())?;
    w.write_all(&(tri[4] as f32).to_le_bytes())?;
    w.write_all(&(tri[5] as f32).to_le_bytes())?;

    w.write_all(&(tri[6] as f32).to_le_bytes())?;
    w.write_all(&(tri[7] as f32).to_le_bytes())?;
    w.write_all(&(tri[8] as f32).to_le_bytes())?;

    w.write_all(&ATTRIBUTES)?;

    Ok(())
}

pub fn write_2d_triangle<T>(w: &mut T, tri: &[f64], height: f64) -> Result<(), Box<dyn Error>>
where
    T: std::io::Write,
{
    assert!(tri.len() == 6);
    write_triangle(
        w,
        &[
            tri[0] as f32,
            tri[1] as f32,
            height as f32,
            tri[2] as f32,
            tri[3] as f32,
            height as f32,
            tri[4] as f32,
            tri[5] as f32,
            height as f32,
        ],
    )
}

pub fn write_stl_2d(filename: &str, tris: Vec<f64>) -> Result<(), Box<dyn Error>> {
    let f = File::create(filename)?;
    let mut writer = BufWriter::new(f);
    assert!(tris.len() % 6 == 0);
    let count: u32 = (tris.len() as u32) / 6;

    writer.write_all(&HEADER)?;
    writer.write_all(&count.to_le_bytes())?;
    for tri in tris.chunks(6) {
        write_2d_triangle(&mut writer, tri, 0.0)?;
    }

    Ok(())
}

pub fn triangles_to_stl_writer<T: Write>(
    writer: &mut BufWriter<T>,
    tris: Vec<f64>,
) -> Result<(), Box<dyn Error>> {
    let n3d = 9;
    assert!(tris.len() % n3d == 0);
    let count: u32 = (tris.len() / n3d) as u32;

    writer.write_all(&HEADER)?;
    writer.write_all(&count.to_le_bytes())?;
    for tri in tris.chunks(n3d) {
        write_triangle_f64(writer, tri)?;
    }
    Ok(())
}

pub fn write_stl_3d(filename: &str, tris: Vec<f64>) -> Result<(), Box<dyn Error>> {
    let f = File::create(filename)?;
    let mut writer = BufWriter::new(f);
    triangles_to_stl_writer(&mut writer, tris)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::write_stl_2d;

    #[test]
    fn test_chunks() {
        let data = vec![1, 1, 2, -2, 6, 0, 3, 1];

        let _r: Vec<Vec<&i32>> = data
            .chunks(3)
            .map(|c| c.iter().collect::<Vec<&i32>>())
            .collect();
    }
    #[test]
    fn test_write_stl_2d() -> Result<(), Box<dyn Error>> {
        let tris: Vec<f64> = vec![0., 0., 10., 0., 10., 10.];
        write_stl_2d("test.stl", tris)
    }
}
