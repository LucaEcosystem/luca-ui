//! Smallest Luca Code integration seam (one-way).
//!
//! Luca UI depends on Luca Code; Luca Code never depends on Luca UI.
//! The seam reuses the published `luca_code` library API only:
//! `luca_code::run` / `run_with_input` for paired `.lucc` logic and
//! `luca_code::error::LucaError` for source-located diagnostics. It does
//! not copy the compiler or interpreter and does not change `.lucc`
//! behavior. `.lucu` files are handled here (extension enforcement +
//! delegation to the UI parser); `.lucc` files stay with the `luca` CLI.

use std::path::Path;

use crate::{Component, LucaError};

/// Run paired Luca Code logic and collect its `print` output.
pub fn run_logic(source: &str) -> Result<Vec<String>, LucaError> {
    luca_code::run(source)
}

/// Run paired logic with scripted `ask` input lines.
pub fn run_logic_with_input(source: &str, input: Vec<String>) -> Result<Vec<String>, LucaError> {
    luca_code::run_with_input(source, input)
}

/// Enforce the `.lucu` extension (mirrors the `.lucc` enforcement in
/// the Luca Code CLI) and parse + validate the source.
pub fn parse_source(source: &str) -> Result<Component, LucaError> {
    crate::parse(source)
}

/// Load a `.lucu` file, enforcing the extension so `.lucc` user work is
/// never overwritten or reinterpreted by this toolchain.
pub fn load_file(path: &Path) -> Result<Component, LucaError> {
    let name = path.to_string_lossy();
    if !name.ends_with(".lucu") {
        return Err(LucaError::new("Luca UI source files must use the .lucu extension", 1, 1));
    }
    let source = std::fs::read_to_string(path)
        .map_err(|e| LucaError::new(format!("Could not read {}: {e}", path.display()), 1, 1))?;
    crate::parse(&source)
}
