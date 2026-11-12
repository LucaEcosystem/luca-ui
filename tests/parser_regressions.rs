//! Parser regression tests for string boundaries and comment scanning.
//!
//! These pin two defects that shipped in 1.0.2 Beta: trailing tokens after a
//! quoted string were folded into the string, and quote escaping ignored
//! backslash parity, so `\\"` left the scanner inside a string.

use luca_ui::{parse, PropValue};

fn text_of(source: &str) -> String {
    let tree = parse(source).unwrap();
    let panel = tree
        .children
        .iter()
        .find(|child| child.name == "Panel")
        .expect("Panel child");
    match &panel
        .props
        .iter()
        .find(|prop| prop.key == "text")
        .unwrap()
        .value
    {
        PropValue::Str(text) => text.clone(),
        other => panic!("expected string, found {other:?}"),
    }
}

#[test]
fn rejects_trailing_text_after_a_string_literal() {
    let error = parse("App:\n  Panel:\n    text = \"a\" \"b\"\n").unwrap_err();
    assert_eq!(
        error.message,
        "Unexpected text after string literal: '\"b\"'"
    );
    assert_eq!((error.line, error.column), (3, 11));
}

#[test]
fn keeps_escaped_quotes_inside_strings() {
    assert_eq!(
        text_of("App:\n  Panel:\n    text = \"say \\\"hi\\\"\"\n"),
        "say \"hi\""
    );
}

#[test]
fn closing_quote_after_escaped_backslash_ends_the_string() {
    // File content: text = "a\\"
    let source = "App:\n  Panel:\n    text = \"a\\\\\"\n";
    assert_eq!(text_of(source), "a\\");
}

#[test]
fn comment_after_escaped_backslash_and_closing_quote_is_stripped() {
    // File content: text = "a\\" -- comment
    let source = "App:\n  Panel:\n    text = \"a\\\\\" -- comment\n    id = \"panel\"\n";
    assert_eq!(text_of(source), "a\\");
}

#[test]
fn escaped_backslash_also_closes_strings_before_multiline_comments() {
    // File content: text = "a\\" [[ ignored ]] -- trailing
    let source = "App:\n  Panel:\n    text = \"a\\\\\" [[ ignored ]] -- trailing\n";
    assert_eq!(text_of(source), "a\\");
}

#[test]
fn an_escaped_quote_does_not_close_the_string() {
    // File content: text = "a\"
    let error = parse("App:\n  Panel:\n    text = \"a\\\"\n").unwrap_err();
    assert_eq!(error.message, "Unterminated string literal");
}
