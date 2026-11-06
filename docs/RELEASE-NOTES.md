# Luca UI 1.0.2 Beta — release notes

> Part of the **Luca UI 1.0 Beta** product line; the package version for this build is **1.0.2**.

## What this release is

The documentation and labeling release of the declarative UI layer for Luca Code:

- `.lucu` declarations, Luca Code `.lucc` logic, one-way integration seam.
- Batched state with bindings; events routed to handler names.
- Text-backend rendering target; deterministic `TestHarness`; frozen diagnostics.
- A complete user-facing guide set (getting started → language → components → state/events → testing → rendering → styling → examples → CLI), each snippet verified against this build.

**No `.lucu`/`.lucc` language behavior changed since the 1.0.0 initial publication** — this release refreshes documentation, examples, packaging labels, and the extension. The frozen diagnostics catalog is unchanged.

## Verified pair

| Product | Version | Checks |
|---|---|---|
| Luca UI | 1.0.2 Beta (package 1.0.2) | Full Rust suite green; Clippy `-D warnings` clean; all examples + both cargo examples verified; extension 10/10 unit tests |
| Luca Code | 1.0 Beta (package 1.0.0) | 39 tests passed; untouched by this release (zero UI edits in its tree) |

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

## VS Code extension (1.0.2)

`editors/vscode/` builds `luca-ui-1.0.2.vsix`: `.lucu` syntax highlighting, snippets, a starter-file command, and one-key run through the CLI (with `Error at line:column:` surfaced as editor diagnostics). Install the local build:

```sh
code --install-extension editors/vscode/luca-ui-1.0.2.vsix
```

The extension's own README covers commands, settings, and troubleshooting. A Marketplace listing is pending publisher access.

## Artifacts

Reproduce a release build from a checkout:

```sh
cargo test --offline                                               # full suite
cargo clippy --all-targets --all-features -- -D warnings           # clean
cargo build --release --offline                                    # the luca-ui binary
(cd editors/vscode && npx @vscode/vsce package)                    # luca-ui-1.0.2.vsix
```

## Known limits of this Beta

- Text backend only; native / custom-GPU / web are architectural options, not implemented.
- The extension ships in-repo (and as a locally built `.vsix`) but is not yet on the Marketplace (needs a `VSCE_PAT` publisher secret).
- Assumption syntax throughout — `state:`/`events:`/`layout:`/`style:`/`transitions:`/`theme:` blocks and their entry names work exactly as documented and are flagged `◐ Assumption` in the guides; the full component inventory, layout/style semantics, transition timing, and theme tokens await the design record.
- Not published to a crate registry (`luca-code` is not published; the package license is proprietary) or any marketplace.
- No install scripts ship with this Beta.

## What changed in 1.0.2

See [CHANGELOG.md](CHANGELOG.md). Summary: docs overhaul, version labels → 1.0.2 Beta, extension 1.0.2 + rebuilt `.vsix`, example headers refreshed, generation-level compatibility test.

## Next

1.0 stable follows after Beta validation: defect fixes and compatibility confirmation only.
