// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { CliError } from "./error.js";

const DEFAULT_API_URL = "https://api.supersynth.ai";

export function configDir() {
  const xdg = process.env.XDG_CONFIG_HOME;
  const base = xdg || path.join(os.homedir(), ".config");
  return path.join(base, "ssynth");
}

export function configPath() {
  return path.join(configDir(), "config.json");
}

export function loadConfig() {
  const p = configPath();
  if (!fs.existsSync(p)) {
    return { api_url: DEFAULT_API_URL, auth: {}, defaults: {} };
  }
  const raw = fs.readFileSync(p, "utf8");
  let config;
  try {
    config = JSON.parse(raw);
  } catch {
    throw CliError.config(`Config file is not valid JSON: ${p}. Delete it and run 'ssynth login'.`);
  }
  config.auth = config.auth || {};
  config.defaults = config.defaults || {};
  config.api_url = config.api_url || DEFAULT_API_URL;
  return config;
}

export function saveConfig(config) {
  const dir = configDir();
  fs.mkdirSync(dir, { recursive: true });
  const p = configPath();
  fs.writeFileSync(p, JSON.stringify(config, null, 2) + "\n", { mode: 0o600 });
}

export function resolveAuthToken(config) {
  const envKey = process.env.SSYNTH_API_KEY;
  if (envKey) {
    return { type: "api-key", value: envKey };
  }
  if (config.auth.api_key) {
    return { type: "api-key", value: config.auth.api_key };
  }
  if (config.auth.dev_token) {
    if (config.auth.dev_token_expires) {
      const exp = new Date(config.auth.dev_token_expires);
      if (exp < new Date()) {
        throw CliError.notAuthenticated();
      }
    }
    return { type: "jwt", value: config.auth.dev_token };
  }
  throw CliError.notAuthenticated();
}

export function effectiveApiUrl(config, cliFlag) {
  if (cliFlag) {
    return cliFlag;
  }
  const envUrl = process.env.SSYNTH_API_URL;
  if (envUrl) {
    return envUrl;
  }
  return config.api_url || DEFAULT_API_URL;
}

export function effectiveTenantId(config) {
  return config.defaults?.tenant_id || config.auth?.tenant_id || null;
}

export function requireTenant(config) {
  const tid = effectiveTenantId(config);
  if (!tid) {
    throw CliError.config("No tenant ID configured. Run `ssynth login` first.");
  }
  return tid;
}
