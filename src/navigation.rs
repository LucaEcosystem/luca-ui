//! Component-based navigation for 0.6 Alpha (`compiler/navigation` in ARCHITECTURE).
//!
//! Locked facts from `docs/SPEC.md` (decision 90): navigation is
//! component-based; string-based routes are not the core navigation model.
//! The navigation API shape is NOT locked, so this stack is a minimal
//! compatible assumption (see `docs/implementation-status.md`):
//!
//! - The stack holds component `id`s — the locked cross-component identity
//!   mechanism — never route strings. There is no route table, no path
//!   syntax, and no URL parsing; pushing an id navigates to that declared
//!   component. Targets must exist in the tree, and every operation that
//!   names a component reports a source-located Luca error otherwise.
//! - The stack always has a current screen: it starts with one entry and
//!   popping the last entry is an error. Revisiting an id pushes a new entry
//!   (back returns through each visit in order).
//! - Navigation exposes ids only — never component references — and changes
//!   nothing by itself; pairing a screen change with state/lifecycle effects
//!   is the host's job (e.g. `Runtime::update_tree`).

use crate::{Component, LucaError};

/// A component-id navigation stack over one declared tree.
#[derive(Debug, Clone)]
pub struct Navigator<'a> {
    tree: &'a Component,
    stack: Vec<String>,
}

impl<'a> Navigator<'a> {
    /// Start navigating with `initial_id` as the current screen.
    pub fn new(tree: &'a Component, initial_id: &str) -> Result<Self, LucaError> {
        if tree.find_by_id(initial_id).is_none() {
            return Err(LucaError::new(
                format!("No component with id '{initial_id}'"),
                tree.line,
                tree.column,
            ));
        }
        Ok(Self { tree, stack: vec![initial_id.to_owned()] })
    }

    /// Id of the current screen (always present).
    pub fn current(&self) -> &str {
        self.stack.last().expect("navigation stack is never empty")
    }

    /// The whole stack from root screen to current, oldest first.
    pub fn stack(&self) -> &[String] {
        &self.stack
    }

    /// Navigate to a declared component, pushing it as the current screen.
    pub fn push(&mut self, component_id: &str) -> Result<(), LucaError> {
        if self.tree.find_by_id(component_id).is_none() {
            return Err(LucaError::new(
                format!("No component with id '{component_id}'"),
                self.tree.line,
                self.tree.column,
            ));
        }
        self.stack.push(component_id.to_owned());
        Ok(())
    }

    /// Return to the previous screen. Errors when already at the root
    /// screen, so a current screen always exists.
    pub fn pop(&mut self) -> Result<String, LucaError> {
        if self.stack.len() <= 1 {
            return Err(LucaError::new(
                "Cannot navigate back from the root screen",
                self.tree.line,
                self.tree.column,
            ));
        }
        Ok(self.stack.pop().expect("stack length checked above"))
    }

    /// Replace the current screen without growing the stack.
    pub fn replace(&mut self, component_id: &str) -> Result<(), LucaError> {
        if self.tree.find_by_id(component_id).is_none() {
            return Err(LucaError::new(
                format!("No component with id '{component_id}'"),
                self.tree.line,
                self.tree.column,
            ));
        }
        *self.stack.last_mut().expect("navigation stack is never empty") =
            component_id.to_owned();
        Ok(())
    }
}
