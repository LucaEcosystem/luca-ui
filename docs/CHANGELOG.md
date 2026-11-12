# Changelog — Luca UI

All notable changes, per release. `PRODUCT_VERSION` is the user-facing
label; `Cargo.toml` carries the numeric package version. Feature status
(● locked / ◐ assumption / ○ not implemented) is documented in the
guides under `docs/`.

## 1.0.2 Beta (`PRODUCT_VERSION = "1.0.2 Beta"`, package `1.0.2`)

The 1.0.2 Beta drop: the documentation overhaul plus the first static visual
rendering, a stronger Luca Code seam, and `check`/`render` CLI commands.

### Added — visual rendering

- **Static SVG visual backend** (`SvgBackend`, `src/backends/svg.rs`): a
  deterministic, dependency-free snapshot renderer over a flat display list.
  Emits a self-contained document beginning with `<svg` (with `xmlns`,
  `width`, `height`, `viewBox`), escaped text, `<rect>`/`<text>` primitives.
  No interaction, animation, fonts, images, clipping, or accessibility tree.
- **Layout/style assumption specs** (`LayoutSpec`, `StyleSpec`, `Color`):
  the documented ◐ subset — `direction`, `gap`, `padding`, `width`, `height`,
  `align` and `background`, `color`, `border`, `radius`, `font_size` with
  `#rrggbb` colors. Unknown entries stay opaque (accepted, preserved,
  ignored); recognized-but-invalid values produce non-fatal
  `RenderDiagnostic` warnings instead of parse errors.
- **Layout engine** (`src/layout_engine.rs`): column/row stacking, uniform
  padding, main-axis gap, cross-axis alignment (start/center/end/stretch),
  explicit or intrinsic sizes, and approximate text metrics
  (`font_size × 0.6` per character, `font_size × 1.2` per line). Hidden
  subtrees are excluded before measurement.
- **Paint + pipeline** (`src/paint.rs`, `src/pipeline.rs`): a flat
  `DisplayList` (rect/text commands in deterministic paint order) and
  `build_visual`, which turns a resolved tree into a `Visual` (layout tree,
  display list, diagnostics).
- **Static hit testing** (`src/input.rs`): `hit_test`, `hit_test_id`, and
  `click_target` (nearest identified ancestor). Reports identity only; it does
  not dispatch events or execute handlers.
- **Geometry** (`src/geometry.rs`): platform-independent `Point`, `Size`,
  `Rect`.

### Added — integration seam

- `load_logic`, `run_logic_file`, `paired_logic_path`, `load_paired`,
  `declared_handlers`, `check_handlers`, `HandlerDeclarations`,
  `HandlerCheck`, `MissingHandler`. Handler validation is declaration-only
  (public lexer/parser/AST); imports are reported as unverified.
- Key-aware state subscriptions: `State::subscribe_key`, `unsubscribe_key`,
  `is_subscribed_to`, with key-aware `flush` and `prune`.

### Added — tooling

- CLI `check` and `render` subcommands. `check` validates the whole static
  picture and enforces locally-declared handlers; `render` writes a static
  SVG snapshot to stdout or `--out`, with `--width`/`--height`.
- `TestHarness` gained `click`, `visual`, `visual_default`, `bounds`,
  `hit_test`, and `compose`.
- `examples/visual.lucu` and `examples/visual.rs`; `tests/visual.rs`
  (visual foundation) and `tests/cli.rs` (binary-level CLI).

### Fixed

- Parser: trailing text after a string literal is now an error; backslash
  parity in escape handling corrected. Six regression tests.
- `TextBackend::render` clears prior output so repeated renders cannot leak
  stale lines.
- Stale 0.x Alpha headers updated; wrong Luca Code links corrected.
- Removed the empty `compiler/`, `runtime/`, and `std/` scaffolding
  directories.

### Documentation and packaging

- Package version `1.0.2`; `PRODUCT_VERSION` reads `1.0.2 Beta`.
- Documentation overhaul: a user-facing guide set under `docs/`
  (getting started, `.lucu` language reference, components, state and
  events, testing, rendering, styling/theming/localization, examples
  index, CLI reference, architecture, plus a docs index) with verified
  outputs for every snippet; the root README is a rewritten landing page.
- VS Code extension updated to 1.0.2 with an expanded README
  (settings, troubleshooting, Marketplace-style install paths).
- Root `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, GitHub issue/PR templates, and a refreshed
  `docs/RELEASE-NOTES.md`.
- `tests/compatibility.rs` now compares the two products' release labels
  at generation level (shared `1.0` prefix) so patch releases within the
  generation stay green.

No frozen structural diagnostics wording changed; new render warnings cover
the visual assumption subset only.

## 1.0 Beta (initial publication, `PRODUCT_VERSION = "1.0 Beta"`, package `1.0.0`)

Publication Beta — **not stable 1.0**. Validation and stabilization of the
0.9 candidate; no locked language behavior changed.

- `luca-code` path dependency gains its `version = "1.0.0"` requirement
  (packaging metadata only; resolution and behavior unchanged).
- Release artifacts: offline-capable two-tree source package, release
  binary tarball, `SHA256SUMS` (see `docs/RELEASE-NOTES.md`).
- Publication prep: `luca-ui` VS Code extension (`.lucu` highlighting,
  snippets, run command; 10 unit tests; packaged `.vsix`), CI workflow,
  tag-driven release workflow (GitHub release + optional Marketplace
  publish via `VSCE_PAT`), repository initialized for first push.
- Docs: `docs/CHANGELOG.md`, `docs/RELEASE-NOTES.md`,
  `docs/compatibility.md` refreshed for the Beta pair.

## 0.9 Alpha (Beta candidate, unpublished)

- Minimal core library (App/Text/Button) with per-component validation;
  open vocabulary elsewhere (`src/library.rs`).
- Deterministic `TestHarness` UI testing API (`src/testing.rs`).
- Frozen diagnostics catalog (`docs/diagnostics.md`) with exact-message tests.
- Development-only inspector: read-only snapshots + debug-gated
  `luca-ui inspect` (`src/inspector.rs`).
- Full SPEC section/decision reconciliation matrix with test evidence.

## 0.8 Alpha

- Declarative `transitions:` data blocks with reduced-motion gating.
- First-class `Theme` (name/mode, system-follow default) + `Environment`.
- Built-in `Resources` exact-locale string lookup.

## 0.7 Alpha

- `Renderer` contract + headless `TextBackend`; native/GPU/web explicitly
  unimplemented options.

## 0.6 Alpha

- Component-id `Navigator` stack (no routes); core `Description`
  accessibility model (explicit labels win, `text` infers).

## 0.5 Alpha

- `events:` routing blocks with declarative `dispatch`; `Runtime`
  lifecycle (tree swaps, state merge, listener prune, scoped-task
  cancellation).

## 0.4 Alpha

- `Value`-typed `State` with Luca Code semantics; `state:` literal blocks;
  bare-identifier bindings resolved by `resolve_tree`; batched
  `set`/`flush`.

## 0.3 Alpha

- Platform-independent `layout:`/`style:` declaration blocks, structural
  validation only.

## 0.2 Alpha

- Component declarations, `id` identity/uniqueness, locked prop rules,
  tree structure (`count`/`depth`/`find_by_id`).

## 0.1 Alpha

- Bare-minimum foundation: crate layout, `.lucu` handling, `LucaError`
  diagnostics, generic component tree, one-way Luca Code seam, first
  example and tests.
