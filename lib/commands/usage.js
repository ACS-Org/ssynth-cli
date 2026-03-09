// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { newTable, formatTime } from "../output.js";

export async function usageCommand(client, tenantId, json) {
  const resp = await client.get(`/v1/tenants/${tenantId}/usage`);
  const usage = await resp.json();

  if (json) {
    console.log(JSON.stringify(usage, null, 2));
    return;
  }

  const dollars = usage.balance_cents / 100;
  console.log(`Credit Balance: $${dollars.toFixed(2)}\n`);

  if (!usage.recent_transactions || usage.recent_transactions.length === 0) {
    console.log("No recent transactions.");
    return;
  }

  const table = newTable();
  table.push(["DATE", "AMOUNT", "DESCRIPTION"]);
  for (const tx of usage.recent_transactions) {
    const amtDollars = tx.amount_cents / 100;
    const sign = tx.tx_type === "credit" ? "+" : "-";
    table.push([
      formatTime(tx.created_at),
      `${sign}$${amtDollars.toFixed(2)}`,
      tx.description,
    ]);
  }
  console.log(table.toString());
}
