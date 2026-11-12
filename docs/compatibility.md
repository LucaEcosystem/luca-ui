# Compatibility notes — Luca UI 1.0.2 Beta

## Generation and versions

Luca Code and Luca UI share one compatibility generation (the locked
compatibility decisions). The verified pair for this release:

| Product | Version | Status |
|---|---|---|
| Luca Code | 1.0.3 Beta (`PRODUCT_VERSION`, `Cargo.toml` 1.0.3) | Published host language; 179-test suite green |
| Luca UI | 1.0.2 Beta (`PRODUCT_VERSION`, `Cargo.toml` 1.0.2) | Publication Beta; **do not label stable** |

Minor releases within the generation stay compatible. A new generation
starts the SPEC migration timetable; historic releases are archived, and
installed copies keep running (no remote revocation).

## Source boundary (`.lucc` vs `.lucu`)

- Luca Code owns general-purpose logic in `.lucc`; Luca UI owns declarative
  descriptions in `.lucu`. Extensions are enforced by both CLIs and never
  change meaning across the boundary.
- The dependency is one-way: `luca-ui` depends on `luca-code` by relative
  path and reuses only its public library (`run`/`run_with_input`,
  `LucaError`, `types::{Value, Type, Dec}`). Luca Code never depends on
  Luca UI; its suite, behavior, and user files are untouched by UI work.
- Paired `.lucc` logic runs on the real interpreter through the seam;
  results re-enter UI state only via `State::set`/`flush`. There is no
  automatic `.lucc`→state protocol in 1.0.2 Beta.

## Backend support matrix

| Target | Status in 1.0.2 Beta |
|---|---|
| Headless text backend | Implemented (`TextBackend`); all examples verified through it |
| Static SVG snapshot | Implemented (`SvgBackend`); deterministic vector primitives, non-interactive |
| Static layout/paint/hit testing | Implemented; documented ◐ assumption subset |
| Native / custom-GPU / web | Architectural options only — NOT implemented, NOT claimed |
| Interactive event/pointer routing | Not implemented; only declarative dispatch and static hit testing |

The core vocabulary (App/Text/Button) carries no platform concepts, so no
core definition blocks any future backend. Platform-specific APIs, when
they arrive, live outside the core vocabulary per SPEC.

## Diagnostics stability

Structural toolchain failures are `Error at line:column: message` with
frozen wording (`docs/diagnostics.md`, enforced by `tests/diagnostics.rs`).
Any wording change is a breaking change and needs a version note. The visual
pipeline adds non-fatal `Render warning at line:column: message` lines for
recognized-but-invalid layout/style entries; they never fail a render and
unknown entries produce no warning.

## What 1.0 stable still needs

Validation and stabilization of this Beta only: defect fixes plus the design
record for the explicitly open items (full component inventory, complete
layout/style semantics beyond the documented subset, timing, theme tokens,
resource formats, interactive/native adapters; routes are locked out). No
new feature milestones are planned before 1.0 stable.
