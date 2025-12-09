use why_rs::dag::DAG;

fn main() {
    //diagram.dot is as follows:
    // digraph G {
    //     A -> B -> C;
    //     B -> D;
    //     A -> D;
    //     D -> E;
    //     C -> E;
    //
    //     A [label="Start"];
    //     E [label="End"];
    // }
    let dag = DAG::from_dot("examples/diagram.dot").unwrap();
    println!("{}", dag);
}