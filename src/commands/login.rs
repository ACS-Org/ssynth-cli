// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use colored::Colorize;

use crate::cli::LoginArgs;
use crate::client::{check_response, ApiClient};
use crate::config::{AuthConfig, AuthToken, Config};
use crate::models::{DevLoginRequest, LoginResponse, Tenant};

pub async fn run(args: &LoginArgs, config: &mut Config, api_url: &str) -> Result<()> {
    if args.dev {
        login_dev(args, config, api_url).await
    } else {
        login_api_key(args, config, api_url).await
    }
}

async fn login_api_key(args: &LoginArgs, config: &mut Config, api_url: &str) -> Result<()> {
    let key = if let Some(ref k) = args.api_key {
        k.clone()
    } else {
        rpassword::prompt_password("API Key: ").context("Failed to read API key")?
    };

    if !key.starts_with("sk_live_") {
        anyhow::bail!("Invalid API key format. Keys must start with 'sk_live_'.");
    }

    // Validate by listing tenants
    let client = ApiClient::new(api_url, Some(AuthToken::ApiKey(key.clone())))?;
    let resp = client
        .get("/v1/tenants")
        .send()
        .await
        .context("Failed to connect to API")?;
    let resp = check_response(resp).await?;
    let tenants: Vec<Tenant> = resp.json().await.context("Failed to parse tenants")?;

    config.auth = AuthConfig {
        api_key: Some(key),
        dev_token: None,
        dev_token_expires: None,
        tenant_id: tenants.first().map(|t| t.id.to_string()),
    };
    if let Some(t) = tenants.first() {
        config.defaults.tenant_id = Some(t.id.to_string());
    }
    config.save()?;

    println!("{}", "Login successful!".green().bold());
    if let Some(t) = tenants.first() {
        println!("  Tenant: {} ({})", t.display_name, t.slug);
        println!("  ID:     {}", t.id);
    }

    Ok(())
}

async fn login_dev(args: &LoginArgs, config: &mut Config, api_url: &str) -> Result<()> {
    let username = args.username.as_deref().unwrap_or("dev");

    let client = ApiClient::new(api_url, None)?;
    let resp = client
        .post("/v1/auth/dev-login")
        .json(&DevLoginRequest {
            username: username.to_string(),
        })
        .send()
        .await
        .context("Failed to connect to API")?;
    let resp = check_response(resp).await?;
    let login: LoginResponse = resp
        .json()
        .await
        .context("Failed to parse login response")?;

    let expires = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();

    config.auth = AuthConfig {
        api_key: None,
        dev_token: Some(login.access_token),
        dev_token_expires: Some(expires),
        tenant_id: Some(login.user.tenant_id.to_string()),
    };
    config.defaults.tenant_id = Some(login.user.tenant_id.to_string());
    config.save()?;

    println!("{}", "Dev login successful!".green().bold());
    println!("  User:   {} ({})", login.user.username, login.user.id);
    println!("  Tenant: {}", login.user.tenant_id);

    Ok(())
}
