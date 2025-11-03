use crate::core::CausalGraph;
use crate::error::CausalError;
use polars::prelude::*;
use petgraph::algo::toposort;
use rand::Rng;
use rand::distr::{Distribution, StandardUniform};
use std::collections::HashMap;

/// Defines the functional assignment for a node in the SCM.
/// For v1.0, we only support linear models.
pub enum FunctionalAssignment {
    Linear {
        /// Map of: parent_name -> coefficient
        coefficients: HashMap<String, f64>,
        noise_std: f64,
    }
}

/// A Structural Causal Model (SCM) containing a graph and assignments.
pub struct StructuralCausalModel {
    pub graph: CausalGraph,
    pub assignments: HashMap<String, FunctionalAssignment>,
}

impl StructuralCausalModel {

}