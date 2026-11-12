# Luca UI for VS Code — 1.0.2 Beta

Official language support for [Luca UI](https://github.com/LucaEcosystem/luca-ui) 1.0.2 Beta. Zero runtime dependencies.

- `.lucu` syntax highlighting (components, properties, comments, strings)
- Snippets for the common declarations
- **Luca UI: New Luca File** — opens a starter view instantly
- **Luca UI: Run Luca File** — saves and runs the active `.lucu` file through the `luca-ui` CLI, with `Error at line:column:` output surfaced as clickable editor diagnostics

## Install

### 1. Install the `luca-ui` CLI

Follow [Getting started](https://github.com/LucaEcosystem/luca-ui/blob/main/docs/getting-started.md) in the Luca UI repository. In short (from the Luca UI checkout, with the Luca Code checkout as its sibling):

```sh
cargo install --path . --root ~/.luca-ui
export PATH="$HOME/.luca-ui/bin:$PATH"
luca-ui --version   # luca-ui 1.0.2 Beta
```

### 2. Install the extension

The extension is available on the Visual Studio Marketplace
(`luca-ecosystem.luca-ui`) and through the repository. Two options:

- **From the GitHub release:** download `luca-ui-1.0.2.vsix` and run

  ```sh
  code --install-extension luca-ui-1.0.2.vsix
  ```

- **From a checkout of this repository:**

  ```sh
  code --install-extension editors/vscode/luca-ui-1.0.2.vsix
  # or rebuild it:
  cd editors/vscode && npx @vscode/vsce package
  ```

### 3. Open a `.lucu` file

Done — highlighting activates on any `.lucu` file.

## Use

| Action | How |
|---|---|
| New starter file | Command Palette → **Luca UI: New Luca File** |
| Run the active file | Command Palette → **Luca UI: Run Luca File**, or the ▶ title-bar button when a `.lucu` editor is focused (the file is saved first) |
| See errors inline | Runs in the **integrated terminal** by default; switch `lucaui.runInTerminal` **off** to capture `Error at line:column:` output as squiggles + Problems entries |
| Read full CLI output | **Output** panel → channel **Luca UI** |

When diagnostics mode is on, each `Error at line:column: message` line becomes a VS Code diagnostic at that exact position. Clicking it jumps to the location; wordings come straight from the frozen [diagnostics catalog](https://github.com/LucaEcosystem/luca-ui/blob/main/docs/diagnostics.md).

## Settings

| Setting | Default | Description |
|---|---|---|
| `lucaui.cliPath` | `"luca-ui"` | Path to the CLI. Leave it if `luca-ui` is on your `PATH`; set an absolute path otherwise. |
| `lucaui.runInTerminal` | `true` | Run in the integrated terminal (familiar raw output). When off, output goes to the **Output > Luca UI** channel and errors become editor diagnostics. |

## Snippets

`snippets/lucu.json` provides prefixes for the common shapes (App root, nested components, `state:`/`events:`/`layout:`/`style:` blocks, and so on). Type the prefix and accept with Tab.

## Troubleshooting

- **“luca-ui not found”** when running a file: the CLI is not on your `PATH`. Set `lucaui.cliPath` to the absolute path of your `luca-ui` binary (for example `~/.luca-ui/bin/luca-ui`).
- **No squiggles after a failed run:** terminal mode is on. Set `lucaui.runInTerminal` to `false`.
- **Everything looks plain text:** make sure the file has the `.lucu` extension and the extension is installed/enabled. Language id: `lucu`.

## Development

```sh
cd editors/vscode
npm test          # runs the unit tests (no dependencies to install)
npx @vscode/vsce package   # builds luca-ui-1.0.2.vsix
```

Source map: `extension.js` (commands, CLI discovery, diagnostics parsing), `syntaxes/lucu.tmLanguage.json` (TextMate grammar), `snippets/lucu.json` (snippets), `language-configuration.json` (brackets, comments: `--` and `[[ … ]]`).
