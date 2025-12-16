use why_rs::pc::PC;
use why_rs::dag::{Value, DAG};
use why_rs::{dag};
use rand::Rng;
use polars::prelude::*;
use why_rs::fcm::FCM;
use why_rs::mechanism::{EmpiricalRoot, LinearRegression};
use why_rs::intervene;
use why_rs::intervention::Intervention;

fn main() {
    let dag: DAG = dag!(
            "A" => "C",
            "B" => "C",
            "C" => "D"
        );

    let mut fcm = FCM::from_dag(dag);

    let noise_c = rand::thread_rng().gen_range(0..5);
    let noise_d = rand::thread_rng().gen_range(0..5);
    fcm = fcm.rule("A", EmpiricalRoot::new(vec![1, 2, 1, 2, 2, 1].into_iter().map(|x| x as Value).collect()));
    fcm = fcm.rule("B", EmpiricalRoot::new(vec![0,1,0,0,1,1,1,0].into_iter().map(|x| x as Value).collect()));
    fcm = fcm.rule("C", LinearRegression::from(vec![0.33, 0.5], 3, noise_c));
    fcm = fcm.rule("D", LinearRegression::from(vec![0.8], 1, noise_d));

    let result = fcm.sample(15);
    println!("normal sampling: {}", result);

    let df = fcm.interventional_samples(
        intervene!('B': 0), // The Macro in action
        5 // n_samples
    );

    println!("{}", df);
}