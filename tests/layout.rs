//! 0.3 Alpha: platform-independent layout/style representation and validation.
//!
//! Uses only SPEC-locked facts: layout/styling belong to the declarative
//! layer; the UI model is platform-independent with no backend concepts;
//! invalid values are source-located Luca errors. Entry names, dimensions,
//! defaults, and precedence are NOT locked, so entries are validated
//! structurally only (see `docs/implementation-status.md`).

use luca_ui::{parse, PropValue};

fn err_msg(src: &str) -> String {
    parse(src).unwrap_err().to_string()
}

#[test]
fn accepts_valid_layouts_and_styles() {
    // Representative example: blocks on root and children, plus an empty block.
    let src = std::fs::read_to_string("examples/layout.lucu").unwrap();
    let tree = parse(&src).unwrap();
    assert_eq!(tree.name, "App");
    let layout = tree.layout.as_ref().expect("root layout");
    assert_eq!(layout.entries.len(), 2);
    assert_eq!(layout.entries[0].key, "gap");
    assert_eq!(layout.entries[0].value, PropValue::Number("2".to_owned()));
    let style = tree.style.as_ref().expect("root style");
    assert_eq!(style.entries.len(), 1);
    let header = tree.find_by_id("header").unwrap();
    assert!(header.layout.is_none());
    assert_eq!(header.style.as_ref().unwrap().entries.len(), 1);
    // Empty blocks are allowed and report empty.
    let body = tree.find_by_id("body").unwrap();
    assert!(body.layout.as_ref().unwrap().is_empty());

    // Blocks are leaf attachments: they never affect tree shape or identity.
    assert_eq!((tree.count(), tree.depth()), (5, 3));
    assert_eq!(tree.ids(), vec!["app", "header", "title", "body", "greeting"]);

    // All scalar value kinds, including opaque references, are accepted.
    let tree = parse(
        "App:\n  layout:\n    a = \"x\"\n    b = true\n    c = 1.50\n    d = none\n    e = other\n  style:\n    f = -3\n",
    )
    .unwrap();
    assert_eq!(tree.layout.as_ref().unwrap().entries.len(), 5);
    assert_eq!(tree.style.as_ref().unwrap().entries.len(), 1);

    // A component without blocks still parses with none attached.
    let tree = parse("App:\n  id = \"a\"\n").unwrap();
    assert!(tree.layout.is_none() && tree.style.is_none());
}

#[test]
fn rejects_invalid_layout_and_style_values() {
    let cases = [
        // Malformed entry values surface as ordinary Luca errors.
        ("App:\n  layout:\n    gap = \"oops\n", "Unterminated string literal"),
        ("App:\n  style:\n    tone = \"a\\qb\"\n", "Unknown escape"),
        ("App:\n  layout:\n    gap =\n", "Expected a value after '='"),
        ("App:\n  style:\n    bad-prop = 1\n", "Invalid property name 'bad-prop'"),
        // Call-shaped entries stay imperative and declarative-only.
        ("App:\n  layout:\n    gap = fit(1)\n", "Imperative"),
        ("App:\n  style:\n    hide = true\n", "Imperative"),
        // Duplicate entries within one block.
        ("App:\n  layout:\n    gap = 1\n    gap = 2\n", "Duplicate 'gap' in `layout`"),
        ("App:\n  style:\n    tone = \"a\"\n    tone = \"b\"\n", "Duplicate 'tone' in `style`"),
        // Same key across layout vs style vs props is fine (separate namespaces).
    ];
    for (src, fragment) in cases {
        let msg = err_msg(src);
        assert!(msg.contains(fragment), "src={src:?} got {msg}");
        assert!(msg.starts_with("Error at "), "src={src:?} got {msg}");
    }
    let tree = parse("Panel:\n  gap = 1\n  layout:\n    gap = 2\n  style:\n    gap = 3\n").unwrap();
    assert_eq!(tree.layout.as_ref().unwrap().entries.len(), 1);
}

#[test]
fn rejects_misplaced_and_duplicate_blocks() {
    let cases = [
        // At most one block of each kind per component.
        (
            "App:\n  layout:\n    a = 1\n  layout:\n    b = 2\n",
            "Duplicate 'layout' block on component 'App'",
        ),
        (
            "App:\n  style:\n    a = 1\n  style:\n    b = 2\n",
            "Duplicate 'style' block on component 'App'",
        ),
        // Blocks hold entries only, never nested components.
        (
            "App:\n  layout:\n    Text:\n      id = \"t\"\n",
            "Only `key = value` entries are allowed inside `layout`",
        ),
        (
            "App:\n  style:\n    layout:\n      a = 1\n",
            "Only `key = value` entries are allowed inside `style`",
        ),
        // Blocks must belong to a component, never stand at the root.
        ("layout:\n  a = 1\n", "`layout` must belong to a component"),
        ("style:\n  a = 1\n", "`style` must belong to a component"),
    ];
    for (src, fragment) in cases {
        let msg = err_msg(src);
        assert!(msg.contains(fragment), "src={src:?} got {msg}");
    }
}

#[test]
fn blocks_do_not_pollute_identity_or_props() {
    // Block entries named `id`/`visible` are generic entries, not identity.
    let tree = parse("App:\n  id = \"a\"\n  layout:\n    id = \"a\"\n    visible = true\n").unwrap();
    assert_eq!(tree.id(), Some("a"));
    assert_eq!(tree.ids(), vec!["a"]);
    // Component-level duplicate detection still applies to real props.
    let msg = err_msg("App:\n  id = \"a\"\n  Text:\n    id = \"a\"\n  layout:\n    gap = 1\n");
    assert!(msg.contains("Duplicate id"), "{msg}");
}

#[test]
fn reports_precise_block_locations() {
    let err = parse("App:\n  layout:\n    gap = 1\n    gap = 2\n").unwrap_err();
    assert_eq!((err.line, err.column), (4, 5));
    assert!(err.to_string().contains("first at 3:5"));

    let err = parse("App:\n  layout:\n    a = 1\n  layout:\n    b = 2\n").unwrap_err();
    assert_eq!((err.line, err.column), (4, 3));

    let err = parse("App:\n  layout:\n    Text:\n").unwrap_err();
    assert_eq!((err.line, err.column), (3, 5));

    let err = parse("layout:\n  a = 1\n").unwrap_err();
    assert_eq!((err.line, err.column), (1, 1));

    let err = parse("App:\n  style:\n    tone = \"x\n").unwrap_err();
    assert_eq!((err.line, err.column), (3, 11));
}
