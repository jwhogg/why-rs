use why_rs::intervention::Intervention;
use why_rs::fcm::FCM;
use why_rs::dag::{Value, Variable, DAG};
use why_rs::{dag, intervene};
use why_rs::mechanism::{EmpiricalRoot, LinearRegression};
use polars::prelude::*;
use why_rs::mechanism::Mechanism;
use rand::Rng;

fn generate_data(n: usize) -> DataFrame {
    let mut rng = rand::thread_rng();

    let a: Vec<f64> = (0..n).map(|_| rng.gen_range(0.0..2.0)).collect();
    let b: Vec<f64> = (0..n).map(|_| rng.gen_range(0.0..2.0)).collect();

    let c: Vec<f64> = a.iter()
        .zip(&b)
        .map(|(a, b)| 0.33 * a + 0.5 * b + 3.0 + rng.gen_range(0.0..1.0))
        .collect();

    let d: Vec<f64> = c.iter()
        .map(|c| 0.8 * c + 1.0 + rng.gen_range(0.0..1.0))
        .collect();

    DataFrame::new(vec![
        Column::from(Series::new(PlSmallStr::from("A"), a)),
        Column::from(Series::new(PlSmallStr::from("B"), b)),
        Column::from(Series::new(PlSmallStr::from("C"), c)),
        Column::from(Series::new(PlSmallStr::from("D"), d)),
    ])
        .unwrap()
}

#[test]
fn test_linear_regression_sampling() {
// 1. DAG
    let dag: DAG = dag!(
        "A" => "C",
        "B" => "C",
        "C" => "D"
    );

    // 2. Observational data
    let df = generate_data(500);

    // 3. Build FCM
    let mut fcm = FCM::from_dag(dag);

    // 4. Root mechanisms (empirical, NO .fit)
    let a_history: Vec<Value> = df //TODO: move this logic to the empirical root method
        .column("A").unwrap()
        .f64().unwrap()
        .into_no_null_iter()
        .collect();

    let b_history: Vec<Value> = df
        .column("B").unwrap()
        .f64().unwrap()
        .into_no_null_iter()
        .collect();

    fcm = fcm
        .rule("A", EmpiricalRoot::new(a_history))
        .rule("B", EmpiricalRoot::new(b_history));

    // 5. Learned mechanisms
    let mut c_lr = LinearRegression::new();
    c_lr.fit(df.clone(), Variable::from("C"), &fcm);

    let mut d_lr = LinearRegression::new();
    d_lr.fit(df.clone(), Variable::from("D"), &fcm);

    fcm = fcm
        .rule("C", c_lr)
        .rule("D", d_lr);

    let num_samples: usize = 30;

    // 6. Sample
    let samples = fcm.sample(num_samples);
    assert_eq!(samples.width(), 4, "DataFrame should have 4 columns");
    assert_eq!(samples.height(), num_samples, "DataFrame should have 4 columns height");
}

#[test]
fn test_intervention_propagation() {
    // 1. Setup DAG: A -> C <- B
    let dag = dag!(
            "A" => "C",
            "B" => "C"
        );

    let mut fcm = FCM::from_dag(dag);

    // 2. Setup Mechanisms (Deterministic / Zero Noise)

    // Root A: Always returns 1.0
    fcm = fcm.rule("A", EmpiricalRoot::new(vec![1.0]));

    // Root B: Normally returns 1.0 (We will override this)
    fcm = fcm.rule("B", EmpiricalRoot::new(vec![1.0]));

    // Child C: Linear Combination (A + B)
    // Weights: [1.0, 1.0], Bias: 0.0, Noise: 0.0
    fcm = fcm.rule("C", LinearRegression::from(vec![1.0, 1.0], 0.0, 0.0));

    // 3. Normal Baseline Check (Optional sanity check)
    let normal_df = fcm.sample(1);
    let normal_c = normal_df.column("C").unwrap().f64().unwrap().get(0).unwrap();
    assert_eq!(normal_c, 2.0, "Normally C should be 1+1=2");

    // 4. Apply Intervention: Force B = 100.0
    // "Graph Surgery": The edge B->C remains, but B's value is fixed.
    let df = fcm.interventional_samples(
        intervene!("B": 100.0),
        5
    );

    // 5. Assertions

    // Check Intervened Variable (B)
    let b_vals: Vec<f64> = df.column("B").unwrap().f64().unwrap().into_no_null_iter().collect();
    assert!(b_vals.iter().all(|&x| x == 100.0), "B should be fixed to 100.0");

    // Check Unaffected Parent (A)
    let a_vals: Vec<f64> = df.column("A").unwrap().f64().unwrap().into_no_null_iter().collect();
    assert!(a_vals.iter().all(|&x| x == 1.0), "A should remain 1.0");

    // Check Downstream Effect (C)
    // C = 1.0 * A + 1.0 * B_forced
    // C = 1.0 + 100.0 = 101.0
    let c_vals: Vec<f64> = df.column("C").unwrap().f64().unwrap().into_no_null_iter().collect();
    assert!(c_vals.iter().all(|&x| x == 101.0), "C should be 101.0, reflecting the intervention on B");
}