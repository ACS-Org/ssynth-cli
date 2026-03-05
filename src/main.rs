// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

mod cli;
mod client;
mod commands;
mod config;
mod error;
mod hwbuild;
mod ignore;
mod models;
mod output;
mod parse;
mod upload;

use std::process;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use cli::{ApiKeyCommand, ArtifactCommand, Cli, Command, ConfigAction, JobCommand, ProjectCommand};
use client::ApiClient;
use config::Config;
use error::CliError;
use output::OutputMode;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli).await {
        let exit_code = if let Some(cli_err) = err.downcast_ref::<CliError>() {
            eprintln!("{} {cli_err}", "Error:".red().bold());
            cli_err.exit_code()
        } else {
            eprintln!("{} {err:?}", "Error:".red().bold());
            1
        };
        process::exit(exit_code);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let mut config = Config::load()?;
    let mode = OutputMode::from_flag(cli.json);
    let api_url = config.effective_api_url(cli.api_url.as_deref());

    match cli.command {
        Command::Login(ref args) => {
            commands::login::run(args, &mut config, &api_url).await?;
        }
        Command::Config(ref args) => match args.action {
            ConfigAction::Show => {
                commands::config_show::run(&config, &api_url, mode);
            }
        },
        Command::Job(ref cmd) => {
            let auth = config.resolve_auth_token()?;
            let client = ApiClient::new(&api_url, Some(auth))?;
            dispatch_job(cmd, &config, &client, mode).await?;
        }
        Command::Artifact(ref cmd) => {
            let auth = config.resolve_auth_token()?;
            let client = ApiClient::new(&api_url, Some(auth))?;
            dispatch_artifact(cmd, &client, mode).await?;
        }
        Command::Project(ref cmd) => {
            let auth = config.resolve_auth_token()?;
            let client = ApiClient::new(&api_url, Some(auth))?;
            dispatch_project(cmd, &config, &client, mode).await?;
        }
        Command::ApiKey(ref cmd) => {
            let auth = config.resolve_auth_token()?;
            let client = ApiClient::new(&api_url, Some(auth))?;
            let tenant_id = require_tenant(&config)?;
            dispatch_api_key(cmd, &client, &tenant_id, mode).await?;
        }
        Command::Usage => {
            let auth = config.resolve_auth_token()?;
            let client = ApiClient::new(&api_url, Some(auth))?;
            let tenant_id = require_tenant(&config)?;
            commands::usage::run(&client, &tenant_id, mode).await?;
        }
        Command::Targets => {
            let auth = config.resolve_auth_token()?;
            let client = ApiClient::new(&api_url, Some(auth))?;
            commands::targets::run(&client, mode).await?;
        }
    }

    Ok(())
}

async fn dispatch_job(
    cmd: &JobCommand,
    config: &Config,
    client: &ApiClient,
    mode: OutputMode,
) -> Result<()> {
    match cmd {
        JobCommand::Submit(args) => {
            let tenant_id = require_tenant(config)?;
            commands::job_submit::run(args, client, config, &tenant_id, mode).await?;
        }
        JobCommand::Status(args) => {
            commands::job_status::run(args, client, mode).await?;
        }
        JobCommand::List(args) => {
            let tenant_id = require_tenant(config)?;
            commands::job_list::run(args, client, &tenant_id, mode).await?;
        }
        JobCommand::Logs(args) => {
            commands::job_logs::run(args, client, mode).await?;
        }
        JobCommand::Cancel(args) => {
            commands::job_cancel::run(args, client, mode).await?;
        }
        JobCommand::Retry(args) => {
            commands::job_retry::run(args, client, mode).await?;
        }
        JobCommand::Clone(args) => {
            commands::job_clone::run(args, client, mode).await?;
        }
    }
    Ok(())
}

async fn dispatch_artifact(
    cmd: &ArtifactCommand,
    client: &ApiClient,
    mode: OutputMode,
) -> Result<()> {
    match cmd {
        ArtifactCommand::List(args) => {
            commands::artifact::list(args, client, mode).await?;
        }
        ArtifactCommand::Download(args) => {
            commands::artifact::download(args, client, mode).await?;
        }
    }
    Ok(())
}

async fn dispatch_project(
    cmd: &ProjectCommand,
    config: &Config,
    client: &ApiClient,
    mode: OutputMode,
) -> Result<()> {
    match cmd {
        ProjectCommand::List(args) => {
            let tenant_id = require_tenant(config)?;
            commands::project::list(args, client, &tenant_id, mode).await?;
        }
        ProjectCommand::Create(args) => {
            let tenant_id = require_tenant(config)?;
            commands::project::create(args, client, &tenant_id, mode).await?;
        }
        ProjectCommand::Get(args) => {
            commands::project::get(args, client, mode).await?;
        }
        ProjectCommand::Update(args) => {
            commands::project::update(args, client, mode).await?;
        }
        ProjectCommand::Delete(args) => {
            commands::project::delete(args, client, mode).await?;
        }
    }
    Ok(())
}

async fn dispatch_api_key(
    cmd: &ApiKeyCommand,
    client: &ApiClient,
    tenant_id: &str,
    mode: OutputMode,
) -> Result<()> {
    match cmd {
        ApiKeyCommand::Create(args) => {
            commands::api_key::create(args, client, tenant_id, mode).await?;
        }
        ApiKeyCommand::List => {
            commands::api_key::list(client, tenant_id, mode).await?;
        }
        ApiKeyCommand::Revoke(args) => {
            commands::api_key::revoke(args, client, tenant_id, mode).await?;
        }
    }
    Ok(())
}

fn require_tenant(config: &Config) -> Result<String, CliError> {
    config
        .effective_tenant_id()
        .map(String::from)
        .ok_or_else(|| {
            CliError::Config("No tenant ID configured. Run `ssynth login` first.".to_string())
        })
}
