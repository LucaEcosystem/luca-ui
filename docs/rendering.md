# Rendering — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. Rendering in this release means the headless **text backend**: a deterministic, dependency-free renderer used for validation, debugging, and tests. ● The rendering *contract* is locked; ◐ the text backend's exact line format is a labeled assumption.

## What you will learn

- What a backend receives and must guarantee.
- What the text backend prints, and how to read it.
- How to render from code.

## The rendering contract

Every backend — today's text backend, and any future native/GPU/web backend — receives the **resolved tree** and must honor five rules:

1. It receives the resolved root exactly once per `render` call — *UI output already derived from state*, not a tree with bindings unresolved.
2. It visits visible nodes depth-first, parents before children.
3. It skips `visible = false` subtrees entirely — no partial rendering of hidden branches.
4. It reads only the platform-independent model (resolved components, labels, values). It introduces no platform components and names no platform, renderer, or unit concepts.
5. It is deterministic — same input, same output, every run.

Two consequences worth knowing:

- **Absent `visible` means shown.** Only an explicit `visible = false` hides anything; nothing else in the model distinguishes hidden output.
- **The model has no platform vocabulary.** The same resolved tree renders the same way on every backend by construction — specifics (colors, fonts, hit testing) belong to platform adapters (○ not implemented).

## The text backend

The shipped backend, `TextBackend`, prints structural lines. For the snippet below (`examples/render.lucu` in the repository, rendered via `cargo run --example render`):

```text
App id="app" label="Demo app"
  .id = "app"
  .label = "Demo app"
  layout:
    .gap = 2
  Header id="header"
    .id = "header"
    Text id="title" label="Hello, Luca!"
      .id = "title"
      .text = "Hello, Luca!"
  Footer id="footer"
    .id = "footer"
    style:
      .tone = "soft"
    Text id="scale"
      .id = "scale"
      .text = 2.50
```

How to read each line:

| Output piece | Meaning |
|---|---|
| `Text id="title" label="Hello, Luca!"` | Component name, its `id` (if declared), its accessible label (if any) — the label is persisted into the resolved model (explicit `label` wins, else string `text` infers). |
| `.text = 2.50` | Resolved properties, `key = value`. Strings print quoted; numbers print with **exact decimal scale** (`2.50`, not `2.5`) — exact-decimal semantics come from Luca Code values. |
| `layout:` / `style:` sections | The component's layout/style entries, resolved like props but carrying no rendering semantics. Entry names are data only (○ layout/style semantics still open). |

Indentation is two spaces per depth level, and hidden subtrees vanish completely (rule 3).

What the text backend is **not**: it is not a visual preview. It exists so the whole pipeline (parse → state → resolve → render) can run headless in CI, in tests, and in the inspector (`luca-ui inspect`, see the [CLI reference](cli.md)).

## Rendering from code

```rust
use luca_ui::{resolve_tree, State, TextBackend};

let tree = luca_ui::parse(source)?;              // parse + lock-rule validation
let state = State::from_tree(&tree)?;             // declare all `state:` blocks
let resolved = resolve_tree(&tree, &state)?;      // substitute bindings
let text = TextBackend::render_to_string(&resolved)?;  // structural lines
```

The same steps power `TestHarness::render()` (see [Testing](testing.md)) and the `render` cargo example.

Custom backends implement the `Renderer` trait (one method, `render(&mut self, resolved_root)`), name themselves for selection, and stay inside the contract above. `is_visible` and `format_value` are provided as contract helpers so every backend skips and formats identically.

## Backend status matrix

| Backend | Status |
|---|---|
| Headless text backend (`TextBackend`) | ● Implemented — this release's verified rendering target |
| Native platform backend | ○ Architectural option — not implemented |
| Custom / GPU backend | ○ Architectural option — not implemented |
| Web backend | ○ Architectural option — not implemented |
| Platform accessibility adapters (label mapping) | ○ Architectural option — not implemented |

Because the core component vocabulary carries no platform concepts, none of the unimplemented options require changing `.lucu` files that target the text backend today (● locked design: no platform-specific components in the core vocabulary).

## Where next

- [Styling and theming](styling-theming.md) — what the `layout:`/`style:`/`theme:` blocks really are today.
- [CLI reference](cli.md) — rendering through `inspect` (debug builds).
- The `render.rs` and `localize.rs` cargo examples — runnable, checked-in, verified.
