# Luca UI 1.0 design specification

## Status and authority

This document consolidates the finalized architectural decisions available from the Luca UI design conversation. It is the implementation baseline for Luca UI. Luca Code's own language reference remains authoritative for `.lucc` syntax and semantics. The design conversation contains more detailed decisions than the cached transcript available during this setup; this document records confirmed decisions and deliberately avoids inventing missing syntax or API signatures. Add a decision here only when it is supported by the full design record or an explicit user decision.

## Product definition

Luca UI is a declarative UI layer built directly on Luca Code, not a second general-purpose programming language. Luca Code remains the home for imperative and general-purpose logic in `.lucc`; UI descriptions use `.lucu`. Luca Code must not depend on Luca UI. The two products share a compatibility generation and are designed to work together within it.

## Core architecture

The UI layer covers components, state, events, bindings, layout, styling, navigation, accessibility, animation, themes, and localization. It targets a platform-independent API backed by a rendering abstraction. Native, custom/GPU, and web rendering are intended backend options; the UI API should not force platform-specific components into the core vocabulary.

Luca UI should reuse Luca Code infrastructure wherever possible. It must not grow a parallel general-purpose compiler or interpreter. The current Luca Code crate and its public APIs must be inspected before selecting the exact integration seam.

## Declarative behavior

- UI output is derived from application state and declared properties.
- Do not expose programmatic references to component instances as the normal control mechanism.
- Do not add imperative component operations such as `hide()`, `show()`, or `set_text()`; update state and allow the UI to react.
- Every component may have an `id` property.
- `visible` and `disabled` are ordinary properties, where applicable.
- State updates are batched. Component-scoped asynchronous work is cancelled when its component disappears.

## Components, layout, and styling

Core components are cross-platform. They should adapt to the target platform while retaining Luca UI's shared API. Platform-specific APIs may exist outside the core standard component vocabulary. Layout and styling are part of the declarative layer. The exact component inventory, property types, layout syntax, and styling syntax must follow the complete design record; do not infer them from the directory names in this scaffold.

## Events, navigation, and lifecycle

Events are the interaction boundary between components and Luca Code state/logic. Navigation is component-based; string-based routes are not the core navigation model. Component lifecycle is managed by the runtime, including cancellation of component-scoped asynchronous work when a component leaves the rendered tree.

## Accessibility

Accessibility is a core capability, not a platform-only add-on. Components expose accessibility labels and related semantics. Luca UI infers accessible information when it can and allows explicit overrides. Platform adapters map the shared accessibility model to their native accessibility systems.

## Animation, themes, and localization

Animation is part of the core API and transitions are declarative. Localization support is built in. Themes are first-class, and system appearance is followed automatically by default. Detailed syntax, resource formats, animation timing defaults, and theme token names must be confirmed from the complete design record before implementation.

## Diagnostics and developer tools

Invalid UI properties should produce normal Luca errors. Invalid component arguments should be compile-time errors when the compiler can detect them. Luca UI includes a dedicated UI testing API and explicit component identifiers suitable for tests. A development-only visual inspector is part of the intended toolset and may inspect the component tree, state, and layout bounds.

## Known locked decisions

The following numbered choices were directly visible in the referenced conversation:

| Decision | Locked behavior |
| --- | --- |
| 90 | Navigation is component-based; no string-based routes |
| 95 | Component-scoped async work is cancelled when the component disappears |
| 96–100 | Accessibility is core; labels are supported; inference is used where possible; core components are cross-platform and adapt to the target platform |
| 106–115 | Invalid property values and detectable invalid arguments use Luca errors; dedicated UI tests and IDs are supported; a development inspector is included; state updates batch |
| 116–125 | `id` is available to every component; no component references or imperative manipulation; `visible` and `disabled` are ordinary properties; animation, declarative transitions, localization, themes, and automatic system appearance are core |

## Compatibility and release

Luca Code and Luca UI versions share a generation and should remain compatible within it. Treat Luca Code 1.0 Beta as the real host language foundation. Preserve its behavior and source extension. Use `.lucu` for Luca UI files. Luca Build integration is a later phase, after Luca Code and Luca UI foundations.

## Open specification material

The cached conversation excerpt does not expose every earlier decision or the complete syntax examples. Before declaring this document exhaustive, reconcile it against the full finalized design record. In particular, do not guess exact grammar, component signatures, state declaration syntax, binding syntax, layout constraints, style property names, transition syntax, localization resource format, or renderer-specific behavior.

## 0.9 reconciliation pointer (informative, adds no decisions)

`docs/implementation-status.md` §15 maps every section and numbered
decision above to implementing code and running tests, or marks it
explicitly open with its closer. `docs/diagnostics.md` freezes diagnostic
wording; `docs/compatibility.md` records the verified Code/UI pair and the
backend matrix. This section adds no language decisions; per §Status and
authority, new decisions still require the full design record or an
explicit user decision.
