use crate::dag::{Variable, DAG};
use polars::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::consts::SQRT_2;
use petgraph::visit::EdgeRef;

// Use your existing structs

/// A "Proper" PC Algorithm implementation
pub struct PC {
    pub graph: DAG,
    pub sepsets: HashMap<(Variable, Variable), HashSet<Variable>>,
    data: DataFrame,
    correlation_matrix: HashMap<(Variable, Variable), f64>,
    n_samples: usize,
}

impl PC {
    pub fn new(variables: Vec<Variable>, data: DataFrame) -> Self {
        let n_samples = data.height();

        // 1. Precompute Correlation Matrix (Optimization)
        // Calculating Pearson correlation once makes the recursive steps faster.
        let mut corr_map = HashMap::new();
        for i in 0..variables.len() {
            for j in 0..variables.len() {
                if i == j { continue; }
                let u = &variables[i];
                let v = &variables[j];

                // Get columns from Polars
                let s_u = data.column(u).unwrap().f64().unwrap();
                let s_v = data.column(v).unwrap().f64().unwrap();

                // Calculate Pearson Correlation
                // Note: Polars specific API
                let corr = pearson_corr(s_u, s_v);
                corr_map.insert((u.clone(), v.clone()), corr);
            }
        }

        // 2. Initialize Complete Graph (Undirected)
        // We represent A - B as edges A->B AND B->A
        let mut dag = DAG::new();
        for v in &variables { dag.add_node(v.clone()); }

        for i in 0..variables.len() {
            for j in (i + 1)..variables.len() {
                let u = &variables[i];
                let v = &variables[j];
                dag = dag.edge(u, v).edge(v, u);
            }
        }

        PC {
            graph: dag,
            sepsets: HashMap::new(),
            data,
            correlation_matrix: corr_map,
            n_samples,
        }
    }

    /// Run the full pipeline
    pub fn run(&mut self, alpha: f64) {
        println!("Phase 1: Learning Skeleton...");
        self.learn_skeleton(alpha);

        println!("Phase 2: Orienting Colliders...");
        self.orient_colliders();

        println!("Phase 3: Propagating Directions (Meek Rules)...");
        self.orient_meek_rules();
    }

    // ==========================================
    // Phase 1: Skeleton Discovery
    // ==========================================
    fn learn_skeleton(&mut self, alpha: f64) {
        let mut depth = 0;
        loop {
            let mut edges_removed_this_round = false;
            let nodes = self.graph.variables();

            // We collect edges to remove first to avoid mutating graph while iterating
            let mut removals = Vec::new();

            for x in &nodes {
                let neighbors = self.get_neighbors(x);
                for y in &neighbors {
                    // Enforce Order to avoid duplicate checks (check A-B, skip B-A)
                    if x >= y { continue; }

                    // Potential conditioning sets are neighbors of X (excluding Y)
                    let adj_x: Vec<Variable> = neighbors.iter()
                        .filter(|&n| n != y)
                        .cloned()
                        .collect();

                    if adj_x.len() < depth { continue; }

                    // Check all subsets of size `depth`
                    let combos = get_combinations(&adj_x, depth);
                    for sepset in combos {
                        // STATISTICAL TEST
                        if self.is_independent(x, y, &sepset, alpha) {
                            // Found separation!
                            removals.push((x.clone(), y.clone()));

                            // Record SepSet
                            let mut key = vec![x.clone(), y.clone()];
                            key.sort();
                            let sep_set_data: HashSet<Variable> = sepset.into_iter().collect();
                            self.sepsets.insert((key[0].clone(), key[1].clone()), sep_set_data);

                            edges_removed_this_round = true;
                            break; // Stop looking for other sepsets for this edge
                        }
                    }
                }
            }

            // Apply removals
            for (u, v) in removals {
                self.remove_undirected_edge(&u, &v);
            }

            if !edges_removed_this_round {
                // If we went through a whole depth and removed nothing, or
                // max degree is smaller than depth, we stop.
                let max_degree = nodes.iter().map(|n| self.get_degree(n)).max().unwrap_or(0);
                if depth > max_degree { break; }
            }

            depth += 1;
        }
    }

    // ==========================================
    // Phase 2: Unshielded Colliders
    // ==========================================
    fn orient_colliders(&mut self) {
        let triples = self.find_unshielded_triples();
        for (x, y, z) in triples {
            let mut key = vec![x.clone(), z.clone()];
            key.sort();
            let k_tuple = (key[0].clone(), key[1].clone());

            // Check if Y is in the Separation Set of (X, Z)
            let default_set = HashSet::new();
            let sepset = self.sepsets.get(&k_tuple).unwrap_or(&default_set);

            if !sepset.contains(&y) {
                // Orient X -> Y <- Z
                self.orient_directed(&x, &y);
                self.orient_directed(&z, &y);
                println!("Oriented Collider: {} -> {} <- {}", x, y, z);
            }
        }
    }

    // ==========================================
    // Phase 3: Meek Rules (Propagation)
    // ==========================================
    fn orient_meek_rules(&mut self) {
        let mut change = true;
        while change {
            change = false;
            let nodes = self.graph.variables();

            // Rule 1: X -> Y - Z  =>  X -> Y -> Z
            // (If X->Y and Y-Z (undirected), and X not connected to Z)
            for y in &nodes {
                let y_parents = self.graph.get_parents(y); // Directed incoming
                let y_neighbors = self.get_undirected_neighbors(y); // Undirected connected

                for x in &y_parents {
                    for z in &y_neighbors {
                        if !self.are_adjacent(x, z) {
                            // Apply Rule 1
                            self.orient_directed(y, z);
                            change = true;
                            println!("Meek Rule 1: {} -> {} -> {}", x, y, z);
                        }
                    }
                }
            }

            // (Rule 2 and 3 can be added here for completeness, but Rule 1 is the workhorse)
        }
    }

    // ==========================================
    // Statistical Math (Fisher Z-Test)
    // ==========================================
    fn is_independent(&self, x: &str, y: &str, z: &[Variable], alpha: f64) -> bool {
        // 1. Calculate Partial Correlation
        let r = self.partial_correlation(x, y, z);

        // 2. Fisher Z-Transform
        if r.abs() >= 1.0 { return false; } // Correlation of 1 means dependent
        let z_stat = 0.5 * ((1.0 + r) / (1.0 - r)).ln();

        // 3. Z-Test Statistic
        // Degrees of freedom = N - |Z| - 3
        let n = self.n_samples as f64;
        let k = z.len() as f64;
        let scale = (n - k - 3.0).sqrt();
        let statistic = (scale * z_stat).abs();

        // 4. Critical Value (Standard Normal)
        // Approx for alpha=0.05 is 1.96.
        // We can use a simple inverse error function approximation or hardcode thresholds.
        let critical_val = match alpha {
            0.01 => 2.576,
            0.05 => 1.960,
            0.10 => 1.645,
            _ => 1.960 // Default
        };

        // If statistic < critical, we cannot reject Null Hypothesis (Independence)
        // Therefore: They ARE Independent.
        statistic < critical_val
    }

    /// Recursive Partial Correlation: rho_xy|z
    fn partial_correlation(&self, x: &str, y: &str, z: &[Variable]) -> f64 {
        if z.is_empty() {
            return *self.correlation_matrix.get(&(x.to_string(), y.to_string())).unwrap_or(&0.0);
        }

        // Pop last element of Z
        let k = &z[0];
        let z_rest = &z[1..];

        let r_xy = self.partial_correlation(x, y, z_rest);
        let r_xk = self.partial_correlation(x, k, z_rest);
        let r_yk = self.partial_correlation(y, k, z_rest);

        let num = r_xy - (r_xk * r_yk);
        let den = ((1.0 - r_xk.powi(2)) * (1.0 - r_yk.powi(2))).sqrt();

        if den == 0.0 { 0.0 } else { num / den }
    }

    // ==========================================
    // Graph Helpers
    // ==========================================
    fn get_neighbors(&self, node: &str) -> Vec<Variable> {
        let idx = self.graph.get_index(&node.to_string()).unwrap();
        // Standard Neighbors (Incoming + Outgoing)
        self.graph.graph.neighbors_undirected(idx)
            .map(|i| self.graph.graph[i].clone())
            .collect()
    }

    // Specifically finds 'Y' where X - Y exists (bidirectional)
    fn get_undirected_neighbors(&self, node: &str) -> Vec<Variable> {
        let idx = self.graph.get_index(&node.to_string()).unwrap();
        let mut res = Vec::new();
        for neighbor_idx in self.graph.graph.neighbors_undirected(idx) {
            let neighbor_name = self.graph.graph[neighbor_idx].clone();
            // Check for edge back
            if self.graph.graph.contains_edge(idx, neighbor_idx) &&
                self.graph.graph.contains_edge(neighbor_idx, idx) {
                res.push(neighbor_name);
            }
        }
        res
    }

    fn get_degree(&self, node: &str) -> usize {
        self.get_neighbors(node).len()
    }

    fn remove_undirected_edge(&mut self, u: &str, v: &str) {
        let u_idx = self.graph.get_index(&u.to_string()).unwrap();
        let v_idx = self.graph.get_index(&v.to_string()).unwrap();
        if let Some(e) = self.graph.graph.find_edge(u_idx, v_idx) { self.graph.graph.remove_edge(e); }
        if let Some(e) = self.graph.graph.find_edge(v_idx, u_idx) { self.graph.graph.remove_edge(e); }
    }

    // Sets X -> Y (removes Y -> X)
    fn orient_directed(&mut self, x: &str, y: &str) {
        let u_idx = self.graph.get_index(&x.to_string()).unwrap();
        let v_idx = self.graph.get_index(&y.to_string()).unwrap();

        // Remove Y -> X
        if let Some(e) = self.graph.graph.find_edge(v_idx, u_idx) {
            self.graph.graph.remove_edge(e);
        }
        // Ensure X -> Y
        if self.graph.graph.find_edge(u_idx, v_idx).is_none() {
            self.graph.graph.add_edge(u_idx, v_idx, ());
        }
    }

    fn are_adjacent(&self, x: &str, y: &str) -> bool {
        let u_idx = self.graph.get_index(&x.to_string()).expect("Node missing");
        let v_idx = self.graph.get_index(&y.to_string()).expect("Node missing");
        self.graph.graph.contains_edge(u_idx, v_idx) || self.graph.graph.contains_edge(v_idx, u_idx)
    }

    fn find_unshielded_triples(&self) -> Vec<(Variable, Variable, Variable)> {
        let mut triples = Vec::new();
        let nodes = self.graph.variables();
        for y in &nodes {
            let neighbors = self.get_neighbors(y);
            if neighbors.len() < 2 { continue; }

            for i in 0..neighbors.len() {
                for j in (i+1)..neighbors.len() {
                    let x = &neighbors[i];
                    let z = &neighbors[j];
                    if !self.are_adjacent(x, z) {
                        triples.push((x.clone(), y.clone(), z.clone()));
                    }
                }
            }
        }
        triples
    }
}

// --- Utilities ---

fn pearson_corr(a: &ChunkedArray<Float64Type>, b: &ChunkedArray<Float64Type>) -> f64 {
    let mean_a = a.mean().unwrap_or(0.0);
    let mean_b = b.mean().unwrap_or(0.0);

    let mut num = 0.0;
    let mut den_a = 0.0;
    let mut den_b = 0.0;

    for (val_a, val_b) in a.into_iter().zip(b.into_iter()) {
        if let (Some(va), Some(vb)) = (val_a, val_b) {
            let da = va - mean_a;
            let db = vb - mean_b;
            num += da * db;
            den_a += da * da;
            den_b += db * db;
        }
    }

    if den_a == 0.0 || den_b == 0.0 { 0.0 } else { num / (den_a.sqrt() * den_b.sqrt()) }
}

fn get_combinations(pool: &[Variable], k: usize) -> Vec<Vec<Variable>> {
    if k == 0 { return vec![vec![]]; }
    if pool.is_empty() { return vec![]; }

    let mut ret = Vec::new();
    let head = pool[0].clone();
    let tail = &pool[1..];

    // Include head
    for mut subset in get_combinations(tail, k - 1) {
        subset.insert(0, head.clone());
        ret.push(subset);
    }
    // Exclude head
    ret.extend(get_combinations(tail, k));
    ret
}