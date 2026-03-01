use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::cli::JobSubmitArgs;
use crate::client::{check_response, ApiClient};
use crate::error::CliError;
use crate::hwbuild::HwBuild;
use crate::ignore::build_ignore;
use crate::models::{CreateJobRequest, JobDetailResponse, Target};
use crate::output::{format_time, new_table, print_output, OutputMode};
use crate::upload::{create_tarball, upload_source as do_upload};

pub async fn run(
    args: &JobSubmitArgs,
    client: &ApiClient,
    _tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    let dir = PathBuf::from(&args.path);
    if !dir.is_dir() {
        return Err(CliError::FileNotFound(dir).into());
    }

    let hwbuild = HwBuild::load(&dir)?;
    let target_id = resolve_target(args, hwbuild.as_ref(), client).await?;
    let top_module = args
        .top
        .as_deref()
        .or(hwbuild.as_ref().and_then(|h| h.top_module.as_deref()))
        .unwrap_or("top")
        .to_string();
    let project_id = args
        .project
        .as_deref()
        .context("--project is required (no default set)")?;

    let source_key = upload_source(&dir, client).await?;
    let req = build_job_request(args, hwbuild.as_ref(), target_id, top_module, source_key);
    let job = submit_job(client, project_id, &req).await?;
    print_job_result(mode, &job, args.wait, client).await
}

async fn upload_source(dir: &std::path::Path, client: &ApiClient) -> Result<String> {
    eprintln!("{}", "Packaging source files...".dimmed());
    let ignore = build_ignore(dir);
    let tarball = create_tarball(dir, &ignore)?;
    let size_kb = tarball.len() / 1024;
    eprintln!("  Archive size: {size_kb} KB");

    eprintln!("{}", "Uploading source...".dimmed());
    let resp = do_upload(client, tarball).await?;
    eprintln!("{}", "Upload complete.".green());
    Ok(resp.source_key)
}

fn build_job_request(
    args: &JobSubmitArgs,
    hwbuild: Option<&HwBuild>,
    target_id: uuid::Uuid,
    top_module: String,
    source_key: String,
) -> CreateJobRequest {
    let constraint_files = args
        .constraints
        .clone()
        .or_else(|| hwbuild.and_then(|h| h.constraints.clone()));
    let steps = args
        .steps
        .clone()
        .or_else(|| hwbuild.and_then(|h| h.steps.clone()));

    CreateJobRequest {
        target_id,
        source_type: "upload".to_string(),
        source_upload_key: Some(source_key),
        top_module,
        constraint_files,
        extra_args: None,
        search_seeds: args.seeds.or(hwbuild.and_then(|h| h.seeds)),
        search_pick: args
            .pick
            .clone()
            .or_else(|| hwbuild.and_then(|h| h.pick.clone())),
        compute_parallelism: args.parallelism.or(hwbuild.and_then(|h| h.parallelism)),
        compute_priority: args
            .priority
            .clone()
            .or_else(|| hwbuild.and_then(|h| h.priority.clone())),
        requested_steps: steps,
        idempotency_key: args.idempotency_key.clone(),
    }
}

async fn submit_job(
    client: &ApiClient,
    project_id: &str,
    req: &CreateJobRequest,
) -> Result<JobDetailResponse> {
    eprintln!("{}", "Submitting job...".dimmed());
    let resp = client
        .post(&format!("/v1/projects/{project_id}/jobs"))
        .json(req)
        .send()
        .await
        .context("Failed to submit job")?;
    let resp = check_response(resp).await?;
    resp.json().await.context("Failed to parse job response")
}

async fn print_job_result(
    mode: OutputMode,
    job: &JobDetailResponse,
    wait: bool,
    client: &ApiClient,
) -> Result<()> {
    print_output(mode, job, |j| {
        let mut table = new_table();
        table.set_header(["FIELD", "VALUE"]);
        table.add_row(["Job ID", &j.id.to_string()]);
        table.add_row(["Status", &j.status]);
        table.add_row(["Top Module", &j.top_module]);
        table.add_row(["Seeds", &j.search_seeds.to_string()]);
        table.add_row(["Pick", &j.search_pick]);
        table.add_row(["Priority", &j.compute_priority]);
        table.add_row(["Created", &format_time(&j.created_at)]);
        table
    });

    if mode == OutputMode::Human {
        eprintln!();
        eprintln!("  {} ssynth job status {}", "View status:".dimmed(), job.id);
        eprintln!(
            "  {} ssynth job logs {} --follow",
            "Stream logs:".dimmed(),
            job.id
        );
    }

    if wait {
        eprintln!();
        eprintln!("{}", "Waiting for job to complete...".dimmed());
        let log_args = crate::cli::JobLogsArgs {
            job_id: job.id.to_string(),
            follow: true,
            offset: None,
            limit: None,
        };
        crate::commands::job_logs::run(&log_args, client, mode).await?;
    }

    Ok(())
}

async fn resolve_target(
    args: &JobSubmitArgs,
    hwbuild: Option<&HwBuild>,
    client: &ApiClient,
) -> Result<uuid::Uuid> {
    if let Some(ref tid) = args.target {
        return tid.parse::<uuid::Uuid>().context("Invalid target ID");
    }

    let hw = hwbuild.and_then(|h| h.target.as_ref());

    if let Some(spec) = hw {
        let resp = client
            .get("/v1/targets")
            .send()
            .await
            .context("Failed to list targets")?;
        let resp = check_response(resp).await?;
        let targets: Vec<Target> = resp.json().await.context("Failed to parse targets")?;

        for t in &targets {
            let family_match = spec
                .family
                .as_deref()
                .is_none_or(|f| t.family.eq_ignore_ascii_case(f));
            let device_match = spec
                .device
                .as_deref()
                .is_none_or(|d| t.device.eq_ignore_ascii_case(d));
            let package_match = spec.package.as_deref().is_none_or(|p| {
                t.package
                    .as_deref()
                    .is_some_and(|tp| tp.eq_ignore_ascii_case(p))
            });
            let board_match = spec.board.as_deref().is_none_or(|b| {
                t.board
                    .as_deref()
                    .is_some_and(|tb| tb.eq_ignore_ascii_case(b))
            });

            if family_match && device_match && package_match && board_match {
                eprintln!("  Matched target: {} {} ({})", t.family, t.device, t.id);
                return Ok(t.id);
            }
        }

        bail!("No matching target found for {spec:?}. Use --target to specify explicitly.");
    }

    bail!("No target specified. Use --target ID or add target spec to hwbuild.yml.");
}
