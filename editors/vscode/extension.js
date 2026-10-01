// Luca UI VS Code extension (zero runtime dependencies).
//
// Commands:
//   lucaui.createFile - open a new .lucu file with a starter view
//   lucaui.runFile    - save and run the active .lucu file with the luca-ui CLI
//
// The CLI is discovered via the `lucaui.cliPath` setting (default "luca-ui"
// on PATH). `Error at line:column:` output becomes editor diagnostics;
// everything the CLI prints goes to the Output channel "Luca UI".
"use strict";

const childProcess = require("child_process");
const fs = require("fs");
const path = require("path");

// Placeholder until the public launch; points at the install guide.
const INSTALL_GUIDE_URL = "https://github.com/LucaEcosystem/luca-ui#install-10-beta";

const STARTER_VIEW = 'App:\n  id = "app"\n  Text:\n    id = "hello"\n    text = "Hello, Luca!"\n';

const DIAGNOSTIC_PATTERN = /(Error|Warning) at (\d+):(\d+):\s*(.*)/;

/**
 * Parse CLI output lines into plain diagnostics.
 * Returns [{ kind: "error"|"warning", line (1-based), column (1-based), message }].
 */
function parseDiagnostics(output) {
  const diagnostics = [];
  for (const rawLine of String(output).split(/\r?\n/)) {
    const match = DIAGNOSTIC_PATTERN.exec(rawLine);
    if (!match) {
      continue;
    }
    diagnostics.push({
      kind: match[1] === "Warning" ? "warning" : "error",
      line: Number(match[2]),
      column: Number(match[3]),
      message: match[4].trim(),
    });
  }
  return diagnostics;
}

/**
 * Find an executable by name on PATH. Returns the full path or null.
 * `envPath` defaults to process.env.PATH so tests can inject a fixture.
 */
function findOnPath(name, envPath) {
  const pathValue = envPath !== undefined ? envPath : process.env.PATH || "";
  const directories = pathValue.split(path.delimiter).filter(Boolean);
  const candidates = process.platform === "win32" ? [`${name}.exe`, `${name}.cmd`, name] : [name];
  for (const directory of directories) {
    for (const candidate of candidates) {
      const full = path.join(directory, candidate);
      try {
        fs.accessSync(full, fs.constants.X_OK);
        return full;
      } catch {
        // Not usable here; keep searching.
      }
    }
  }
  return null;
}

/**
 * Build the shell command that runs a file, quoting for paths with spaces.
 */
function buildRunCommand(cliPath, filePath) {
  return `"${cliPath}" "${filePath}"`;
}

/**
 * Resolve the CLI to run. `configured` is the `lucaui.cliPath` setting value.
 * Returns an absolute path, or null when no usable CLI is found.
 */
function resolveCli(configured) {
  const value = (configured || "luca-ui").trim() || "luca-ui";
  if (value.includes(path.sep) || (process.platform === "win32" && value.includes("/"))) {
    try {
      fs.accessSync(value, fs.constants.X_OK);
      return value;
    } catch {
      return null;
    }
  }
  return findOnPath(value);
}

function activate(context) {
  const vscode = require("vscode");
  const output = vscode.window.createOutputChannel("Luca UI");
  const diagnostics = vscode.languages.createDiagnosticCollection("lucu");
  context.subscriptions.push(output, diagnostics);

  function toVscodeDiagnostics(fileUri, parsed) {
    return parsed.map((entry) => {
      const position = new vscode.Position(Math.max(entry.line - 1, 0), Math.max(entry.column - 1, 0));
      const range = new vscode.Range(position, position);
      const severity =
        entry.kind === "warning" ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;
      return new vscode.Diagnostic(range, entry.message, severity);
    });
  }

  async function runFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== "lucu") {
      vscode.window.showErrorMessage("Luca UI: open a .lucu file first, then run it.");
      return;
    }
    const document = editor.document;
    if (document.isDirty) {
      await document.save();
    }
    const config = vscode.workspace.getConfiguration("lucaui");
    const cliPath = resolveCli(config.get("cliPath"));
    if (!cliPath) {
      const choice = await vscode.window.showErrorMessage(
        "Luca UI: the 'luca-ui' CLI was not found. Install it, or point the 'lucaui.cliPath' setting at it.",
        "Open install guide",
        "Set CLI path"
      );
      if (choice === "Open install guide") {
        vscode.env.openExternal(vscode.Uri.parse(INSTALL_GUIDE_URL));
      } else if (choice === "Set CLI path") {
        vscode.commands.executeCommand("workbench.action.openSettings", "lucaui.cliPath");
      }
      return;
    }

    if (config.get("runInTerminal")) {
      let terminal = vscode.window.terminals.find((candidate) => candidate.name === "Luca UI");
      if (!terminal) {
        terminal = vscode.window.createTerminal("Luca UI");
      }
      terminal.show();
      terminal.sendText(buildRunCommand(cliPath, document.fileName));
      return;
    }

    diagnostics.delete(document.uri);
    output.clear();
    output.appendLine(`$ ${cliPath} ${document.fileName}`);
    output.show(true);
    const child = childProcess.spawn(cliPath, [document.fileName], { cwd: path.dirname(document.fileName) });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
      output.append(chunk.toString());
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
      output.append(chunk.toString());
    });
    child.on("error", (error) => {
      output.appendLine(`could not start the luca-ui CLI: ${error.message}`);
      vscode.window.showErrorMessage(`Luca UI: could not start '${cliPath}': ${error.message}`);
    });
    child.on("close", (code) => {
      const parsed = parseDiagnostics(`${stdout}\n${stderr}`);
      if (parsed.length > 0) {
        diagnostics.set(document.uri, toVscodeDiagnostics(document.uri, parsed));
        const first = parsed.find((entry) => entry.kind === "error") || parsed[0];
        output.show(true);
        vscode.window.showErrorMessage(
          `Luca UI: ${first.kind} at ${first.line}:${first.column}: ${first.message}`
        );
      } else if (code !== 0) {
        output.show(true);
        vscode.window.showErrorMessage(`Luca UI: program exited with code ${code}; see Output > Luca UI.`);
      } else {
        vscode.window.showInformationMessage(`Luca UI: ran ${path.basename(document.fileName)} successfully.`);
      }
    });
  }

  async function createFile() {
    const document = await vscode.workspace.openTextDocument({ language: "lucu", content: STARTER_VIEW });
    await vscode.window.showTextDocument(document);
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("lucaui.runFile", runFile),
    vscode.commands.registerCommand("lucaui.createFile", createFile)
  );
}

function deactivate() {}

if (typeof module !== "undefined") {
  module.exports = { parseDiagnostics, findOnPath, resolveCli, buildRunCommand, INSTALL_GUIDE_URL };
}
module.exports.activate = activate;
module.exports.deactivate = deactivate;
