// Manifest/grammar checks: every contributed file must exist and parse,
// and the wiring (language id, commands, config) must stay consistent.
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.join(__dirname, "..");

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

test("package.json is a valid extension manifest", () => {
  const manifest = readJson("package.json");
  assert.equal(manifest.name, "luca-ui");
  assert.match(manifest.version, /^\d+\.\d+\.\d+$/);
  assert.equal(manifest.main, "./extension.js");
  assert.ok(fs.existsSync(path.join(root, "extension.js")));
  assert.deepEqual(manifest.activationEvents, ["onLanguage:lucu"]);
});

test("language, grammar, and snippets wiring is consistent", () => {
  const manifest = readJson("package.json");
  const [language] = manifest.contributes.languages;
  assert.equal(language.id, "lucu");
  assert.ok(language.extensions.includes(".lucu"));
  for (const file of [
    language.configuration,
    manifest.contributes.grammars[0].path,
    manifest.contributes.snippets[0].path,
  ]) {
    assert.ok(file, "contributed path is set");
    assert.ok(fs.existsSync(path.join(root, file)), `${file} exists`);
  }
  assert.equal(manifest.contributes.grammars[0].scopeName, "source.lucu");
  assert.equal(manifest.contributes.grammars[0].language, "lucu");
  assert.equal(manifest.contributes.snippets[0].language, "lucu");
  const commands = manifest.contributes.commands.map((command) => command.command);
  assert.ok(commands.includes("lucaui.runFile"));
  assert.ok(commands.includes("lucaui.createFile"));
  assert.ok(manifest.contributes.configuration.properties["lucaui.cliPath"]);
  assert.equal(manifest.contributes.configuration.properties["lucaui.runInTerminal"].default, true);
});

test("grammar covers Luca UI blocks and rejects imperative calls", () => {
  const grammar = readJson("syntaxes/lucu.tmLanguage.json");
  assert.equal(grammar.scopeName, "source.lucu");
  const text = JSON.stringify(grammar);
  for (const word of ["layout", "state", "events", "transitions", "theme", "style"]) {
    assert.ok(text.includes(word), `grammar mentions ${word}`);
  }
  assert.ok(text.includes("comment.line.double-dash"));
  assert.ok(text.includes("invalid.illegal.imperative"));
});

test("grammar regexes all compile", () => {
  const grammar = readJson("syntaxes/lucu.tmLanguage.json");
  const patterns = [];
  (function collect(node) {
    if (!node || typeof node !== "object") {
      return;
    }
    for (const key of ["match", "begin", "end"]) {
      if (typeof node[key] === "string") {
        patterns.push(node[key]);
      }
    }
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) {
        value.forEach(collect);
      } else if (value && typeof value === "object") {
        collect(value);
      }
    }
  })(grammar.repository);
  assert.ok(patterns.length > 0);
  for (const pattern of patterns) {
    assert.doesNotThrow(() => new RegExp(pattern), `compiles: ${pattern}`);
  }
});

test("snippets parse and cover the core forms", () => {
  const snippets = readJson("snippets/lucu.json");
  for (const key of ["App skeleton", "Button", "State block", "Events block"]) {
    assert.ok(snippets[key], `snippet present: ${key}`);
    assert.ok(snippets[key].prefix);
    assert.ok(snippets[key].body);
  }
});
