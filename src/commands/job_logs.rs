// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use colored::Colorize;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::cli::JobLogsArgs;
use crate::client::{check_response, ApiClient};
use crate::config::AuthToken;
use crate::error::CliError;
use crate::models::{LogLine, TokenExchangeResponse, WsMessage};
use crate::output::OutputMode;

pub async fn run(args: &JobLogsArgs, client: &ApiClient, mode: OutputMode) -> Result<()> {
    // Fetch initial HTTP logs
    let offset = args.offset.unwrap_or(0);
    let limit = args.limit.unwrap_or(1000);

    let url = format!(
        "/v1/jobs/{}/logs?offset={offset}&limit={limit}",
        args.job_id
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch logs")?;
    let resp = check_response(resp).await?;
    let logs: Vec<LogLine> = resp.json().await.context("Failed to parse logs")?;

    match mode {
        OutputMode::Json if !args.follow => {
            let json = serde_json::to_string_pretty(&logs)?;
            println!("{json}");
            return Ok(());
        }
        OutputMode::Json => {
            for line in &logs {
                print_log_line_json(line);
            }
        }
        OutputMode::Human => {
            for line in &logs {
                print_log_line(line);
            }
        }
    }

    if args.follow {
        stream_ws_logs(&args.job_id, client, mode).await?;
    }

    Ok(())
}

fn print_log_line(line: &LogLine) {
    let prefix = if line.stream == "stderr" {
        "ERR".red().to_string()
    } else {
        "OUT".dimmed().to_string()
    };
    println!("{prefix} | {}", line.content);
}

fn print_log_line_json(line: &LogLine) {
    if let Ok(json) = serde_json::to_string(line) {
        println!("{json}");
    }
}

async fn stream_ws_logs(job_id: &str, client: &ApiClient, mode: OutputMode) -> Result<()> {
    let jwt = resolve_ws_jwt(client).await?;

    let base = client.base_url();
    let ws_url = base
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    let url = format!("{ws_url}/v1/jobs/{job_id}/logs/ws?token={jwt}");

    let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| CliError::WebSocket(format!("Failed to connect: {e}")))?;

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg.map_err(|e| CliError::WebSocket(e.to_string()))?;
        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Log {
                            line_num,
                            stream,
                            content,
                        } => {
                            let line = LogLine {
                                line_num,
                                stream,
                                content,
                            };
                            if mode == OutputMode::Json {
                                print_log_line_json(&line);
                            } else {
                                print_log_line(&line);
                            }
                        }
                        WsMessage::Status { status } => {
                            eprintln!();
                            eprintln!("{} {}", "Job finished:".bold(), colorize_status(&status));
                            break;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

async fn resolve_ws_jwt(client: &ApiClient) -> Result<String> {
    match client.auth_token() {
        Some(AuthToken::Jwt(token)) => Ok(token.clone()),
        Some(AuthToken::ApiKey(_)) => {
            // Exchange API key for short-lived JWT
            let resp = client
                .post("/v1/auth/token-exchange")
                .send()
                .await
                .context("Failed to exchange API key for JWT")?;
            let resp = check_response(resp).await?;
            let exchange: TokenExchangeResponse = resp
                .json()
                .await
                .context("Failed to parse token exchange")?;
            Ok(exchange.access_token)
        }
        None => Err(CliError::NotAuthenticated.into()),
    }
}

fn colorize_status(status: &str) -> String {
    match status {
        "completed" => status.green().bold().to_string(),
        "failed" => status.red().bold().to_string(),
        "cancelled" => status.yellow().to_string(),
        _ => status.to_string(),
    }
}
