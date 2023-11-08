use std::collections::vec_deque::{IntoIter, Iter};
use std::collections::VecDeque;
use std::ops::Index;

use serde::{Deserialize, Serialize};

use crate::vert::Vert;

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub verts: VecDeque<Vert>,
    _is_hole: Option<bool>,
    min_x: Option<f64>,
    max_x: Option<f64>,
    min_y: Option<f64>,
    max_y: Option<f64>,
}

impl Edge {
    pub fn new(&self) -> Edge {
        Edge {
            verts: VecDeque::new(),
            _is_hole: None,
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
        }
    }

    pub fn maybe_set_extremes(&mut self, v: &Vert) {
        if let Some(min_x) = self.min_x {
            if v.x < min_x {
                self.min_x = Some(min_x);
            }
        } else {
            self.min_x = Some(v.x);
        }

        if let Some(max_x) = self.max_x {
            if v.x < max_x {
                self.max_x = Some(max_x);
            }
        } else {
            self.max_x = Some(v.x);
        }

        if let Some(min_y) = self.min_y {
            if v.y < min_y {
                self.min_y = Some(min_y);
            }
        } else {
            self.min_y = Some(v.y);
        }

        if let Some(max_y) = self.max_y {
            if v.y < max_y {
                self.max_y = Some(max_y);
            }
        } else {
            self.max_y = Some(v.y);
        }
    }

    pub fn front(&self) -> Option<&Vert> {
        self.verts.front()
    }
    pub fn back(&self) -> Option<&Vert> {
        self.verts.back()
    }
    pub fn iter(&self) -> Iter<'_, Vert> {
        self.verts.iter()
    }
    pub fn len(&self) -> usize {
        self.verts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn push_front(&mut self, v: Vert) {
        self.maybe_set_extremes(&v);
        self.verts.push_front(v)
    }
    pub fn pop_front(&mut self) -> Option<Vert> {
        self.verts.pop_front()
    }
    pub fn pop_back(&mut self) -> Option<Vert> {
        self.verts.pop_back()
    }
    pub fn append(&mut self, e: &mut Edge) {
        for v in e.verts.iter() {
            self.maybe_set_extremes(v);
        }
        self.verts.append(&mut e.verts)
    }
    pub fn make_contiguous(&mut self) -> &mut [Vert] {
        self.verts.make_contiguous()
    }
    pub fn is_closed(&self) -> bool {
        self.verts[0] == self.verts[self.verts.len() - 1]
    }
    pub fn set_is_hole(&mut self) {
        if !self.is_closed() {
            panic!("we can only determine if closed edges are holes or not");
        }
        self._is_hole = Some(is_hole(self));
    }
    pub fn is_hole(&self) -> bool {
        if !self.is_closed() {
            panic!("we can only determine if closed edges are holes or not");
        }
        is_hole(self)
    }
    pub fn possibly_contains(&self, v: &Vert) -> bool {
        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) =
            (self.min_x, self.max_x, self.min_y, self.max_y)
        {
            if v.x <= max_x && v.x >= min_x && v.y <= max_y && v.y >= min_y {
                return true;
            }
            return false;
        }
        true
    }
}

impl From<&Edge> for Vec<[f64; 2]> {
    fn from(x: &Edge) -> Vec<[f64; 2]> {
        x.iter().map(|x| x.into()).collect()
    }
}
impl From<Edge> for Vec<[f64; 2]> {
    fn from(x: Edge) -> Vec<[f64; 2]> {
        x.iter().map(|x| x.into()).collect()
    }
}
impl<const N: usize> From<[Vert; N]> for Edge {
    fn from(verts: [Vert; N]) -> Edge {
        let (min_x, max_x, min_y, max_y) = get_extremes(&verts);
        Edge {
            verts: VecDeque::from(verts),
            _is_hole: None,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

fn get_extremes(verts: &[Vert]) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if verts.is_empty() {
        return (None, None, None, None);
    }

    let first = &verts[0];
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (first.x, first.x, first.y, first.y);
    for v in verts {
        if v.x < min_x {
            min_x = v.x
        }
        if v.x > max_x {
            max_x = v.x
        }
        if v.y < min_y {
            min_y = v.y
        }
        if v.y > max_y {
            max_y = v.y
        }
    }

    (Some(min_x), Some(max_x), Some(min_y), Some(max_y))
}

impl From<Vec<Vert>> for Edge {
    fn from(verts: Vec<Vert>) -> Edge {
        let (min_x, max_x, min_y, max_y) = get_extremes(&verts);

        Edge {
            verts: VecDeque::from(verts),
            _is_hole: None,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

impl Index<usize> for Edge {
    type Output = Vert;

    fn index(&self, i: usize) -> &Vert {
        &self.verts[i]
    }
}

impl IntoIterator for Edge {
    type Item = Vert;
    type IntoIter = IntoIter<Vert>;

    fn into_iter(self) -> Self::IntoIter {
        self.verts.into_iter()
    }
}

pub fn edge_to_geojson(edge: &Edge) -> Vec<Vec<f64>> {
    edge.iter()
        .map(|vert| vec![vert.x, vert.y])
        .collect()
}

fn edge_to_segments<'a>(edge: &'a Edge) -> Box<dyn Iterator<Item = [&Vert; 2]> + 'a> {
    let before_iter = edge.iter();
    let after_iter = edge.iter();
    Box::new(
        before_iter
            .zip(after_iter.skip(1))
            .map(|segment: (&Vert, &Vert)| [segment.0, segment.1]),
    )
}

fn get_upper_and_lower_bounds<'a>(
    edge: &'a Edge,
    target: &Vert,
) -> (Option<[&'a Vert; 2]>, Option<[&'a Vert; 2]>) {
    let segments = edge_to_segments(edge);
    let xcross = segments.filter(|[before, after]| {
        (before.x <= target.x && after.x >= target.x)
            || (before.x >= target.x && after.x <= target.x)
    });

    // find the least upper bound and greatest lower bound
    let (above, below) = xcross.fold((None, None), |acc, [before, after]| {
        let current_y = y_value_at(target.x, [before, after]);

        if current_y > target.y {
            if let Some((lowest_y, _, _)) = acc.0 {
                if current_y < lowest_y {
                    (Some((current_y, before, after)), acc.1)
                } else {
                    acc
                }
            } else {
                (Some((current_y, before, after)), acc.1)
            }
        } else if let Some((highest, _, _)) = acc.1 {
            if current_y > highest {
                (acc.0, Some((current_y, before, after)))
            } else {
                acc
            }
        } else {
            (acc.0, Some((current_y, before, after)))
        }
    });
    let mut out: (Option<[&Vert; 2]>, Option<[&Vert; 2]>) = (None, None);
    if let Some((_, before, after)) = above {
        out.0 = Some([before, after]);
    }
    if let Some((_, before, after)) = below {
        out.1 = Some([before, after]);
    }
    out
}

// check a closed edge includes a point
pub fn edge_contains(edge: &Edge, target: &Vert) -> bool {
    // find the least upper bound and greatest lower bound
    let (above, below) = get_upper_and_lower_bounds(edge, target);
    // check the handedness is correct
    if let Some([a_before, a_after]) = above {
        if a_before.x > a_after.x {
            // point is below least upper bound
            if let Some([b_before, b_after]) = below {
                if b_after.x > b_before.x {
                    // point is abover greatest lower bound
                    return true;
                }
            }
        }
    }
    false
}

pub fn hole_contains(edge: &Edge, target: &Vert) -> bool {
    // find the least upper bound and greatest lower bound
    let (above, below) = get_upper_and_lower_bounds(edge, target);
    // check the handedness is correct
    if let Some([a_before, a_after]) = above {
        if a_before.x < a_after.x {
            // point is below least upper bound
            if let Some([b_before, b_after]) = below {
                if b_after.x < b_before.x {
                    // point is abover greatest lower bound
                    return true;
                }
            }
        }
    }
    false
}

fn y_value_at(x: f64, segment: [&Vert; 2]) -> f64 {
    let left = segment[0];
    let right = segment[1];

    if left.y == right.y {
        return left.y;
    }

    if left.x == right.x {
        return left.y; // left or right should not matter since this just selects this segment, which isn't crossed by another segment
    }

    // the above removes the div by zero case
    let m = (right.y - left.y) / (right.x - left.x);
    let b = left.y - (m * left.x);
    m * x + b // = y
}

pub fn is_hole(edge: &Edge) -> bool {
    if let Some(result) = edge._is_hole {
        return result;
    }
    let first_x_point = edge[0].x;
    let next_x = edge
        .iter()
        .find(|vert| vert.x != first_x_point)
        .expect("this is a closed edge so it must have some change in x")
        .x;
    let some_x_point = (first_x_point + next_x) / 2.;

    let segments = edge_to_segments(edge);
    let xcross = segments.filter(|[before, after]| {
        (before.x < some_x_point && after.x > some_x_point)
            || (before.x > some_x_point && after.x < some_x_point)
    });

    let lowest = xcross.reduce(|acc, segment| {
        if y_value_at(some_x_point, segment) < y_value_at(some_x_point, acc) {
            segment
        } else {
            acc
        }
    });

    if let Some([before, after]) = lowest {
        after.x - before.x <= 0.
    } else {
        panic!("edges should always have two points so this should never happen");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::util::read_file_to_value;
    use crate::util::testing::closed_edge;
    use std::error::Error;

    #[test]
    fn test_possibly_contains() -> Result<(), Box<dyn Error>> {
        let e = closed_edge(10., 0., 0., false);
        assert_eq!(e.min_x, Some(0.));
        assert_eq!(e.max_y, Some(10.));
        assert!(e.possibly_contains(&Vert::new(5., 5.)), "edge does contain");
        assert!(
            !e.possibly_contains(&Vert::new(50., 50.)),
            "edge does not contain"
        );
        Ok(())
    }

    #[test]
    fn test_edge_contains_broken() -> Result<(), Box<dyn Error>> {
        let child: Edge = serde_json::from_value(read_file_to_value(
            "./test_data/edge_contains_broken_child.json",
        )?)?;
        let parent: Edge = serde_json::from_value(read_file_to_value(
            "./test_data/edge_contains_broken_parent.json",
        )?)?;
        assert!(edge_contains(&parent, &child[0]));
        Ok(())
    }

    #[test]
    fn test_edge_eq() {
        let e = closed_edge(1., 0., 0., true);
        let e2 = closed_edge(1., 0., 0., true);
        let e3 = closed_edge(1., 0., 0., false);
        assert!(e == e);
        assert!(e == e2);
        assert!(e2 == e);
        assert!(e != e3);
    }

    #[test]
    fn test_is_hole() {
        let e = closed_edge(1., 0., 0., true);
        assert!(is_hole(&e));
        let e2 = closed_edge(1., 0., 0., false);
        assert!(!is_hole(&e2));
    }

    #[test]
    fn test_segent() {
        let a = vec![1, 2, 3];
        let befor_iter = a.iter();
        let after_iter = a.iter();
        let segments: Vec<[&i32; 2]> = befor_iter
            .zip(after_iter.skip(1))
            .map(|segment: (&i32, &i32)| [segment.0, segment.1])
            .collect();
        let expected = vec![[&1, &2], [&2, &3]];
        assert_eq!(segments, expected);
    }
    #[test]
    fn test_edge_contains() {
        let e = Edge::from(vec![
            Vert::new(0., 0.),
            Vert::new(1., 0.),
            Vert::new(1., 1.),
            Vert::new(0., 1.),
            Vert::new(0., 0.),
        ]);
        assert_eq!(edge_contains(&e, &Vert::new(0.5, 0.5)), true);

        assert_eq!(edge_contains(&e, &Vert::new(0.5, 10.)), false);

        assert_eq!(edge_contains(&e, &Vert::new(10., 0.5)), false);
    }

    #[test]
    fn test_hole_contains() {
        let e = Edge::from(vec![
            Vert::new(0., 0.),
            Vert::new(0., 1.),
            Vert::new(1., 1.),
            Vert::new(1., 0.),
            Vert::new(0., 0.),
        ]);
        assert_eq!(hole_contains(&e, &Vert::new(0.5, 0.5)), true);

        assert_eq!(hole_contains(&e, &Vert::new(0.5, 10.)), false);

        assert_eq!(hole_contains(&e, &Vert::new(10., 0.5)), false);
    }
}
