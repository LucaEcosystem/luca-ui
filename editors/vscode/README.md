# Luca UI for VS Code (1.0 Beta)

Official language support for [Luca UI](https://github.com/LucaEcosystem/luca-ui) 1.0 Beta: `.lucu` syntax highlighting, snippets, and one-key run.

## Setup

1. Install the `luca-ui` CLI (see `Install (1.0 Beta)` in the Luca UI repository README, or `cargo install --path <checkout>`).
2. Install this extension (search "Luca UI" in the Extensions view after it is published, or package it locally with `npx @vscode/vsce package`).
3. Open any `.lucu` file. If the CLI is not on your `PATH`, set the `lucaui.cliPath` setting to its absolute path.

## Use

- **Luca UI: New Luca File** — open a starter `.lucu` view.
- **Luca UI: Run Luca File** — save and run the active file (editor title-bar button when a `.lucu` file is open). CLI output appears in **Output > Luca UI**; `Error at line:column:` becomes editor diagnostics.

## Develop

- `npm test` runs the unit tests (no dependencies to install).
- Snippets live in `snippets/lucu.json`; the TextMate grammar in `syntaxes/lucu.tmLanguage.json`.
