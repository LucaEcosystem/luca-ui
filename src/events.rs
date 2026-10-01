//! Declarative events for 0.5 Alpha: the interaction boundary.
//!
//! Locked facts from `docs/SPEC.md`: events are the interaction boundary
//! between components and Luca Code state/logic; components change through
//! state and events, never through component references or imperative
//! operations; invalid declarations are source-located Luca errors. Event
//! declaration and dispatch syntax are NOT locked, so the `events:` block
//! and the rules below are a minimal compatible assumption
//! (see `docs/implementation-status.md`).
//!
//! Model: a component declares `event = handler` entries in an `events:`
//! block, where the handler is a bare identifier naming Luca Code logic
//! owned by the paired `.lucc` side. Dispatch is routing, not execution:
//! `dispatch` validates the target component id and the declared event and
//! returns the handler name. Running that logic stays on the existing seam
//! (`seam::run_logic`), and resulting changes flow back only through
//! `State::set`/`flush` — dispatch never touches components or state, and
//! exposes no component references.

use crate::{layout::check_block_entries, Component, LucaError, Prop, PropValue};

/// One declared `event = handler` routing entry.
#[derive(Debug, Clone, PartialEq)]
pub struct EventDecl {
    pub event: String,
    pub handler: String,
    pub line: usize,
    pub column: usize,
}

/// An `events:` declaration block attached to one component.
#[derive(Debug, Clone, PartialEq)]
pub struct EventBlock {
    pub events: Vec<EventDecl>,
    pub line: usize,
    pub column: usize,
}

impl EventBlock {
    /// Validate raw entries harvested from an `events:` block. Handlers must
    /// be bare identifiers (Luca Code logic names); literals are rejected so
    /// no meaning is invented for them.
    pub(crate) fn from_entries(
        component: &str,
        entries: Vec<Prop>,
        line: usize,
        column: usize,
    ) -> Result<Self, LucaError> {
        check_block_entries("events", component, &entries)?;
        let mut events = Vec::with_capacity(entries.len());
        for entry in entries {
            match entry.value {
                PropValue::Ref(handler) => events.push(EventDecl {
                    event: entry.key,
                    handler,
                    line: entry.line,
                    column: entry.column,
                }),
                _ => {
                    return Err(LucaError::new(
                        format!(
                            "Event '{}' handler must be a Luca Code handler name, found {}",
                            entry.key,
                            entry.value.kind(),
                        ),
                        entry.line,
                        entry.column,
                    ));
                }
            }
        }
        Ok(Self { events, line, column })
    }
}

/// A validated routing decision: run `handler` for `event` on the component
/// with `component_id`, then feed resulting changes back through state.
/// Carries names only — never a component reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Dispatch {
    pub component_id: String,
    pub event: String,
    pub handler: String,
}

/// Route an event to its declared handler.
///
/// Errors (all source-located Luca errors): no component carries the id;
/// the component declares no such event. Dispatch addresses declared
/// literal ids only; binding-resolved ids are a later milestone.
pub fn dispatch(
    tree: &Component,
    component_id: &str,
    event: &str,
) -> Result<Dispatch, LucaError> {
    let target = tree.find_by_id(component_id).ok_or_else(|| {
        LucaError::new(
            format!("No component with id '{component_id}'"),
            tree.line,
            tree.column,
        )
    })?;
    let block = target.events.as_ref().ok_or_else(|| {
        LucaError::new(
            format!("Component '{component_id}' declares no events"),
            target.line,
            target.column,
        )
    })?;
    let decl = block.events.iter().find(|d| d.event == event).ok_or_else(|| {
        LucaError::new(
            format!("Component '{component_id}' declares no event '{event}'"),
            block.line,
            block.column,
        )
    })?;
    Ok(Dispatch {
        component_id: component_id.to_owned(),
        event: event.to_owned(),
        handler: decl.handler.clone(),
    })
}
