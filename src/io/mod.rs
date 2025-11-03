use polars::prelude::*;
use crate::error::CausalError;
use std::path::Path;

/// Loads a CSV file from a given path into a Polars DataFrame.
pub fn load_csv<P: AsRef<Path>>(path: P) -> Result<DataFrame, CausalError> {
    // The `?` operator will auto-convert PolarsError into CausalError
    // thanks to the `#[from]` in `error.rs`
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some("iris.csv".into()))?
        .finish();
    Ok(df?)
}