// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { parseDuration, parseMemory } from "../lib/parse.js";

describe("parseDuration", () => {
  it("parses hours", () => {
    assert.equal(parseDuration("2h"), 7200);
  });

  it("parses minutes", () => {
    assert.equal(parseDuration("30m"), 1800);
  });

  it("parses seconds", () => {
    assert.equal(parseDuration("3600s"), 3600);
  });

  it("parses combined h+m", () => {
    assert.equal(parseDuration("1h30m"), 5400);
  });

  it("parses full combo h+m+s", () => {
    assert.equal(parseDuration("1h30m45s"), 5445);
  });

  it("parses plain number as seconds", () => {
    assert.equal(parseDuration("86400"), 86400);
  });

  it("throws on empty", () => {
    assert.throws(() => parseDuration(""), /empty/);
  });

  it("throws on bad unit", () => {
    assert.throws(() => parseDuration("5x"), /unknown duration unit/);
  });

  it("throws on trailing digits with unit present", () => {
    assert.throws(() => parseDuration("1h30"), /trailing digits/);
  });
});

describe("parseMemory", () => {
  it("parses GB", () => {
    assert.equal(parseMemory("16GB"), 16384);
  });

  it("parses gb lowercase", () => {
    assert.equal(parseMemory("4gb"), 4096);
  });

  it("parses MB", () => {
    assert.equal(parseMemory("4096MB"), 4096);
  });

  it("parses mb lowercase", () => {
    assert.equal(parseMemory("512mb"), 512);
  });

  it("throws on empty", () => {
    assert.throws(() => parseMemory(""), /empty/);
  });

  it("throws on no unit", () => {
    assert.throws(() => parseMemory("1024"), /unknown memory unit/);
  });
});
