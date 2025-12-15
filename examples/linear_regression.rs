use why_rs::fcm::FCM;
use why_rs::dag::{Value, Variable, DAG};
use why_rs::dag;
use why_rs::mechanism::{EmpiricalRoot, LinearRegression};
use polars::prelude::*;
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
        Series::new("A", a),
        Series::new("B", b),
        Series::new("C", c),
        Series::new("D", d),
    ])
        .unwrap()
}

fn main() {
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
    let a_history: Vec<Value> = df
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
    c_lr.fit(df.clone(), Variable::from("C"));

    let mut d_lr = LinearRegression::new();
    d_lr.fit(df.clone(), Variable::from("D"));

    fcm = fcm
        .rule("C", c_lr)
        .rule("D", d_lr);

    // 6. Sample
    let samples = fcm.sample(15);
    println!("{}", samples);
}
