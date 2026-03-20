/**
 * API integration tests using real keygen.sh credentials from .env
 *
 * Run: node --test __test__/api.test.mjs
 * Requires: ../../.env with KEYGEN_ACCOUNT, KEYGEN_API_URL, etc.
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
const keygen = require("../index.js");

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
  keygen.setConfig({
    account: env.KEYGEN_ACCOUNT,
    product: env.KEYGEN_PRODUCT,
    apiUrl: env.KEYGEN_API_URL,
    licenseKey: env.KEYGEN_LICENSE_KEY,
    publicKey: env.KEYGEN_PUBLIC_KEY,
  });
}

function setTokenAuth() {
  keygen.setConfig({
    account: env.KEYGEN_ACCOUNT,
    product: env.KEYGEN_PRODUCT,
    apiUrl: env.KEYGEN_API_URL,
    publicKey: env.KEYGEN_PUBLIC_KEY,
    token: env.KEYGEN_ADMIN_TOKEN,
  });
}

// ---------------------------------------------------------------------------
// Service (no auth needed, disable signature verification)
// ---------------------------------------------------------------------------

describe("service", () => {
  before(() => {
    keygen.setConfig({
      account: env.KEYGEN_ACCOUNT,
      product: env.KEYGEN_PRODUCT,
      apiUrl: env.KEYGEN_API_URL,
      verifyKeygenSignature: false,
    });
  });

  after(() => keygen.resetConfig());

  it("ping returns a response", async () => {
    const res = await keygen.ping();
    assert.equal(typeof res.message, "string");
    console.log(`    Ping: "${res.message}" (version: ${res.version})`);
  });

  it("getServiceInfo returns API info", async () => {
    const info = await keygen.getServiceInfo();
    assert.ok(info.apiVersion);
    assert.ok(info.headers);
    console.log(`    API version: ${info.apiVersion}`);
  });

  it("supportsProductCode returns boolean", async () => {
    const result = await keygen.supportsProductCode();
    assert.equal(typeof result, "boolean");
    console.log(`    Supports product code: ${result}`);
  });
});

// ---------------------------------------------------------------------------
// License validation (license-key auth)
// ---------------------------------------------------------------------------

describe("license validation", () => {
  before(() => setLicenseKeyAuth());
  after(() => keygen.resetConfig());

  it("validate returns a license", async () => {
    const license = await keygen.validate([], []);
    assert.ok(license.id);
    assert.ok(license.key);
    assert.equal(typeof license.id, "string");
    assert.equal(typeof license.key, "string");
    console.log(`    License: ${license.id} (${license.status})`);
  });

  it("validate with unregistered fingerprint rejects with NO_MACHINE", async () => {
    await assert.rejects(
      () => keygen.validate(["unregistered-napi-fp"], []),
      (err) => {
        assert.ok(err.message.includes("NO_MACHINE"));
        return true;
      },
    );
  });

  it("license has expected fields", async () => {
    const license = await keygen.validate([], []);
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
  after(() => keygen.resetConfig());

  it("createLicense", async () => {
    const license = await keygen.createLicense({
      policyId: env.KEYGEN_POLICY_ID,
      name: "napi-test-license",
    });

    assert.ok(license.id);
    assert.equal(license.name, "napi-test-license");
    createdLicenseId = license.id;
    console.log(`    Created: ${license.id}`);
  });

  it("getLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const license = await keygen.getLicense(createdLicenseId);
    assert.equal(license.id, createdLicenseId);
    assert.equal(license.name, "napi-test-license");
  });

  it("listLicenses", async () => {
    const licenses = await keygen.listLicenses({ limit: 5 });
    assert.ok(Array.isArray(licenses));
    assert.ok(licenses.length > 0);
    assert.ok(licenses[0].id);
  });

  it("updateLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const updated = await keygen.updateLicense(createdLicenseId, {
      name: "napi-test-updated",
    });
    assert.equal(updated.name, "napi-test-updated");
  });

  it("suspendLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const suspended = await keygen.suspendLicense(createdLicenseId);
    assert.ok(suspended.id);
  });

  it("reinstateLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    const reinstated = await keygen.reinstateLicense(createdLicenseId);
    assert.ok(reinstated.id);
  });

  it("deleteLicense", async () => {
    assert.ok(createdLicenseId, "requires createLicense");
    await keygen.deleteLicense(createdLicenseId);
    console.log(`    Deleted: ${createdLicenseId}`);
  });
});

// ---------------------------------------------------------------------------
// Product CRUD (token auth)
// ---------------------------------------------------------------------------

describe("product CRUD", () => {
  let createdProductId;

  before(() => setTokenAuth());
  after(() => keygen.resetConfig());

  it("createProduct", async () => {
    const product = await keygen.createProduct({
      name: "napi-test-product",
      code: `napi-test-${Date.now()}`,
    });

    assert.ok(product.id);
    assert.equal(product.name, "napi-test-product");
    createdProductId = product.id;
    console.log(`    Created: ${product.id}`);
  });

  it("getProduct", async () => {
    assert.ok(createdProductId);
    const product = await keygen.getProduct(createdProductId);
    assert.equal(product.id, createdProductId);
  });

  it("listProducts", async () => {
    const products = await keygen.listProducts({ limit: 5 });
    assert.ok(Array.isArray(products));
    assert.ok(products.length > 0);
  });

  it("updateProduct", async () => {
    assert.ok(createdProductId);
    const updated = await keygen.updateProduct(createdProductId, {
      name: "napi-test-product-updated",
    });
    assert.equal(updated.name, "napi-test-product-updated");
  });

  it("deleteProduct", async () => {
    assert.ok(createdProductId);
    await keygen.deleteProduct(createdProductId);
    console.log(`    Deleted: ${createdProductId}`);
  });
});

// ---------------------------------------------------------------------------
// User CRUD (token auth)
// ---------------------------------------------------------------------------

describe("user CRUD", () => {
  let createdUserId;

  before(() => setTokenAuth());
  after(() => keygen.resetConfig());

  it("createUser", async () => {
    const user = await keygen.createUser({
      email: `napi-test-${Date.now()}@example.com`,
      role: "user",
      password: "test-password-123!",
    });

    assert.ok(user.id);
    assert.ok(user.email);
    createdUserId = user.id;
    console.log(`    Created: ${user.id}`);
  });

  it("getUser", async () => {
    assert.ok(createdUserId);
    const user = await keygen.getUser(createdUserId);
    assert.equal(user.id, createdUserId);
  });

  it("listUsers", async () => {
    const users = await keygen.listUsers({ limit: 5 });
    assert.ok(Array.isArray(users));
  });

  it("banUser", async () => {
    assert.ok(createdUserId);
    const banned = await keygen.banUser(createdUserId);
    assert.ok(banned.id);
  });

  it("unbanUser", async () => {
    assert.ok(createdUserId);
    const unbanned = await keygen.unbanUser(createdUserId);
    assert.ok(unbanned.id);
  });

  it("deleteUser", async () => {
    assert.ok(createdUserId);
    await keygen.deleteUser(createdUserId);
    console.log(`    Deleted: ${createdUserId}`);
  });
});

// ---------------------------------------------------------------------------
// Entitlement CRUD (token auth)
// ---------------------------------------------------------------------------

describe("entitlement CRUD", () => {
  let createdEntitlementId;

  before(() => setTokenAuth());
  after(() => keygen.resetConfig());

  it("createEntitlement", async () => {
    const ent = await keygen.createEntitlement({
      name: "napi-test-entitlement",
      code: `napi-test-${Date.now()}`,
    });

    assert.ok(ent.id);
    assert.equal(ent.name, "napi-test-entitlement");
    createdEntitlementId = ent.id;
    console.log(`    Created: ${ent.id}`);
  });

  it("getEntitlement", async () => {
    assert.ok(createdEntitlementId);
    const ent = await keygen.getEntitlement(createdEntitlementId);
    assert.equal(ent.id, createdEntitlementId);
  });

  it("listEntitlements", async () => {
    const list = await keygen.listEntitlements({ limit: 5 });
    assert.ok(Array.isArray(list));
  });

  it("deleteEntitlement", async () => {
    assert.ok(createdEntitlementId);
    await keygen.deleteEntitlement(createdEntitlementId);
    console.log(`    Deleted: ${createdEntitlementId}`);
  });
});

// ---------------------------------------------------------------------------
// Group CRUD (token auth)
// ---------------------------------------------------------------------------

describe("group CRUD", () => {
  let createdGroupId;

  before(() => setTokenAuth());
  after(() => keygen.resetConfig());

  it("createGroup", async () => {
    const group = await keygen.createGroup({
      name: `napi-test-group-${Date.now()}`,
    });

    assert.ok(group.id);
    createdGroupId = group.id;
    console.log(`    Created: ${group.id}`);
  });

  it("getGroup", async () => {
    assert.ok(createdGroupId);
    const group = await keygen.getGroup(createdGroupId);
    assert.equal(group.id, createdGroupId);
  });

  it("listGroups", async () => {
    const list = await keygen.listGroups({ limit: 5 });
    assert.ok(Array.isArray(list));
  });

  it("updateGroup", async () => {
    assert.ok(createdGroupId);
    const updated = await keygen.updateGroup(createdGroupId, {
      name: "napi-test-group-updated",
    });
    assert.equal(updated.name, "napi-test-group-updated");
  });

  it("deleteGroup", async () => {
    assert.ok(createdGroupId);
    await keygen.deleteGroup(createdGroupId);
    console.log(`    Deleted: ${createdGroupId}`);
  });
});

// ---------------------------------------------------------------------------
// Token operations (token auth)
// ---------------------------------------------------------------------------

describe("token operations", () => {
  before(() => setTokenAuth());
  after(() => keygen.resetConfig());

  it("listTokens", async () => {
    const tokens = await keygen.listTokens({ limit: 5 });
    assert.ok(Array.isArray(tokens));
    assert.ok(tokens.length > 0);
    assert.ok(tokens[0].id);
    assert.ok(tokens[0].kind);
  });
});

// ---------------------------------------------------------------------------
// Distribution read-only (token auth)
// ---------------------------------------------------------------------------

describe("distribution (read-only)", () => {
  before(() => {
    keygen.setConfig({
      account: env.KEYGEN_ACCOUNT,
      product: env.KEYGEN_PRODUCT,
      apiUrl: env.KEYGEN_API_URL,
      token: env.KEYGEN_ADMIN_TOKEN,
      verifyKeygenSignature: false,
    });
  });
  after(() => keygen.resetConfig());

  it("listArches", async () => {
    const arches = await keygen.listArches({});
    assert.ok(Array.isArray(arches));
  });

  it("listChannels", async () => {
    const channels = await keygen.listChannels({});
    assert.ok(Array.isArray(channels));
  });

  it("listPlatforms", async () => {
    const platforms = await keygen.listPlatforms({});
    assert.ok(Array.isArray(platforms));
  });
});
