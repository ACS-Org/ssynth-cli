// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import fs from "node:fs";
import path from "node:path";
import { create } from "tar";
import { buildIgnore } from "./ignore.js";
import { CliError } from "./error.js";

export async function createTarball(dir) {
  const ig = buildIgnore(dir);
  const files = collectFiles(dir, dir, ig);

  const chunks = [];
  const stream = create({ gzip: true, cwd: dir }, files);

  for await (const chunk of stream) {
    chunks.push(chunk);
  }

  return Buffer.concat(chunks);
}

function collectFiles(root, current, ig) {
  const entries = fs.readdirSync(current, { withFileTypes: true });
  entries.sort((a, b) => a.name.localeCompare(b.name));

  const result = [];
  for (const entry of entries) {
    const abs = path.join(current, entry.name);
    const rel = path.relative(root, abs);
    const relForIgnore = entry.isDirectory() ? rel + "/" : rel;

    if (ig.ignores(relForIgnore)) {
      continue;
    }

    if (entry.isDirectory()) {
      result.push(...collectFiles(root, abs, ig));
    } else if (entry.isFile()) {
      result.push(rel);
    }
  }
  return result;
}

export async function uploadSource(client, sourcePath) {
  const stat = fs.statSync(sourcePath, { throwIfNoEntry: false });
  if (!stat) {
    throw CliError.fileNotFound(sourcePath);
  }

  let data;
  let contentType;

  if (stat.isDirectory()) {
    process.stderr.write("Packaging source files...\n");
    data = await createTarball(sourcePath);
    const sizeKb = Math.floor(data.length / 1024);
    process.stderr.write(`  Archive size: ${sizeKb} KB\n`);
    contentType = "application/gzip";
  } else if (stat.isFile()) {
    contentType = contentTypeForFile(sourcePath);
    data = fs.readFileSync(sourcePath);
    const sizeKb = Math.floor(data.length / 1024);
    process.stderr.write(`  Bundle size: ${sizeKb} KB\n`);
  } else {
    throw CliError.fileNotFound(sourcePath);
  }

  process.stderr.write("Uploading source...\n");
  const resp = await client.postRaw("/v1/jobs/upload", data, contentType);
  const result = await resp.json();
  process.stderr.write("Upload complete.\n");
  return result.source_key;
}

function contentTypeForFile(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  if (ext === ".gz" || ext === ".tgz") {
    return "application/gzip";
  }
  if (ext === ".zip") {
    return "application/zip";
  }
  throw new Error("unsupported file format: expected .tar.gz, .tgz, or .zip");
}
