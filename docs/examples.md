# Examples — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. Nine `.lucu` examples (two with paired `.lucc` logic) and three Rust examples ship in [`examples/`](../examples). Every output below was captured by actually running the example with this release. Each `.lucu` example also runs in CI (`tests/examples.rs` renders every shipped `.lucu` file through the text backend; the SVG smoke test renders `visual.lucu`).

Run any of them with:

```sh
cargo run -- examples/<name>.lucu
```

## Index

| File | Teaches |
|---|---|
| [`hello.lucu`](../examples/hello.lucu) | Smallest correct tree (with paired `hello.lucc`) |
| [`declarations.lucu`](../examples/declarations.lucu) | Nesting, ids, locked props on several components |
| [`layout.lucu`](../examples/layout.lucu) | `layout:`/`style:` blocks, including the ◐ visual subset and opaque entries |
| `state.lucu` | `state:` blocks + bare-identifier bindings |
| `events.lucu` + `events.lucc` | Event routing to Luca Code handler names |
| `navigation.lucu` | Component-stack navigation shape + labels |
| `themes.lucu` | `transitions:`, named/pinned `theme:` blocks |
| `render.lucu` (via `cargo run --example render`) | The text backend: resolution, labels, visibility skip |
| `visual.lucu` (via `cargo run --example visual` or `luca-ui render`) | The static SVG backend: layout, paint, colors, text |
| `localize.rs` (via `cargo run --example localize`) | Built-in `Resources` exact-locale lookup |

---

## `hello.lucu` — first tree

```sh
luca-ui examples/hello.lucu        # or: cargo run -- examples/hello.lucu
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

`hello.lucc` (paired logic, run by the `luca` CLI / `luca_code`):

```sh
cargo run -p luca-code --manifest-path "../Luca Code/Cargo.toml" -- examples/hello.lucc
```

```text
Hello, Luca!
```

## `declarations.lucu` — nesting and locked props

```text
App (4:1) id="app"
  .id = "app"
  .visible = true
  Header (7:3) id="header"
    .id = "header"
    .visible = true
    Text (10:5) id="title"
      .id = "title"
      .text = "Luca UI"
  Body (13:3) id="body"
    .id = "body"
    .disabled = false
    Text (16:5) id="greeting"
      .id = "greeting"
      .text = "Hello, Luca!"
      .visible = true
```

Read it top-down: one `App` root; `Header` and `Body` may coexist as siblings; `disabled = false` is an ordinary property; ids are unique across all five nodes.

## `layout.lucu` — layout/style blocks

```text
App (5:1) id="app"
  .id = "app"
  .visible = true
  layout (8:3)
    .gap = 2
    .direction = "column"
  style (11:3)
    .tone = "soft"
  Header (13:3) id="header"
    .id = "header"
    style (15:5)
      .emphasis = true
    Text (17:5) id="title"
      .id = "title"
      .text = "Luca UI"
  Body (20:3) id="body"
    .id = "body"
    .disabled = false
    layout (23:5)
    Text (24:5) id="greeting"
      .id = "greeting"
      .text = "Hello, Luca!"
      .visible = true
```

Blocks appear as sections on the component that owns them. An empty block (like `Body`'s `layout:`) is valid. `gap` and `direction` belong to the documented ◐ visual subset; `tone` and `emphasis` are opaque tokens. See [Styling and theming](styling-theming.md).

## `state.lucu` — state and bindings

```text
App (6:1) id="app"
  .id = "app"
  state (8:3)
    .title = "Luca UI"
    .visits = 2
    .ratio = 2.50
    .show_detail = true
  Text (13:3) id="title"
    .id = "title"
    .text = title
    .visible = show_detail
  Text (17:3) id="counter"
    .id = "counter"
    .text = visits
```

The CLI prints declarations; bindings stay bare names in this view. To see the *resolved* output of exactly this file:

```text
App id="app"
  Text id="title" label="Luca UI"
    .id = "title"
    .text = "Luca UI"
    .visible = true
  Text id="counter"
    .text = 2
```

`text = title` resolved to `"Luca UI"`, `visible = show_detail` resolved to `true`, and `text = visits` — an `int` — resolved to `2`. Verified through the resolution + text-backend path this release uses everywhere.

## `events.lucu` + `events.lucc` — routing to logic

```text
App (6:1) id="app"
  .id = "app"
  state (8:3)
    .visits = 0
    .greeting = "Hello, Luca!"
  Text (11:3) id="greeting"
    .id = "greeting"
    .text = greeting
  Button (14:3) id="visit"
    .id = "visit"
    .text = "Visit"
    events (17:5)
      .tap -> record_visit
  Button (19:3) id="reset"
    .id = "reset"
    .text = "Reset"
    events (22:5)
      .tap -> reset_visits
```

Two components may each declare their own `tap`; routing is per component. The paired logic runs on the real Luca Code interpreter:

```sh
cargo run -p luca-code --manifest-path "../Luca Code/Cargo.toml" -- examples/events.lucc
```

```text
visits = 1
visits = 2
visits = 0
```

This pair is the [wiring contract](state-and-events.md#5-wiring-events-to-logic-the-intended-contract) exercised end-to-end: dispatch does routing; the handlers are ordinary Luca Code functions; results application goes through `set`/`flush`.

## `navigation.lucu` — screens as components

```text
App (5:1) id="app"
  .id = "app"
  Home (7:3) id="home"
    .id = "home"
    .label = "Home screen"
    Text (10:5) id="welcome"
      .id = "welcome"
      .text = "Welcome"
    Button (13:5) id="open"
      .id = "open"
      .text = "Open details"
      events (16:7)
        .tap -> show_detail
  Detail (18:3) id="detail"
    .id = "detail"
    Text (20:5) id="body"
      .id = "body"
      .text = "Details here"
    Button (23:5) id="back"
      .id = "back"
      .label = "Go back"
      .text = "Back"
      events (27:7)
        .tap -> show_home
```

Both screens are declared in one tree and addressed by `id` (`home`, `detail`) — there are no routes. A `Navigator` stack (in Rust: `luca_ui::Navigator`) pushes/pops/replaces those ids; unknown ids and popping the root screen are errors (● locked: component-based navigation). Note the labels: `home` gets an explicit label; `back` demonstrates `label` winning over `text` ("Back") for accessibility.

## `themes.lucu` — transitions and themes

```text
App (7:1) id="app"
  .id = "app"
  theme (9:3)
    .name = "Demo"
    .mode = system
  Header (11:3) id="header"
    .id = "header"
    transitions (13:5)
      .fade = "in"
    Text (15:5) id="title"
      .id = "title"
      .text = "Luca UI"
  Detail (18:3) id="detail"
    .id = "detail"
    theme (20:5)
      .name = "Demo dark"
      .mode = dark
    transitions (23:5)
```

`mode = system` follows platform appearance; `Demo dark` pins dark. The `transitions:` entries are data only (no timing semantics yet; reduced-motion gates them off), and `Detail`'s empty `transitions:` block shows an empty set is valid.

## `visual.lucu` — static SVG snapshot (`cargo run --example visual`)

```sh
cargo run --example visual                          # SVG on standard output
# or: luca-ui render examples/visual.lucu --out visual.svg --width 320 --height 200
```

```text
<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200" viewBox="0 0 320 200">
<rect x="0" y="0" width="320" height="200" fill="#101418" stroke="#30363d" rx="8"/>
<rect x="12" y="12" width="296" height="32" fill="#1f6feb" rx="4"/>
<text x="18" y="34" font-size="16">Luca UI visual snapshot</text>
<text x="12" y="68" font-size="16">Hello, Luca!</text>
<text x="140" y="66" font-size="12" fill="#8b949e">Static SVG</text>
</svg>
```

What the snapshot demonstrates: the root fills the viewport (`320×200`), the column stack uses `gap = 8` and `padding = 12`, the header stretches across the cross axis, and the body is a row with centered cross-axis alignment. Text sizes come from `font_size`; positions come from the approximate ◐ cell metrics. Styles do **not** inherit: each node paints only its own `style:` block. The output is static — no interaction, fonts, or accessibility tree.

## `render.rs` — the text backend (`cargo run --example render`)

```sh
cargo run --example render
```

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
      .text = 2.50
```

Three contract behaviors in one output: `2.50` keeps its exact decimal scale; `title`'s label is **inferred** from its string text; and the `Preview` subtree (`visible = show_preview`, a state variable committing `false`) is skipped entirely — backend rule 3.

## `localize.rs` — resource lookup (`cargo run --example localize`)

```sh
cargo run --example localize
```

```text
en: Hello, Luca!
de: Hallo, Luca!
de farewell: (missing — no fallback chain)
```

Exact-locale lookup, no fallback chains — a missing key/locale returns nothing and the caller decides (see [Styling and theming — localization](styling-theming.md#localization-resources)). *(The repository example prints its missing-key line as `de farewell: (missing — no fallback chain in 0.8)` in versions prior to 1.0.2 Beta; this output reflects 1.0.2 Beta.)*
