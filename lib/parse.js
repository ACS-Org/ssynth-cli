// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

export function parseDuration(s) {
  const input = s.trim().toLowerCase();
  if (input.length === 0) {
    throw new Error("empty duration string");
  }

  let totalSecs = 0;
  let numBuf = "";
  let foundUnit = false;

  for (const ch of input) {
    if (ch >= "0" && ch <= "9") {
      numBuf += ch;
    } else {
      if (numBuf.length === 0) {
        throw new Error(`unexpected '${ch}' in duration: "${s}"`);
      }
      const n = parseInt(numBuf, 10);
      numBuf = "";
      foundUnit = true;
      if (ch === "h") {
        totalSecs += n * 3600;
      } else if (ch === "m") {
        totalSecs += n * 60;
      } else if (ch === "s") {
        totalSecs += n;
      } else {
        throw new Error(`unknown duration unit '${ch}' in "${s}"; use h/m/s`);
      }
    }
  }

  if (numBuf.length > 0) {
    if (foundUnit) {
      throw new Error(`trailing digits without unit in "${s}"`);
    }
    totalSecs = parseInt(numBuf, 10);
  }

  return totalSecs;
}

export function parseMemory(s) {
  const input = s.trim();
  if (input.length === 0) {
    throw new Error("empty memory string");
  }

  const lower = input.toLowerCase();

  if (lower.endsWith("gb")) {
    const n = parseInt(lower.slice(0, -2), 10);
    if (isNaN(n)) {
      throw new Error(`invalid memory: "${s}"`);
    }
    return n * 1024;
  }

  if (lower.endsWith("mb")) {
    const n = parseInt(lower.slice(0, -2), 10);
    if (isNaN(n)) {
      throw new Error(`invalid memory: "${s}"`);
    }
    return n;
  }

  throw new Error(`unknown memory unit in "${s}"; use GB or MB (e.g., "16GB", "4096MB")`);
}
