# Styling, theming, and localization — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. This page covers the `layout:`, `style:`, `transitions:`, and `theme:` blocks plus the localization `Resources` API. The visual pipeline interprets a documented **◐ assumption subset** of `layout:`/`style:` entries; unknown entries stay opaque. Transition timing, theme tokens, and resource file formats remain ○ not implemented.

## What you will learn

- What each block declares, what the toolchain validates, and what it does **not** do.
- The ◐ layout/style subset used by the static visual pipeline.
- Accessibility in styling (reduced motion).
- The built-in `Resources` localization lookup.

## `layout:` and `style:` blocks

Every component may carry at most one `layout:` and one `style:` block:

```text
App:
  id = "app"
  layout:
    gap = 2
    direction = "column"
  style:
    tone = "soft"
```

Rules (● enforced):

- Entries are `key = value` scalar lines only — identifier keys, scalar values (literals or bindings).
- No duplicates, and imperative keys are rejected like everywhere else.
- Entries resolve against state like component properties (bindings substitute committed values).

### ◐ What the visual pipeline interprets

`luca-ui render`, `build_visual`, and `TestHarness::compose()` interpret this subset. Everything else remains opaque data: accepted, preserved, and ignored by the visual layer.

| `layout:` entry | Meaning |
|---|---|
| `direction = "column" \| "row"` | Main-axis stacking (default `column`). |
| `gap` | Space between siblings on the main axis (default `0`). |
| `padding` | Uniform inset around content (default `0`). |
| `width`, `height` | Explicit size; otherwise intrinsic content size. The root defaults to the viewport. |
| `align = "start" \| "center" \| "end" \| "stretch"` | Cross-axis alignment (default `start`); `stretch` fills the parent cross axis unless the child declares that axis. |

| `style:` entry | Meaning |
|---|---|
| `background = "#rrggbb"` | Rectangle fill. |
| `color = "#rrggbb"` | Text color. |
| `border = "#rrggbb"` | Rectangle stroke. |
| `radius` | Uniform corner radius in abstract units. |
| `font_size` | Text size in abstract units (default `16`). |

All numeric values are non-negative integers in abstract units. Colors accept six-digit hex only.

An entry the visual layer recognizes but cannot interpret is a **render warning**, not a parse error. The view still renders with the default:

```text
Render warning at 4:5: Unknown layout direction "grid"; expected "column" or "row"
```

Unknown entry names produce no warning — opacity is the contract. `tone = "soft"` above is a perfectly valid opaque token.

What is still **not** implemented (○ open): wrapping, clipping, scrolling, flex/grid algorithms, inheritance, responsive behavior, named colors, font shaping, theme-token resolution, and style precedence beyond “the component's own block.”

Why ship the subset at all? It proves the declarative shape end to end without inventing a full layout engine: entries flow through resolution, produce deterministic geometry and paint commands, and can gain richer semantics later without changing the source shape.

## `transitions:` blocks

Declarative animation entries, one block per component:

```text
Header:
  id = "header"
  transitions:
    fade = "in"
```

- Transitions are **declaration data** — there are no start/stop calls (● locked, consistent with the no-imperative rule).
- Timing, easing, interpolation: ○ not implemented. Entries carry no timing semantics.
- One real behavior is locked into this block: **reduced motion**. `TransitionSet::active(reduced_motion)` returns no transitions when the platform reports reduced motion, preserving the accessibility model:

```rust
use luca_ui::Environment;
let env = Environment::new(SystemAppearance::Light, /* reduced_motion */ true);
let entries = transitions.active(env.reduced_motion);  // empty slice
```

## `theme:` blocks

One block per component selects a theme:

```text
App:
  id = "app"
  theme:
    name = "Demo"
    mode = system
```

- `name` — optional display string.
- `mode` — `system` (the default), `light`, or `dark`. `system` follows the platform appearance automatically (● locked: automatic system appearance is core); the other two pin it.
- Unknown mode words fail: `Unknown theme mode 'sepia'; expected system, light, or dark`.
- Any other entry in the block is accepted as an **opaque token** with no styling semantics (○ theme tokens await the design record).

Theme resolution is explicit, not magical:

```rust
use luca_ui::{Environment, SystemAppearance, ThemeMode};

let effective = ThemeMode::System.effective(SystemAppearance::Light);
// → SystemAppearance::Light — "system" defers to the platform signal.
let effective = ThemeMode::Dark.effective(SystemAppearance::Light);
// → SystemAppearance::Dark — "dark" pins, regardless of the platform.
```

`theme::Environment` bundles the two platform signals (`SystemAppearance`, `reduced_motion`) that future adapters will supply. It is constructed explicitly in tests; nothing invents defaults.

## Localization: `Resources`

Localization is built in (● locked) as programmatic resource tables:

```rust
use luca_ui::Resources;

let mut resources = Resources::new("en");
resources.add("en", "greeting", "Hello, Luca!");
resources.add("de", "greeting", "Hallo, Luca!");

assert_eq!(resources.lookup("de", "greeting"), Some("Hallo, Luca!"));
assert_eq!(resources.lookup("de", "farewell"), None);           // missing key
assert_eq!(resources.lookup("fr", "greeting"), None);           // missing locale
assert_eq!(resources.default_locale(), "en");
assert_eq!(resources.locales(), vec!["de".to_owned(), "en".to_owned()]);
```

- Lookup is **exact-locale**: no fallback chains. A missing key or locale returns `None`, and the caller decides presentation.
- ○ Not implemented: file formats (`.json`/`.toml`/`.arb` …) and `.lucu` resource syntax — linking these tables to source files waits on the resource-format design record. The `localize` cargo example shows the verified lookup behavior.

## What the inspect output shows

Development builds snapshot layout declarations (see the [CLI reference](cli.md)):

```text
layout:
  App id="app":
    .gap = 2
    .direction = "column"
```

The inspector remains a **declaration** snapshot. Computed bounds are available through the visual pipeline:

```sh
luca-ui render examples/visual.lucu --out target/visual.svg
```

or in Rust via `build_visual(...)` / `TestHarness::bounds(id)` / `TestHarness::hit_test(point)` (see [Rendering](rendering.md) and [Testing](testing.md)).

## Where next

- [Rendering](rendering.md) — how layout/style data appears in backend output.
- [State and events](state-and-events.md) — the state model that layout/style entries bind to.
- [Examples](examples.md) — `layout.lucu` and `themes.lucu` with verified output.
