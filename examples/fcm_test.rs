use why_rs::fcm::FCM;
use why_rs::dag::DAG;
use why_rs::dag;
use why_rs::fcm::multivariate_linear;
use rand::Rng;

fn main() {
    let dag: DAG = dag!(
            "A" => "C",
            "B" => "C"
        );

    let fcm = FCM::from_dag(dag);

    let noise_c = rand::thread_rng().gen_range(0..5);
    fcm.rule("C", multivariate_linear(vec![0.2 as u32, 0.5 as u32], 2, noise_c));
}