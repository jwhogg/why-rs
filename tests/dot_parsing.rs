use std::fs::File;
use std::io::Write;
use why_rs::dag::DAG;
use why_rs::{dag};
use petgraph::algo::is_isomorphic;

#[test]
fn test_from_dot_file_loads_correct_topology() {
    // 1. SETUP: Create a real temporary file with the DOT content
    let file_path = "test_diagram.dot";
    let dot_content = r#"
    digraph G {
        A -> B -> C;
        B -> D;
        A -> D;
        D -> E;
        C -> E;

        A [label="Start"];
        E [label="End"];
    }
    "#;

    {
        let mut file = File::create(file_path).expect("Failed to create temp test file");
        file.write_all(dot_content.as_bytes()).expect("Failed to write to temp test file");
    } // file is closed here automatically

    // 2. EXECUTE: Run the function you are testing
    let loaded_dag = DAG::from_dot(file_path).expect("DAG::from_dot failed to load file");

    // 3. VERIFY: Construct the expected graph manually

    let expected: DAG = dag!(
            "A" => "B",
            "A" => "D",
            "B" => "D",
            "B" => "C",
            "C" => "E",
            "D" => "E"
    );

    assert!(
        is_isomorphic(&loaded_dag.graph, &expected.graph),
        "The graph loaded from the file does not match the expected structure!"
    );

    // 4. CLEANUP: Remove the temp file
    let _ = std::fs::remove_file(file_path);
}