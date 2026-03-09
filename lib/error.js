// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

export class CliError extends Error {
  constructor(message, code = 1) {
    super(message);
    this.name = "CliError";
    this.exitCode = code;
  }

  static notAuthenticated() {
    return new CliError("Not authenticated. Run `ssynth login` first.", 2);
  }

  static notFound(message) {
    return new CliError(message, 3);
  }

  static api(status, errorCode, message) {
    const err = new CliError(`API error (${status}): ${message}`, status === 404 ? 3 : 4);
    err.status = status;
    err.errorCode = errorCode;
    return err;
  }

  static fileNotFound(path) {
    return new CliError(`File not found: ${path}`, 5);
  }

  static hwbuild(message) {
    return new CliError(`Invalid hwbuild.yml: ${message}`, 6);
  }

  static config(message) {
    return new CliError(`Config error: ${message}`, 1);
  }

  static webSocket(message) {
    return new CliError(`WebSocket error: ${message}`, 1);
  }
}
