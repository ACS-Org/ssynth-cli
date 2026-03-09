// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { describe, it, afterEach } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

// We test the config module by temporarily pointing XDG_CONFIG_HOME to a temp dir
describe("config", () => {
  let tmpDir;
  let origXdg;

  afterEach(() => {
    if (tmpDir) {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
    if (origXdg !== undefined) {
      process.env.XDG_CONFIG_HOME = origXdg;
    } else {
      delete process.env.XDG_CONFIG_HOME;
    }
  });

  async function setupTmpConfig() {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ssynth-test-"));
    origXdg = process.env.XDG_CONFIG_HOME;
    process.env.XDG_CONFIG_HOME = tmpDir;
    // Dynamic import to pick up env change each time
    const mod = await import(`../lib/config.js?t=${Date.now()}`);
    return mod;
  }

  it("loadConfig returns defaults when no file exists", async () => {
    const { loadConfig } = await setupTmpConfig();
    const config = loadConfig();
    assert.equal(config.api_url, "https://api.supersynth.ai");
    assert.equal(config.auth.api_key, undefined);
  });

  it("saveConfig + loadConfig roundtrip", async () => {
    const { loadConfig, saveConfig } = await setupTmpConfig();
    const config = {
      api_url: "http://localhost:3000",
      auth: { api_key: "sk_live_abc123" },
      defaults: { tenant_id: "test-tenant" },
    };
    saveConfig(config);
    const loaded = loadConfig();
    assert.equal(loaded.api_url, "http://localhost:3000");
    assert.equal(loaded.auth.api_key, "sk_live_abc123");
    assert.equal(loaded.defaults.tenant_id, "test-tenant");
  });

  it("resolveAuthToken prefers env var", async () => {
    const { resolveAuthToken } = await setupTmpConfig();
    const origKey = process.env.SSYNTH_API_KEY;
    process.env.SSYNTH_API_KEY = "sk_live_env";
    try {
      const config = { auth: { api_key: "sk_live_config" }, defaults: {} };
      const token = resolveAuthToken(config);
      assert.equal(token.type, "api-key");
      assert.equal(token.value, "sk_live_env");
    } finally {
      if (origKey !== undefined) {
        process.env.SSYNTH_API_KEY = origKey;
      } else {
        delete process.env.SSYNTH_API_KEY;
      }
    }
  });

  it("resolveAuthToken uses config api_key", async () => {
    const { resolveAuthToken } = await setupTmpConfig();
    const origKey = process.env.SSYNTH_API_KEY;
    delete process.env.SSYNTH_API_KEY;
    try {
      const config = { auth: { api_key: "sk_live_config" }, defaults: {} };
      const token = resolveAuthToken(config);
      assert.equal(token.type, "api-key");
      assert.equal(token.value, "sk_live_config");
    } finally {
      if (origKey !== undefined) {
        process.env.SSYNTH_API_KEY = origKey;
      } else {
        delete process.env.SSYNTH_API_KEY;
      }
    }
  });

  it("resolveAuthToken throws when no auth", async () => {
    const { resolveAuthToken } = await setupTmpConfig();
    const origKey = process.env.SSYNTH_API_KEY;
    delete process.env.SSYNTH_API_KEY;
    try {
      const config = { auth: {}, defaults: {} };
      assert.throws(() => resolveAuthToken(config), /Not authenticated/);
    } finally {
      if (origKey !== undefined) {
        process.env.SSYNTH_API_KEY = origKey;
      } else {
        delete process.env.SSYNTH_API_KEY;
      }
    }
  });
});
