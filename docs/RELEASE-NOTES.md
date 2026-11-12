# Luca UI 1.0.2 Beta — release notes

> Part of the **Luca UI 1.0 Beta** product line; the package version for this
> build is **1.0.2**. This is the published 1.0.2 Beta release; the full
> per-release detail lives in [`CHANGELOG.md`](CHANGELOG.md).

## What this release is

The declarative UI layer for Luca Code, now with a real static visual path:

- `.lucu` declarations, Luca Code `.lucc` logic, one-way integration seam
  with declaration-only handler checking.
- Batched state with bindings (global and per-key subscriptions); events
  routed to handler names.
- Two concrete backends over one rendering contract: the headless **text
  backend** and a dependency-free, static **SVG backend** built from a layout
  engine, display list, and hit testing.
- Deterministic `TestHarness`; frozen error diagnostics plus non-fatal render
  warnings; a `check` CLI that validates the whole static picture and a
  `render` CLI that writes SVG snapshots.
- A complete user-facing guide set (getting started → language → components →
  state/events → testing → rendering → styling → examples → CLI), each snippet
  verified against this build.

The frozen structural diagnostics catalog is unchanged. New **render
warnings** cover the documented layout/style assumption subset and never fail
a render.

## Verified pair

| Product | Version | Checks |
|---|---|---|
| Luca UI | 1.0.2 Beta (package 1.0.2) | 93 Rust tests green; Clippy `-D warnings` clean; all examples + all cargo examples verified; extension 10/10 unit tests |
| Luca Code | 1.0.3 Beta (package 1.0.3) | 179 tests passed; untouched by this release (zero UI edits in its tree) |

Luca Code and Luca UI stay one compatibility generation: Luca UI depends on
Luca Code by relative path and reuses only its public library; Luca Code never
depends on Luca UI.

## Install

Requires Rust (stable) and the Luca Code checkout as a sibling directory
(the `luca-code` dependency is a relative path dependency):

```sh
cargo install --path /path/to/Luca-UI --root ~/.luca-ui
export PATH="$HOME/.luca-ui/bin:$PATH"
luca-ui --version   # luca-ui 1.0.2 Beta
```

Then run a UI file: `luca-ui hello.lucu`. Debug builds additionally offer
`luca-ui inspect <file.lucu>` (development-only; refused by release builds).

Full walkthrough: [docs/getting-started.md](getting-started.md).

## Rendering

```sh
luca-ui render examples/visual.lucu --out visual.svg --width 320 --height 200
cargo run --example visual    # same SVG on standard output
```

The SVG snapshot is deterministic and self-contained (`<svg` with `xmlns`,
`width`, `height`, `viewBox`, escaped `<rect>`/`<text>` primitives). It is
**static**: no interaction, animation, fonts, images, clipping, or
accessibility tree, and it is not a browser/rendering-engine claim.

`luca-ui check <file.lucu>` builds the same visual and prints render warnings
without writing a file. Recognized-but-invalid layout/style values warn and
fall back to defaults; unknown entry names stay opaque and produce no warning.

Details: [docs/rendering.md](rendering.md), [docs/cli.md](cli.md).

## VS Code extension (1.0.2)

`editors/vscode/` builds `luca-ui-1.0.2.vsix`: `.lucu` syntax highlighting,
snippets, a starter-file command, and one-key run through the CLI (with
`Error at line:column:` surfaced as editor diagnostics). Install the local
build:

```sh
code --install-extension editors/vscode/luca-ui-1.0.2.vsix
```

The extension's own README covers commands, settings, and troubleshooting. The
Marketplace listing (`luca-ecosystem.luca-ui`) exists; upload
`luca-ui-1.0.2.vsix` to update it to 1.0.2 Beta.

## Artifacts

Reproduce a release build from a checkout:

```sh
cargo test --offline                                               # full suite
cargo clippy --all-targets --all-features -- -D warnings           # clean
cargo build --release --offline                                    # the luca-ui binary
(cd editors/vscode && npx @vscode/vsce package)                    # luca-ui-1.0.2.vsix
```

## Known limits of this Beta

- Two backends ship: headless text and **static** SVG. Native / custom-GPU /
  web remain architectural options, not implemented; interactive
  pointer/keyboard routing is not implemented (only static hit testing).
- Visual semantics are a documented **◐ assumption subset** — `direction`,
  `gap`, `padding`, `width`, `height`, `align` and `background`, `color`,
  `border`, `radius`, `font_size` (`#rrggbb`). No wrapping, clipping,
  scrolling, flex/grid algorithms, inheritance, named colors, or font shaping.
- Text metrics are approximate (`font_size × 0.6` per character,
  `font_size × 1.2` per line); not typographic.
- Automatic `.lucu`→`.lucc` event execution is not implemented; the host
  application performs dispatch → run logic → state update → re-render. Handler
  checking is declaration-only and reports imports as unverified.
- The extension ships in-repo, on GitHub Releases, and on the Visual Studio
  Marketplace (`luca-ecosystem.luca-ui`). The 1.0.2 Beta `.vsix` is attached to
  this release and can be uploaded to the Marketplace to update the listing.
- The full component inventory, complete layout/style semantics, transition
  timing, theme tokens, and resource file formats await the design record.
- Not published to a crate registry (`luca-code` is not published; the
  package license is proprietary). GitHub Releases and the Visual Studio
  Marketplace are the distribution channels. No install scripts ship with this
  Beta.

## What changed in 1.0.2

See [CHANGELOG.md](CHANGELOG.md). Summary: foundation fixes, a stronger Luca Code
seam (logic loading, paired paths, declaration-only handler checks, per-key
subscriptions), the static SVG visual pipeline, `check`/`render` CLI commands,
new tests and examples, and refreshed documentation.

## Next

1.0 stable follows after Beta validation: defect fixes and compatibility
confirmation only.
