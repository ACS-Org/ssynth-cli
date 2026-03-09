// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { printList, shortUuid } from "../output.js";

export async function targetsCommand(client, json) {
  const resp = await client.get("/v1/targets");
  const targets = await resp.json();

  printList(
    json,
    targets,
    ["ID", "FAMILY", "DEVICE", "PACKAGE", "BOARD", "LANE"],
    (t) => [
      shortUuid(t.id),
      t.family,
      t.device,
      t.package || "-",
      t.board || "-",
      t.toolchain_lane,
    ],
  );
}
