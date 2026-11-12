# Luca UI architecture — 1.0.2 Beta

This document describes the **shipped** architecture. It intentionally avoids
planning a future tree; the current modules are the contract. Status markers
match the user guides: ● locked, ◐ assumption, ○ not implemented.

## Product boundary

Luca UI is the declarative UI layer for Luca Code.

- `.lucu` files declare components, props, state, events, layout, style,
  themes, and transitions.
- `.lucc` files contain general-purpose logic and belong to Luca Code.
- Luca UI reuses Luca Code through its **public library API** only
  (`luca_code::interpreter`, `lexer`, `parser`, `ast`, `error`, `types`).
- **Luca Code never depends on Luca UI.** The dependency arrow is one-way and
  enforced by the package graph (`luca-ui → luca-code`).
- Luca UI is not a second general-purpose language and does not copy Luca
  Code's lexer, parser, type system, or interpreter.

## Runtime pipeline

```text
.lucu source
   │ parser + component validation
   ▼
Component tree ──────────────► State declarations
   │                              │
   │ resolve_tree                 │ set / flush
   ▼                              ▼
ResolvedComponent ◄──────────── committed values
   │
   ├── TextBackend  (Renderer)        structural text lines
   │
   └── build_visual
         │ compute_layout
         ▼
       LayoutNode tree
         │ build_display_list
         ▼
       DisplayList ──► SvgBackend (VisualBackend / Renderer)
         │
         └── input::hit_test / click_target
```

Everything above the backend boundary is platform-independent. Backends
receive UI output that is already derived from state; they never resolve
bindings, mutate state, or dispatch events.

## Modules (`src/`)

| Module | Responsibility |
|---|---|
| `lib.rs` | Public facade: module exports, `parse`, `run_paired`, product version. |
| `parser.rs` | `.lucu` indentation parser, scalar literals, comments, source-located errors. |
| `component.rs` | Component tree, props, identity, block extraction, per-component validation. |
| `library.rs` | Registered core inventory (`App`, `Text`, `Button`) and supported props. |
| `layout.rs` | `layout:` structural validation + ◐ `LayoutSpec` (`direction`, `gap`, `padding`, `width`, `height`, `align`). |
| `style.rs` | `style:` structural validation + ◐ `StyleSpec` (`background`, `color`, `border`, `radius`, `font_size`) and `Color`. |
| `state.rs` | Typed state entries, staged `set`/`flush`, global and per-key subscriptions, listener pruning. |
| `bindings.rs` | Bare-identifier binding resolution into `ResolvedComponent`. |
| `events.rs` | Declarative `events:` routing; dispatch returns a handler name. |
| `runtime.rs` | Tree swaps, state merging, mounted ids, scoped-task cancellation. |
| `seam.rs` | One-way Luca Code integration: run logic, paired paths, declaration-only handler checks. |
| `geometry.rs` | Platform-independent `Point`, `Size`, `Rect` and rectangle operations. |
| `layout_engine.rs` | Column/row layout, padding/gap/alignment, intrinsic sizing, approximate text metrics. |
| `paint.rs` | Display list: rect/text commands in deterministic paint order. |
| `pipeline.rs` | `Visual` and `build_visual`: resolved tree → layout → display list + diagnostics. |
| `input.rs` | Static hit testing over the layout tree; nearest-identified click target. |
| `rendering.rs` | `Renderer`, `VisualBackend`, `RenderDiagnostic`, `TextBackend`, `is_visible`, `format_value`. |
| `backends/svg.rs` | `SvgBackend`: deterministic, escaped, static SVG snapshots. |
| `testing.rs` | `TestHarness`: dispatch, state, readback, text/SVG rendering, bounds, hit testing. |
| `accessibility.rs` | Accessible descriptions: explicit label wins, string `text` infers. |
| `navigation.rs` | Component-id navigation stack (no routes). |
| `inspector.rs` | Development-only read-only snapshots (debug builds). |
| `animation.rs` | Declarative transition data + reduced-motion gating. |
| `themes.rs` | Theme name/mode, system appearance, environment signals. |
| `localization.rs` | Exact-locale `Resources` tables. |

## Invariants

1. **One-way seam.** No module imports UI types from Luca Code, and Luca Code
   has no reference to this repository. `tests/compatibility.rs` checks the
   generation pair.
2. **No component references or imperative mutation.** State changes only
   through `State::set`/`flush`; events return handler names; there is no
   `hide()`, `show()`, or `set_text()`.
3. **Errors are source-located.** Structural, state, binding, event, and seam
   failures are `LucaError` with frozen wording
   (`docs/diagnostics.md`, `tests/diagnostics.rs`).
4. **Visual assumption failures are non-fatal.** Recognized-but-invalid
   layout/style values produce `RenderDiagnostic` warnings; unknown entries
   stay opaque. Parsing remains structural.
5. **Backends are deterministic.** Same resolved input → same text/SVG output.
6. **Text metrics are approximate and documented.** No font shaping is
   claimed; geometry is a ◐ assumption.
7. **Hidden subtrees disappear completely.** Layout, paint, and hit testing
   exclude `visible = false` branches.

## The Luca Code seam

`seam.rs` offers:

- execution: `run_logic`, `run_logic_with_input` (guest `System.exit` is
  returned as an exit-marked error, never host termination);
- loading: `load_file` (`.lucu`), `load_logic` (`.lucc`), `paired_logic_path`,
  `load_paired`;
- declaration inspection: `declared_handlers` (public lexer/parser/AST) and
  `check_handlers` (`HandlerCheck`, `MissingHandler`).

Handler checking is **declaration-only**. It does not execute handlers and
does not resolve imports; with imports, results are reported as unverified.
Automatic execution remains ○ not implemented because Luca Code exposes no
public "call this function" API.

## Tests

- Per-module behavior: `tests/foundation.rs`, `layout.rs`, `state.rs`,
  `events.rs`, `navigation.rs`, `themes.rs` (via `library`/other suites),
  `inspector.rs`, `testing.rs`.
- Contracts: `tests/rendering.rs` (backend contract with an independent
  recorder), `tests/visual.rs` (geometry, paint, SVG, hit testing),
  `tests/diagnostics.rs` (frozen wording).
- Integration: `tests/seam.rs`, `tests/parser_regressions.rs`,
  `tests/examples.rs`, `tests/cli.rs` (real binary invocation).
- Compatibility: `tests/compatibility.rs` (Luca Code/Luca UI generation).

CI runs formatting, all-target tests, Clippy with `-D warnings`,
documentation, example smoke tests, an SVG smoke test, a release build, and
the VS Code extension tests.

## Not implemented

Native/GPU/web backends, interactive pointer/keyboard routing, platform
accessibility adapters, font shaping, wrapping/clipping/scrolling, full
layout/style semantics beyond the documented subset, transition timing,
theme-token resolution, resource file formats, and automatic
`.lucc`→state execution.
