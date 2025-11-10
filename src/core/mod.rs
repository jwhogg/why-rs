use petgraph::{Graph, Directed};
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::NodeIndex;
use petgraph::graph::EdgeIndex;
use std::collections::HashMap;
use polars::prelude::*;

use crate::error::CausalError;


pub enum NoiseDistribution {
    Gaussian(f64, f64),
    Uniform(f64, f64),
}

pub type Variable = String;

pub struct CausalGraph {
    graph: Graph<Variable, ()>
}

impl CausalGraph {
    pub fn new() -> Self { CausalGraph { graph: Graph::new() } }

    pub fn add_node(&mut self, variable: Variable) {
        todo!()
    }

    pub fn add_edge(&mut self, from: Variable, to: Variable) {
        todo!()
    }

    pub fn get_nodes(&self) -> Vec<Variable> {
        todo!()
    }

    pub fn get_edges(&self) -> Vec<Variable> {
        todo!()
    }

    pub fn get_parents(&self, node: &Variable) -> Vec<Variable> {
        todo!()
    }

    pub fn is_root(&self, node: &Variable) -> bool {
        todo!()
    }

    pub fn is_d_seperated(&self, x: &Variable, y: &Variable, z: Vec<Variable>) -> bool {
        // Returns true if there is no directed path between x and y that the set z isn't blocking
        todo!()
    }
}

pub trait Function {
    fn fit(&mut self, data: &DataFrame, parents: &HashMap<Variable, usize>, noise_distribution: &NoiseDistribution);
    fn evaluate(&self, data: &DataFrame, parents: &HashMap<Variable, usize>, noise: &NoiseDistribution) -> usize;
}

// TODO: impl linearFunction for Function

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


pub struct FunctionalCausalMechanism {
    //Defines the equation Y = f(X,N)
    // SCM will use a FCM for each non-root node

    parents: HashMap<Variable, usize>,
    noise_distribution: NoiseDistribution,
    // function: f(PA_x, N) = Y, Hashmap<variable, usize> is Parent Node name, and its value
    function: Box<dyn Function>
}

impl Mechanism for FunctionalCausalMechanism {
    fn parents(&self) {
        &self.parents;
    }

    fn fit(&mut self, data: DataFrame) {
        &self.function.fit(&data, &self.parents, &self.noise_distribution);
    }

    fn evaluate(&self, data: DataFrame) -> DataFrame {
        &self.function.evaluate(&data, &self.parents, &self.noise_distribution);
    }

    fn draw_noise_samples(&self, n: NoiseDistribution) -> Vec<DataFrame> {
        todo!()
    }
}

pub trait CausalModel {
    fn graph(&self) -> CausalGraph;
    fn variables(&self) -> HashMap<Variable, usize>;
}

pub trait ProbabilisticCausalModel: CausalModel { //TODO: is this the right name for what this is?
    fn equations(&self) -> Vec<dyn Mechanism>;
    // fn intervene?
}

pub struct FunctionalCausalModel {
    pub graph: CausalGraph,
    pub variables: HashMap<Variable, usize>,
    pub mechanisms: Vec<FunctionalCausalMechanism>,
}

impl CausalModel for FunctionalCausalModel {
    fn graph(&self) -> CausalGraph {
        self.graph
    }

    fn variables(&self) -> HashMap<Variable, usize> {
        self.variables.clone()
    }
}

impl ProbabilisticCausalModel for FunctionalCausalModel {
    fn equations(&self) -> Vec<FunctionalCausalMechanism> {
        self.mechanisms
    }
}

// TODO: invertiblemodel, implement a function type





