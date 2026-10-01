//! 0.9 Alpha: stable diagnostics — every reachable message asserted
//! byte-for-byte against `docs/diagnostics.md`. Any wording change breaks
//! these tests on purpose.

use luca_code::types::Value;
use luca_ui::{parse, resolve_tree, Navigator, State, TestHarness};

fn exact(src: &str) -> String {
    parse(src).unwrap_err().to_string()
}

#[test]
fn parse_and_structure_messages_are_stable() {
    assert_eq!(exact(""), "Error at 1:1: Expected a component, found empty file");
    assert_eq!(
        exact("  App:\n"),
        "Error at 1:3: Root component must start at column 1"
    );
    assert_eq!(exact("id = \"x\"\n"), "Error at 1:1: Property outside a component");
    assert_eq!(exact(":\n"), "Error at 1:1: Expected a component name before ':'");
    assert_eq!(exact("123:\n"), "Error at 1:1: Invalid component name '123'");
    assert_eq!(
        exact("App\n"),
        "Error at 1:1: Expected 'Name:' or 'key = value', found 'App'"
    );
    assert_eq!(
        exact("App:\nApp:\n"),
        "Error at 2:1: Expected a single root component per .lucu file"
    );
    assert_eq!(
        exact("App:\n\tvisible = true\n"),
        "Error at 2:1: Tabs are not allowed for indentation; use spaces"
    );
    assert_eq!(
        exact("[[ oops\n"),
        "Error at 1:1: Unterminated multiline comment"
    );
    assert_eq!(
        exact("App:\n  text = \"oops\n"),
        "Error at 2:9: Unterminated string literal"
    );
    assert_eq!(
        exact("App:\n  my-prop = 1\n"),
        "Error at 2:3: Invalid property name 'my-prop'"
    );
    assert_eq!(
        exact("App:\n  text =\n"),
        "Error at 2:9: Expected a value after '='"
    );
    assert_eq!(
        exact("App:\n  text = \"a\\qb\"\n"),
        "Error at 2:9: Unknown escape '\\q' in string"
    );
    assert_eq!(
        exact("App:\n  text = 1.2.3\n"),
        "Error at 2:9: Invalid value '1.2.3'"
    );
    assert_eq!(
        exact("App:\n  show()\n"),
        "Error at 2:3: Imperative component operations are not supported; update state and allow the UI to react"
    );
    assert_eq!(
        exact("App:\n  hide = true\n"),
        "Error at 2:3: Imperative component operation 'hide' is not supported; update state and allow the UI to react"
    );
    assert_eq!(
        exact("layout:\n  a = 1\n"),
        "Error at 1:1: `layout` must belong to a component"
    );
}

#[test]
fn component_validation_messages_are_stable() {
    assert_eq!(
        exact("App:\n  id = \"a\"\n  id = \"b\"\n"),
        "Error at 3:3: Duplicate property 'id' on component 'App' (first at 2:3)"
    );
    assert_eq!(
        exact("App:\n  id = \"a\"\n  Text:\n    id = \"a\"\n"),
        "Error at 4:5: Duplicate id 'a' (first at 2:3); ids must be unique for tests"
    );
    assert_eq!(
        exact("App:\n  id = 3\n"),
        "Error at 2:3: Property 'id' must be a non-empty string"
    );
    assert_eq!(
        exact("App:\n  visible = \"yes\"\n"),
        "Error at 2:3: Property 'visible' must be bool, found str"
    );
    assert_eq!(
        exact("App:\n  label = 7\n"),
        "Error at 2:3: Property 'label' must be str, found number"
    );
    assert_eq!(
        exact("App:\n  count = 1\n"),
        "Error at 2:3: Unknown property 'count' on component 'App' (supports: disabled, id, label, visible)"
    );
    assert_eq!(
        exact("Text:\n  no = 1\n"),
        "Error at 2:3: Unknown property 'no' on component 'Text' (supports: disabled, id, label, text, visible)"
    );
}

#[test]
fn block_messages_are_stable() {
    assert_eq!(
        exact("App:\n  layout:\n    a = 1\n  layout:\n    b = 2\n"),
        "Error at 4:3: Duplicate 'layout' block on component 'App'"
    );
    assert_eq!(
        exact("App:\n  layout:\n    Text:\n"),
        "Error at 3:5: Only `key = value` entries are allowed inside `layout` (found component 'Text')"
    );
    assert_eq!(
        exact("App:\n  layout:\n    gap = 1\n    gap = 2\n"),
        "Error at 4:5: Duplicate 'gap' in `layout` block of component 'App' (first at 3:5)"
    );
    assert_eq!(
        exact("App:\n  id = \"a\"\n  events:\n    tap = \"go\"\n"),
        "Error at 4:5: Event 'tap' handler must be a Luca Code handler name, found str"
    );
    assert_eq!(
        exact("App:\n  theme:\n    mode = sepia\n"),
        "Error at 3:5: Unknown theme mode 'sepia'; expected system, light, or dark"
    );
    assert_eq!(
        exact("App:\n  theme:\n    mode = 3\n"),
        "Error at 3:5: Unknown theme mode; expected system, light, or dark"
    );
    assert_eq!(
        exact("App:\n  theme:\n    name = 3\n"),
        "Error at 3:5: Theme 'name' must be a string"
    );
}

#[test]
fn state_and_binding_messages_are_stable() {
    let err = State::from_tree(&parse("App:\n  state:\n    a = b\n").unwrap())
        .unwrap_err()
        .to_string();
    assert_eq!(
        err,
        "Error at 3:5: State initial values must be literals (found reference 'b')"
    );
    let err = State::from_tree(
        &parse("App:\n  state:\n    a = 1\n  Panel:\n    state:\n      a = 2\n").unwrap(),
    )
    .unwrap_err()
    .to_string();
    assert_eq!(err, "Error at 6:7: Duplicate state 'a' (first at 3:5)");
    let mut state =
        State::from_tree(&parse("App:\n  state:\n    visits = 2\n").unwrap()).unwrap();
    assert_eq!(
        state.set("visits", Value::Str("x".to_owned())).unwrap_err().to_string(),
        "Error at 3:5: Expected int, found str for state 'visits'"
    );
    assert_eq!(
        state.set("missing", Value::Int(1)).unwrap_err().to_string(),
        "Error at 1:1: 'missing' is not declared"
    );
    let err = State::from_tree(
        &parse("App:\n  state:\n    n = 9999999999999999999999\n").unwrap(),
    )
    .unwrap_err()
    .to_string();
    assert_eq!(
        err,
        "Error at 3:5: Invalid number '9999999999999999999999' for state 'n'"
    );

    let tree = parse("App:\n  Text:\n    text = nowhere\n").unwrap();
    let state = State::from_tree(&tree).unwrap();
    assert_eq!(
        resolve_tree(&tree, &state).unwrap_err().to_string(),
        "Error at 3:5: 'nowhere' is not declared"
    );
    let tree = parse("Panel:\n  count = 9999999999999999999999\n").unwrap();
    let state = State::from_tree(&tree).unwrap();
    assert_eq!(
        resolve_tree(&tree, &state).unwrap_err().to_string(),
        "Error at 2:3: Invalid number '9999999999999999999999'"
    );
    let tree = parse("App:\n  state:\n    x = \"a\"\n    y = \"a\"\n  id = x\n  Text:\n    id = y\n")
        .unwrap();
    let state = State::from_tree(&tree).unwrap();
    assert_eq!(
        resolve_tree(&tree, &state).unwrap_err().to_string(),
        "Error at 7:5: Duplicate id 'a' (first at 5:3); ids must be unique for tests"
    );
    let tree = parse("App:\n  state:\n    n = 3\n  Text:\n    label = n\n").unwrap();
    let state = State::from_tree(&tree).unwrap();
    assert_eq!(
        resolve_tree(&tree, &state).unwrap_err().to_string(),
        "Error at 5:5: Property 'label' must be str, found int"
    );
}

#[test]
fn routing_navigation_and_harness_messages_are_stable() {
    let tree = parse("App:\n  id = \"a\"\n  Button:\n    id = \"b\"\n    events:\n      tap = go\n")
        .unwrap();
    assert_eq!(
        luca_ui::dispatch(&tree, "missing", "tap").unwrap_err().to_string(),
        "Error at 1:1: No component with id 'missing'"
    );
    assert_eq!(
        luca_ui::dispatch(&tree, "a", "tap").unwrap_err().to_string(),
        "Error at 1:1: Component 'a' declares no events"
    );
    assert_eq!(
        luca_ui::dispatch(&tree, "b", "swipe").unwrap_err().to_string(),
        "Error at 5:5: Component 'b' declares no event 'swipe'"
    );
    assert_eq!(
        Navigator::new(&tree, "missing").unwrap_err().to_string(),
        "Error at 1:1: No component with id 'missing'"
    );
    let mut nav = Navigator::new(&tree, "a").unwrap();
    assert_eq!(
        nav.pop().unwrap_err().to_string(),
        "Error at 1:1: Cannot navigate back from the root screen"
    );
    let mut runtime = luca_ui::Runtime::new(tree).unwrap();
    assert_eq!(
        runtime.scope_task("gone", "w").unwrap_err().to_string(),
        "Error at 1:1: Cannot scope work to unmounted component 'gone'"
    );
    let harness = TestHarness::new("App:\n  id = \"a\"\n").unwrap();
    assert_eq!(
        harness.text("missing").unwrap_err().to_string(),
        "Error at 1:1: No component with id 'missing'"
    );
    assert_eq!(
        luca_ui::seam::load_file(std::path::Path::new("x.lucc")).unwrap_err().to_string(),
        "Error at 1:1: Luca UI source files must use the .lucu extension"
    );
}
