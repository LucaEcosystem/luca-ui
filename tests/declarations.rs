//! 0.2 Alpha: component declarations, identity, property validation, tree.
//!
//! Uses only SPEC-locked rules: `id` on every component (unique, non-empty
//! string, suitable for tests), `visible`/`disabled` as ordinary bool props,
//! no component references or imperative ops, source-located Luca errors.
//! Per-component property inventories are NOT locked, so generic props
//! (e.g. `text`) are accepted; only malformed or locked-rule-violating
//! properties are rejected (see `docs/implementation-status.md`).

use luca_ui::{parse, PropValue};

fn err_msg(src: &str) -> String {
    parse(src).unwrap_err().to_string()
}

#[test]
fn accepts_valid_trees() {
    // Documented 0.2 example: root, siblings, depth 3, ids everywhere.
    let src = std::fs::read_to_string("examples/declarations.lucu").unwrap();
    let tree = parse(&src).unwrap();
    assert_eq!(tree.name, "App");
    assert_eq!(tree.count(), 5);
    assert_eq!(tree.depth(), 3);
    assert_eq!(tree.ids(), vec!["app", "header", "title", "body", "greeting"]);
    assert_eq!(tree.children.len(), 2);
    assert_eq!(tree.children[0].name, "Header");
    assert_eq!(tree.children[1].name, "Body");
    assert_eq!(tree.find_by_id("title").unwrap().name, "Text");
    assert!(tree.find_by_id("missing").is_none());

    // Leaf without props and minimal nesting.
    let tree = parse("App:\n").unwrap();
    assert_eq!((tree.count(), tree.depth()), (1, 1));
    assert!(tree.ids().is_empty());

    // Locked props on root and children; generic props pass through on
    // unregistered components (open vocabulary), while registered core
    // components enforce their prop list.
    let tree = parse(
        "App:\n  id = \"a\"\n  visible = true\n  disabled = false\n  Panel:\n    id = \"b\"\n    count = 3\n    ratio = -2.50\n    missing = none\n    disabled = true\n",
    )
    .unwrap();
    assert_eq!(tree.ids(), vec!["a", "b"]);
    let root_keys: Vec<&str> = tree.props.iter().map(|p| p.key.as_str()).collect();
    assert!(root_keys.contains(&"visible") && root_keys.contains(&"disabled"));

    // Comments, blank lines, and escapes do not disturb the tree.
    let tree = parse(
        "-- leading comment\n[[ multiline\ncomment ]]\nApp: -- trailing\n\n  id = \"a\\\"b\"\n  Text:\n    id = \"t\"\n    text = \"hi\"\n",
    )
    .unwrap();
    assert_eq!(tree.id(), Some("a\"b"));
    assert_eq!(tree.count(), 2);

    // A bare identifier value stays an opaque reference: parsed, never
    // resolved to a component instance (no component-reference mechanism).
    // Unregistered names keep the open vocabulary (checked on Panel since
    // core App enforces its prop list).
    let tree = parse("Panel:\n  text = title\n").unwrap();
    assert_eq!(tree.props[0].value, PropValue::Ref("title".to_owned()));

    // Bindings parse on locked keys too; their types enforce at resolution
    // time (0.4), not at parse time.
    let tree = parse("App:\n  id = app_id\n  visible = is_open\n  label = caption\n").unwrap();
    assert_eq!(tree.props.len(), 3);
}

#[test]
fn rejects_invalid_declarations() {
    let cases = [
        (":\n", "Expected a component name"),
        ("123:\n", "Invalid component name '123'"),
        ("My Component:\n", "Invalid component name"),
        ("my-comp:\n", "Invalid component name"),
        ("App\n", "Expected 'Name:' or 'key = value'"),
        ("", "Expected a component"),
        ("   \n", "Expected a component"),
        ("  App:\n", "Root component must start at column 1"),
        ("id = \"x\"\n", "Property outside a component"),
        ("App:\nApp:\n", "single root"),
        ("App:\n\tvisible = true\n", "Tabs"),
        ("App:\n  A:\nvisible = true\n", "Property outside a component"),
    ];
    for (src, fragment) in cases {
        let msg = err_msg(src);
        assert!(msg.contains(fragment), "src={src:?} got {msg}");
        assert!(msg.starts_with("Error at "), "src={src:?} got {msg}");
    }
    // Call-shaped declarations are imperative operations, not components.
    for src in ["App():\n", "App:\n  Child():\n", "App:\n  show()\n"] {
        let msg = err_msg(src);
        assert!(msg.contains("Imperative"), "src={src:?} got {msg}");
    }
}

#[test]
fn rejects_invalid_properties() {
    // Malformed names and missing values are compile-time Luca errors.
    let cases = [
        ("App:\n  my-prop = 1\n", "Invalid property name 'my-prop'"),
        ("App:\n  1abc = 1\n", "Invalid property name '1abc'"),
        ("App:\n  text =\n", "Expected a value after '='"),
        ("App:\n  text = \"oops\n", "Unterminated string literal"),
        ("App:\n  text = \"a\\qb\"\n", "Unknown escape"),
        ("App:\n  text = 1.2.3\n", "Invalid value '1.2.3'"),
    ];
    for (src, fragment) in cases {
        let msg = err_msg(src);
        assert!(msg.contains(fragment), "src={src:?} got {msg}");
    }
    // Imperative keys and call values stay rejected (declarative UI only).
    for src in [
        "App:\n  hide = true\n",
        "App:\n  show = false\n",
        "App:\n  set_text = \"hi\"\n",
        "App:\n  text = hide()\n",
        "App:\n  on_tap = do_thing(1)\n",
    ] {
        let msg = err_msg(src);
        assert!(msg.contains("Imperative"), "src={src:?} got {msg}");
    }
    // Duplicate properties on one component.
    let msg = err_msg("App:\n  id = \"a\"\n  id = \"b\"\n");
    assert!(msg.contains("Duplicate property 'id'"), "{msg}");
    assert!(msg.contains("first at 2:3"), "{msg}");
}

#[test]
fn validates_identifiers_as_specified() {
    // Duplicate ids: siblings, nested, and root-vs-deep.
    for src in [
        "App:\n  id = \"a\"\n  Text:\n    id = \"a\"\n",
        "App:\n  A:\n    id = \"x\"\n  B:\n    id = \"x\"\n",
        "App:\n  id = \"root\"\n  A:\n    B:\n      id = \"root\"\n",
    ] {
        let msg = err_msg(src);
        assert!(msg.contains("Duplicate id"), "src={src:?} got {msg}");
        assert!(msg.contains("unique for tests"), "src={src:?} got {msg}");
    }
    // Invalid ids: non-string literals and the empty string. Bare
    // identifiers parse as bindings (0.4) and enforce at resolution time.
    for src in [
        "App:\n  id = 3\n",
        "App:\n  id = true\n",
        "App:\n  id = none\n",
        "App:\n  id = \"\"\n",
    ] {
        let msg = err_msg(src);
        assert!(msg.contains("'id'"), "src={src:?} got {msg}");
    }
    // Locked bool props reject every non-bool literal; bindings defer.
    for (key, value) in [
        ("visible", "\"yes\""),
        ("visible", "1"),
        ("visible", "none"),
        ("disabled", "\"no\""),
        ("disabled", "0"),
        ("disabled", "none"),
    ] {
        let msg = err_msg(&format!("App:\n  {key} = {value}\n"));
        assert!(msg.contains(&format!("Property '{key}' must be bool")), "got {msg}");
    }
    // Accessibility labels stay strings when present.
    let msg = err_msg("App:\n  label = 7\n");
    assert!(msg.contains("Property 'label' must be str"), "{msg}");
}

#[test]
fn reports_precise_source_locations() {
    let err = parse("App:\n  visible = \"yes\"\n").unwrap_err();
    assert_eq!((err.line, err.column), (2, 3));
    assert_eq!(err.to_string(), "Error at 2:3: Property 'visible' must be bool, found str");

    let err = parse("App:\n  id = \"a\"\n  Text:\n    id = \"a\"\n").unwrap_err();
    assert_eq!((err.line, err.column), (4, 5));

    let err = parse("123:\n").unwrap_err();
    assert_eq!((err.line, err.column), (1, 1));

    let err = parse("id = \"x\"\n").unwrap_err();
    assert_eq!((err.line, err.column), (1, 1));

    let err = parse("App:\nApp:\n").unwrap_err();
    assert_eq!((err.line, err.column), (2, 1));

    let err = parse("App:\n  my-prop = 1\n").unwrap_err();
    assert_eq!((err.line, err.column), (2, 3));

    // Multiline comments preserve line numbers for later diagnostics.
    let err = parse("[[ c1\nc2 ]]\nApp:\n  disabled = \"x\"\n").unwrap_err();
    assert_eq!((err.line, err.column), (4, 3));
}
