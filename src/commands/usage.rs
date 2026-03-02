// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};

use crate::client::{check_response, ApiClient};
use crate::models::UsageSummary;
use crate::output::{format_time, new_table, OutputMode};

pub async fn run(client: &ApiClient, tenant_id: &str, mode: OutputMode) -> Result<()> {
    let resp = client
        .get(&format!("/v1/tenants/{tenant_id}/usage"))
        .send()
        .await
        .context("Failed to fetch usage")?;
    let resp = check_response(resp).await?;
    let usage: UsageSummary = resp.json().await.context("Failed to parse usage")?;

    match mode {
        OutputMode::Json => {
            let json = serde_json::to_string_pretty(&usage).expect("JSON serialization failed");
            println!("{json}");
        }
        OutputMode::Human => {
            let cents = usage.balance_cents;
            let dollars = f64::from(i32::try_from(cents).unwrap_or(i32::MAX)) / 100.0;
            println!("Credit Balance: ${dollars:.2}\n");

            if usage.recent_transactions.is_empty() {
                println!("No recent transactions.");
            } else {
                let mut table = new_table();
                table.set_header(["DATE", "AMOUNT", "DESCRIPTION"]);
                for tx in &usage.recent_transactions {
                    let amt_dollars =
                        f64::from(i32::try_from(tx.amount_cents).unwrap_or(i32::MAX)) / 100.0;
                    let sign = if tx.tx_type == "credit" { "+" } else { "-" };
                    table.add_row([
                        format_time(&tx.created_at),
                        format!("{sign}${amt_dollars:.2}"),
                        tx.description.clone(),
                    ]);
                }
                println!("{table}");
            }
        }
    }

    Ok(())
}
