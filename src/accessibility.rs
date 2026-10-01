//! Core accessibility model for 0.6 Alpha (`runtime/accessibility` in ARCHITECTURE).
//!
//! Locked facts from `docs/SPEC.md` (decisions 96–100): accessibility is a
//! core capability, not a platform-only add-on; components expose
//! accessibility labels and related semantics; Luca UI infers accessible
//! information when it can and allows explicit overrides; platform adapters
//! map the shared model to native systems. Only labels are locked (plus the
//! inference/override shape); no semantic vocabulary beyond `label` is
//! defined here, and this module carries zero platform concepts — mapping
//! behind adapters is deferred with the adapter milestone.
//!
//! Minimal assumption (see `docs/implementation-status.md`): an explicit
//! `label` always wins; otherwise a `text` value supplies the inferred
//! label when it is a string; otherwise there is no label (valid — not
//! every component has accessible text). `describe` reads the declared
//! tree (literals; bindings are opaque there) while `describe_resolved`
//! reads state-resolved output, so bound labels and texts participate.
//! Non-string labels never reach either function through the toolchain:
//! literals fail parse-time validation and bindings fail resolution, both
//! as source-located Luca errors.

use luca_code::types::Value;

use crate::{Component, PropValue, ResolvedComponent};

/// The accessible label of one component and how it was determined.
#[derive(Debug, Clone, PartialEq)]
pub struct Description {
    /// The usable label, if any component text determines one.
    pub label: Option<String>,
    /// True when inferred from `text`; false when explicitly declared
    /// (or when there is no label at all).
    pub inferred: bool,
}

impl Description {
    /// True when the component exposes any accessible label.
    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }
}

/// Describe a declared component: an explicit string `label` wins, else a
/// string `text` is inferred, else there is no label.
pub fn describe(component: &Component) -> Description {
    if let Some(label) = literal_str(component, "label") {
        return Description { label: Some(label), inferred: false };
    }
    if let Some(text) = literal_str(component, "text") {
        return Description { label: Some(text), inferred: true };
    }
    Description { label: None, inferred: false }
}

/// Describe state-resolved output with the same explicit-wins rule, so
/// bound labels and texts participate once committed state backs them.
pub fn describe_resolved(component: &ResolvedComponent) -> Description {
    if let Some(Value::Str(label)) = component.prop("label") {
        return Description { label: Some(label.clone()), inferred: false };
    }
    if let Some(Value::Str(text)) = component.prop("text") {
        return Description { label: Some(text.clone()), inferred: true };
    }
    Description { label: None, inferred: false }
}

fn literal_str(component: &Component, key: &str) -> Option<String> {
    component.props.iter().find(|p| p.key == key).and_then(|p| match &p.value {
        PropValue::Str(s) => Some(s.clone()),
        _ => None,
    })
}
