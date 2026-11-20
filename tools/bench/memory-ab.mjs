#!/usr/bin/env bun
/**
 * The renderer's memory, two builds against each other: `gameplay.mjs` run on build A and build B in
 * turn (A B, B A, A B, …, so a drift in the machine falls on both alike), `--runs` times each, each run
 * a new browser through the menus into the same world (`--code`) and the same kind of home (`--home`),
 * with its collected reads (paused, every heap collected, a memory-pressure notification, then
 * `phys_footprint` and the allocators). Each run's number is the median over its reads; the builds' are
 * the medians over their runs, with the least and most, and B − A for every key: the footprints, each
 * process's allocators (V8, PartitionAlloc, malloc, the GPU's), the sim's WASM, the workers' heaps, the
 * renderer's census and the world's plots, residents and objects.
 *
 * A build is a tree (a checkout or a worktree, its wasm built: `bun run wasm` and `bun run sound` in it),
 * served by its own Vite on a free port, or the URL of a dev server already running. `--prod`: each tree's
 * production build for the benchmark instead (`lib/build.mjs`, made one after the other and kept while the tree's web/
 * is the same), served by `vite preview`. The harness is this tree's `gameplay.mjs` for both, so both are measured the
 * same way.
 *
 *   bun tools/bench/memory-ab.mjs --a <tree|url> --b <tree|url> [--runs 3] [--prod] [--out <dir>] [-- <gameplay.mjs options>]
 *
 * For example, release/0.8.3 against this tree, 5 runs each, 300 s of play, reads at 30/90/150/210/270 s:
 *
 *   git worktree add --detach ../base 85936d1d0
 *   ln -s "$(pwd)/web/node_modules" ../base/web/node_modules
 *   (cd ../base/web && bun run wasm && bun run sound)
 *   bun tools/bench/memory-ab.mjs --a ../base --b . --runs 5
 *
 * Writes each run's `gameplay.mjs` output under `--out` (default `bench-results/memory-ab-<time>/`,
 * `a-1/`, `b-1/`, …) and `compare.json` and `compare.md` there, and prints the table.
 */
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:net';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { cpus, loadavg } from 'node:os';
import { basename, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { benchBuild } from './lib/build.mjs';

const HERE = fileURLToPath(new URL('./', import.meta.url));
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const cut = process.argv.indexOf('--');
const args = process.argv.slice(2, cut < 0 ? undefined : cut);
const passed = cut < 0 ? [] : process.argv.slice(cut + 1);
/** A run's steady window (s): gameplay.mjs's `--secs` as passed, else its default. */
const runSecs = passed.includes('--secs') ? Number(passed[passed.indexOf('--secs') + 1]) || 150 : 150;
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
if (args.includes('--help') || args.includes('-h') || !flag('a', null) || !flag('b', null)) {
  console.log(`Usage: bun tools/bench/memory-ab.mjs --a <tree|url> --b <tree|url> [--runs 3] [--prod] [--out <dir>] [-- <gameplay.mjs options>]

  --a, --b     a build: a tree (its web/ served by its own Vite on a free port) or a dev server's URL
  --runs 3     runs of each, in turn (A B, B A, …)
  --prod       each tree's production build for the benchmark (lib/build.mjs), served by vite preview
  --out <dir>  default bench-results/memory-ab-<time>/
  -- …         options for gameplay.mjs (--secs, --reads, --code, --home, --renderer, --preset, …)`);
  process.exit(flag('a', null) && flag('b', null) ? 0 : 2);
}
const runs = Number(flag('runs', '3'));
const prod = args.includes('--prod');
if (!Number.isInteger(runs) || runs < 1) {
  console.error('memory-ab.mjs: --runs must be a positive integer');
  process.exit(2);
}
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = resolve(flag('out', join(ROOT, 'bench-results', `memory-ab-${stamp}`)));
mkdirSync(out, { recursive: true });

/** A free TCP port on this machine. */
function freePort() {
  return new Promise((ok, no) => {
    const srv = createServer();
    srv.on('error', no);
    srv.listen(0, '127.0.0.1', () => {
      const { port } = srv.address();
      srv.close(() => ok(port));
    });
  });
}

const servers = [];
process.on('exit', () => servers.forEach((s) => s.kill()));
/** A build to measure: a URL as it is, or a tree served by its own Vite (the tree's own dependencies and wasm). */
async function serve(name, where) {
  if (/^https?:\/\//.test(where)) return { name, where, url: where.replace(/\/$/, ''), commit: null };
  const web = join(resolve(where), 'web');
  const vite = join(web, 'node_modules/.bin/vite');
  if (!existsSync(vite)) throw new Error(`${where}: no web/node_modules/.bin/vite (link or install its node_modules)`);
  if (!existsSync(join(web, 'src/core/wasm-pkg/sim_wasm_bg.wasm'))) throw new Error(`${where}: its wasm isn't built (bun run wasm and bun run sound in its web/)`);
  const port = await freePort();
  const dist = prod ? (await benchBuild({ tree: where })).dir : null;
  const server = spawn(vite, [...(dist ? ['preview', '--outDir', dist] : []), '--port', String(port), '--strictPort', '--host', '127.0.0.1'], { cwd: web, stdio: ['ignore', 'pipe', 'inherit'] });
  servers.push(server);
  await new Promise((ok, no) => {
    server.stdout.on('data', (d) => String(d).includes('Local') && ok());
    server.on('exit', (code) => no(new Error(`${where}: vite exited (${code})`)));
    setTimeout(() => no(new Error(`${where}: vite did not report "Local" in 120 s`)), 120_000);
  });
  const commit = spawnSync('git', ['-C', resolve(where), 'rev-parse', '--short', 'HEAD'], { encoding: 'utf8' }).stdout.trim() || null;
  return { name, where: resolve(where), url: `http://127.0.0.1:${port}`, commit };
}

const builds = [await serve('a', flag('a', null)), await serve('b', flag('b', null))];
for (const b of builds) console.log(`${b.name.toUpperCase()}: ${b.where}${b.commit ? ` (${b.commit})` : ''} at ${b.url}`);

/** Each build's runs: the per-run medians of its collected reads by key, and what it played. */
const results = { a: [], b: [] };
const failures = [];
for (let i = 0; i < runs; i++) {
  for (const b of i % 2 ? [builds[1], builds[0]] : builds) {
    const dir = join(out, `${b.name}-${i + 1}`);
    const load = loadavg()[0];
    console.log(`\n${b.name.toUpperCase()} run ${i + 1}/${runs} (load ${load.toFixed(1)} on ${cpus().length} cores)`);
    // (A run that outlives its window by ten minutes is stuck, not slow: it is stopped and counted as failed.)
    const r = spawnSync('bun', [join(HERE, 'gameplay.mjs'), '--url', b.url, '--runs', '1', '--out', dir, ...passed], { stdio: 'inherit', cwd: ROOT, env: process.env, timeout: (runSecs + 600) * 1000, killSignal: 'SIGTERM' });
    const file = join(dir, 'summary.json');
    if (r.status !== 0 || !existsSync(file)) {
      failures.push(`${b.name}-${i + 1}: gameplay.mjs ${r.error?.code === 'ETIMEDOUT' ? `stopped after ${runSecs + 600} s` : `exited ${r.status}`}`);
      continue;
    }
    const run = JSON.parse(readFileSync(file, 'utf8')).runs[0];
    const medians = Object.fromEntries(Object.entries(run.memory.collected ?? {}).map(([k, v]) => [k, v.median]));
    results[b.name].push({ run: i + 1, load: +load.toFixed(1), played: run.played, medians, spreads: Object.fromEntries(Object.entries(run.memory.collected ?? {}).map(([k, v]) => [k, v.spread])) });
  }
}
servers.forEach((s) => s.kill());

const median = (xs) => {
  const s = xs.filter(Number.isFinite).sort((p, q) => p - q);
  if (!s.length) return null;
  const n = s.length;
  return +(n % 2 ? s[(n - 1) / 2] : (s[n / 2 - 1] + s[n / 2]) / 2).toFixed(1);
};
// (Allocators that hold nothing in either build are left out.)
const keys = [...new Set([...results.a, ...results.b].flatMap((r) => Object.keys(r.medians)))].filter((k) => k.startsWith('count ') || [...results.a, ...results.b].some((r) => Math.abs(r.medians[k] ?? 0) >= 0.5));
const rows = keys.map((key) => {
  const of = (side) => results[side].map((r) => r.medians[key]).filter(Number.isFinite);
  const [xa, xb] = [of('a'), of('b')];
  const [ma, mb] = [median(xa), median(xb)];
  const diff = ma != null && mb != null ? +(mb - ma).toFixed(1) : null;
  return {
    key,
    a: { median: ma, min: xa.length ? Math.min(...xa) : null, max: xa.length ? Math.max(...xa) : null, n: xa.length },
    b: { median: mb, min: xb.length ? Math.min(...xb) : null, max: xb.length ? Math.max(...xb) : null, n: xb.length },
    diff,
    pct: diff != null && ma ? +((diff / ma) * 100).toFixed(1) : null,
  };
});
const worlds = Object.fromEntries(['a', 'b'].map((s) => [s, [...new Set(results[s].map((r) => `${r.played?.code ?? '?'} · ${r.played?.homeKind ?? '?'} ${r.played?.home ?? '?'} (${r.played?.household ?? '?'})`))]]));
const f = (x) => (x == null ? '—' : x);
const md = [
  `# Memory, A against B: ${runs} run${runs === 1 ? '' : 's'} each, in turn`,
  '',
  ...builds.map((b) => `- ${b.name.toUpperCase()}: \`${b.where}\`${b.commit ? ` (${b.commit})` : ''}; played ${worlds[b.name].join('; ') || '—'}`),
  `- gameplay.mjs options: \`${passed.join(' ') || '(its defaults)'}\``,
  `- load at each run's start: A ${results.a.map((r) => r.load).join(', ')}; B ${results.b.map((r) => r.load).join(', ')}`,
  ...(failures.length ? [`- **failed:** ${failures.join('; ')}`] : []),
  '',
  "Each run's number is the median of its collected reads; here the median over the runs, (least–most), MB unless a count.",
  '',
  '| key | A | B | B − A | % |',
  '| --- | --- | --- | --- | --- |',
  ...rows.map((r) => `| ${r.key} | ${f(r.a.median)} (${f(r.a.min)}–${f(r.a.max)}) | ${f(r.b.median)} (${f(r.b.min)}–${f(r.b.max)}) | ${r.diff == null ? '—' : `${r.diff >= 0 ? '+' : ''}${r.diff}`} | ${r.pct == null ? '—' : `${r.pct >= 0 ? '+' : ''}${r.pct} %`} |`),
  '',
];
writeFileSync(join(out, 'compare.md'), md.join('\n'));
writeFileSync(join(out, 'compare.json'), JSON.stringify({ builds, runs, options: passed, results, rows, failures }, null, 2));
console.log(`\n${md.join('\n')}\nwritten to ${out} (compare.md, compare.json, ${basename(out)}/<a|b>-<n>/)`);
if (worlds.a.length > 1 || worlds.b.length > 1) console.warn('note: a build played more than one world or home across its runs (see compare.json results[].played)');
process.exit(failures.length ? 1 : 0);
