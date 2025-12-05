use why_rs::fcm::FCM;
use why_rs::dag::{Value, Variable, DAG};
use why_rs::dag;
use why_rs::fcm::{multivariate_linear, empirical_root};
use rand::Rng;

fn main() {
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
    println!("{}", result);

    // let m = fcm.get_mechanism(&Variable::from("C")).unwrap();
    // let inputs = vec![10, 20]; // Value of A=10, Value of B=20
    //
    // // 3. Execute the closure
    // let result = m(&inputs);
    // println!("Input: A=10, B=20");
    // println!("Calculation: (10 * 0.2) + (20 * 0.5) + bias(2) + noise({})", noise_c);
    // println!("Output: {}", result);
}