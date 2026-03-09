// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import js from "@eslint/js";
import globals from "globals";

export default [
  js.configs.recommended,
  {
    languageOptions: {
      ecmaVersion: 2025,
      sourceType: "module",
      globals: {
        ...globals.node,
      },
    },
    rules: {
      "strict": ["error", "never"],
      "no-var": "error",
      "prefer-const": "error",
      "eqeqeq": ["error", "always"],
      "no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "no-implicit-globals": "error",
      "no-shadow": "error",
      "curly": ["error", "all"],
      "no-throw-literal": "error",
      "no-constant-condition": ["error", { checkLoops: false }],
    },
  },
  {
    ignores: ["node_modules/"],
  },
];
