#!/usr/bin/env node
/**
 * Timing the repo's tools (crates/tools): each case's Rust binary and, while its Python
 * predecessor is still in the tree, the Python script beside it. Median and best wall time of
 * `--runs` after one warm-up, and peak memory (`/usr/bin/time -l`, macOS). Writes results.json and
 * summary.md to `--out` (default <repo>/bench-results/tools-<time>/).
 *
 *   node tools/bench/tools.mjs [--runs 5] [--only characters-build,check-glb] [--out <dir>]
 *                              [--tex-src <dir>]   (Poly Haven/ambientCG sources for `tex process`)
 *                              [--content <dir>]   (content with a Validate category, for `content`)
 *                              [--content-id <id> --content-group <group>]   (`content in --dry-run`)
 *
 * Builds the release binaries first (`cargo build -p ll-tools --release --bins`) and runs them
 * directly, so cargo's own start-up isn't timed. Everything runs one process at a time with one
 * thread for numpy's maths libraries (the Rust tools are single-threaded), into scratch folders:
 * nothing in the repo is written.
 */

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const runs = Number(flag('runs', 5));
const only = flag('only', '').split(',').filter(Boolean);
const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
const out = flag('out', join(ROOT, 'bench-results', `tools-${stamp}`));
const texSrc = flag('tex-src');
const contentId = flag('content-id');
const contentGroup = flag('content-group', 'bench');
const contentDir = flag('content');
const withContent = contentDir ? ['--content', contentDir] : [];

const BIN = join(ROOT, 'target/release');
const ONE_THREAD = { OMP_NUM_THREADS: '1', OPENBLAS_NUM_THREADS: '1', MKL_NUM_THREADS: '1', VECLIB_MAXIMUM_THREADS: '1' };
const py = (script, ...rest) => (existsSync(join(ROOT, script)) ? ['python3', '-B', ...(script.endsWith('check_glb.py') ? ['-I'] : []), join(ROOT, script), ...rest] : null);
const rs = (bin, ...rest) => [join(BIN, bin), ...rest];
const scratch = () => mkdtempSync(join(tmpdir(), 'tools-bench-'));

/** Each case: a name, and per language a function giving [argv, cwd] for one run (null: absent). */
const CASES = [
  { name: 'characters-build', note: 'the whole set, then its check', python: (d) => py('tools/characters/prim/build.py', d), rust: (d) => rs('characters', 'build', d) },
  { name: 'characters-check', note: 'the committed set', python: () => py('tools/characters/prim/check.py', 'web/public/assets/characters'), rust: () => rs('characters', 'check', 'web/public/assets/characters') },
  { name: 'check-glb', note: 'every shipped model', python: () => py('tools/art/blender/check_glb.py'), rust: () => rs('check-glb') },
  { name: 'content-list', note: 'the Validate category', python: () => py('tools/content/validate.py', 'list', ...withContent), rust: () => rs('content', 'list', ...withContent) },
  contentId && { name: 'content-in', note: `in ${contentId} --dry-run`, python: () => py('tools/content/validate.py', 'in', contentId, '--group', contentGroup, '--dry-run', ...withContent), rust: () => rs('content', 'in', contentId, '--group', contentGroup, '--dry-run', ...withContent) },
  { name: 'tex-clouds', note: '1024x256 PNG', python: (d) => py('tools/art/clouds.py', join(d, 'clouds.png')), rust: (d) => rs('tex', 'clouds', join(d, 'clouds.png')) },
  { name: 'tex-materials', note: 'materials.json', cwd: true, python: () => py('tools/art/materials.py'), rust: () => rs('tex', 'materials') },
  texSrc && { name: 'tex-process', note: 'albedo/normal/arm JPGs', python: (d) => py('tools/art/process.py', texSrc, d), rust: (d) => rs('tex', 'process', texSrc, d) },
].filter(Boolean).filter((c) => !only.length || only.includes(c.name));

function once(argv, cwd) {
  const t0 = process.hrtime.bigint();
  const r = spawnSync('/usr/bin/time', ['-l', ...argv], { cwd, env: { ...process.env, ...ONE_THREAD }, encoding: 'utf8', maxBuffer: 1 << 28 });
  const ms = Number(process.hrtime.bigint() - t0) / 1e6;
  if (r.status !== 0) throw new Error(`${argv.join(' ')} exited ${r.status}:\n${(r.stdout + r.stderr).slice(-2000)}`);
  const rss = Number(/(\d+)\s+maximum resident set size/.exec(r.stderr)?.[1] ?? NaN);
  return { ms, rssMB: rss / 2 ** 20 };
}

function measure(c, lang) {
  const times = [];
  let rssMB = 0;
  for (let i = 0; i <= runs; i++) {
    const dir = scratch();
    const argv = c[lang](dir);
    try {
      if (!argv) return null;
      const r = once(argv, c.cwd ? dir : ROOT);
      if (i > 0) times.push(r.ms); // run 0 warms the caches
      rssMB = Math.max(rssMB, r.rssMB);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  }
  times.sort((a, b) => a - b);
  return { median: times[Math.floor(times.length / 2)], best: times[0], runs: times, rssMB };
}

execFileSync('cargo', ['build', '-q', '-p', 'll-tools', '--release', '--bins', '--locked'], { cwd: ROOT, stdio: 'inherit' });
mkdirSync(out, { recursive: true });
const results = [];
for (const c of CASES) {
  const r = { name: c.name, note: c.note, rust: measure(c, 'rust'), python: measure(c, 'python') };
  results.push(r);
  const f = (m) => (m ? `${m.median.toFixed(0)} ms` : '—');
  console.log(`${c.name.padEnd(18)} rust ${f(r.rust).padStart(9)}   python ${f(r.python).padStart(9)}`);
}
writeFileSync(join(out, 'results.json'), JSON.stringify({ runs, date: new Date().toISOString(), results }, null, 1));
const ms = (m) => (m ? (m.median >= 1000 ? `${(m.median / 1000).toFixed(2)} s` : `${m.median.toFixed(0)} ms`) : '—');
const mb = (m) => (m ? `${m.rssMB.toFixed(0)} MB` : '—');
const md = [
  `Tools, median of ${runs} (after a warm-up), one thread, ${new Date().toISOString().slice(0, 10)}`,
  '',
  '| Case | What | Python | Rust | Speed-up | Peak memory (Py / Rust) |',
  '|---|---|---:|---:|---:|---:|',
  ...results.map((r) => `| ${r.name} | ${r.note} | ${ms(r.python)} | ${ms(r.rust)} | ${r.python && r.rust ? `${(r.python.median / r.rust.median).toFixed(1)}×` : '—'} | ${mb(r.python)} / ${mb(r.rust)} |`),
  '',
].join('\n');
writeFileSync(join(out, 'summary.md'), md);
console.log(`\n${md}\nwrote ${out}`);
