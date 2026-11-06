# Testing — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. The `TestHarness` API on this page is implemented and tested. It addresses components **by `id` only** (● locked design: a dedicated UI testing API and explicit component identifiers are part of the core).

## What you will learn

- Driving a `.lucu` tree deterministically from a Rust test with `TestHarness`: no I/O, no randomness, no threads.
- Dispatching events, applying state, reading resolved output, and rendering.

## The idea

A UI test answers questions: *“When the button is tapped, does the counter show the new value? Is that panel still visible after the swap?”*

One driver answers them all, in-memory, on the same resolution and rendering path production uses.

## Setup

```rust
use luca_code::types::Value;
use luca_ui::TestHarness;

const SOURCE: &str = r#"
App:
  id = "app"
  state:
    visits = 0
    greeting = "Hello, Luca!"
    flag = true
  Text:
    id = "greeting"
    text = greeting
    visible = flag
  Text:
    id = "counter"
    text = visits
  Button:
    id = "visit"
    text = "Visit"
    events:
      tap = record_visit
"#;

#[test]
fn visit_button_updates_the_counter() {
    let mut harness = TestHarness::new(SOURCE).unwrap();
    // ... steps below ...
}
```

> Naming note: never name a state variable `hide`, `show`, or `set_text`. Those identifiers are reserved (the toolchain rejects any line where they appear as imperative-looking operations), and a state entry named `show = true` fails with `Imperative component operation 'show' is not supported; update state and allow the UI to react`. It was verified this way, so prefer names like `is_hidden` or `flag`.

The steps below are one verified flow (they compile and run together; every claim was checked against `luca-ui 1.0.2 Beta` before being shortened into these listings).

### 1. Tapping routes the event — it does not run logic

```rust
let routed = harness.tap("visit", "tap").unwrap();
assert_eq!(routed.handler, "record_visit");
```

`tap` validates the component id and the declared event, then returns the handler *name* — the same `Dispatch` as [`luca_ui::dispatch`](state-and-events.md#4-events-the-interaction-boundary). Deciding when that logic runs is your step, exactly like production wiring (see [State and events — wiring](state-and-events.md#5-wiring-events-to-logic-the-intended-contract)).

### 2. State updates: set, then flush

```rust
// Apply the handler's result as state: two sets coalesce into one commit.
harness.set("visits", Value::Int(1)).unwrap();
harness.set("visits", Value::Int(5)).unwrap();
harness.flush();
assert_eq!(harness.text("counter").unwrap().as_deref(), Some("5"));
```

Both sets are staged; `flush` commits the **coalesced final value** — the bound `counter` text reads back as `5`. Type errors surface at `set` time, at the declaration site of the change.

### 3. Reading resolved output

```rust
// The bound Text's resolved text.
assert_eq!(harness.text("greeting").unwrap().as_deref(), Some("Hello, Luca!"));
// `visible` resolves through the binding; absent `visible` means shown.
assert!(harness.visible("greeting").unwrap());
// Accessible label: explicit `label` wins, else string `text` is inferred.
assert_eq!(harness.label("visit").unwrap().as_deref(), Some("Visit"));
// Any resolved prop as a full Luca Code value.
assert_eq!(
    harness.prop("greeting", "text").unwrap(),
    Some(Value::Str("Hello, Luca!".to_owned()))
);
```

All readers take the `id` string and re-resolve on each call, so they always reflect the last `flush` and the current tree. A missing id is a `LucaError`: `No component with id 'summary'` — the same diagnostic everywhere in the toolchain.

### 4. Rendering

```rust
let rendered = harness.render().unwrap();
assert!(rendered.contains(".text = \"Hello, Luca!\""), "{rendered}");
```

`render()` resolves the current tree against committed state and runs the [text backend](rendering.md) over it. If you want to assert on the *shape* of the output, assert on its lines (for example the `.text =` line above) rather than a whole-block string — line-level assertions survive cosmetic changes while staying meaningful.

### 5. Swapping trees, reading lifecycle diffs

```rust
let diff = harness
    .update(r#"
App:
  id = "app"
  state:
    visits = 0
    greeting = "Hello, Luca!"
  Text:
    id = "greeting"
    text = greeting
"#)
    .unwrap();
assert_eq!(diff.unmounted, vec!["counter".to_owned(), "visit".to_owned()]);
// Committed state survives the swap:
assert_eq!(
    harness.prop("greeting", "text").unwrap(),
    Some(Value::Str("Hello, Luca!".to_owned()))
);
```

- `update(source)` re-parses the replacement and performs a lifecycle swap through `Runtime::update_tree`.
- `LifecycleDiff` lists both mounted and unmounted ids, **sorted**.
- Committed state persists across swaps. Newly declared variables merge in; existing ones keep their values (a `visits = 0` in the new tree does *not* reset a committed `visits`).
- Unmounted components lose their listeners and get their scoped tasks cancelled (● locked: component-scoped async work dies with the component).

## The complete harness surface

| Method | What it does |
|---|---|
| `TestHarness::new(source)` | Parse, validate, declare state, mount every id |
| `tap(id, event)` | Dispatch one event; returns the `Dispatch` (handler name) |
| `set(key, value)` | Stage a typed update (type-checked immediately) |
| `flush()` | Commit staged updates; returns the notification count |
| `update(source)` | Swap in a replacement tree (returns `LifecycleDiff`) |
| `resolve()` | Resolved tree from committed state |
| `render()` | Resolve + render with the text backend |
| `prop(id, key)` | Resolved value of one property (or `None`) |
| `text(id)` | Resolved `text` as a string (any value via `display()`) |
| `visible(id)` | Whether the component is shown (absent `visible` = shown) |
| `label(id)` | Accessible label, `Some("…")` or `None` |

## Testing rules of thumb

- **Test behavior, not compilation.** “Invalid input is rejected” tests are a handful of `unwrap_err()` assertions on exact messages, like `tests/foundation.rs` and `tests/diagnostics.rs` in this repository do.
- **One behavior per test.** The harness has no per-test isolation cost (no threads, no timers), so granular tests are cheap.
- **Keep sources inline.** `r#"…"#` literals keep failure output readable end to end. Loading example files works too, but inline sources make CI failures self-contained.
- **Never mutate components.** If a test wants to “hide” something, set the state the `visible` binding reads — that is the only hide/show mechanism there is (● locked).

## Where next

- [State and events](state-and-events.md) — what `tap`/`set`/`flush` really touch.
- [Rendering](rendering.md) — the backend contract `render()` uses.
- [Diagnostics catalog](diagnostics.md) — assert exact error messages.

*The code on this page was verified as a scratch integration test against `luca-ui 1.0.2 Beta`, then shortened into the listings above.*
