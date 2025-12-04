use why_rs::pc::PC;
use why_rs::dag::DAG;
use why_rs::{dag, macros};
use rand::Rng;
use polars::prelude::*;

fn main() {
    let dag: DAG = dag!(
            "A" => "B",
            "B" => "C",
            "A" => "C"
        );

    let mut rng = rand::thread_rng();
    let n = 10;

    let col_a: Vec<i32> = (0..n).map(|_| rng.gen_range(0..100)).collect();
    let col_b: Vec<i32> = (0..n).map(|_| rng.gen_range(0..100)).collect();
    let col_c: Vec<i32> = (0..n).map(|_| rng.gen_range(0..100)).collect();



    let df = df![
        "a" => col_a,
        "b" => col_b,
        "c" => col_c
    ].unwrap();

    println!("{:?}", df);
    let pc_dag = PC(df);
    println!("{}", pc_dag);
}