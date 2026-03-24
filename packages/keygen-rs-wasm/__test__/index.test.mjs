/**
 * WASM unit tests — export completeness, config management, offline operations.
 *
 * Run: node --test __test__/index.test.mjs
 * Requires: wasm-pack build --target nodejs --dev --out-dir pkg-node
 */
import { describe, it, beforeEach } from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const wasm = require("../pkg-node/keygen_rs_wasm.js");

// ---------------------------------------------------------------------------
// Export completeness
// ---------------------------------------------------------------------------

describe("exports", () => {
  const expectedExports = [
    // Config
    "setConfig",
    "getConfig",
    "resetConfig",
    // License core
    "validate",
    "verify",
    // License CRUD
    "createLicense",
    "listLicenses",
    "getLicense",
    "updateLicense",
    "deleteLicense",
    // License actions
    "suspendLicense",
    "reinstateLicense",
    "renewLicense",
    "revokeLicense",
    "incrementLicenseUsage",
    "decrementLicenseUsage",
    "resetLicenseUsage",
    "checkoutLicense",
    "checkInLicense",
    "attachLicenseEntitlements",
    "detachLicenseEntitlements",
    "attachLicenseUsers",
    "detachLicenseUsers",
    "listLicenseUsers",
    "changeLicensePolicy",
    "changeLicenseOwner",
    "changeLicenseGroup",
    "generateLicenseToken",
    // Machine
    "createMachine",
    "listMachines",
    "getMachine",
    "updateMachine",
    "deactivateMachine",
    "checkoutMachine",
    "pingMachine",
    "resetMachine",
    "changeMachineOwner",
    "changeMachineGroup",
    // Service
    "getServiceInfo",
    "ping",
    "supportsProductCode",
    // License/Machine file
    "licenseFileFromCert",
    "verifyLicenseFile",
    "decryptLicenseFile",
    "machineFileFromCert",
    "verifyMachineFile",
    "decryptMachineFile",
    // Product
    "createProduct",
    "listProducts",
    "getProduct",
    "updateProduct",
    "deleteProduct",
    "generateProductToken",
    // Policy
    "createPolicy",
    "listPolicies",
    "getPolicy",
    "updatePolicy",
    "deletePolicy",
    "attachPolicyEntitlements",
    "detachPolicyEntitlements",
    "listPolicyEntitlements",
    // User
    "createUser",
    "listUsers",
    "getUser",
    "updateUser",
    "deleteUser",
    "banUser",
    "unbanUser",
    "generateUserToken",
    "changeUserGroup",
    "updateUserPassword",
    "resetUserPassword",
    // Token
    "listTokens",
    "getToken",
    "regenerateToken",
    "revokeToken",
    // Group
    "createGroup",
    "listGroups",
    "getGroup",
    "updateGroup",
    "deleteGroup",
    "listGroupOwners",
    "listGroupUsers",
    "listGroupLicenses",
    "listGroupMachines",
    // Entitlement
    "createEntitlement",
    "listEntitlements",
    "getEntitlement",
    "updateEntitlement",
    "deleteEntitlement",
    // Component
    "createComponent",
    "listComponents",
    "getComponent",
    "updateComponent",
    "deleteComponent",
    // Environment
    "createEnvironment",
    "listEnvironments",
    "getEnvironment",
    "updateEnvironment",
    "deleteEnvironment",
    "generateEnvironmentToken",
    // Release
    "createRelease",
    "listReleases",
    "getRelease",
    "updateRelease",
    "deleteRelease",
    "publishRelease",
    "yankRelease",
    "upgradeRelease",
    "downloadReleaseArtifact",
    "changeReleasePackage",
    "releaseArtifacts",
    "attachReleaseConstraints",
    "detachReleaseConstraints",
    "releaseConstraints",
    // Webhook
    "createWebhookEndpoint",
    "listWebhookEndpoints",
    "getWebhookEndpoint",
    "updateWebhookEndpoint",
    "deleteWebhookEndpoint",
    "listWebhookEvents",
    "getWebhookEvent",
    "retryWebhookEvent",
    "deleteWebhookEvent",
  ];

  for (const name of expectedExports) {
    it(`exports ${name}`, () => {
      assert.equal(typeof wasm[name], "function", `${name} should be a function`);
    });
  }

  it("all expected exports are present", () => {
    const publicExports = Object.keys(wasm).filter(
      (k) => !k.startsWith("__") && typeof wasm[k] === "function",
    );
    const missing = expectedExports.filter((e) => !publicExports.includes(e));
    assert.deepStrictEqual(missing, [], `Missing exports: ${missing.join(", ")}`);
  });
});

// ---------------------------------------------------------------------------
// Config management
// ---------------------------------------------------------------------------

describe("config", () => {
  beforeEach(() => {
    try {
      wasm.resetConfig();
    } catch {
      // ignore if no config set
    }
  });

  it("setConfig and getConfig roundtrip", () => {
    wasm.setConfig({
      account: "test-account",
      product: "test-product",
      apiUrl: "https://api.keygen.sh",
    });

    const cfg = wasm.getConfig();
    assert.equal(cfg.account, "test-account");
    assert.equal(cfg.product, "test-product");
  });

  it("setConfig with all optional fields", () => {
    wasm.setConfig({
      account: "test-account",
      product: "test-product",
      apiUrl: "https://api.keygen.sh",
      licenseKey: "test-key",
      publicKey: "test-pub-key",
      environment: "test-env",
      token: "test-token",
    });

    const cfg = wasm.getConfig();
    assert.equal(cfg.licenseKey, "test-key");
    assert.equal(cfg.publicKey, "test-pub-key");
    assert.equal(cfg.environment, "test-env");
    assert.equal(cfg.token, "test-token");
  });

  it("resetConfig does not throw", () => {
    wasm.setConfig({
      account: "test-account",
      product: "test-product",
    });
    assert.doesNotThrow(() => wasm.resetConfig());
  });

  it("async functions reject without config", async () => {
    await assert.rejects(() => wasm.ping());
  });
});

// ---------------------------------------------------------------------------
// Type validation
// ---------------------------------------------------------------------------

describe("input validation", () => {
  beforeEach(() => {
    wasm.setConfig({
      account: "test-account",
      product: "test-product",
      apiUrl: "https://api.keygen.sh",
      token: "test-token",
    });
  });

  it("setConfig rejects missing required fields", () => {
    assert.throws(() => wasm.setConfig({}));
    assert.throws(() => wasm.setConfig({ account: "a" }));
  });
});
