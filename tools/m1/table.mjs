/**
 * The M1 estimate's report (tools/m1/run.sh, README.md), from the run's logs: per tier and world,
 * the sim's ticks a second measured on this machine (natively, in the container, under the M1's
 * quota, and as wasm under Node and Bun), the M1's estimated from them, and the margin against
 * the 3,600 the fastest speed needs.
 *
 *   node tools/m1/table.mjs bench-results/m1/<time>   (markdown on stdout)
 *
 * A log the run skipped (run.sh's SKIP) is taken from the newest earlier run that has it.
 */
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';

const dir = process.argv[2];
if (!dir) throw new Error('table: which run?');
const runs = dirname(dir);
const root = join(runs, '..', '..');
const tiersDir = join(root, 'bench-results', 'tiers');

/** The M1's single-core speed against the M3 Pro's (the plan's research): expected, and its band (sustained in a fanless Air, and at best). */
const RATIO = 0.72;
const BAND = [0.62, 0.78];
/** Ticks a second the fastest speed needs (60 a step, 60 steps a second), and the tier rule's floor (`web/src/game/people.ts` `TIER_FOR`). */
const NEED = 3600;
const TIER_FLOOR = 4250;

const env = (name, fallback) => process.env[name] ?? fallback;
const tiers = env('TIERS', 'full half near').split(/\s+/).filter(Boolean);
const at = env('WORLDS_AT', '0 1 2').split(/\s+/).filter(Boolean).map(Number);
const watch = env('WATCH', '320');
const hours = env('HOURS', '24');
const KINDS = ['seaside', 'river', 'forest'];

/** The file `name` of this run, else of the newest earlier run that has it (null: none). */
function logOf(name) {
  const own = join(dir, name);
  if (existsSync(own)) return readFileSync(own, 'utf8');
  const earlier = readdirSync(runs)
    .filter((d) => d < basename(dir) && existsSync(join(runs, d, name)))
    .sort()
    .pop();
  return earlier ? readFileSync(join(runs, earlier, name), 'utf8') : null;
}

/** tiers.sh's best run of `label` on tier `t`'s world `k` (`bench:` line), and the residents. */
function bench(label, t, k) {
  const d = join(tiersDir, label);
  if (!existsSync(d)) return null;
  const suffix = watch ? `-w${watch}` : '';
  let best = null;
  let residents = null;
  for (const f of readdirSync(d).filter((f) => f.startsWith(`${t}${k}${suffix}-r`) && f.endsWith('.log'))) {
    const text = readFileSync(join(d, f), 'utf8');
    residents ??= Number(text.match(/(\d+) residents, grid/)?.[1] ?? NaN);
    const m = text.match(/^bench: rate (\d+) worst_hour (\d+) at (\d+) step_p99 ([\d.]+) full (\d+) full_most (\d+)/m);
    if (m && (!best || Number(m[1]) > best.rate)) best = { rate: Number(m[1]), worst: Number(m[2]), at: m[3], p99: Number(m[4]), full: Number(m[5]) };
  }
  return best && { ...best, residents };
}

/** simtick's best line for `engine` on tier `t`'s world `k`. */
const wasmLog = logOf('wasm.log') ?? '';
function wasm(engine, t, k) {
  let best = null;
  for (const m of wasmLog.matchAll(/^(\w+) (\d+) simtick: engine (\w+) rate (\d+) worst_hour (\d+) at (\d+) step_p99 ([\d.]+)(?: ticks \d+ cpu_rate (\d+))?/gm)) {
    if (m[1] !== t || Number(m[2]) !== k || m[3] !== engine) continue;
    // (Ticks a second of CPU time: what it ran, not what it waited for a core on a busy machine.)
    const cpu = m[8] ? Number(m[8]) : Number(m[4]);
    if (!best || cpu > best.cpu) best = { rate: Number(m[4]), cpu, worst: Number(m[5]), at: m[6], p99: Number(m[7]) };
  }
  return best;
}

const calibration = (name) => {
  const m = (logOf(name) ?? '').match(/calibrate: 7z (\d+) tick_us ([\d.]+)/);
  return m ? { mips: Number(m[1]), us: Number(m[2]) } : null;
};
const [m3, m1] = [calibration('calibrate-m3.log'), calibration('calibrate-m1.log')];
const quota = m3 && m1 ? { sevenZip: m1.mips / m3.mips, tick: m3.us / m1.us } : null;

const n = (v) => (v == null || Number.isNaN(v) ? '–' : Math.round(v).toLocaleString('en-GB'));
const pc = (v) => (v == null ? '–' : `${v >= 0 ? '+' : ''}${Math.round(v * 100)} %`);
const lines = [];
const say = (s = '') => lines.push(s);

say(`# M1 estimate, ${basename(dir)}`);
say();
say(`The sim's ticks a second over ${hours} game hours, ${watch ? `the camera's ${watch}-tile square round the player's home at full detail at speed 5 (big_world's WATCH, simtick's --watch)` : "the view on the player's lot"}; best of ${env('ROUNDS', '1')} each.`);
say(`The fastest speed needs ${n(NEED)}; a preset gets a tier when the estimated wasm rate on its slowest intended machine is at least ${n(TIER_FLOOR)}.`);
say();
say('## Calibration');
say();
if (quota) {
  say(`| | 7-Zip 1 thread (MIPS) | tickbench (µs a tick) |`);
  say(`|---|---:|---:|`);
  say(`| container (m3) | ${n(m3.mips)} | ${m3.us.toFixed(1)} |`);
  say(`| under the quota (m1, ${(Number(env('M1_QUOTA', '7200')) / 100).toFixed(0)} %) | ${n(m1.mips)} | ${m1.us.toFixed(1)} |`);
  say(`| ratio | ${quota.sevenZip.toFixed(2)} | ${quota.tick.toFixed(2)} |`);
} else say('No calibration logs.');
say();
say(`The estimate scales this machine's numbers by ${RATIO} (an M1's single core against an M3 Pro's), with a band of ${BAND[0]} (sustained, fanless) to ${BAND[1]}. The quota's own ratio above shows the container's "M1" runs at about that.`);
const limits = logOf('limits.log');
if (limits) say(`The m1 container: ${limits.trim()}.`);
say();
say('## Ticks a second');
say();
say('Wasm: ticks a second by the wall clock, and (in brackets) of the process\'s CPU time, which leaves out the waits for a core on a busy machine; the M1 wasm estimate scales the latter.');
say();
say('| tier | world | residents | M3 native | container | under quota | wasm Node | wasm Bun | M1 native est. | M1 wasm est. (band) | margin vs 3,600 | slowest hour (M1 wasm est.) |');
say('|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|');
const verdict = {};
for (const t of tiers) {
  for (const k of at) {
    const native = bench('m1-native', t, k);
    const ctr = bench('m1-ctr-m3', t, k);
    const quotaRun = bench('m1-ctr-m1', t, k);
    const node = wasm('node', t, k);
    const bun = wasm('bun', t, k);
    const est = node ? node.cpu * RATIO : null;
    const band = node ? BAND.map((r) => node.cpu * r) : null;
    // (The slowest hour by the wall clock, scaled as the CPU time scaled the run.)
    const worst = node ? node.worst * (node.cpu / node.rate) * RATIO : null;
    (verdict[t] ??= []).push({ est, low: band?.[0], m3: node?.cpu });
    say(
      `| ${t} | ${KINDS[k]} | ${native?.residents ?? '–'} | ${n(native?.rate)} | ${n(ctr?.rate)} | ${n(quotaRun?.rate)} | ${n(node?.rate)} (${n(node?.cpu)}) | ${n(bun?.rate)} (${n(bun?.cpu)}) | ${n(native ? native.rate * RATIO : null)} | ${n(est)} (${band ? band.map(n).join('–') : '–'}) | ${pc(est == null ? null : est / NEED - 1)} | ${n(worst)} (${node?.at ?? '–'}:00) |`,
    );
  }
}
say();
say('## Per tier (the slowest world)');
say();
say(`| tier | M3 wasm | M1 wasm est. | at the band's low end | ≥ ${n(TIER_FLOOR)} on an M1? | on an M3? |`);
say('|---|---:|---:|---:|---|---|');
for (const t of tiers) {
  const rows = verdict[t].filter((r) => r.est != null);
  if (!rows.length) continue;
  const slowest = rows.reduce((a, r) => (r.est < a.est ? r : a));
  say(`| ${t} | ${n(slowest.m3)} | ${n(slowest.est)} | ${n(slowest.low)} | ${slowest.est >= TIER_FLOOR ? 'yes' : 'no'} | ${slowest.m3 >= TIER_FLOOR ? 'yes' : 'no'} |`);
}
say();
say('## What this cannot say');
say();
say('- **The GPU and the frame time.** Docker has no GPU: the renderer is not run, and the sim worker shares the M1 with it and the main thread in the game.');
say('- **Unified-memory pressure.** The quota container is given 8 GB (as much as Docker Desktop has), but an M1 shares its 8 GB with the GPU, the browser and the system; swapping is not modelled.');
say('- **Thermals.** A fanless M1 Air throttles under a sustained load; the band\'s low end stands for that, not a measurement.');
say('- **The browser.** Node runs the same V8 as Chrome, but not Safari\'s or Firefox\'s engines; the worker\'s messaging and the UI updates are not counted.');
say('- **A shared machine.** Other work on this machine slows every run; the best run of each is kept, but the numbers are as good as the quietest of them.');
console.log(lines.join('\n'));
