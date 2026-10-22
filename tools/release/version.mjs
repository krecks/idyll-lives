#!/usr/bin/env bun
// One version for the whole game: web/package.json (shown in the menu and Credits), the Rust
// workspace (Cargo.toml) and its crates' entries in Cargo.lock must always match.
//   bun tools/release/version.mjs          prints the version, fails if they differ (CI runs this)
//   bun tools/release/version.mjs 0.3.0    sets all three to 0.3.0 (then: commit, git tag v0.3.0;
//                                          pushing the tag publishes the game, see .github/workflows/pages.yml)
import { readFileSync, writeFileSync } from 'node:fs';

const root = new URL('../../', import.meta.url);
const pkgPath = new URL('web/package.json', root);
const cargoPath = new URL('Cargo.toml', root);
const lockPath = new URL('Cargo.lock', root);
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
const cargo = readFileSync(cargoPath, 'utf8');
const lock = readFileSync(lockPath, 'utf8');
const section = /(\[workspace\.package\][^[]*?\nversion = ")([^"]+)(")/;
const cargoVersion = cargo.match(section)?.[2];
if (!cargoVersion) throw new Error('No [workspace.package] version in Cargo.toml');

// The workspace's crates (all take the workspace version) and their entries in Cargo.lock.
const members = [...cargo.matchAll(/"crates\/([\w-]+)"/g)].map((m) => readFileSync(new URL(`crates/${m[1]}/Cargo.toml`, root), 'utf8').match(/^name = "([^"]+)"/m)?.[1]);
const entry = (name) => new RegExp(`(\\[\\[package\\]\\]\\nname = "${name}"\\nversion = ")([^"]+)(")`);
const lockVersions = members.map((name) => [name, lock.match(entry(name))?.[2]]);

const next = process.argv[2];
if (!next) {
  const stale = lockVersions.filter(([, v]) => v !== cargoVersion);
  if (pkg.version !== cargoVersion || stale.length) {
    console.error(`Version mismatch: web/package.json ${pkg.version}, Cargo.toml ${cargoVersion}, Cargo.lock ${lockVersions.map(([n, v]) => `${n} ${v}`).join(', ')}`);
    process.exit(1);
  }
  console.log(pkg.version);
  process.exit(0);
}
if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(next)) throw new Error(`Not a version: ${next}`);
pkg.version = next;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
writeFileSync(cargoPath, cargo.replace(section, `$1${next}$3`));
let nextLock = lock;
for (const [name, v] of lockVersions) {
  if (!v) throw new Error(`No ${name} in Cargo.lock`);
  nextLock = nextLock.replace(entry(name), `$1${next}$3`);
}
writeFileSync(lockPath, nextLock);
console.log(`Version set to ${next} (web/package.json, Cargo.toml, Cargo.lock).`);
