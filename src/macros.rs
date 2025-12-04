#[macro_export]
macro_rules! dag {
    // Match "A" => "B" patterns separated by commas
    ( $( $from:expr => $to:expr ),* ) => {
        {
            let mut dag = DAG::new();
            $(
                dag = dag.node($from).node($to).edge($from, $to);
            )*
            dag
        }
    };
}