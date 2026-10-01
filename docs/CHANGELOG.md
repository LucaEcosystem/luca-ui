# Changelog — Luca UI

All notable changes, per staged Alpha milestone. `PRODUCT_VERSION` is the
user-facing label; `Cargo.toml` carries the numeric packaging version.
Assumption-status notes live in `docs/implementation-status.md`.

## 1.0 Beta (`PRODUCT_VERSION = "1.0 Beta"`, package `1.0.0`)

Publication Beta — **not stable 1.0**. Validation and stabilization of the
0.9 candidate; no locked language behavior changed.

- `luca-code` path dependency gains its `version = "1.0.0"` requirement
  (packaging metadata only; resolution and behavior unchanged).
- Release artifacts: offline-capable two-tree source package, release
  binary tarball, `SHA256SUMS` (see `docs/RELEASE-NOTES-1.0-BETA.md`).
- Publication prep: `luca-ui` VS Code extension (`.lucu` highlighting,
  snippets, run command; 10 unit tests; packaged `.vsix`), CI workflow,
  tag-driven release workflow (GitHub release + optional Marketplace
  publish via `VSCE_PAT`), repository initialized for first push.
- Docs: `docs/CHANGELOG.md`, `docs/RELEASE-NOTES-1.0-BETA.md`,
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
