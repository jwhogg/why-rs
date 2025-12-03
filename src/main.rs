// use why_rs::core::CausalGraph;
//

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use petgraph::data::Build;
use petgraph::Direction;
use petgraph::graph::DiGraph;
use petgraph::visit::IntoNeighborsDirected;

// #[derive(Debug, Clone)]
// struct Variable {
//     name: String,
// }
//
// impl Variable {
//     fn from(name: &str) -> Variable {
//         Variable { name: String::from(name) }
//     }
// }
type Variable = String;

pub type Value = u32;
struct DAG {
    graph: DiGraph<Variable, ()>
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

#[macro_export]
macro_rules! fcm {
    // Match "A" => "B" patterns separated by commas
    ( $( $from:expr => $to:expr ),* ) => {
        {
            let mut dag = DAG::new();
            $(
                dag = dag.node($from).node($to).edge($from, $to);
            )*
            dag
        }
    };
}
impl DAG {
    fn new() -> DAG {
        DAG { graph: DiGraph::<Variable, ()>::new()}
    }

    fn get_index(&self, variable: Variable) -> Option<petgraph::graph::NodeIndex> {
        self.graph.node_indices().find(|&node| {
            self.graph[node].eq(&variable)
        })
    }

    fn get_parents(&self, node: Variable) -> Vec<Variable> {
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

struct Noise {
    value: u32,
}

struct FCM {
    graph: DAG,
    // &[] means the function takes a slice args, so any length
    // Box is because we don't know func length at compile time, we move it to the heap
    functions: HashMap<Variable, Box<dyn FnMut(&[Value]) -> Value>>,
}

impl Deref for FCM {
    type Target = DiGraph<Variable, ()>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for FCM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

impl FCM {
    pub fn new() -> Self {
        FCM {
            graph: DAG::new(),
            functions: HashMap::new(),
        }
    }

    pub fn rule<F>(mut self, target: &str, func: F) -> Self
    where
        F: FnMut(&[Value]) -> Value + 'static,
    {
        self.functions.insert(target.to_string(), Box::new(func));
        self
    }

    pub fn node(mut self, name: &str) -> Self {
        // We use the existing logic on DAG, but wrap it to return 'self'
        // Note: You might want to check if it exists first to avoid duplicates
        if self.graph.get_index(Variable::from(name)).is_none() {
            self.graph.add_node(Variable::from(name));
        }
        self
    }

    pub fn edge(mut self, from: &str, to: &str) -> Self {
        let from_idx = self.graph.get_index(Variable::from(from)).expect("Source node missing");
        let to_idx = self.graph.get_index(Variable::from(to)).expect("Target node missing");

        self.graph.add_edge(from_idx, to_idx, ());
        self
    }
}


fn main() {
    let mut model = FCM::new()
        .node("A")
        .node("B")
        .edge("A", "B")
        .rule("B", |pa: &[Value]| pa.iter().sum());

    println!("Graph ready: {} nodes.", model.graph.node_count());
}