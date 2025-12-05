use why_rs::fcm::FCM;
use why_rs::dag::{Variable, DAG};
use why_rs::dag;
use why_rs::fcm::multivariate_linear;
use rand::Rng;

fn main() {
    let dag: DAG = dag!(
            "A" => "C",
            "B" => "C"
        );

    let mut fcm = FCM::from_dag(dag);

    let noise_c = rand::thread_rng().gen_range(0..5);
    fcm = fcm.rule("C", multivariate_linear(vec![0.2 as u32, 0.5 as u32], 2, noise_c));

    let m = fcm.get_mechanism(&Variable::from("C")).unwrap();
    let inputs = vec![10, 20]; // Value of A=10, Value of B=20

    // 3. Execute the closure
    let result = m(&inputs);
    println!("Input: A=10, B=20");
    println!("Calculation: (10 * 0.2) + (20 * 0.5) + bias(2) + noise({})", noise_c);
    println!("Output: {}", result);
}