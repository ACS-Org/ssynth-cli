// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import Table from "cli-table3";

export function printOutput(json, value, tableFn) {
  if (json) {
    console.log(JSON.stringify(value, null, 2));
  } else {
    const table = tableFn(value);
    console.log(table.toString());
  }
}

export function printList(json, items, headers, rowFn) {
  if (json) {
    console.log(JSON.stringify(items, null, 2));
    return;
  }
  if (items.length === 0) {
    console.log("No results.");
    return;
  }
  const table = newTable();
  table.push(headers);
  for (const item of items) {
    table.push(rowFn(item));
  }
  console.log(table.toString());
}

export function newTable() {
  return new Table({
    chars: {
      top: "\u2500", "top-mid": "\u252c", "top-left": "\u256d", "top-right": "\u256e",
      bottom: "\u2500", "bottom-mid": "\u2534", "bottom-left": "\u2570", "bottom-right": "\u256f",
      left: "\u2502", "left-mid": "\u251c", mid: "\u2500", "mid-mid": "\u253c",
      right: "\u2502", "right-mid": "\u2524", middle: "\u2502",
    },
  });
}

export function kvTable(rows) {
  const table = newTable();
  table.push(["FIELD", "VALUE"]);
  for (const [k, v] of rows) {
    table.push([k, v]);
  }
  return table;
}

export function formatTime(dt) {
  if (!dt) {
    return "-";
  }
  const d = new Date(dt);
  const pad = (n) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}:${pad(d.getUTCSeconds())}`;
}

export function formatDurationSecs(secs) {
  if (secs === null || secs === undefined) {
    return "-";
  }
  if (secs < 60) {
    return `${secs.toFixed(1)}s`;
  }
  const mins = Math.floor(secs / 60);
  const remainder = secs - mins * 60;
  return `${mins}m ${remainder.toFixed(1)}s`;
}

export function formatBytes(bytes) {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  if (bytes < KB) {
    return `${bytes} B`;
  }
  if (bytes < MB) {
    return `${(bytes / KB).toFixed(1)} KB`;
  }
  if (bytes < GB) {
    return `${(bytes / MB).toFixed(1)} MB`;
  }
  return `${(bytes / GB).toFixed(2)} GB`;
}

export function formatRuntime(secs) {
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  if (mins > 0) {
    return `${hours}h${mins}m`;
  }
  return `${hours}h`;
}

export function formatMemory(mb) {
  if (mb >= 1024 && mb % 1024 === 0) {
    return `${mb / 1024} GB`;
  }
  return `${mb} MB`;
}
