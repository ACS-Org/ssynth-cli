// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { createRequire } from "node:module";
import { Command } from "commander";
import chalk from "chalk";
import { loadConfig, resolveAuthToken, effectiveApiUrl, requireTenant } from "./config.js";
import { ApiClient } from "./client.js";
import { CliError } from "./error.js";

import { loginCommand } from "./commands/login.js";
import { configShowCommand } from "./commands/config-show.js";
import { jobSubmitCommand } from "./commands/job-submit.js";
import { jobStatusCommand } from "./commands/job-status.js";
import { jobListCommand } from "./commands/job-list.js";
import { jobLogsCommand } from "./commands/job-logs.js";
import { jobCancelCommand } from "./commands/job-cancel.js";
import { jobRetryCommand } from "./commands/job-retry.js";
import { jobCloneCommand } from "./commands/job-clone.js";
import { artifactListCommand } from "./commands/artifact-list.js";
import { artifactDownloadCommand } from "./commands/artifact-download.js";
import {
  projectListCommand,
  projectCreateCommand,
  projectGetCommand,
  projectUpdateCommand,
  projectDeleteCommand,
} from "./commands/project.js";
import { apiKeyCreateCommand, apiKeyListCommand, apiKeyRevokeCommand } from "./commands/api-key.js";
import { targetsCommand } from "./commands/targets.js";
import { usageCommand } from "./commands/usage.js";

const require = createRequire(import.meta.url);
const { version } = require("../package.json");

export function run() {
  const program = new Command();

  program
    .name("ssynth")
    .description("CLI for SuperSynth FPGA synthesis platform")
    .version(version)
    .option("--json", "Output as JSON instead of human-readable tables")
    .option("--api-url <url>", "Override the API URL");

  // ── Login ──
  program
    .command("login")
    .description("Authenticate with the SuperSynth API")
    .option("--api-key <key>", "API key (sk_live_...)")
    .option("--dev", "Use dev-login instead of API key auth")
    .option("--username <name>", "Username for dev-login")
    .action(wrapAction(async (opts) => {
      const config = loadConfig();
      const apiUrl = effectiveApiUrl(config, program.opts().apiUrl);
      await loginCommand(opts, config, apiUrl);
    }));

  // ── Config ──
  const configCmd = program
    .command("config")
    .description("Manage configuration");

  configCmd
    .command("show")
    .description("Show current configuration")
    .action(wrapAction(async () => {
      const config = loadConfig();
      const apiUrl = effectiveApiUrl(config, program.opts().apiUrl);
      configShowCommand(config, apiUrl, program.opts().json);
    }));

  // ── Job ──
  const jobCmd = program
    .command("job")
    .description("Manage synthesis jobs");

  jobCmd
    .command("submit <path>")
    .description("Submit a new synthesis job")
    .option("--project <id>", "Project ID")
    .option("--target <id>", "Target ID")
    .option("--top <name>", "Top module name")
    .option("--constraints <files...>", "Constraint files")
    .option("--seeds <n>", "Number of seeds to search")
    .option("--pick <strategy>", "Seed selection: best_timing or best_area")
    .option("--priority <level>", "Compute priority: interactive, standard, or batch")
    .option("--parallelism <n>", "Parallelism level")
    .option("--steps <list>", "Pipeline steps (comma-delimited)")
    .option("--max-runtime <dur>", "Max runtime (e.g., 2h, 30m)")
    .option("--max-memory <size>", "Max memory (e.g., 16GB, 4096MB)")
    .option("--archive-format <fmt>", "Archive format: tar_gz or zip")
    .option("--wait", "Wait for job to complete, streaming logs")
    .option("--idempotency-key <key>", "Idempotency key for deduplication")
    .action(wrapAction(async (sourcePath, opts) => {
      const { client, config, json } = authedContext(program);
      await jobSubmitCommand(sourcePath, opts, client, config, json);
    }));

  jobCmd
    .command("status <job-id>")
    .description("Show job status and run details")
    .option("--watch", "Watch mode: refresh every 5 seconds")
    .action(wrapAction(async (jobId, opts) => {
      const { client, json } = authedContext(program);
      await jobStatusCommand(jobId, opts, client, json);
    }));

  jobCmd
    .command("list")
    .description("List jobs")
    .option("--status <status>", "Filter by status")
    .option("--project <id>", "Filter by project ID")
    .option("--limit <n>", "Maximum number of results")
    .action(wrapAction(async (opts) => {
      const { client, config, json } = authedContext(program);
      const tenantId = requireTenant(config);
      await jobListCommand(opts, client, tenantId, json);
    }));

  jobCmd
    .command("logs <job-id>")
    .description("View job logs")
    .option("--follow", "Follow log output via WebSocket")
    .option("--offset <n>", "Starting line offset")
    .option("--limit <n>", "Maximum number of lines")
    .action(wrapAction(async (jobId, opts) => {
      const { client, json } = authedContext(program);
      await jobLogsCommand(jobId, opts, client, json);
    }));

  jobCmd
    .command("cancel <job-id>")
    .description("Cancel a running job")
    .action(wrapAction(async (jobId) => {
      const { client, json } = authedContext(program);
      await jobCancelCommand(jobId, client, json);
    }));

  jobCmd
    .command("retry <job-id>")
    .description("Retry failed (or all) seeds of a job")
    .option("--scope <scope>", "Retry scope: failed or all", "failed")
    .action(wrapAction(async (jobId, opts) => {
      const { client, json } = authedContext(program);
      await jobRetryCommand(jobId, opts, client, json);
    }));

  jobCmd
    .command("clone <job-id>")
    .description("Clone a job with optional parameter overrides")
    .option("--seeds <n>", "Number of seeds")
    .option("--parallelism <n>", "Parallelism level")
    .option("--priority <level>", "Compute priority")
    .option("--pick <strategy>", "Seed selection strategy")
    .option("--target <id>", "Target ID")
    .action(wrapAction(async (jobId, opts) => {
      const { client, json } = authedContext(program);
      await jobCloneCommand(jobId, opts, client, json);
    }));

  // ── Artifact ──
  const artifactCmd = program
    .command("artifact")
    .description("Manage build artifacts");

  artifactCmd
    .command("list <job-id>")
    .description("List artifacts for a job")
    .action(wrapAction(async (jobId) => {
      const { client, json } = authedContext(program);
      await artifactListCommand(jobId, client, json);
    }));

  artifactCmd
    .command("download <job-id>")
    .description("Download build archive")
    .option("--output-dir <dir>", "Output directory")
    .action(wrapAction(async (jobId, opts) => {
      const { client } = authedContext(program);
      await artifactDownloadCommand(jobId, opts, client);
    }));

  // ── Project ──
  const projectCmd = program
    .command("project")
    .description("Manage projects");

  projectCmd
    .command("list")
    .description("List projects")
    .action(wrapAction(async () => {
      const { client, config, json } = authedContext(program);
      const tenantId = requireTenant(config);
      await projectListCommand(client, tenantId, json);
    }));

  projectCmd
    .command("create")
    .description("Create a new project")
    .option("--slug <slug>", "Project slug (URL-safe identifier)")
    .option("--name <name>", "Display name")
    .option("--target <id>", "Default target ID")
    .action(wrapAction(async (opts) => {
      const { client, config, json } = authedContext(program);
      const tenantId = requireTenant(config);
      await projectCreateCommand(opts, client, tenantId, json);
    }));

  projectCmd
    .command("get <id>")
    .description("Get project details")
    .action(wrapAction(async (id) => {
      const { client, json } = authedContext(program);
      await projectGetCommand(id, client, json);
    }));

  projectCmd
    .command("update <id>")
    .description("Update a project")
    .option("--name <name>", "New display name")
    .option("--retention-days <n>", "Artifact retention days")
    .action(wrapAction(async (id, opts) => {
      const { client, json } = authedContext(program);
      await projectUpdateCommand(id, opts, client, json);
    }));

  projectCmd
    .command("delete <id>")
    .description("Delete a project")
    .action(wrapAction(async (id) => {
      const { client, json } = authedContext(program);
      await projectDeleteCommand(id, client, json);
    }));

  // ── API Key ──
  const apiKeyCmd = program
    .command("api-key")
    .description("Manage API keys");

  apiKeyCmd
    .command("create")
    .description("Create a new API key")
    .requiredOption("--name <name>", "Key name")
    .option("--expires-at <date>", "Expiration date (ISO 8601)")
    .action(wrapAction(async (opts) => {
      const { client, json } = authedContext(program);
      await apiKeyCreateCommand(opts, client, json);
    }));

  apiKeyCmd
    .command("list")
    .description("List API keys")
    .action(wrapAction(async () => {
      const { client, json } = authedContext(program);
      await apiKeyListCommand(client, json);
    }));

  apiKeyCmd
    .command("revoke <id>")
    .description("Revoke an API key")
    .action(wrapAction(async (id) => {
      const { client } = authedContext(program);
      await apiKeyRevokeCommand(id, client);
    }));

  // ── Usage ──
  program
    .command("usage")
    .description("Show credit balance and usage")
    .action(wrapAction(async () => {
      const { client, config, json } = authedContext(program);
      const tenantId = requireTenant(config);
      await usageCommand(client, tenantId, json);
    }));

  // ── Targets ──
  program
    .command("targets")
    .description("List available FPGA targets")
    .action(wrapAction(async () => {
      const { client, json } = authedContext(program);
      await targetsCommand(client, json);
    }));

  program.parse();
}

function authedContext(program) {
  const config = loadConfig();
  const json = program.opts().json || false;
  const apiUrl = effectiveApiUrl(config, program.opts().apiUrl);
  const auth = resolveAuthToken(config);
  const client = new ApiClient(apiUrl, auth);
  return { client, config, json };
}

function wrapAction(fn) {
  return async (...args) => {
    try {
      await fn(...args);
    } catch (err) {
      if (err instanceof CliError) {
        console.error(`${chalk.red.bold("Error:")} ${err.message}`);
        process.exit(err.exitCode);
      } else {
        console.error(`${chalk.red.bold("Error:")} ${err.message || err}`);
        process.exit(1);
      }
    }
  };
}
