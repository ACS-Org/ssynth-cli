// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use ignore::gitignore::Gitignore;
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::{check_response, ApiClient};
use crate::models::UploadResponse;

/// Create a tar.gz archive of the given directory, respecting ignore rules.
pub fn create_tarball(dir: &Path, ignore: &Gitignore) -> Result<Vec<u8>> {
    let buf = Vec::new();
    let enc = GzEncoder::new(buf, Compression::default());
    let mut ar = tar::Builder::new(enc);

    add_dir_to_tar(&mut ar, dir, dir, ignore)?;

    let enc = ar.into_inner().context("Failed to finalize tar archive")?;
    let compressed = enc.finish().context("Failed to finish gzip compression")?;
    Ok(compressed)
}

fn add_dir_to_tar<W: Write>(
    ar: &mut tar::Builder<W>,
    root: &Path,
    current: &Path,
    ignore: &Gitignore,
) -> Result<()> {
    let entries = std::fs::read_dir(current)
        .with_context(|| format!("Failed to read directory: {}", current.display()))?;

    let mut sorted_entries: Vec<_> = entries.filter_map(Result::ok).collect();
    sorted_entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in sorted_entries {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .with_context(|| format!("Failed to strip prefix from {}", path.display()))?;
        let is_dir = path.is_dir();

        if ignore.matched(rel, is_dir).is_ignore() {
            continue;
        }

        if is_dir {
            add_dir_to_tar(ar, root, &path, ignore)?;
        } else {
            ar.append_path_with_name(&path, rel)
                .with_context(|| format!("Failed to add {} to archive", path.display()))?;
        }
    }
    Ok(())
}

/// Upload archive data to the API, which stores it in S3. Returns the `source_key`.
///
/// `content_type` should be `"application/gzip"` for `.tar.gz` or `"application/zip"` for `.zip`.
pub async fn upload_source(
    client: &ApiClient,
    data: Vec<u8>,
    content_type: &str,
) -> Result<UploadResponse> {
    let size = data.len();
    let pb = ProgressBar::new(size as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} Uploading [{bar:40.cyan/blue}] {bytes}/{total_bytes}")
            .expect("valid template")
            .progress_chars("=> "),
    );

    let resp = client
        .post("/v1/jobs/upload")
        .header("content-type", content_type)
        .body(data)
        .send()
        .await
        .context("Failed to upload source")?;

    pb.finish_and_clear();

    let resp = check_response(resp).await?;
    resp.json().await.context("Failed to parse upload response")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_tarball() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.v"), "module top; endmodule").unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/config"), "git stuff").unwrap();

        let ignore = crate::ignore::build_ignore(dir.path());
        let tarball = create_tarball(dir.path(), &ignore).unwrap();
        assert!(!tarball.is_empty());

        // Verify .git was excluded by extracting
        let decoder = flate2::read::GzDecoder::new(&tarball[..]);
        let mut archive = tar::Archive::new(decoder);
        let entries: Vec<String> = archive
            .entries()
            .unwrap()
            .filter_map(std::result::Result::ok)
            .map(|e| e.path().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(entries.contains(&"main.v".to_string()));
        assert!(!entries.iter().any(|e| e.contains(".git")));
    }
}
