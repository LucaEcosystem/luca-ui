# Luca UI documentation — 1.0.2 Beta

Luca UI is the declarative UI layer for [Luca Code](https://github.com/LucaEcosystem/luca-code): you describe a screen in a `.lucu` file, write its logic in a `.lucc` file, and the toolchain binds them together. Original README of Luca UI is at the repository root; this folder holds the full documentation set.

**This is a Beta release (`1.0.2 Beta`).** Everything documented here is implemented and tested, but details may change before 1.0 stable. Where something is still expected to change, the guides mark it explicitly.

## Status markers used throughout these guides

| Marker | Meaning |
|---|---|
| ● &nbsp;**Locked** | Behavior that is design-locked for the 1.0 generation and enforced by the toolchain. It will not change within the generation. |
| ◐ &nbsp;**Assumption** | Working syntax or behavior, adopted as the smallest compatible choice while the full design record is being finalized. Functional today; may change. |
| ○ &nbsp;**Not implemented** | An intended part of the design with no implementation yet. Never promised for a specific release. |

If a feature is not marked, it is ● locked (stable).

## Reading order

### For newcomers (start here)

1. [Getting started](getting-started.md) — install the CLI, run your first `.lucu` file, understand what Luca UI actually produces. (~10 minutes)
2. [The `.lucu` language](language.md) — components, properties, values, blocks, and the rules the compiler enforces.
3. [Components](components.md) — `App`, `Text`, `Button`, and how unregistered component names are treated.
4. [State and events](state-and-events.md) — how UI output is derived from state, and how events route to Luca Code logic.

### For day-to-day development

5. [Testing](testing.md) — the deterministic `TestHarness` API.
6. [Rendering](rendering.md) — the text backend, the static visual pipeline, and the SVG backend.
7. [Styling, theming, and localization](styling-theming.md) — `layout:`, `style:`, `transitions:`, `theme:`, and resources.
8. [Examples](examples.md) — every shipped example with its verified output.
9. [CLI reference](cli.md) — `check`, `render`, and the development inspector.
10. [Architecture](ARCHITECTURE.md) — the shipped module boundaries and the one-way Luca Code seam.

### Reference

11. [Diagnostics catalog](diagnostics.md) — every error message, frozen; render warnings.
12. [Compatibility notes](compatibility.md) — the verified Luca Code/Luca UI pair and backend matrix.
13. [Release notes](RELEASE-NOTES.md) — what is in this release.
14. [Changelog](CHANGELOG.md) — history of all releases.

## Where to get help

- Install problems, extension setup: [Getting started](getting-started.md) and the [VS Code extension README](../editors/vscode/README.md).
- Explain-the-error lookups: [Diagnostics catalog](diagnostics.md).
- Bug reports and feature requests: [Issues on GitHub](https://github.com/LucaEcosystem/luca-ui/issues) (use the templates).
- Questions and design discussion: [GitHub Discussions](https://github.com/LucaEcosystem/luca-ui/discussions).
- Contributing and security: [CONTRIBUTING.md](../CONTRIBUTING.md) and [SECURITY.md](../SECURITY.md). Luca UI is proprietary and does not accept external code contributions.
