// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import chalk from "chalk";
import { CliError } from "../error.js";

export async function jobLogsCommand(jobId, opts, client, json) {
  const offset = opts.offset || 0;
  const limit = opts.limit || 1000;

  const url = `/v1/jobs/${jobId}/logs?offset=${offset}&limit=${limit}`;
  const resp = await client.get(url);
  const logs = await resp.json();

  if (json && !opts.follow) {
    console.log(JSON.stringify(logs, null, 2));
    return;
  }

  for (const line of logs) {
    if (json) {
      printLogLineJson(line);
    } else {
      printLogLine(line);
    }
  }

  if (opts.follow) {
    await streamWsLogs(jobId, client, json);
  }
}

function printLogLine(line) {
  const prefix = line.stream === "stderr"
    ? chalk.red("ERR")
    : chalk.dim("OUT");
  console.log(`${prefix} | ${line.content}`);
}

function printLogLineJson(line) {
  console.log(JSON.stringify(line));
}

export async function streamWsLogs(jobId, client, json) {
  const jwt = await resolveWsJwt(client);

  const wsUrl = client.baseUrl
    .replace("https://", "wss://")
    .replace("http://", "ws://");
  const url = `${wsUrl}/v1/jobs/${jobId}/logs/ws?token=${jwt}`;

  const ws = new WebSocket(url);

  await new Promise((resolve, reject) => {
    ws.addEventListener("open", () => {});

    ws.addEventListener("message", (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === "log") {
          const line = { line_num: msg.line_num, stream: msg.stream, content: msg.content };
          if (json) {
            printLogLineJson(line);
          } else {
            printLogLine(line);
          }
        } else if (msg.type === "status") {
          console.error();
          console.error(`${chalk.bold("Job finished:")} ${colorizeStatus(msg.status)}`);
          ws.close();
          resolve();
        }
      } catch {
        // ignore unparseable messages
      }
    });

    ws.addEventListener("close", () => {
      resolve();
    });

    ws.addEventListener("error", (err) => {
      reject(CliError.webSocket(`Failed to connect: ${err.message || err}`));
    });
  });
}

async function resolveWsJwt(client) {
  if (!client.authToken) {
    throw CliError.notAuthenticated();
  }

  if (client.authToken.type === "jwt") {
    return client.authToken.value;
  }

  const resp = await client.post("/v1/auth/token-exchange");
  const data = await resp.json();
  return data.access_token;
}

function colorizeStatus(status) {
  if (status === "completed") { return chalk.green.bold(status); }
  if (status === "failed") { return chalk.red.bold(status); }
  if (status === "cancelled") { return chalk.yellow(status); }
  return status;
}
