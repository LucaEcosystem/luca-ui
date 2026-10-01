//! Development-only visual inspector for 0.9 Alpha (SPEC decision 106–115:
//! a development inspector that may inspect the component tree, state, and
//! layout bounds).
//!
//! `Inspector::snapshot` captures one read-only view of a [`Runtime`]:
//! the component outline (names, ids, locations, depth), committed state
//! values, and layout *declarations*. Layout *bounds* are not reported:
//! bounds need dimension semantics the locked design does not define, so
//! the snapshot carries the declared entries bounds will derive from, and
//! says so explicitly (see `docs/implementation-status.md`). Nothing here
//! mutates the runtime, and the CLI exposes `inspect` in debug builds only.

use crate::{Component, LucaError, Runtime, State};

/// One outlined component: identity and position, no resolved values.
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineNode {
    pub name: String,
    pub id: Option<String>,
    pub line: usize,
    pub column: usize,
    pub depth: usize,
    pub visible_declared: bool,
}

/// One committed state variable for inspection.
#[derive(Debug, Clone, PartialEq)]
pub struct StateView {
    pub name: String,
    pub declared: String,
    pub value: String,
}

/// One component's layout declarations for inspection. Bounds are
/// intentionally absent: dimension semantics are not locked.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutView {
    pub component: String,
    pub id: Option<String>,
    pub entries: Vec<(String, String)>,
}

/// Read-only inspector snapshot of component tree, state, and layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub outline: Vec<OutlineNode>,
    pub state: Vec<StateView>,
    pub layouts: Vec<LayoutView>,
}

/// The development inspector: snapshots, never mutations.
#[derive(Debug, Default)]
pub struct Inspector;

impl Inspector {
    /// Capture tree outline, committed state, and layout declarations.
    pub fn snapshot(runtime: &Runtime) -> Snapshot {
        let mut outline = Vec::new();
        outline_component(runtime.tree(), 0, &mut outline);
        let state = snapshot_state(runtime.state());
        let mut layouts = Vec::new();
        outline_layouts(runtime.tree(), &mut layouts);
        Snapshot { outline, state, layouts }
    }

    /// Render a snapshot as inspectable text (the terminal target's visual).
    pub fn format(snapshot: &Snapshot) -> String {
        let mut lines = vec!["tree:".to_owned()];
        for node in &snapshot.outline {
            let pad = "  ".repeat(node.depth + 1);
            let id = node.id.as_deref().map(|id| format!(" id={id:?}")).unwrap_or_default();
            let hidden = if node.visible_declared { "" } else { " [hidden]" };
            lines.push(format!(
                "{pad}{} ({}:{}){id}{hidden}",
                node.name, node.line, node.column
            ));
        }
        lines.push("state:".to_owned());
        if snapshot.state.is_empty() {
            lines.push("  (none)".to_owned());
        }
        for entry in &snapshot.state {
            lines.push(format!("  .{} ({}) = {}", entry.name, entry.declared, entry.value));
        }
        lines.push("layout:".to_owned());
        if snapshot.layouts.is_empty() {
            lines.push("  (none)".to_owned());
        }
        for layout in &snapshot.layouts {
            let id = layout.id.as_deref().map(|id| format!(" id={id:?}")).unwrap_or_default();
            lines.push(format!("  {}{id}:", layout.component));
            for (key, value) in &layout.entries {
                lines.push(format!("    .{key} = {value}"));
            }
        }
        lines.join("\n")
    }
}

fn outline_component(component: &Component, depth: usize, out: &mut Vec<OutlineNode>) {
    let visible_declared = component.props.iter().all(|p| {
        p.key != "visible"
            || matches!(&p.value, crate::PropValue::Bool(true) | crate::PropValue::Ref(_))
    });
    out.push(OutlineNode {
        name: component.name.clone(),
        id: component.id().map(str::to_owned),
        line: component.line,
        column: component.column,
        depth,
        visible_declared,
    });
    for child in &component.children {
        outline_component(child, depth + 1, out);
    }
}

fn snapshot_state(state: &State) -> Vec<StateView> {
    state
        .names()
        .into_iter()
        .map(|name| {
            let entry = state.entry(&name).expect("name listed by state");
            StateView {
                name,
                declared: entry.declared.to_string(),
                value: entry.value.display(),
            }
        })
        .collect()
}

fn outline_layouts(component: &Component, out: &mut Vec<LayoutView>) {
    if let Some(layout) = &component.layout {
        out.push(LayoutView {
            component: component.name.clone(),
            id: component.id().map(str::to_owned),
            entries: layout
                .entries
                .iter()
                .map(|p| (p.key.clone(), p.value.display_text()))
                .collect(),
        });
    }
    for child in &component.children {
        outline_layouts(child, out);
    }
}

/// Load a `.lucu` file and snapshot it (CLI `inspect` backend).
/// Returned as `LucaError` on parse/state failures for uniform reporting.
pub fn inspect_source(source: &str) -> Result<String, LucaError> {
    let tree = crate::parse(source)?;
    let runtime = Runtime::new(tree)?;
    Ok(Inspector::format(&Inspector::snapshot(&runtime)))
}
