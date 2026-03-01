use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::JobRetryArgs;
use crate::client::{check_response, ApiClient};
use crate::models::{JobDetailResponse, RetryJobRequest};
use crate::output::{new_table, print_output, OutputMode};

pub async fn run(args: &JobRetryArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let req = RetryJobRequest {
        scope: args.scope.clone(),
    };

    let resp = client
        .post(&format!("/v1/jobs/{}/retry", args.job_id))
        .json(&req)
        .send()
        .await
        .context("Failed to retry job")?;
    let resp = check_response(resp).await?;
    let job: JobDetailResponse = resp.json().await.context("Failed to parse job")?;

    print_output(mode, &job, |j| {
        let mut table = new_table();
        table.set_header(["FIELD", "VALUE"]);
        table.add_row(["Job ID", &j.id.to_string()]);
        table.add_row(["Status", &j.status]);
        table.add_row(["Runs", &j.runs.len().to_string()]);
        table
    });

    eprintln!(
        "{} Retry submitted (scope: {}).",
        "OK".green().bold(),
        args.scope
    );

    Ok(())
}
