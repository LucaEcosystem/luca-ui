//! Dedicated UI testing API for 0.9 Alpha (SPEC decision 106–115: dedicated
//! UI tests and explicit component identifiers).
//!
//! `TestHarness` drives a parsed tree deterministically with no I/O,
//! randomness, or threads: dispatch events by component id, apply state
//! updates through the single `set`/`flush` path, swap trees, and read back
//! resolved props, labels, visibility, and backend output. Tests address
//! components by `id` only — the harness exposes no component references.

use luca_code::types::Value;

use crate::{
    describe_resolved, dispatch, is_visible, Dispatch, LifecycleDiff, LucaError, ResolvedComponent,
    Runtime, TextBackend,
};

/// Deterministic driver for UI tests, built on [`Runtime`].
#[derive(Debug)]
pub struct TestHarness {
    runtime: Runtime,
}

impl TestHarness {
    /// Parse, validate, declare state, and mount every id.
    pub fn new(source: &str) -> Result<Self, LucaError> {
        Ok(Self { runtime: Runtime::new(crate::parse(source)?)? })
    }

    /// Route `event` on the component with `id`; returns the handler name.
    /// Logic execution and state feedback stay with the caller (seam +
    /// `set`/`flush`), keeping dispatch declarative.
    pub fn tap(&self, id: &str, event: &str) -> Result<Dispatch, LucaError> {
        dispatch(self.runtime.tree(), id, event)
    }

    /// Stage a state update (commits on [`TestHarness::flush`]).
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), LucaError> {
        self.runtime.state_mut().set(key, value)
    }

    /// Commit staged updates; returns the notification count.
    pub fn flush(&mut self) -> usize {
        self.runtime.state_mut().flush().len()
    }

    /// Swap in a replacement render (re-parse + lifecycle handling).
    pub fn update(&mut self, source: &str) -> Result<LifecycleDiff, LucaError> {
        self.runtime.update_tree(crate::parse(source)?)
    }

    /// Resolve current output from committed state.
    pub fn resolve(&self) -> Result<ResolvedComponent, LucaError> {
        self.runtime.resolve()
    }

    /// Render current output with the text backend.
    pub fn render(&self) -> Result<String, LucaError> {
        TextBackend::render_to_string(&self.resolve()?)
    }

    /// Resolved value of one prop on the component with `id`.
    pub fn prop(&self, id: &str, key: &str) -> Result<Option<Value>, LucaError> {
        let resolved = self.resolve()?;
        let node = resolved.find_by_id(id).ok_or_else(|| {
            LucaError::new(
                format!("No component with id '{id}'"),
                resolved.line,
                resolved.column,
            )
        })?;
        Ok(node.prop(key).cloned())
    }

    /// Resolved `text` of the component with `id`, as displayed.
    pub fn text(&self, id: &str) -> Result<Option<String>, LucaError> {
        Ok(self.prop(id, "text")?.map(|v| match v {
            Value::Str(text) => text,
            other => other.display(),
        }))
    }

    /// Whether the component with `id` is shown (absent `visible` = shown).
    pub fn visible(&self, id: &str) -> Result<bool, LucaError> {
        let resolved = self.resolve()?;
        let node = resolved.find_by_id(id).ok_or_else(|| {
            LucaError::new(
                format!("No component with id '{id}'"),
                resolved.line,
                resolved.column,
            )
        })?;
        Ok(is_visible(node))
    }

    /// Accessible label of the component with `id` (`None` = no label).
    pub fn label(&self, id: &str) -> Result<Option<String>, LucaError> {
        let resolved = self.resolve()?;
        let node = resolved.find_by_id(id).ok_or_else(|| {
            LucaError::new(
                format!("No component with id '{id}'"),
                resolved.line,
                resolved.column,
            )
        })?;
        Ok(describe_resolved(node).label)
    }
}
