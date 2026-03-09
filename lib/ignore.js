// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import fs from "node:fs";
import path from "node:path";
import ignore from "ignore";

export function buildIgnore(dir) {
  const ig = ignore();

  ig.add(".git/");
  ig.add("build/");

  const ignoreFile = path.join(dir, ".ssynthignore");
  if (fs.existsSync(ignoreFile)) {
    const contents = fs.readFileSync(ignoreFile, "utf8");
    ig.add(contents);
  }

  return ig;
}
