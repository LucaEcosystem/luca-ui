# Luca UI 1.0 Beta — release notes (publication Beta, NOT stable 1.0)

## What this is

The publication Beta of the declarative UI layer for Luca Code 1.0 Beta:
`.lucu` descriptions, Luca Code `.lucc` logic, one-way integration seam,
text-backend rendering target. The full locked design from `docs/SPEC.md`
is implemented with running-test evidence
(`docs/implementation-status.md` §15); open items stay explicitly open.

## Verified pair

| Product | Version | Checks |
|---|---|---|
| Luca UI | 1.0 Beta (package 1.0.0) | 53 tests passed; Clippy `-D warnings` clean; all 8 `.lucu` examples + both cargo examples verified |
| Luca Code | 1.0 Beta (package 1.0.0) | 39 tests passed; untouched by this release (zero UI edits in its tree) |

## Install (verified method)

Requires Rust (stable) and the Luca Code checkout as a sibling directory
(the `luca-code` dependency is a relative path dependency):

```sh
cargo install --path /path/to/Luca-UI --root ~/.luca-ui-1.0-beta
export PATH="$HOME/.luca-ui-1.0-beta/bin:$PATH"
luca-ui --version   # luca-ui 1.0 Beta
```

Then run a UI file: `luca-ui hello.lucu`. Debug builds additionally offer
`luca-ui inspect <file.lucu>` (development-only; refused by release builds).

## Package contents (`dist/`)

- `luca-ui-1.0-beta-<target>.tar.gz` — ready-to-run release binary for the
  stated target, plus `README.md`, release notes, and examples.
- `luca-ui-1.0.0-source.tar.gz` — reproducible source package: both trees
  (`Packages/Luca UI` + `Packages/Luca Code`, no `target/`), fully
  offline-capable (zero registry dependencies; verified with empty
  `CARGO_HOME`: `cargo test --offline` → 53 passed).
- `SHA256SUMS` — SHA-256 checksums for every artifact above.

Reproduce: unpack the source package, run `cargo test` (53 passed),
`cargo clippy --all-targets --all-features -- -D warnings` (clean), and
`cargo build --release --offline`.

## Known limits of this Beta

- Text backend only; native / custom-GPU / web are architectural options,
  not implemented.
- `.lucu` VS Code extension ships in-repo (`editors/vscode/`, packaged
  `luca-ui-1.0.0.vsix`, also attached to the GitHub release) but is not yet
  on the Marketplace (needs a `VSCE_PAT` secret); the Luca Code extension
  covers `.lucc` only. No install scripts ship with this Beta.
- Assumption syntax throughout (see implementation status §17 plus the
  Deferred open register); full component inventory, timing, theme tokens,
  resource formats, and adapters await the design record.
- Not published to crates.io (`luca-code` is not on the registry; package
  license is proprietary `UNLICENSED`) or any marketplace. See the
  publishing checklist in `docs/implementation-status.md` §18.

## Next

1.0 stable follows after Beta validation: defect fixes and compatibility
confirmation only.
