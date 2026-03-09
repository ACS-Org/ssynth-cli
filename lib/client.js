// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { createRequire } from "node:module";
import { CliError } from "./error.js";

const require = createRequire(import.meta.url);
const { version } = require("../package.json");

export class ApiClient {
  constructor(baseUrl, authToken) {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.authToken = authToken || null;
  }

  headers(extra = {}) {
    const h = {
      "user-agent": `ssynth-cli/${version}`,
      ...extra,
    };
    if (this.authToken) {
      if (this.authToken.type === "api-key") {
        h["x-api-key"] = this.authToken.value;
      } else {
        h["authorization"] = `Bearer ${this.authToken.value}`;
      }
    }
    return h;
  }

  async get(urlPath) {
    const resp = await fetch(`${this.baseUrl}${urlPath}`, {
      method: "GET",
      headers: this.headers(),
    });
    return checkResponse(resp);
  }

  async post(urlPath, body) {
    const opts = {
      method: "POST",
      headers: this.headers(),
    };
    if (body !== undefined) {
      opts.headers["content-type"] = "application/json";
      opts.body = JSON.stringify(body);
    }
    const resp = await fetch(`${this.baseUrl}${urlPath}`, opts);
    return checkResponse(resp);
  }

  async postRaw(urlPath, buffer, contentType) {
    const resp = await fetch(`${this.baseUrl}${urlPath}`, {
      method: "POST",
      headers: this.headers({ "content-type": contentType }),
      body: buffer,
    });
    return checkResponse(resp);
  }

  async patch(urlPath, body) {
    const resp = await fetch(`${this.baseUrl}${urlPath}`, {
      method: "PATCH",
      headers: this.headers({ "content-type": "application/json" }),
      body: JSON.stringify(body),
    });
    return checkResponse(resp);
  }

  async delete(urlPath) {
    const resp = await fetch(`${this.baseUrl}${urlPath}`, {
      method: "DELETE",
      headers: this.headers(),
    });
    return checkResponse(resp);
  }
}

async function checkResponse(resp) {
  if (resp.ok) {
    return resp;
  }

  const status = resp.status;
  const body = await resp.text();

  try {
    const apiErr = JSON.parse(body);
    if (apiErr.error && apiErr.message) {
      throw CliError.api(status, apiErr.error, apiErr.message);
    }
  } catch (e) {
    if (e instanceof CliError) {
      throw e;
    }
  }

  const message = body || resp.statusText || "Unknown error";
  const errorCode = {
    401: "unauthorized",
    403: "forbidden",
    404: "not_found",
    429: "rate_limited",
  }[status] || "error";

  throw CliError.api(status, errorCode, message);
}
