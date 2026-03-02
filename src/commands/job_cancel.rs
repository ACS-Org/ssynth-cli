// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::JobCancelArgs;
use crate::client::{check_response, ApiClient};
use crate::models::Job;
use crate::output::{new_table, print_output, OutputMode};

pub async fn run(args: &JobCancelArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let resp = client
        .post(&format!("/v1/jobs/{}/cancel", args.job_id))
        .send()
        .await
        .context("Failed to cancel job")?;
    let resp = check_response(resp).await?;
    let job: Job = resp.json().await.context("Failed to parse job")?;

    print_output(mode, &job, |j| {
        let mut table = new_table();
        table.set_header(["FIELD", "VALUE"]);
        table.add_row(["Job ID", &j.id.to_string()]);
        table.add_row(["Status", &j.status]);
        table
    });

    eprintln!("{}", "Job cancellation requested.".yellow());

    Ok(())
}
