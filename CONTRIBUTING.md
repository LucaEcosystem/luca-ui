# Contributing to Luca UI

Thanks for taking the time to help improve Luca UI. This document explains
how to report problems, propose ideas, and work on the codebase locally.

## License and contribution model

Luca UI is **proprietary software**. The [LICENSE](LICENSE) grants you the
right to use it and to write your own interfaces with it, but it **prohibits
modifying the software or redistributing it** without prior written
permission from Luca Ecosystem.

Because of that, this project **does not accept external code
contributions**:

- We do **not** accept pull requests from outside the maintainers.
- We **do** welcome bug reports, feature requests, questions, and
  discussions — see below.
- Forking for personal experimentation is not permitted by the license; do
  not redistribute or republish the code.

## Reporting bugs

Open an issue with the **Bug report** template. A good report includes:

- Your **Luca UI version** (`luca-ui --version`) and **Luca Code version**
  (`luca` or `luca-code --version`).
- The **operating system** and **Rust version** (`rustc --version`).
- The **full text of the error** — `Error at line:column: message` wordings
  are frozen, so paste them exactly rather than paraphrasing.
- The **smallest `.lucu` file** that reproduces the problem, plus any paired
  `.lucc` file if the problem involves logic.
- What you expected to happen and what actually happened.
- Whether the problem also appears in `luca-ui check` / `luca-ui render`.

Never include secrets, private data, or proprietary application code in a
public issue. Describe the failure and attach only the minimal synthetic
snippet needed to reproduce it.

## Requesting features

Open an issue with the **Feature request** template. Explain the user problem
first, then the proposed behavior, and why it fits the declarative model
(state-derived output, no imperative component operations, source-located
diagnostics). Feature status is marked throughout the docs: ● locked, ◐
assumption, ○ not implemented.

## Asking questions / discussing

Use **GitHub Discussions** for questions, usage help, and design
conversation. For issues that belong to the language itself (`.lucc` syntax,
the interpreter, type system), please raise them against
[Luca Code](https://github.com/LucaEcosystem/luca-code) instead — Luca UI
reuses Luca Code and must not maintain a second implementation.

## Working on the code locally

You may read, build, and test the source to understand it or to prepare a
report. You may not publish or redistribute modifications.

**Requirements:** Rust (stable) and a
[Luca Code](https://github.com/LucaEcosystem/luca-code) checkout as a
**sibling directory** named `Luca Code` (the `luca-ui` package depends on
`luca-code` by relative path).

```sh
cargo test                                          # full Rust suite
cargo fmt --check                                   # formatting
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps
cargo run -- examples/hello.lucu                    # parse/print a view
cargo run -- check examples/events.lucu             # validate + handler check
cargo run -- render examples/visual.lucu --out visual.svg
cargo run --example render                          # text backend
cargo run --example visual                          # static SVG backend
(cd editors/vscode && npm test)                     # extension unit tests
```

The VS Code extension tests are in `editors/vscode` and run with `npm test`.

### Conventions that matter

- **One-way dependency.** `luca-ui` depends on `luca-code`; Luca Code must
  never depend on Luca UI. Keep the seam one-way and reuse only Luca Code's
  public library.
- **No new dependencies without a stated reason.** The SVG backend and the
  layout engine are deliberately dependency-free.
- **Preserve frozen diagnostics.** Structural error wordings are cataloged in
  `docs/diagnostics.md` and asserted byte-for-byte by
  `tests/diagnostics.rs`. Only add diagnostics for new features; never edit
  existing wording.
- **Label assumptions.** Syntax and behavior that are not locked must be
  marked ◐ in the guides, with the open part stated explicitly.
- **Tests verify meaningful behavior.** Add tests for new behavior; keep them
  deterministic and I/O-free where practical.
- **Keep the license and docs in sync.** If behavior changes, update the
  relevant guide and the changelog entries under `Unreleased`.

## Security

See [SECURITY.md](SECURITY.md) for how to report vulnerabilities. Do **not**
open a public issue for a security problem.
