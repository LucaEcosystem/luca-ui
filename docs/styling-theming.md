# Styling, theming, and localization — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. This page covers the `layout:`, `style:`, `transitions:`, and `theme:` blocks plus the localization `Resources` API. Where the topic is honest about its limits, read: layout/style entry semantics, transition timing, theme tokens, and resource file formats are **not implemented** — the blocks exist, they are validated, and they are deliberately left semantics-free until the full design record lands.

## What you will learn

- What each block declares, what the toolchain validates, and what it does **not** do.
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

What they do **not** do (○ open): entry names have **no global meaning**. No dimensions, no units, no defaults, no precedence, no inheritance, no responsive behavior — there is no grid engine or flex algorithm behind `gap = 2`. The blocks are validated containers awaiting layout/style **semantics** from the design record. Treat every entry as opaque data with a stable home in the tree and diagnostics.

Why ship them at all? Two reasons: the *shape* of the declarative syntax is exercised and validated now, so moving names into real semantics later is a toolchain change — not a source break for files already written this way; and they flow through resolution and every backend honestly (the text backend prints them as data sections).

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

Layout **bounds** (computed rectangles) are not reported anywhere — they need dimension semantics that do not exist yet (○ open, explicitly deferred rather than silently assumed).

## Where next

- [Rendering](rendering.md) — how layout/style data appears in backend output.
- [State and events](state-and-events.md) — the state model that layout/style entries bind to.
- [Examples](examples.md) — `layout.lucu` and `themes.lucu` with verified output.
