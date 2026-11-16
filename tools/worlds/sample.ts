/**
 * Makes sample worlds of every place (web/src/game/world.ts) with the shipped content, the way a
 * new game does (its land, step 1's `land` job: `macro.ts` `landDraft`; its town laid on that, step
 * 2's `town` job: `layOn` and `landFollow.ts` `settleLand`; its neighbours, a player household
 * moving into an empty house), and
 * writes their town files for the simulation to check
 * (`cargo test -p sim-core --test sim generated_worlds:: -- --ignored`, reading WORLDS). Checks each
 * world's layout with `web/src/game/townCheck.ts` `townProblems` (its roads, areas, water, bridges,
 * trails, names, lots, the living town's backdrop, centre and places, and how filled it is against
 * `world.ts` `expectations`), and no country road the layout found no way for. And the neighbours:
 * every household's savings cover its home's price (web/src/game/wealth.ts, price.ts); per kind of
 * house, how old its owners are and how high up they work. And the file under 4 MB, and the time a
 * big world takes (its land, its town's layout without generating houses, its land settled under
 * it, and assembling its file: a problem at twice the budget). Problems are listed (and the exit
 * code is 1). Every world is a Big one now (cosy and normal were dropped in 0.8.0).
 * Each world's line counts its terraced houses, backdrop buildings and places.
 *
 * A place's land (`meta.land`, its town graded into it: `landFollow.ts` `withLand`, timed against
 * `LAND_BUDGET`), checked on the surface the renderer works out from the town file
 * (`builtSurface.ts`): no road steeper than its kind's `GRADE` by more than 1 % (a trail, a footpath on
 * the land as it lies, listed only), no road (not a trail) or lot cut
 * into the land as it was seeded or standing over it by more than `CUT_MAX`, no lot's edge over
 * the land or the pad beside it by more than `RETAIN_MAX`, every bridge's deck `BRIDGE_CLEAR` over
 * its water. (A lot's edge over its road along its front is listed, not checked.)
 * A table of each world's steepest ways, tallest edges and deepest cuts follows the worlds.
 *
 * Every place at its own options (`perKind` worlds each), and one world each of `EXTREMES`, the
 * options at their furthest (`--no-extremes`: not those). `--options town=city,villages=many`:
 * every place with those options instead.
 *
 * The same arguments make the same files, byte for byte (`Math.random` seeded per world:
 * web/tests/sampleWorlds.slow.test.ts), so a world a check fails on can be made again and run alone.
 * `--seed-base n` (or WORLDS_SEED_BASE; default 0, the worlds CI checks) draws other worlds: the
 * release gate runs the checks over five (`tools/ci-local.sh`, SEED_BASE).
 *
 * Run: bun tools/worlds/sample.ts worlds.json [worlds per kind] [--kinds farm,forest] [--options town=city,...] [--no-extremes] [--seed-base n] [--no-timing]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';
import type { Templates } from '../../web/src/game/town.ts';

// The loaders fetch from the page's address: here, files under web/public.
const root = new URL('../../web/public/', import.meta.url);
Object.assign(globalThis, { location: { href: root.href, search: '' } });
const fetchFile = async (url: string | URL) => {
  const text = readFileSync(fileURLToPath(new URL(url, root)), 'utf8');
  return { ok: true, status: 200, text: async () => text, json: async () => JSON.parse(text) };
};
Object.assign(globalThis, { fetch: fetchFile });

const { Content } = await import('../../web/src/content/content.ts');
const { AssetRegistry } = await import('../../web/src/assets/registry.ts');
const { objectInfos } = await import('../../web/src/game/housegen.ts');
const { randomHousehold } = await import('../../web/src/game/household.ts');
const { assembleTown, generateStats, houseTemplate, populate, vacantSlots } = await import('../../web/src/game/town.ts');
const { landDraft, layOn } = await import('../../web/src/game/macro.ts');
const { expectations, WORLD_KINDS } = await import('../../web/src/game/world.ts');
type WorldOptions = import('../../web/src/game/world.ts').WorldOptions;
const { homePrice } = await import('../../web/src/game/price.ts');
const { savingsOf } = await import('../../web/src/game/wealth.ts');
const { mulberry32 } = await import('../../web/src/game/noise.ts');
const { fillOf, townProblems } = await import('../../web/src/game/townCheck.ts');
type TownFile = import('../../web/src/game/townCheck.ts').CheckedTown;
const { previewWorld } = await import('../../web/src/game/townPreview.ts');
const { measureLand, settleLand } = await import('../../web/src/game/landFollow.ts');
type Slopes = import('../../web/src/game/landFollow.ts').Slopes;
type FollowWorld = import('../../web/src/game/landFollow.ts').FollowWorld;
const { decodeLand } = await import('../../web/src/game/land.ts');
type Land = import('../../web/src/game/land.ts').Land;

/**
 * Budgets (ms): a place's land with no town on it (step 1's `land` job: planned, seeded and
 * previewed), a big world's layout on it (without generating its houses) and assembling its file.
 * (Not targets: a big world may take a while to build. They catch one that runs away.)
 */
const LAND_JOB_BUDGET = 400;
const LAYOUT_BUDGET = 1500;
const ASSEMBLE_BUDGET = 1500;
const MAX_FILE = 4e6;
/** A place's land seeded and its town graded into it, round after round where it gives up what breaks the slope limits (`settleLand`), ms: about twice the slowest world measured (2026-11-16, M3 Pro). */
const LAND_BUDGET = 1300;

/**
 * The land checks (see the top of this file, `landFollow.ts` `measureLand`) on the surface the
 * renderer works out from a town file (its `meta.land`), against `seeded`, the land before the town
 * was graded in.
 */
function slopeChecks(town: TownFile, seeded: Land): { problems: string[]; slopes: Slopes } {
  const { problems, slopes } = measureLand(town as unknown as FollowWorld, decodeLand(town.meta.land!), seeded);
  return { problems, slopes };
}

const problems: string[] = [];

const content = await Content.load(new URL('content/base.json', root).href);
const assets = await AssetRegistry.load(new URL('assets/manifest.json', root).href);
const read = (name: string) => JSON.parse(readFileSync(fileURLToPath(new URL(`content/${name}`, root)), 'utf8'));
const templates: Templates = { ...(read('houses.json') as Templates), gen: read('housegen.json') as HouseGenConfig, info: objectInfos(content), lampTiles: content.lampTiles };

const flag = (name: string) => {
  const i = process.argv.indexOf(name);
  return i < 0 ? undefined : process.argv.splice(i, 2)[1];
};
/** Which worlds of each kind (0: CI's); each world's seed moves on `SEED_STEP` per one. */
const seedBase = Number(flag('--seed-base') ?? process.env.WORLDS_SEED_BASE ?? 0);
if (!Number.isInteger(seedBase) || seedBase < 0) throw new Error(`no seed base ${seedBase}`);
const SEED_STEP = 7919;
/** `--no-timing`: the layout's and the file's time not checked against their budgets (a run beside others, on a slow runner: `web/tests/sampleWorlds.slow.test.ts`). */
const timing = !process.argv.includes('--no-timing');
if (!timing) process.argv.splice(process.argv.indexOf('--no-timing'), 1);
const kinds = (flag('--kinds')?.split(',') ?? [...WORLD_KINDS]) as (typeof WORLD_KINDS)[number][];
for (const kind of kinds) if (!WORLD_KINDS.includes(kind)) throw new Error(`no kind ${kind}`);
/** `--options town=city,villages=many`: every place with these options (else its own). */
const OPTION_VALUES: Record<keyof WorldOptions, readonly string[]> = { town: ['hamlet', 'village', 'town', 'city'], villages: ['none', 'few', 'many'], hills: ['flat', 'rolling', 'hilly', 'mountains'], woods: ['open', 'some', 'wooded'], size: ['small', 'medium', 'large'] };
const optionsOf = (text: string): Partial<WorldOptions> => {
  const out: Record<string, string> = {};
  for (const pair of text.split(',').filter(Boolean)) {
    const [key, value] = pair.split('=');
    if (!(key in OPTION_VALUES) || !OPTION_VALUES[key as keyof WorldOptions].includes(value)) throw new Error(`no option ${pair}`);
    out[key] = value;
  }
  return out as Partial<WorldOptions>;
};
const forced = ((text) => (text === undefined ? undefined : optionsOf(text)))(flag('--options'));
/**
 * The options at their furthest, one world of each (of the places sampled; not with `--options`
 * or `--no-extremes`): the biggest and smallest worlds, the woods and hills at their most and
 * least, each town shape at a size it has never had.
 */
const EXTREMES: [(typeof WORLD_KINDS)[number], string][] = [
  ['country', 'town=city,villages=many'],
  ['country', 'town=hamlet,villages=none,hills=flat,woods=open'],
  ['forest', 'town=hamlet,villages=none,woods=wooded'],
  ['mountain', 'town=city,villages=many,woods=wooded'],
  ['island', 'town=city,villages=many'],
  ['river', 'town=village,villages=none'],
  ['river', 'town=city,villages=many'],
  ['heath', 'town=hamlet,villages=few,hills=hilly,woods=open'],
  ['seaside', 'town=town,villages=many,hills=mountains'],
  ['farm', 'town=city,villages=few,hills=flat,woods=open'],
  ['lakeside', 'town=hamlet,villages=many,hills=hilly,woods=wooded'],
];
const extremes = !process.argv.includes('--no-extremes') && !forced;
if (process.argv.includes('--no-extremes')) process.argv.splice(process.argv.indexOf('--no-extremes'), 1);
const out = process.argv[2] ?? 'worlds.json';
const perKind = Number(process.argv[3] ?? 3);
// (A flag this script no longer takes: a call site that still passes one, rather than one silently
// ignored and the file it wanted never written.)
const leftover = process.argv.slice(2).filter((a) => a.startsWith('--'));
if (leftover.length) throw new Error(`no flag ${leftover.join(' ')}`);
const worlds: unknown[] = [];
/** The slowest world (ms): the layout (without houses), the houses, the file, the land (step 1's job). */
const slowest: [number, number, number, number] = [0, 0, 0, 0];
/** The house classes, smallest first (`housegen.json` `classes`, by the label houses carry as `class`). */
const HOUSE_CLASSES = Object.values(templates.gen!.classes).map((c) => c.label);
/** Per house class: homes lived in, and their owners' summed eldest grown-up's age, top job grade (-1 none), how many are settled (`SETTLED_AGE` or `SENIOR`), price, savings. */
const owned: Record<string, { homes: number; age: number; level: number; settled: number; price: number; savings: number }> = {};
const SETTLED_AGE = 35;
/** Grade F, Senior. */
const SENIOR = 5;
const forSale: Record<string, number> = {};
/** Worlds, courts in all and the most in one world. */
const courtsOf: [number, number, number] = [0, 0, 0];
/** Each world's land: its slopes (`slopeChecks`), how long it took to grade (ms), its size in the file (KB). */
const landTable: { tag: string; slopes: Slopes; ms: number; kb: number; given: { lots: number; backdrop: number; roads: number; courts: number } }[] = [];
/** The worlds to make: each place `perKind` times at its own options (or `--options`), then the extremes. */
const runs: { kind: (typeof WORLD_KINDS)[number]; seed: number; options?: Partial<WorldOptions> }[] = [
  ...WORLD_KINDS.filter((k) => kinds.includes(k)).flatMap((kind) => Array.from({ length: perKind }, (_, i) => ({ kind, seed: (i + 1) * 104729 + kind.length + seedBase * SEED_STEP, ...(forced ? { options: forced } : {}) }))),
  ...(extremes ? EXTREMES.flatMap(([kind, text], k) => (kinds.includes(kind) ? [{ kind, seed: (100 + k) * 104729 + kind.length + seedBase * SEED_STEP, options: optionsOf(text) }] : [])) : []),
];
for (const { kind, seed, options } of runs) {
  const tag = `${kind} ${seed}${options ? ` (${Object.values(options).join(', ')})` : ''}`;
  const expect = expectations(kind, options);
  // (The households, the player's and the neighbours', drawn with `Math.random`: seeded here
  // by the world's seed, the same seeds make the same files, so a failure in a
  // check of them can be run again.)
  Math.random = mulberry32(seed);
  // Its land (step 1's job, with its preview), then its town laid on it from the same seed (step 2's).
  let t0 = performance.now();
  const bare = landDraft(content, templates, kind, options, seed);
  previewWorld(content, templates, bare, null);
  const landJob = performance.now() - t0;
  generateStats.houseMs = 0;
  t0 = performance.now();
  const laid = layOn(content, templates, bare, {}, { waterKept: true });
  const houses = generateStats.houseMs;
  const layout = performance.now() - t0 - houses;
  // The land graded under the town, as the town job does it (from the preview of the world, giving up what still breaks the slope limits).
  let seeded: Land | null = null;
  t0 = performance.now();
  const world = settleLand(laid, (d) => previewWorld(content, templates, d, null), (l) => (seeded = l)).draft;
  const landMs = performance.now() - t0;
  // (Among mountains, the fill a hamlet left out for want of a site a road could reach, and a lot given up to the slope limits, would have brought.)
  const left = world.diagnostics?.villages;
  const given = world.diagnostics?.slopes;
  if (left || given) Object.assign(expect, { quarters: Math.max(0, expect.quarters - (left?.left ?? 0)), homes: expect.homes - (left?.homes ?? 0) - (given?.lots ?? 0) });
  const town = populate(content, assets, templates, world);
  const home = vacantSlots(town)[0];
  if (!home) throw new Error(`${tag}: no empty house`);
  const player = randomHousehold(content, assets, 2);
  t0 = performance.now();
  const text = assembleTown(content, templates, town, player, home.index);
  const assemble = performance.now() - t0;
  const file = JSON.parse(text);
  [slowest[0], slowest[1], slowest[2], slowest[3]] = [Math.max(slowest[0], layout), Math.max(slowest[1], houses), Math.max(slowest[2], assemble), Math.max(slowest[3], landJob)];
  const found = townProblems(file, expect, { river: kind === 'river', careers: content.careerCategories.map((c) => c.id), given: world.diagnostics?.slopes?.backdropIn });
  for (const p of [...found.hard, ...found.soft, ...(world.diagnostics?.roads ?? [])]) problems.push(`${tag}: ${p}`);
  // (A terrace lot's house is a generated terraced house, its lot cut to it: a fixed layout is wider than the lot.)
  for (const s of town.slots) if (s.lot === 'terrace' && houseTemplate(templates, s.template)?.class !== 'terraced house') problems.push(`${tag}: terrace lot ${s.index} has a ${s.template}`);
  if (text.length > MAX_FILE) problems.push(`${tag}: the town file is ${(text.length / 1e6).toFixed(1)} MB`);
  if (timing && landJob > LAND_JOB_BUDGET * 2) problems.push(`${tag}: its land made in ${landJob.toFixed(0)} ms (budget ${LAND_JOB_BUDGET})`);
  if (timing && layout > LAYOUT_BUDGET * 2) problems.push(`${tag}: laid out in ${layout.toFixed(0)} ms (budget ${LAYOUT_BUDGET})`);
  if (timing && assemble > ASSEMBLE_BUDGET * 2) problems.push(`${tag}: assembled in ${assemble.toFixed(0)} ms (budget ${ASSEMBLE_BUDGET})`);
  if (timing && seeded && landMs > LAND_BUDGET * 2) problems.push(`${tag}: its land graded in ${landMs.toFixed(0)} ms (budget ${LAND_BUDGET})`);
  if (!file.meta.land) problems.push(`${tag}: it carries no land`);
  if (seeded && file.meta.land) {
    const { problems: found, slopes } = slopeChecks(file, seeded);
    for (const p of found) problems.push(`${tag}: ${p}`);
    landTable.push({ tag, slopes, ms: landMs, kb: JSON.stringify(file.meta.land).length / 1000, given: given ?? { lots: 0, backdrop: 0, roads: 0, courts: 0 } });
  }
  for (const { household, slot } of town.households) {
    const s = town.slots[slot];
    const price = homePrice(content, templates, s);
    const savings = savingsOf(content, household);
    if (savings < price) problems.push(`${tag}: the ${household.name}s saved $${savings} for ${s.name}, which costs $${price}`);
    const house = houseTemplate(templates, s.template);
    const cls = house?.class ?? house?.name ?? s.template;
    const adults = household.members.filter((m) => content.isAdult(m.age));
    const owners = (owned[cls] ??= { homes: 0, age: 0, level: 0, settled: 0, price: 0, savings: 0 });
    const eldest = Math.max(0, ...adults.map((m) => m.age));
    const level = Math.max(-1, ...adults.map((m) => m.job?.level ?? -1));
    owners.homes++;
    owners.age += eldest;
    owners.level += level;
    if (eldest >= SETTLED_AGE || level >= SENIOR) owners.settled++;
    owners.price += price;
    owners.savings += savings;
  }
  for (const s of vacantSlots(town)) {
    const house = houseTemplate(templates, s.template);
    const cls = house?.class ?? house?.name ?? s.template;
    forSale[cls] = (forSale[cls] ?? 0) + 1;
  }
  worlds.push({ kind, seed, ...(options ? { options } : {}), name: town.name, town: file });
  const farms = town.slots.filter((s) => s.lot === 'farm').length;
  const basements = town.slots.filter((s) => s.kind === 'house' && houseTemplate(templates, s.template)?.basement).length;
  const spots = town.slots.filter((s) => s.kind === 'outdoor').length;
  const dropped = world.diagnostics?.dropped.length ?? 0;
  const courts = town.roads.filter((r) => r.kind !== 'trail').reduce((n, r) => n + r.ends.filter((e) => e === 'dead').length, 0);
  [courtsOf[0], courtsOf[1], courtsOf[2]] = [courtsOf[0] + 1, courtsOf[1] + courts, Math.max(courtsOf[2], courts)];
  console.log(
    `${tag}: ${town.name} (${town.world?.tagline}) ${town.width}×${town.depth}, ${town.roads.length} road(s) (${courts} courts), ` +
      `${town.open ? `${town.open.areas.length} areas, ${spots} outdoor places, ${((f) => `${(f.built * 100).toFixed(0)}% built and farmed, ${(f.empty * 100).toFixed(0)}% squares empty, `)(fillOf(file, expect.meadow))}` : ''}` +
      `${town.slots.filter((s) => s.kind === 'house').length} houses${farms ? ` (${farms} farms)` : ''} (${town.slots.filter((s) => s.lot === 'terrace').length} terraced)${dropped ? `, ${dropped} lots left out` : ''}, ${basements} with a basement, ${town.households.length} households, ` +
      `${town.open?.backdrop ? `${town.open.backdrop.length} backdrop buildings (${town.open.backdrop.filter((b) => b.area === 0).length} in the main town), ${town.open.places?.length ?? 0} places, ` : ''}` +
      `${file.sims.length} residents, ${file.objects.length} things, ${(text.length / 1e6).toFixed(2)} MB; land ${landJob.toFixed(0)} ms, town ${(layout + houses + landMs).toFixed(0)} ms (${layout.toFixed(0)} layout + ${houses.toFixed(0)} houses + ${landMs.toFixed(0)} settled), file ${assemble.toFixed(0)} ms`,
  );
}
console.log(`Owners per kind of house (the eldest grown-up, the highest job grade, A = 0; settled: ${SETTLED_AGE}+ or grade F up):`);
for (const cls of [...HOUSE_CLASSES, ...Object.keys(owned).filter((c) => !HOUSE_CLASSES.includes(c))]) {
  const o = owned[cls];
  if (!o && !forSale[cls]) continue;
  const n = o?.homes ?? 0;
  const mean = (v: number) => (n ? (v / n).toFixed(1) : '-');
  const dollars = (v: number) => (n ? `$${Math.round(v / n / 1000)}k` : '-');
  console.log(
    `  ${cls.padEnd(13)} ${String(n).padStart(3)} lived in, ${String(forSale[cls] ?? 0).padStart(3)} for sale: age ${mean(o?.age ?? 0)}, grade ${mean(o?.level ?? 0)}, ${n ? Math.round((100 * o!.settled) / n) : '-'}% settled, price ${dollars(o?.price ?? 0)}, saved ${dollars(o?.savings ?? 0)}`,
  );
}
if (landTable.length) {
  const pc = (v: number | undefined) => (v === undefined ? '    -' : `${(v * 100).toFixed(1).padStart(5)}`);
  const m = (v: number) => v.toFixed(2).padStart(5);
  console.log('Land (grades %: street, lane, country, trail (a measure only); lot edges m: over the land, another pad, a road; cuts m: roads, lots; bridge clearance m; drift m; given up: lots, backdrop, roads out, courts cut back; ms; KB):');
  for (const { tag, slopes: s, ms, kb, given: g } of landTable) {
    console.log(
      `  ${tag.padEnd(58)} ${pc(s.grade.street)} ${pc(s.grade.lane)} ${pc(s.grade.country)} ${pc(s.grade.trail)} | ${m(s.retain.land)} ${m(s.retain.lot)} ${m(s.retain.road)} | ${m(s.cut.road)} ${m(s.cut.lot)} | ${Number.isFinite(s.clear) ? m(s.clear) : '    -'} | ${m(s.drift)} | ${[g.lots, g.backdrop, g.roads, g.courts].map((v) => String(v).padStart(2)).join(' ')} | ${ms.toFixed(0).padStart(4)} ${kb.toFixed(0).padStart(4)}`,
    );
  }
  console.log(`Slowest land: ${Math.max(...landTable.map((l) => l.ms)).toFixed(0)} ms (budget ${LAND_BUDGET})`);
}
const [n, all, most] = courtsOf;
console.log(`Courts: ${all} in ${n} worlds (${(all / n).toFixed(1)} a world, ${most} at most)`);
console.log(`Slowest: land ${slowest[3].toFixed(0)} ms, layout ${slowest[0].toFixed(0)} ms, houses ${slowest[1].toFixed(0)} ms, file ${slowest[2].toFixed(0)} ms`);
writeFileSync(out, JSON.stringify(worlds));
console.log(`${worlds.length} worlds → ${out}`);
if (problems.length) {
  console.log(`${problems.length} problems:`);
  for (const p of problems.slice(0, 60)) console.log(`  ${p}`);
  process.exitCode = 1;
}
