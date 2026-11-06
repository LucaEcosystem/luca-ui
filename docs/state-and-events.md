# State and events — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. This page explains how UI output is derived from state, and how user interactions route to Luca Code logic. It is the conceptual heart of Luca UI.

## What you will learn

- What `state:` blocks declare and how types work.
- How bindings make components react to state.
- The batched update model (`set` → `flush`).
- The `events:` block and the four-step wiring contract for running Luca Code logic in response to interaction.
- Lifecycle: tree swaps, listener pruning, and scoped-task cancellation.

All outputs below are verified with `luca-ui 1.0.2 Beta`.

## The central rule

> **UI output derives from state.** There is no `hide()`, no `show()`, no `set_text()` — no component-mutation API at all (● locked). Components read state through bindings; when state changes and the tree is re-resolved, output changes. `set`/`flush` on state is the only path a change takes.

---

## 1. Declaring state with `state:` blocks

A `state:` block lives inside a component and declares variables:

```text
App:
  id = "app"
  state:
    title = "Luca UI"
    visits = 2
    ratio = 2.50
    show_detail = true
    missing = none
```

Rules:

- Literal values only — a state initial value must be written as a literal, never as a binding (`State initial values must be literals (found reference 'b')`). This keeps declarations order-free: it never matters whether a state line appears before or after the names it would reference.
- Every state variable is **scalar** (collections are not accepted: `State 'x' must be a scalar value in 0.4`).
- Types are inferred and enforced:
  - `str` — quoted string
  - `bool` — `true`/`false`
  - `int` — whole number
  - `dec` — number with a dot, exact scale (`2.50` keeps scale `2`)
  - `uni` — inferred when the initial value is `none`; a `uni` variable accepts any later value
- Duplicate declarations fail: `Duplicate state 'visits' (first at 3:5)`.
- Assignments never silently convert: `int` does **not** implicitly become `dec`. Storing `Value::Int` in a `dec` variable fails `Expected dec, found int for state …`. (Luca Code's explicit-conversion `change` is available in your `.lucc` logic.)
- State variables declared anywhere in the tree go into **one shared store**, keyed by variable name.

## 2. Bindings: how components react

A bare identifier in a property is a binding to that state variable:

```text
  Text:
    id = "title"
    text = title
```

- When the tree is **resolved** against committed state, `text` obtains the committed value of `title`.
- The name must be declared somewhere in the tree, or resolution fails at the binding site: `'title' is not declared`.
- Locked property typing applies to the resolved value: if `title` were an `int`, `visible = title` would fail with `Property 'visible' must be bool, found int`.
- Bindings are values, never component references (● locked — no component references exist).

### What "re-resolve" means

Output is not a stream; it is a function. Given the tree and the committed state,

```text
resolve_tree(tree, state)  →  resolved output
```

produces a fresh resolved tree in which every binding is a concrete `Value`. Nothing else in the model mutates output — re-rendering simply means resolving again (and rendering the new resolved tree).

## 3. Updates are batched: `set` and `flush`

The update model is two-step:

```rust
state.set("visits", Value::Int(n + 1))?;  // stage (type-checked now)
state.flush();                            // commit + notify together
state.pending_count() == 0;               // after flush, the batch is empty
```

- `set` **stages** an update. Type-checks happen immediately at the declaration site, so errors are caught at the change site; nothing is committed yet.
- `flush` **commits** every staged update at once and emits notifications: one notification per subscribed component per changed key, holding the final value.
- Repeated sets of one key **coalesce** to the final value: `set(x, 1) → set(x, 2) → flush()` is exactly one committed change (`x = 2`).
- `set` of an undeclared name fails immediately: `'visits' is not declared`.

This batching is locked design (● locked): a long handler logic can make dozens of state changes and produce exactly one update round.

## 4. Events: the interaction boundary

Events as the interaction boundary are locked (●); the `events:` block spelling is a labeled assumption. A component declares, in one `events:` block, that certain events route to **handler names**:

```text
  Button:
    id = "visit"
    text = "Visit"
    events:
      tap = record_visit
```

- The right-hand side must be a bare identifier — a handler *name* (literals are rejected: `Event 'tap' handler must be a Luca Code handler name, found str`).
- One `events:` block per component.
- Events are names, and each component declares its own; two components may both declare `tap` with different handlers.

### `dispatch` returns a name, not a call

Dispatch validates and routes:

```rust
let routed = luca_ui::dispatch(&tree, "visit", "tap")?;
assert_eq!(routed.handler, "record_visit");
```

`Dispatch` carries three names — `component_id`, `event`, `handler` — and **never a component reference**. Errors, all source-located [Luca errors](diagnostics.md):

| Mistake | Message |
|---|---|
| No such component id | `No component with id 'missing'` |
| Component with no `events:` block | `Component 'greeting' declares no events` |
| Event not declared on that component | `Component 'visit' declares no event 'swipe'` |

Dispatch intentionally does **not** run anything. It is routing, not execution.

## 5. Wiring events to logic (the intended contract)

This contract is the working, intended wiring (◐ assumption — awaiting the full design record, like all event execution details). The full flow — demonstrated in `tests/events.rs` and the `examples/events*.{lucu,lucc}` pair (all of them run in CI) — is:

1. **Dispatch** the event on the component id → obtain the handler name.
2. **Run the handler logic** through the Luca Code seam (`luca_ui::seam::run_logic` — see below).
3. **Apply results with `State::set`, then `State::flush`** (batched, coalesced).
4. **Re-resolve and re-render** the tree from the committed state.

No automatic event-to-state system exists, and none is planned for the 1.0 generation — an application host (or future app framework) brings logic and state together. In practice:

- Tests use `TestHarness::tap(id, event)` (see [Testing](testing.md)), which performs step 1 and hands back the handler name.
- Application code performs steps 1–4 itself, exactly like the repository's `tests/events.rs` does end-to-end.

### Why not automatic? (open)

Automatically executing `.lucc` handlers directly from UI events requires an event-execution protocol between the two languages (argument passing, result application, error policy). That protocol awaits the design record; this release deliberately implements the smaller, composable contract above.

## 6. Lifecycle swaps: `Runtime`

`Runtime` owns a tree, its state, the set of mounted component ids, and component-scoped tasks.

```rust
let mut runtime = Runtime::new(tree)?;        // declares state, mounts ids
let diff = runtime.update_tree(new_tree)?;
// diff.mounted / diff.unmounted
```

On every tree swap:

- Newly declared state merges in; existing variables keep their **committed values** — state outlives any single tree.
- Components present before but absent now are **unmounted**; new ids are **mounted** (`LifecycleDiff` lists both, sorted).
- Listeners of unmounted ids are **pruned** so their pending notifications are never emitted.
- Tasks scoped to unmounted components are **cancelled** (● locked: component-scoped async work dies with the component).
- Scoping work to a component that is not mounted is an error: `Cannot scope work to unmounted component 'x'`.

○ Not implemented: an async executor. A `ScopedTask` is a labeled unit with a cancellation flag; actual executors arrive with the platform/adapter milestone.

## 7. What the diagnostics look like

Every failure in this area is a cataloged Luca error, frozen in the [diagnostics catalog](diagnostics.md):

```text
Error at 1:1: 'missing' is not declared
Error at 6:7: Duplicate state 'a' (first at 3:5)
Error at 3:5: Expected int, found str for state 'visits'
```

## Where next

- [Testing](testing.md) — all of this, behind a simple deterministic API.
- [CLI reference](cli.md) — the `inspect` development tool.
- [State and events examples](examples.md) — `state.lucu`, `events.lucu` with verified output.
