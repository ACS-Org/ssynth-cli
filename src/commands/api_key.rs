use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::{ApiKeyCreateArgs, ApiKeyRevokeArgs};
use crate::client::{check_response, ApiClient};
use crate::models::{ApiKey, CreateApiKeyRequest, CreateApiKeyResponse};
use crate::output::{format_time, new_table, print_list, print_output, OutputMode};

// tenant_id params are kept for API consistency but the api-keys endpoint
// derives tenant from the auth token, not from the URL path.

pub async fn create(
    args: &ApiKeyCreateArgs,
    client: &ApiClient,
    _tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    let req = CreateApiKeyRequest {
        name: args.name.clone(),
        expires_at: args.expires_at.clone(),
    };

    let resp = client
        .post("/v1/api-keys")
        .json(&req)
        .send()
        .await
        .context("Failed to create API key")?;
    let resp = check_response(resp).await?;
    let key: CreateApiKeyResponse = resp.json().await.context("Failed to parse API key")?;

    print_output(mode, &key, |k| {
        let mut table = new_table();
        table.set_header(["FIELD", "VALUE"]);
        table.add_row(["ID", &k.id.to_string()]);
        table.add_row(["Name", &k.name]);
        table.add_row(["Key", &k.key]);
        table.add_row(["Prefix", &k.prefix]);
        if let Some(ref exp) = k.expires_at {
            table.add_row(["Expires", &format_time(exp)]);
        }
        table
    });

    eprintln!(
        "\n{}",
        "Save this key now — it will not be shown again.".yellow()
    );

    Ok(())
}

pub async fn list(
    client: &ApiClient,
    _tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    let resp = client
        .get("/v1/api-keys")
        .send()
        .await
        .context("Failed to list API keys")?;
    let resp = check_response(resp).await?;
    let keys: Vec<ApiKey> = resp.json().await.context("Failed to parse API keys")?;

    print_list(
        mode,
        &keys,
        &["ID", "NAME", "PREFIX", "EXPIRES", "STATUS", "CREATED"],
        |k| {
            vec![
                k.id.to_string(),
                k.name.clone(),
                k.prefix.clone(),
                k.expires_at.as_ref().map_or("-".into(), format_time),
                if k.is_revoked { "revoked".into() } else { "active".into() },
                format_time(&k.created_at),
            ]
        },
    );

    Ok(())
}

pub async fn revoke(
    args: &ApiKeyRevokeArgs,
    client: &ApiClient,
    _tenant_id: &str,
    _mode: OutputMode,
) -> Result<()> {
    let resp = client
        .delete(&format!("/v1/api-keys/{}", args.id))
        .send()
        .await
        .context("Failed to revoke API key")?;
    check_response(resp).await?;

    eprintln!("{}", "API key revoked.".yellow());

    Ok(())
}
