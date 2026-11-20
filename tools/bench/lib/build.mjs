/**
 * A production build made for the benchmark (`VITE_BENCH=1`: the hooks a bench needs kept, `web/src/debug/hooks.ts`;
 * no voice or language model copied, no licence file), built once per content and reused:
 * `bench-results/.dist/<key>/`, the key a hash of the tree's web/ (its git tree, the uncommitted changes, the untracked
 * files), the git-ignored files a build takes in (the wasm packages, the bench scenes) and how it was built. The textures
 * stay as their sources by default (`LL_ASSETS=source`, `assets: 'source'`): pictures compare pixel for pixel with
 * the dev server's, and a worktree with no `web/.assets` cache doesn't encode every texture first; `assets: 'prod'`
 * builds what players get. Builds run one after another, never side by side (a lock in the store).
 *
 *   import { benchBuild } from './lib/build.mjs';
 *   const { dir } = await benchBuild({ tree: '../base' });   // then serve({ mode: 'preview', web, outDir: dir })
 *
 * `bun tools/bench/lib/build.mjs [--tree <dir>] [--assets source|prod]` builds (or finds) one and prints its folder.
 */

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, renameSync, rmSync, statSync, utimesSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));

/** Where the builds are kept (this tree's, for every tree built): the newest `KEEP` (about 190 MB each), none older than `MAX_AGE_DAYS`. */
const STORE = join(ROOT, 'bench-results', '.dist');
const KEEP = 4;
const MAX_AGE_DAYS = 7;
/** What the key is made of besides the tree: bump it when what a build holds changes. */
const RECIPE = 'bench-1';

const git = (tree, args) => {
  const r = spawnSync('git', ['-C', tree, ...args], { encoding: 'buffer', maxBuffer: 1 << 30 });
  if (r.status !== 0) throw new Error(`git ${args.join(' ')} in ${tree}: ${r.stderr}`);
  return r.stdout;
};

/** Every file under `dir` (sorted), for the git-ignored inputs. */
function files(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter((e) => e.isFile())
    .map((e) => join(e.parentPath, e.name))
    .sort();
}

/** The key of `tree`'s web/ built with `assets`. */
export function buildKey({ tree = ROOT, assets = 'source' } = {}) {
  const h = createHash('sha256');
  h.update(`${RECIPE} assets=${assets}\n`);
  h.update(git(tree, ['rev-parse', 'HEAD:web']));
  h.update(git(tree, ['diff', 'HEAD', '--binary', '--', 'web']));
  for (const f of git(tree, ['ls-files', '--others', '--exclude-standard', '--', 'web']).toString().split('\n').filter(Boolean)) {
    h.update(f);
    h.update(readFileSync(join(tree, f)));
  }
  // (Git-ignored, but in the build: the wasm packages and the bench scenes made by scene.mjs.)
  for (const dir of ['web/src/core/wasm-pkg', 'web/src/audio/wasm-pkg', 'web/src/voice/wasm-pkg', 'web/public/bench']) {
    for (const f of files(join(tree, dir))) {
      h.update(f.slice(tree.length));
      h.update(readFileSync(f));
    }
  }
  return h.digest('hex').slice(0, 20);
}

/** The newest `KEEP` builds kept, none older than `MAX_AGE_DAYS` (a build used is touched: its age is since then). */
function prune() {
  const oldest = Date.now() - MAX_AGE_DAYS * 86_400_000;
  const builds = readdirSync(STORE)
    .filter((d) => /^[0-9a-f]{20}$/.test(d))
    .map((d) => ({ d, t: statSync(join(STORE, d)).mtimeMs }))
    .sort((a, b) => b.t - a.t);
  for (const [i, { d, t }] of builds.entries()) if (i >= KEEP || t < oldest) rmSync(join(STORE, d), { recursive: true, force: true });
}

/** The build of `tree` (this one by default): `{ dir, key, hit, secs }`, built on a miss. */
export async function benchBuild({ tree = ROOT, assets = 'source', log = console.log } = {}) {
  tree = resolve(tree);
  const web = join(tree, 'web');
  if (!existsSync(join(web, 'src/core/wasm-pkg/sim_wasm_bg.wasm'))) throw new Error(`${tree}: its wasm isn't built (bun run wasm and bun run sound in its web/)`);
  const key = buildKey({ tree, assets });
  const dir = join(STORE, key);
  mkdirSync(STORE, { recursive: true });
  const lock = join(STORE, 'lock');
  const started = performance.now();
  // One build at a time on this machine (each wants every core); a lock whose holder is gone is taken over.
  for (;;) {
    try {
      mkdirSync(lock);
      writeFileSync(join(lock, 'pid'), String(process.pid));
      break;
    } catch {
      const holder = Number(existsSync(join(lock, 'pid')) ? readFileSync(join(lock, 'pid'), 'utf8') : 0);
      let alive = false;
      try {
        alive = holder > 0 && process.kill(holder, 0);
      } catch {}
      if (!alive) rmSync(lock, { recursive: true, force: true });
      else await new Promise((r) => setTimeout(r, 2000));
    }
  }
  try {
    if (existsSync(join(dir, 'index.html'))) {
      const now = new Date();
      utimesSync(dir, now, now);
      prune();
      log(`bench build ${key} kept (${tree})`);
      return { dir, key, hit: true, secs: (performance.now() - started) / 1000 };
    }
    log(`bench build ${key} of ${tree} (assets: ${assets}) …`);
    const tmp = `${dir}.new`;
    rmSync(tmp, { recursive: true, force: true });
    const env = { ...process.env, VITE_BENCH: '1', ...(assets === 'source' ? { LL_ASSETS: 'source' } : {}) };
    const r = spawnSync(join(web, 'node_modules/.bin/vite'), ['build', '--outDir', tmp, '--emptyOutDir'], { cwd: web, stdio: ['ignore', 'ignore', 'inherit'], env });
    if (r.status !== 0) throw new Error(`the bench build of ${tree} failed (${r.status})`);
    renameSync(tmp, dir);
    prune();
    const secs = (performance.now() - started) / 1000;
    log(`bench build ${key} made in ${secs.toFixed(0)} s`);
    return { dir, key, hit: false, secs };
  } finally {
    rmSync(lock, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  const args = process.argv.slice(2);
  const flag = (name, fallback) => {
    const i = args.indexOf(`--${name}`);
    return i >= 0 && args[i + 1] !== undefined ? args[i + 1] : fallback;
  };
  const { dir } = await benchBuild({ tree: flag('tree', ROOT), assets: flag('assets', 'source') });
  console.log(dir);
}
