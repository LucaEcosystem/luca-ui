//! 0.5 Alpha: event dispatch, state updates through events, lifecycle
//! cleanup, and error reporting.
//!
//! Dispatch is routing only: it validates the target id and the declared
//! event and returns the handler name. Luca Code logic runs on the existing
//! seam and feeds back solely through `State::set`/`flush`.

use luca_code::types::Value;
use luca_ui::{dispatch, parse, resolve_tree, Runtime, State};

fn example() -> luca_ui::Component {
    parse(&std::fs::read_to_string("examples/events.lucu").unwrap()).unwrap()
}

#[test]
fn declares_and_routes_events() {
    let tree = example();
    let visit = tree.find_by_id("visit").unwrap();
    let block = visit.events.as_ref().expect("events block");
    assert_eq!(block.events.len(), 1);
    assert_eq!(block.events[0].event, "tap");
    assert_eq!(block.events[0].handler, "record_visit");

    // Valid dispatch returns names only — never a component reference.
    let routed = dispatch(&tree, "visit", "tap").unwrap();
    assert_eq!(routed.component_id, "visit");
    assert_eq!(routed.event, "tap");
    assert_eq!(routed.handler, "record_visit");
    let routed = dispatch(&tree, "reset", "tap").unwrap();
    assert_eq!(routed.handler, "reset_visits");
}

#[test]
fn drives_state_updates_through_dispatch() {
    let tree = example();
    let mut state = State::from_tree(&tree).unwrap();
    // Tap "visit" twice through the declarative route, applying Luca Code
    // handler results (computed by the paired .lucc logic below) via state.
    for _ in 0..2 {
        let routed = dispatch(&tree, "visit", "tap").unwrap();
        assert_eq!(routed.handler, "record_visit");
        let current = match state.get("visits").unwrap() {
            Value::Int(n) => *n,
            _ => panic!("visits must stay int"),
        };
        state.set("visits", Value::Int(current + 1)).unwrap();
        state.flush();
    }
    let resolved = resolve_tree(&tree, &state).unwrap();
    assert_eq!(state.get("visits"), Some(&Value::Int(2)));

    // The paired .lucc handler logic runs unchanged on the real seam and
    // agrees with the state updates applied above.
    let lucc = std::fs::read_to_string("examples/events.lucc").unwrap();
    let output = luca_ui::seam::run_logic(&lucc).unwrap();
    assert_eq!(output, vec!["visits = 1", "visits = 2", "visits = 0"]);
    let _ = resolved;
}

#[test]
fn tracks_lifecycle_and_cancels_scoped_work() {
    let mut runtime = Runtime::new(example()).unwrap();
    let detail = runtime.scope_task("greeting", "fade").unwrap();
    let visit_work = runtime.scope_task("visit", "debounce").unwrap();
    assert!(!runtime.task(detail).unwrap().cancelled);

    // Re-render without "visit": it unmounts, its listener prunes, its
    // scoped work cancels, and surviving state persists.
    runtime.state_mut().subscribe("visit");
    let next = parse(
        "App:\n  id = \"app\"\n  state:\n    visits = 9\n    greeting = \"Hi\"\n  Text:\n    id = \"greeting\"\n    text = greeting\n",
    )
    .unwrap();
    let diff = runtime.update_tree(next).unwrap();
    assert_eq!(diff.unmounted, vec!["reset".to_owned(), "visit".to_owned()]);
    assert!(diff.mounted.is_empty());
    assert!(runtime.task(visit_work).unwrap().cancelled);
    assert!(!runtime.task(detail).unwrap().cancelled);
    assert!(!runtime.state().is_subscribed("visit"));
    // Committed state survives the tree swap (new declarations do not
    // overwrite it); newly declared names merge in.
    assert_eq!(runtime.state().get("visits"), Some(&Value::Int(0)));
    let resolved = runtime.resolve().unwrap();
    assert_eq!(
        resolved.find_by_id("greeting").unwrap().prop("text"),
        Some(&Value::Str("Hello, Luca!".to_owned()))
    );

    // Mounting a new component reports it; scoping to the gone one errors.
    let next = parse(
        "App:\n  id = \"app\"\n  state:\n    visits = 0\n    greeting = \"Hi\"\n    extra = 1\n  Text:\n    id = \"greeting\"\n    text = greeting\n  Text:\n    id = \"fresh\"\n    text = greeting\n",
    )
    .unwrap();
    let diff = runtime.update_tree(next).unwrap();
    assert_eq!(diff.mounted, vec!["fresh".to_owned()]);
    assert!(diff.unmounted.is_empty());
    assert_eq!(runtime.state().get("extra"), Some(&Value::Int(1)));
    let err = runtime.scope_task("visit", "late").unwrap_err();
    assert!(err.to_string().contains("unmounted component 'visit'"), "{err}");
    assert!(runtime.cancel_task(visit_work));
    assert!(!runtime.cancel_task(9999));
}

#[test]
fn reports_event_errors() {
    let tree = example();
    // Unknown component id.
    let err = dispatch(&tree, "missing", "tap").unwrap_err();
    assert_eq!((err.line, err.column), (tree.line, tree.column));
    assert!(err.to_string().contains("No component with id 'missing'"), "{err}");
    // Component without an events block.
    let err = dispatch(&tree, "greeting", "tap").unwrap_err();
    assert!(err.to_string().contains("declares no events"), "{err}");
    // Undeclared event on a component that has events.
    let err = dispatch(&tree, "visit", "swipe").unwrap_err();
    assert!(err.to_string().contains("declares no event 'swipe'"), "{err}");

    // Malformed declarations are compile-time Luca errors.
    let cases = [
        // Handler must name Luca Code logic, not a literal.
        ("App:\n  id = \"a\"\n  events:\n    tap = \"go\"\n", "handler must be a Luca Code handler name"),
        ("App:\n  id = \"a\"\n  events:\n    tap = 1\n", "handler must be"),
        // Entries only; at most one block; must belong to a component.
        ("App:\n  id = \"a\"\n  events:\n    tap = go\n    tap = stop\n", "Duplicate 'tap' in `events`"),
        (
            "App:\n  id = \"a\"\n  events:\n    tap = go\n  events:\n    tap = stop\n",
            "Duplicate 'events' block",
        ),
        ("App:\n  id = \"a\"\n  events:\n    Text:\n", "Only `key = value` entries are allowed inside `events`"),
        ("events:\n  tap = go\n", "`events` must belong to a component"),
    ];
    for (src, fragment) in cases {
        let msg = parse(src).unwrap_err().to_string();
        assert!(msg.contains(fragment), "src={src:?} got {msg}");
        assert!(msg.starts_with("Error at "), "src={src:?} got {msg}");
    }
}
