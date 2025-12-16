use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use petgraph::graph::DiGraph;
use polars::frame::DataFrame;
use polars::prelude::{Column, NamedFrom, PlSmallStr, Series};
pub use crate::dag::{DAG, Variable, Value};
use rand::Rng;
use crate::intervention::Intervention;
use petgraph::algo::toposort;
use crate::mechanism::Mechanism;

pub struct FCM {
    pub graph: DAG,
    // &[] means the function takes a slice args, so any length
    // Box is because we don't know func length at compile time, we move it to the heap
    pub mechanisms: HashMap<Variable, Box<dyn Mechanism>>,
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
            mechanisms: HashMap::new(),
        }
    }

    pub fn from_dag(graph: DAG) -> Self {
        FCM {
            graph,
            mechanisms: HashMap::new(),
        }
    }

    pub fn rule<M: Mechanism + 'static>(mut self, target: &str, mech: M) -> Self {
        self.mechanisms.insert(Variable::from(target), Box::new(mech));
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

    pub fn get_mechanism(&mut self, variable: &Variable) -> Option<&mut Box<dyn Mechanism>> {
        self.mechanisms.get_mut(variable)
    }
    pub fn generate(&mut self, variable: &Variable, parent_data: &[Value]) -> Value {
        let mechanism = self.get_mechanism(variable)
            .expect("No mechanism defined for variable");
        mechanism.predict(parent_data.to_vec())
    }

    pub fn interventional_samples(
        &mut self,
        interventions: Vec<Intervention>,
        n_samples: usize
    ) -> DataFrame {

        // 0. Pre-process interventions
        let intervention_map: HashMap<Variable, Value> = interventions
            .into_iter()
            .map(|i| (i.variable, i.value))
            .collect();

        // 1. Determine Calculation Order
        let ordered = self.graph.sort();

        // 2. Initialize Empty Dataset
        let mut data_store: HashMap<Variable, Vec<Value>> = HashMap::new();
        for node in self.graph.variables() {
            data_store.insert(node.clone(), Vec::with_capacity(n_samples));
        }

        // 3. Loop: Generate N samples
        for _ in 0..n_samples {
            for node in &ordered {
                // Check for Intervention
                if let Some(forced_value) = intervention_map.get(node) {
                    // "Graph Surgery": Force value, ignore parents
                    data_store.get_mut(node).unwrap().push(forced_value.clone());
                } else {
                    // Standard Logic
                    let mut parents = self.graph.get_parents(node);
                    parents.sort();

                    let parent_values: Vec<Value> = parents.iter()
                        .map(|p_name| {
                            data_store.get(p_name).unwrap().last().unwrap().clone()
                        })
                        .collect();

                    let new_value = if let Some(mechanism) = self.get_mechanism(node) {
                        mechanism.predict(parent_values)
                    } else {
                        0.0 // Fallback
                    };

                    data_store.get_mut(node).unwrap().push(new_value);
                }
            }
        }

        // 4. CONVERT TO DATAFRAME
        let columns: Vec<Column> = data_store
            .into_iter()
            .map(|(var_name, values)| {
                // Unwrap Vec<Value> into Vec<f64>
                let raw_values: Vec<f64> = values.into_iter().map(|v| v).collect();

                // Create Series and convert it to Column using .into()
                Series::new(PlSmallStr::from(var_name), raw_values).into()
            })
            .collect();

        DataFrame::new(columns).expect("Failed to create DataFrame")
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
                    mechanism.predict(parent_values)
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