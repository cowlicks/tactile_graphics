use log::info;

use crate::edge::{edge_contains, hole_contains, Edge};

#[derive(Debug)]
pub struct Tree {
    children: Vec<Node>,
}

/// insert new_node into the tree
fn insert(nodes: &mut Vec<Node>, mut new_node: Node) {
    // get children of the new data
    let data_children: Vec<Node> = nodes
        .drain_filter(|child| child.is_child_of(&new_node))
        .collect();

    // insert those children as children of the new data
    for data_child in data_children {
        new_node.insert(data_child);
    }

    nodes.push(new_node);
    //// previously here we would handle the case where nodes here were parents of the current node.
    //// but if we insert nodes in the order that they were created, this is never the case

    //// find the parent of the new data
    //let some_parent_index = nodes.iter().position(|child| child.is_parent_of(&new_node));

    //// insert the new data into its parent
    //if let Some(index) = some_parent_index {
    //    nodes[index].insert(new_node);
    //} else {
    //    // or insert it as a sibling
    //    nodes.push(new_node);
    //}
}

/// flatten_nodes to be passed to earcuttr
fn flatten_nodes(t: Tree) -> Vec<Node> {
    let mut out: Vec<Node> = vec![];
    let mut holes: Vec<Node> = vec![];

    for child in t.children.iter() {
        for cc in child.children.iter() {
            holes.push(cc.clone());
        }
    }
    out.append(&mut t.children.clone());

    while !holes.is_empty() {
        let h = holes.pop().expect("we just checked holes len above");
        for child in h.children.iter() {
            for cc in child.children.iter() {
                holes.push(cc.clone());
            }
        }
        out.append(&mut h.children.clone());
    }
    out
}

impl Tree {
    fn new() -> Tree {
        Tree { children: vec![] }
    }

    fn insert(&mut self, data: Edge) {
        let new_node = Node::new(data);
        insert(&mut self.children, new_node);
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Node {
    pub data: Edge,
    pub children: Vec<Node>,
}

 /// maybe make this generic and have it take a "family" trait'd object
 /// with has a "get_relationship(
impl Node {
    fn new(data: Edge) -> Node {
        Node {
            data,
            children: vec![],
        }
    }
    fn is_child_of(&self, other: &Node) -> bool {
        if other.data.is_hole() {
            hole_contains(&other.data, &self.data[0])
        } else {
            edge_contains(&other.data, &self.data[0])
        }
    }
    fn _is_parent_of(&self, other: &Node) -> bool {
        if self.data.is_hole() {
            hole_contains(&self.data, &other.data[0])
        } else {
            edge_contains(&self.data, &other.data[0])
        }
    }

    fn insert(&mut self, new_node: Node) {
        insert(&mut self.children, new_node);
    }

    pub fn to_geojson_polygon_arr(self) -> Vec<Vec<[f64; 2]>> {
        let mut out = vec![self.data.into()];
        for c in self.children {
            out.push(c.data.into());
        }
        out
    }
}

pub fn tree_from_edges(edges: Vec<Edge>) -> Tree {
    let mut t = Tree::new();
    let l = edges.len();
    for (i, e) in edges.into_iter().enumerate() {
        info!("inserting item {i}/{l}");
        t.insert(e);
    }
    t
}

pub fn sort_edges(edges: Vec<Edge>) -> Vec<Node> {
    let t = tree_from_edges(edges);
    flatten_nodes(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::testing::{closed_edge, donut};
    use std::error::Error;
    use crate::edge_file;
    use crate::components::constants::DEFAULT_THRESHOLD_VALUE;

    static SIDE: f32 = 100.;
    static X0: f32 = 0.;
    static Y0: f32 = 0.;

    #[test]
    fn test_tree() -> Result<(), Box<dyn Error>> {
        let edges = edge_file(DEFAULT_THRESHOLD_VALUE, "./images/stick-figure.png")?;
        let t = tree_from_edges(edges.closed_edges);
        assert_eq!(t.children.len(), 1);
        assert_eq!(t.children[0].children.len(), 4);
        Ok(())
    }

    #[test]
    fn test_insert() {
        let outer = Node::new(closed_edge(SIDE, X0, Y0, false));
        let inner = Node::new(closed_edge(SIDE / 2., X0 + SIDE / 4., Y0 + SIDE / 4., true));
        assert_eq!(inner.is_child_of(&outer), true, "inner is child of outer");
        assert_eq!(outer.is_parent_of(&inner), true, "outer is parent of inner");

        let mut nodes = vec![];
        insert(&mut nodes, outer);
        assert_eq!(nodes.len(), 1);
        insert(&mut nodes, inner);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].children.len(), 1);
    }

    #[test]
    fn test_sort_edges() -> Result<(), Box<dyn Error>> {
        let d = donut(100., 1., 0., 0.);
        let nodes = sort_edges(d.clone());
        assert_eq!(nodes[0].data, d[0]);
        assert_eq!(nodes[0].children[0].data, d[1]);

        Ok(())
    }

    #[test]
    fn test_sort_edges_stick_figure() -> Result<(), Box<dyn Error>> {
        let edges = edge_file(DEFAULT_THRESHOLD, "./images/stick-figure.png")?;
        let sorted = sort_edges(edges.closed_edges);
        //assert_eq!(sorted.len(), 5, "Should equal the total number of shapes");
        Ok(())
    }

    #[test]
    fn test_sort_edges_donut_within_donut() {
        let mut d = donut(100., 1., 0., 0.);
        d.append(&mut donut(50., 1., 10., 10.));
        let nodes = sort_edges(d.clone());
        assert_eq!(nodes.len(), 2, "should be two donuts");
        assert_eq!(nodes[0].data, d[0]);
        assert_eq!(nodes[0].children[0].data, d[1]);
    }
}
