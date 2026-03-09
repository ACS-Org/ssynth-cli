// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import {
  kvTable,
  printOutput,
  formatTime,
  formatDurationSecs,
  shortUuid,
  newTable,
  formatRuntime,
  formatMemory,
} from "../output.js";

const TERMINAL_STATUSES = new Set(["completed", "failed", "cancelled", "canceled", "terminated"]);

export async function jobStatusCommand(jobId, opts, client, json) {
  while (true) {
    const job = await fetchJobDetail(jobId, client);
    const isTerminal = TERMINAL_STATUSES.has(job.status);

    printOutput(json, job, (j) => {
      const rows = [
        ["Job ID", j.id],
        ["Status", j.status],
        ["Project", shortUuid(j.project_id)],
        ["Target", shortUuid(j.target_id)],
        ["Top Module", j.top_module],
        ["Seeds", String(j.search_seeds)],
        ["Pick", j.search_pick],
        ["Priority", j.compute_priority],
      ];
      if (j.max_runtime_secs !== null && j.max_runtime_secs !== undefined) {
        rows.push(["Max Runtime", formatRuntime(j.max_runtime_secs)]);
      }
      if (j.max_memory_mb !== null && j.max_memory_mb !== undefined) {
        rows.push(["Max Memory", formatMemory(j.max_memory_mb)]);
      }
      if (j.parent_job_id) {
        rows.push(["Parent Job", j.parent_job_id]);
      }
      rows.push(["Created", formatTime(j.created_at)]);
      rows.push(["Updated", formatTime(j.updated_at)]);
      return kvTable(rows);
    });

    if (!json && job.runs && job.runs.length > 0) {
      console.error();
      const runTable = newTable();
      runTable.push(["RUN", "SEED", "STATUS", "TIMING", "LUTs", "FFs", "BRAMs", "CRIT PATH", "WINNER"]);
      for (const r of job.runs) {
        const statusDisplay = r.termination_reason
          ? `${r.status} (${r.termination_reason})`
          : r.status;
        runTable.push([
          shortUuid(r.id),
          String(r.seed),
          statusDisplay,
          r.timing_mhz !== null && r.timing_mhz !== undefined ? `${r.timing_mhz.toFixed(1)} MHz` : "-",
          r.area_luts !== null && r.area_luts !== undefined ? String(r.area_luts) : "-",
          r.area_ffs !== null && r.area_ffs !== undefined ? String(r.area_ffs) : "-",
          r.area_brams !== null && r.area_brams !== undefined ? String(r.area_brams) : "-",
          r.critical_path_ns !== null && r.critical_path_ns !== undefined ? `${r.critical_path_ns.toFixed(2)} ns` : "-",
          r.is_winner ? "yes" : "-",
        ]);

        for (const s of r.steps || []) {
          runTable.push([
            `  ${s.step_name}`,
            "",
            s.status,
            "",
            "",
            "",
            "",
            formatDurationSecs(s.duration_secs),
            "",
          ]);
        }
      }
      console.log(runTable.toString());
    }

    if (!opts.watch || isTerminal) {
      break;
    }

    await sleep(5000);
    process.stderr.write("\x1B[2J\x1B[H");
  }
}

export async function fetchJobDetail(jobId, client) {
  const resp = await client.get(`/v1/jobs/${jobId}`);
  return resp.json();
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
