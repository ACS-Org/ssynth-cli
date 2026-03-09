// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { kvTable, printOutput, printList, formatTime, shortUuid } from "../output.js";

export async function profileListCommand(projectId, client, json) {
  const resp = await client.get(`/v1/projects/${projectId}/profiles`);
  const page = await resp.json();

  printList(
    json,
    page.data,
    ["ID", "NAME", "TARGET", "DESCRIPTION", "CREATED"],
    (p) => [
      shortUuid(p.id),
      p.name,
      shortUuid(p.target_id),
      p.description || "-",
      formatTime(p.created_at),
    ],
  );
}

export async function profileCreateCommand(projectId, opts, client, json) {
  if (!opts.name) { throw new Error("--name is required"); }
  if (!opts.target) { throw new Error("--target is required"); }

  const req = {
    name: opts.name,
    target_id: opts.target,
  };
  if (opts.description) { req.description = opts.description; }

  if (opts.yosysPasses) {
    req.yosys = { extra_passes: opts.yosysPasses.split(",").map((s) => s.trim()) };
  }
  if (opts.nextpnrFlags) {
    req.nextpnr = { extra_flags: opts.nextpnrFlags.split(",").map((s) => s.trim()) };
  }
  if (opts.bitstreamFlags) {
    req.bitstream = { extra_flags: opts.bitstreamFlags.split(",").map((s) => s.trim()) };
  }

  const resp = await client.post(`/v1/projects/${projectId}/profiles`, req);
  const profile = await resp.json();
  printProfile(json, profile);
}

export async function profileGetCommand(id, client, json) {
  const resp = await client.get(`/v1/profiles/${id}`);
  const profile = await resp.json();
  printProfile(json, profile);
}

export async function profileUpdateCommand(id, opts, client, json) {
  const req = {};
  if (opts.name) { req.name = opts.name; }
  if (opts.description) { req.description = opts.description; }
  if (opts.target) { req.target_id = opts.target; }

  if (opts.yosysPasses) {
    req.yosys = { extra_passes: opts.yosysPasses.split(",").map((s) => s.trim()) };
  }
  if (opts.nextpnrFlags) {
    req.nextpnr = { extra_flags: opts.nextpnrFlags.split(",").map((s) => s.trim()) };
  }
  if (opts.bitstreamFlags) {
    req.bitstream = { extra_flags: opts.bitstreamFlags.split(",").map((s) => s.trim()) };
  }

  const resp = await client.patch(`/v1/profiles/${id}`, req);
  const profile = await resp.json();
  printProfile(json, profile);
}

export async function profileDeleteCommand(id, client, json) {
  await client.delete(`/v1/profiles/${id}`);

  if (json) {
    console.log(JSON.stringify({ deleted: true, id }));
  } else {
    console.error(`${chalk.green.bold("OK")} Profile ${id} deleted.`);
  }
}

function printProfile(json, profile) {
  printOutput(json, profile, (p) => {
    return kvTable([
      ["ID", p.id],
      ["Name", p.name],
      ["Description", p.description || "-"],
      ["Target", p.target_id],
      ["Yosys Passes", formatArgs(p.yosys_args?.extra_passes)],
      ["Nextpnr Flags", formatArgs(p.nextpnr_args?.extra_flags)],
      ["Bitstream Flags", formatArgs(p.bitstream_args?.extra_flags)],
      ["Created", formatTime(p.created_at)],
      ["Updated", formatTime(p.updated_at)],
    ]);
  });
}

function formatArgs(arr) {
  if (!arr || arr.length === 0) { return "-"; }
  return arr.join(", ");
}
