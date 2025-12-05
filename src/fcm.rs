use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use petgraph::graph::DiGraph;
use polars::frame::DataFrame;
use polars::prelude::{Column, NamedFrom, PlSmallStr, Series};
pub use crate::dag::{DAG, Variable, Value};
use rand::Rng;

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

    pub fn from_dag(graph: DAG) -> Self {
        FCM {
            graph,
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
        if self.graph.get_index(&Variable::from(name)).is_none() {
            self.graph.add_node(Variable::from(name));
        }
        self
    }

    pub fn edge(mut self, from: &str, to: &str) -> Self {
        let from_idx = self.graph.get_index(&Variable::from(from)).expect("Source node missing");
        let to_idx = self.graph.get_index(&Variable::from(to)).expect("Target node missing");

        self.graph.add_edge(from_idx, to_idx, ());
        self
    }

    pub fn get_mechanism(&mut self, variable: &Variable) -> Option<&mut Box<dyn FnMut(&[Value]) -> Value>> {
        self.functions.get_mut(variable)
    }


    pub fn sample(&mut self, n_samples: usize) -> DataFrame {
        // 1. Determine Calculation Order
        let ordered = self.graph.sort();

        // 2. Initialize Empty Dataset
        // We use a HashMap to hold vectors of values (columns)
        let mut data_store: HashMap<Variable, Vec<Value>> = HashMap::new();

        // Initialize vectors for every node to avoid unwrap errors later
        for node in self.graph.variables() {
            data_store.insert(node, Vec::with_capacity(n_samples));
        }

        // 3. Iterate to generate N samples (Rows)
        for _ in 0..n_samples {

            // Iterate through nodes in causal order (Columns)
            for node in &ordered {

                // # A. Gather Parent Data
                // Get parents from the graph
                let mut parents = self.graph.get_parents(node);

                // CRITICAL: Sort parents alphabetically so they match
                // the order expected by your multivariate_linear function
                parents.sort();

                // Retrieve the values for these parents for the CURRENT row.
                // We use .last() because we are currently building this row.
                let parent_values: Vec<Value> = parents.iter()
                    .map(|p_name| {
                        *data_store.get(p_name)
                            .expect("Parent column missing")
                            .last()
                            .expect("Parent value missing for this row")
                    })
                    .collect();

                // # B. Apply the Structural Equation
                // Fetch the mutable function (mechanism)
                let new_value = if let Some(mechanism) = self.get_mechanism(node) {
                    // Execute the function with the parent values
                    // (Noise is generated INSIDE this mechanism)
                    mechanism(&parent_values)
                } else {
                    // Fallback if no rule is defined (e.g., default to 0)
                    0 as Value
                };

                // # C. Store result
                data_store.get_mut(node).unwrap().push(new_value);
            }
        }

        // 4. Return as DataFrame
        // Convert HashMap<String, Vec<u32>> into Vec<Series>
        let columns: Vec<Column> = data_store
            .into_iter()
            .map(|(name, values)| Series::new(PlSmallStr::from(&name), values).into())
            .collect();

        DataFrame::new(columns).expect("Failed to create DataFrame")
    }
}

// ----- Mechanisms: ---------------
pub fn multivariate_linear(weights: Vec<f64>, bias: u32, noise: u32) -> impl FnMut(&[Value]) -> Value {
    move |parents: &[Value]| {
        let mut total = bias as f64;

        // Iterate over parents and weights simultaneously
        // .zip() stops at the shortest list, preventing index out of bounds
        for (parent_val, weight) in parents.iter().zip(weights.iter()) {
            total += parent_val * weight;
        }

        // Add Noise (e.g., random 0-5)
        // let noise = rand::thread_rng().gen_range(0..5);
        total + noise as f64
    }
}

pub fn empirical_root(history: Vec<Value>) -> impl FnMut(&[Value]) -> Value { //randomly choose a value from the history of values for that variable
    // Safety check
    if history.is_empty() {
        panic!("Cannot create an empirical root with empty data!");
    }

    move |_parents| {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..history.len());
        history[index]
    }
}

// ----------------------------------