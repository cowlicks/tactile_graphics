use serde_json::{json, Value};
use std::error::Error;

use crate::edge::Edge;
use crate::tree::Node;
use crate::util::write_file;

pub fn feature_collection_from_values(values: Vec<Value>) -> Value {
    json!({
            "type": "FeatureCollection",
            "features": values,
    })
}

pub fn multiline_string_from_vec_edge(data: Vec<Edge>) -> Value {
    let coordinates: Vec<Vec<[f64; 2]>> = data.iter().map(|edge| edge.into()).collect();
    json!({
        "type": "Feature",
        "geometry": {
              "type": "MultiLineString",
              "coordinates": coordinates,
              "properties": {
                  "name": "closed_edges"
              },
        },
    })
}

pub fn save_vec_edge_as_geojson(data: Vec<Edge>, filename: &str) -> Result<(), Box<dyn Error>> {
    let value = feature_collection_from_values(vec![multiline_string_from_vec_edge(data)]);
    write_file(value.to_string(), filename)
}

fn edge_to_line_arr(e: Edge) -> Vec<[f64; 2]> {
    e.into()
}

fn node_to_polygon_arr(node: Node) -> Vec<Vec<[f64; 2]>> {
    let mut out = vec![node.data.into()];
    let mut holes: Vec<Vec<[f64; 2]>> = node
        .children
        .into_iter()
        .map(|child_node| edge_to_line_arr(child_node.data))
        .collect();
    out.append(&mut holes);
    out
}

pub fn multipolygon_file_from_nodes(
    filename: &str,
    nodes: Vec<Node>,
) -> Result<(), Box<dyn Error>> {
    let v = multipolygon_from_vec_edge_and_holes(nodes);
    write_file(v.to_string(), filename)
}

// TODO make json from tree node
pub fn multipolygon_from_vec_edge_and_holes(data: Vec<Node>) -> Value {
    let coordinates: Vec<Vec<Vec<[f64; 2]>>> = data.into_iter().map(node_to_polygon_arr).collect();
    let feature = json!({
        "type": "Feature",
        "geometry": {
              "type": "MultiPolygon",
              "coordinates": coordinates,
              "properties": {
                  "name": "closed_edges"
              },
        },
    });
    feature_collection_from_values(vec![feature])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::sort_edges;
    use crate::util::write_file;
    use std::error::Error;
    use crate::edge_file;
    use crate::components::constants::DEFAULT_THRESHOLD_VALUE;

    #[test]
    fn test_geojson() -> Result<(), Box<dyn Error>> {
        let edges = edge_file(DEFAULT_THRESHOLD_VALUE, "./images/stick-figure.png")?;
        let nodes = sort_edges(edges.closed_edges);
        for (i, n) in nodes.into_iter().enumerate() {
            info!("node {:?} has {:?} children", i, n.children.len());
            let json_value = multipolygon_from_vec_edge_and_holes(vec![n]);
            write_file(json_value.to_string(), &format!("out_{i}.json"))?;
        }
        Ok(())
    }
}
