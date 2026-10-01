//! 0.6 Alpha: component-based navigation and core accessibility.
//!
//! Navigation holds component ids only — no routes, no route table, no path
//! syntax. Accessibility is core: explicit labels win, `text` infers, and
//! bound values participate through resolved output.

use luca_ui::{
    describe, describe_resolved, parse, resolve_tree, Navigator, State,
};

fn example() -> luca_ui::Component {
    parse(&std::fs::read_to_string("examples/navigation.lucu").unwrap()).unwrap()
}

#[test]
fn navigates_between_declared_components() {
    let tree = example();
    let mut nav = Navigator::new(&tree, "home").unwrap();
    assert_eq!(nav.current(), "home");
    assert_eq!(nav.stack(), &["home".to_owned()]);

    // Push navigates forward; pop returns through each visit in order.
    nav.push("detail").unwrap();
    assert_eq!(nav.current(), "detail");
    nav.push("home").unwrap();
    assert_eq!(nav.stack(), &["home".to_owned(), "detail".to_owned(), "home".to_owned()]);
    assert_eq!(nav.pop().unwrap(), "home");
    assert_eq!(nav.pop().unwrap(), "detail");
    assert_eq!(nav.current(), "home");

    // Replace swaps the current screen without growing the stack.
    nav.push("detail").unwrap();
    nav.replace("home").unwrap();
    assert_eq!(nav.stack(), &["home".to_owned(), "home".to_owned()]);

    // The current screen always resolves to its declared subtree.
    nav = Navigator::new(&tree, "detail").unwrap();
    let screen = tree.find_by_id(nav.current()).unwrap();
    assert_eq!(screen.name, "Detail");
    assert!(screen.find_by_id("body").is_some());
}

#[test]
fn rejects_navigation_errors() {
    let tree = example();
    // Unknown targets are Luca errors at the tree location.
    let err = Navigator::new(&tree, "missing").unwrap_err();
    assert_eq!((err.line, err.column), (tree.line, tree.column));
    assert!(err.to_string().contains("No component with id 'missing'"), "{err}");
    let mut nav = Navigator::new(&tree, "home").unwrap();
    let err = nav.push("missing").unwrap_err();
    assert!(err.to_string().contains("No component with id 'missing'"), "{err}");
    let err = nav.replace("missing").unwrap_err();
    assert!(err.to_string().contains("No component with id 'missing'"), "{err}");
    // The stack always has a current screen: popping the root errors.
    let err = nav.pop().unwrap_err();
    assert!(err.to_string().contains("root screen"), "{err}");
    assert_eq!(nav.current(), "home");
}

#[test]
fn labels_explicit_wins_and_text_infers() {
    let tree = example();
    // Explicit label wins over text.
    let back = tree.find_by_id("back").unwrap();
    let desc = describe(back);
    assert_eq!(desc.label.as_deref(), Some("Go back"));
    assert!(!desc.inferred);
    assert!(desc.has_label());
    // Plain text infers a label.
    let body = tree.find_by_id("body").unwrap();
    let desc = describe(body);
    assert_eq!(desc.label.as_deref(), Some("Details here"));
    assert!(desc.inferred);
    // A screen label describes the screen component itself.
    let home = tree.find_by_id("home").unwrap();
    assert_eq!(describe(home).label.as_deref(), Some("Home screen"));
    // No label and no text is valid: simply no accessible label.
    let plain = parse("App:\n  Box:\n    id = \"b\"\n").unwrap();
    let desc = describe(plain.find_by_id("b").unwrap());
    assert_eq!(desc.label, None);
    assert!(!desc.has_label() && !desc.inferred);
}

#[test]
fn bound_labels_participate_through_resolution() {
    let tree = parse(
        "App:\n  state:\n    caption = \"Live caption\"\n    hint = \"Live hint\"\n  Text:\n    id = \"a\"\n    label = caption\n  Text:\n    id = \"b\"\n    text = hint\n",
    )
    .unwrap();
    let state = State::from_tree(&tree).unwrap();
    let resolved = resolve_tree(&tree, &state).unwrap();
    // Bound explicit label wins and is not inference.
    let desc = describe_resolved(resolved.find_by_id("a").unwrap());
    assert_eq!(desc.label.as_deref(), Some("Live caption"));
    assert!(!desc.inferred);
    // Bound text infers.
    let desc = describe_resolved(resolved.find_by_id("b").unwrap());
    assert_eq!(desc.label.as_deref(), Some("Live hint"));
    assert!(desc.inferred);

    // State updates propagate into accessible output on re-resolution.
    let mut state = state;
    state.set("hint", luca_code::types::Value::Str("Updated".to_owned())).unwrap();
    state.flush();
    let resolved = resolve_tree(&tree, &state).unwrap();
    assert_eq!(
        describe_resolved(resolved.find_by_id("b").unwrap()).label.as_deref(),
        Some("Updated")
    );
}

#[test]
fn reports_missing_and_invalid_accessibility_data() {
    // Non-string literal labels are compile-time Luca errors.
    let msg = parse("App:\n  Text:\n    id = \"a\"\n    label = 7\n").unwrap_err().to_string();
    assert!(msg.contains("Property 'label' must be str"), "{msg}");
    // Bound labels resolving to the wrong type fail at the binding site.
    let tree = parse(
        "App:\n  state:\n    n = 3\n  Text:\n    id = \"a\"\n    label = n\n",
    )
    .unwrap();
    let state = State::from_tree(&tree).unwrap();
    let err = resolve_tree(&tree, &state).unwrap_err();
    assert_eq!((err.line, err.column), (6, 5));
    assert!(err.to_string().contains("Property 'label' must be str, found int"), "{err}");
    // Unbound label references fail at the binding site.
    let tree = parse("App:\n  Text:\n    id = \"a\"\n    label = missing\n").unwrap();
    let state = State::from_tree(&tree).unwrap();
    let err = resolve_tree(&tree, &state).unwrap_err();
    assert!(err.to_string().contains("'missing' is not declared"), "{err}");
}
