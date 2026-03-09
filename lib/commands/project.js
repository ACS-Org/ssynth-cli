// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { kvTable, printOutput, printList, formatTime, shortUuid } from "../output.js";

export async function projectListCommand(client, tenantId, json) {
  const resp = await client.get(`/v1/tenants/${tenantId}/projects`);
  const page = await resp.json();

  printList(
    json,
    page.data,
    ["ID", "SLUG", "NAME", "TARGET", "CREATED"],
    (p) => [
      shortUuid(p.id),
      p.slug,
      p.display_name,
      p.default_target_id ? shortUuid(p.default_target_id) : "-",
      formatTime(p.created_at),
    ],
  );
}

export async function projectCreateCommand(opts, client, tenantId, json) {
  if (!opts.slug) { throw new Error("--slug is required"); }
  if (!opts.name) { throw new Error("--name is required"); }

  const req = {
    slug: opts.slug,
    display_name: opts.name,
  };
  if (opts.target) { req.default_target_id = opts.target; }

  const resp = await client.post(`/v1/tenants/${tenantId}/projects`, req);
  const project = await resp.json();
  printProject(json, project);
}

export async function projectGetCommand(id, client, json) {
  const resp = await client.get(`/v1/projects/${id}`);
  const project = await resp.json();
  printProject(json, project);
}

export async function projectUpdateCommand(id, opts, client, json) {
  const req = {};
  if (opts.name) { req.display_name = opts.name; }
  if (opts.retentionDays !== undefined) { req.retention_days = Number(opts.retentionDays); }

  const resp = await client.patch(`/v1/projects/${id}`, req);
  const project = await resp.json();
  printProject(json, project);
}

export async function projectDeleteCommand(id, client, json) {
  await client.delete(`/v1/projects/${id}`);

  if (json) {
    console.log(JSON.stringify({ deleted: true, id }));
  } else {
    console.error(`${chalk.green.bold("OK")} Project ${id} deleted.`);
  }
}

function printProject(json, project) {
  printOutput(json, project, (p) => {
    return kvTable([
      ["ID", p.id],
      ["Slug", p.slug],
      ["Name", p.display_name],
      ["Target", p.default_target_id || "-"],
      ["Retention", `${p.retention_days} days`],
      ["Created", formatTime(p.created_at)],
    ]);
  });
}
