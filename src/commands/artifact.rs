// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::{ArtifactDownloadArgs, ArtifactListArgs};
use crate::client::{check_response, ApiClient};
use crate::models::Artifact;
use crate::output::{format_bytes, format_time, print_list, short_uuid, OutputMode};

pub async fn list(args: &ArtifactListArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let resp = client
        .get(&format!("/v1/jobs/{}/artifacts", args.job_id))
        .send()
        .await
        .context("Failed to list artifacts")?;
    let resp = check_response(resp).await?;
    let artifacts: Vec<Artifact> = resp.json().await.context("Failed to parse artifacts")?;

    print_list(
        mode,
        &artifacts,
        &["ID", "TYPE", "FILENAME", "SIZE", "CREATED"],
        |a| {
            vec![
                short_uuid(&a.id),
                a.kind.clone(),
                a.filename.clone(),
                format_bytes(a.size_bytes),
                format_time(&a.created_at),
            ]
        },
    );

    Ok(())
}

pub async fn download(
    args: &ArtifactDownloadArgs,
    client: &ApiClient,
    mode: OutputMode,
) -> Result<()> {
    let output_dir = args
        .output_dir
        .as_deref()
        .map_or_else(|| PathBuf::from("."), PathBuf::from);

    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    if let Some(ref artifact_id) = args.artifact_id {
        // Download single artifact
        download_single(client, artifact_id, &output_dir).await?;
    } else {
        // List all artifacts for the job, then download each one
        let resp = client
            .get(&format!("/v1/jobs/{}/artifacts", args.job_id))
            .send()
            .await
            .context("Failed to list artifacts")?;
        let resp = check_response(resp).await?;
        let artifacts: Vec<Artifact> = resp.json().await?;

        if artifacts.is_empty() {
            match mode {
                OutputMode::Json => println!("[]"),
                OutputMode::Human => eprintln!("No artifacts to download."),
            }
            return Ok(());
        }

        for artifact in &artifacts {
            eprintln!("{} {}...", "Downloading".dimmed(), artifact.filename);
            let download_url = format!("/v1/artifacts/{}/download", artifact.id);
            let resp = client
                .get(&download_url)
                .send()
                .await
                .with_context(|| format!("Failed to download {}", artifact.filename))?;
            let resp = check_response(resp).await?;
            let bytes = resp.bytes().await?;
            let path = output_dir.join(&artifact.filename);
            std::fs::write(&path, &bytes)
                .with_context(|| format!("Failed to write {}", path.display()))?;
            eprintln!(
                "  {} {} ({})",
                "Saved".green(),
                path.display(),
                format_bytes(i64::try_from(bytes.len()).unwrap_or(i64::MAX))
            );
        }
    }

    Ok(())
}

async fn download_single(client: &ApiClient, artifact_id: &str, output_dir: &Path) -> Result<()> {
    let resp = client
        .get(&format!("/v1/artifacts/{artifact_id}/download"))
        .send()
        .await
        .context("Failed to download artifact")?;
    let resp = check_response(resp).await?;

    // Try to get filename from Content-Disposition header
    let filename = resp
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split("filename=")
                .nth(1)
                .map(|s| s.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| format!("{artifact_id}.bin"));

    let bytes = resp.bytes().await?;
    let path = output_dir.join(&filename);
    std::fs::write(&path, &bytes).with_context(|| format!("Failed to write {}", path.display()))?;
    eprintln!(
        "{} {} ({})",
        "Downloaded".green(),
        path.display(),
        format_bytes(i64::try_from(bytes.len()).unwrap_or(i64::MAX))
    );

    Ok(())
}
