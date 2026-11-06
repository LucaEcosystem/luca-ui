# The `.lucu` language — 1.0.2 Beta

> Luca UI **1.0.2 Beta**. This page is the full reference for the surface syntax of a `.lucu` file. The locked *behaviors* are stable for the 1.0 generation; the exact *spelling* of some blocks is still a labeled assumption that may change before 1.0 stable. Wherever that matters, the section says so.

## Contents

1. [Files](#1-files)
2. [Comments](#2-comments)
3. [Components](#3-components)
4. [Properties](#4-properties)
5. [Values and types](#5-values-and-types)
6. [The locked properties: id, visible, disabled, label](#6-the-locked-properties-id-visible-disabled-label)
7. [Declaration blocks](#7-declaration-blocks)
8. [Bindings](#8-bindings)
9. [What is not allowed](#9-what-is-not-allowed)
10. [Indentation and layout of the file](#10-indentation-and-layout-of-the-file)

---

## 1. Files

- A `.lucu` file is one UI declaration: exactly **one root component**, with children nested under it (● locked: a single-root tree).
- The CLI refuses every source that does not end in `.lucu`; `.lucc` means Luca Code logic and never goes through this parser (● locked boundary). See the [CLI reference](cli.md).
- Every failure is a source-located diagnostic: `Error at line:column: message` (● locked — see the [diagnostics catalog](diagnostics.md)). `line:column` is 1-based and counts **characters**, not bytes (Unicode-safe: a file containing `こんにちは` still gets correct columns).

## 2. Comments

Two comment forms, matching Luca Code (● locked):

```text
-- Everything from `--` to end of line is ignored (outside a string).
[[ this spans
   multiple lines ]]
```

- `--` comments are stripped outside double-quoted strings, so `text = "-- hi"` is the string `"-- hi"`, not a comment.
- An unterminated `[[` is an error: `Unterminated multiline comment`, located at the `[[`.
- Multiline comments keep line numbers intact (the parser preserves the content after the comment).

## 3. Components

Every component is one line ending with a colon:

```text
App:
Text:
  ...
MyWidget:
```

- The name is an identifier: ASCII letters, digits, and `_`, starting with a letter or `_`. [`App`, `Button`, `Text` are registered](components.md); every other name is accepted as an **unregistered** component (the component vocabulary is open).
- Nesting is expressed by indentation (spaces only — see [§10](#10-indentation-and-layout-of-the-file)).
- A component at column 1 when the root already exists is a second root — error: `Expected a single root component per .lucu file`.
- The root itself must start at column 1: an indented first line fails with `Root component must start at column 1`.

## 4. Properties

Properties are `key = value` lines that belong to the component they are indented under:

```text
App:
  id = "app"
  visible = true
  custom_note = "anything"
```

- The key is an identifier (names like `my-prop` fail with `Invalid property name 'my-prop'`).
- Duplicates are errors: `Duplicate property 'text' on component 'Text' (first at 3:3)`.
- The property name must be written before `=` on one line; `text =` with no value is `Expected a value after '='`.

## 5. Values and types

Every property value is either a literal or a [binding](#8-bindings):

| Literal form | Meaning | Example |
|---|---|---|
| `"…"` | A string. Escapes: `\"`, `\\`, `\n`. Any other backslash escape is an error (`Unknown escape '\q' in string`). | `text = "Hello\nline two"` |
| `true` / `false` | Booleans | `visible = false` |
| `integer` | A whole number | `count = 12` |
| `decimal` | A number containing a dot. The written **scale is preserved exactly** (`2.50` stays `2.50` — Luca Code exact-decimal semantics) | `ratio = -3.50` |
| `none` | The absent value; occupies any type | `missing = none` |
| bare identifier | A [binding](#8-bindings) to a state variable | `text = title` |

Types of state variables are the Luca Code types: `str`, `bool`, `int`, `dec`, and `uni` (inferred when a state entry is `none`).

- Strings never span lines; a missing closing quote is `Unterminated string literal`.
- Malformed numbers (`1.2.3`) fail with `Invalid value '1.2.3'`.

## 6. The locked properties: id, visible, disabled, label

Four property names are **locked** (●): the toolchain enforces their type everywhere they appear, on registered and unregistered components alike.

| Property | Rule |
|---|---|
| `id` | Optional on every component. A **non-empty unique string**; duplicates across the tree fail: `Duplicate id 'a' (first at 2:3); ids must be unique for tests`. Ids are *the* addressing mechanism — tests, events, and navigation address components by `id` only. |
| `visible` | Optional; must be a boolean (or a binding resolving to one). **Absent `visible` means shown.** When it is `false`, the component and its whole subtree are skipped entirely by every backend. There is no `hide()` — visibility is always just this property. |
| `disabled` | Optional; must be a boolean (or a binding resolving to one). An ordinary property; how "disabled" presents is a backend concern. |
| `label` | Optional; must be a string (or a binding resolving to one). The explicit accessible label — when present, it **wins** over text inference (see below). |

Accessibility from these (● locked): every component's accessible label is either its explicit `label`, or — only when there is no label — its *string* `text`, inferred automatically. Missing labels are perfectly valid; not every component needs one. Platform mapping of labels is part of the backend adapter milestone (○ not implemented).

## 7. Declaration blocks

Six block names attach to a component and each may appear **at most once** per component:

```text
App:
  id = "app"
  layout:
    gap = 2
  style:
    tone = "soft"
  state:
    title = "Luca UI"
  events:
    tap = record_visit
```

| Block | Purpose | Details |
|---|---|---|
| `layout:` | Platform-independent layout entries | [Styling and theming](styling-theming.md) |
| `style:` | Platform-independent style entries | [Styling and theming](styling-theming.md) |
| `state:` | Typed scalar variables | [State and events](state-and-events.md) |
| `events:` | `event = handler` routing declarations | [State and events](state-and-events.md) |
| `transitions:` | Declarative animation entries (reduced-motion aware) | [Styling and theming](styling-theming.md) |
| `theme:` | Theme selection (`name`, `mode`) | [Styling and theming](styling-theming.md) |

Block rules shared by all six:

- Must live *inside* a component; one written at the file root is rejected: `` `layout` must belong to a component``.
- Hold only `key = value` entries — nesting a component inside a block fails: `Only `key = value` entries are allowed inside `state` (found component 'Text')`.
- A duplicated block fails (`Duplicate 'layout' block on component 'App'`), and a duplicated entry within a block fails naming the first location (`Duplicate 'gap' in `layout` block of component 'App' (first at 3:5)`).
- `layout:`/`style:` entry names carry no global meaning — they are validated **structurally only** (identifier keys, scalar values, no duplicates). Layout and style semantics (dimensions, defaults, precedence) are explicitly open (○ not defined).

## 8. Bindings

A bare-identifier value is a **binding** to a state variable:

```text
App:
  id = "app"
  state:
    title = "Luca UI"
  Text:
    id = "t"
    text = title
```

Rules:

- The name must be declared in some `state:` block in the tree; otherwise the binding site reports `'title' is not declared`.
- Locked property rules apply to the **resolved** value: `visible = text_field` where `text_field` holds a string fails at the binding with `Property 'visible' must be bool, found str`.
- Bindings are allowed on component properties only; `state:` initial values must be literals (this keeps declaration order-free), and `layout:`/`style:` entries bind like component props but carry no locked type checks.
- A binding resolves to a **value** only — never to a component reference (● locked: no component references exist in the model).

## 9. What is not allowed

Every line below is a compile-time Luca error:

| You write | Rejected message |
|---|---|
| `hide()` / `show()` / `set_text = "hi"` | `Imperative component operation 'hide' is not supported; update state and allow the UI to react` |
| `text = greet("hi")` (call-shaped value) | `Imperative component operations are not supported; update state and allow the UI to react` |
| `hide = true` / `show = false` / `set_text` keys | same imperative-operation message |

Why: Luca UI is deliberately declarative (● locked design). You cannot command a component; you change **state**, and components that bind to it re-render. There is no component-mutation API anywhere — `set`/`flush` on state is the only path a change takes.

## 10. Indentation and layout of the file

```text
App:
  id = "app"
  Text:
    id = "greeting"
    text = "Hello, Luca!"
```

- Indentation is spaces; tabs are rejected: `Tabs are not allowed for indentation; use spaces`.
- The root component must start at column 1 (`Root component must start at column 1`); a second column-1 component is `Expected a single root component per .lucu file`.
- Levels are **lenient but deterministic**: any deeper indentation under a parent opens a child level; you do not have to use consistent multiples of any number. Properties and components belong to the most recent component line above them with smaller indentation.
- Once an indent level exists, later lines may return to it. The flagged errors cover the pathological cases only (`Property outside a component`, `Inconsistent indentation`).
