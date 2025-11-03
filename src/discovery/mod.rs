// 1. DECLARE SUB-MODULES
// This tells Rust to look for `src/discovery/pc.rs`
pub mod pc;

// 2. RE-EXPORT
// This re-exports the main function from the `pc` submodule
// so `lib.rs` and users can access it easily.
pub use pc::pc_algorithm;