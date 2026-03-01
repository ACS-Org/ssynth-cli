use anyhow::{Context, Result};

use crate::cli::JobListArgs;
use crate::client::{check_response, ApiClient};
use crate::models::{Job, PageResponse};
use crate::output::{format_time, print_list, short_uuid, OutputMode};

pub async fn run(
    args: &JobListArgs,
    client: &ApiClient,
    tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    let mut url = format!("/v1/tenants/{tenant_id}/jobs");
    let mut params = Vec::new();

    if let Some(ref status) = args.status {
        params.push(format!("status={status}"));
    }
    if let Some(ref project_id) = args.project {
        params.push(format!("project_id={project_id}"));
    }
    if let Some(limit) = args.limit {
        params.push(format!("limit={limit}"));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to list jobs")?;
    let resp = check_response(resp).await?;
    let page: PageResponse<Job> = resp.json().await.context("Failed to parse jobs")?;

    print_list(
        mode,
        &page.data,
        &["ID", "STATUS", "MODULE", "SEEDS", "PRIORITY", "CREATED"],
        |j| {
            vec![
                short_uuid(&j.id),
                j.status.clone(),
                j.top_module.clone(),
                j.search_seeds.to_string(),
                j.compute_priority.clone(),
                format_time(&j.created_at),
            ]
        },
    );

    if page.has_more {
        if let Some(ref cursor) = page.next_cursor {
            eprintln!("\nMore results available. Use --after={cursor} to see next page.");
        }
    }

    Ok(())
}
