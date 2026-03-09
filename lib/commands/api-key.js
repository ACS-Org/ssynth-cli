// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { kvTable, printOutput, printList, formatTime } from "../output.js";

export async function apiKeyCreateCommand(opts, client, json) {
  const req = { name: opts.name };
  if (opts.expiresAt) { req.expires_at = opts.expiresAt; }

  const resp = await client.post("/v1/api-keys", req);
  const key = await resp.json();

  printOutput(json, key, (k) => {
    const rows = [
      ["ID", k.id],
      ["Name", k.name],
      ["Key", k.key],
      ["Prefix", k.prefix],
    ];
    if (k.expires_at) {
      rows.push(["Expires", formatTime(k.expires_at)]);
    }
    return kvTable(rows);
  });

  console.error(`\n${chalk.yellow("Save this key now — it will not be shown again.")}`);
}

export async function apiKeyListCommand(client, json) {
  const resp = await client.get("/v1/api-keys");
  const keys = await resp.json();

  printList(
    json,
    keys,
    ["ID", "NAME", "PREFIX", "EXPIRES", "STATUS", "CREATED"],
    (k) => [
      k.id,
      k.name,
      k.prefix,
      k.expires_at ? formatTime(k.expires_at) : "-",
      k.is_revoked ? "revoked" : "active",
      formatTime(k.created_at),
    ],
  );
}

export async function apiKeyRevokeCommand(id, client) {
  await client.delete(`/v1/api-keys/${id}`);
  console.error(chalk.yellow("API key revoked."));
}
