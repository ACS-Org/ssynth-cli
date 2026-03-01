use anyhow::{Context, Result};

use crate::client::{check_response, ApiClient};
use crate::models::Target;
use crate::output::{print_list, short_uuid, OutputMode};

pub async fn run(client: &ApiClient, mode: OutputMode) -> Result<()> {
    let resp = client
        .get("/v1/targets")
        .send()
        .await
        .context("Failed to list targets")?;
    let resp = check_response(resp).await?;
    let targets: Vec<Target> = resp.json().await.context("Failed to parse targets")?;

    print_list(
        mode,
        &targets,
        &["ID", "FAMILY", "DEVICE", "PACKAGE", "BOARD", "LANE"],
        |t| {
            vec![
                short_uuid(&t.id),
                t.family.clone(),
                t.device.clone(),
                t.package.clone().unwrap_or_else(|| "-".into()),
                t.board.clone().unwrap_or_else(|| "-".into()),
                t.toolchain_lane.clone(),
            ]
        },
    );

    Ok(())
}
