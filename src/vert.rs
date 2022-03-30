use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::{From, Into};
use std::usize;

use crate::edge::Edge;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vert {
    pub x: f64,
    pub y: f64,
}

impl From<Vec<f64>> for Vert {
    fn from(item: Vec<f64>) -> Self {
        Self::new(item[0], item[1])
    }
}

impl From<Vert> for Vec<f64> {
    fn from(item: Vert) -> Self {
        vec![item.x as f64, item.y as f64]
    }
}
impl From<&Vert> for Vec<f64> {
    fn from(item: &Vert) -> Self {
        vec![item.x as f64, item.y as f64]
    }
}

impl From<&Vert> for [f64; 2] {
    fn from(item: &Vert) -> Self {
        [(item.x as f64), (item.y as f64)]
    }
}

impl Vert {
    pub fn new<T: Into<f64>>(x: T, y: T) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub fn translate(&mut self, x: f64, y: f64) {
        self.x += x;
        self.y += y;
    }

    pub fn diff(&self, v: &Vert) -> Self {
        Self::new(self.x - v.x, self.y - v.y)
    }

    pub fn dot_product(&self, other: &Vert) -> f64 {
        (self.x * other.x) + (self.y * other.y)
    }

    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let xb = self.x.to_le_bytes();
        let yb = self.y.to_le_bytes();
        let o: [u8; 8] = [xb[0], xb[1], xb[2], xb[3], yb[0], yb[1], yb[2], yb[3]];
        o
    }
    pub fn to3(&self, z: f64) -> Vec<f64> {
        vec![self.x, self.y, z]
    }
}

impl Ord for Vert {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.x == other.x && self.y == other.y {
            return Ordering::Equal;
        }
        if self.x < other.x {
            return Ordering::Less;
        }
        Ordering::Greater
    }
}
impl PartialOrd for Vert {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Vert {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Vert {}

/*
 * o x ->
 * y  0-1-2
 * |  |   |
 * v  3 4 5
 *    |   |
 *    6-7-8
 */
fn get_xy_vert(i: u8, x: f64, y: f64, height: f64) -> Vert {
    match i {
        0 => Vert::new(x + 0., height - (y + 0.)),
        1 => Vert::new(x + 0.5, height - (y + 0.)),
        2 => Vert::new(x + 1., height - (y + 0.)),
        3 => Vert::new(x + 0., height - (y + 0.5)),
        4 => Vert::new(x + 0.5, height - (y + 0.5)),
        5 => Vert::new(x + 1., height - (y + 0.5)),
        6 => Vert::new(x + 0., height - (y + 1.)),
        7 => Vert::new(x + 0.5, height - (y + 1.)),
        8 => Vert::new(x + 1., height - (y + 1.)),
        _ => panic!("This should never happen"),
    }
}

pub type QuadCase = (bool, bool, bool, bool);
// top, right, bottom, left
pub type BorderCase = (bool, bool, bool, bool);

fn get_xy_e(a: u8, b: u8, x: f64, y: f64, height: f64) -> Edge {
    Edge::from([get_xy_vert(a, x, y, height), get_xy_vert(b, x, y, height)])
}

pub fn get_quad_edge(
    border_case: BorderCase,
    quad: QuadCase,
    x: f64,
    y: f64,
    height: f64,
) -> Vec<Edge> {
    let mut out = match quad {
        (false, false, false, false) => vec![],
        (false, false, false, true) => vec![get_xy_e(7, 3, x, y, height)],
        (false, false, true, false) => vec![get_xy_e(5, 7, x, y, height)],
        (false, false, true, true) => vec![get_xy_e(3, 5, x, y, height)],
        (false, true, false, false) => vec![get_xy_e(1, 5, x, y, height)],
        (false, true, false, true) => {
            vec![get_xy_e(1, 5, x, y, height), get_xy_e(7, 3, x, y, height)]
        }
        (false, true, true, false) => vec![get_xy_e(1, 7, x, y, height)],
        (false, true, true, true) => vec![get_xy_e(1, 3, x, y, height)],
        (true, false, false, false) => vec![get_xy_e(1, 3, x, y, height)],
        (true, false, false, true) => vec![get_xy_e(7, 1, x, y, height)],
        (true, false, true, false) => {
            vec![get_xy_e(3, 1, x, y, height), get_xy_e(5, 7, x, y, height)]
        }
        (true, false, true, true) => vec![get_xy_e(5, 1, x, y, height)],
        (true, true, false, false) => vec![get_xy_e(3, 5, x, y, height)],
        (true, true, false, true) => vec![get_xy_e(7, 5, x, y, height)],
        (true, true, true, false) => vec![get_xy_e(3, 7, x, y, height)],
        (true, true, true, true) => vec![],
    };

    if border_case.0 {
        if quad.0 {
            out.push(get_xy_e(1, 0, x, y, height))
        };
        if quad.1 {
            out.push(get_xy_e(2, 1, x, y, height))
        };
    }
    if border_case.1 {
        if quad.1 {
            out.push(get_xy_e(5, 2, x, y, height))
        };
        if quad.2 {
            out.push(get_xy_e(8, 5, x, y, height))
        };
    }
    if border_case.2 {
        if quad.2 {
            out.push(get_xy_e(7, 8, x, y, height))
        };
        if quad.3 {
            out.push(get_xy_e(6, 7, x, y, height))
        };
    }

    if border_case.3 {
        if quad.3 {
            out.push(get_xy_e(3, 6, x, y, height))
        };
        if quad.0 {
            out.push(get_xy_e(0, 3, x, y, height))
        };
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earcut::{deviation, earcut, flatten};
    use std::error::Error;

    #[test]
    fn move_vert() -> Result<(), Box<dyn Error>> {
        let mut v = Vert::new(0., 1.);
        v.translate(1., 2.);
        assert_eq!(v.x, 1.);
        assert_eq!(v.y, 3.);
        Ok(())
    }
}
