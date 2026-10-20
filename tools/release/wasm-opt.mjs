#!/usr/bin/env node
/**
 * Optimises the game's WebAssembly (the simulation and the voice) for release with binaryen's
 * wasm-opt (the exact-pinned `binaryen` devDependency): `-O3`, and the name and producers
 * sections stripped. Production builds only: vite.config.ts runs it on the built `dist/assets/`
 * files, so `npm run wasm` for the dev server keeps the names (readable stack traces) and builds
 * no slower.
 *
 * - The features are passed explicitly, the set rustc writes into the files (`target_features`,
 *   with simd128 from .cargo/config.toml), so nothing depends on that section surviving.
 * - Never `-tnh` / `-iit` (traps-never-happen, ignore-implicit-traps): with `panic = "abort"`
 *   every Rust panic is a trap, and those flags let the optimiser assume it can't happen.
 * - About 48 s for the simulation on an M3 (binaryen.js is slow), so results are kept in
 *   `web/.assets/wasm-opt/{release,names}/` by the input's SHA-256 and a rebuild of the same wasm is instant.
 *
 *   node tools/release/wasm-opt.mjs <file.wasm>...           optimises the files in place
 *   node tools/release/wasm-opt.mjs --names <file.wasm>...   keeps the name section, for crash stacks
 *
 * The `--names` variant runs the same optimisations but leaves the names in, so its function
 * numbers match the shipped wasm and a stack from the site can be read with it.
 */

import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { copyFile, mkdir, readdir, readFile, rename, rm, stat } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const CACHE = join(WEB, '.assets/wasm-opt');
const binaryen = dirname(createRequire(join(WEB, 'package.json')).resolve('binaryen/package.json'));
const WASM_OPT = join(binaryen, 'bin/wasm-opt');
const VERSION = JSON.parse(await readFile(join(binaryen, 'package.json'), 'utf8')).version;

const FEATURES = [
  '--enable-simd',
  '--enable-bulk-memory',
  '--enable-bulk-memory-opt',
  '--enable-nontrapping-float-to-int',
  '--enable-sign-ext',
  '--enable-mutable-globals',
  '--enable-reference-types',
  '--enable-multivalue',
  '--enable-call-indirect-overlong',
];
export const FLAGS = [...FEATURES, '-O3', '--strip-debug', '--strip-producers'];
/** The shipped optimisations without the name strip (CI's wasm-with-names artifact). */
const NAMED_FLAGS = [...FEATURES, '-O3', '-g', '--strip-producers'];

/**
 * Optimises `path` in place (`names`: keeping the name section). Returns the sizes before and
 * after, and whether the result came from the cache. Older entries for the same file are removed.
 */
export async function optimizeWasm(path, { names = false } = {}) {
  const flags = names ? NAMED_FLAGS : FLAGS;
  const name = basename(path).replace(/-[\w-]{8}\.wasm$|\.wasm$/, '');
  const dir = join(CACHE, names ? 'names' : 'release');
  const input = await readFile(path);
  const key = createHash('sha256').update(input).update(`\0binaryen ${VERSION}\0${flags.join(' ')}`).digest('hex').slice(0, 16);
  const cached = join(dir, `${name}-${key}.wasm`);
  const hit = await stat(cached).then(() => true, () => false);
  if (!hit) {
    await mkdir(dir, { recursive: true });
    // Exactly `name-<key>`: another file's name may start with this one's.
    const older = (file) => file.startsWith(`${name}-`) && /^[0-9a-f]{16}\.wasm(\.part)?$/.test(file.slice(name.length + 1));
    for (const old of await readdir(dir)) if (older(old)) await rm(join(dir, old), { force: true });
    const tmp = `${cached}.part`;
    await promisify(execFile)(process.execPath, [WASM_OPT, ...flags, path, '-o', tmp], { maxBuffer: 1 << 26 });
    await rename(tmp, cached);
  }
  await copyFile(cached, `${path}.part`);
  await rename(`${path}.part`, path);
  return { before: input.length, after: (await stat(path)).size, cached: hit };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const names = args[0] === '--names';
  const files = names ? args.slice(1) : args;
  if (!files.length) throw new Error('usage: node tools/release/wasm-opt.mjs [--names] <file.wasm>...');
  await Promise.all(
    files.map(async (file) => {
      const start = performance.now();
      const { before, after, cached } = await optimizeWasm(file, { names });
      console.log(`wasm-opt ${file}: ${before} -> ${after} bytes${cached ? ' (cached)' : ''}, ${((performance.now() - start) / 1000).toFixed(1)} s`);
    }),
  );
}
