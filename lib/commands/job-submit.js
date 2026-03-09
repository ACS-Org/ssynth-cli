// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import fs from "node:fs";
import path from "node:path";
import chalk from "chalk";
import { loadHwBuild } from "../hwbuild.js";
import { uploadSource } from "../upload.js";
import { parseDuration, parseMemory } from "../parse.js";
import { kvTable, printOutput, formatTime } from "../output.js";
import { streamWsLogs } from "./job-logs.js";

export async function jobSubmitCommand(sourcePath, opts, client, config, json) {
  const absPath = path.resolve(sourcePath);

  const hwbuildDir = fs.statSync(absPath).isDirectory()
    ? absPath
    : path.dirname(absPath);
  const hwbuild = loadHwBuild(hwbuildDir);

  const targetId = await resolveTarget(opts.target, hwbuild, client);
  const topModule = opts.top || hwbuild?.top_module || "top";
  const projectId = resolveProjectId(opts.project, config);

  const sourceKey = await uploadSource(client, absPath);
  const req = buildJobRequest(opts, hwbuild, targetId, topModule, sourceKey);

  process.stderr.write("Submitting job...\n");
  const resp = await client.post(`/v1/projects/${projectId}/jobs`, req);
  const job = await resp.json();

  printOutput(json, job, (j) => {
    return kvTable([
      ["Job ID", j.id],
      ["Status", j.status],
      ["Top Module", j.top_module],
      ["Seeds", String(j.search_seeds)],
      ["Pick", j.search_pick],
      ["Priority", j.compute_priority],
      ["Created", formatTime(j.created_at)],
    ]);
  });

  if (!json) {
    console.error();
    console.error(`  ${chalk.dim("View status:")} ssynth job status ${job.id}`);
    console.error(`  ${chalk.dim("Stream logs:")} ssynth job logs ${job.id} --follow`);
  }

  if (opts.wait) {
    console.error();
    console.error(chalk.dim("Waiting for job to complete..."));
    await streamWsLogs(job.id, client, json);
  }
}

function resolveProjectId(cliProject, config) {
  const projectId = cliProject || process.env.SSYNTH_PROJECT || config.defaults?.project_id;
  if (!projectId) {
    throw new Error(
      "--project is required (set via flag, SSYNTH_PROJECT env, or defaults.project_id in config)"
    );
  }
  return projectId;
}

async function resolveTarget(cliTarget, hwbuild, client) {
  if (cliTarget) {
    return cliTarget;
  }

  const spec = hwbuild?.target;
  if (spec) {
    const resp = await client.get("/v1/targets");
    const targets = await resp.json();

    for (const t of targets) {
      const familyMatch = !spec.family || t.family.toLowerCase() === spec.family.toLowerCase();
      const deviceMatch = !spec.device || t.device.toLowerCase() === spec.device.toLowerCase();
      const packageMatch = !spec.package || (t.package && t.package.toLowerCase() === spec.package.toLowerCase());
      const boardMatch = !spec.board || (t.board && t.board.toLowerCase() === spec.board.toLowerCase());

      if (familyMatch && deviceMatch && packageMatch && boardMatch) {
        process.stderr.write(`  Matched target: ${t.family} ${t.device} (${t.id})\n`);
        return t.id;
      }
    }

    throw new Error(
      `No matching target found for ${JSON.stringify(spec)}. Use --target to specify explicitly.`
    );
  }

  throw new Error("No target specified. Use --target ID or add target spec to hwbuild.yml.");
}

function buildJobRequest(opts, hwbuild, targetId, topModule, sourceKey) {
  const constraints = opts.constraints || hwbuild?.constraints || undefined;
  const steps = opts.steps
    ? opts.steps.split(",").map((s) => s.trim())
    : hwbuild?.steps || undefined;

  const maxRuntimeStr = opts.maxRuntime || hwbuild?.max_runtime;
  const maxRuntimeSecs = maxRuntimeStr ? parseDuration(maxRuntimeStr) : undefined;

  const maxMemoryStr = opts.maxMemory || hwbuild?.max_memory;
  const maxMemoryMb = maxMemoryStr ? parseMemory(maxMemoryStr) : undefined;

  const req = {
    target_id: targetId,
    source_type: "upload",
    source_upload_key: sourceKey,
    top_module: topModule,
  };

  if (constraints) { req.constraint_files = constraints; }

  const seeds = opts.seeds !== undefined ? Number(opts.seeds) : hwbuild?.seeds;
  if (seeds !== undefined) { req.search_seeds = seeds; }

  const pick = opts.pick || hwbuild?.pick;
  if (pick) { req.search_pick = pick; }

  const parallelism = opts.parallelism !== undefined ? Number(opts.parallelism) : hwbuild?.parallelism;
  if (parallelism !== undefined) { req.compute_parallelism = parallelism; }

  const priority = opts.priority || hwbuild?.priority;
  if (priority) { req.compute_priority = priority; }

  if (steps) { req.requested_steps = steps; }
  if (opts.idempotencyKey) { req.idempotency_key = opts.idempotencyKey; }
  if (maxRuntimeSecs !== undefined) { req.max_runtime_secs = maxRuntimeSecs; }
  if (maxMemoryMb !== undefined) { req.max_memory_mb = maxMemoryMb; }
  if (opts.archiveFormat) { req.archive_format = opts.archiveFormat; }

  return req;
}
