// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import fs from "node:fs";
import path from "node:path";
import chalk from "chalk";
import { formatBytes } from "../output.js";

export async function artifactDownloadCommand(jobId, opts, client) {
  const outputDir = opts.outputDir || ".";
  fs.mkdirSync(outputDir, { recursive: true });

  process.stderr.write(`${chalk.dim("Downloading")} build archive...\n`);

  const resp = await client.get(`/v1/jobs/${jobId}/archive`);

  const disposition = resp.headers.get("content-disposition") || "";
  let filename = `${jobId}.tar.gz`;
  const match = disposition.match(/filename="?([^"]+)"?/);
  if (match) {
    filename = path.basename(match[1]);
  }

  const buffer = Buffer.from(await resp.arrayBuffer());
  const outPath = path.join(outputDir, filename);
  fs.writeFileSync(outPath, buffer);

  console.error(`${chalk.green("Downloaded")} ${outPath} (${formatBytes(buffer.length)})`);
}
