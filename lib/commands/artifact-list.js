// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { printList, formatTime, formatBytes, shortUuid } from "../output.js";

export async function artifactListCommand(jobId, client, json) {
  const resp = await client.get(`/v1/jobs/${jobId}/artifacts`);
  const artifacts = await resp.json();

  printList(
    json,
    artifacts,
    ["ID", "TYPE", "FILENAME", "SIZE", "CREATED"],
    (a) => [
      shortUuid(a.id),
      a.kind || a.artifact_type || "-",
      a.filename,
      formatBytes(a.size_bytes),
      formatTime(a.created_at),
    ],
  );
}
