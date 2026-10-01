//! 0.9 Alpha: dedicated UI testing API — deterministic, id-addressed,
//! no component references.

use luca_code::types::Value;
use luca_ui::TestHarness;

const COUNTER: &str = "App:\n  id = \"app\"\n  state:\n    n = 0\n    shown = true\n  Text:\n    id = \"count\"\n    text = n\n    visible = shown\n  Button:\n    id = \"more\"\n    text = \"More\"\n    events:\n      tap = bump\n";

#[test]
fn drives_dispatch_state_and_readback() {
    let mut harness = TestHarness::new(COUNTER).unwrap();
    // Dispatch routes declaratively; feedback flows through set/flush.
    let routed = harness.tap("more", "tap").unwrap();
    assert_eq!(routed.handler, "bump");
    harness.set("n", Value::Int(1)).unwrap();
    // Staged edits stay invisible until flush.
    assert_eq!(harness.text("count").unwrap(), Some("0".to_owned()));
    assert_eq!(harness.flush(), 0); // no listeners subscribed: deterministic
    assert_eq!(harness.text("count").unwrap(), Some("1".to_owned()));
    harness.set("n", Value::Int(2)).unwrap();
    harness.flush();
    assert_eq!(harness.text("count").unwrap(), Some("2".to_owned()));
    assert_eq!(harness.prop("count", "visible").unwrap(), Some(Value::Bool(true)));
    assert!(harness.visible("count").unwrap());
    // Non-string text infers no accessible label (inference only where
    // the model defines it).
    assert_eq!(harness.label("count").unwrap(), None);
}

#[test]
fn renders_and_updates_trees_deterministically() {
    let mut harness = TestHarness::new(COUNTER).unwrap();
    let before = harness.render().unwrap();
    assert!(before.contains("Text id=\"count\""), "{before}");
    harness.set("shown", Value::Bool(false)).unwrap();
    harness.flush();
    let after = harness.render().unwrap();
    assert!(!after.contains("Text id=\"count\""), "{after}");
    assert!(!harness.visible("count").unwrap());
    // Tree swaps flow through the same lifecycle as Runtime.
    let diff = harness
        .update("App:\n  id = \"app\"\n  state:\n    n = 0\n  Text:\n    id = \"fresh\"\n    text = n\n")
        .unwrap();
    assert_eq!(diff.unmounted, vec!["count".to_owned(), "more".to_owned()]);
    assert_eq!(diff.mounted, vec!["fresh".to_owned()]);
    assert_eq!(harness.text("fresh").unwrap(), Some("0".to_owned()));
}

#[test]
fn reports_harness_lookup_errors() {
    let harness = TestHarness::new(COUNTER).unwrap();
    let err = harness.text("missing").unwrap_err().to_string();
    assert!(err.contains("No component with id 'missing'"), "{err}");
    assert!(err.starts_with("Error at 1:1"), "{err}");
    let err = harness.tap("count", "tap").unwrap_err().to_string();
    assert!(err.contains("declares no events"), "{err}");
    let err = TestHarness::new("App:\n  count = 1\n").unwrap_err().to_string();
    assert!(err.contains("Unknown property 'count'"), "{err}");
}
