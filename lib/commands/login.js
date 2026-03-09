// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { ApiClient } from "../client.js";
import { saveConfig } from "../config.js";

export async function loginCommand(opts, config, apiUrl) {
  if (opts.dev) {
    await loginDev(opts, config, apiUrl);
  } else {
    await loginApiKey(opts, config, apiUrl);
  }
}

async function loginApiKey(opts, config, apiUrl) {
  let key = opts.apiKey;
  if (!key) {
    key = await promptPassword("API Key: ");
  }

  if (!key.startsWith("sk_live_")) {
    throw new Error("Invalid API key format. Keys must start with 'sk_live_'.");
  }

  const client = new ApiClient(apiUrl, { type: "api-key", value: key });
  const resp = await client.get("/v1/tenants");
  const page = await resp.json();
  const tenants = page.data;

  config.auth = {
    api_key: key,
  };
  if (tenants.length > 0) {
    config.auth.tenant_id = tenants[0].id;
    config.defaults = config.defaults || {};
    config.defaults.tenant_id = tenants[0].id;
  }
  saveConfig(config);

  console.error(chalk.green.bold("Login successful!"));
  if (tenants.length > 0) {
    console.error(`  Tenant: ${tenants[0].display_name} (${tenants[0].slug})`);
    console.error(`  ID:     ${tenants[0].id}`);
  }
}

async function loginDev(opts, config, apiUrl) {
  const username = opts.username || "dev";

  const client = new ApiClient(apiUrl, null);
  const resp = await client.post("/v1/auth/dev-login", { username });
  const login = await resp.json();

  const expires = new Date(Date.now() + 3600 * 1000).toISOString();

  config.auth = {
    dev_token: login.access_token,
    dev_token_expires: expires,
    tenant_id: login.user.tenant_id,
  };
  config.defaults = config.defaults || {};
  config.defaults.tenant_id = login.user.tenant_id;
  saveConfig(config);

  console.error(chalk.green.bold("Dev login successful!"));
  console.error(`  User:   ${login.user.username} (${login.user.id})`);
  console.error(`  Tenant: ${login.user.tenant_id}`);
}

function promptPassword(prompt) {
  return new Promise((resolve, reject) => {
    process.stderr.write(prompt);
    const stdin = process.stdin;
    const wasRaw = stdin.isRaw;
    if (stdin.isTTY) {
      stdin.setRawMode(true);
    }
    stdin.resume();
    stdin.setEncoding("utf8");

    let input = "";
    const onData = (ch) => {
      if (ch === "\n" || ch === "\r" || ch === "\u0004") {
        if (stdin.isTTY) {
          stdin.setRawMode(wasRaw || false);
        }
        stdin.pause();
        stdin.removeListener("data", onData);
        process.stderr.write("\n");
        resolve(input);
      } else if (ch === "\u0003") {
        if (stdin.isTTY) {
          stdin.setRawMode(wasRaw || false);
        }
        stdin.pause();
        stdin.removeListener("data", onData);
        reject(new Error("Cancelled"));
      } else if (ch === "\u007f" || ch === "\b") {
        if (input.length > 0) {
          input = input.slice(0, -1);
        }
      } else {
        input += ch;
      }
    };
    stdin.on("data", onData);
  });
}
