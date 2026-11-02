/**
 * Makes sample worlds of every kind (web/src/game/world.ts) with the shipped content, the way a
 * new game does (a world, its neighbours, a player household moving into an empty house), and
 * writes their town files for the simulation to check
 * (`cargo test -p sim-core --test generated_worlds -- --ignored`, reading WORLDS). Checks the
 * layout first: lots inside the town, apart from each other and off the roads, each by its road (a
 * park may stand on a green),
 * the ways out of town on its edge. And the neighbours: every household's savings cover its
 * home's price (web/src/game/wealth.ts, price.ts); per kind of house, how old its owners are and
 * how high up they work. Problems are listed (and the exit code is 1).
 *
 * Sizes (docs/design/open-world.md): `--sizes cosy,normal,big` (default cosy). Big worlds don't load
 * in the simulation until the open-world sim branches are in, so they're written to a file of
 * their own (`--big-out`, default worlds-big.json beside the output), `--big-per-kind` of each
 * (default 1). Besides the checks above, for every size: the roads (country roads and trails too)
 * one network as the sim joins them (an end within half the wider road's width and a metre of the
 * other's middle), every way out on it, no more courts (dead ends) than an area keeps, none
 * reaching more than 45 m past the last road it meets, and no country road the layout found no
 * way for. And for big worlds: every area reaching a way out, nothing on the
 * water, every outdoor place at a trail's end and near its area, no trail through a lot, areas
 * apart, hamlets of four lots or more, every house near a stop or a way out, residents within the
 * cap, the world filled (a quarter of it or more built and farmed, at most a tenth of its 100 m
 * squares with nothing in them, every quarter of it with an area or homes out in the country),
 * every road named (no two alike), the file under 4 MB, and the time a big world takes (its
 * layout without generating houses, and assembling its file: a problem at twice the budget).
 *
 * Run: bun tools/worlds/sample.ts worlds.json [worlds per kind] [--sizes cosy,normal,big] [--big-out file] [--big-per-kind n] [--kinds farm,forest]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';
import type { Facing, Templates } from '../../web/src/game/town.ts';

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
const { assembleTown, BIG_COURTS, COURTS, generateStats, generateWorld, houseTemplate, neighbourCap, populate, turner, vacantSlots } = await import('../../web/src/game/town.ts');
const { WORLD_KINDS, WORLD_SIZES } = await import('../../web/src/game/world.ts');
const { unrle } = await import('../../web/src/game/macro.ts');
const { homePrice } = await import('../../web/src/game/price.ts');
const { savingsOf } = await import('../../web/src/game/wealth.ts');
const { COURT, lengths, nearest, roadMask } = await import('../../web/src/game/roads.ts');

type Rect = { x: number; z: number; w: number; d: number };
type Pt = [number, number];
type Road = { points: Pt[]; width: number; ends: [string, string]; name?: string; kind?: string; area?: number };
type Exit = Pt | { x: number; z: number; kind: 'road' | 'stop' };
type TownFile = {
  size?: 'cosy' | 'normal' | 'big';
  width: number;
  depth: number;
  plots: (Rect & { public: boolean; area?: number; kind?: string })[];
  exits: Exit[];
  sims: { household: number }[];
  households: { player: boolean }[];
  roads?: Road[];
  areas?: { name: string; x: number; z: number; w: number; d: number }[];
  terrain?: { water?: number[] };
  meta: { roads?: Road[]; facing?: number[]; zones?: { kind: string; poly: Pt[] }[] };
};
const exitAt = (e: Exit): Pt => (Array.isArray(e) ? e : [e.x, e.z]);
const isRoadExit = (e: Exit) => Array.isArray(e) || e.kind === 'road';
/** How far a point is from a rectangle (0 inside it). */
const offRect = (r: Rect, x: number, z: number) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
/**
 * Budgets (ms): a big world's layout (without generating its houses) and assembling its file. (Not
 * targets: a big world may take a while to build. They catch one that runs away.)
 */
const LAYOUT_BUDGET = 1500;
const ASSEMBLE_BUDGET = 1500;
const MAX_FILE = 4e6;

/** A big world filled at least this much with what is built and farmed (its areas' boxes, lots, fields and orchards). */
const BUILT_SHARE = 0.25;
/** At most this share of a big world's 100 m squares hold nothing (no lot, road, woods, field, orchard or water). */
const EMPTY_SHARE = 0.1;
/** Areas' middles at least this far apart (m). */
const AREAS_APART = 170;
/** A square holds something if this much of it does (m²). */
const HOLDS = 200;

/** How filled a big world is: the share of it built and farmed, the share of its 100 m squares with nothing in them. */
function fillOf(town: TownFile): { built: number; empty: number } {
  const { width: W, depth: D } = town;
  const C = 2;
  const [gw, gd] = [Math.ceil(W / C), Math.ceil(D / C)];
  const built = new Uint8Array(gw * gd);
  const some = new Uint8Array(gw * gd);
  const box = (mask: Uint8Array, r: Rect) => {
    for (let z = Math.max(0, Math.floor(r.z / C)); z < Math.min(gd, Math.ceil((r.z + r.d) / C)); z++) for (let x = Math.max(0, Math.floor(r.x / C)); x < Math.min(gw, Math.ceil((r.x + r.w) / C)); x++) mask[z * gw + x] = 1;
  };
  const poly = (mask: Uint8Array, ring: Pt[]) => {
    for (let z = 0; z < gd; z++) {
      const zc = (z + 0.5) * C;
      const xs: number[] = [];
      for (let i = 0, j = ring.length - 1; i < ring.length; j = i++) {
        const [[xi, zi], [xj, zj]] = [ring[i], ring[j]];
        if (zi > zc !== zj > zc) xs.push(((xj - xi) * (zc - zi)) / (zj - zi) + xi);
      }
      xs.sort((a, b) => a - b);
      for (let k = 0; k + 1 < xs.length; k += 2) for (let x = Math.max(0, Math.ceil(xs[k] / C - 0.5)); x < Math.min(gw, Math.ceil(xs[k + 1] / C - 0.5)); x++) mask[z * gw + x] = 1;
    }
  };
  for (const a of town.areas ?? []) box(built, a);
  for (const p of town.plots) {
    box(built, p);
    box(some, p);
  }
  for (const z of town.meta.zones ?? []) {
    if (z.kind === 'fields' || z.kind === 'orchard') {
      poly(built, z.poly);
      poly(some, z.poly);
    } else if (z.kind === 'woods' || z.kind === 'water') poly(some, z.poly);
  }
  const roads = roadMask(town.roads ?? town.meta.roads ?? [], W, D, 0);
  for (let z = 0; z < D; z += C) for (let x = 0; x < W; x += C) if (roads[z * W + x]) some[(z / C) * gw + x / C] = 1;
  let filled = 0;
  for (const v of built) filled += v;
  let [squares, empty] = [0, 0];
  for (let sz = 0; sz < D; sz += 100) {
    for (let sx = 0; sx < W; sx += 100) {
      let n = 0;
      for (let z = sz / C; z < Math.min(gd, (sz + 100) / C); z++) for (let x = sx / C; x < Math.min(gw, (sx + 100) / C); x++) n += some[z * gw + x];
      const cells = (Math.min(gd, (sz + 100) / C) - sz / C) * (Math.min(gw, (sx + 100) / C) - sx / C);
      // (A sliver of a square along the world's edge counts as its share of one.)
      squares += (cells * C * C) / 1e4;
      if (n * C * C < Math.min(HOLDS, (cells * C * C) / 50)) empty += (cells * C * C) / 1e4;
    }
  }
  return { built: filled / (gw * gd), empty: empty / squares };
}

/** What's wrong with how a town's roads join up, its areas, water, trails and names (the open-world checks). */
function openProblems(town: TownFile): string[] {
  const out: string[] = [];
  const roads = town.roads ?? town.meta.roads ?? [];
  // Joined as the sim joins them (travel.rs `Graph::build`): an end within half the wider road's width and a metre of the other's middle.
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  /** Per road, where along it the other roads' ends meet it (m). */
  const joins: number[][] = roads.map(() => []);
  roads.forEach((r, i) => {
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      roads.forEach((o, j) => {
        const near = j === i ? null : nearest(o.points, end[0], end[1]);
        if (!near || near.dist > Math.max(o.width, r.width) / 2 + 1) return;
        parent[find(i)] = find(j);
        joins[j].push(near.s);
      });
    }
  });
  const leaves = new Set(roads.flatMap((r, i) => (r.ends.includes('out') ? [find(i)] : [])));
  roads.forEach((r, i) => {
    if (!leaves.has(find(i))) out.push(`${r.kind ?? 'street'} ${r.name ?? i} doesn't join a road out of town`);
  });
  // One network: every road (country roads and trails too) and every way out on it.
  const networks = new Set(roads.map((_, i) => find(i)));
  if (networks.size > 1) out.push(`the roads are ${networks.size} networks, not one`);
  for (const e of town.exits) {
    const [x, z] = exitAt(e);
    if (roads.length && !roads.some((r) => nearest(r.points, x, z).dist <= r.width / 2 + 1.5)) out.push(`the way out at ${x}, ${z} is off the roads`);
  }
  const areas = town.areas ?? [{ name: 'the town', x: 0, z: 0, w: town.width, d: town.depth }];
  // Courts (dead ends but the trails'): a few per area, none reaching more than `COURT` m past the last road it meets.
  const courts = areas.map(() => 0);
  roads.forEach((r, i) => {
    if (r.kind === 'trail') return;
    const L = lengths(r.points).at(-1)!;
    for (const e of [0, 1] as const) {
      if (r.ends[e] !== 'dead') continue;
      courts[r.area ?? 0]++;
      const met = joins[i].map((s) => (e === 1 ? s : L - s));
      const stub = L - Math.max(0, ...met);
      if (stub > COURT + 0.5) out.push(`${r.name ?? `road ${i}`} ends ${stub.toFixed(0)} m past the last road it meets (a court reaches ${COURT} m)`);
    }
  });
  courts.forEach((n, k) => {
    const most = town.size === 'big' && k === 0 ? BIG_COURTS : COURTS;
    if (n > most) out.push(`${areas[k].name} has ${n} courts (at most ${most})`);
  });
  areas.forEach((a, k) => {
    if (!roads.some((r, i) => (r.area ?? 0) === k && leaves.has(find(i)))) out.push(`${a.name} has no way out`);
  });
  // Names: every road one, none alike.
  const names = roads.map((r) => r.name ?? '');
  roads.forEach((r, i) => {
    if (!r.name) out.push(`road ${i} (${r.kind ?? 'street'}) has no name`);
    else if (names.indexOf(r.name) !== i) out.push(`two roads called ${r.name}`);
  });
  // Nothing on the water.
  const water = town.terrain?.water ? unrle(town.terrain.water, town.width * town.depth) : null;
  const wet = (x: number, z: number) => !!water && x >= 0 && z >= 0 && x < town.width && z < town.depth && water[Math.floor(z) * town.width + Math.floor(x)] === 1;
  if (water) {
    town.plots.forEach((p, i) => {
      let n = 0;
      for (let z = p.z; z < p.z + p.d; z++) for (let x = p.x; x < p.x + p.w; x++) n += wet(x + 0.5, z + 0.5) ? 1 : 0;
      if (n) out.push(`plot ${i} has ${n} tiles of water`);
    });
    roads.forEach((r, i) => {
      if (r.points.some(([x, z]) => wet(x, z))) out.push(`road ${r.name ?? i} runs over water`);
    });
  }
  // Trails: each outdoor place at the end of one, near its area; no trail through a lot.
  const ends = roads.flatMap((r) => [r.points[0], r.points[r.points.length - 1]]);
  town.plots.forEach((p, i) => {
    if (p.kind !== 'countryside') return;
    const gap = Math.min(...ends.map(([x, z]) => offRect(p, x, z)));
    if (gap > 6) out.push(`outdoor place ${i} is ${gap.toFixed(1)} m from a trail's end`);
    const far = offRect(areas[p.area ?? 0], p.x + p.w / 2, p.z + p.d / 2);
    if (far > 150) out.push(`outdoor place ${i} is ${far.toFixed(0)} m from its area`);
  });
  for (const trail of roads.filter((r) => r.kind === 'trail')) {
    const [ex, ez] = trail.points[trail.points.length - 1];
    for (const [i, p] of town.plots.entries()) {
      if (offRect(p, ex, ez) < 1) continue;
      if (trail.points.some(([x, z]) => x > p.x + 0.05 && x < p.x + p.w - 0.05 && z > p.z + 0.05 && z < p.z + p.d - 0.05)) out.push(`trail ${trail.name} runs through plot ${i}`);
    }
  }
  if (town.size === 'big') {
    const centres = areas.map((a): Pt => [a.x + a.w / 2, a.z + a.d / 2]);
    centres.forEach((c, i) =>
      centres.forEach((o, j) => {
        const d = Math.hypot(c[0] - o[0], c[1] - o[1]);
        if (j > i && d < AREAS_APART) out.push(`${areas[i].name} and ${areas[j].name} ${d.toFixed(0)} m apart`);
      }),
    );
    const fill = fillOf(town);
    if (fill.built < BUILT_SHARE) out.push(`${(fill.built * 100).toFixed(0)}% of the world built and farmed (${BUILT_SHARE * 100}% or more wanted)`);
    if (fill.empty > EMPTY_SHARE) out.push(`${(fill.empty * 100).toFixed(0)}% of its 100 m squares hold nothing (${EMPTY_SHARE * 100}% at most)`);
    // Spread over the whole world: each quarter of it an area's middle, or a farmstead or cottage out in the country.
    const inArea = (p: Rect) => areas.some((a) => offRect(a, p.x + p.w / 2, p.z + p.d / 2) === 0);
    const country = town.plots.filter((p) => (p.kind === 'home' || p.kind === 'farm') && !inArea(p)).map((p): Pt => [p.x + p.w / 2, p.z + p.d / 2]);
    const quarter = ([x, z]: Pt) => (x < town.width / 2 ? 0 : 1) + (z < town.depth / 2 ? 0 : 2);
    const held = new Set([...centres, ...country].map(quarter));
    for (const q of [0, 1, 2, 3]) if (!held.has(q)) out.push(`the ${['north-west', 'north-east', 'south-west', 'south-east'][q]} quarter has no area, farmstead or cottage`);
    areas.forEach((a, k) => {
      const lots = town.plots.filter((p) => (p.area ?? 0) === k && p.kind !== 'countryside').length;
      if (k > 0 && lots < 4) out.push(`hamlet ${a.name} has ${lots} lots`);
    });
    const homes = town.plots.filter((p) => p.kind === 'home' || p.kind === 'farm');
    if (homes.length < 70) out.push(`${homes.length} homes (a big world has 70 or more)`);
    // Every house within ~120 m of a stop or a way out.
    for (const [i, p] of town.plots.entries()) {
      if (p.kind !== 'home' && p.kind !== 'farm') continue;
      const far = Math.min(...town.exits.map((e) => Math.hypot(exitAt(e)[0] - (p.x + p.w / 2), exitAt(e)[1] - (p.z + p.d / 2))));
      if (far > 125) out.push(`house ${i} is ${far.toFixed(0)} m from a stop`);
    }
  }
  // Neighbours within the cap (the player's household aside).
  const player = town.households.findIndex((h) => h.player);
  const neighbours = town.sims.filter((s) => s.household !== player).length;
  if (neighbours > neighbourCap(town.size)) out.push(`${neighbours} neighbours (the cap is ${neighbourCap(town.size)})`);
  return out;
}

/** What's wrong with a town's layout. */
function layoutProblems(town: TownFile): string[] {
  const out: string[] = openProblems(town);
  // (Trails are on foot, between the lots: `openProblems` checks they keep off them.)
  const roads = (town.meta.roads ?? []).filter((r) => r.kind !== 'trail');
  const mask = roadMask(roads, town.width, town.depth, 0);
  town.plots.forEach((p, i) => {
    if (p.x < 0 || p.z < 0 || p.x + p.w > town.width || p.z + p.d > town.depth) out.push(`plot ${i} off the town`);
    for (const [j, q] of town.plots.entries()) if (j > i && p.x < q.x + q.w && q.x < p.x + p.w && p.z < q.z + q.d && q.z < p.z + p.d) out.push(`plots ${i} and ${j} overlap`);
    let onRoad = false;
    for (let z = p.z; z < p.z + p.d; z++) for (let x = p.x; x < p.x + p.w; x++) onRoad ||= mask[z * town.width + x] === 1 && roads.some((r) => nearest(r.points, x + 0.5, z + 0.5).dist < r.width / 2);
    if (onRoad) out.push(`plot ${i} on a road`);
    // The middle of its front, by a road.
    const f = (town.meta.facing?.[i] ?? 0) as Facing;
    // The lot's width along its street and its depth (turned a quarter, it lies across).
    const [lw, ld] = f % 2 ? [p.d, p.w] : [p.w, p.d];
    const [fx, fz] = turner(p.x, p.z, lw, ld, f).point([lw / 2, 0]);
    const gap = Math.min(...roads.map((r) => nearest(r.points, fx, fz).dist - r.width / 2));
    // (A park may stand in the middle of a green.)
    if (roads.length && gap > 13 && !p.public) out.push(`plot ${i} ${gap.toFixed(1)} m from its road`);
  });
  for (const [x, z] of town.exits.filter(isRoadExit).map(exitAt)) if (Math.min(x, z, town.width - x, town.depth - z) > 1) out.push(`a way out of town at ${x}, ${z}, not on its edge`);
  if (!town.exits.some(isRoadExit)) out.push('no way out of town');
  return out;
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
const sizes = (flag('--sizes') ?? 'cosy').split(',') as (typeof WORLD_SIZES)[number][];
for (const size of sizes) if (!WORLD_SIZES.includes(size)) throw new Error(`no size ${size}`);
const bigOutFlag = flag('--big-out');
const bigPerKind = Number(flag('--big-per-kind') ?? 1);
const kinds = flag('--kinds')?.split(',') ?? WORLD_KINDS;
const out = process.argv[2] ?? 'worlds.json';
const bigOut = bigOutFlag ?? out.replace(/(\.json)?$/, '-big.json');
const perKind = Number(process.argv[3] ?? 3);
const worlds: unknown[] = [];
const bigWorlds: unknown[] = [];
/** The slowest of each size (ms): the layout (without houses), the houses, the file. */
const slowest: Record<string, [number, number, number]> = {};
/** The house classes, smallest first (`housegen.json` `classes`, by the label houses carry as `class`). */
const HOUSE_CLASSES = Object.values(templates.gen!.classes).map((c) => c.label);
/** Per house class: homes lived in, and their owners' summed eldest grown-up's age, top job grade (-1 none), how many are settled (`SETTLED_AGE` or `SENIOR`), price, savings. */
const owned: Record<string, { homes: number; age: number; level: number; settled: number; price: number; savings: number }> = {};
const SETTLED_AGE = 35;
/** Grade F, Senior. */
const SENIOR = 5;
const forSale: Record<string, number> = {};
/** Per size: worlds, courts in all and the most in one world. */
const courtsOf: Record<string, [number, number, number]> = {};
for (const size of sizes) {
  for (const kind of WORLD_KINDS.filter((k) => kinds.includes(k))) {
    for (let i = 1; i <= (size === 'big' ? bigPerKind : perKind); i++) {
      const seed = i * 104729 + kind.length;
      const tag = `${kind}${size === 'cosy' ? '' : ` ${size}`} ${seed}`;
      generateStats.houseMs = 0;
      let t0 = performance.now();
      const world = generateWorld(content, templates, kind, seed, undefined, size);
      const houses = generateStats.houseMs;
      const layout = performance.now() - t0 - houses;
      const town = populate(content, assets, templates, world);
      const home = vacantSlots(town)[0];
      if (!home) throw new Error(`${tag}: no empty house`);
      const player = randomHousehold(content, assets, 2);
      t0 = performance.now();
      const text = assembleTown(content, templates, town, player, home.index);
      const assemble = performance.now() - t0;
      const file = JSON.parse(text);
      const worst = (slowest[size] ??= [0, 0, 0]);
      [worst[0], worst[1], worst[2]] = [Math.max(worst[0], layout), Math.max(worst[1], houses), Math.max(worst[2], assemble)];
      for (const p of [...layoutProblems(file), ...(world.diagnostics?.roads ?? [])]) problems.push(`${tag}: ${p}`);
      if (text.length > MAX_FILE) problems.push(`${tag}: the town file is ${(text.length / 1e6).toFixed(1)} MB`);
      if (size === 'big' && layout > LAYOUT_BUDGET * 2) problems.push(`${tag}: laid out in ${layout.toFixed(0)} ms (budget ${LAYOUT_BUDGET})`);
      if (size === 'big' && assemble > ASSEMBLE_BUDGET * 2) problems.push(`${tag}: assembled in ${assemble.toFixed(0)} ms (budget ${ASSEMBLE_BUDGET})`);
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
      (size === 'big' ? bigWorlds : worlds).push({ kind, size, seed, name: town.name, town: file });
      const farms = town.slots.filter((s) => s.lot === 'farm').length;
      const basements = town.slots.filter((s) => s.kind === 'house' && houseTemplate(templates, s.template)?.basement).length;
      const spots = town.slots.filter((s) => s.kind === 'outdoor').length;
      const dropped = world.diagnostics?.dropped.length ?? 0;
      const courts = town.roads.filter((r) => r.kind !== 'trail').reduce((n, r) => n + r.ends.filter((e) => e === 'dead').length, 0);
      const c = (courtsOf[size] ??= [0, 0, 0]);
      [c[0], c[1], c[2]] = [c[0] + 1, c[1] + courts, Math.max(c[2], courts)];
      console.log(
        `${tag}: ${town.name} (${town.world?.tagline}) ${town.width}×${town.depth}, ${town.roads.length} road(s) (${courts} courts), ` +
          `${town.open ? `${town.open.areas.length} areas, ${spots} outdoor places, ${((f) => `${(f.built * 100).toFixed(0)}% built and farmed, ${(f.empty * 100).toFixed(0)}% squares empty, `)(fillOf(file))}` : ''}` +
          `${town.slots.filter((s) => s.kind === 'house').length} houses${farms ? ` (${farms} farms)` : ''}${dropped ? `, ${dropped} lots left out` : ''}, ${basements} with a basement, ${town.households.length} households, ` +
          `${file.sims.length} residents, ${file.objects.length} things, ${(text.length / 1e6).toFixed(2)} MB; ${layout.toFixed(0)} ms + ${houses.toFixed(0)} ms houses, file ${assemble.toFixed(0)} ms`,
      );
    }
  }
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
for (const [size, [n, all, most]] of Object.entries(courtsOf)) console.log(`Courts ${size}: ${all} in ${n} worlds (${(all / n).toFixed(1)} a world, ${most} at most)`);
for (const [size, [layout, houses, assemble]] of Object.entries(slowest)) console.log(`Slowest ${size}: layout ${layout.toFixed(0)} ms, houses ${houses.toFixed(0)} ms, file ${assemble.toFixed(0)} ms`);
writeFileSync(out, JSON.stringify(worlds));
console.log(`${worlds.length} worlds → ${out}`);
if (bigWorlds.length) {
  writeFileSync(bigOut, JSON.stringify(bigWorlds));
  console.log(`${bigWorlds.length} big worlds → ${bigOut}`);
}
if (problems.length) {
  console.log(`${problems.length} problems:`);
  for (const p of problems.slice(0, 60)) console.log(`  ${p}`);
  process.exitCode = 1;
}
