// Declare all your main modules
// This tells Rust to look for `src/core/mod.rs` (or `src/core.rs`)
pub mod core;
pub mod io;
pub mod stats;
pub mod scm;
pub mod discovery;
pub mod error;

// You can also "re-export" the most important items
// so users can access them easily.
// e.g., `use causal_rs::CausalGraph` instead of `use causal_rs::core::CausalGraph`
pub use core::CausalGraph;
pub use io::load_csv;
pub use scm::StructuralCausalModel;
pub use discovery::pc_algorithm; // Assuming the fn is `pc_algorithm`
pub use error::CausalError;