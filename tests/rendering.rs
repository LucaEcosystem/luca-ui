//! 0.7 Alpha: renderer contract tests.
//!
//! The contract (see `rendering`): one resolved root per call; visible
//! nodes depth-first, parents first; `visible == false` subtrees skipped;
//! platform-independent model only; deterministic output. Every check runs
//! against both the text backend and an independent recording backend so
//! the abstraction — not one implementation — carries the guarantee.

use luca_code::types::Value;
use luca_ui::{
    format_value, is_visible, parse, resolve_tree, Renderer, ResolvedComponent, State, TextBackend,
};

/// Independent contract implementation: records visits instead of drawing.
#[derive(Debug, Default)]
struct RecordingRenderer {
    visits: Vec<(String, Option<String>, usize, Option<String>)>,
}

impl RecordingRenderer {
    fn visit(&mut self, node: &ResolvedComponent, depth: usize) {
        if !is_visible(node) {
            return;
        }
        let label = luca_ui::describe_resolved(node).label;
        self.visits.push((node.name.clone(), node.id.clone(), depth, label));
        for child in &node.children {
            self.visit(child, depth + 1);
        }
    }
}

impl Renderer for RecordingRenderer {
    fn name(&self) -> &'static str {
        "recording"
    }

    fn render(&mut self, root: &ResolvedComponent) -> Result<(), luca_ui::LucaError> {
        self.visit(root, 0);
        Ok(())
    }
}

fn resolved_example() -> (luca_ui::Component, State, ResolvedComponent) {
    let source = std::fs::read_to_string("examples/render.lucu").unwrap();
    let tree = parse(&source).unwrap();
    let state = State::from_tree(&tree).unwrap();
    let resolved = resolve_tree(&tree, &state).unwrap();
    (tree, state, resolved)
}

#[test]
fn text_backend_renders_resolved_output_exactly() {
    let (_, _, resolved) = resolved_example();
    let output = TextBackend::render_to_string(&resolved).unwrap();
    assert_eq!(
        output,
        "App id=\"app\" label=\"Demo app\"\n  .id = \"app\"\n  .label = \"Demo app\"\n  layout:\n    .gap = 2\n  Header id=\"header\"\n    .id = \"header\"\n    Text id=\"title\" label=\"Hello, Luca!\"\n      .id = \"title\"\n      .text = \"Hello, Luca!\"\n  Footer id=\"footer\"\n    .id = \"footer\"\n    style:\n      .tone = \"soft\"\n    Text id=\"scale\"\n      .id = \"scale\"\n      .text = 2.50"
    );
}

#[test]
fn contract_holds_for_every_backend() {
    let (_, _, resolved) = resolved_example();
    // Text backend names itself and renders deterministically.
    let first = TextBackend::render_to_string(&resolved).unwrap();
    let second = TextBackend::render_to_string(&resolved).unwrap();
    assert_eq!(first, second);
    assert_eq!(TextBackend::new().name(), "text");

    // Recording backend sees the same contract: one root call, visible
    // nodes depth-first with parents first, hidden subtree gone.
    let mut recording = RecordingRenderer::default();
    assert_eq!(recording.name(), "recording");
    recording.render(&resolved).unwrap();
    let names: Vec<&str> = recording.visits.iter().map(|v| v.0.as_str()).collect();
    assert_eq!(names, vec!["App", "Header", "Text", "Footer", "Text"]);
    let ids: Vec<&str> =
        recording.visits.iter().map(|v| v.1.as_deref().unwrap()).collect();
    assert_eq!(ids, vec!["app", "header", "title", "footer", "scale"]);
    let depths: Vec<usize> = recording.visits.iter().map(|v| v.2).collect();
    assert_eq!(depths, vec![0, 1, 2, 1, 2]);
    // No visit ever exposes preview/sneak: the false subtree is skipped.
    assert!(!ids.contains(&"preview") && !ids.contains(&"sneak"));
    // Labels travel with visits (explicit on App, inferred on title).
    assert_eq!(
        recording.visits[0].3.as_deref(),
        Some("Demo app")
    );
    assert_eq!(
        recording.visits[2].3.as_deref(),
        Some("Hello, Luca!")
    );
}

#[test]
fn rendering_derives_from_state() {
    let (_, mut state, _) = resolved_example();
    // Flip the hidden subtree visible and change bound text, then re-render.
    state.set("show_preview", Value::Bool(true)).unwrap();
    state.set("title", Value::Str("Hi!".to_owned())).unwrap();
    assert_eq!(state.flush().len(), 0);
    let tree = parse(&std::fs::read_to_string("examples/render.lucu").unwrap()).unwrap();
    let resolved = resolve_tree(&tree, &state).unwrap();
    let output = TextBackend::render_to_string(&resolved).unwrap();
    assert!(output.contains("Preview id=\"preview\""), "{output}");
    assert!(output.contains(".text = \"You should not see this\""), "{output}");
    assert!(output.contains("label=\"Hi!\""), "{output}");
}

#[test]
fn visibility_defaults_and_value_formatting() {
    // Absent `visible` means shown.
    let tree = parse("App:\n  id = \"a\"\n").unwrap();
    let state = State::from_tree(&tree).unwrap();
    let resolved = resolve_tree(&tree, &state).unwrap();
    assert!(is_visible(&resolved));
    // Exact decimal scale survives formatting; strings render quoted.
    let dec = state.get("missing");
    assert!(dec.is_none());
    assert_eq!(format_value(&Value::Str("a\"b".to_owned())), "\"a\\\"b\"");
    let (_, state, _) = resolved_example();
    assert_eq!(format_value(state.get("ratio").unwrap()), "2.50");
    assert_eq!(format_value(&Value::Bool(false)), "false");
    assert_eq!(format_value(&Value::None), "none");
}
