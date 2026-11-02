// Regenerate with: node packages/tauri-plugin-keygen-rs2/tests/fixtures/generate.cjs
// The seed, license key, and IV are public test data, never production credentials.
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const key = 'CACHE-TEST-LICENSE-KEY';
const privateKey = crypto.createPrivateKey({
  key: Buffer.concat([
    Buffer.from('302e020100300506032b657004220420', 'hex'),
    Buffer.alloc(32, 7),
  ]),
  format: 'der',
  type: 'pkcs8',
});
const publicKey = crypto.createPublicKey(privateKey)
  .export({ format: 'der', type: 'spki' }).subarray(-32).toString('hex');
const issued = '2020-01-01T00:00:00Z';
const expiry = '2020-01-01T01:00:00Z';
const fixtures = { key, publicKey, files: {} };

for (const encrypted of [false, true]) {
  for (const expired of [false, true]) {
    const ttl = expired ? 3600 : 0;
    const dataset = {
      meta: { issued, expiry, ttl },
      data: {
        type: 'licenses', id: 'cache-test-license', attributes: { key }, relationships: {},
      },
      included: [{
        type: 'entitlements', id: 'entitlement-1',
        attributes: { name: 'Feature', code: 'PRO', created: issued, updated: issued },
        relationships: {},
      }],
    };
    const bytes = Buffer.from(JSON.stringify(dataset));
    let enc = bytes.toString('base64');
    if (encrypted) {
      const iv = Buffer.alloc(12, 7);
      const cipher = crypto.createCipheriv('aes-256-gcm', crypto.createHash('sha256').update(key).digest(), iv);
      const ciphertext = Buffer.concat([cipher.update(bytes), cipher.final()]);
      enc = [ciphertext, iv, cipher.getAuthTag()].map((part) => part.toString('base64')).join('.');
    }
    for (const tampered of [false, true]) {
      const signature = crypto.sign(null, Buffer.from(`license/${enc}`), privateKey);
      if (tampered) signature[0] ^= 1;
      const cert = {
        enc, sig: signature.toString('base64'),
        alg: encrypted ? 'aes-256-gcm+ed25519' : 'base64+ed25519',
      };
      const encoded = Buffer.from(JSON.stringify(cert)).toString('base64');
      const name = `${encrypted ? 'encrypted' : 'plain'}${expired ? '-expired' : ''}${tampered ? '-tampered' : ''}`;
      fixtures.files[name] = {
        id: 'file-1', issued, expiry, ttl,
        certificate: `-----BEGIN LICENSE FILE-----\n${encoded}\n-----END LICENSE FILE-----`,
      };
    }
  }
}
fs.writeFileSync(path.join(__dirname, 'license-files.json'), `${JSON.stringify(fixtures, null, 2)}\n`);
