//! Unicode regression tests for multiline-comment stripping (M4.2).
//!
//! `strip_multiline` previously iterated source bytes (`bytes[i] as char`),
//! corrupting every non-ASCII character and misaligning columns. These
//! tests pin exact text and positions through the real `parse` path.
use luca_ui::{parse, TextBackend};

fn render(source: &str) -> String {
    let tree = parse(source).unwrap();
    let state = luca_ui::State::from_tree(&tree).unwrap();
    let resolved = luca_ui::resolve_tree(&tree, &state).unwrap();
    TextBackend::render_to_string(&resolved).unwrap()
}

#[test]
fn preserves_accented_latin_text() {
    let tree = parse("App:\n  id = \"app\"\n  Text:\n    id = \"t\"\n    text = \"héllo\"\n").unwrap();
    let text = tree.find_by_id("t").unwrap();
    let prop = text.props.iter().find(|p| p.key == "text").unwrap();
    assert_eq!(prop.value.display_text(), "\"héllo\"");
    assert!(render("App:\n  id = \"app\"\n  Text:\n    id = \"t\"\n    text = \"héllo\"\n").contains("héllo"));
}

#[test]
fn preserves_non_latin_scripts_across_lines() {
    let source = "App:\n  id = \"app\"\n  Text:\n    id = \"jp\"\n    text = \"こんにちは\"\n  Text:\n    id = \"emoji\"\n    text = \"✓ ok\"\n";
    let tree = parse(source).unwrap();
    let jp = tree.find_by_id("jp").unwrap();
    assert_eq!(
        jp.props.iter().find(|p| p.key == "text").unwrap().value.display_text(),
        "\"こんにちは\""
    );
    let rendered = render(source);
    assert!(rendered.contains("こんにちは"), "{rendered}");
    assert!(rendered.contains("✓ ok"), "{rendered}");
}

#[test]
fn preserves_unicode_around_multiline_delimiters() {
    let source = "App:\n  [[ ünïcodé comment ]]\n  id = \"app\"\n  Text:\n    id = \"t\"\n    text = \"après [[ ]] fin\"\n";
    let tree = parse(source).unwrap();
    assert!(tree.find_by_id("app").is_some());
    let text = tree.find_by_id("t").unwrap();
    assert_eq!(
        text.props.iter().find(|p| p.key == "text").unwrap().value.display_text(),
        "\"après [[ ]] fin\""
    );
}

#[test]
fn keeps_line_numbers_across_multiline_unicode_comments() {
    let tree = parse("App:\n  [[ ünïcodé\n  still comment ]]\n  id = \"app\"\n  Text:\n    id = \"t\"\n    text = \"x\"\n").unwrap();
    assert_eq!(tree.find_by_id("t").unwrap().line, 5);
}

#[test]
fn reports_comment_error_at_character_column_after_unicode() {
    // `[[` starts at character column 9 (`"héllo" ` is 8 chars); byte
    // counting would report 10 because `é` is two bytes in UTF-8.
    let err = parse("\"héllo\" [[ unterminated\n").unwrap_err();
    assert_eq!((err.line, err.column), (1, 9), "{err}");
}

#[test]
fn reports_later_errors_at_correct_lines_after_unicode() {
    let err = parse("App:\n  id = \"app\"\n  Text:\n    id = \"t\"\n    text = \"héllo\"\n  Button:\n    id = \"b\"\n    bogus = 1\n").unwrap_err();
    assert_eq!((err.line, err.column), (8, 5), "{err}");
}
