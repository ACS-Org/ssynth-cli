// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { describe, it, afterEach } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { buildIgnore } from "../lib/ignore.js";

describe("buildIgnore", () => {
  let tmpDir;

  afterEach(() => {
    if (tmpDir) {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("ignores .git and build by default", () => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ssynth-ignore-"));
    const ig = buildIgnore(tmpDir);
    assert.equal(ig.ignores(".git/"), true);
    assert.equal(ig.ignores("build/"), true);
    assert.equal(ig.ignores("src/main.v"), false);
  });

  it("reads .ssynthignore file", () => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ssynth-ignore-"));
    fs.writeFileSync(path.join(tmpDir, ".ssynthignore"), "*.log\ntmp/\n");
    const ig = buildIgnore(tmpDir);
    assert.equal(ig.ignores("synth.log"), true);
    assert.equal(ig.ignores("tmp/"), true);
    assert.equal(ig.ignores("design.v"), false);
  });
});
