//! 0.9 Alpha: every shipped `.lucu` example parses, resolves against its
//! declared state, and renders on the supported (text) target.

use luca_ui::{parse, resolve_tree, State, TextBackend};

/// All shipped examples are self-contained: each carries the state its
/// bindings need (or binds nothing), so parse → state → resolve → render
/// succeeds end to end on the text target.
const EXAMPLES: &[&str] = &[
    "examples/hello.lucu",
    "examples/declarations.lucu",
    "examples/layout.lucu",
    "examples/state.lucu",
    "examples/events.lucu",
    "examples/navigation.lucu",
    "examples/themes.lucu",
    "examples/render.lucu",
];

#[test]
fn every_example_renders_on_the_text_target() {
    for file in EXAMPLES {
        let source = std::fs::read_to_string(file).unwrap();
        let tree = parse(&source).unwrap();
        let state = State::from_tree(&tree).unwrap();
        let resolved = resolve_tree(&tree, &state).unwrap();
        let output = TextBackend::render_to_string(&resolved).unwrap();
        assert!(!output.is_empty(), "{file}");
        // The root component reaches the backend exactly once, first.
        let first = output.lines().next().unwrap();
        assert!(first.starts_with(&tree.name), "{file}: {first}");
    }
}
