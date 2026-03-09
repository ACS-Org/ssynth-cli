// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { kvTable, printOutput } from "../output.js";

export async function jobCancelCommand(jobId, client, json) {
  const resp = await client.post(`/v1/jobs/${jobId}/cancel`);
  const job = await resp.json();

  printOutput(json, job, (j) => {
    return kvTable([
      ["Job ID", j.id],
      ["Status", j.status],
    ]);
  });

  console.error(chalk.yellow("Job cancellation requested."));
}
