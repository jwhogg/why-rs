use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use petgraph::graph::DiGraph;
pub use crate::dag::{DAG, Variable, Value};

pub struct FCM {
    pub graph: DAG,
    // &[] means the function takes a slice args, so any length
    // Box is because we don't know func length at compile time, we move it to the heap
    pub functions: HashMap<Variable, Box<dyn FnMut(&[Value]) -> Value>>,
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