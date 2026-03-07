// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::error::CliError;

/// Represents the `hwbuild.yml` project configuration file.
#[derive(Debug, Deserialize)]
pub struct HwBuild {
    #[serde(alias = "top")]
    pub top_module: Option<String>,
    pub target: Option<TargetSpec>,
    pub constraints: Option<Vec<String>>,
    pub seeds: Option<i32>,
    pub pick: Option<String>,
    pub priority: Option<String>,
    pub parallelism: Option<i32>,
    pub steps: Option<Vec<String>>,
    pub max_runtime: Option<String>,
    pub max_memory: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TargetSpec {
    pub family: Option<String>,
    pub device: Option<String>,
    pub package: Option<String>,
    pub board: Option<String>,
}

impl HwBuild {
    pub fn load(dir: &Path) -> Result<Option<Self>> {
        let short_ext = dir.join("hwbuild.yml");
        let long_ext = dir.join("hwbuild.yaml");
        let path = if short_ext.exists() {
            short_ext
        } else if long_ext.exists() {
            long_ext
        } else {
            return Ok(None);
        };
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let hw: Self = serde_yaml::from_str(&contents)
            .map_err(|e| CliError::HwBuild(format!("{}: {e}", path.display())))?;
        Ok(Some(hw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_hwbuild() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("hwbuild.yml"),
            r"
top_module: blinky
target:
  family: ice40
  device: lp8k
  package: cm81
constraints:
  - constraints.pcf
seeds: 4
pick: best_timing
",
        )
        .unwrap();
        let hw = HwBuild::load(dir.path()).unwrap().unwrap();
        assert_eq!(hw.top_module.as_deref(), Some("blinky"));
        assert_eq!(hw.seeds, Some(4));
        let target = hw.target.unwrap();
        assert_eq!(target.family.as_deref(), Some("ice40"));
    }

    #[test]
    fn test_no_hwbuild() {
        let dir = tempfile::tempdir().unwrap();
        let hw = HwBuild::load(dir.path()).unwrap();
        assert!(hw.is_none());
    }
}
