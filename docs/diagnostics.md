# Diagnostics catalog — Luca UI 0.9 Alpha (stable)

Every toolchain failure is a source-located Luca error rendered as
`Error at line:column: message` (`luca_code::error::LucaError`). Luca UI
produces errors only — no warnings. This catalog freezes the message shapes:
`tests/diagnostics.rs` asserts each reachable message byte-for-byte, so any
wording change is a breaking change requiring a version note.

Conventions: `line:column` is 1-based. Unless noted, the location is the
offending declaration (prop key start, entry start, block start, or root).

## Parse and structure (`parser`)

| # | Message | Location | Example |
|---|---|---|---|
| P1 | `Expected a component, found empty file` | 1:1 | empty input |
| P2 | `Root component must start at column 1` | offending indent | `␣␣App:` |
| P3 | `Property outside a component` | prop/line start | `id = "x"` alone |
| P4 | `Expected a component name before ':'` | `:` line | `:` |
| P5 | `Invalid component name '123'` | name start | `123:` |
| P6 | `Expected 'Name:' or 'key = value', found 'App'` | line start | `App` |
| P7 | `Expected a single root component per .lucu file` | second root | two `App:` blocks |
| P8 | `Tabs are not allowed for indentation; use spaces` | offending line, col 1 | tab indent |
| P9 | `Unterminated multiline comment` | `[[` start | `[[ oops` |
| P10 | `Unterminated string literal` | value start | `text = "oops` |
| P11 | `Invalid property name 'my-prop'` | key start | `my-prop = 1` |
| P12 | `Expected a value after '='` | after `=` | `text =` |
| P13 | `Unknown escape '\q' in string` | value start | `text = "a\qb"` |
| P14 | `Invalid value '1.2.3'` | value start | `text = 1.2.3` |
| P15 | `Imperative component operations are not supported; update state and allow the UI to react` | call line/value start | `show()`, `text = hide()` |
| P16 | `Imperative component operation 'hide' is not supported; update state and allow the UI to react` | key start | `hide = true` |
| P17 | `` `{block}` must belong to a component `` | root block | `layout:` at column 1 |

Defensive (in code, unreachable through `parse`; not asserted): `Component
outside a parent component`, both `Inconsistent indentation` branches.

## Component validation (`component`, `library`)

| # | Message | Location |
|---|---|---|
| C1 | `Duplicate property 'id' on component 'App' (first at 2:3)` | second key |
| C2 | `Duplicate id 'a' (first at 2:3); ids must be unique for tests` | second `id` |
| C3 | `Property 'id' must be a non-empty string` | `id` key |
| C4 | `Property 'visible' must be bool, found str` | key (`str`/`number`/`none`/`ref` as found) |
| C5 | `Property 'label' must be str, found number` | key |
| C6 | `Unknown property 'count' on component 'App' (supports: disabled, id, label, visible)` | key; registered components only, keys sorted |

Literal `Ref` values on locked keys (C3–C5 shapes) parse and enforce at
resolution instead (see R4–R6).

## Blocks (`component` extraction, `layout` shared checks)

| # | Message | Location |
|---|---|---|
| B1 | `Duplicate 'layout' block on component 'App'` | second block |
| B2 | `` Only `key = value` entries are allowed inside `layout` (found component 'Text') `` | nested component |
| B3 | `Duplicate 'gap' in `layout` block of component 'App' (first at 3:5)` | second entry |

B1–B3 apply to every block kind with its own name (`layout`, `style`,
`state`, `events`, `transitions`, `theme`).

## State (`state`)

| # | Message | Location |
|---|---|---|
| S1 | `State initial values must be literals (found reference 'b')` | entry |
| S2 | `Duplicate state 'a' (first at 3:5)` | second declaration |
| S3 | `State 'x' must be a scalar value in 0.4` | declaration/update site |
| S4 | `Expected int, found str for state 'visits'` | declaration site |
| S5 | `'missing' is not declared` | 1:1 (programmatic update) |
| S6 | `Invalid number '999…' for state 'n'` | entry (overflow/scale) |

## Bindings (`bindings`)

| # | Message | Location |
|---|---|---|
| R1 | `'nowhere' is not declared` | binding site |
| R2 | `Invalid number '999…'` (no `for` suffix) | literal site |
| R3 | `Duplicate id 'a' (first at 5:3); ids must be unique for tests` | second resolved `id` |
| R4 | `Property 'id' must be a non-empty string` | binding site |
| R5 | `Property 'visible' must be bool, found str` | binding site |
| R6 | `Property 'label' must be str, found int` | binding site |

Defensive: the `Invalid value` fallback and the imperative-key arm (parse
rejects both shapes first).

## Events, navigation, runtime (`events`, `navigation`, `runtime`)

| # | Message | Location |
|---|---|---|
| E1 | `No component with id 'missing'` | tree root |
| E2 | `Component 'greeting' declares no events` | component |
| E3 | `Component 'visit' declares no event 'swipe'` | `events:` block |
| E4 | `Event 'tap' handler must be a Luca Code handler name, found str` | entry |
| N1 | `Cannot navigate back from the root screen` | tree root |
| L1 | `Cannot scope work to unmounted component 'x'` | tree root |

`Navigator` reuses E1's shape for unknown targets.

## Themes (`themes`)

| # | Message | Location |
|---|---|---|
| T1 | `Theme 'name' must be a string` | entry |
| T2 | `Unknown theme mode 'sepia'; expected system, light, or dark` | entry |
| T3 | `Unknown theme mode; expected system, light, or dark` | entry (non-scalar) |

## Seam (`seam`)

| # | Message | Location |
|---|---|---|
| F1 | `Luca UI source files must use the .lucu extension` | 1:1 |
| F2 | `Could not read {path}: {os error}` | 1:1 |

## Testing harness (`testing`)

Reuses E1's shape (`No component with id '…'`) at the resolved root for
`prop`/`text`/`visible`/`label` lookups.
