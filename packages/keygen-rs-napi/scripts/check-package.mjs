import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const manifest = JSON.parse(readFileSync("package.json", "utf8"));
const [packed] = JSON.parse(
  execFileSync("npm", ["pack", "--ignore-scripts", "--dry-run", "--json"], {
    encoding: "utf8",
  }),
);

for (const field of ["main", "types"]) {
  const entry = manifest[field];
  assert.equal(typeof entry, "string", `Missing package.json ${field}`);
  assert(
    packed.files.some((file) => file.path === entry && file.size > 0),
    `Published package is missing ${field}: ${entry}`,
  );
}

console.log(`Package includes ${manifest.main} and ${manifest.types}`);
