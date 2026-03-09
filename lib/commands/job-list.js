// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { printList, formatTime, shortUuid } from "../output.js";

export async function jobListCommand(opts, client, tenantId, json) {
  const params = new URLSearchParams();
  if (opts.status) { params.set("status", opts.status); }
  if (opts.project) { params.set("project_id", opts.project); }
  if (opts.limit) { params.set("limit", opts.limit); }

  const qs = params.toString();
  const url = `/v1/tenants/${tenantId}/jobs${qs ? "?" + qs : ""}`;

  const resp = await client.get(url);
  const page = await resp.json();

  printList(
    json,
    page.data,
    ["ID", "STATUS", "MODULE", "SEEDS", "PRIORITY", "CREATED"],
    (j) => [
      shortUuid(j.id),
      j.status,
      j.top_module,
      String(j.search_seeds),
      j.compute_priority,
      formatTime(j.created_at),
    ],
  );

  if (page.has_more && page.next_cursor) {
    console.error(`\nMore results available. Use --after=${page.next_cursor} to see next page.`);
  }
}
