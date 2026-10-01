//! Declarative application state for 0.4 Alpha.
//!
//! Locked facts from `docs/SPEC.md`: UI output is derived from application
//! state and declared properties; state updates are batched; components are
//! manipulated through state, never through component references or
//! imperative operations; invalid values are source-located Luca errors.
//! State declaration and binding syntax are NOT locked, so the `state:`
//! block and the rules below are a minimal compatible assumption
//! (see `docs/implementation-status.md`).
//!
//! Value semantics come from Luca Code through the integration seam:
//! state values are `luca_code::types::Value` scalars (exact `Dec`
//! arithmetic, `display`, and `none`-occupies-any-type), and declared types
//! are `luca_code::types::Type`. No interpreter is copied or reimplemented:
//! this module stores, type-checks, batches, and notifies — it evaluates no
//! expressions. Paired `.lucc` logic still runs via `seam::run_logic`.
//!
//! Batching model: `set` type-checks immediately (fail fast) but only stages
//! the update; `flush` commits every staged update at once and emits one
//! notification per listener per changed key holding the final value, so any
//! number of sets propagates as a single update round.
//!
//! Removal model (assumption mapping of "component-scoped work is cancelled
//! when its component disappears" onto this milestone's machinery):
//! listeners are keyed by component `id`; `prune` drops listeners whose id
//! no longer exists in the tree, so their pending notifications are never
//! emitted. There is no async runtime yet; timers/tasks arrive later.

use std::collections::{HashMap, HashSet};

use luca_code::types::{Dec, Type, Value};

use crate::{layout::check_block_entries, Component, LucaError, PropValue};

/// A `state:` declaration block attached to one component.
#[derive(Debug, Clone, PartialEq)]
pub struct StateBlock {
    pub entries: Vec<crate::Prop>,
    pub line: usize,
    pub column: usize,
}

impl StateBlock {
    /// Validate raw entries harvested from a `state:` block.
    pub(crate) fn from_entries(
        component: &str,
        entries: Vec<crate::Prop>,
        line: usize,
        column: usize,
    ) -> Result<Self, LucaError> {
        check_block_entries("state", component, &entries)?;
        Ok(Self { entries, line, column })
    }
}

/// One declared state variable with its Luca Code type and current value.
#[derive(Debug, Clone)]
pub struct StateEntry {
    pub name: String,
    pub declared: Type,
    pub value: Value,
    pub line: usize,
    pub column: usize,
}

/// One committed change delivered to one subscribed component.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub target: String,
    pub key: String,
    pub value: Value,
}

/// Application state: typed variables, a staged-update batch, and
/// component listeners.
#[derive(Debug, Default)]
pub struct State {
    entries: HashMap<String, StateEntry>,
    order: Vec<String>,
    pending: HashMap<String, Value>,
    listeners: HashSet<String>,
}

impl State {
    /// Empty state with nothing declared, staged, or subscribed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect every `state:` block in the tree (depth-first) into a store.
    pub fn from_tree(tree: &Component) -> Result<Self, LucaError> {
        let mut state = Self::new();
        state.collect_from(tree)?;
        Ok(state)
    }

    /// Merge `state:` blocks from a replacement tree: declare names not yet
    /// known, keep existing variables (with committed values) untouched so
    /// state outlives any single tree render.
    pub fn merge_tree(&mut self, tree: &Component) -> Result<(), LucaError> {
        self.merge_from(tree)
    }

    fn collect_from(&mut self, component: &Component) -> Result<(), LucaError> {
        if let Some(block) = &component.state {
            for entry in &block.entries {
                self.declare_entry(entry)?;
            }
        }
        for child in &component.children {
            self.collect_from(child)?;
        }
        Ok(())
    }

    fn merge_from(&mut self, component: &Component) -> Result<(), LucaError> {
        if let Some(block) = &component.state {
            for entry in &block.entries {
                if self.entries.contains_key(&entry.key) {
                    continue;
                }
                self.declare_entry(entry)?;
            }
        }
        for child in &component.children {
            self.merge_from(child)?;
        }
        Ok(())
    }

    fn declare_entry(&mut self, entry: &crate::Prop) -> Result<(), LucaError> {
        // References bind at resolution time; initial values must be
        // literals so collection needs no evaluation order.
        if let PropValue::Ref(name) = &entry.value {
            return Err(LucaError::new(
                format!(
                    "State initial values must be literals (found reference '{name}')"
                ),
                entry.line,
                entry.column,
            ));
        }
        let value = literal_to_value_checked(
            &entry.value,
            &format!("state '{}'", entry.key),
            entry.line,
            entry.column,
        )?;
        self.declare(entry.key.clone(), value, entry.line, entry.column)
    }

    /// Declare one variable; duplicates and non-scalar initials are errors.
    /// `none` infers `uni` (it may occupy any declared type, per Luca Code).
    pub fn declare(
        &mut self,
        name: String,
        initial: Value,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        if let Some(first) = self.entries.get(&name) {
            return Err(LucaError::new(
                format!(
                    "Duplicate state '{name}' (first at {}:{})",
                    first.line, first.column
                ),
                line,
                column,
            ));
        }
        if initial.is_collection() {
            return Err(LucaError::new(
                format!("State '{name}' must be a scalar value in 0.4"),
                line,
                column,
            ));
        }
        let declared = match &initial {
            Value::None => Type::Uni,
            v => v.value_type(),
        };
        self.order.push(name.clone());
        self.entries.insert(name.clone(), StateEntry { name, declared, value: initial, line, column });
        Ok(())
    }

    /// Committed value of a variable, if declared.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.entries.get(name).map(|entry| &entry.value)
    }

    /// Declared variable names in declaration order (for inspection).
    pub fn names(&self) -> Vec<String> {
        self.order.clone()
    }

    /// Full declared entry for inspection, if declared.
    pub fn entry(&self, name: &str) -> Option<&StateEntry> {
        self.entries.get(name)
    }

    /// Declared type of a variable, if declared.
    pub fn declared_type(&self, name: &str) -> Option<Type> {
        self.entries.get(name).map(|entry| entry.declared)
    }

    /// Stage an update. Type-checks now (fail fast, at the declaration site)
    /// but commits only at [`State::flush`].
    ///
    /// `none` is accepted for any declared type, `uni` accepts any value;
    /// otherwise the value type must match exactly. In particular `int` does
    /// not implicitly convert to `dec` (conversion is `change` in Luca Code);
    /// this is the smallest consistent choice, recorded in the status docs.
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), LucaError> {
        let entry = self.entries.get(name).ok_or_else(|| {
            LucaError::new(format!("'{name}' is not declared"), 1, 1)
        })?;
        if value.is_collection() {
            return Err(LucaError::new(
                format!("State '{name}' must be a scalar value in 0.4"),
                entry.line,
                entry.column,
            ));
        }
        if !value.is_none() && entry.declared != Type::Uni && value.value_type() != entry.declared {
            return Err(LucaError::new(
                format!(
                    "Expected {}, found {} for state '{name}'",
                    entry.declared,
                    value.value_type()
                ),
                entry.line,
                entry.column,
            ));
        }
        self.pending.insert(name.to_owned(), value);
        Ok(())
    }

    /// Number of staged (unflushed) updates.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Commit every staged update at once. Returns one notification per
    /// subscribed component per changed key with the final value; repeated
    /// sets of one key coalesce. Clears the batch.
    pub fn flush(&mut self) -> Vec<Notification> {
        let mut notifications = Vec::new();
        if self.pending.is_empty() {
            return notifications;
        }
        let mut keys: Vec<String> = self.pending.keys().cloned().collect();
        keys.sort();
        for key in keys {
            if let Some(value) = self.pending.remove(&key) {
                if let Some(entry) = self.entries.get_mut(&key) {
                    entry.value = value.clone();
                }
                let mut targets: Vec<&String> = self.listeners.iter().collect();
                targets.sort();
                for target in targets {
                    notifications.push(Notification {
                        target: (*target).clone(),
                        key: key.clone(),
                        value: value.clone(),
                    });
                }
            }
        }
        notifications
    }

    /// Subscribe a component id to committed updates.
    pub fn subscribe(&mut self, component_id: &str) {
        self.listeners.insert(component_id.to_owned());
    }

    /// Remove one subscription; returns true when one existed.
    pub fn unsubscribe(&mut self, component_id: &str) -> bool {
        self.listeners.remove(component_id)
    }

    /// True when the component id currently subscribes.
    pub fn is_subscribed(&self, component_id: &str) -> bool {
        self.listeners.contains(component_id)
    }

    /// Drop listeners whose component id no longer exists in the tree, so
    /// their pending notifications are never emitted (cancellation on
    /// disappear, applied to this milestone's batch machinery).
    pub fn prune(&mut self, tree: &Component) {
        let alive: HashSet<&str> = tree.ids().into_iter().collect();
        self.listeners.retain(|id| alive.contains(id.as_str()));
    }
}

/// Convert a literal prop value to Luca Code semantics. Returns `None` for
/// references, which bind at resolution time instead.
pub(crate) fn literal_to_value(value: &PropValue) -> Option<Value> {
    match value {
        PropValue::Str(s) => Some(Value::Str(s.clone())),
        PropValue::Bool(b) => Some(Value::Bool(*b)),
        PropValue::None => Some(Value::None),
        PropValue::Ref(_) => None,
        PropValue::Number(text) => number_to_value(text),
    }
}

fn number_to_value(text: &str) -> Option<Value> {
    if text.contains('.') {
        let (negative, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let mut dec = Dec::parse_literal(digits).ok()?;
        if negative {
            dec = dec.neg()?;
        }
        Some(Value::Dec(dec))
    } else {
        Some(Value::Int(text.parse::<i64>().ok()?))
    }
}

/// Convert a literal prop value, reporting malformed numbers at `line:column`.
pub(crate) fn literal_to_value_checked(
    value: &PropValue,
    what: &str,
    line: usize,
    column: usize,
) -> Result<Value, LucaError> {
    match literal_to_value(value) {
        Some(v) => Ok(v),
        None => match value {
            PropValue::Number(text) => Err(LucaError::new(
                format!("Invalid number '{text}' for {what}"),
                line,
                column,
            )),
            PropValue::Ref(name) => Err(LucaError::new(
                format!("'{name}' is not declared"),
                line,
                column,
            )),
            _ => Err(LucaError::new(format!("Invalid value for {what}"), line, column)),
        },
    }
}
