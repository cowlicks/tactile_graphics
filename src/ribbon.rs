use crate::edge::Edge;

/// create a vertical wall of triangles along an edge with a given height
/// used to enclose the sides of the shape
pub fn edge_to_ribbon(edge: &Edge, height: f64) -> Vec<f64> {
    let is_hole = edge.is_hole();
    let verts = &edge.verts;

    (0..(edge.verts.len() - 1))
        .flat_map(|i| {
            let vn = &verts[i];
            let vnp1 = &verts[i + 1];
            if !is_hole {
                return vec![
                    vn.to3(0.),
                    vn.to3(height),
                    vnp1.to3(0.),
                    vn.to3(height),
                    vnp1.to3(height),
                    vnp1.to3(0.),
                ];
            } else {
                return vec![
                    vn.to3(0.),
                    vnp1.to3(0.),
                    vn.to3(height),
                    vn.to3(height),
                    vnp1.to3(0.),
                    vnp1.to3(height),
                ];
            };
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::testing::closed_edge;

    #[test]
    fn test_edge_to_ribbon() {
        let edge = closed_edge(10., 0., 0., false);
        let height = 6.;

        let tris = edge_to_ribbon(&edge, height);
        assert_eq!(tris.len() / 9, 8)
    }
}
