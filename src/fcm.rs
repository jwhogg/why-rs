use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use petgraph::graph::DiGraph;
use polars::frame::DataFrame;
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


    pub fn sample(&self) -> DataFrame {
        // FUNCTION Sample_FCM(FCM, N):
        //
        // # 1. Determine Calculation Order
        // # We must compute parents before children.
        // # A Topological Sort linearizes the DAG (e.g., X -> Z -> Y becomes [X, Z, Y])
        // Order = TopologicalSort(FCM.Graph)
        //

        let ordered = self.graph.sort();
        let mut df = DataFrame::new(vec![]).unwrap();

        for node in ordered.iter() {

        }

        df
        // # 2. Initialize Empty Dataset
        // # Structure to hold columns of data
        // Data = NewDictionary()
        //
        // # 3. Iterate through nodes in causal order
        // for each Node X in Order:
        //
        // # A. Sample Noise/Exogenous variables
        // # Generate a vector of size N from the specific distribution for X
        // # e.g., Normal(0,1), Uniform, etc.
        //     U_x = Sample(FCM.NoiseDistributions[X], size=N)
        //
        // # B. Gather Parent Data
        // # Retrieve the data columns for X's parents that we already computed
        // # (Because of topological sort, parents are guaranteed to be in Data)
        // Parents = FCM.Graph.GetParents(X)
        // Parent_Data = [Data[P] for P in Parents]
        //
        // # C. Apply the Structural Equation
        // # X = f(Parents, Noise)
        // # This applies the function element-wise across the vectors
        // X_values = FCM.Functions[X](Parent_Data, U_x)
        //
        // # D. Store result
        // Data[X] = X_values
        //
        // # 4. Return as DataFrame/Table
        // return DataFrame(Data)
    }
}

// ----- Mechanisms: ---------------
pub fn multivariate_linear(weights: Vec<u32>, bias: u32, noise: u32) -> impl FnMut(&[Value]) -> Value {
    move |parents: &[Value]| {
        let mut total = bias;

        // Iterate over parents and weights simultaneously
        // .zip() stops at the shortest list, preventing index out of bounds
        for (parent_val, weight) in parents.iter().zip(weights.iter()) {
            total += parent_val * weight;
        }

        // Add Noise (e.g., random 0-5)
        // let noise = rand::thread_rng().gen_range(0..5);
        total + noise
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