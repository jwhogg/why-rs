use petgraph::{Graph, Directed};
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::graph::EdgeIndex;
use std::collections::HashMap;
use std::ptr::null;
use polars::prelude::*;

use crate::error::CausalError;

#[derive(Debug, Clone)]
pub enum NoiseDistribution {
    Gaussian(f64, f64),
    Uniform(f64, f64),
}

pub type Variable = String;

pub struct CausalGraph {
    graph: Graph<Variable, ()>,
    index_map: HashMap<Variable, NodeIndex>
}

impl CausalGraph {
    pub fn new() -> Self {
        CausalGraph {
            graph: Graph::new(),
            index_map: HashMap::new()
        }
    }

    pub fn add_node(&mut self, variable: Variable) -> NodeIndex {
        if let Some(idx) = self.index_map.get(&variable) {
            *idx
        } else {
            let idx = self.graph.add_node(variable.clone());
            self.index_map.insert(variable, idx);
            idx
        }
    }

    pub fn add_edge(&mut self, from: Variable, to: Variable) -> Result<(), String> {
        let from_idx = self
            .index_map
            .get(&from)
            .ok_or_else(|| format!("Variable '{}' does not exist in the graph", from))?;

        let to_idx = self
            .index_map
            .get(&to)
            .ok_or_else(|| format!("Variable '{}' does not exist in the graph", to))?;

        self.graph.add_edge(*from_idx, *to_idx, ());
        Ok(())
    }

    pub fn get_nodes(&self) -> Vec<Variable> {
        self.graph.node_weights().cloned().collect()
    }

    /// Returns a vector of edges as (from, to) pairs of variable names.
    pub fn get_edges(&self) -> Vec<(Variable, Variable)> {
        self.graph
            .edge_references()
            .map(|e| {
                let source = self.graph.node_weight(e.source()).unwrap().clone();
                let target = self.graph.node_weight(e.target()).unwrap().clone();
                (source, target)
            })
            .collect()
    }


    pub fn get_parents(&self, node: &Variable) -> Vec<Variable> {
        todo!()
    }

    pub fn get_ancestors(&self, nodes: &[Variable]) -> Vec<Variable> {
        todo!()
    }

    pub fn get_descendants(&self, nodes: &[Variable]) -> Vec<Variable> {
        todo!()
    }

    pub fn get_all_paths(&self, from: &Variable, to: &Variable) -> Vec<Vec<Variable>> {
        todo!()
    }

    pub fn moralize(&self) -> Self {
        todo!()
    }

    pub fn create_latent_projection(&self) -> Self {
        todo!()
    }

    pub fn is_root(&self, node: &Variable) -> bool {
        todo!()
    }

    pub fn is_d_seperated(&self, x: &Variable, y: &Variable, z: Vec<Variable>) -> bool {
        // Returns true if there is no directed path between x and y that the set z isn't blocking
        todo!()
    }

    pub fn can_find_causal_effect(&self, x: &Variable, y: &Variable) -> bool {
        //Checks if the causal effect P(Y∣do(X)) is identifiable using known algorithms (like checking for the existence of a valid adjustment set).
        todo!()
    }

    pub fn get_backdoor_paths(&self, x: &Variable, y: &Variable) -> Vec<Vec<Variable>> {
        //Finds all paths between X and Y that start with a directed edge pointing into X.
        todo!()
    }

    pub fn is_valid_backdoor_set(&self, x: &Variable, y: &Variable, z: &[Variable]) -> bool {
        //Checks if the set Z satisfies the Backdoor Criterion: 1) Z blocks all backdoor paths from X to Y; and 2) No node in Z is a descendant of X that lies on a directed path to Y.
        todo!()
    }

    pub fn find_minimal_backdoor_set(&self, x: &Variable, y: &Variable) -> Option<Vec<Variable>> {
        //Returns the smallest set Z that satisfies the Backdoor Criterion, or None if no such set exists.
        todo!()
    }

    pub fn is_valid_frontdoor_set(&self, x: &Variable, y: &Variable, m: &[Variable]) -> bool {
        // Checks if the set M satisfies the Front-door Criterion: 1) M completely mediates the effect of X on Y; 2) There are no unblocked backdoor paths between X and M; and 3) All backdoor paths between M and Y are blocked by X.
        todo!()
    }

    pub fn find_frontdoor_set(&self, x: &Variable, y: &Variable) -> Option<Vec<Variable>> {
        //Returns a set M that satisfies the Front-door Criterion, or None. This is useful when the simple Backdoor check fails.
        todo!()
    }

    pub fn display(&self) {
        println!("Nodes: {:?}", self.get_nodes());
        println!("Edges: {:?}", self.get_edges());
    }
}





//------------------- Function Trait -----------------------------------------------
pub trait Function {
    fn fit(&mut self, data: &DataFrame, parents: &HashMap<Variable, usize>, noise_distribution: &NoiseDistribution);
    fn evaluate(&self, data: &DataFrame, parents: &HashMap<Variable, usize>, noise: &NoiseDistribution) -> DataFrame;
}

// TODO: impl linearFunction for Function

pub struct LinearFunction {
    pub coefficients: Vec<f64>,
    pub intercept: f64,
}

impl LinearFunction {
    pub fn new() -> Self {
        Self {
            coefficients: vec![],
            intercept: 0.0,
        }
    }
}

impl Function for LinearFunction {
    fn fit(
        &mut self,
        _data: &DataFrame,
        _parents: &HashMap<Variable, usize>,
        _noise_distribution: &NoiseDistribution,
    ) {
        // TODO: perform regression and set coeffs/intercept
    }

    fn evaluate(
        &self,
        _data: &DataFrame,
        _parents: &HashMap<Variable, usize>,
        _noise: &NoiseDistribution,
    ) -> DataFrame {
        // TODO: compute linear combination and return a Series
        DataFrame::empty()
    }
}





//---------------------- Mechanism -----------------------------
pub trait Mechanism {
    //The generic trait for a causal mechanism.
    // ie: a FCM will implement this: Y = f(X,N), where RHS is the mechanism
    fn parents(&self) -> Vec<Variable>;
    fn fit(&mut self, data: DataFrame);
        // eg: Linear Regression function fitting covariates given some data
        // the type of mechanism will need to implement this

    fn evaluate(&self, data: DataFrame) -> DataFrame;
        //evaluates the mechanism given parent values and a noise values

    fn draw_noise_samples(&self, n: NoiseDistribution) -> Vec<DataFrame>;
}

pub trait InvertibleMechanism: Mechanism {
    //Needs to be implemented for counterfactual reasoning
    fn invert(&mut self, y_value: usize, parent_values: &HashMap<Variable, usize>) -> usize;
}





//------------------ Functional Causal Mechanism --------------------------
pub struct FunctionalCausalMechanism {
    //Defines the equation Y = f(X,N)
    // SCM will use a FCM for each non-root node

    parents: HashMap<Variable, usize>,
    noise_distribution: NoiseDistribution,
    // function: f(PA_x, N) = Y, Hashmap<variable, usize> is Parent Node name, and its value
    function: Box<dyn Function>
}


impl FunctionalCausalMechanism {
    pub fn new(
        parents: HashMap<Variable, usize>,
        noise_distribution: NoiseDistribution,
        function: Box<dyn Function>,
    ) -> Self {
        Self {
            parents,
            noise_distribution,
            function,
        }
    }
}

impl Mechanism for FunctionalCausalMechanism {
    fn parents(&self) -> Vec<Variable> {
        self.parents.keys().cloned().collect()
    }

    fn fit(&mut self, data: DataFrame) {
        &self.function.fit(&data, &self.parents, &self.noise_distribution);
    }

    fn evaluate(&self, data: DataFrame) -> DataFrame {
        self.function.evaluate(&data, &self.parents, &self.noise_distribution)
    }

    fn draw_noise_samples(&self, n: NoiseDistribution) -> Vec<DataFrame> {
        todo!()
    }
}




//----------------- Causal Model -----------------------
pub trait CausalModel {
    fn graph(&self) -> &CausalGraph;
    fn variables(&self) -> &HashMap<Variable, usize>;
}

pub trait ProbabilisticCausalModel: CausalModel {
    fn equations(&self) -> &Vec<FunctionalCausalMechanism>;
}

pub struct FunctionalCausalModel {
    pub graph: CausalGraph,
    pub variables: HashMap<Variable, usize>,
    pub mechanisms: Vec<FunctionalCausalMechanism>,
}

impl CausalModel for FunctionalCausalModel {
    fn graph(&self) -> &CausalGraph {
        &self.graph
    }

    fn variables(&self) -> &HashMap<Variable, usize> {
        &self.variables
    }
}

impl ProbabilisticCausalModel for FunctionalCausalModel {
    fn equations(&self) -> &Vec<FunctionalCausalMechanism> {
        &self.mechanisms
    }
}

// TODO: invertiblemodel, implement a function type





