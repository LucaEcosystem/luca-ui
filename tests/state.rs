//! 0.4 Alpha: state and property bindings.
//!
//! Covers initial values (with Luca Code value semantics), updates,
//! binding propagation (UI output derives from state), batching, type
//! errors, and component-removal behavior for listeners.

use luca_code::types::{Type, Value};
use luca_ui::{parse, resolve_tree, State};

fn state_of(src: &str) -> (luca_ui::Component, State) {
    let tree = parse(src).unwrap();
    let state = State::from_tree(&tree).unwrap();
    (tree, state)
}

#[test]
fn declares_initial_values_with_luca_code_semantics() {
    let src = std::fs::read_to_string("examples/state.lucu").unwrap();
    let (tree, state) = state_of(&src);
    assert_eq!(tree.state.as_ref().unwrap().entries.len(), 4);
    assert_eq!(state.get("title"), Some(&Value::Str("Luca UI".to_owned())));
    assert_eq!(state.get("visits"), Some(&Value::Int(2)));
    // Exact decimal scale is retained through the seam (2.50, not 2.5).
    assert_eq!(state.get("ratio").unwrap().display(), "2.50");
    assert_eq!(state.get("show_detail"), Some(&Value::Bool(true)));
    assert_eq!(state.declared_type("visits"), Some(Type::Int));
    assert_eq!(state.declared_type("title"), Some(Type::Str));
    // `none` infers `uni` and accepts any later value.
    let (_, mut state) = state_of("App:\n  state:\n    slot = none\n");
    assert_eq!(state.declared_type("slot"), Some(Type::Uni));
    state.set("slot", Value::Int(1)).unwrap();
    assert_eq!(state.flush().len(), 0);
    assert_eq!(state.get("slot"), Some(&Value::Int(1)));

    // State blocks merge across components; duplicates are Luca errors.
    let (_, state) = state_of("App:\n  state:\n    a = 1\n  Child:\n    state:\n      b = 2\n");
    assert!(state.get("a").is_some() && state.get("b").is_some());
    let tree = parse("App:\n  state:\n    a = 1\n  Child:\n    state:\n      a = 2\n").unwrap();
    let err = State::from_tree(&tree).unwrap_err();
    assert!(err.to_string().contains("Duplicate state 'a'"), "{err}");
    assert!(err.to_string().contains("first at 3:5"), "{err}");
}

#[test]
fn propagates_bindings_from_state() {
    let (tree, state) = state_of(&std::fs::read_to_string("examples/state.lucu").unwrap());
    let resolved = resolve_tree(&tree, &state).unwrap();
    let title = resolved.find_by_id("title").unwrap();
    assert_eq!(title.prop("text"), Some(&Value::Str("Luca UI".to_owned())));
    assert_eq!(title.prop("visible"), Some(&Value::Bool(true)));
    assert_eq!(
        resolved.find_by_id("counter").unwrap().prop("text"),
        Some(&Value::Int(2))
    );
    // Literals resolve untouched alongside bindings.
    let (tree, state) = state_of("Panel:\n  id = \"a\"\n  visible = true\n  text = \"hi\"\n");
    let resolved = resolve_tree(&tree, &state).unwrap();
    assert_eq!(resolved.prop("text"), Some(&Value::Str("hi".to_owned())));
}

#[test]
fn batches_updates_into_one_round() {
    let (tree, mut state) = state_of(&std::fs::read_to_string("examples/state.lucu").unwrap());
    state.subscribe("title");
    state.subscribe("counter");
    // Staged updates are invisible until flush (batching).
    state.set("title", Value::Str("Hi".to_owned())).unwrap();
    state.set("visits", Value::Int(3)).unwrap();
    state.set("visits", Value::Int(4)).unwrap();
    assert_eq!(state.pending_count(), 2);
    assert_eq!(
        resolve_tree(&tree, &state).unwrap().find_by_id("title").unwrap().prop("text"),
        Some(&Value::Str("Luca UI".to_owned()))
    );
    // One flush commits everything; repeated sets coalesce to the final value.
    let notes = state.flush();
    assert_eq!(state.pending_count(), 0);
    assert_eq!(notes.len(), 4); // 2 listeners x 2 changed keys
    assert_eq!(state.get("visits"), Some(&Value::Int(4)));
    let resolved = resolve_tree(&tree, &state).unwrap();
    assert_eq!(
        resolved.find_by_id("counter").unwrap().prop("text"),
        Some(&Value::Int(4))
    );
    // Empty flush notifies nobody.
    assert!(state.flush().is_empty());
}

#[test]
fn reports_state_type_errors() {
    let (_, mut state) = state_of("App:\n  state:\n    visits = 2\n    flag = true\n");
    // int slot rejects str, at the declaration site.
    let err = state.set("visits", Value::Str("x".to_owned())).unwrap_err();
    assert_eq!((err.line, err.column), (3, 5));
    assert!(err.to_string().contains("Expected int, found str for state 'visits'"), "{err}");
    // bool slot rejects int; int and dec do not mix implicitly.
    assert!(state.set("flag", Value::Int(1)).is_err());
    let (_, mut state) = state_of("App:\n  state:\n    ratio = 2.50\n");
    assert!(state.set("ratio", Value::Int(3)).is_err());
    // none is accepted for any declared type.
    state.set("ratio", Value::None).unwrap();
    state.flush();
    assert_eq!(state.get("ratio"), Some(&Value::None));
    // Unknown names are Luca errors.
    let err = state.set("missing", Value::Int(1)).unwrap_err();
    assert!(err.to_string().contains("'missing' is not declared"), "{err}");
    // Failed sets stage nothing.
    assert_eq!(state.pending_count(), 0);

    // Bindings resolving to the wrong locked type fail at the binding site.
    let (tree, state) = state_of("App:\n  state:\n    name = \"Luca\"\n  Text:\n    visible = name\n");
    let err = resolve_tree(&tree, &state).unwrap_err();
    assert_eq!((err.line, err.column), (5, 5));
    assert!(err.to_string().contains("Property 'visible' must be bool, found str"), "{err}");
    // Unbound references fail at the binding site.
    let (tree, state) = state_of("App:\n  Text:\n    text = nowhere\n");
    let err = resolve_tree(&tree, &state).unwrap_err();
    assert!(err.to_string().contains("'nowhere' is not declared"), "{err}");
    // State initials must be literals, never references.
    let err = parse("App:\n  state:\n    a = b\n").map(|t| State::from_tree(&t)).unwrap().unwrap_err();
    assert!(err.to_string().contains("must be literals"), "{err}");
}

#[test]
fn drops_listeners_when_components_disappear() {
    let v1 = "App:\n  id = \"app\"\n  state:\n    title = \"a\"\n  Text:\n    id = \"title\"\n    text = title\n  Text:\n    id = \"gone\"\n    text = title\n";
    let v2 = "App:\n  id = \"app\"\n  state:\n    title = \"a\"\n  Text:\n    id = \"title\"\n    text = title\n";
    let (_, mut state) = state_of(v1);
    state.subscribe("title");
    state.subscribe("gone");
    assert!(state.is_subscribed("gone"));
    // Pruning against a tree without "gone" cancels its pending work.
    let pruned = parse(v2).unwrap();
    state.prune(&pruned);
    assert!(!state.is_subscribed("gone"));
    assert!(state.is_subscribed("title"));
    state.set("title", Value::Str("b".to_owned())).unwrap();
    let notes = state.flush();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].target, "title");
    assert!(state.unsubscribe("title"));
    assert!(!state.unsubscribe("title"));
}
