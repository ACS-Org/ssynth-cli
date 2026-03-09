// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { kvTable, printOutput, formatTime } from "../output.js";

export async function jobCloneCommand(jobId, opts, client, json) {
  const req = {};
  if (opts.target) { req.target_id = opts.target; }
  if (opts.seeds !== undefined) { req.search_seeds = Number(opts.seeds); }
  if (opts.pick) { req.search_pick = opts.pick; }
  if (opts.parallelism !== undefined) { req.compute_parallelism = Number(opts.parallelism); }
  if (opts.priority) { req.compute_priority = opts.priority; }

  const resp = await client.post(`/v1/jobs/${jobId}/clone`, req);
  const job = await resp.json();

  printOutput(json, job, (j) => {
    const rows = [
      ["Job ID", j.id],
      ["Status", j.status],
      ["Seeds", String(j.search_seeds)],
      ["Priority", j.compute_priority],
    ];
    if (j.parent_job_id) {
      rows.push(["Parent Job", j.parent_job_id]);
    }
    rows.push(["Created", formatTime(j.created_at)]);
    return kvTable(rows);
  });

  console.error(`${chalk.green.bold("OK")} Job cloned successfully.`);
}
