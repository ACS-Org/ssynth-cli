// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Not authenticated. Run `ssynth login` first.")]
    NotAuthenticated,

    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        error_code: String,
        message: String,
    },

    #[error("Config error: {0}")]
    Config(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid hwbuild.yml: {0}")]
    HwBuild(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NotAuthenticated => 2,
            Self::Api { status, .. } if *status == 404 => 3,
            Self::Api { .. } => 4,
            Self::FileNotFound(_) => 5,
            Self::HwBuild(_) => 6,
            _ => 1,
        }
    }
}
