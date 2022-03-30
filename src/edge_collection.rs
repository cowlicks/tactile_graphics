use crate::{vert::Vert, edge::Edge};

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
        }
        _ if (target_back == other_back && target_front == other_front) => {
            other.pop_back();
            other.make_contiguous().reverse();
            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
        }
        _ if target_back == other_front => {
            other.pop_front();

            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
        }
        _ if target_back == other_back => {
            other.pop_back();
            other.make_contiguous().reverse();

            let l = target.len();
            if tip_redundant(&target[l - 2], &target[l - 1], &other[0]) {
                target.pop_back();
            }
            target.append(&mut other);
        }

        _ if target_front == other_front => {
            other.pop_front();
            if tip_redundant(&other[0], &target[0], &target[1]) {
                target.pop_front();
            }
            for v in other {
                target.push_front(v);
            }
        }

        _ if target_front == other_back => {
            other.pop_back();

            if tip_redundant(&other[other.len() - 1], &target[0], &target[1]) {
                target.pop_front();
            }

            for v in other.into_iter().rev() {
                target.push_front(v);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

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
