use why_rs::fcm::FCM;
use why_rs::dag::Value;

fn main() {
    let mut model = FCM::new()
        .node("A")
        .node("B")
        .edge("A", "B")
        .rule("B", |pa: &[Value]| pa.iter().sum());

    println!("Graph ready: {} nodes.", model.graph.node_count());
}