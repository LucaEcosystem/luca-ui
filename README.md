# Luca UI

**Declarative UI for [Luca Code](https://github.com/LucaEcosystem/luca-code)** — describe a screen in `.lucu`, write its logic in `.lucc`.

```text
App:
  id = "app"
  state:
    greeting = "Hello, Luca!"
  Text:
    id = "greeting"
    text = greeting
```

No `hide()` or `set_text()`: components never get commanded. You change state, and the UI derives from it. Events route interaction to Luca Code logic, and every failure is a source-located `Error at line:column: message`.

> **Version 1.0.2 Beta** — of the Luca UI 1.0 Beta line. This is a Beta: the toolchain is real and tested (93 Rust tests, frozen diagnostics catalog, static SVG rendering), but some syntax is a labeled assumption that may change before 1.0 stable. The docs mark anything that may change with an explicit status marker.

---

## Quick start (~5 minutes)

**Requirements:** Rust (stable) and a [Luca Code](https://github.com/LucaEcosystem/luca-code) checkout as a **sibling directory** named `Luca Code` (the `luca-ui` package depends on `luca-code` by relative path — the build reads it directly):

```text
Packages/
  Luca Code/    ← Luca Code repository
  Luca UI/      ← this repository
```

**Install:**

```sh
cargo install --path . --root ~/.luca-ui
export PATH="$HOME/.luca-ui/bin:$PATH"
luca-ui --version        # luca-ui 1.0.2 Beta
```

**Run the hello example:**

```sh
cargo run -- examples/hello.lucu
```

```text
App (4:1) id="app"
  .id = "app"
  .visible = true
  .label = "Demo app"
  Text (8:3) id="greeting"
    .id = "greeting"
    .text = "Hello, Luca!"
    .visible = true
```

**Try state + bindings:**

```sh
cargo run -- examples/state.lucu
```

**Try events (routing to Luca Code logic):**

```sh
cargo run -- examples/events.lucu
cargo run -p luca-code --manifest-path "../Luca Code/Cargo.toml" -- examples/events.lucc
```

```text
visits = 1
visits = 2
visits = 0
```

**Render through the text backend:**

```sh
cargo run --example render
```

**Render a static SVG snapshot:**

```sh
luca-ui render examples/visual.lucu --out visual.svg --width 320 --height 200
# or: cargo run --example visual
```

The snapshot is deterministic and self-contained: `<rect>`/`<text>` primitives in a `<svg>` document. It is **static** — no interaction, animation, fonts, or accessibility tree. The layout/style entries it understands are a documented ◐ assumption subset (`direction`, `gap`, `padding`, `width`, `height`, `align` and `background`, `color`, `border`, `radius`, `font_size`); unknown entries stay opaque.

**Validate the whole static picture (including paired handlers):**

```sh
luca-ui check examples/events.lucu
```

Everything is explained step by step in [Getting started](docs/getting-started.md).

## The VS Code extension

`editors/vscode/` ships the official Luca UI extension: `.lucu` syntax highlighting, snippets, a starter-file command, and one-key run through the CLI. Zero runtime dependencies.

```sh
code --install-extension editors/vscode/luca-ui-1.0.2.vsix
```

Details and settings: [`editors/vscode/README.md`](editors/vscode/README.md).

## What works, what doesn't

| Area | Status |
|---|---|
| `.lucu` parse/validation, component tree, ids | ● Locked, enforced |
| State (`state:` blocks, batched `set`/`flush`, global + per-key subscriptions), bindings | ● Locked model, ◐ assumption syntax |
| Events (`events:` routing to handler names) | ● Locked boundary, ◐ assumption syntax |
| Accessibility labels (explicit wins, `text` infers) | ● Locked |
| Navigation (component-id stack, no routes) | ● Locked |
| Text rendering backend + rendering contract | ● Locked contract, ◐ assumption format |
| Static SVG visual backend (layout, paint, hit testing) | ● Implemented; documented ◐ layout/style subset |
| `check` / `render` CLI, static hit testing | ● Implemented |
| `TestHarness` UI testing, dev inspector | ● Implemented |
| Native / GPU / web backends, interactive routing, platform adapters | ○ Not implemented |
| Full component inventory, layout/style semantics beyond the subset, transition timing, theme tokens, resource files | ○ Open — awaiting the locked design record |

Links for every claim: [docs README](docs/README.md) → the guides.

## Documentation

Start at **[docs/README.md](docs/README.md)**. Newcomers: [Getting started](docs/getting-started.md) → [The `.lucu` language](docs/language.md) → [Components](docs/components.md) → [State and events](docs/state-and-events.md). Day-to-day: [Testing](docs/testing.md), [Rendering](docs/rendering.md), [Styling and theming](docs/styling-theming.md), [Examples](docs/examples.md), [CLI reference](docs/cli.md). Reference: [Architecture](docs/ARCHITECTURE.md), [Diagnostics catalog](docs/diagnostics.md) (frozen message wording), [Compatibility](docs/compatibility.md), [Release notes](docs/RELEASE-NOTES.md), [Changelog](CHANGELOG.md).

## Rules at a glance

The full explanation lives in the [language guide](docs/language.md); the short version:

- One `.lucu` file declares **one root component**; children nest by indentation. `.lucc` files are logic and belong to Luca Code — this toolchain only reads them through the published library.
- Every component may have `id` (unique, non-empty string — the addressing mechanism for tests, events, navigation). `visible`, `disabled`, and `label` are enforced wherever they appear: booleans, boolean, string.
- Output = state + declared properties. Bare-identifier values (`text = title`) are bindings; updates go through `set` (stage) and `flush` (commit, coalesced).
- Events are the interaction boundary: `events:` blocks map an event name to a handler *name*; dispatch validates and returns that name — no component references exist anywhere.
- Imperative component operations (`hide()`, `show()`, `set_text`) do not exist and are compile-time errors.
- Accessibility is core: an explicit `label` wins, a string `text` infers the label otherwise, missing labels are valid.
- Navigation is component-based: a stack of component ids. No routes, no path syntax.
- `layout:`/`style:` blocks are validated structurally; the visual pipeline interprets a documented ◐ subset and leaves every other entry opaque.
- Every failure is `Error at line:column: message` with cataloged, frozen wording; visual assumption problems are non-fatal `Render warning at line:column: message` lines.

## Development

```sh
cargo test                   # the full suite
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- examples/hello.lucu
cargo run -- check examples/events.lucu
cargo run -- render examples/visual.lucu --out visual.svg
cargo run --example render   # text-backend rendering
cargo run --example visual   # static SVG rendering
cargo run -- inspect examples/layout.lucu   # dev inspector (debug builds)
```

The extension's tests: `npm test` inside `editors/vscode/` (10 unit tests, no dependencies).

Packaging: `cargo build --release` produces the `luca-ui` binary; `editors/vscode` packages the `.vsix` with `npx @vscode/vsce package`.

## Contributing

Luca UI is proprietary software. Bug reports, feature requests, and discussions are welcome, but **external code contributions are not accepted** — see [CONTRIBUTING.md](CONTRIBUTING.md). Security issues are reported privately per [SECURITY.md](SECURITY.md).

## License

Proprietary — see [LICENSE](LICENSE). Programs and views you write with Luca UI are yours; the toolchain itself may not be modified or redistributed.
