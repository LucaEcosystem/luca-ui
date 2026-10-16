//! Genuine closed-loop integration: event dispatch drives real Luca Code
//! handler logic through the seam, and the interpreter's own result flows
//! back through state into the rendered tree. No business logic is
//! duplicated here: the increment lives only in the executed `.lucc`, and
//! the test merely invokes the dispatched handler with live state values
//! and parses the printed protocol line back into state.

use luca_code::types::Value;
use luca_ui::{dispatch, parse, resolve_tree, State, TextBackend};

const APP: &str = "App:\n  id = \"app\"\n  state:\n    visits = 0\n  Text:\n    id = \"counter\"\n    text = visits\n  Button:\n    id = \"visit\"\n    text = \"Visit\"\n    events:\n      tap = record_visit\n";

const HANDLERS: &str = "def func record_visit(n (int)) (int):\n    return n + 1\n";

/// Invoke the dispatched handler with the live counter value, executing
/// the real handler body in the interpreter and returning its result.
fn invoke_handler(handler: &str, current: i64) -> i64 {
    assert_eq!(handler, "record_visit");
    let program = format!("{HANDLERS}print({handler}({current}))\n");
    let output = luca_ui::seam::run_logic(&program).unwrap();
    assert_eq!(output.len(), 1, "handler must print exactly one line");
    output[0].parse().expect("handler must print an integer")
}

fn rendered_counter(tree: &luca_ui::Component, state: &State) -> String {
    let resolved = resolve_tree(tree, state).unwrap();
    assert_eq!(
        resolved.find_by_id("counter").unwrap().prop("text"),
        Some(&Value::Int(state_value(state)))
    );
    TextBackend::render_to_string(&resolved).unwrap()
}

fn state_value(state: &State) -> i64 {
    match state.get("visits").unwrap() {
        Value::Int(n) => *n,
        other => panic!("visits must stay int, got {other:?}"),
    }
}

#[test]
fn tap_drives_real_logic_into_state_and_render() {
    let tree = parse(APP).unwrap();
    let mut state = State::from_tree(&tree).unwrap();
    assert!(rendered_counter(&tree, &state).contains(".text = 0"));

    for expected in [1, 2, 3] {
        // Dispatch routes; the routed handler name selects the logic.
        let routed = dispatch(&tree, "visit", "tap").unwrap();
        // Real interpreter result from live state — never computed here.
        let next = invoke_handler(&routed.handler, state_value(&state));
        state.set("visits", Value::Int(next)).unwrap();
        state.flush();
        assert_eq!(state_value(&state), expected);
        assert!(rendered_counter(&tree, &state).contains(&format!(".text = {expected}")));
    }
}
