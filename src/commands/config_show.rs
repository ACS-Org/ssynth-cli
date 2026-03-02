// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::Config;
use crate::output::{new_table, print_output, OutputMode};

pub fn run(config: &Config, api_url: &str, mode: OutputMode) {
    let redacted = RedactedConfig::from(config, api_url);

    print_output(mode, &redacted, |c| {
        let mut table = new_table();
        table.set_header(["SETTING", "VALUE"]);
        table.add_row(["API URL", &c.api_url]);
        table.add_row(["Auth Method", &c.auth_method]);
        table.add_row(["Auth Value", &c.auth_value]);
        table.add_row(["Tenant ID", &c.tenant_id]);
        table.add_row(["Default Project", &c.default_project]);
        table.add_row(["Config File", &c.config_path]);
        table
    });
}

#[derive(serde::Serialize)]
struct RedactedConfig {
    api_url: String,
    auth_method: String,
    auth_value: String,
    tenant_id: String,
    default_project: String,
    config_path: String,
}

impl RedactedConfig {
    fn from(config: &Config, api_url: &str) -> Self {
        let (auth_method, auth_value) = if std::env::var("SSYNTH_API_KEY").is_ok() {
            ("env:SSYNTH_API_KEY".to_string(), redact_key("sk_live_..."))
        } else if let Some(ref key) = config.auth.api_key {
            ("config:api_key".to_string(), redact_key(key))
        } else if config.auth.dev_token.is_some() {
            let expires = config
                .auth
                .dev_token_expires
                .as_deref()
                .unwrap_or("unknown");
            (
                "config:dev_token".to_string(),
                format!("(expires {expires})"),
            )
        } else {
            ("none".to_string(), "-".to_string())
        };

        Self {
            api_url: api_url.to_string(),
            auth_method,
            auth_value,
            tenant_id: config.effective_tenant_id().unwrap_or("-").to_string(),
            default_project: config
                .defaults
                .project_id
                .as_deref()
                .unwrap_or("-")
                .to_string(),
            config_path: Config::config_path()
                .map_or_else(|_| "unknown".to_string(), |p| p.display().to_string()),
        }
    }
}

fn redact_key(key: &str) -> String {
    if key.len() > 16 {
        format!("{}...{}", &key[..12], &key[key.len() - 4..])
    } else {
        "****".to_string()
    }
}
