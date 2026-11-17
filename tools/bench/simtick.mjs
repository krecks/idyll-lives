/**
 * The sim's speed as the game runs it: the wasm build (`web/src/core/wasm-pkg`, `bun run wasm`)
 * ticking a town under Node or Bun, step by step as the worker does (`Game.advance` at 10×, speed 3,
 * the snapshot written each step), with no renderer. What the browser's sim worker would manage on
 * this machine, give or take the browser's own engine (Chrome's is Node's, V8).
 *
 *   node tools/bench/simtick.mjs <worlds.json> [--world 0] [--hours 24] [--watch 320] [--pkg dir] [--runs 1]
 *   bun tools/bench/simtick.mjs ...
 *
 * `worlds.json` is `tools/worlds/sample.ts`'s (`tools/bench/tiers.sh` makes them under
 * bench-results/tiers). `--watch 320`: the camera's square of that side round the player's home
 * is the full-detail area (`set_watch`, as the renderer reports it); without it, the view is the
 * player's lot (`set_view`), as the big-world benchmark's lot view. `--pkg`: another wasm-bindgen
 * output (the M1 estimate builds its own: `tools/m1/`). `--runs`: the world loaded again and run
 * again, the best kept.
 *
 * The content is merged as the game loads it (`web/src/content/content.ts` `mergeContent`): under
 * Bun from the source, and kept in bench-results/simtick-content.json for Node (run it under Bun
 * once first).
 *
 * Prints one line a script reads:
 * `simtick: engine node rate 4100 worst_hour 2100 at 08 step_p99 31.2 ticks 108000 cpu_rate 4300`
 * (ticks a second over the run, the slowest game hour's, a step's 99th percentile in ms, and ticks a
 * second of the process's CPU time: the GC's and compiler's threads count, waiting for a core doesn't).
 */
import { existsSync, readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(name);
  if (i < 0) return fallback;
  const [, value] = args.splice(i, 2);
  return value;
};
const world = Number(flag('--world', '0'));
const hours = Number(flag('--hours', '24'));
const watchSide = flag('--watch', undefined);
const pkg = resolve(flag('--pkg', `${root}/web/src/core/wasm-pkg`));
const runs = Number(flag('--runs', '1'));
const file = args[0];
if (!file) throw new Error('simtick: which worlds file?');
const engine = typeof Bun !== 'undefined' ? 'bun' : 'node';

// The content, merged as the game does.
const merged = `${root}/bench-results/simtick-content.json`;
let content;
if (engine === 'bun') {
  const { mergeContent } = await import(`${root}/web/src/content/content.ts`);
  const dir = `${root}/web/public/content`;
  const base = readFileSync(`${dir}/base.json`, 'utf8');
  const includes = JSON.parse(base).include ?? [];
  content = JSON.stringify(mergeContent([['base.json', base], ...includes.map((n) => [n, readFileSync(`${dir}/${n}`, 'utf8')])]));
  mkdirSync(dirname(merged), { recursive: true });
  if (!existsSync(merged) || readFileSync(merged, 'utf8') !== content) writeFileSync(merged, content);
} else {
  if (!existsSync(merged)) throw new Error(`simtick: no ${merged}: run it under Bun once first`);
  content = readFileSync(merged, 'utf8');
}

const wasm = await import(`${pkg}/sim_wasm.js`);
const { memory } = wasm.initSync({ module: readFileSync(`${pkg}/sim_wasm_bg.wasm`) });

const worlds = JSON.parse(readFileSync(file, 'utf8'));
const town = worlds[world]?.town;
if (!town) throw new Error(`simtick: no world ${world} in ${file}`);
const lot = JSON.stringify(town);
// The player's lot, and (`--watch`) the camera's square round it, inside the town.
const home = town.households.find((h) => h.player)?.plot;
const plot = town.plots[home];
if (!plot) throw new Error('simtick: no player household with a home');
const view = [plot.x, plot.z, plot.x + plot.w, plot.z + plot.d];
let watch = null;
if (watchSide !== undefined) {
  const side = Number(watchSide);
  const [cx, cz] = [Math.trunc((view[0] + view[2]) / 2), Math.trunc((view[1] + view[3]) / 2)];
  const x0 = Math.min(Math.max(0, cx - Math.trunc(side / 2)), Math.max(0, town.width - side));
  const z0 = Math.min(Math.max(0, cz - Math.trunc(side / 2)), Math.max(0, town.depth - side));
  watch = [x0, z0, Math.min(town.width, x0 + side), Math.min(town.depth, z0 + side)];
}

const TICKS_PER_HOUR = 60 * 75;
const wanted = hours * TICKS_PER_HOUR;
let best = null;
for (let run = 0; run < runs; run++) {
  const game = new wasm.Game(content, lot, 7);
  game.set_view(...view);
  if (watch) game.set_watch(...watch);
  game.command(JSON.stringify({ type: 'setSpeed', speed: 3 }));
  // (The first step finds the rooms and the walkable regions from scratch: starting the game.)
  game.advance();
  const snap = () => new Float32Array(memory.buffer, game.snapshot_ptr(), game.snapshot_len());
  const tickOf = () => snap()[0];
  const start = tickOf();
  // (The snapshot's header: the tick, the day, the minute of the day.)
  const firstHour = Math.floor(snap()[2] / 60);
  let ticks = 0;
  let total = 0;
  // (The process's CPU time over the run too: on a busy machine the wall clock counts the time it
  // waited for a core, the CPU time only what it ran.)
  const cpu0 = process.cpuUsage();
  const steps = [];
  const hourly = new Array(hours).fill(0);
  while (ticks < wanted) {
    const t0 = performance.now();
    game.advance();
    const dt = performance.now() - t0;
    // (The snapshot's tick counts round at 2^24.)
    const now = (tickOf() - start + (1 << 24)) % (1 << 24);
    const ran = Math.max(1, now - ticks);
    const hour = Math.floor(ticks / TICKS_PER_HOUR);
    if (hour < hours) hourly[hour] += dt;
    ticks = now;
    total += dt;
    steps.push(dt);
  }
  const cpu = process.cpuUsage(cpu0);
  steps.sort((a, b) => a - b);
  const worstAt = hourly.indexOf(Math.max(...hourly));
  const result = {
    rate: (ticks / total) * 1000,
    worst: (TICKS_PER_HOUR / hourly[worstAt]) * 1000,
    at: (firstHour + worstAt) % 24,
    p99: steps[Math.min(steps.length - 1, Math.floor((steps.length * 99) / 100))],
    ticks,
    cpuRate: ticks / ((cpu.user + cpu.system) / 1e6),
  };
  game.free();
  if (!best || result.rate > best.rate) best = result;
}
const made = worlds[world];
console.error(`${made.kind} ${made.seed}: ${town.sims.length} residents, ${watch ? `watch ${watch.join(',')}` : `view ${view.join(',')}`}, ${hours} game hours under ${engine}`);
console.log(
  `simtick: engine ${engine} rate ${best.rate.toFixed(0)} worst_hour ${best.worst.toFixed(0)} at ${String(best.at).padStart(2, '0')} step_p99 ${best.p99.toFixed(2)} ticks ${best.ticks} cpu_rate ${best.cpuRate.toFixed(0)}`,
);
