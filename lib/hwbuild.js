// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import fs from "node:fs";
import path from "node:path";
import yaml from "js-yaml";
import { CliError } from "./error.js";

export function loadHwBuild(dir) {
  const shortPath = path.join(dir, "hwbuild.yml");
  const longPath = path.join(dir, "hwbuild.yaml");

  let filePath = null;
  if (fs.existsSync(shortPath)) {
    filePath = shortPath;
  } else if (fs.existsSync(longPath)) {
    filePath = longPath;
  } else {
    return null;
  }

  try {
    const contents = fs.readFileSync(filePath, "utf8");
    const hw = yaml.load(contents);
    if (hw.top) {
      hw.top_module = hw.top_module || hw.top;
    }
    return hw;
  } catch (e) {
    throw CliError.hwbuild(`${filePath}: ${e.message}`);
  }
}
