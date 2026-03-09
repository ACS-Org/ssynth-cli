// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { configPath, effectiveTenantId } from "../config.js";
import { kvTable, printOutput } from "../output.js";

export function configShowCommand(config, apiUrl, json) {
  const redacted = buildRedacted(config, apiUrl);

  printOutput(json, redacted, (c) => {
    return kvTable([
      ["API URL", c.api_url],
      ["Auth Method", c.auth_method],
      ["Auth Value", c.auth_value],
      ["Tenant ID", c.tenant_id],
      ["Default Project", c.default_project],
      ["Config File", c.config_path],
    ]);
  });
}

function buildRedacted(config, apiUrl) {
  let authMethod;
  let authValue;

  if (process.env.SSYNTH_API_KEY) {
    authMethod = "env:SSYNTH_API_KEY";
    authValue = redactKey("sk_live_...");
  } else if (config.auth.api_key) {
    authMethod = "config:api_key";
    authValue = redactKey(config.auth.api_key);
  } else if (config.auth.dev_token) {
    const expires = config.auth.dev_token_expires || "unknown";
    authMethod = "config:dev_token";
    authValue = `(expires ${expires})`;
  } else {
    authMethod = "none";
    authValue = "-";
  }

  return {
    api_url: apiUrl,
    auth_method: authMethod,
    auth_value: authValue,
    tenant_id: effectiveTenantId(config) || "-",
    default_project: config.defaults?.project_id || "-",
    config_path: configPath(),
  };
}

function redactKey(key) {
  if (key.length > 16) {
    return `${key.slice(0, 12)}...${key.slice(-4)}`;
  }
  return "****";
}
