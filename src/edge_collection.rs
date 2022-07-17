use log::{info, trace};

use crate::{vert::Vert, edge::Edge};

static EPSILON: f64 = 1e-8;

#[derive(Debug)]
pub struct Edges {
    pub edges: Vec<Edge>,
    pub closed_edges: Vec<Edge>,
}

/// check if two edge tips have a redundant vertex when combining them
fn tip_redundant(left: &Vert, middle: &Vert, right: &Vert) -> bool {
    if left.x == middle.x {
        return middle.x == right.x;
    }
    if middle.x == right.x {
        return left.x == middle.x;
    }
    //
    // the above removes the possibility of dividing by zero
    //
    let l_slope = (middle.y - left.y) / (middle.x - left.x);
    let r_slope = (right.y - middle.y) / (right.x - middle.x);

    l_slope == r_slope
}

/// Connect other edge to target edge
fn connect(target: &mut Edge, mut other: Edge) {
    let target_front = target.front();
    let target_back = target.back();

    let other_front = other.front();
    let other_back = other.back();

    return match (target_front, target_back) {
        _ if (target_back == other_front && target_front == other_back) => {
            other.pop_front();
            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
            maybe_remove_zig_zags(target);
        }
        _ if (target_back == other_back && target_front == other_front) => {
            other.pop_back();
            other.make_contiguous().reverse();
            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
            maybe_remove_zig_zags(target);
        }
        _ if target_back == other_front => {
            other.pop_front();

            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
            maybe_remove_zig_zags(target);
        }
        _ if target_back == other_back => {
            other.pop_back();
            other.make_contiguous().reverse();

            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
            maybe_remove_zig_zags(target);
        }

        _ if target_front == other_front => {
            other.pop_front();
            if tip_redundant(&other[0], &target[0], &target[1]) {
                target.pop_front();
            }
            for v in other {
                target.push_front(v);
            }
            maybe_remove_zig_zags(target);
        }

        _ if target_front == other_back => {
            other.pop_back();

            if tip_redundant(&other[other.len() - 1], &target[0], &target[1]) {
                target.pop_front();
            }

            for v in other.into_iter().rev() {
                target.push_front(v);
            }
            maybe_remove_zig_zags(target);
        }
        _ => panic!(
            "No edges lined up, can't connect
                    target_front = {:?}
                    target_back = {:?}
                    other_front = {:?}
                    other_back = {:?}
                    ",
            target_front, target_back, other_front, other_back
        ),
    };
}

impl Default for Edges {
    fn default() -> Self {
        Self::new()
    }
}

impl Edges {
    pub fn new() -> Self {
        Self {
            edges: vec![],
            closed_edges: vec![],
        }
    }

    /// handle when an edge connects to itself
    pub fn mark_closed(&mut self, edge_i: usize) {
        let mut e = self.edges.remove(edge_i);
        if e[0] != e[e.len() - 1] {
            panic!("tail and head should match!");
        }

        e.set_is_hole();

        self.closed_edges.push(e);
    }

    /// a GeoJSON Polygon is composed of "linear rings" which must have the same first and last
    /// value
    /// https://datatracker.ietf.org/doc/html/rfc7946#section-3.1.6
    pub fn add_edge(&mut self, x: Edge) {
        let other_back = x.back();
        let other_front = x.front();

        if other_back == other_front {
            self.closed_edges.push(x);
            return;
        }
        let mut i = 0;
        let mut found: Vec<usize> = vec![];

        while i < self.edges.len() {
            let target_front = self.edges[i].front();
            let target_back = self.edges[i].back();

            if other_front == target_front
                || other_front == target_back
                || other_back == target_front
                || other_back == target_back
            {
                found.push(i);
                if found.len() == 2 {
                    break;
                }
            }
            i += 1;
        }

        match found.len() {
            0 => self.edges.push(x),
            1 => {
                connect(&mut self.edges[found[0]], x);
                let edge = &self.edges[found[0]];
                if edge[0] == edge[edge.len() - 1] {
                    self.mark_closed(found[0]);
                }
            }
            2 => {
                connect(&mut self.edges[found[0]], x);
                let other = self.edges.remove(found[1]);
                connect(&mut self.edges[found[0]], other);
                let edge = &self.edges[found[0]];
                if edge[0] == edge[edge.len() - 1] {
                    self.mark_closed(found[1]);
                }
            }
            _ => panic!("We should only have 0, 1, or 2 matched edges"),
        }
    }
}

/// the angle at a vertex on an edge with the given index
fn edge_angle(edge: &Edge, index: usize) -> f64 {
    let len = edge.len();
    if index == 0 || index >= len -1 {
        panic!("cant compute angle edge index must be between 1 and {}, got {}", len - 1, index);
    }
    let middle = &edge.verts[index];

    let a = edge.verts[index + 1].diff(&middle);
    let b = edge.verts[index - 1].diff(&middle);

    a.angle_between(&b)
}

/// the length of an edge segment to the right of the given index
fn edge_segment_length(edge: &Edge, index: usize) -> f64 {
    let len = edge.len();
    let n_segments = len - 1;

    if index >= n_segments {
        panic!("Given bad index to get segment length from. There is no edge segment to the right of index = {index}, there are only n_segments = {n_segments}");
    }

    let segment = edge.verts[index].diff(&edge.verts[index + 1]);
    segment.magnitude()
}

static MIN_ZIG_ZAG_COUNT: usize  = 2;

fn maybe_remove_zig_zags(edge: &mut Edge) {
    // 2 zig-zags + a non-zig-zag point on each end
    let min_length = MIN_ZIG_ZAG_COUNT * 2 + 2;

    // explanation of the indices used
    // len      no index
    // len - 1  last vert, no angle
    // len - 2  last angle, first angle past zig-zag.
    //          Angle should not equal len - 4, or this would be in the zig-zig
    // len - 3  last zig-zag angle, start here
    let len = edge.len();

    if len < min_length {
        trace!("Edge too short (len = {len}) must be at least {min_length}");
        return;
    }

    let mut index = len - 3;
    let non_zig_zag_angle = edge_angle(edge, index + 1);
    let initial_zag_angle = edge_angle(edge, index);
    let initial_zig_angle = edge_angle(edge, index - 1);

    // last angle equals last zig, so zig zag is ongoing, so not over
    if (non_zig_zag_angle - initial_zig_angle).abs() < EPSILON {
        trace!("Can't remove zig-zig because there is none, or it has not ended");
        return; // not end of zig-zag
    }

    if (initial_zag_angle + initial_zig_angle).abs() > EPSILON {
        trace!("Angles don't cancel so this is not a zig-zag");
        return; // angles don't cancel out, not zig-zag
    }

    let initial_zag_length = edge_segment_length(edge, index);
    let initial_zig_length = edge_segment_length(edge, index - 1);

    let mut count: usize = 0;

    while index - 1 > 0 {
        let zag_angle = edge_angle(edge, index);
        let zig_angle = edge_angle(edge, index - 1);

        if (zag_angle + zig_angle).abs() > EPSILON {
            trace!("Angles in zig-zag don't cancel, we must have reached the end");
            break; // angles don't cancel out
        }

        let zag_length = edge_segment_length(edge, index);
        let zig_length = edge_segment_length(edge, index - 1);
        if zig_length != initial_zig_length || zag_length != initial_zag_length {
            trace!("lengths in zig-zags don't match the original zig-zag length, we must have reached the end");
            break;
        }


        count += 1;
        index -= 2;
    }

    // check for a final straggling zag
    let mut straggling_zag = 0;
    if index >= 2 { // might have a straggling zag
        trace!("Checking for a straggling zag");
        let zag_angle = edge_angle(edge, index);
        let zag_length = edge_segment_length(edge, index);
        if (zag_angle - initial_zag_angle).abs() < EPSILON && (zag_length - initial_zag_length).abs() < EPSILON {
            straggling_zag = 1;
        }
    }

    trace!("finished walking through zig-zag, counted [{count}] zig-zags
          final index [{index}]");

    if count + straggling_zag >= MIN_ZIG_ZAG_COUNT {
        info!("Removing zig zags:
              got [{count}]
              stragling [{straggling_zag}]
              final index [{index}]");
        // remove from start which is len - count * 2 up to len - 2
        let split_at_n = len - 2 - (2 * count) - straggling_zag;
        let mut zig_zag_tail = edge.verts.split_off(split_at_n);
        let last = zig_zag_tail.pop_back().unwrap();
        edge.verts.push_back(zig_zag_tail.pop_back().unwrap());
        edge.verts.push_back(last);
    } else {
        trace!("number of zig-zags ({count}) didn't reach the minimum zig-zag count ({MIN_ZIG_ZAG_COUNT}) so doing nothing");
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn test_rm_zig_zag_too_short_edge() {
        let mut edge = Edge::from([
        ]);
        maybe_remove_zig_zags(&mut edge);
        assert_eq!(edge.len(), 0);

        let mut edge = Edge::from([
                              Vert::new(0., 10.),   // outside zig-zag
                              Vert::new(0., 0.),    // start zig-zag
                              Vert::new(1., 0.),    // zig
                              Vert::new(1., 0.5),   // zag
                              Vert::new(5., 1.),    // zag

        ]);
        maybe_remove_zig_zags(&mut edge);
        assert_eq!(edge.len(), 5);
    }
    #[test]
    fn test_rm_zig_zag_shortest_edge() {
        let mut edge = Edge::from([
                              Vert::new(-2., -10.),   // outside zig-zag
                              Vert::new(0., 0.),    // start zig-zag
                                                    //
                              Vert::new(1., 0.),    // zig
                              Vert::new(1., 0.5),   // zag
                                                    //
                              Vert::new(2., 0.5),   // zig
                              Vert::new(10., 4.),   // outside zig-zag
        ]);
        maybe_remove_zig_zags(&mut edge);
        assert_eq!(
            edge,
            Edge::from([
                              Vert::new(-2., -10.),   // outside zig-zag
                              Vert::new(0., 0.),    // start zig-zag
                              Vert::new(2., 0.5),   // zig
                              Vert::new(10., 4.),   // outside zig-zag
            ]),
            );
    }
    #[test]
    fn test_rm_zig_zag_1() {
        let mut edge = Edge::from([
                              Vert::new(-2., -10.),   // outside zig-zag
                              Vert::new(0., 0.),    // start zig-zag
                                                    //
                              Vert::new(1., 0.),    // zig
                              Vert::new(1., 0.5),   // zag
                                                    //
                              Vert::new(2., 0.5),   // zig
                              Vert::new(2., 1.),    // zag
                                                    //
                              Vert::new(3., 1.),    // zig
                              Vert::new(3., 1.5),   // zag last point
                                                    //
                              Vert::new(10., 4.),   // outside zig-zag
        ]);
        // len = 9
        // // want split_at = 2
        // len - 3 - (count * 2) = 2

        let expected = Edge::from([
                              Vert::new(-2., -10.),   // outside zig-zag
                              Vert::new(0., 0.),    // start zig-zag
                              Vert::new(3., 1.5),   // zag last point
                              Vert::new(10., 4.),   // outside zig-zag
            ]);
        maybe_remove_zig_zags(&mut edge);
        assert_eq!(edge, expected);

    }

    #[test]
    fn test_edge_angle() {
        let left_turn = Edge::from([
                               Vert::new(0., 0.),
                               Vert::new(1., 0.),
                               Vert::new(1., 1.),
        ]);
        let angle = edge_angle(&left_turn, 1);
        assert!(angle > 0.);
        assert!((angle - FRAC_PI_2).abs() < EPSILON);

        let right_turn = Edge::from([
                               Vert::new(0., 0.),
                               Vert::new(1., 0.),
                               Vert::new(1., -1.),
        ]);
        let angle = edge_angle(&right_turn, 1);
        assert!(angle < 0.);
        assert!((angle + FRAC_PI_2).abs() < EPSILON);
    }

    #[test]
    fn test_maybe_remove_zig_zags() {
        let mut e = Edge::from([
                               Vert::new(1., 50.),      // start not in zig-zag
                                                        //
                               Vert::new(0., 0.),       // first point in zig-zag
                               Vert::new(3., 1.),

                               Vert::new(3., 2.),
                               Vert::new(6., 3.),

                               Vert::new(6., 4.),
                               Vert::new(9., 5.),

                               Vert::new(9., 6.),
                               Vert::new(12., 7.),

                               Vert::new(13., 40.0),    // end, not in zig-zag
        ]);
        let expected = Edge::from([
                               Vert::new(1., 50.),      // start
                               Vert::new(0., 0.),       // start zig-zag
                               Vert::new(12., 7.),       // first point in zig-zag
                               Vert::new(13., 40.0),    // end
        ]);
        maybe_remove_zig_zags(&mut e);
        assert_eq!(e, expected);
    }

    #[test]
    fn test_add_edge_creates_closed_edge() {
        let mut es = Edges::new();
        es.add_edge(Edge::from([Vert::new(0., 0.), Vert::new(100., 0.)]));
        es.add_edge(Edge::from([Vert::new(100., 0.), Vert::new(100., 100.)]));
        es.add_edge(Edge::from([Vert::new(100., 100.), Vert::new(0., 100.)]));
        es.add_edge(Edge::from([Vert::new(0., 100.), Vert::new(0., 0.)]));
        assert_eq!(es.closed_edges.len(), 1, "closed edge created");
    }

    #[test]
    fn test_from_into() {
        //let e = Edge::from([Vert::new(0., 0.), Vert::new(0., 0.1)]);
        let v = Vert::new(0., 0.);
        let v2: Vec<f64> = v.into();
        assert_eq!(v2, vec![0., 0.]);
        // let e = Edge::from([Vert::new(0., 0.), Vert::new(0., 0.1)]);
        // let e2: Vec<Vec<f64>> = e.iter().map(|v| v.into()).collect();
        //assert_eq!(e2, vec![vec![0., 0.], vec![0., 0.1]]);
    }
    #[test]
    fn test_edges_add_edge() {
        let e = Edge::from([Vert::new(0., 0.), Vert::new(0., 0.1)]);
        let mut es = Edges::new();
        es.add_edge(e);
    }
    #[test]
    fn test_edges_add_edge_back_to_front() {
        let e = Edge::from([Vert::new(0., 0.), Vert::new(0., 1.)]);
        let i = Edge::from([Vert::new(0., 1.), Vert::new(2., 2.)]);
        let mut es = Edges::new();
        es.add_edge(e);
        es.add_edge(i);
        assert_eq!(es.edges.len(), 1);
        assert_eq!(es.edges[0].len(), 3);
    }
    #[test]
    fn test_edges_add_edge_front_to_front() {
        let e = Edge::from([Vert::new(0., 0.), Vert::new(0., 1.)]);
        let i = Edge::from([Vert::new(0., 0.), Vert::new(2., 2.)]);
        let mut es = Edges::new();
        es.add_edge(e);
        es.add_edge(i);
        assert_eq!(es.edges.len(), 1);
        assert_eq!(es.edges[0].len(), 3);
    }

}
