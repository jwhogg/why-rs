# 🧩🦀 why-rs: A Causal Inference library for Rust 
![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/jwhogg/why-rs?label=version)
[![CI](https://github.com/jwhogg/why-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jwhogg/why-rs/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/why_rs.svg)](https://crates.io/crates/why_rs)
[![License: MIT OR Apache‑2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](./LICENSE)

### Usage

To use the crate, you can include it in your `cargo.toml`:
```rust
[dependencies]
why_rs = "0.2.0"
```

Or, get the latest version directly from github:

```rust
[dependencies]
my_crate = { git = "https://github.com/jwhogg/why-rs.git" }
```

### Example usage
```rust
    let dag: DAG = dag!( //macro for defining DAGs similar to DOT files
        "A" => "C",
        "B" => "C",
        "C" => "D"
    );

    let mut fcm = FCM::from_dag(dag);

    let df = generate_data(500); //simulate some practice data

    //fit the LR mechanism given training data
    let mut c_mechanism = LinearRegression::new();
    c_mechanism.fit(df.clone(), Variable::from("C"), &fcm);

    //there is also the option to manually define coefficients:
    let noise_d = rand::thread_rng().gen_range(0..5);
    let mut d_mechanism = LinearRegression::from(vec![0.8], 1, noise_d)

    fcm = fcm //for this to actually run, you would need to define rules for A and B- see examples/linear_regression
        .rule("C", c_mechanism)
        .rule("D", d_mechanism);

    let result = fcm.sample(15); //sample 15 rows
    println!("normal sampling: {}", result);

    let intervened_df = fcm.interventional_samples(
    intervene!('B': 0), // The Macro in action
    5 // n_samples
    );

    println!("Intervened Graph: {}", intervened_df);
```

### Running examples
Currently, there is support for the DAG and FCM (Functional Causal Model) data types.
The following functionality is supported:
- Parsing a .dot file
- Sampling from an FCM
- Causal Discovery with the PC algorithm
- Intervening on a FCM
- Plugging-in custom models for FCM mechanisms

#### Try it yourself:

First, clone the repository:
`git clone https://github.com/jwhogg/why-rs.git`
`cd why-rs`

Sampling:
```rust
cargo run --example fcm_test
```

PC algorithm:
```rust
cargo run --example pc_test
```

Interventions:
```rust
cargo run --example intervention
```

Parsing .DOT files:
```rust
cargo run --example io_test
```



