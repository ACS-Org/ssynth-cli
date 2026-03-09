// Copyright 2026 Hemi Labs, Inc.
// SPDX-License-Identifier: GPL-3.0-only

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { ApiClient } from "../lib/client.js";

describe("ApiClient", () => {
  it("strips trailing slashes from baseUrl", () => {
    const client = new ApiClient("https://api.example.com///", null);
    assert.equal(client.baseUrl, "https://api.example.com");
  });

  it("sets api-key header", () => {
    const client = new ApiClient("https://api.example.com", {
      type: "api-key",
      value: "sk_live_test",
    });
    const headers = client.headers();
    assert.equal(headers["x-api-key"], "sk_live_test");
    assert.equal(headers["authorization"], undefined);
  });

  it("sets bearer header for jwt", () => {
    const client = new ApiClient("https://api.example.com", {
      type: "jwt",
      value: "eyJtoken",
    });
    const headers = client.headers();
    assert.equal(headers["authorization"], "Bearer eyJtoken");
    assert.equal(headers["x-api-key"], undefined);
  });

  it("includes user-agent", () => {
    const client = new ApiClient("https://api.example.com", null);
    const headers = client.headers();
    assert.match(headers["user-agent"], /^ssynth-cli\//);
  });

  it("sets no auth headers when no token", () => {
    const client = new ApiClient("https://api.example.com", null);
    const headers = client.headers();
    assert.equal(headers["x-api-key"], undefined);
    assert.equal(headers["authorization"], undefined);
  });
});
