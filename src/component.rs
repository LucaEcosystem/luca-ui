//! Declarative component tree: declarations, identity, blocks, and events.
//!
//! Locked rules enforced here (from `docs/SPEC.md`): components nest into a
//! tree; every component may carry `id` (unique, non-empty string, suitable
//! for tests); `visible`/`disabled` are ordinary boolean properties; layout,
//! styling, state, and events are part of the declarative layer via
//! platform-independent, backend-agnostic blocks; no component references or
//! imperative component operations (`hide()`/`show()`/`set_text()`);
//! diagnostics are ordinary source-located Luca errors. Block entry names
//! and semantics not locked in SPEC are validated structurally only
//! (see `docs/implementation-status.md`).
//!
//! Per-component inventories for the registered core set are enforced
//! (`crate::library`); unregistered names stay generic with locked rules
//! only (open vocabulary preserved).

use std::collections::HashMap;

use crate::animation::TransitionSet;
use crate::events::EventBlock;
use crate::layout::Layout;
use crate::library;
use crate::state::StateBlock;
use crate::style::Style;
use crate::themes::Theme;
use crate::LucaError;

/// A scalar property value. 0.1 supports only literals plus a bare
/// identifier reference (opaque; binding resolution is deferred).
#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    Str(String),
    Bool(bool),
    Number(String),
    None,
    Ref(String),
}

impl PropValue {
    pub fn kind(&self) -> &'static str {
        match self {
            PropValue::Str(_) => "str",
            PropValue::Bool(_) => "bool",
            PropValue::Number(_) => "number",
            PropValue::None => "none",
            PropValue::Ref(_) => "ref",
        }
    }

    /// Deterministic display text: quoted strings, raw scalars otherwise.
    pub fn display_text(&self) -> String {
        match self {
            PropValue::Str(text) => format!("{text:?}"),
            PropValue::Bool(value) => value.to_string(),
            PropValue::Number(text) => text.clone(),
            PropValue::None => "none".to_owned(),
            PropValue::Ref(name) => name.clone(),
        }
    }
}

/// A single `key = value` property with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub key: String,
    pub value: PropValue,
    pub line: usize,
    pub column: usize,
}

/// A declarative component node.
///
/// `layout`/`style`/`state`/`events` are optional declaration blocks; they
/// are leaf attachments and never affect `count`, `depth`, `ids`, or
/// `find_by_id`.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub props: Vec<Prop>,
    pub children: Vec<Component>,
    pub layout: Option<Layout>,
    pub style: Option<Style>,
    pub state: Option<StateBlock>,
    pub events: Option<EventBlock>,
    pub transitions: Option<TransitionSet>,
    pub theme: Option<Theme>,
}

impl Component {
    pub fn new(name: String, line: usize, column: usize) -> Self {
        Self {
            name,
            line,
            column,
            props: Vec::new(),
            children: Vec::new(),
            layout: None,
            style: None,
            state: None,
            events: None,
            transitions: None,
            theme: None,
        }
    }

    /// Convert reserved `layout:`/`style:`/`state:`/`events:`/`transitions:`/
    /// `theme:` child blocks into typed attachments, recursively. Must run
    /// before [`Component::validate`] so block entries never pollute
    /// component identity or property checks.
    pub fn extract_layout_style(&mut self) -> Result<(), LucaError> {
        let mut kept: Vec<Component> = Vec::new();
        let mut layout: Option<Layout> = None;
        let mut style: Option<Style> = None;
        let mut state: Option<StateBlock> = None;
        let mut events: Option<EventBlock> = None;
        let mut transitions: Option<TransitionSet> = None;
        let mut theme: Option<Theme> = None;
        for mut child in std::mem::take(&mut self.children) {
            match child.name.as_str() {
                "layout" | "style" | "state" | "events" | "transitions" | "theme" => {
                    let block = child.name.clone();
                    let taken = match block.as_str() {
                        "layout" => layout.is_some(),
                        "style" => style.is_some(),
                        "state" => state.is_some(),
                        "transitions" => transitions.is_some(),
                        "theme" => theme.is_some(),
                        _ => events.is_some(),
                    };
                    if taken {
                        return Err(LucaError::new(
                            format!(
                                "Duplicate '{block}' block on component '{}'",
                                self.name
                            ),
                            child.line,
                            child.column,
                        ));
                    }
                    if !child.children.is_empty() {
                        let bad = &child.children[0];
                        return Err(LucaError::new(
                            format!(
                                "Only `key = value` entries are allowed inside `{block}` (found component '{}')",
                                bad.name
                            ),
                            bad.line,
                            bad.column,
                        ));
                    }
                    let entries = std::mem::take(&mut child.props);
                    match block.as_str() {
                        "layout" => {
                            layout = Some(Layout::from_entries(
                                &self.name,
                                entries,
                                child.line,
                                child.column,
                            )?);
                        }
                        "style" => {
                            style = Some(Style::from_entries(
                                &self.name,
                                entries,
                                child.line,
                                child.column,
                            )?);
                        }
                        "state" => {
                            state = Some(StateBlock::from_entries(
                                &self.name,
                                entries,
                                child.line,
                                child.column,
                            )?);
                        }
                        "transitions" => {
                            transitions = Some(TransitionSet::from_entries(
                                &self.name,
                                entries,
                                child.line,
                                child.column,
                            )?);
                        }
                        "theme" => {
                            theme = Some(Theme::from_entries(
                                &self.name,
                                entries,
                                child.line,
                                child.column,
                            )?);
                        }
                        _ => {
                            events = Some(EventBlock::from_entries(
                                &self.name,
                                entries,
                                child.line,
                                child.column,
                            )?);
                        }
                    }
                }
                _ => {
                    child.extract_layout_style()?;
                    kept.push(child);
                }
            }
        }
        self.children = kept;
        self.layout = layout;
        self.style = style;
        self.state = state;
        self.events = events;
        self.transitions = transitions;
        self.theme = theme;
        Ok(())
    }

    /// The `id` property value, if present.
    pub fn id(&self) -> Option<&str> {
        self.props.iter().find(|p| p.key == "id").and_then(|p| match &p.value {
            PropValue::Str(s) => Some(s.as_str()),
            _ => None,
        })
    }

    /// Find a component by `id` (depth-first).
    pub fn find_by_id(&self, id: &str) -> Option<&Component> {
        if self.id() == Some(id) {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find_by_id(id))
    }

    /// Count all components in the tree including self.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(Component::count).sum::<usize>()
    }

    /// Depth of the tree; a single node has depth 1.
    pub fn depth(&self) -> usize {
        1 + self.children.iter().map(Component::depth).max().unwrap_or(0)
    }

    /// All `id` values in depth-first order (for test identifiers).
    pub fn ids(&self) -> Vec<&str> {
        let mut out = Vec::new();
        self.collect_ids(&mut out);
        out
    }

    fn collect_ids<'a>(&'a self, out: &mut Vec<&'a str>) {
        if let Some(id) = self.id() {
            out.push(id);
        }
        for child in &self.children {
            child.collect_ids(out);
        }
    }

    /// Validate locked rules across the whole tree.
    pub fn validate(&self) -> Result<(), LucaError> {
        let mut seen: HashMap<String, (usize, usize)> = HashMap::new();
        self.validate_into(&mut seen)
    }

    fn validate_into(
        &self,
        seen: &mut HashMap<String, (usize, usize)>,
    ) -> Result<(), LucaError> {
        let mut keys: HashMap<&str, (usize, usize)> = HashMap::new();
        for prop in &self.props {
            if let Some((line, column)) = keys.get(prop.key.as_str()) {
                return Err(LucaError::new(
                    format!(
                        "Duplicate property '{}' on component '{}' (first at {}:{})",
                        prop.key, self.name, line, column
                    ),
                    prop.line,
                    prop.column,
                ));
            }
            keys.insert(prop.key.as_str(), (prop.line, prop.column));

            match prop.key.as_str() {
                "hide" | "show" | "set_text" => {
                    return Err(LucaError::new(
                        format!(
                            "Imperative component operation '{}' is not supported; update state and allow the UI to react",
                            prop.key
                        ),
                        prop.line,
                        prop.column,
                    ));
                }
                "id" => match &prop.value {
                    PropValue::Str(s) if !s.is_empty() => {
                        if let Some((line, column)) = seen.get(s) {
                            return Err(LucaError::new(
                                format!(
                                    "Duplicate id '{s}' (first at {line}:{column}); ids must be unique for tests"
                                ),
                                prop.line,
                                prop.column,
                            ));
                        }
                        seen.insert(s.clone(), (prop.line, prop.column));
                    }
                    // A binding defers identity checks to resolution time,
                    // where the resolved value must be a unique string.
                    PropValue::Ref(_) => {}
                    _ => {
                        return Err(LucaError::new(
                            "Property 'id' must be a non-empty string",
                            prop.line,
                            prop.column,
                        ));
                    }
                },
                "visible" | "disabled" => match &prop.value {
                    PropValue::Bool(_) => {}
                    // A binding defers the bool check to resolution time.
                    PropValue::Ref(_) => {}
                    _ => {
                        return Err(LucaError::new(
                            format!(
                                "Property '{}' must be bool, found {}",
                                prop.key,
                                prop.value.kind()
                            ),
                            prop.line,
                            prop.column,
                        ));
                    }
                },
                "label" => match &prop.value {
                    // Accessibility labels are core; 0.1 assumes a string.
                    PropValue::Str(_) => {}
                    // A binding defers the str check to resolution time.
                    PropValue::Ref(_) => {}
                    _ => {
                        return Err(LucaError::new(
                            format!("Property 'label' must be str, found {}", prop.value.kind()),
                            prop.line,
                            prop.column,
                        ));
                    }
                },
                _ => {
                    // Registered core components enforce their prop list;
                    // unregistered names accept generic props (open vocabulary).
                    if let Some(core) = library::lookup(&self.name) {
                        if !core.supports(&prop.key) {
                            return Err(LucaError::new(
                                format!(
                                    "Unknown property '{}' on component '{}' (supports: {})",
                                    prop.key,
                                    self.name,
                                    core.supports_list(),
                                ),
                                prop.line,
                                prop.column,
                            ));
                        }
                    }
                }
            }
        }
        for child in &self.children {
            child.validate_into(seen)?;
        }
        Ok(())
    }
}
