/**
 * What `tools/bench/state.mjs` runs, bundled for Node (see there): the game's generator and the sim's wasm, under V8 as in
 * Chrome. Not run on its own.
 */

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { gunzipSync, gzipSync } from 'node:zlib';
import { Content } from '../../../web/src/content/content.ts';
import { AssetRegistry } from '../../../web/src/assets/registry.ts';
import { objectInfos } from '../../../web/src/game/housegen.ts';
import { useTownContent, generateWorld } from '../../../web/src/game/town.ts';
import { runTownJob } from '../../../web/src/game/townJobs.ts';
import { peopleStreams, populate } from '../../../web/src/game/people.ts';
import { firstCode, layoutPlan, moveInRequest, offerPage, optionsOf, peopleSeedOf, townOnMoveIn } from '../../../web/src/ui/newGame.ts';
import { homeKey } from '../../../web/src/ui/homes.ts';
import { readLayout } from '../../../web/src/core/snapshot.ts';
import { sceneHome, sceneTown } from '../../../web/src/debug/sceneRecipe.ts';
import { writeCode } from '../../../web/src/game/worldCode.ts';
import { worldFlags } from '../worldFlags.mjs';

// (Run as a bundle for Node, `state.mjs`: where the tree is comes from it.)
const ROOT = process.env.LL_ROOT;
if (!ROOT) throw new Error('state: run tools/bench/state.mjs, not this');
const WEB = ROOT + 'web/';
/** This tool's own files: in what keeps a state. */
const TOOL = [ROOT + 'tools/bench/state.mjs', ROOT + 'tools/bench/lib/stateMake.mjs'];
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

/** The game's content, loaded the first time it is asked for: a state already kept needs none. */
let loaded = null;
const contentNow = async () => (loaded ??= await Content.load('http://localhost/content/base.json'));

/** The town file and the sim's dice of a new game, made as the page makes them. */
async function newGame() {
  const content = await contentNow();
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
    const { world, options } = worldFlags(flag);
    const seed = flag('seed', null) === null ? null : Number(flag('seed', null));
    if (seed === null) stop('--scene needs --seed (the same town every time)');
    const empty = generateWorld(content, t, world, seed, undefined, undefined, options);
    const { town, household } = sceneTown(content, assets, t, empty, seed);
    let home;
    let ref;
    try {
      ({ home, ref } = sceneHome(content, t, town, homeKind));
    } catch (err) {
      stop(err.message);
    }
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
  for (const tool of TOOL) h.update(readFileSync(tool));
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
const when = flag('tick', null) !== null ? { tick: Number(flag('tick', null)) } : flag('ticks', null) !== null ? { ticks: Number(flag('ticks', null)) } : flag('at', null) !== null ? { at: flag('at', '') } : { at: flag('day', '1') + ':' + flag('hour', '13') + ':-5' };
const watchSide = flag('watch', '320');

// (This tool's own way of running the sim is in the key too.)
const key = sha(JSON.stringify([from ? sha(source.save) : sha(source.lot), from ? 0 : source.simSeed, sha(wasmBytes), sha(TOOL.map((tool) => readFileSync(tool, 'utf8')).join('')), watchSide, when])).slice(0, 16);
const file = dir + key + '.json';
const metaFile = dir + key + '.meta.json';
const cached = existsSync(file) && !has('fresh');
/** What the state is (its tick, day and minute, how it was made); its save as made, when made here. */
let meta;
let save = null;
if (!cached) {
  const content = await contentNow();
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
    target = tick + Math.round(((d - day0) * 1440 + h * 60 + m - minute0) * readLayout(game.snapshot_layout()).ticksPerMinute);
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
  save = game.save();
  const at = JSON.parse(save).tick;
  if (at !== target) stop('ran to tick ' + at + ', not ' + target);
  const [, dayAt, minuteAt] = snap();
  game.free();
  mkdirSync(dir, { recursive: true });
  writeFileSync(file, save);
  meta = { key, tick: target, day: dayAt, minute: minuteAt, from, made: source.made ?? null, simSeed: source.simSeed ?? null, watch: watchSide, ticksRun: target - save0.tick, ms: Math.round(performance.now() - t0), makeMs: Math.round(madeIn), wasm: sha(wasmBytes).slice(0, 16) };
  writeFileSync(metaFile, JSON.stringify(meta, null, 2));
} else meta = JSON.parse(readFileSync(metaFile, 'utf8'));
const name = flag('name', null);
if (name) {
  if (!/^[\w.-]+$/.test(name)) stop('--name: letters, digits, dots and dashes only');
  save ??= readFileSync(file, 'utf8');
  mkdirSync(ROOT + 'debug-states', { recursive: true });
  writeFileSync(ROOT + 'debug-states/' + name + '.json', save);
  if (has('bench')) writeFileSync(WEB + 'public/bench/' + name + '.json.gz', gzipSync(save, { level: 9 }));
}
if (meta.made) console.error('state: ' + JSON.stringify(meta.made));
console.log('state: ' + file + ' tick ' + meta.tick + ' day ' + meta.day + ' minute ' + Math.round(meta.minute) + ' ' + (cached ? 'cached' : 'made') + ' ' + Math.round(performance.now() - started) + (name ? ' name ' + name : ''));
