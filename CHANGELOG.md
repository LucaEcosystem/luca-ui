# Changelog — Luca UI

All notable changes to Luca UI, newest first. `PRODUCT_VERSION` is the
user-facing label; `Cargo.toml` carries the numeric package version. The
full per-release detail lives in [`docs/CHANGELOG.md`](docs/CHANGELOG.md);
this file is the short history at the repository root. Status markers are
explained in [`docs/README.md`](docs/README.md): ● locked, ◐ assumption,
○ not implemented.

## [1.0.2] — 1.0.2 Beta

The 1.0.2 Beta drop: documentation overhaul, the first static SVG visual
rendering (layout/paint/hit testing), a stronger Luca Code seam with
declaration-only handler checks, per-key state subscriptions, and
`check`/`render` CLI commands. Package version `1.0.2`;
`PRODUCT_VERSION = "1.0.2 Beta"`. No frozen structural diagnostics wording
changed; new non-fatal render warnings cover the visual assumption subset.

## [1.0.0] — 1.0 Beta

Initial publication Beta — not stable 1.0. Release artifacts, CI, the VS Code
extension, and publication documentation.

## [0.9] — Alpha (Beta candidate, unpublished)

Minimal core library (App/Text/Button), deterministic `TestHarness`, frozen
diagnostics catalog, development-only inspector.

## [0.8] — Alpha

Declarative `transitions:` data blocks, first-class `Theme` + `Environment`,
built-in `Resources` exact-locale lookup.

## [0.7] — Alpha

`Renderer` contract and the headless `TextBackend`.

## [0.6] — Alpha

Component-id `Navigator` stack and the core `Description` accessibility model.

## [0.5] — Alpha

`events:` routing with declarative dispatch; `Runtime` lifecycle.

## [0.4] — Alpha

`Value`-typed `State`, `state:` blocks, bare-identifier bindings, batched
`set`/`flush`.

## [0.3] — Alpha

Platform-independent `layout:`/`style:` declaration blocks (structural).

## [0.2] — Alpha

Component declarations, `id` identity, locked prop rules, tree structure.

## [0.1] — Alpha

Bare-minimum foundation: crate layout, `.lucu` handling, `LucaError`
diagnostics, generic component tree, one-way Luca Code seam.
