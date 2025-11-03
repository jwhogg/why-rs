use petgraph::{Graph, Directed};
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

use crate::error::CausalError;

// The main struct for your project
pub struct CausalGraph {
    pub graph: Graph<String, (), Directed>,
    pub nodes: HashMap<String, NodeIndex>,
}