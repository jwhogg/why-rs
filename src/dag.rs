use std::fmt;
use std::ops::{Deref, DerefMut};
use petgraph::Direction;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;

pub type Variable = String;

pub type Value = u32;
pub struct DAG {
    pub graph: DiGraph<Variable, ()>
}

impl Deref for DAG {
    type Target = DiGraph<Variable, ()>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for DAG {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

impl DAG {
    pub fn new() -> DAG {
        DAG { graph: DiGraph::<Variable, ()>::new()}
    }

    pub fn get_index(&self, variable: Variable) -> Option<petgraph::graph::NodeIndex> {
        self.graph.node_indices().find(|&node| {
            self.graph[node].eq(&variable)
        })
    }

    pub fn get_parents(&self, node: Variable) -> Vec<Variable> {
        let node_index = self.get_index(node).expect("Node not found");
        self.graph.neighbors_directed(node_index, Direction::Incoming)
            .map(|neighbor_index| {
                // Indexing the graph (self.graph[i]) returns a reference (&Variable).
                // Since your return type is Vec<Variable>, you must CLONE it.
                self.graph[neighbor_index].clone()
            })
            .collect()
    }

    pub fn node(mut self, name: &str) -> Self {
        // Only add if it doesn't exist to prevent duplicates
        if self.get_index(name.to_string()).is_none() {
            self.graph.add_node(name.to_string());
        }
        self
    }

    // Handles index lookup internally
    pub fn edge(mut self, from: &str, to: &str) -> Self {
        let from_idx = self.get_index(from.to_string()).expect("Node not found");
        let to_idx = self.get_index(to.to_string()).expect("Node not found");
        self.graph.add_edge(from_idx, to_idx, ());
        self
    }

    pub fn add_node<S: Into<String>>(&mut self, name: S) {
        let s = name.into();
        self.graph.add_node(s);
    }
}

impl fmt::Display for DAG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DAG {{")?;

        for node_idx in self.graph.node_indices() {
            let name = &self.graph[node_idx];
            write!(f, "  {} -> ", name)?;

            let neighbors: Vec<_> = self.graph
                .edges(node_idx)
                .map(|e| self.graph[e.target()].clone())
                .collect();

            writeln!(f, "{:?}", neighbors)?;
        }

        writeln!(f, "}}")
    }
}