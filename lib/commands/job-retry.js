// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { kvTable, printOutput } from "../output.js";

export async function jobRetryCommand(jobId, opts, client, json) {
  const scope = opts.scope || "failed";
  const resp = await client.post(`/v1/jobs/${jobId}/retry`, { scope });
  const job = await resp.json();

  printOutput(json, job, (j) => {
    return kvTable([
      ["Job ID", j.id],
      ["Status", j.status],
      ["Runs", String(j.runs ? j.runs.length : 0)],
    ]);
  });

  console.error(`${chalk.green.bold("OK")} Retry submitted (scope: ${scope}).`);
}
