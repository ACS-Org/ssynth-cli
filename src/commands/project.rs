// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::cli::{
    ProjectCreateArgs, ProjectDeleteArgs, ProjectGetArgs, ProjectListArgs, ProjectUpdateArgs,
};
use crate::client::{check_response, ApiClient};
use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};
use crate::output::{format_time, new_table, print_list, print_output, short_uuid, OutputMode};

pub async fn list(
    _args: &ProjectListArgs,
    client: &ApiClient,
    tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    let resp = client
        .get(&format!("/v1/tenants/{tenant_id}/projects"))
        .send()
        .await
        .context("Failed to list projects")?;
    let resp = check_response(resp).await?;
    let projects: Vec<Project> = resp.json().await.context("Failed to parse projects")?;

    print_list(
        mode,
        &projects,
        &["ID", "SLUG", "NAME", "TARGET", "CREATED"],
        |p| {
            vec![
                short_uuid(&p.id),
                p.slug.clone(),
                p.display_name.clone(),
                p.default_target_id
                    .as_ref()
                    .map_or_else(|| "-".to_string(), short_uuid),
                format_time(&p.created_at),
            ]
        },
    );

    Ok(())
}

pub async fn create(
    args: &ProjectCreateArgs,
    client: &ApiClient,
    tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    if args.slug.is_empty() {
        bail!("--slug is required");
    }
    if args.name.is_empty() {
        bail!("--name is required");
    }

    let target_id = args
        .target
        .as_deref()
        .map(str::parse::<uuid::Uuid>)
        .transpose()
        .context("Invalid target ID")?;

    let req = CreateProjectRequest {
        slug: args.slug.clone(),
        display_name: args.name.clone(),
        default_target_id: target_id,
    };

    let resp = client
        .post(&format!("/v1/tenants/{tenant_id}/projects"))
        .json(&req)
        .send()
        .await
        .context("Failed to create project")?;
    let resp = check_response(resp).await?;
    let project: Project = resp.json().await.context("Failed to parse project")?;

    print_project(mode, &project);

    Ok(())
}

fn print_project(mode: OutputMode, project: &Project) {
    print_output(mode, project, |p| {
        let mut table = new_table();
        table.set_header(["FIELD", "VALUE"]);
        table.add_row(["ID", &p.id.to_string()]);
        table.add_row(["Slug", &p.slug]);
        table.add_row(["Name", &p.display_name]);
        table.add_row([
            "Target",
            &p.default_target_id
                .map_or("-".to_string(), |t| t.to_string()),
        ]);
        table.add_row(["Retention", &format!("{} days", p.retention_days)]);
        table.add_row(["Created", &format_time(&p.created_at)]);
        table
    });
}

pub async fn get(args: &ProjectGetArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let resp = client
        .get(&format!("/v1/projects/{}", args.id))
        .send()
        .await
        .context("Failed to fetch project")?;
    let resp = check_response(resp).await?;
    let project: Project = resp.json().await.context("Failed to parse project")?;

    print_project(mode, &project);

    Ok(())
}

pub async fn update(args: &ProjectUpdateArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let req = UpdateProjectRequest {
        display_name: args.name.clone(),
        retention_days: args.retention_days,
    };

    let resp = client
        .patch(&format!("/v1/projects/{}", args.id))
        .json(&req)
        .send()
        .await
        .context("Failed to update project")?;
    let resp = check_response(resp).await?;
    let project: Project = resp.json().await.context("Failed to parse project")?;

    print_project(mode, &project);

    Ok(())
}

pub async fn delete(args: &ProjectDeleteArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let resp = client
        .delete(&format!("/v1/projects/{}", args.id))
        .send()
        .await
        .context("Failed to delete project")?;
    let _resp = check_response(resp).await?;

    match mode {
        OutputMode::Json => {
            println!(r#"{{"deleted": true, "id": "{}"}}"#, args.id);
        }
        OutputMode::Human => {
            eprintln!("{} Project {} deleted.", "OK".green().bold(), args.id);
        }
    }

    Ok(())
}
