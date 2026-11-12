# Rendering — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. Rendering ships two concrete backends: the headless **text backend** and a static **SVG visual backend**. ● The rendering contract is locked; ◐ the text line format, layout/style subset, text metrics, and SVG shape are labeled assumptions.

## What you will learn

- What a backend receives and must guarantee.
- How the static visual pipeline turns resolved output into paint commands.
- The ◐ layout/style subset the visual pipeline understands.
- What the text and SVG backends print, and how to render from code.
- What is deliberately **not** implemented.

## The rendering contract

Every backend — today's text and SVG backends, and any future interactive/native/GPU/web backend — must honor five rules:

1. It receives the resolved tree exactly once per `render` call — *UI output already derived from state*, not a tree with bindings unresolved.
2. It visits visible nodes depth-first, parents before children.
3. It skips `visible = false` subtrees entirely — no partial rendering of hidden branches.
4. It reads only the platform-independent model (resolved components, labels, values). It introduces no platform components and names no platform, renderer, or unit concepts.
5. It is deterministic — same input, same output, every run.

Two consequences worth knowing:

- **Absent `visible` means shown.** Only an explicit `visible = false` hides anything.
- **The model has no platform vocabulary.** Interactive behavior, platform fonts, and accessibility adapters belong to future platform layers (○ not implemented).

`TextBackend` implements the `Renderer` trait. The visual pipeline is exposed separately through `VisualBackend`, which consumes a flat `DisplayList`; `SvgBackend` implements both traits.

## The static visual pipeline

```text
resolved tree
   │  resolve_tree
   ▼
layout engine (◐ assumption subset)
   │  compute_layout
   ▼
layout tree (border boxes in abstract units)
   │  build_display_list
   ▼
display list (rect / text commands, paint order)
   │  VisualBackend
   ▼
SVG snapshot
```

```rust
use luca_ui::{build_visual, parse, resolve_tree, Size, State, SvgBackend};

let tree = parse(source)?;
let state = State::from_tree(&tree)?;
let resolved = resolve_tree(&tree, &state)?;
let visual = build_visual(&resolved, Size::new(320, 200))?;

for warning in &visual.diagnostics {
    eprintln!("{warning}");                 // non-fatal assumption diagnostics
}

let mut backend = SvgBackend::new();
backend.render_visual(&visual)?;
println!("{}", backend.output());
```

The same pipeline powers:

```sh
luca-ui render examples/visual.lucu --out target/visual.svg --width 320 --height 200
```

`luca-ui check` builds the visual as well and prints the assumption diagnostics without writing a file.

### ◐ Layout subset

Only these entries have visual meaning. All values are non-negative integers in abstract units unless noted.

| Entry | Meaning |
|---|---|
| `direction = "column" \| "row"` | Main-axis stacking (default `column`). |
| `gap` | Space between siblings on the main axis (default `0`). |
| `padding` | Uniform inset between border box and content (default `0`). |
| `width`, `height` | Explicit size; otherwise intrinsic content size. The root defaults to the viewport. |
| `align = "start" \| "center" \| "end" \| "stretch"` | Cross-axis alignment (default `start`); `stretch` fills the parent cross axis unless the child declares that axis. |

Any other `layout:` entry stays **opaque**: accepted, preserved, and ignored by the visual layer.

### ◐ Style subset

| Entry | Meaning |
|---|---|
| `background = "#rrggbb"` | Rectangle fill. |
| `color = "#rrggbb"` | Text color. |
| `border = "#rrggbb"` | Rectangle stroke. |
| `radius` | Uniform corner radius in abstract units. |
| `font_size` | Text size in abstract units (default `16`). |

Any other `style:` entry stays opaque. Colors accept six-digit hex only; named colors are not interpreted.

### ◐ Text metrics

Text is measured with a documented approximate cell model, not font shaping: one character is `font_size × 0.6` wide, one line is `font_size × 1.2` tall, and `\n` starts a new line. Unicode scalar values each occupy one cell. This makes layout deterministic and dependency-free, and it is intentionally not typographic.

### Paint order and hit testing

The display list paints parents before children; within one node it paints the background/border rectangle, then its text, then its children. There is no clipping, opacity, compositing, or animation.

Hit testing is static geometry over the same layout tree: the deepest node containing a point wins, and `click_target` returns the nearest identified ancestor. Hit testing reports a component `id`; it does **not** dispatch an event or execute a handler.

### ◐ SVG shape

`SvgBackend` emits a deterministic document that starts with `<svg`, includes `xmlns`, `width`, `height`, and `viewBox`, and contains `<rect>`/`<text>` primitives with XML-escaped text. It is a snapshot renderer:

- no XML declaration, stylesheet, external font, image, or script;
- no interaction, animation, clipping, or accessibility tree;
- not a browser/rendering-engine claim — just inspectable vector primitives.

## The text backend

`TextBackend` prints structural lines. For the snippet below (`examples/render.lucu` in the repository, rendered via `cargo run --example render`):

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
| `Text id="title" label="Hello, Luca!"` | Component name, its `id` (if declared), its accessible label (if any) — explicit `label` wins, else string `text` infers. |
| `.text = 2.50` | Resolved properties, `key = value`. Strings print quoted; numbers print with **exact decimal scale** (`2.50`, not `2.5`). |
| `layout:` / `style:` sections | The component's entries, resolved like props. The text backend prints them as data; the visual pipeline interprets the ◐ subset. |

Indentation is two spaces per depth level, and hidden subtrees vanish completely (rule 3).

## Rendering from code

Text backend:

```rust
use luca_ui::{resolve_tree, State, TextBackend};

let tree = luca_ui::parse(source)?;
let state = State::from_tree(&tree)?;
let resolved = resolve_tree(&tree, &state)?;
let text = TextBackend::render_to_string(&resolved)?;
```

SVG backend:

```rust
use luca_ui::{resolve_tree, State, SvgBackend};

let tree = luca_ui::parse(source)?;
let state = State::from_tree(&tree)?;
let resolved = resolve_tree(&tree, &state)?;
let svg = SvgBackend::render_to_string(&resolved)?; // 320×240 default
```

`TestHarness` exposes the same pipeline through `visual()`, `bounds()`, `hit_test()`, and `compose()` (see [Testing](testing.md)).

## Diagnostics

Structural failures remain source-located **errors** (`Error at line:column: message`, frozen catalog). The visual layer adds non-fatal **render warnings**:

```text
Render warning at 4:5: Unknown layout direction "grid"; expected "column" or "row"
```

A recognized entry with an invalid value warns and falls back to its default; the view still renders. Unknown entry names produce no warning at all — they are intentionally opaque.

## Backend status matrix

| Backend | Status |
|---|---|
| Headless text backend (`TextBackend`) | ● Implemented and tested |
| Static SVG snapshot (`SvgBackend`) | ● Implemented and tested; non-interactive |
| Native platform backend | ○ Architectural option — not implemented |
| Custom / GPU backend | ○ Architectural option — not implemented |
| Web backend | ○ Architectural option — not implemented |
| Platform accessibility adapters | ○ Architectural option — not implemented |
| Interactive pointer/keyboard routing | ○ Not implemented; only static hit testing exists |

Because the core component vocabulary carries no platform concepts, none of the unimplemented options require changing `.lucu` files.

## Where next

- [Styling and theming](styling-theming.md) — the full `layout:`/`style:`/`theme:` story.
- [CLI reference](cli.md) — `check` and `render`.
- [Testing](testing.md) — bounds, hit testing, and SVG composition through `TestHarness`.
