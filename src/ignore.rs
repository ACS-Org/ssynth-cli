// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Build a gitignore-style matcher from `.ssynthignore` in the given directory.
/// Always ignores `.git/` and `build/` regardless of file contents.
pub fn build_ignore(dir: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(dir);

    // Always ignore these
    let _ = builder.add_line(None, ".git/");
    let _ = builder.add_line(None, "build/");

    let ignore_file = dir.join(".ssynthignore");
    if ignore_file.exists() {
        let _ = builder.add(&ignore_file);
    }

    builder.build().unwrap_or_else(|_| {
        // Fallback: just the defaults
        let mut b = GitignoreBuilder::new(dir);
        let _ = b.add_line(None, ".git/");
        let _ = b.add_line(None, "build/");
        b.build().expect("default ignore rules should always parse")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ignore::Match;
    use std::fs;

    #[test]
    fn test_default_ignores() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let gi = build_ignore(root);
        // .git and build are directories, matched with is_dir=true
        assert!(gi.matched(root.join(".git"), true).is_ignore());
        assert!(gi.matched(root.join("build"), true).is_ignore());
        // Regular files should not be ignored
        assert!(matches!(
            gi.matched(root.join("src/main.v"), false),
            Match::None | Match::Whitelist(_)
        ));
    }

    #[test]
    fn test_custom_ignore() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join(".ssynthignore"), "*.log\ntmp/\n").unwrap();
        let gi = build_ignore(root);
        assert!(gi.matched(root.join("synth.log"), false).is_ignore());
        assert!(gi.matched(root.join("tmp"), true).is_ignore());
        // Source files should not be ignored
        assert!(matches!(
            gi.matched(root.join("design.v"), false),
            Match::None | Match::Whitelist(_)
        ));
    }
}
