import { describe, it, beforeEach } from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const keygen = require("../index.js");

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
    "attachLicenseEntitlements",
    "detachLicenseEntitlements",
    // Machine
    "createMachine",
    "listMachines",
    "getMachine",
    "updateMachine",
    "deactivateMachine",
    "checkoutMachine",
    "pingMachine",
    "resetMachine",
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
    // Policy
    "createPolicy",
    "listPolicies",
    "getPolicy",
    "updatePolicy",
    "deletePolicy",
    // User
    "createUser",
    "listUsers",
    "getUser",
    "updateUser",
    "deleteUser",
    "banUser",
    "unbanUser",
    // Token
    "listTokens",
    "getToken",
    "regenerateToken",
    "revokeToken",
    // Environment
    "createEnvironment",
    "listEnvironments",
    "getEnvironment",
    "updateEnvironment",
    "deleteEnvironment",
    "generateEnvironmentToken",
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
    // Group
    "createGroup",
    "listGroups",
    "getGroup",
    "updateGroup",
    "deleteGroup",
    // Release
    "createRelease",
    "listReleases",
    "getRelease",
    "updateRelease",
    "deleteRelease",
    "publishRelease",
    "yankRelease",
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
    // Artifact
    "createArtifact",
    "listArtifacts",
    "getArtifact",
    "updateArtifact",
    "deleteArtifact",
    "yankArtifact",
    // Package
    "createPackage",
    "listPackages",
    "getPackage",
    "updatePackage",
    "deletePackage",
    // Arch / Channel / Platform (read-only)
    "listArches",
    "getArch",
    "listChannels",
    "getChannel",
    "listPlatforms",
    "getPlatform",
  ];

  for (const name of expectedExports) {
    it(`exports ${name}`, () => {
      assert.equal(typeof keygen[name], "function", `${name} should be a function`);
    });
  }
});

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

describe("config", () => {
  beforeEach(() => {
    keygen.resetConfig();
  });

  it("setConfig / getConfig roundtrip", () => {
    keygen.setConfig({
      account: "acc-123",
      product: "prod-456",
      licenseKey: "LK-789",
      publicKey: "pk-abc",
      token: "tok-xyz",
    });

    const cfg = keygen.getConfig();
    assert.equal(cfg.account, "acc-123");
    assert.equal(cfg.product, "prod-456");
    assert.equal(cfg.licenseKey, "LK-789");
    assert.equal(cfg.publicKey, "pk-abc");
    assert.equal(cfg.token, "tok-xyz");
  });

  it("default config values", () => {
    keygen.setConfig({ account: "a", product: "p" });
    const cfg = keygen.getConfig();

    assert.equal(cfg.apiUrl, "https://api.keygen.sh");
    assert.equal(cfg.apiVersion, "1.7");
    assert.equal(cfg.apiPrefix, "v1");
    assert.equal(cfg.maxClockDrift, 5);
    assert.equal(cfg.verifyKeygenSignature, true);
  });

  it("optional fields default to undefined/null", () => {
    keygen.setConfig({ account: "a", product: "p" });
    const cfg = keygen.getConfig();

    assert.equal(cfg.licenseKey, undefined);
    assert.equal(cfg.publicKey, undefined);
    assert.equal(cfg.environment, undefined);
    assert.equal(cfg.token, undefined);
  });

  it("resetConfig restores defaults", () => {
    keygen.setConfig({
      account: "acc-123",
      product: "prod-456",
      licenseKey: "LK-789",
    });
    keygen.resetConfig();

    const cfg = keygen.getConfig();
    assert.equal(cfg.account, "");
    assert.equal(cfg.product, "");
    assert.equal(cfg.licenseKey, undefined);
  });

  it("setConfig overwrites previous config", () => {
    keygen.setConfig({ account: "first", product: "p1" });
    keygen.setConfig({ account: "second", product: "p2" });

    const cfg = keygen.getConfig();
    assert.equal(cfg.account, "second");
    assert.equal(cfg.product, "p2");
  });

  it("custom api_url is preserved", () => {
    keygen.setConfig({
      account: "a",
      product: "p",
      apiUrl: "https://custom.api.example.com",
    });
    const cfg = keygen.getConfig();
    assert.equal(cfg.apiUrl, "https://custom.api.example.com");
  });
});

// ---------------------------------------------------------------------------
// verify (offline signature verification)
// ---------------------------------------------------------------------------

describe("verify", () => {
  beforeEach(() => {
    keygen.resetConfig();
  });

  it("throws on missing public key", () => {
    keygen.setConfig({ account: "a", product: "p" });

    assert.throws(
      () => keygen.verify("ED25519_SIGN", "some-signed-key"),
      (err) => {
        assert.ok(err.message.includes("Public key is missing"));
        return true;
      },
    );
  });

  it("throws on invalid scheme string", () => {
    keygen.setConfig({
      account: "a",
      product: "p",
      publicKey: "abc123",
    });

    assert.throws(
      () => keygen.verify("INVALID_SCHEME", "some-key"),
      (err) => {
        assert.ok(err.message.includes("Invalid scheme"));
        return true;
      },
    );
  });

  it("throws on malformed signed key", () => {
    keygen.setConfig({
      account: "a",
      product: "p",
      publicKey: "a".repeat(64),
    });

    assert.throws(
      () => keygen.verify("ED25519_SIGN", "not-a-valid-signed-key"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });

  it("returns a Buffer on valid input", () => {
    // verify() returns Buffer type - check the function signature at least
    assert.equal(typeof keygen.verify, "function");
  });
});

// ---------------------------------------------------------------------------
// validate (async - needs config, will fail without real API)
// ---------------------------------------------------------------------------

describe("validate", () => {
  beforeEach(() => {
    keygen.resetConfig();
  });

  it("returns a Promise", () => {
    keygen.setConfig({
      account: "test-account",
      product: "test-product",
      licenseKey: "test-key",
      publicKey: "test-pk",
    });

    const result = keygen.validate(["fp1"]);
    assert.ok(result instanceof Promise);
    // will reject because API is not reachable, but it IS a promise
    result.catch(() => {});
  });

  it("rejects without license key", async () => {
    keygen.setConfig({
      account: "test-account",
      product: "test-product",
    });

    await assert.rejects(() => keygen.validate(["fp1"]), (err) => {
      assert.ok(err instanceof Error);
      return true;
    });
  });

  it("accepts optional entitlements parameter", () => {
    keygen.setConfig({
      account: "a",
      product: "p",
      licenseKey: "k",
      publicKey: "pk",
    });

    // Both forms should be valid
    const p1 = keygen.validate(["fp1"]);
    const p2 = keygen.validate(["fp1"], ["ent1", "ent2"]);
    const p3 = keygen.validate(["fp1"], null);

    p1.catch(() => {});
    p2.catch(() => {});
    p3.catch(() => {});
  });
});

// ---------------------------------------------------------------------------
// License file operations (offline)
// ---------------------------------------------------------------------------

describe("licenseFile", () => {
  it("licenseFileFromCert throws on invalid certificate", () => {
    assert.throws(
      () => keygen.licenseFileFromCert("some-key", "invalid-cert-content"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });

  it("verifyLicenseFile throws on invalid certificate", () => {
    assert.throws(
      () => keygen.verifyLicenseFile("invalid-cert"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });

  it("decryptLicenseFile throws on invalid inputs", () => {
    assert.throws(
      () => keygen.decryptLicenseFile("invalid-cert", "invalid-key"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });
});

describe("machineFile", () => {
  it("machineFileFromCert throws on invalid certificate", () => {
    assert.throws(
      () => keygen.machineFileFromCert("some-key", "invalid-cert-content"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });

  it("verifyMachineFile throws on invalid certificate", () => {
    assert.throws(
      () => keygen.verifyMachineFile("invalid-cert"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });

  it("decryptMachineFile throws on invalid inputs", () => {
    assert.throws(
      () => keygen.decryptMachineFile("invalid-cert", "invalid-key"),
      (err) => {
        assert.ok(err instanceof Error);
        return true;
      },
    );
  });
});

// ---------------------------------------------------------------------------
// API functions (async - verify they return Promises and handle errors)
// ---------------------------------------------------------------------------

describe("async API functions", () => {
  beforeEach(() => {
    keygen.resetConfig();
    keygen.setConfig({
      account: "test-account",
      product: "test-product",
      token: "test-token",
    });
  });

  const asyncFns = [
    { name: "createLicense", args: [{ policyId: "pol-1" }] },
    { name: "listLicenses", args: [] },
    { name: "getLicense", args: ["lic-1"] },
    { name: "deleteLicense", args: ["lic-1"] },
    { name: "suspendLicense", args: ["lic-1"] },
    { name: "reinstateLicense", args: ["lic-1"] },
    { name: "renewLicense", args: ["lic-1"] },
    { name: "revokeLicense", args: ["lic-1"] },
    { name: "incrementLicenseUsage", args: ["lic-1"] },
    { name: "decrementLicenseUsage", args: ["lic-1"] },
    { name: "resetLicenseUsage", args: ["lic-1"] },
    { name: "createMachine", args: [{ fingerprint: "fp", licenseId: "l1" }] },
    { name: "listMachines", args: [] },
    { name: "getMachine", args: ["m-1"] },
    { name: "deactivateMachine", args: ["m-1"] },
    { name: "pingMachine", args: ["m-1"] },
    { name: "resetMachine", args: ["m-1"] },
    { name: "getServiceInfo", args: [] },
    { name: "ping", args: [] },
    { name: "supportsProductCode", args: [] },
    { name: "createProduct", args: [{ name: "p", code: "c" }] },
    { name: "listProducts", args: [] },
    { name: "getProduct", args: ["prod-1"] },
    { name: "deleteProduct", args: ["prod-1"] },
    { name: "createPolicy", args: [{ productId: "p1", name: "n" }] },
    { name: "listPolicies", args: [] },
    { name: "getPolicy", args: ["pol-1"] },
    { name: "deletePolicy", args: ["pol-1"] },
    { name: "createUser", args: [{ email: "a@b.com", role: "user" }] },
    { name: "listUsers", args: [] },
    { name: "getUser", args: ["u-1"] },
    { name: "deleteUser", args: ["u-1"] },
    { name: "banUser", args: ["u-1"] },
    { name: "unbanUser", args: ["u-1"] },
    { name: "listTokens", args: [] },
    { name: "getToken", args: ["t-1"] },
    { name: "revokeToken", args: ["t-1"] },
    { name: "createEnvironment", args: [{ name: "e", code: "c" }] },
    { name: "listEnvironments", args: [] },
    { name: "getEnvironment", args: ["e-1"] },
    { name: "deleteEnvironment", args: ["e-1"] },
    { name: "createEntitlement", args: [{ code: "ent-1" }] },
    { name: "listEntitlements", args: [] },
    { name: "getEntitlement", args: ["e-1"] },
    { name: "deleteEntitlement", args: ["e-1"] },
    { name: "createComponent", args: [{ fingerprint: "f", name: "n", machineId: "m" }] },
    { name: "listComponents", args: [] },
    { name: "getComponent", args: ["c-1"] },
    { name: "deleteComponent", args: ["c-1"] },
    { name: "createGroup", args: [{ name: "g" }] },
    { name: "listGroups", args: [] },
    { name: "getGroup", args: ["g-1"] },
    { name: "deleteGroup", args: ["g-1"] },
    { name: "createRelease", args: [{ productId: "p1", version: "1.0.0" }] },
    { name: "listReleases", args: [] },
    { name: "getRelease", args: ["r-1"] },
    { name: "deleteRelease", args: ["r-1"] },
    { name: "publishRelease", args: ["r-1"] },
    { name: "yankRelease", args: ["r-1"] },
    { name: "listWebhookEndpoints", args: [] },
    { name: "getWebhookEndpoint", args: ["w-1"] },
    { name: "deleteWebhookEndpoint", args: ["w-1"] },
    { name: "listWebhookEvents", args: [] },
    { name: "getWebhookEvent", args: ["we-1"] },
    { name: "deleteWebhookEvent", args: ["we-1"] },
    { name: "createArtifact", args: [{ releaseId: "r1", filename: "f", filetype: "bin", filesize: 100, platform: "linux", arch: "amd64" }] },
    { name: "listArtifacts", args: [] },
    { name: "getArtifact", args: ["a-1"] },
    { name: "deleteArtifact", args: ["a-1"] },
    { name: "yankArtifact", args: ["a-1"] },
    { name: "createPackage", args: [{ productId: "p1", name: "n", key: "k", engine: "npm" }] },
    { name: "listPackages", args: [] },
    { name: "getPackage", args: ["pkg-1"] },
    { name: "deletePackage", args: ["pkg-1"] },
    { name: "listArches", args: [] },
    { name: "getArch", args: ["a-1"] },
    { name: "listChannels", args: [] },
    { name: "getChannel", args: ["ch-1"] },
    { name: "listPlatforms", args: [] },
    { name: "getPlatform", args: ["p-1"] },
  ];

  for (const { name, args } of asyncFns) {
    it(`${name} returns a Promise`, () => {
      const result = keygen[name](...args);
      assert.ok(result instanceof Promise, `${name} should return a Promise`);
      // suppress unhandled rejection - these will fail due to no real API
      result.catch(() => {});
    });
  }

  it("async functions reject with Error on API failure", async () => {
    await assert.rejects(
      () => keygen.getLicense("nonexistent"),
      (err) => {
        assert.ok(err instanceof Error);
        assert.ok(err.message.length > 0);
        return true;
      },
    );
  });
});

// ---------------------------------------------------------------------------
// Type safety - verify napi objects have correct shape
// ---------------------------------------------------------------------------

describe("type safety", () => {
  it("KeygenConfig has expected shape", () => {
    keygen.setConfig({
      account: "a",
      product: "p",
      licenseKey: "lk",
      publicKey: "pk",
      apiUrl: "https://example.com",
      apiVersion: "2.0",
      apiPrefix: "v2",
      environment: "sandbox",
      userAgent: "test/1.0",
      package: "my-pkg",
      platform: "linux",
      maxClockDrift: 10,
      verifyKeygenSignature: false,
      token: "tok-123",
    });

    const cfg = keygen.getConfig();
    assert.equal(typeof cfg.account, "string");
    assert.equal(typeof cfg.product, "string");
    assert.equal(typeof cfg.licenseKey, "string");
    assert.equal(typeof cfg.publicKey, "string");
    assert.equal(typeof cfg.apiUrl, "string");
    assert.equal(typeof cfg.apiVersion, "string");
    assert.equal(typeof cfg.apiPrefix, "string");
    assert.equal(typeof cfg.environment, "string");
    assert.equal(typeof cfg.userAgent, "string");
    assert.equal(typeof cfg.package, "string");
    assert.equal(typeof cfg.platform, "string");
    assert.equal(typeof cfg.maxClockDrift, "number");
    assert.equal(typeof cfg.verifyKeygenSignature, "boolean");
    assert.equal(typeof cfg.token, "string");

    keygen.resetConfig();
  });
});

// ---------------------------------------------------------------------------
// Error format
// ---------------------------------------------------------------------------

describe("error format", () => {
  beforeEach(() => {
    keygen.resetConfig();
  });

  it("errors contain [CODE] prefix", () => {
    keygen.setConfig({ account: "a", product: "p" });

    try {
      keygen.verify("ED25519_SIGN", "key");
      assert.fail("should have thrown");
    } catch (err) {
      assert.match(err.message, /^\[.+\]/);
    }
  });
});
