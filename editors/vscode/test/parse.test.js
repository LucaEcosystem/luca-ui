// Unit tests for the pure helpers in extension.js (no vscode API needed).
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { parseDiagnostics, findOnPath, resolveCli, buildRunCommand } = require("../extension.js");

test("parses Luca UI error locations", () => {
  const output = [
    "App (1:1) id=\"app\"",
    "Error at 3:3: Unknown property 'count' on component 'App' (supports: disabled, id, label, visible)",
  ].join("\n");
  assert.deepEqual(parseDiagnostics(output), [
    {
      kind: "error",
      line: 3,
      column: 3,
      message: "Unknown property 'count' on component 'App' (supports: disabled, id, label, visible)",
    },
  ]);
});

test("ignores ordinary CLI output", () => {
  assert.deepEqual(parseDiagnostics('.id = "app"\n  Text id="hi"\n'), []);
});

test("findOnPath finds fixtures and misses absent names", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lucu-ext-"));
  const exeName = process.platform === "win32" ? "luca-ui.cmd" : "luca-ui";
  const exePath = path.join(dir, exeName);
  fs.writeFileSync(exePath, process.platform === "win32" ? "@echo off\r\n" : "#!/bin/sh\n");
  if (process.platform !== "win32") {
    fs.chmodSync(exePath, 0o755);
  }
  assert.equal(findOnPath("luca-ui", dir), exePath);
  assert.equal(findOnPath("definitely-not-luca-ui", dir), null);
});

test("resolveCli honors absolute paths and PATH lookup", () => {
  assert.equal(resolveCli("/no/such/luca-ui-binary"), null);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lucu-ext-cli-"));
  const exeName = process.platform === "win32" ? "luca-ui.cmd" : "luca-ui";
  const exePath = path.join(dir, exeName);
  fs.writeFileSync(exePath, process.platform === "win32" ? "@echo off\r\n" : "#!/bin/sh\n");
  if (process.platform !== "win32") {
    fs.chmodSync(exePath, 0o755);
  }
  const savedPath = process.env.PATH;
  process.env.PATH = dir;
  try {
    assert.equal(resolveCli("luca-ui"), exePath);
    assert.equal(resolveCli("  "), exePath); // blank falls back to "luca-ui"
  } finally {
    process.env.PATH = savedPath;
  }
});

test("buildRunCommand quotes paths with spaces", () => {
  assert.equal(
    buildRunCommand("/Users/luca/.cargo/bin/luca-ui", "/tmp/my dir/hello.lucu"),
    '"/Users/luca/.cargo/bin/luca-ui" "/tmp/my dir/hello.lucu"'
  );
});
