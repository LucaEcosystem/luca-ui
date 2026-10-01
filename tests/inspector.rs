//! 0.9 Alpha: development-only visual inspector — read-only snapshots of
//! tree, state, and layout declarations (bounds deferred: no dimension
//! semantics are locked).

use luca_ui::{Inspector, Runtime};

const VIEW: &str = "App:\n  id = \"app\"\n  state:\n    n = 2\n  layout:\n    gap = 2\n  Text:\n    id = \"t\"\n    text = \"Hi\"\n    visible = false\n";

#[test]
fn snapshots_tree_state_and_layout() {
    let runtime = Runtime::new(luca_ui::parse(VIEW).unwrap()).unwrap();
    let snapshot = Inspector::snapshot(&runtime);
    // Outline: every component with identity, location, depth, visibility.
    assert_eq!(snapshot.outline.len(), 2);
    assert_eq!(snapshot.outline[0].name, "App");
    assert_eq!(snapshot.outline[0].id.as_deref(), Some("app"));
    assert_eq!((snapshot.outline[0].line, snapshot.outline[0].column), (1, 1));
    assert_eq!(snapshot.outline[1].depth, 1);
    assert!(!snapshot.outline[1].visible_declared);
    // Committed state with declared types and Luca Code display values.
    assert_eq!(snapshot.state.len(), 1);
    assert_eq!(snapshot.state[0].name, "n");
    assert_eq!(snapshot.state[0].declared, "int");
    assert_eq!(snapshot.state[0].value, "2");
    // Layout declarations (bounds intentionally absent).
    assert_eq!(snapshot.layouts.len(), 1);
    assert_eq!(snapshot.layouts[0].component, "App");
    assert_eq!(
        snapshot.layouts[0].entries,
        vec![("gap".to_owned(), "2".to_owned())]
    );
}

#[test]
fn formats_and_inspects_sources() {
    let runtime = Runtime::new(luca_ui::parse(VIEW).unwrap()).unwrap();
    let text = Inspector::format(&Inspector::snapshot(&runtime));
    assert_eq!(
        text,
        "tree:\n  App (1:1) id=\"app\"\n    Text (7:3) id=\"t\" [hidden]\nstate:\n  .n (int) = 2\nlayout:\n  App id=\"app\":\n    .gap = 2"
    );
    // Source inspection reports parse failures uniformly.
    let err = luca_ui::inspector::inspect_source("App:\n  count = 1\n").unwrap_err().to_string();
    assert!(err.contains("Unknown property 'count'"), "{err}");
    // Empty state/layout sections render explicitly, never silently.
    let runtime = Runtime::new(luca_ui::parse("App:\n  id = \"a\"\n").unwrap()).unwrap();
    let text = Inspector::format(&Inspector::snapshot(&runtime));
    assert!(text.contains("state:\n  (none)"), "{text}");
    assert!(text.contains("layout:\n  (none)"), "{text}");
}
