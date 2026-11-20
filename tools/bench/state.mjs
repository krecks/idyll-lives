#!/usr/bin/env bun
/**
 * Makes a game state without a browser: the new game a world's code (or scene.mjs's recipe) starts, or a save, run
 * to an exact tick by the sim's own WebAssembly (\`web/src/core/wasm-pkg\`, \`bun run wasm\`) under Bun, as the sim
 * worker runs it, and saved paused. What \`?state=<name>\` (\`debug-states/\`) and \`?bench=<name>\`
 * (\`web/public/bench/<name>.json.gz\`) open, and what \`__benchTime.load\` swaps into an open page.
 *
 *   bun tools/bench/state.mjs --code country-lrstm-7 --home house --hour 13 --name day13
 *   bun tools/bench/state.mjs --scene --world country --options town=village --seed 7 --home town --hour 13 --name town-day --bench
 *   bun tools/bench/state.mjs --from town-day --ticks 600 --name town-day-10s
 *
 * Where the game comes from (one of):
 * - \`--code <world code>\` (default \`country-lrstm-7\`, gameplay.mjs's): the five steps of New game as the screens run
 *   them (\`ui/screens/Create*.svelte\`, \`ChooseHousehold.svelte\`): the land, the layout proposed, the town built at
 *   the build's first seed, its households from the code's people seed, then Play on a household offered on the first
 *   page whose home is \`--home house|flat|any\` (the \`--household\`-th of them, from 0) and Move in, the homes still for
 *   sale filled at \`--tier\` (full: every preset's).
 * - \`--scene\`: scene.mjs's recipe (\`--world\`, \`--options\`, \`--seed\`, \`--home first|town|flat\`): the world generated
 *   at once, its neighbours and a household of four from the seed, moving into a home for sale.
 * - \`--from <file or name>\`: a save as it is (a \`.json\`, a \`.json.gz\`, or the name of one in \`debug-states/\` or
 *   \`web/public/bench/\`).
 *
 * To when: \`--hour 13\` (5 minutes before it on day 1, as scene.mjs plays to), \`--at 2:07:30\` (day 2, 07:30),
 * \`--tick <n>\` (the sim's tick count), or \`--ticks <n>\` past where it starts. The sim's dice: the people seed's (as
 * Move in has them), or the page's \`?seed=\` as \`--sim-seed\` (\`--scene\` takes \`--seed\`, as scene.mjs's page does).
 *
 * The sim simulates in full detail only round what the camera watches (\`sim_core::lod\`): here the player's lot is the
 * view and \`--watch 320\` the square of that side round it (about what the opening camera streams), as simtick.mjs's;
 * \`--watch all\` the whole town. A page that plays the state watches what its own camera streams, so the residents out
 * of full detail move otherwise from there on.
 *
 * The state is cached in \`bench-results/.states/\` by what makes it: the town file (so the generator's and the
 * content's code are in it), the sim's wasm, the dice, the watch and the tick; \`--fresh\` makes it again. \`--name\`
 * copies it to \`debug-states/<name>.json\` (\`?state=<name>\`), with \`--bench\` also to \`web/public/bench/<name>.json.gz\`
 * (\`?bench=<name>\`). Prints one line a script reads: \`state: <path> tick <n> day <d> minute <m> made|cached <ms>\`.
 * A state is this build's: another build makes its own from the same code at the same tick (an older one can't
 * read a newer save).
 */

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { gunzipSync, gzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = ROOT + 'web/';
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf('--' + name);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes('--' + name);
const stop = (message) => {
  console.error('state: ' + message);
  process.exit(1);
};

// The page's world in Bun: its files from web/public (the loaders fetch from the page's address).
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
globalThis.fetch = async (input) => {
  const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
  const path = WEB + 'public' + decodeURIComponent(url.pathname);
  return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
};

const pkg = flag('pkg', WEB + 'src/core/wasm-pkg');
const wasmBytes = readFileSync(pkg + '/sim_wasm_bg.wasm');
const sha = (text) => createHash('sha256').update(text).digest('hex');

/** The save to start from: a file, or a name in debug-states/ or web/public/bench/. */
function readSave(what) {
  const tries = [what, ROOT + 'debug-states/' + what + '.json', WEB + 'public/bench/' + what + '.json.gz'];
  const file = tries.find((f) => existsSync(f));
  if (!file) stop('no save ' + what + ' (tried ' + tries.join(', ') + ')');
  const bytes = readFileSync(file);
  return (file.endsWith('.gz') ? gunzipSync(bytes) : bytes).toString('utf8');
}

const { Content } = await import(WEB + 'src/content/content.ts');
const content = await Content.load('http://localhost/content/base.json');

/** The town file and the sim's dice of a new game, made as the page makes them. */
async function newGame() {
  const { AssetRegistry } = await import(WEB + 'src/assets/registry.ts');
  const { objectInfos } = await import(WEB + 'src/game/housegen.ts');
  const { useTownContent, generateWorld } = await import(WEB + 'src/game/town.ts');
  const { runTownJob } = await import(WEB + 'src/game/townJobs.ts');
  const { peopleStreams, populate } = await import(WEB + 'src/game/people.ts');
  const { firstCode, layoutPlan, moveInRequest, offerPage, optionsOf, peopleSeedOf, townOnMoveIn } = await import(WEB + 'src/ui/newGame.ts');
  const { homeKey, homesForSale } = await import(WEB + 'src/ui/homes.ts');
  const { randomHousehold } = await import(WEB + 'src/game/household.ts');
  const { writeCode } = await import(WEB + 'src/game/worldCode.ts');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = (name) => JSON.parse(readFileSync(WEB + 'public/content/' + name, 'utf8'));
  const t = { ...json('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const homeKind = flag('home', has('scene') ? 'first' : 'house');
  const tier = flag('tier', 'full');
  let request;
  let simSeed;
  let made;
  if (has('scene')) {
    // scene.mjs's page, run here: the world at once, its neighbours and a household of four from the seed.
    const { worldFlags } = await import('./worldFlags.mjs');
    const { world, options } = worldFlags(flag);
    const seed = flag('seed', null) === null ? null : Number(flag('seed', null));
    if (seed === null) stop('--scene needs --seed (the same town every time)');
    const empty = generateWorld(content, t, world, seed, undefined, undefined, options);
    const dice = Math.random;
    const seeded = (n) => {
      let a = (n ^ 0x9e3779b9) >>> 0;
      Math.random = () => {
        a = (a + 0x6d2b79f5) >>> 0;
        let x = Math.imul(a ^ (a >>> 15), 1 | a);
        x = (x + Math.imul(x ^ (x >>> 7), 61 | x)) ^ x;
        return ((x ^ (x >>> 14)) >>> 0) / 4294967296;
      };
    };
    seeded(seed);
    const town = populate(content, assets, t, empty);
    seeded(seed + 1);
    const household = randomHousehold(content, assets, 4, town.households.map((h) => h.household.name));
    Math.random = dice;
    const vacant = homesForSale(content, t, town);
    const houses = vacant.filter((h) => h.flat === undefined);
    let home = houses[0] ?? vacant[0];
    const main = town.open?.areas?.[0];
    const [cx, cz] = main ? [main.x + main.w / 2, main.z + main.d / 2] : [town.width / 2, town.depth / 2];
    const far = (h) => Math.hypot(h.lot.x - cx, h.lot.z - cz);
    if (homeKind === 'town') {
      const inMain = houses.filter((h) => (h.lot.area ?? 0) === 0).sort((a, b) => far(a) - far(b) || a.slot - b.slot);
      if (inMain.length) home = inMain[Math.floor(inMain.length / 2)];
    } else if (homeKind === 'flat') {
      const flats = vacant.filter((h) => h.flat !== undefined).sort((a, b) => Number((b.storey ?? 0) > 0) - Number((a.storey ?? 0) > 0) || far(a) - far(b) || a.key.localeCompare(b.key));
      if (!flats.length) stop('no flat for sale in this world');
      home = flats[0];
    }
    const ref = home.flat === undefined ? { slot: home.slot } : { slot: home.slot, flat: home.flat };
    request = { ...moveInRequest({ town, ...ref, household, from: null, emptyLot: false, mode: 'living', lifespan: 'normal' }), peopleSeed: seed };
    simSeed = seed;
    made = { recipe: 'scene', world, options, seed, home: homeKind, homeName: home.name };
  } else {
    // The five steps, as the screens run their jobs.
    const code = firstCode(flag('code', 'country-lrstm-7'), null, 0);
    const land = runTownJob(content, t, { kind: 'land', world: code.kind, seed: code.seed, options: optionsOf(code) });
    const laid = runTownJob(content, t, { kind: 'layout', land: land.draft, seed: code.townSeed ?? code.seed, options: optionsOf(code), waterKept: true });
    const layout = { proposal: laid.proposal, plan: laid.plan ?? laid.proposal.plan };
    const built = runTownJob(content, t, { kind: 'build', plan: layoutPlan(layout), proposal: layout.proposal, seed: 0 });
    const peopleSeed = peopleSeedOf(code);
    const streams = peopleStreams(peopleSeed);
    const town = populate(content, assets, t, built.draft, undefined, streams);
    // Step 4's first page of cards, in their order; Play on the \`--household\`-th whose home is of the kind asked for.
    const byHome = new Map(town.households.map((h) => [homeKey(h), h]));
    const cards = offerPage(town, streams, 0).flatMap((home) => byHome.get(homeKey(home)) ?? []);
    const fitting = cards.filter((h) => homeKind === 'any' || (homeKind === 'flat') === (h.flat !== undefined));
    if (!fitting.length) stop('no household offered lives in a ' + homeKind + ' (of ' + cards.length + ')');
    const chosen = fitting[Math.min(Number(flag('household', '0')), fitting.length - 1)];
    const ref = chosen.flat === undefined ? { slot: chosen.slot } : { slot: chosen.slot, flat: chosen.flat };
    request = moveInRequest({ town, ...ref, household: chosen.household, from: ref, emptyLot: false, mode: 'living', lifespan: 'normal', peopleSeed });
    simSeed = peopleStreams(peopleSeed).sim;
    made = { recipe: 'code', code: writeCode(code), home: homeKind, household: chosen.household.name, at: homeKey(chosen) };
  }
  if (flag('sim-seed', null) !== null) simSeed = Number(flag('sim-seed', null));
  // Move in's way into the game (\`session.ts\`): the homes for sale filled, the town file put together.
  const people = peopleStreams(request.peopleSeed);
  const town = townOnMoveIn(content, assets, t, request, Math.random, tier);
  const { lot } = runTownJob(content, t, { kind: 'assemble', town, player: request.household, slot: request.slot, options: { existing: request.existing, flat: request.flat, mode: request.mode, lifespan: request.lifespan }, random: people.bonds });
  return { lot, simSeed: simSeed >>> 0, made: { ...made, tier, households: town.households.length + 1 } };
}

/**
 * What the town file is made from: this tool, the page's code (`web/src`: the generator and what it reads) and its
 * content and models' manifest, with the flags that choose the game. Its town file is kept by it, so a state asked for
 * again starts without generating the town.
 */
function sourcesHash() {
  const h = createHash('sha256');
  const walk = (dir) => {
    for (const e of readdirSync(dir, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
      const path = dir + '/' + e.name;
      if (e.isDirectory()) walk(path);
      else if (/.(ts|js|json|svelte)$/.test(e.name)) h.update(path.slice(ROOT.length)).update(readFileSync(path));
    }
  };
  walk(WEB + 'src');
  walk(WEB + 'public/content');
  h.update(readFileSync(WEB + 'public/assets/manifest.json'));
  h.update(readFileSync(fileURLToPath(import.meta.url)));
  return h.digest('hex');
}

const dir = ROOT + 'bench-results/.states/';
const started = performance.now();
const from = flag('from', null);
let source;
if (from) source = { save: readSave(from) };
else {
  const asked = ['code', 'scene', 'world', 'options', 'seed', 'home', 'household', 'tier', 'sim-seed'].map((k) => [k, flag(k, has(k) || null)]);
  const townFile = dir + 'town-' + sha(JSON.stringify([asked, sourcesHash()])).slice(0, 16) + '.json';
  if (existsSync(townFile) && !has('fresh')) source = JSON.parse(readFileSync(townFile, 'utf8'));
  else {
    source = await newGame();
    mkdirSync(dir, { recursive: true });
    writeFileSync(townFile, JSON.stringify(source));
  }
}
const madeIn = performance.now() - started;

// When it is asked for: a tick, ticks on, or a moment of the game's clock (worked out once the game is open: a new
// game's tick 0 is its first morning, not midnight).
const TICKS_PER_MINUTE = 75;
const when = flag('tick', null) !== null ? { tick: Number(flag('tick', null)) } : flag('ticks', null) !== null ? { ticks: Number(flag('ticks', null)) } : flag('at', null) !== null ? { at: flag('at', '') } : { at: flag('day', '1') + ':' + flag('hour', '13') + ':-5' };
const watchSide = flag('watch', '320');

// (This tool's own way of running the sim is in the key too.)
const key = sha(JSON.stringify([from ? sha(source.save) : sha(source.lot), from ? 0 : source.simSeed, sha(wasmBytes), sha(readFileSync(fileURLToPath(import.meta.url))), watchSide, when])).slice(0, 16);
const file = dir + key + '.json';
let cached = existsSync(file) && !has('fresh');
if (!cached) {
  const wasm = await import(pkg + '/sim_wasm.js');
  const { memory } = wasm.initSync({ module: wasmBytes });
  const game = from ? wasm.Game.fromSave(content.json, source.save) : new wasm.Game(content.json, source.lot, source.simSeed);
  const snap = () => new Float32Array(memory.buffer, game.snapshot_ptr(), game.snapshot_len());
  // Held paused, as the worker starts a game; the tick it opens at.
  game.command(JSON.stringify({ type: 'setSpeed', speed: 0 }));
  const save0 = JSON.parse(game.save());
  let tick = save0.tick;
  // (The snapshot's header: the tick, the day from 1, the minute of the day.)
  const [, day0, minute0] = snap();
  let target;
  if ('tick' in when) target = when.tick;
  else if ('ticks' in when) target = tick + when.ticks;
  else {
    const [d, h, m] = when.at.split(':').map(Number);
    target = tick + Math.round(((d - day0) * 1440 + h * 60 + m - minute0) * TICKS_PER_MINUTE);
  }
  if (!Number.isInteger(target) || target < tick) stop('the moment asked for (' + JSON.stringify(when) + ', tick ' + target + ') is before the game starts (tick ' + tick + ', day ' + day0 + ' minute ' + minute0 + ')');
  // The view: the player's lot (\`session.ts\` \`showPlot\`); the watch round it.
  const player = save0.households.findIndex((h) => h.player);
  const plot = save0.plots?.[save0.households[player]?.plot] ?? null;
  if (plot) game.set_view(plot.x, plot.z, plot.x + plot.w, plot.z + plot.d);
  if (watchSide !== 'all' && plot) {
    const side = Number(watchSide);
    const [cx, cz] = [Math.trunc(plot.x + plot.w / 2), Math.trunc(plot.z + plot.d / 2)];
    const width = save0.lot.width;
    const depth = save0.lot.depth;
    const x0 = Math.min(Math.max(0, cx - Math.trunc(side / 2)), Math.max(0, width - side));
    const z0 = Math.min(Math.max(0, cz - Math.trunc(side / 2)), Math.max(0, depth - side));
    game.set_watch(x0, z0, Math.min(width, x0 + side), Math.min(depth, z0 + side));
  } else if (watchSide === 'all') game.clear_view();
  // Ten ticks a step while ten are left (speed 3), then one (speed 1): the same ticks, fewer calls.
  const t0 = performance.now();
  let speed = -1;
  while (tick < target) {
    const want = target - tick >= 10 ? 3 : 1;
    if (want !== speed) game.command(JSON.stringify({ type: 'setSpeed', speed: (speed = want) }));
    tick += game.start_step();
    game.end_step();
  }
  game.command(JSON.stringify({ type: 'setSpeed', speed: 0 }));
  const save = game.save();
  const at = JSON.parse(save).tick;
  if (at !== target) stop('ran to tick ' + at + ', not ' + target);
  const [, dayAt, minuteAt] = snap();
  game.free();
  mkdirSync(dir, { recursive: true });
  writeFileSync(file, save);
  writeFileSync(dir + key + '.meta.json', JSON.stringify({ key, tick: target, day: dayAt, minute: minuteAt, from, made: source.made ?? null, simSeed: source.simSeed ?? null, watch: watchSide, ticksRun: target - save0.tick, ms: Math.round(performance.now() - t0), makeMs: Math.round(madeIn), wasm: sha(wasmBytes).slice(0, 16) }, null, 2));
}
const save = readFileSync(file, 'utf8');
const state = JSON.parse(save);
const name = flag('name', null);
if (name) {
  if (!/^[\w.-]+$/.test(name)) stop('--name: letters, digits, dots and dashes only');
  mkdirSync(ROOT + 'debug-states', { recursive: true });
  writeFileSync(ROOT + 'debug-states/' + name + '.json', save);
  if (has('bench')) writeFileSync(WEB + 'public/bench/' + name + '.json.gz', gzipSync(save, { level: 9 }));
}
const meta = existsSync(dir + key + '.meta.json') ? JSON.parse(readFileSync(dir + key + '.meta.json', 'utf8')) : {};
if (meta.made) console.error('state: ' + JSON.stringify(meta.made));
console.log('state: ' + file + ' tick ' + state.tick + ' day ' + meta.day + ' minute ' + Math.round(meta.minute) + ' ' + (cached ? 'cached' : 'made') + ' ' + Math.round(performance.now() - started) + (name ? ' name ' + name : ''));
