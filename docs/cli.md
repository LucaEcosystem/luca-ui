# CLI reference — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. Command-line behavior on this page is implemented and tested.

```sh
Usage: luca-ui <view.lucu> | luca-ui inspect <view.lucu>

Validate a Luca UI file and print its component tree, or (debug builds
only) inspect tree, state, and layout declarations.

Options:
  --help     Show this help and exit
  --version  Show the Luca UI version and exit

Examples:
  luca-ui hello.lucu
  luca-ui inspect hello.lucu
```

## Verify your install

```sh
luca-ui --version
```

```text
luca-ui 1.0.2 Beta
```

## `luca-ui <view.lucu>` — validate and print the tree

```sh
luca-ui hello.lucu
```

Parses, validates, and prints the declarative tree (see [Getting started](getting-started.md#3-run-your-first-ui-file)). The file must end in `.lucu` — for anything else:

```text
Error at 1:1: Luca UI source files must use the .lucu extension
```

Exit codes: `0` on success, `1` on a diagnostic, `2` for usage errors (missing argument; help goes to standard output with exit 0).

```sh
luca-ui examples/lorem.lucu
```

```text
Error at 1:1: Could not read examples/lorem.lucu: No such file or directory (os error 2)
```

## `luca-ui inspect <view.lucu>` — development inspector (debug builds only)

```sh
luca-ui inspect examples/layout.lucu
```

```text
tree:
  App (5:1) id="app"
    Header (13:3) id="header"
      Text (17:5) id="title"
    Body (20:3) id="body"
      Text (24:5) id="greeting"
state:
  (none)
layout:
  App id="app":
    .gap = 2
    .direction = "column"
  Body id="body":
```

The inspect snapshot shows three read-only sections:

- **`tree:`** — every component with its `line:column` declaration position, its `id` (if any), and a `[hidden]` marker when it declares `visible = false`.
- **`state:`** — committed values in declaration order with their declared types (`.name (type) = value`); `(none)` when the tree declares no state.
- **`layout:`** — every component's `layout:` entries, per component.

(Compiled a `--release` binary? `inspect` is refused there on purpose: it is a development tool, not a product surface. Build without `--release`, or run it through `cargo run -- inspect …`.)

## What `inspect` does not show

- **Layout bounds** (computed rectangles/sizes): ○ not implemented — they need dimension semantics the model does not define yet. The snapshot shows the declared entries that bounds would one day derive from, and says nothing more.
- **Resolved output**: `inspect` reads declarations; use the text backend ([Rendering](rendering.md)) or `TestHarness::render()` ([Testing](testing.md)) for resolved views.

## Common exit paths

| Situation | Behavior |
|---|---|
| File missing / unreadable | `Error at 1:1: Could not read <path>: …`, exit 1 |
| Non-`.lucu` extension | `Error at 1:1: Luca UI source files must use the .lucu extension`, exit 1 |
| Any parse/validation failure | `Error at line:column: message`, exit 1 |
| No argument | usage text, exit 2 |

All error wordings are frozen in the [diagnostics catalog](diagnostics.md).
