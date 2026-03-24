/**
 * WASM API integration tests using real keygen.sh credentials from .env
 *
 * Run: node --test __test__/api.test.mjs
 * Requires:
 *   - wasm-pack build --target nodejs --dev --out-dir pkg-node
 *   - ../../.env with KEYGEN_ACCOUNT, KEYGEN_API_URL, etc.
 *
 * Auth note: keygen-rs uses license_key auth when set, token auth otherwise.
 * Tests are grouped by auth mode to avoid conflicts.
 */
import { describe, it, before, after } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const wasm = require("../pkg-node/keygen_rs_wasm.js");

const __dirname = dirname(fileURLToPath(import.meta.url));
const envPath = resolve(__dirname, "../../../.env");

function loadEnv() {
  const content = readFileSync(envPath, "utf-8");
  const vars = {};
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const idx = trimmed.indexOf("=");
    if (idx === -1) continue;
    vars[trimmed.slice(0, idx)] = trimmed.slice(idx + 1);
  }
  return vars;
}

const env = loadEnv();

function setLicenseKeyAuth() {
  wasm.setConfig({
    account: env.KEYGEN_ACCOUNT,
    product: env.KEYGEN_PRODUCT,
    apiUrl: env.KEYGEN_API_URL,
    licenseKey: env.KEYGEN_LICENSE_KEY,
    publicKey: env.KEYGEN_PUBLIC_KEY,
  });
}

function setTokenAuth() {
  wasm.setConfig({
    account: env.KEYGEN_ACCOUNT,
    product: env.KEYGEN_PRODUCT,
    apiUrl: env.KEYGEN_API_URL,
    publicKey: env.KEYGEN_PUBLIC_KEY,
    token: env.KEYGEN_ADMIN_TOKEN,
  });
}

// ---------------------------------------------------------------------------
// Service (no auth needed)
// ---------------------------------------------------------------------------

describe("service", () => {
  before(() => {
    wasm.setConfig({
      account: env.KEYGEN_ACCOUNT,
      product: env.KEYGEN_PRODUCT,
      apiUrl: env.KEYGEN_API_URL,
      verifyKeygenSignature: false,
    });
  });

  after(() => wasm.resetConfig());

  it("ping returns a response", async () => {
    const res = await wasm.ping();
    assert.equal(typeof res.message, "string");
    console.log(`    Ping: "${res.message}" (version: ${res.version})`);
  });

  it("getServiceInfo returns API info", async () => {
    const info = await wasm.getServiceInfo();
    assert.ok(info.apiVersion);
    console.log(`    API version: ${info.apiVersion}`);
  });

  it("supportsProductCode returns boolean", async () => {
    const result = await wasm.supportsProductCode();
    assert.equal(typeof result, "boolean");
    console.log(`    Supports product code: ${result}`);
  });
});

// ---------------------------------------------------------------------------
// License validation (license-key auth)
// ---------------------------------------------------------------------------

describe("license validation", () => {
  before(() => setLicenseKeyAuth());
  after(() => wasm.resetConfig());

  it("validate returns a license", async () => {
    const license = await wasm.validate([], []);
    assert.ok(license.id);
    assert.ok(license.key);
    assert.equal(typeof license.id, "string");
    assert.equal(typeof license.key, "string");
    console.log(`    License: ${license.id} (${license.status})`);
  });

  it("validate with unregistered fingerprint rejects with NO_MACHINE", async () => {
    await assert.rejects(
      () => wasm.validate(["unregistered-wasm-fp"], []),
      (err) => {
        assert.ok(err.message.includes("NO_MACHINE"));
        return true;
      },
    );
  });

  it("license has expected fields", async () => {
    const license = await wasm.validate([], []);
    assert.equal(typeof license.id, "string");
    assert.equal(typeof license.key, "string");
    assert.ok(license.metadata !== undefined);
  });
});

// ---------------------------------------------------------------------------
// License CRUD (token auth)
// ---------------------------------------------------------------------------

describe("license CRUD", () => {
  let createdLicenseId;

  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("createLicense", async () => {
    const license = await wasm.createLicense({
      policyId: env.KEYGEN_POLICY_ID,
      name: "wasm-test-license",
    });

    assert.ok(license.id);
    assert.equal(license.name, "wasm-test-license");
    createdLicenseId = license.id;
    console.log(`    Created: ${license.id}`);
  });

  it("getLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const license = await wasm.getLicense(createdLicenseId);
    assert.equal(license.id, createdLicenseId);
    assert.equal(license.name, "wasm-test-license");
  });

  it("listLicenses", async () => {
    const licenses = await wasm.listLicenses({ limit: 5 });
    assert.ok(Array.isArray(licenses));
    assert.ok(licenses.length > 0);
    assert.ok(licenses[0].id);
  });

  it("updateLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const updated = await wasm.updateLicense(createdLicenseId, {
      name: "wasm-test-updated",
    });
    assert.equal(updated.name, "wasm-test-updated");
  });

  it("suspendLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const suspended = await wasm.suspendLicense(createdLicenseId);
    assert.ok(suspended.id);
  });

  it("reinstateLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const reinstated = await wasm.reinstateLicense(createdLicenseId);
    assert.ok(reinstated.id);
  });

  it("deleteLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    await wasm.deleteLicense(createdLicenseId);
    console.log(`    Deleted: ${createdLicenseId}`);
  });
});

// ---------------------------------------------------------------------------
// Product CRUD (token auth)
// ---------------------------------------------------------------------------

describe("product CRUD", () => {
  let createdProductId;

  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("createProduct", async () => {
    const product = await wasm.createProduct({
      name: "wasm-test-product",
      code: `wasm-test-${Date.now()}`,
    });

    assert.ok(product.id);
    assert.equal(product.name, "wasm-test-product");
    createdProductId = product.id;
    console.log(`    Created: ${product.id}`);
  });

  it("getProduct", async () => {
    assert.ok(createdProductId);
    const product = await wasm.getProduct(createdProductId);
    assert.equal(product.id, createdProductId);
  });

  it("listProducts", async () => {
    const products = await wasm.listProducts({ limit: 5 });
    assert.ok(Array.isArray(products));
    assert.ok(products.length > 0);
  });

  it("updateProduct", async () => {
    assert.ok(createdProductId);
    const updated = await wasm.updateProduct(createdProductId, {
      name: "wasm-test-product-updated",
    });
    assert.equal(updated.name, "wasm-test-product-updated");
  });

  it("deleteProduct", async () => {
    assert.ok(createdProductId);
    await wasm.deleteProduct(createdProductId);
    console.log(`    Deleted: ${createdProductId}`);
  });
});

// ---------------------------------------------------------------------------
// Policy CRUD (token auth)
// ---------------------------------------------------------------------------

describe("policy CRUD", () => {
  let createdPolicyId;

  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("createPolicy", async () => {
    const policy = await wasm.createPolicy({
      name: `wasm-test-policy-${Date.now()}`,
      productId: env.KEYGEN_PRODUCT,
    });

    assert.ok(policy.id);
    createdPolicyId = policy.id;
    console.log(`    Created: ${policy.id}`);
  });

  it("getPolicy", async () => {
    assert.ok(createdPolicyId);
    const policy = await wasm.getPolicy(createdPolicyId);
    assert.equal(policy.id, createdPolicyId);
  });

  it("listPolicies", async () => {
    const policies = await wasm.listPolicies({ limit: 5 });
    assert.ok(Array.isArray(policies));
    assert.ok(policies.length > 0);
  });

  it("deletePolicy", async () => {
    assert.ok(createdPolicyId);
    await wasm.deletePolicy(createdPolicyId);
    console.log(`    Deleted: ${createdPolicyId}`);
  });
});

// ---------------------------------------------------------------------------
// User CRUD (token auth)
// ---------------------------------------------------------------------------

describe("user CRUD", () => {
  let createdUserId;

  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("createUser", async () => {
    const user = await wasm.createUser({
      email: `wasm-test-${Date.now()}@example.com`,
      role: "user",
    });

    assert.ok(user.id);
    assert.ok(user.email);
    createdUserId = user.id;
    console.log(`    Created: ${user.id}`);
  });

  it("getUser", async () => {
    assert.ok(createdUserId);
    const user = await wasm.getUser(createdUserId);
    assert.equal(user.id, createdUserId);
  });

  it("listUsers", async () => {
    const users = await wasm.listUsers({ limit: 5 });
    assert.ok(Array.isArray(users));
  });

  it("banUser", async () => {
    assert.ok(createdUserId);
    const banned = await wasm.banUser(createdUserId);
    assert.ok(banned.id);
  });

  it("unbanUser", async () => {
    assert.ok(createdUserId);
    const unbanned = await wasm.unbanUser(createdUserId);
    assert.ok(unbanned.id);
  });

  it("deleteUser", async () => {
    assert.ok(createdUserId);
    await wasm.deleteUser(createdUserId);
    console.log(`    Deleted: ${createdUserId}`);
  });
});

// ---------------------------------------------------------------------------
// Entitlement CRUD (token auth)
// ---------------------------------------------------------------------------

describe("entitlement CRUD", () => {
  let createdEntitlementId;

  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("createEntitlement", async () => {
    const ent = await wasm.createEntitlement({
      name: "wasm-test-entitlement",
      code: `wasm-test-${Date.now()}`,
    });

    assert.ok(ent.id);
    assert.equal(ent.name, "wasm-test-entitlement");
    createdEntitlementId = ent.id;
    console.log(`    Created: ${ent.id}`);
  });

  it("getEntitlement", async () => {
    assert.ok(createdEntitlementId);
    const ent = await wasm.getEntitlement(createdEntitlementId);
    assert.equal(ent.id, createdEntitlementId);
  });

  it("listEntitlements", async () => {
    const list = await wasm.listEntitlements({ limit: 5 });
    assert.ok(Array.isArray(list));
  });

  it("deleteEntitlement", async () => {
    assert.ok(createdEntitlementId);
    await wasm.deleteEntitlement(createdEntitlementId);
    console.log(`    Deleted: ${createdEntitlementId}`);
  });
});

// ---------------------------------------------------------------------------
// Group CRUD (token auth)
// ---------------------------------------------------------------------------

describe("group CRUD", () => {
  let createdGroupId;

  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("createGroup", async () => {
    const group = await wasm.createGroup({
      name: `wasm-test-group-${Date.now()}`,
    });

    assert.ok(group.id);
    createdGroupId = group.id;
    console.log(`    Created: ${group.id}`);
  });

  it("getGroup", async () => {
    assert.ok(createdGroupId);
    const group = await wasm.getGroup(createdGroupId);
    assert.equal(group.id, createdGroupId);
  });

  it("listGroups", async () => {
    const list = await wasm.listGroups({ limit: 5 });
    assert.ok(Array.isArray(list));
  });

  it("updateGroup", async () => {
    assert.ok(createdGroupId);
    const updated = await wasm.updateGroup(createdGroupId, {
      name: "wasm-test-group-updated",
    });
    assert.equal(updated.name, "wasm-test-group-updated");
  });

  it("deleteGroup", async () => {
    assert.ok(createdGroupId);
    await wasm.deleteGroup(createdGroupId);
    console.log(`    Deleted: ${createdGroupId}`);
  });
});

// ---------------------------------------------------------------------------
// Token operations (token auth)
// ---------------------------------------------------------------------------

describe("token operations", () => {
  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("listTokens", async () => {
    const tokens = await wasm.listTokens({ limit: 5 });
    assert.ok(Array.isArray(tokens));
    assert.ok(tokens.length > 0);
    assert.ok(tokens[0].id);
    assert.ok(tokens[0].kind);
  });
});

// ---------------------------------------------------------------------------
// Release operations (token auth)
// ---------------------------------------------------------------------------

describe("release operations", () => {
  before(() => setTokenAuth());
  after(() => wasm.resetConfig());

  it("listReleases returns array", async () => {
    const releases = await wasm.listReleases({ limit: 5 });
    assert.ok(Array.isArray(releases));
  });

  it("downloadReleaseArtifact returns a URL (not a redirect)", async () => {
    // On WASM, downloadReleaseArtifact returns the constructed URL
    // rather than following the redirect (reqwest redirect not available on wasm32).
    // We can only test this if there are releases.
    const releases = await wasm.listReleases({ limit: 1 });
    if (releases.length > 0) {
      const result = await wasm.downloadReleaseArtifact(releases[0].id, "test.txt");
      assert.ok(result.location);
      assert.ok(result.location.startsWith("https://"));
      console.log(`    Download URL: ${result.location.slice(0, 80)}...`);
    } else {
      console.log("    Skipped: no releases available");
    }
  });
});
