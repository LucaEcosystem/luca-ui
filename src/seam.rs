//! Smallest Luca Code integration seam (one-way).
//!
//! Luca UI depends on Luca Code; Luca Code never depends on Luca UI.
//! The seam reuses the published `luca_code` library API only:
//! `luca_code::interpreter` execution, the public lexer/parser/AST for
//! declaration inspection, and `luca_code::error::LucaError` for
//! source-located diagnostics. It does not copy the compiler or interpreter
//! and does not change `.lucc` behavior. `.lucu` files are handled here
//! (extension enforcement + delegation to the UI parser); `.lucc` files stay
//! with the `luca` CLI.
//!
//! Execution uses the non-terminating `run_outcome` API so guest code can
//! never terminate the embedding host: a guest `System.exit(code)` is
//! returned as an exit-marked `Err` (detectable via `is_exit()`) instead of
//! killing the process. Buffered output on exit is discarded, matching how
//! ordinary runtime errors already discard buffered output.
//!
//! Handler validation is deliberately declaration-only: it parses the
//! paired `.lucc` source with Luca Code's own public parser and checks that
//! every `events:` handler names a top-level `def func`. It does not execute
//! handlers, and it never claims to resolve imported names (see
//! [`HandlerCheck::verified`]).

use std::path::{Path, PathBuf};

use crate::{Component, LucaError};
use luca_code::ast::Statement;
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
        return Err(LucaError::new(
            "Luca UI source files must use the .lucu extension",
            1,
            1,
        ));
    }
    let source = std::fs::read_to_string(path)
        .map_err(|e| LucaError::new(format!("Could not read {}: {e}", path.display()), 1, 1))?;
    crate::parse(&source)
}

/// Load `.lucc` logic source, enforcing the extension.
pub fn load_logic(path: &Path) -> Result<String, LucaError> {
    let name = path.to_string_lossy();
    if !name.ends_with(".lucc") {
        return Err(LucaError::new(
            "Luca Code source files must use the .lucc extension",
            1,
            1,
        ));
    }
    std::fs::read_to_string(path)
        .map_err(|e| LucaError::new(format!("Could not read {}: {e}", path.display()), 1, 1))
}

/// Run a `.lucc` file through the interpreted seam.
pub fn run_logic_file(path: &Path) -> Result<Vec<String>, LucaError> {
    run_logic(&load_logic(path)?)
}

/// The sibling `.lucc` path a `.lucu` file pairs with (`note.lucu` →
/// `note.lucc` in the same directory).
pub fn paired_logic_path(lucu_path: &Path) -> PathBuf {
    lucu_path.with_extension("lucc")
}

/// Load a `.lucu` file and run its sibling `.lucc` logic, if one exists.
///
/// Returns the parsed UI tree plus the logic's collected output. A missing
/// sibling is reported rather than silently skipped, so callers can decide
/// whether logic is optional.
pub fn load_paired(lucu_path: &Path) -> Result<(Component, Vec<String>), LucaError> {
    let tree = load_file(lucu_path)?;
    let logic_path = paired_logic_path(lucu_path);
    if !logic_path.exists() {
        return Err(LucaError::new(
            format!(
                "No paired Luca Code source found at {}",
                logic_path.display()
            ),
            1,
            1,
        ));
    }
    let output = run_logic_file(&logic_path)?;
    Ok((tree, output))
}

/// Top-level function names and whether the source imports other modules.
///
/// Imports are reported, not resolved: the seam has no module search-path
/// contract yet, so a handler may legitimately live in an imported file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerDeclarations {
    pub functions: Vec<String>,
    pub uses_imports: bool,
}

/// One event handler name that the paired source does not declare locally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingHandler {
    pub component: String,
    pub component_id: Option<String>,
    pub event: String,
    pub handler: String,
    pub line: usize,
    pub column: usize,
}

impl MissingHandler {
    /// Source-located diagnostic for this missing declaration.
    pub fn to_error(&self) -> LucaError {
        let target = match &self.component_id {
            Some(id) => format!("component '{id}'"),
            None => format!("component '{}'", self.component),
        };
        LucaError::new(
            format!(
                "Event '{}' on {target} names handler '{}', which is not defined in the paired Luca Code source",
                self.event, self.handler
            ),
            self.line,
            self.column,
        )
    }
}

/// Result of checking every declared event handler against a paired source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerCheck {
    /// True only when the source declares no imports, so every handler name
    /// is checkable locally. False means imported handlers may exist.
    pub verified: bool,
    pub missing: Vec<MissingHandler>,
}

impl HandlerCheck {
    /// True when nothing is known to be missing.
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Collect top-level `def func` names using Luca Code's public parser.
pub fn declared_handlers(source: &str) -> Result<HandlerDeclarations, LucaError> {
    let tokens = luca_code::lexer::lex(source)?;
    let program = luca_code::parser::parse(tokens)?;
    let mut functions = Vec::new();
    let mut uses_imports = false;
    for statement in &program.statements {
        match statement {
            Statement::FuncDef { name, .. } => functions.push(name.clone()),
            Statement::Import { .. } => uses_imports = true,
            _ => {}
        }
    }
    functions.sort();
    functions.dedup();
    Ok(HandlerDeclarations {
        functions,
        uses_imports,
    })
}

/// Check every `events:` handler in `tree` against a paired `.lucc` source.
///
/// Parse failures in the paired source are returned as Luca Code errors.
/// Missing local declarations are reported in [`HandlerCheck::missing`];
/// when the source imports modules, `verified` is false and callers must not
/// treat missing names as errors.
pub fn check_handlers(tree: &Component, logic_source: &str) -> Result<HandlerCheck, LucaError> {
    let declarations = declared_handlers(logic_source)?;
    let mut missing = Vec::new();
    collect_missing(tree, &declarations.functions, &mut missing);
    Ok(HandlerCheck {
        verified: !declarations.uses_imports,
        missing,
    })
}

fn collect_missing(component: &Component, functions: &[String], missing: &mut Vec<MissingHandler>) {
    if let Some(block) = &component.events {
        for event in &block.events {
            if !functions.iter().any(|name| name == &event.handler) {
                missing.push(MissingHandler {
                    component: component.name.clone(),
                    component_id: component.id().map(str::to_owned),
                    event: event.event.clone(),
                    handler: event.handler.clone(),
                    line: event.line,
                    column: event.column,
                });
            }
        }
    }
    for child in &component.children {
        collect_missing(child, functions, missing);
    }
}
