use crate::dag::{Variable, DAG};
use polars::prelude::*;

pub fn PC(df: DataFrame) -> DAG {

    //TODO: write based on causal-learn's implementation: https://github.com/py-why/causal-learn/blob/main/causallearn/search/ConstraintBased/PC.py

    let cols: Vec<String> = df.get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    println!("{:?}", cols);

    let mut dag = DAG::new();

    //create nodes
    for var in &cols {
        dag.add_node(var);
    }

    //create fully connected edges
    for var1 in &cols {
        for var2 in &cols {
            if var1 != var2 {
                let var1_index = dag.get_index(&Variable::from(var1)).unwrap();
                let var2_index = dag.get_index(&Variable::from(var2)).unwrap();
                dag.add_edge(
                    var1_index,
                    var2_index,
                    (),
                );
            }
        }
    }

    dag
}