use polars::prelude::*;

// 1. DECLARE SUB-MODULES
// This tells Rust to look for `src/stats/fisher_z.rs`
pub mod fisher_z;

// 2. DEFINE THE PUBLIC TRAIT
/// A trait for Conditional Independence (CI) tests.
/// All CI tests must implement this, returning a p-value.
pub trait ConditionalIndependenceTest {
    fn test(
        &self,
        data: &DataFrame,
        x: &str, // Variable 1
        y: &str, // Variable 2
        given: &[&str] // The conditioning set
    ) -> f64; // Returns the p-value
}

// 3. RE-EXPORT
// Re-export the concrete implementation from the submodule
// so `lib.rs` can easily access it.
pub use fisher_z::FishersZ;