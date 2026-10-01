# Implementation status — Luca UI 1.0 Beta (publication Beta, NOT stable 1.0)

Current release: **1.0 Beta**. Validation/stabilization of the 0.9 candidate; no locked behavior changed.

## 1. Baseline audit (2026-09-30, before changes)

- `Luca UI` is not a git repository (`git status` → `fatal: not a git repository`).
  Baseline file tree: `compiler/` (9 scaffold dirs, `.gitkeep` only), `runtime/`
  (4 dirs), `std/` (4 dirs), `docs/{SPEC,ARCHITECTURE}.md`, `examples/` +
  `tests/` (`.gitkeep` only), `logo/`. No existing edits to preserve; no
  deletions made by this milestone.
- `Luca Code` (`../Luca Code`, branch `main`, `3870808 Luca Code 1.0 Beta`):
  `cargo test` 39 passed, Clippy clean (re-verified). Pre-existing working-tree
  changes preserved untouched — `M .gitignore, README.md,
  docs/implementation-status.md, src/main.rs` plus untracked distribution work
  (`LICENSE`, `Icon`, `docs/{first-program,install,troubleshooting,vscode}.md`,
  `editors/vscode/*`, `scripts/install.*`, `.github/workflows/ci.yml`). This
  milestone makes zero edits inside `../Luca Code`.

## 2. Luca Code public surface (inspected)

`src/lib.rs:1` exposes `run`, `run_with_input`, `PRODUCT_VERSION = "1.0 Beta"`,
plus public modules `ast, error, interpreter, lexer, parser, stdlib, types`.
`interpreter::Interpreter:105` offers `new`, `with_search_paths`,
`with_input`, `with_cli_args`, `run`, `run_collect:287`. `error::LucaError:4`
is `{message, line, column, exit_code, error_value}` with `Display` as
`Error at line:column: message`. `lexer::lex:17` and `parser::parse:8` are
public; `types` exposes `Type/Dec/Value/CollectionKind`. CLI (`src/main.rs:1`)
enforces `.lucc` and passes program-dir search paths + full argv.

## 3. Reconciliation vs `docs/SPEC.md` (authority: SPEC + Luca Code reference)

SPEC locks: declarative state-driven UI on Luca Code (`.lucu` vs `.lucc`);
no second general-purpose compiler/interpreter; no component refs or
`hide()/show()/set_text()`; `id` on every component; `visible`/`disabled` as
ordinary props; batched state; async cancel on disappear; component-based
navigation (no string routes); core accessibility (labels, inference,
overrides); core animation/transitions, localization, themes, auto system
appearance; Luca errors for invalid props/args; UI testing API + IDs; dev
inspector. The complete design conversation was not available in this
session, so SPEC §"Open specification material" + §"Known locked decisions"
(90, 95–125) remain the authority; nothing beyond them is claimed as locked.

Still missing from local docs (do NOT implement until the full design record
or an explicit decision supplies them): exact `.lucu` grammar; component
inventory, signatures, and property types; state declaration and binding
syntax; layout constraints and style property names; transition syntax and
timing defaults; theme token names; localization resource format; renderer
backends and platform adaptation specifics; UI testing API shape; inspector
protocol. The 0.9 syntax is a documented minimal assumption, not design.

## 4. Integration seam (smallest viable, one-way)

`luca-ui` depends on `luca-code` by relative path (`../Luca Code`); Luca Code
never depends on Luca UI; no compiler/interpreter code is copied; `.lucc`
behavior, extension, and user files are untouched. The seam (`src/seam.rs:1`)
reuses only the published library: `luca_code::run` / `run_with_input` for
paired `.lucc` logic and `luca_code::error::LucaError` for every diagnostic.
`.lucu` extension enforcement mirrors the `.lucc` CLI rule; `load_file`
rejects non-`.lucu` paths so `.lucc` work is never overwritten. `run_paired`
parses the UI tree then runs the logic and returns `(Component, output)`.

## 5. Milestone map 0.1 Alpha → 1.0 Beta

| Release | Depends on | Scope (coherent slice) | Acceptance |
|---|---|---|---|
| 0.1 Alpha (done) | Luca Code 1.0 Beta (read-only) | Crate layout, `.lucu` handling, `LucaError` diagnostics, generic `Name:`/`key = value` tree, locked prop rules (`id`/`visible`/`disabled`/no-imperative), seam, one example, focused tests | Example accepted; invalid input → actionable `Error at line:col`; Luca Code suite still green; Clippy clean |
| 0.2 Alpha (done) | 0.1 | Component declarations, identity, property validation, tree structure — SPEC-locked rules only, no invented inventory | Valid trees accepted; unknown/invalid props, duplicate/invalid ids → Luca errors with source locations; Luca Code suite still green; Clippy clean |
| 0.3 Alpha (done) | 0.2 | Layout + styling: platform-independent `layout:`/`style:` declaration blocks, structural validation, no backend concepts, no invented names/dimensions/defaults/precedence | Valid layouts accepted; invalid values → Luca errors with source locations; Luca Code suite still green; Clippy clean |
| 0.4 Alpha (done) | 0.3 | State + bindings: `Value`-typed store with Luca Code semantics, `state:` literal blocks, bare-identifier bindings resolved to UI output, batched set/flush, prune-on-remove listeners | Initials/updates/propagation/batching/type errors/removal tested; Luca Code suite still green; Clippy clean |
| 0.5 Alpha (done) | 0.4 | Events + lifecycle: `events:` routing blocks, declarative dispatch to handler names, runtime tree swaps with mount/unmount diffs, scoped-task cancellation on disappear | Dispatch/state-update/cleanup/error tests; Luca Code suite still green; Clippy clean |
| 0.6 Alpha (done) | 0.5 | Navigation + accessibility: component-id `Navigator` stack (no routes), core `Description` model (explicit labels win, `text` infers, bindings participate; platform mapping deferred) | Transition/inference/override/missing-data tests; Luca Code suite still green; Clippy clean |
| 0.7 Alpha (done) | 0.6 | Rendering abstraction + text backend: `Renderer` contract (resolved root, depth-first visible visits, hidden subtrees skipped, model-only reads, deterministic); `TextBackend` structural lines; native/GPU/web NOT implemented | Contract tests on two backends + runnable example; Luca Code suite still green; Clippy clean |
| 0.8 Alpha (done) | 0.6–0.7 | Declarative transitions, first-class themes, system appearance, localization foundations: `transitions:` data blocks with reduced-motion gating; `Theme` name/mode with system-follow default; `Resources` exact-locale lookup; no timing defaults, token names, or resource formats | Transition/theme/lookup tests + examples; Luca Code suite still green; Clippy clean |
| 0.9 Alpha (done) | 0.6–0.8 | Beta candidate: minimal core library (App/Text/Button) with per-component validation, dedicated `TestHarness` API, frozen diagnostics catalog, dev-only inspector; full section-by-section reconciliation with evidence | 0.9 checklist green incl. reconciliation matrix; Luca Code suite still green; Clippy clean; NOT published |
| 1.0 Beta (this) | 0.9 | Publication Beta: 0.9 matrix re-verified; release blockers fixed without behavior change; version/changelog/release notes; reproducible vendored source package + binary + checksums; install + clean-consumer verification; NOT published (no channel/credentials) | 1.0 Beta checklist green; Luca Code suite still green; Clippy clean; NOT stable 1.0 |

Map note: the 0.3 prompt ordered layout/styling ahead of state/bindings, so
those rows shifted down one release from the 0.2-era map; the 0.6 prompt
ordered navigation and accessibility together, so the old 0.6/0.7 rows merged
into one 0.6 slice; the 0.7 prompt ordered the rendering abstraction with
its first backend next, so a 0.7 row slots between 0.6 and 0.8. No scope
was added or dropped.

No milestone advances its label until its row passes. Luca Build remains out
of scope until Code + UI foundations land (per SPEC).

## 6. 0.1 Alpha checklist (done)

- [x] `Cargo.toml` + `src/{lib,component,parser,seam,main}.rs` single-crate layout
- [x] `.lucu` handling (`seam::load_file`, `--help/--version`, `0.1 Alpha` label)
- [x] Source-located diagnostics via `LucaError` (`Error at line:column`)
- [x] Generic component tree: parse, locked-rule validation, `find_by_id`
- [x] Seam: `run_logic`, `run_paired`, no Code edits/copies/`.lucc` changes
- [x] `examples/hello.{lucu,lucc}` end-to-end pair (assumption syntax, labeled)
- [x] `tests/foundation.rs` 6 tests: example, validation, errors, seam
- [x] `cargo test` + Clippy clean; Luca Code 39-test suite still green

## 7. 0.2 Alpha checklist (done)

- [x] Declarations: `Name:` shape, single-root tree, props belong to a
  component, malformed declarations are compile-time Luca errors
- [x] Identity: `id` on every component, non-empty string, tree-unique with
  first-use location in the message; `Component::ids()` for test identifiers
- [x] Validation: `visible`/`disabled` bool, `label` str, duplicate props,
  imperative keys/lines/values rejected; bare identifier values stay opaque
  (no component-reference mechanism); `text`-style generic props accepted
  pending the locked per-component inventory
- [x] Tree structure: nesting, siblings, order preserved; `count`/`depth`/
  `find_by_id` helpers
- [x] `examples/declarations.lucu` valid-tree example (5 nodes, depth 3)
- [x] `tests/declarations.rs` 5 tests: valid trees, invalid declarations,
  invalid properties, identifiers, precise locations
- [x] Label at `0.2 Alpha` (`Cargo.toml` 0.2.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 8. 0.3 Alpha checklist (done)

- [x] Reconciled against the full decision record: no local design record
  exists beyond SPEC/ARCHITECTURE/this file, so entry names, dimensions,
  defaults, and precedence stay undefined (not implemented, not claimed)
- [x] `src/layout.rs` + `src/style.rs`: platform-independent,
  backend-agnostic `Layout`/`Style` entry containers; structural validation
  only (identifier keys, scalar values, no duplicates, no imperative ops)
- [x] `Component.layout` / `Component.style` leaf attachments; extraction
  before locked-rule validation so block entries never pollute identity;
  `count`/`depth`/`ids`/`find_by_id` unaffected
- [x] Parser: reserved `layout:`/`style:` child blocks (entries only, at most
  one of each per component, must belong to a component); every violation a
  source-located Luca error
- [x] Public model holds no renderer/platform/unit/theme/precedence concepts
- [x] `examples/layout.lucu` representative example (blocks, empty block,
  nesting); CLI prints layout/style sections
- [x] `tests/layout.rs` 5 tests: valid layouts, invalid values, misplaced/
  duplicate blocks, identity isolation, precise locations
- [x] Label at `0.3 Alpha` (`Cargo.toml` 0.3.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 9. 0.4 Alpha checklist (done)

- [x] Reconciled: state/binding syntax not locked locally, so `state:`
  literal blocks, bare-identifier bindings, and batch/removal rules are a
  labeled minimal assumption, not design
- [x] `src/state.rs`: `State` store over `luca_code::types::Value`/`Type`
  (exact `Dec`, `none`-occupies-any-type, `uni` inference); `declare`/`get`/
  `set`/`flush`; no interpreter copied, no expressions evaluated
- [x] Batching: `set` type-checks immediately (fail fast at declaration site)
  and stages; `flush` commits at once with one notification per listener per
  changed key (repeated sets coalesce to the final value)
- [x] `src/bindings.rs`: `resolve_tree` derives UI output from committed
  state; undeclared names and wrongly-typed locked props are Luca errors at
  the binding site; locked `id` uniqueness enforced on resolved values
- [x] Removal: `subscribe`/`prune` drops listeners whose component id left
  the tree so their pending notifications never emit; no component-mutation
  API exists (`set`/`flush` is the only change path)
- [x] Parse-time literal checks for locked props kept; `Ref` on locked keys
  now parses and enforces at resolution (0.2 tests updated for the designed
  evolution, no locked rule weakened)
- [x] `examples/state.lucu` state+binding example; CLI prints `state:` blocks
- [x] `tests/state.rs` 5 tests: initials, propagation, batching, type errors,
  removal
- [x] Label at `0.4 Alpha` (`Cargo.toml` 0.4.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 10. 0.5 Alpha checklist (done)

- [x] Reconciled: event/lifecycle syntax not locked locally, so `events:`
  entries, routing-only dispatch, and the task-cancellation registry are a
  labeled minimal assumption, not design
- [x] `src/events.rs`: `EventBlock` (`event = handler` names only; literals
  rejected); `dispatch` validates target id + declared event and returns the
  handler name — never a component reference, never touches state
- [x] `src/runtime.rs`: `Runtime` owns tree + state; `update_tree` reports
  mounted/unmounted diffs, merges new state (committed values persist),
  prunes listeners, and cancels tasks scoped to removed components; scoping
  to unmounted components is a Luca error
- [x] `.lucc` logic runs on the existing seam only (`run_logic`); changes
  feed back solely through `State::set`/`flush`; no automatic `.lucc`→state
  protocol invented (awaits the dispatch design record)
- [x] `Component.events` leaf attachment; extraction + CLI printing; `count`/
  `depth`/`ids`/`find_by_id` unaffected
- [x] `examples/events.lucu` + `examples/events.lucc` pair (verified output
  `visits = 1/2/0` on the real interpreter)
- [x] `tests/events.rs` 4 tests: declaration + routing, dispatch-driven
  state updates with seam agreement, lifecycle diffs + task cancellation +
  state persistence, error reporting with locations
- [x] Label at `0.5 Alpha` (`Cargo.toml` 0.5.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 11. 0.6 Alpha checklist (done)

- [x] Reconciled: navigation API shape and accessibility vocabulary beyond
  `label` are not locked locally, so the id-stack and label/inference model
  are a labeled minimal assumption, not design
- [x] `src/navigation.rs`: `Navigator` stack of component ids over one tree
  (push/pop/replace/current); targets validated against declared components;
  root screen always present; no routes, route tables, or path syntax; ids
  only, never component references
- [x] `src/accessibility.rs`: core `Description` model with zero platform
  concepts — explicit `label` wins, string `text` infers, bound values
  participate via `describe_resolved`; missing labels are valid (`None`);
  non-string labels stay Luca errors at parse (literals) and resolution
  (bindings) with source locations
- [x] `examples/navigation.lucu` two-screen example with labels, texts,
  and routed events
- [x] `tests/navigation.rs` 5 tests: transitions, navigation errors,
  explicit/inferred/missing labels, bound-label participation + updates,
  missing/invalid data with locations
- [x] Label at `0.6 Alpha` (`Cargo.toml` 0.6.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 12. 0.7 Alpha checklist (done)

- [x] Reconciled: renderer-specific behavior is explicitly open in SPEC, so
  the contract points and the text line format are a labeled minimal
  assumption; native, custom/GPU, and web stay unimplemented options
- [x] `src/rendering.rs`: `Renderer` trait (name + render over resolved
  output), `is_visible`/`format_value` contract helpers, `TextBackend`
  (structural lines; interprets no component names; exact decimal scale)
- [x] Core models preserved untouched: component, layout, styling, state,
  and accessibility modules unchanged; renderer consumes them read-only
- [x] No platform-specific components added anywhere; the public model
  names no platform, renderer, or unit concepts
- [x] `examples/render.lucu` + `examples/render.rs` runnable via
  `cargo run --example render`
- [x] `tests/rendering.rs` 4 tests: exact backend output, two-backend
  contract parity, state-derived re-rendering, visibility default + formats
- [x] Label at `0.7 Alpha` (`Cargo.toml` 0.7.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 13. 0.8 Alpha checklist (done)

- [x] Reconciled: transition syntax, timing defaults, theme token names, and
  resource formats are explicitly open in SPEC, so all four stay undefined
  (not implemented, not claimed); foundations carry locked behavior only
- [x] `src/animation.rs`: `TransitionSet` declarative data blocks
  (structural validation only, no timing/interpolation); `active` gates on
  platform reduced-motion, preserving the core accessibility model
- [x] `src/themes.rs`: first-class `Theme` (optional name, system-follow
  default mode, opaque tokens); `ThemeMode::effective` pins or follows;
  `Environment` carries explicit platform signals, no invented defaults
- [x] `src/localization.rs`: built-in `Resources` tables with exact-locale
  lookup; missing keys/locales return `None`; no fallback chains, no file
  formats, no `.lucu` resource syntax
- [x] `Component.transitions` / `Component.theme` leaf attachments with the
  shared extraction rules; `count`/`depth`/`ids`/`find_by_id` unaffected;
  CLI prints both sections
- [x] `examples/themes.lucu` (transitions + named/pinned themes) and
  `examples/localize.rs` runnable via `cargo run --example localize`
- [x] `tests/animation.rs` 5 tests: transition data, reduced-motion gating,
  invalid declarations, theme selection/system follow, resource lookup
- [x] Label at `0.8 Alpha` (`Cargo.toml` 0.8.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 14. 0.9 Alpha checklist (done — candidate, was not published)

- [x] Audit re-run: baselines recorded, pre-existing edits in both repos
  preserved, Luca Code untouched (39 green), design record reconciled
  (complete conversation still unavailable; SPEC remains authority)
- [x] `src/library.rs`: minimal core (App/Text/Button) sharing the locked
  API, cross-platform by construction, per-component prop enforcement on
  registered names, open vocabulary elsewhere; `tests/library.rs` 3 tests
  incl. platform-concept scan and backend adaptation evidence
- [x] `src/testing.rs`: deterministic `TestHarness` (dispatch, set/flush,
  tree swaps, resolved reads, render) addressing components by `id` only;
  `tests/testing.rs` 3 tests
- [x] `docs/diagnostics.md` frozen catalog; `tests/diagnostics.rs` 5 tests
  asserting every reachable message byte-for-byte
- [x] `src/inspector.rs`: read-only snapshots (outline, committed state,
  layout declarations; bounds explicitly deferred); `luca-ui inspect`
  gated to debug builds; `tests/inspector.rs` 2 tests
- [x] Full reconciliation matrix (§16): every SPEC section and numbered
  decision mapped to code + test evidence or explicit open status
- [x] User guides (`README.md`) and `docs/compatibility.md` written
- [x] `SPEC.md` reconciliation pointer appended (no new decisions added)
- [x] Label at `0.9 Alpha` (`Cargo.toml` 0.9.0 + `PRODUCT_VERSION`)
- [x] `cargo test` + Clippy clean; Luca Code suite still green

## 15. 1.0 Beta checklist (this milestone — publication Beta, NOT stable 1.0)

- [x] 0.9 acceptance matrix re-confirmed: 53 tests passed, Clippy
  `-D warnings` clean, all examples + both cargo examples verified
- [x] Release channels audited: no crates.io/CI/Marketplace channel for
  Luca UI; Luca Code CI explicitly publishes nothing; VS Code extension
  covers `.lucc` only (no `.lucu` tooling — reported gap, Code repo untouched)
- [x] Release blockers fixed without behavior change: `luca-code` path dep
  gains its `version = "1.0.0"` requirement (packaging metadata only)
- [x] Version + changelog set to 1.0 Beta (`Cargo.toml` 1.0.0 following the
  Luca Code convention; `PRODUCT_VERSION = "1.0 Beta"`; `docs/CHANGELOG.md`);
  stable `1.0` label NOT applied anywhere
- [x] Release notes written (`docs/RELEASE-NOTES-1.0-BETA.md`): verified
  pair, install method, package contents, known limits, next step
- [x] Reproducible artifacts in `dist/`: offline-capable source package
  (both trees, zero registry dependencies) + release binary tarball +
  `SHA256SUMS` (see §17)
- [x] Install verified with clean `CARGO_HOME` (`cargo install --path`);
  clean consumer project (fresh `.lucu` + lib consumer) verified in /tmp
- [x] Luca Code generation compatibility re-verified: Code 1.0 Beta
  (package 1.0.0), 39-test suite green, zero UI edits in its tree, paired
  `.lucc` example output unchanged
- [ ] Publish (blocked: no release channel, no registry credentials, no UI
  repo/remote — exact checklist in §18; package is ready to run)

## 16. Locked-design reconciliation matrix (evidence maintained for 1.0 Beta)

“Locked” = stated in `docs/SPEC.md` §§Status–Compatibility or the numbered
table. “Open” = listed in §Open material or unspecifiable without the full
design conversation. Every locked item has running-test evidence; every
open item names what would close it. No open item is marked complete.

| SPEC item | Status | Evidence / closer |
|---|---|---|
| Declarative UI layer on Luca Code; `.lucu`/`.lucc` split; Code never depends on UI | Locked, done | `src/seam.rs:1`, one-way path dep in `Cargo.toml`; `tests/foundation.rs` seam tests |
| UI covers components, state, events, bindings, layout, styling, navigation, accessibility, animation, themes, localization | Locked, done | modules `component`+`library`, `state`, `events`, `bindings`, `layout`, `style`, `navigation`, `accessibility`, `animation`, `themes`, `localization` |
| Platform-independent API + rendering abstraction; native/GPU/web are options | Locked, done | `src/rendering.rs` contract + `TextBackend`; options explicitly unimplemented |
| No platform-specific components in core vocabulary | Locked, done | `tests/library.rs` platform-concept scan; backend presents all core structurally |
| Reuse Luca Code infra; no parallel compiler/interpreter | Locked, done | seam uses only `run`/`run_with_input`, `LucaError`, `types::{Value,Type,Dec}`; no expression evaluation in UI |
| Output derives from state + declared props | Locked, done | `resolve_tree`; `tests/state.rs` propagation, `tests/rendering.rs` re-render |
| No component references; no `hide()`/`show()`/`set_text()` | Locked, done | parser + validate + bindings rejections; `tests/declarations.rs`, `tests/events.rs` |
| `id` on every component; unique; test-suitable | Locked, done | parse + resolution uniqueness; `tests/declarations.rs`, `tests/diagnostics.rs` C2/R3 |
| `visible`/`disabled` ordinary props | Locked, done | bool enforcement parse + resolve; renderer skips `false` subtrees; `tests/rendering.rs` |
| Batched state updates | Locked, done | `set` stages / `flush` commits + coalesces; `tests/state.rs`, `tests/testing.rs` |
| Scoped async cancel on disappear (95) | Locked, done | task registry + `update_tree` cancellation + listener prune; `tests/events.rs` |
| Component-based navigation, no string routes (90) | Locked, done | `Navigator` id-stack, no route/table/path concepts; `tests/navigation.rs` |
| Component lifecycle managed by runtime | Locked, done | `Runtime::update_tree` diffs + merge + prune + cancel; `tests/events.rs` |
| Accessibility core; labels; inference; overrides (96–100); cross-platform core + adaptation | Locked, done | `describe`/`describe_resolved`; `tests/navigation.rs`; backend presents labels |
| Platform adapters map accessibility (96–100 part) | Open | needs adapter milestone + native targets; model carries zero platform concepts |
| Invalid props/args → Luca errors (106–115 part) | Locked, done | `docs/diagnostics.md` + `tests/diagnostics.rs` exact-message suite |
| Dedicated UI tests + IDs (106–115 part) | Locked, done | `src/testing.rs` `TestHarness`; `tests/testing.rs`; ids everywhere |
| Dev inspector: tree/state/layout bounds (106–115 part) | Locked, done (bounds scoped) | `src/inspector.rs` snapshots tree/state/layout *declarations*; CLI `inspect` debug-gated; `tests/inspector.rs`. Bounds need dimension semantics — open, explicitly deferred |
| `id` universal; no refs/manipulation; `visible`/`disabled` ordinary (116–125 part) | Locked, done | same evidence as above |
| Animation core; declarative transitions; localization; themes; auto system appearance (116–125 part) | Locked, done | `animation`/`themes`/`localization` modules + `tests/animation.rs` |
| Generation compat; preserve Code behavior/extensions; Build later | Locked, done | `docs/compatibility.md`; Code suite green, zero Code edits |
| Exact grammar, full inventory/signatures/prop types | Open | needs design record; 0.9 core (App/Text/Button) enforced, rest generic |
| State/binding syntax; layout constraints; style names | Open | assumption blocks; needs design record |
| Transition syntax/timing; theme tokens; l10n formats; renderer behavior | Open | foundations only; needs design record |
| Testing API shape beyond harness; inspector protocol beyond snapshots | Open | needs design record; current APIs documented as 0.9 surface |
| New Beta-scope decisions | None added | SPEC.md untouched except reconciliation pointer |

## 17. Verification (1.0 Beta)

```
cargo test                          — 53 passed, 0 failed (6 foundation + 5 declarations + 5 diagnostics + 4 events + 1 examples + 5 layout + 2 inspector + 3 library + 5 navigation + 4 rendering + 5 state + 3 testing + 5 animation)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo build --release               — clean; release binary refuses `inspect` (dev-only gate verified)
cargo run -- examples/hello.lucu    — prints App/Text tree with ids
cargo run -- examples/declarations.lucu — prints 5-node, depth-3 tree
cargo run -- examples/layout.lucu   — prints tree with layout/style sections
cargo run -- examples/state.lucu    — prints tree with state block + bindings
cargo run -- examples/events.lucu   — prints tree with events blocks
cargo run -- examples/navigation.lucu — prints two-screen tree
cargo run --example render          — renders examples/render.lucu via the text backend
cargo run -- examples/themes.lucu   — prints tree with transitions/theme sections
cargo run --example localize        — en/de lookups with an explicit missing case
cargo run -- inspect examples/layout.lucu — inspector snapshot (debug builds)
tests/examples.rs                  — all 8 shipped .lucu files render on the text target
cargo install --path . (clean CARGO_HOME) — installs `luca-ui 1.0 Beta`; fresh consumer project verified
cargo run --manifest-path "../Luca Code/Cargo.toml" -- examples/events.lucc — visits = 1/2/0
../Luca Code: cargo test            — 39 passed (untouched, still green)
dist/                              — vendored source tarball + binary tarball + SHA256SUMS (see §18)
```

Deferred with reason — explicit open register for 1.0 Beta (see also the
§16 matrix; nothing here is marked complete):
per-component unknown-property validation is enforced for the registered
0.9 core (App/Text/Button) and remains open for the full inventory, which
needs the component inventory from the full design record; unregistered
names reject only malformed properties and locked-rule violations. Layout/style
entry names, dimensions, defaults, and precedence likewise await the design
record; 0.3 validates block structure only and defines no rendering
semantics. State/binding syntax is likewise assumed: `state:` blocks hold
literals only (references would need an evaluation order), updates are
programmatic `set`/`flush` and no automatic `.lucc`→state update protocol
is defined, `int` never implicitly converts to `dec` on assignment, and
listener prune is the 0.4 mapping of cancellation-on-disappear (no async
runtime exists yet). Event routing is likewise assumed: dispatch returns
handler names without executing logic, and no automatic `.lucc`→state update
protocol is defined; scoped-task cancellation is the 0.5 mapping of
cancellation-on-disappear (cooperative flags, no executor yet). Navigation
targets declared component ids only (no routes by construction); the
accessibility vocabulary beyond `label`/`text` and platform-adapter mapping
await the design record. Rendering is likewise assumed: absent `visible`
means shown, and the text line format is backend-defined; native,
custom/GPU, and web backends are not implemented. Animation, theme tokens,
and localization resources are likewise assumed: transition entries carry
no timing semantics, theme `name`/`mode` words plus `system`/`light`/`dark`
are the minimal selection vocabulary (token entries stay opaque), and
resource tables are programmatic with exact-locale lookup only. Bare
identifier values (`Ref`) resolve against state at
`resolve_tree`; nothing resolves them to component instances, so no
component reference mechanism exists. Indentation levels are
lenient-but-deterministic (a deeper indent opens a new level; properties at
column 1 are rejected); exact layout grammar awaits the design record.

First-slice verdict: no missing locked decision blocks 0.1 — the generic
tree + assumption syntax covers the locked rules without inventing inventory.
The smallest 0.1 slice above is implemented; broader libraries/grammar are
intentionally deferred to 0.2+ pending the full design record.

## 18. Publishing checklist (1.0 Beta — NOT published)

No release channel is configured for Luca UI, so nothing was published.
When the project is ready, in order:

1. Create the `luca-ui` repository/remote and push this tree (today Luca UI
   is not a git repository; there is no remote, no CI, no release workflow).
2. Decide distribution: (a) crates.io requires publishing `luca-code`
   first (absent from the registry — verified 2026-09-30), choosing a real
   license (`UNLICENSED` blocks publishing), and registry credentials
   (`~/.cargo/credentials.toml` absent); (b) GitHub releases need a release
   workflow + tag + artifact upload (none configured; `gh` auth exists but
   no UI repo to attach to); (c) VS Code Marketplace needs a `.lucu`
   language extension (the Luca Code extension covers `.lucc` only).
3. Attach `dist/` artifacts (offline two-tree source tarball, binary
   tarball, `SHA256SUMS`) to the release; verify checksums on download.
4. Publish the Beta and announce with `docs/RELEASE-NOTES-1.0-BETA.md`.
5. Never label this build stable `1.0`: `PRODUCT_VERSION` reads `1.0 Beta`
   and the package stays a Beta until the 1.0 stabilization milestone.

Missing access/configuration reported 2026-09-30: no UI repo/remote/CI;
no crates.io credentials; `luca-code`+`luca-ui` absent from crates.io;
proprietary `UNLICENSED` license; no `.lucu` editor tooling.

## 19. Publication prep (1.0 Beta — local commit, no push, no publish)

- [x] Repository initialized (`git init -b main`); `/target`, `/dist`,
  `node_modules`, `.DS_Store` ignored; release tarballs stay on disk only
- [x] `editors/vscode/`: official `.lucu` extension mirroring the Luca Code
  extension conventions (manifest, TextMate grammar, language config,
  6 snippets, run/create commands, `lucaui.*` settings, proprietary
  LICENSE, README); `npm test` 10/10; packaged `luca-ui-1.0.0.vsix`
  (9 files) via `npx @vscode/vsce package`
- [x] `.github/workflows/ci.yml` (rust matrix + extension test/package) and
  `release.yml` (tag-triggered artifacts + GitHub release + conditional
  Marketplace publish on `VSCE_PAT`); both YAML-validated
- [x] `dist/` rebuilt from the final tree (binary + source tarballs +
  `SHA256SUMS` re-verified)
- [x] Cargo suite (53), extension suite (10), Clippy, and all examples
  re-verified after every publication-prep change; Luca Code untouched
- [ ] User creates the GitHub repo + remote, pushes, tags, releases;
  Marketplace publish needs publisher access + `VSCE_PAT` (see handoff)
