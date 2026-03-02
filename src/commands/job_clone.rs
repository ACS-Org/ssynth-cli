// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::JobCloneArgs;
use crate::client::{check_response, ApiClient};
use crate::models::{CloneJobRequest, JobDetailResponse};
use crate::output::{format_time, new_table, print_output, OutputMode};

pub async fn run(args: &JobCloneArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let target_id = args
        .target
        .as_deref()
        .map(str::parse::<uuid::Uuid>)
        .transpose()
        .context("Invalid target ID")?;

    let req = CloneJobRequest {
        target_id,
        search_seeds: args.seeds,
        search_pick: args.pick.clone(),
        compute_parallelism: args.parallelism,
        compute_priority: args.priority.clone(),
        extra_args: None,
    };

    let resp = client
        .post(&format!("/v1/jobs/{}/clone", args.job_id))
        .json(&req)
        .send()
        .await
        .context("Failed to clone job")?;
    let resp = check_response(resp).await?;
    let job: JobDetailResponse = resp.json().await.context("Failed to parse job")?;

    print_output(mode, &job, |j| {
        let mut table = new_table();
        table.set_header(["FIELD", "VALUE"]);
        table.add_row(["Job ID", &j.id.to_string()]);
        table.add_row(["Status", &j.status]);
        table.add_row(["Seeds", &j.search_seeds.to_string()]);
        table.add_row(["Priority", &j.compute_priority]);
        if let Some(ref parent) = j.parent_job_id {
            table.add_row(["Parent Job", &parent.to_string()]);
        }
        table.add_row(["Created", &format_time(&j.created_at)]);
        table
    });

    eprintln!("{} Job cloned successfully.", "OK".green().bold());

    Ok(())
}
