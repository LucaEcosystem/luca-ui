# Getting started — Luca UI 1.0.2 Beta

> Luca UI **1.0.2 Beta** — a Beta release of the Luca UI 1.0 line. Everything on this page is implemented and tested; details may change before 1.0 stable.

## What you will learn

By the end of this guide you will have:

1. Installed the `luca-ui` CLI.
2. Run your first `.lucu` file and understood its output.
3. Added state so the UI derives its output from it.
4. Connected an event to Luca Code logic in a `.lucc` file.
5. Set up the VS Code extension.

## How Luca UI fits together

Luca UI is **not** a general-purpose programming language. It is a thin, declarative UI layer on top of [Luca Code](https://github.com/LucaEcosystem/luca-code), which is the general-purpose language of the Luca Ecosystem:

| File | Role | Owner |
|---|---|---|
| `.lucu` | The UI description — components, state, bindings, events | Luca UI |
| `.lucc` | The logic — functions, variables, flow | Luca Code |

The dependency is one-way: Luca UI reads `.lucc` logic through the published Luca Code library. Luca Code knows nothing about Luca UI. You can always run a `.lucc` file by itself with the `luca` CLI.

---

## 1. Requirements

- **Rust (stable)** — install from [rustup.rs](https://rustup.rs) if you do not have it.
- **A Luca Code checkout** placed next to your Luca UI checkout, with the folder name `Luca Code`:

```text
Packages/
  Luca Code/    ← Luca Code repository
  Luca UI/      ← Luca UI repository (this one)
```

Why? The `luca-ui` package depends on `luca-code` by a relative path (`../Luca Code`) because Luca Code is not published to a registry yet. The Rust build reads the sibling checkout directly. If the sibling is missing, the build fails with a path-not-found error about `../Luca Code`.

## 2. Install the CLI

From the Luca UI directory:

```sh
cargo install --path . --root ~/.luca-ui
export PATH="$HOME/.luca-ui/bin:$PATH"
```

Verify:

```sh
luca-ui --version
```

```text
luca-ui 1.0.2 Beta
```

## 3. Run your first UI file

Create a file `hello.lucu`:

```text
-- My first Luca UI file.
App:
  id = "app"
  label = "Demo app"
  Text:
    id = "greeting"
    text = "Hello, Luca!"
```

Run it:

```sh
luca-ui hello.lucu
```

```text
App (2:1) id="app"
  .id = "app"
  .label = "Demo app"
  Text (5:3) id="greeting"
    .id = "greeting"
    .text = "Hello, Luca!"
```

*All outputs on this page are verified with `luca-ui 1.0.2 Beta`.*

Reading the output:

- The file is parsed into a **component tree** — one root (`App`), children nested by indentation.
- Each component reports its **props** as `.key = value` lines.
- `Text` sits *under* `App` because it is indented inside it.
- The `(line:column)` after each component is where that component was declared — that is where diagnostics will point.
- There is no `visible` in the output because we did not declare it. Absent `visible` means the component is shown; nothing here hides anything unless you say `visible = false`.

This prints the **declarative tree** — validation plus structure, not pixels. Rendering is the job of backends (see [Rendering](rendering.md)). This release ships a **text backend** that renders the resolved tree as structural lines and a **static SVG backend** that renders layout, paint, and text as vector primitives.

## 4. Make the UI derive from state

UI output is derived from **state** — there is no `show()`, `hide()`, or `set_text()` API in Luca UI (● locked design). You change state; the UI reacts.

Create `first-state.lucu`:

```text
App:
  id = "app"
  state:
    title = "Luca UI"
    visits = 2
  Text:
    id = "title"
    text = title
  Text:
    id = "counter"
    text = visits
```

Run it:

```sh
luca-ui first-state.lucu
```

```text
App (1:1) id="app"
  .id = "app"
  state (3:3)
    .title = "Luca UI"
    .visits = 2
  Text (6:3) id="title"
    .id = "title"
    .text = title
  Text (9:3) id="counter"
    .text = visits
```

Two new things:

- The `state:` block (under `App`) declares typed variables. Types are inferred from the initial literal: `title` is `str` (a string), `visits` is `int`. ◐ **Assumption** — the `state:` block syntax works exactly like this, but is flagged as an assumption because the full design record may adjust it.
- `text = title` (no quotes) is a **binding**: the property takes its value from the state variable `title`. Quoted `text = "title"` would be the literal text `title`. The CLI prints the declaration as written — the binding stays a bare name.

To see the **resolved** output (bindings substituted from state), render the tree with the text backend. `TestHarness::render()` does this, and it is the same resolution the toolchain performs before any backend runs (see [Rendering](rendering.md)):

```text
App id="app"
  .id = "app"
  Text id="title" label="Luca UI"
    .id = "title"
    .text = "Luca UI"
  Text id="counter"
    .id = "counter"
    .text = 2
```

- `.text = title` became `.text = "Luca UI"`, and `.text = visits` became `.text = 2`.
- The state block is not part of rendered output — rendered output is the component tree after state substitution.
- `label="Luca UI"` appears automatically: when a component has no explicit `label`, its string `text` is offered as the accessible label (● locked accessibility inference — see [Components](components.md)).

### See a static visual

The same resolved output can become an SVG snapshot. Add a `layout:`/`style:` block to the example above (or run the shipped one):

```sh
luca-ui render examples/visual.lucu --out visual.svg --width 320 --height 200
```

`visual.svg` contains `<rect>`/`<text>` primitives built from the documented ◐ layout/style subset. It is deliberately **static**: no interaction, platform fonts, or accessibility tree. To validate the view *and* see warnings about unrecognized visual values, use:

```sh
luca-ui check examples/visual.lucu
```

Details and the exact subset: [Rendering](rendering.md) and [Styling and theming](styling-theming.md).

## 5. Connect an event to logic

Events are the interaction boundary (● locked). A component declares an `events:` block mapping an event name to a **handler name**. The handler names logic that lives in the paired `.lucc` file.

`events-demo.lucu`:

```text
App:
  id = "app"
  state:
    visits = 0
    greeting = "Hello, Luca!"
  Text:
    id = "greeting"
    text = greeting
  Button:
    id = "visit"
    text = "Visit"
    events:
      tap = record_visit
```

Run it:

```sh
luca-ui events-demo.lucu
```

```text
App (1:1) id="app"
  .id = "app"
  state (3:3)
    .visits = 0
    .greeting = "Hello, Luca!"
  Text (6:3) id="greeting"
    .id = "greeting"
    .text = greeting
  Button (9:3) id="visit"
    .id = "visit"
    .text = "Visit"
    events (12:5)
      .tap -> record_visit
```

The `events` section prints the routing: event `tap` goes to handler `record_visit`.

The paired logic, `events-demo.lucc` (Luca Code syntax — full explanation in [State and events](state-and-events.md)):

```text
def func record_visit(n (int)) (int):
    return n + 1

def var visits = 0
visits = record_visit(visits)
print("visits = {visits}")
```

Running the logic file on its own with the `luca` CLI:

```sh
cargo run -p luca-code --manifest-path "../Luca Code/Cargo.toml" -- events-demo.lucc
```

```text
visits = 1
```

How the two connect (● locked):

- `tap = record_visit` declares that when `record_visit` should run for the `tap` event — the `.lucu` side only names the handler; it never calls it.
- Dispatch verifies the component id and the declared event and returns the handler name (see [`dispatch`](state-and-events.md#dispatch-returns-a-name-not-a-call)). Your application code decides when the handler runs.

There is **no automatic `.lucu`→`.lucc` event hookup** in this release (○ not implemented): the host application performs dispatch → run logic → state update → re-render. The [render example in `examples/`](examples.md) and [`tests/events.rs`](../tests/events.rs) in the repository show the full wiring.

When a sibling `.lucc` file exists, `luca-ui check events-demo.lucu` verifies that every declared handler has a top-level `def func`. Imports make that check unverified rather than failing (see [CLI reference](cli.md#luca-ui-check-viewlucu--validate-the-whole-static-picture)).

## 6. Set up the VS Code extension

`editors/vscode/` ships the official extension: `.lucu` syntax highlighting, snippets, a starter-file command, and one-key run through the CLI. Install the local build:

```sh
code --install-extension editors/vscode/luca-ui-1.0.2.vsix
```

Details: [VS Code extension README](../editors/vscode/README.md).

## Troubleshooting

### `Could not read ../Luca Code: …` during install

The `Luca Code` checkout must be a sibling folder with that exact name. See [Requirements](#1-requirements).

### An error like `Error at line:column: message`

Every failure is a source-located Luca error, and every possible message is cataloged. Two quick habits:

1. The numbers point into your file — `Error at 2:3:` is line 2, column 3.
2. [Look the message up in the diagnostics catalog](diagnostics.md); each entry says what causes it.

### `'x' is not declared` around a bare identifier

A bare identifier value (like `text = title`) is a binding. The name must be declared in a `state:` block somewhere in the tree before it can bind. Add it to a `state:` block.

### `Unknown property 'x' on component 'Text' (supports: …)`

Registered core components (`App`, `Text`, `Button`) reject unknown property names — the error lists what each supports. See [Components](components.md).

## Where next

- [The `.lucu` language](language.md) — full grammar reference.
- [Components](components.md) — `App`/`Text`/`Button` and the open component vocabulary.
- [State and events](state-and-events.md) — deep dive into state and the event wiring contract.
- [Examples](examples.md) — every shipped example with verified output.
