use why_rs::pc::PC;
use why_rs::dag::{Value, DAG};
use why_rs::{dag};
use rand::Rng;
use polars::prelude::*;
use why_rs::fcm::{empirical_root, multivariate_linear, FCM};
use why_rs::intervene;
use why_rs::intervention::Intervention;

fn main() {
    // let intervention = intervene!('A': 2, 'B': 0);
    // println!("{:?}", intervention);

    let dag: DAG = dag!(
            "A" => "C",
            "B" => "C",
            "C" => "D"
        );

    let mut fcm = FCM::from_dag(dag);

    let noise_c = rand::thread_rng().gen_range(0..5);
    let noise_d = rand::thread_rng().gen_range(0..5);
    fcm = fcm.rule("A", empirical_root(vec![1, 2, 1, 2, 2, 1].into_iter().map(|x| x as Value).collect()));
    fcm = fcm.rule("B", empirical_root(vec![0,1,0,0,1,1,1,0].into_iter().map(|x| x as Value).collect()));
    fcm = fcm.rule("C", multivariate_linear(vec![0.33, 0.5], 3, noise_c));
    fcm = fcm.rule("D", multivariate_linear(vec![0.8], 1, noise_d));

    let result = fcm.sample(15);
    println!("normal sampling: {}", result);

    let df = fcm.interventional_samples(
        intervene!('B': 0), // The Macro in action
        5 // n_samples
    );

    println!("{}", df);
}