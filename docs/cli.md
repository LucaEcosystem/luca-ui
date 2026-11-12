# CLI reference — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. Command-line behavior on this page is implemented and tested.

```sh
Usage:
  luca-ui <view.lucu>
  luca-ui check <view.lucu>
  luca-ui render <view.lucu> [--out <file.svg>] [--width <n>] [--height <n>]
  luca-ui inspect <view.lucu>            (debug builds only)

Validate a Luca UI file and print its component tree; check paired Luca Code
handlers and visual diagnostics; or render a static SVG snapshot.

Options:
  --help     Show this help and exit
  --version  Show the Luca UI version and exit

Examples:
  luca-ui hello.lucu
  luca-ui check hello.lucu
  luca-ui render hello.lucu --out hello.svg --width 480 --height 320
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

## `luca-ui check <view.lucu>` — validate the whole static picture

`check` parses, declares state, resolves bindings, builds the visual, prints render warnings, and (when a sibling `.lucc` file exists) checks declared event handlers against Luca Code's own parser.

```sh
luca-ui check examples/visual.lucu
```

```text
OK: examples/visual.lucu
```

Render warnings are printed but do **not** fail the command:

```text
Render warning at 4:5: Unknown layout direction "grid"; expected "column" or "row"
OK: warnings.lucu
```

Handler policy is deliberately honest:

- **No paired `.lucc` file** — prints a note and does not fail; handlers were not checked.
- **Local declarations only** (no imports) — a missing handler is an error, exit `1`, with the handler's source location.
- **Imports present** — a missing local name is reported as *unverified*, not an error, because it may live in an imported module.

```text
Note: handler 'go' on component 'visit' was not found locally; the paired source imports modules, so it is unverified
```

`check` runs no handlers and changes no state.

## `luca-ui render <view.lucu>` — static SVG snapshot

```sh
luca-ui render examples/visual.lucu --out target/visual.svg --width 320 --height 200
```

Options:

| Option | Meaning |
|---|---|
| `--out`, `-o <file.svg>` | Write the document to a file instead of standard output. |
| `--width <n>` | Document width in abstract units (default `320`). |
| `--height <n>` | Document height in abstract units (default `240`). |

The output is a deterministic, self-contained SVG document starting with `<svg` (see [Rendering](rendering.md)). Render warnings go to standard error; the SVG itself is never mixed with warnings. Invalid dimensions and unknown options exit `2`.

This is a **static snapshot**: no interaction, animation, fonts, or accessibility tree.

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

- **Computed layout bounds**: use `luca-ui render`, `build_visual`, or `TestHarness::bounds()` — the inspector stays a declaration snapshot.
- **Resolved output**: `inspect` reads declarations; use the text backend ([Rendering](rendering.md)) or `TestHarness::render()` ([Testing](testing.md)).

## Common exit paths

| Situation | Behavior |
|---|---|
| File missing / unreadable | `Error at 1:1: Could not read <path>: …`, exit 1 |
| Non-`.lucu` extension | `Error at 1:1: Luca UI source files must use the .lucu extension`, exit 1 |
| Any parse/validation failure | `Error at line:column: message`, exit 1 |
| Missing locally-declared handler | `Error at line:column: Event '…' on component '…' names handler '…'…`, exit 1 |
| Render warnings only | warnings on stdout (`check`) or stderr (`render`), exit 0 |
| Invalid usage / dimension | usage text, exit 2 |

All error wordings are frozen in the [diagnostics catalog](diagnostics.md). Render warnings are new and non-fatal.
