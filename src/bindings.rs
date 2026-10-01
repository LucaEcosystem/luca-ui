//! Property bindings for 0.4 Alpha: UI output derived from state.
//!
//! Locked facts from `docs/SPEC.md`: UI output derives from application
//! state and declared properties; components change through state, never
//! through references or imperative operations; invalid values are
//! source-located Luca errors. Binding syntax is NOT locked, so the rule
//! below is a minimal compatible assumption
//! (see `docs/implementation-status.md`):
//!
//! A bare-identifier property value (`Ref`, e.g. `text = title`) is a
//! binding: resolving the tree substitutes the committed state value. All
//! other values are literals. Resolution evaluates nothing and creates no
//! component references; it just reads state.
//!
//! Locked property rules apply to *resolved* values, so a binding that
//! resolves to the wrong type is a Luca error at the binding site
//! (e.g. `visible = name` with `name` a string reports
//! `Property 'visible' must be bool, found str`).

use std::collections::HashMap;

use luca_code::types::Value;

use crate::{state::literal_to_value, Component, LucaError, PropValue, State};

/// One property with its state-resolved Luca Code value.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProp {
    pub key: String,
    pub value: Value,
    pub line: usize,
    pub column: usize,
}

/// A component with every property, layout entry, and style entry resolved
/// against committed state. This is the 0.4 "UI output derives from state"
/// artifact; rendering it is a later milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedComponent {
    pub name: String,
    pub id: Option<String>,
    pub line: usize,
    pub column: usize,
    pub props: Vec<ResolvedProp>,
    pub layout: Vec<ResolvedProp>,
    pub style: Vec<ResolvedProp>,
    pub children: Vec<ResolvedComponent>,
}

impl ResolvedComponent {
    /// Find a resolved component by `id` (depth-first).
    pub fn find_by_id(&self, id: &str) -> Option<&ResolvedComponent> {
        if self.id.as_deref() == Some(id) {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find_by_id(id))
    }

    /// Look up one resolved property value by key.
    pub fn prop(&self, key: &str) -> Option<&Value> {
        self.props.iter().find(|p| p.key == key).map(|p| &p.value)
    }
}

/// Resolve a whole tree against committed state, enforcing locked property
/// rules on the resolved values.
pub fn resolve_tree(tree: &Component, state: &State) -> Result<ResolvedComponent, LucaError> {
    let mut seen: HashMap<String, (usize, usize)> = HashMap::new();
    resolve_into(tree, state, &mut seen)
}

fn resolve_into(
    component: &Component,
    state: &State,
    seen: &mut HashMap<String, (usize, usize)>,
) -> Result<ResolvedComponent, LucaError> {
    let mut props = Vec::with_capacity(component.props.len());
    for prop in &component.props {
        let value = resolve_value(&prop.value, state, prop.line, prop.column)?;
        check_locked_rule(&component.name, &prop.key, &value, prop.line, prop.column)?;
        if prop.key == "id" {
            // Validated Str above; register for tree-wide uniqueness.
            if let Value::Str(id) = &value {
                if let Some((line, column)) = seen.get(id) {
                    return Err(LucaError::new(
                        format!(
                            "Duplicate id '{id}' (first at {line}:{column}); ids must be unique for tests"
                        ),
                        prop.line,
                        prop.column,
                    ));
                }
                seen.insert(id.clone(), (prop.line, prop.column));
            }
        }
        props.push(ResolvedProp {
            key: prop.key.clone(),
            value,
            line: prop.line,
            column: prop.column,
        });
    }
    // Layout/style entries resolve the same way but carry no locked rules
    // (consistent with parse-time validation).
    let mut layout = Vec::new();
    if let Some(block) = &component.layout {
        for entry in &block.entries {
            let value = resolve_value(&entry.value, state, entry.line, entry.column)?;
            layout.push(ResolvedProp {
                key: entry.key.clone(),
                value,
                line: entry.line,
                column: entry.column,
            });
        }
    }
    let mut style = Vec::new();
    if let Some(block) = &component.style {
        for entry in &block.entries {
            let value = resolve_value(&entry.value, state, entry.line, entry.column)?;
            style.push(ResolvedProp {
                key: entry.key.clone(),
                value,
                line: entry.line,
                column: entry.column,
            });
        }
    }
    let id = props
        .iter()
        .find(|p| p.key == "id")
        .and_then(|p| match &p.value {
            Value::Str(s) => Some(s.clone()),
            _ => None,
        });
    let mut children = Vec::with_capacity(component.children.len());
    for child in &component.children {
        children.push(resolve_into(child, state, seen)?);
    }
    Ok(ResolvedComponent {
        name: component.name.clone(),
        id,
        line: component.line,
        column: component.column,
        props,
        layout,
        style,
        children,
    })
}

fn resolve_value(
    value: &PropValue,
    state: &State,
    line: usize,
    column: usize,
) -> Result<Value, LucaError> {
    match value {
        PropValue::Ref(name) => state.get(name).cloned().ok_or_else(|| {
            LucaError::new(format!("'{name}' is not declared"), line, column)
        }),
        literal => match literal_to_value(literal) {
            Some(v) => Ok(v),
            None => match literal {
                PropValue::Number(text) => Err(LucaError::new(
                    format!("Invalid number '{text}'"),
                    line,
                    column,
                )),
                _ => Err(LucaError::new("Invalid value".to_owned(), line, column)),
            },
        },
    }
}

fn check_locked_rule(
    component: &str,
    key: &str,
    value: &Value,
    line: usize,
    column: usize,
) -> Result<(), LucaError> {
    let _ = component;
    match key {
        "hide" | "show" | "set_text" => Err(LucaError::new(
            format!(
                "Imperative component operation '{key}' is not supported; update state and allow the UI to react"
            ),
            line,
            column,
        )),
        "id" => match value {
            Value::Str(s) if !s.is_empty() => Ok(()),
            _ => Err(LucaError::new(
                "Property 'id' must be a non-empty string",
                line,
                column,
            )),
        },
        "visible" | "disabled" => match value {
            Value::Bool(_) => Ok(()),
            _ => Err(LucaError::new(
                format!("Property '{key}' must be bool, found {}", value_kind(value)),
                line,
                column,
            )),
        },
        "label" => match value {
            Value::Str(_) => Ok(()),
            _ => Err(LucaError::new(
                format!("Property 'label' must be str, found {}", value_kind(value)),
                line,
                column,
            )),
        },
        _ => Ok(()),
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "int",
        Value::Dec(_) => "dec",
        Value::Str(_) => "str",
        Value::Bool(_) => "bool",
        Value::None => "none",
        _ => "collection",
    }
}
