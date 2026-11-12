//! Dedicated UI testing API for 1.0.2 Beta (locked decision 106–115: dedicated
//! UI tests and explicit component identifiers).
//!
//! `TestHarness` drives a parsed tree deterministically with no I/O,
//! randomness, or threads: dispatch events by component id, apply state
//! updates through the single `set`/`flush` path, swap trees, and read back
//! resolved props, labels, visibility, and backend output. Tests address
//! components by `id` only — the harness exposes no component references.

use luca_code::types::Value;

use crate::{
    build_visual, describe_resolved, dispatch, is_visible, Dispatch, LifecycleDiff, LucaError,
    Point, Rect, ResolvedComponent, Runtime, Size, SvgBackend, TextBackend, Visual, DEFAULT_HEIGHT,
    DEFAULT_WIDTH,
};

/// Deterministic driver for UI tests, built on [`Runtime`].
#[derive(Debug)]
pub struct TestHarness {
    runtime: Runtime,
}

impl TestHarness {
    /// Parse, validate, declare state, and mount every id.
    pub fn new(source: &str) -> Result<Self, LucaError> {
        Ok(Self {
            runtime: Runtime::new(crate::parse(source)?)?,
        })
    }

    /// Route `event` on the component with `id`; returns the handler name.
    /// Logic execution and state feedback stay with the caller (seam +
    /// `set`/`flush`), keeping dispatch declarative.
    pub fn tap(&self, id: &str, event: &str) -> Result<Dispatch, LucaError> {
        dispatch(self.runtime.tree(), id, event)
    }

    /// Alias for [`TestHarness::tap`] — a named click on an identified
    /// component. No pointer is involved; hit testing is available
    /// separately through [`TestHarness::hit_test`].
    pub fn click(&self, id: &str, event: &str) -> Result<Dispatch, LucaError> {
        self.tap(id, event)
    }

    /// Build a static visual at an explicit document size.
    pub fn visual(&self, width: u32, height: u32) -> Result<Visual, LucaError> {
        build_visual(&self.resolve()?, Size::new(width, height))
    }

    /// Build a static visual at the default document size.
    pub fn visual_default(&self) -> Result<Visual, LucaError> {
        self.visual(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    /// Layout rectangle of the component with `id` at the default size.
    pub fn bounds(&self, id: &str) -> Result<Rect, LucaError> {
        let resolved = self.resolve()?;
        let visual = self.visual_default()?;
        visual
            .layout
            .find_by_id(id)
            .map(|node| node.rect)
            .ok_or_else(|| {
                LucaError::new(
                    format!("No component with id '{id}'"),
                    resolved.line,
                    resolved.column,
                )
            })
    }

    /// The identified component a click at `point` targets in the default
    /// visual (the nearest identified ancestor of the deepest hit node).
    pub fn hit_test(&self, point: Point) -> Result<Option<String>, LucaError> {
        let visual = self.visual_default()?;
        Ok(crate::input::click_target(&visual.layout, point).map(str::to_owned))
    }

    /// Render current output as a static SVG document.
    pub fn compose(&self, width: u32, height: u32) -> Result<String, LucaError> {
        let visual = self.visual(width, height)?;
        let mut backend = SvgBackend::new();
        backend.render_visual(&visual)?;
        Ok(backend.output().to_owned())
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
