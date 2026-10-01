# Luca UI architecture

## Product boundary

Luca UI is the declarative UI layer for Luca Code. It uses `.lucu` source files and Luca Code for general-purpose logic in `.lucc` files. Luca UI is not a second general-purpose language, and Luca Code must not depend on Luca UI.

Luca Code and Luca UI belong to one compatibility generation. Luca UI should reuse Luca Code's language infrastructure where possible instead of maintaining a second lexer, parser, type system, or interpreter. Luca Build is a later integrated environment and is outside this scaffold.

## Planned source tree

```text
compiler/
  parser/       .lucu syntax and Luca Code integration
  components/   component declarations and validation
  state/        declarative state and update model
  events/       event declarations and dispatch model
  bindings/     data and property bindings
  layout/       cross-platform layout model
  styling/      component and theme styling
  navigation/   component-based navigation
  codegen/      platform-independent lowering and backends
runtime/
  core/         component lifecycle and state runtime
  rendering/    renderer abstraction
  platform/     platform adapters
  accessibility/ accessible semantics and platform mapping
std/
  components/   core cross-platform component library
  themes/       theme definitions and system appearance
  localization/ localization primitives and resources
  animation/    declarative animation and transitions
tests/          compiler, runtime, accessibility, and UI tests
examples/       small runnable .lucu and paired .lucc examples
docs/           specification and implementation notes
```

This tree describes intended responsibilities, not a commitment to one crate/package per directory. Keep early implementation boundaries small and follow the existing Luca Code repository's package conventions after confirming the integration model.

## Luca Code integration checkpoint

The inspected Luca Code 1.0 Beta is a Rust crate. Its `src/lib.rs` exposes `run` and `run_with_input`, and its lexer, parser, AST, interpreter, and type modules are public. Before implementing Luca UI, OpenCode must inspect those APIs and identify the smallest maintainable integration seam. Do not copy Luca Code's compiler or fork its syntax/semantics. Record whether Luca UI should use a path dependency, a shared workspace, or an intentionally expanded Luca Code library API; do not relocate or edit the Luca Code repository as part of this scaffold.

## Design constraints

- UI description is declarative and state-driven.
- Components are manipulated through state and events, not component references or imperative `show`, `hide`, or setter calls.
- Visibility and disabled behavior are ordinary component properties.
- The core component vocabulary stays cross-platform; target platforms adapt its rendering.
- Accessibility is part of the core component model, with sensible inferred semantics and explicit overrides.
- Navigation is component-based; string route identifiers are not the core navigation model.
- Animation, declarative transitions, localization, and themes are part of the intended core API. System appearance is followed automatically by default.
- Invalid component arguments and detectable invalid properties should produce ordinary, source-located Luca errors. UI testing and a development inspector are part of the intended developer experience.
- Component-scoped asynchronous work is cancelled when its component leaves the tree.

Unspecified platform details and syntax are implementation questions. OpenCode must not silently turn those questions into new language design decisions; document a minimal compatible assumption or report the unresolved dependency.
