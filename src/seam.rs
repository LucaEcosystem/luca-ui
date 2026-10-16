//! Smallest Luca Code integration seam (one-way).
//!
//! Luca UI depends on Luca Code; Luca Code never depends on Luca UI.
//! The seam reuses the published `luca_code` library API only:
//! `luca_code::interpreter` execution plus `luca_code::error::LucaError`
//! for source-located diagnostics. It does not copy the compiler or
//! interpreter and does not change `.lucc` behavior. `.lucu` files are
//! handled here (extension enforcement + delegation to the UI parser);
//! `.lucc` files stay with the `luca` CLI.
//!
//! Execution uses the non-terminating `run_outcome` API so guest code can
//! never terminate the embedding host: a guest `System.exit(code)` is
//! returned as an exit-marked `Err` (detectable via `is_exit()`) instead
//! of killing the process. Buffered output on exit is discarded, matching
//! how ordinary runtime errors already discard buffered output.

use std::path::Path;

use crate::{Component, LucaError};
use luca_code::interpreter::{Interpreter, RunOutcome};

/// Run paired Luca Code logic and collect its `print` output.
///
/// A guest `System.exit(code)` returns `Err` carrying the exit marker
/// (`is_exit()` is true, message `System.exit(code)`) rather than
/// terminating the host. Location is reported as 1:1, consistent with the
/// other synthesized diagnostics in this module.
pub fn run_logic(source: &str) -> Result<Vec<String>, LucaError> {
    match Interpreter::new().run_outcome(source) {
        Ok(RunOutcome::Completed(output)) => Ok(output),
        Ok(RunOutcome::Exited { code, .. }) => Err(LucaError::exit(code, 1, 1)),
        Err(error) => Err(error),
    }
}

/// Run paired logic with scripted `ask` input lines.
///
/// Same exit contract as [`run_logic`]: guest `System.exit` is returned,
/// never executed as host termination.
pub fn run_logic_with_input(source: &str, input: Vec<String>) -> Result<Vec<String>, LucaError> {
    match Interpreter::new().with_input(input).run_outcome(source) {
        Ok(RunOutcome::Completed(output)) => Ok(output),
        Ok(RunOutcome::Exited { code, .. }) => Err(LucaError::exit(code, 1, 1)),
        Err(error) => Err(error),
    }
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
