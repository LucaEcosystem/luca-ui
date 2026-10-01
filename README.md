# Luca UI 1.0 Beta (publication Beta — not stable 1.0)

Declarative UI layer on Luca Code. Files: `.lucu` for UI, `.lucc` for logic.
Luca Code stays independent — this toolchain only reads `.lucc` via the
published `luca_code` library; it never copies the compiler or edits `.lucc`
behavior.

## Install (1.0 Beta)

Requires Rust (stable) with the Luca Code checkout as a sibling directory
(the `luca-code` dependency is a relative path dependency):

```sh
cargo install --path /path/to/Luca-UI --root ~/.luca-ui-1.0-beta
export PATH="$HOME/.luca-ui-1.0-beta/bin:$PATH"
luca-ui --version   # luca-ui 1.0 Beta
```

No install scripts ship with this Beta; no Marketplace/registry release
exists yet (see `docs/implementation-status.md` §18).

## VS Code extension (1.0 Beta)

`editors/vscode/` holds the official Luca UI extension: `.lucu` syntax
highlighting, snippets, a starter-file command, and one-key run through
the `luca-ui` CLI (`lucaui.cliPath` setting, `Output > Luca UI` channel,
`Error at line:column:` diagnostics). Zero runtime dependencies;
`npm test` runs its 10 unit tests. Until it is published, install the
local build with `code --install-extension editors/vscode/luca-ui-1.0.0.vsix`.

## Run the examples

```sh
cargo run -- examples/hello.lucu
cargo run -- examples/declarations.lucu
cargo run -- examples/layout.lucu
cargo run -- examples/state.lucu
cargo run -- examples/events.lucu
cargo run -- examples/navigation.lucu
```

Render `examples/render.lucu` with the text backend (the implemented target):

```sh
cargo run --example render
```

Validate `examples/themes.lucu` and run the localization lookup demo:

```sh
cargo run -- examples/themes.lucu
cargo run --example localize
```

Inspect tree, state, and layout declarations (debug builds only):

```sh
cargo run -- inspect examples/layout.lucu
```

`examples/declarations.lucu` (assumption syntax, not locked design):

```text
App:
  id = "app"
  visible = true
  Header:
    id = "header"
    visible = true
    Text:
      id = "title"
      text = "Luca UI"
  Body:
    id = "body"
    disabled = false
    Text:
      id = "greeting"
      text = "Hello, Luca!"
      visible = true
```

Paired logic `examples/hello.lucc` runs with the `luca` interpreter:

```sh
cargo run -p luca-code --manifest-path "../Luca Code/Cargo.toml" -- examples/hello.lucc
```

## Rules enforced in 1.0 Beta (SPEC-locked only)

- Components nest into a single-root tree; malformed declarations
  (`Name:` shape, properties outside a component, second roots) are
  compile-time Luca errors.
- `id` is supported on every component: non-empty string, unique across the
  tree (suitable for tests); duplicates and non-string ids are Luca errors.
- `visible`/`disabled` are ordinary boolean properties wherever they appear;
  non-bool values are Luca errors. `label`, when present, must be a string.
- Declarative only: `hide`/`show`/`set_text` keys, call-shaped lines, and
  call values are rejected — update state and allow the UI to react. There
  is no component-mutation API; `set`/`flush` on `State` is the only way
  output changes.
- State and bindings: one `state:` literal block per component declares typed
  variables (`str`/`bool`/`int`/`dec` inferred, `none` infers `uni`) with Luca
  Code value semantics (exact decimals, `none`-occupies-any-type). A
  bare-identifier value (e.g. `text = title`) is a binding resolved against
  committed state when the tree is resolved; undeclared names and
  wrongly-typed locked props fail at the binding site. Updates are staged by
  `set` and committed together by `flush` (one notification per listener per
  changed key); listeners of removed components are pruned so their pending
  work is never emitted.
- Events are the interaction boundary: one `events:` block per component
  holds `event = handler` entries where the handler names Luca Code logic.
  Dispatch validates the component id and event and returns the handler
  name; the logic runs on the existing seam and feeds back only through
  state. Dispatch exposes no component references.
- Lifecycle is runtime-managed: tree swaps report mounted/unmounted ids,
  keep committed state, prune listeners of removed components, and cancel
  their scoped tasks. Scoping work to an unmounted component is an error.
- Navigation is component-based: a stack of component ids with push / pop /
  replace over one declared tree. There are no routes, route tables, or path
  syntax; unknown ids and popping the root screen are Luca errors.
- Accessibility is core: every component describes an optional label —
  an explicit `label` always wins, a string `text` is inferred otherwise,
  and bound values participate through resolved output. Missing labels are
  valid; non-string labels are Luca errors. The model carries no platform
  concepts; adapter mapping is deferred.
- Layout and styling are declarative, platform-independent attachments: at
  most one `layout:` and one `style:` block per component, holding only
  `key = value` scalar entries. No platform, unit, theme-token, or
  precedence concepts exist in the component model. Entry names carry no locked
  meaning; no dimensions, defaults, or precedence are defined.
- Rendering is a platform-independent abstraction: every backend receives
  the resolved tree once, visits visible nodes depth-first, skips
  `visible == false` subtrees, reads only the shared model, and renders
  deterministically. The implemented target is the headless text backend
  (structural lines; exact decimal scale; interprets no component names).
  Native, custom/GPU, and web backends are intended options and are NOT
  implemented.
- Animation is declarative data: at most one `transitions:` block per
  component holds generic entries with no timing semantics and no
  start/stop calls. Transitions gate on platform reduced-motion (on:
  suppressed), preserving accessibility.
- Themes are first-class: at most one `theme:` block per component with an
  optional `name` and `mode` (`system` follows platform appearance
  automatically by default; `light`/`dark` pin it). Other entries are
  opaque future tokens with no styling semantics.
- Localization is built in as resource tables with exact-locale lookup;
  missing keys/locales return nothing. No file formats, fallback chains,
  or `.lucu` resource syntax exist yet.
- Core library: `App` (root container), `Text` (labeled display), and
  `Button` (labeled action) share the locked API and are cross-platform by
  construction. Unknown props on registered components are Luca errors
  naming the supported list; unregistered names stay generic (open
  vocabulary — the full inventory awaits the design record).
- Testing API: `TestHarness` drives a tree deterministically — dispatch by
  id, `set`/`flush` state, swap trees, read resolved props/labels/
  visibility, render — addressing components by `id` only.
- Diagnostics are frozen: every failure is `Error at line:column: message`
  with cataloged wording (`docs/diagnostics.md`); wording changes are
  breaking changes.
- Inspector (development only): read-only snapshots of component tree,
  committed state, and layout declarations. Layout *bounds* need dimension
  semantics the locked design does not define, so they are explicitly
  deferred, not silently assumed.
- Every failure is a source-located Luca diagnostic:
  `Error at line:column: message`.
- Only `.lucu` files are loaded; other extensions are rejected.

Unregistered component names accept generic props; only malformed or
locked-rule-violating properties are rejected there. Broader grammar and
the full component library await the full design record — see
`docs/implementation-status.md` (§16 reconciliation matrix),
`docs/compatibility.md`, and `docs/SPEC.md`. Do
not treat 1.0 Beta assumption syntax as locked.
