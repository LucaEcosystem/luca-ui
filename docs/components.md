# Components — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. Everything on this page is enforced behavior.

## What you will learn

- The three registered core components and their accepted properties.
- How every other component name is treated (the open vocabulary).
- How accessibility labels work.
- What the text backend does with each of them.

## The registered core set

Three components form the registered core — the only names with enforced per-name property lists:

| Component | Purpose |
|---|---|
| `App` | Application root container |
| `Text` | Labeled text display |
| `Button` | Labeled action with routed events |

All three share the same locked property set:

| Property | Meaning |
|---|---|
| `id` | Unique string identifier |
| `visible` | boolean; false hides the subtree |
| `disabled` | boolean; ordinary property |
| `label` | string; explicit accessible label |

And they add one generic property each:

| Component | Extra property | Meaning |
|---|---|---|
| `App` | — | — |
| `Text` | `text` | shown content (string) |
| `Button` | `text` | shown content (string) |

Validation is per-name and strict: an unknown property on a registered component is a compile-time error listing what it supports.

```text
-- `Text` rejects anything beyond its list.
App:
  id = "app"
  Text:
    id = "t"
    width = 12
```

```text
Error at 6:5: Unknown property 'width' on component 'Text' (supports: disabled, id, label, text, visible)
```

*Verified with `luca-ui 1.0.2 Beta`.*

## Unregistered components (open vocabulary)

Any other component name parses and validates by the universal locked rules only:

```text
App:
  id = "app"
  Label:
    id = "x"
    text = "ok"
```

```text
App (1:1) id="app"
  Label (2:3) id="x"
    .id = "x"
    .text = "ok"
```

Unregistered components:

- Accept generic properties (no per-name list is enforced).
- Still enforce the universal rules: duplicates, `id` uniqueness and shape, `visible`/`disabled`/`label` types, and no imperative operations.
- Support `state:`, `events:`, `layout:`, `style:`, `transitions:`, `theme:` blocks exactly like registered ones.
- Are addressed by `id` in tests, events, and navigation like any other component.

Why open: the full component inventory awaits the locked design record (it is not invented). Until then, only `App`/`Text`/`Button` carry enforced name semantics; everything else is a valid component with open properties. ○ **Not implemented:** the full inventory.

## How components are addressed: by `id` only

Id values are strings, and every test, event dispatch, and navigation step uses the `id` — never an object reference. There is no API for getting a "component handle"; addressing happens through the string id. This is why uniqueness across the tree is enforced.

## Accessibility labels

Every component describes an optional accessible label with this precedence:

1. An explicit `label` property — always wins.
2. Otherwise, if the component has a `text` property whose resolved value **is a string**, the text is offered as the inferred label.
3. Otherwise: no label. This is valid — not every component has accessible text.

Examples:

```text
Text:
  id = "a"
  text = "Open details"
```

Label: `"Open details"` (inferred).

```text
Text:
  id = "b"
  label = "Open details"
  text = "…"
```

Label: `"Open details"` (explicit wins).

Bound values participate: if `text = dynamic_text` and `dynamic_text` commits a string, the inferred label follows it.

The label is carried through the resolution and rendering contract — the text backend prints `label="…"` for each component that has one (see [Rendering](rendering.md)); the TestHarness exposes it ([`label`](testing.md#3-reading-resolved-output)).

○ Not implemented: mapping the shared labels to platform accessibility systems (native UI kits, web ARIA). The model itself stays platform-independent.

## Rendering them

The text backend renders every component structurally, with no special casing:

```text
App id="app" label="Demo app"
  .id = "app"
  .label = "Demo app"
  Text id="greeting" label="Hello, Luca!"
    .id = "greeting"
    .text = "Hello, Luca!"
```

The backend does not interpret component names at all — it renders the same structure for every name (see [Rendering](rendering.md)). Real visual presentation comes from future platform backends (both native and web), which will adapt the same shared names.

## API surface for components

If you drive Luca UI from Rust, components are the parsed tree (`luca_ui::Component`), the resolved form (`luca_ui::ResolvedComponent`), and their helpers; every registered name is listed in `luca_ui::CORE_COMPONENTS`. See [testing.md](testing.md) for the driver used in tests.
