/**
 * Makes sample worlds of every kind (web/src/game/world.ts) with the shipped content, the way a
 * new game does (a world, its neighbours, a player household moving into an empty house), and
 * writes their town files for the simulation to check
 * (`cargo test -p sim-core --test sim generated_worlds:: -- --ignored`, reading WORLDS). Checks the
 * layout first: lots inside the town, apart from each other and off the roads, each by its road (a
 * park may stand on a green),
 * the ways out of town on its edge. And the neighbours: every household's savings cover its
 * home's price (web/src/game/wealth.ts, price.ts); per kind of house, how old its owners are and
 * how high up they work. Problems are listed (and the exit code is 1).
 *
 * Sizes: every world is a Big one now (cosy and normal were dropped
 * in 0.8.0). For every world: the roads (country roads and trails too)
 * one network as the sim joins them (an end within half the wider road's width and a metre of the
 * other's middle), every way out on it, no more courts (dead ends) than an area keeps, none
 * reaching more than 45 m past the last road it meets, and no country road the layout found no
 * way for. And for big worlds: every area reaching a way out, nothing on the
 * water, every outdoor place at a trail's end and near its area, no trail through a lot, no
 * country road turning further than 70° over 40 m (no hairpins: `COUNTRY_TURN`), none running
 * alongside another road a few metres off for more than `SIDE_BY_SIDE` m, areas apart,
 * hamlets of four lots or more, residents within the
 * cap, the world filled (a quarter of it or more built and farmed, at most a tenth of its 100 m
 * squares with nothing in them, every quarter of it with an area or homes out in the country),
 * every road named (no two alike), the file under 4 MB, and the time a big world takes (its
 * layout without generating houses, and assembling its file: a problem at twice the budget).
 * And the living town's backdrop, centre and places (`livingProblems`).
 * Each world's line counts its terraced houses, backdrop buildings and places.
 *
 * The same arguments make the same files, byte for byte (`Math.random` seeded per world:
 * web/tests/sampleWorlds.test.ts), so a world a check fails on can be made again and run alone.
 * `--seed-base n` (or WORLDS_SEED_BASE; default 0, the worlds CI checks) draws other worlds: the
 * release gate runs the checks over five (`tools/ci-local.sh`, SEED_BASE).
 *
 * Run: bun tools/worlds/sample.ts worlds.json [worlds per kind] [--kinds farm,forest] [--seed-base n] [--no-timing]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';
import type { Facing, Templates } from '../../web/src/game/town.ts';
import type { BackdropBuilding, RoadPlan, TownPlace } from '../../web/src/core/protocol.ts';

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
const { assembleTown, BIG_COURTS, COURTS, generateStats, generateWorld, houseTemplate, NEIGHBOUR_CAP, populate, turner, vacantSlots } = await import('../../web/src/game/town.ts');
const { WORLD_KINDS } = await import('../../web/src/game/world.ts');
const { AREA_APART, COUNTRY_TURN, unrle } = await import('../../web/src/game/macro.ts');
const { homePrice } = await import('../../web/src/game/price.ts');
const { savingsOf } = await import('../../web/src/game/wealth.ts');
const { COURT, lengths, nearest, pointAt, roadMask, sharpestTurn, sideBySide } = await import('../../web/src/game/roads.ts');
const { mulberry32 } = await import('../../web/src/game/noise.ts');
const { centreDensity, facesRoad, fillPoly, HAMLET_LEAST, LOT_CLEAR, MAIN_LEAST, overlaps, signedArea2 } = await import('../../web/src/game/backdrop.ts');
const { RoadIndex, inside } = await import('../../web/src/game/roads.ts');

type Rect = { x: number; z: number; w: number; d: number };
type Pt = [number, number];
type Road = { points: Pt[]; width: number; ends: [string, string]; name?: string; kind?: string; area?: number };
type Exit = Pt | { x: number; z: number; kind: 'road' };
type TownFile = {
  size?: 'big';
  width: number;
  depth: number;
  plots: (Rect & { public: boolean; area?: number; kind?: string })[];
  exits: Exit[];
  sims: { household: number }[];
  households: { player: boolean }[];
  roads?: Road[];
  areas?: { name: string; x: number; z: number; w: number; d: number }[];
  terrain?: { water?: number[] };
  meta: {
    roads?: Road[];
    facing?: number[];
    zones?: { kind: string; poly: Pt[] }[];
    greens?: Pt[][];
    backdrop?: BackdropBuilding[];
    places?: TownPlace[];
    centre?: { area: number; poly: Pt[] }[];
  };
  parking?: { x: number; z: number; rot: number; area: number }[];
};
const exitAt = (e: Exit): Pt => (Array.isArray(e) ? e : [e.x, e.z]);
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
/** A square holds something if this much of it does (m²). */
const HOLDS = 200;

/** How filled a big world is: the share of it built and farmed, the share of its 100 m squares with nothing in them. */
function fillOf(town: TownFile): { built: number; empty: number } {
  const had = fills.get(town);
  if (had) return had;
  const fill = measureFill(town);
  fills.set(town, fill);
  return fill;
}
/** Each town file's `fillOf`, measured once (the checks and the log line both ask). */
const fills = new WeakMap<TownFile, { built: number; empty: number }>();
function measureFill(town: TownFile): { built: number; empty: number } {
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
    const most = k === 0 ? BIG_COURTS : COURTS;
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
  const centres = areas.map((a): Pt => [a.x + a.w / 2, a.z + a.d / 2]);
  centres.forEach((c, i) =>
    centres.forEach((o, j) => {
      const d = Math.hypot(c[0] - o[0], c[1] - o[1]);
      if (j > i && d < AREA_APART) out.push(`${areas[i].name} and ${areas[j].name} ${d.toFixed(0)} m apart`);
    }),
  );
  // Country roads without hairpins: none turning further than `COUNTRY_TURN` over 40 m of its own
  // middle line (two spans of 20 m: a steady bend of 16 m radius or more), where it meets
  // another road aside.
  for (const r of roads) {
    if (r.kind !== 'country') continue;
    const { angle, s } = sharpestTurn(r.points, 20);
    const [x, z] = pointAt(r.points, s).p;
    const deg = (a: number) => ((a * 180) / Math.PI).toFixed(0);
    if (angle > COUNTRY_TURN + 1e-3) out.push(`country road ${r.name} turns ${deg(angle)}° over 40 m at ${x.toFixed(0)}, ${z.toFixed(0)} (${deg(COUNTRY_TURN)}° at most)`);
  }
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
  // Neighbours within the cap (the player's household aside).
  const player = town.households.findIndex((h) => h.player);
  const neighbours = town.sims.filter((s) => s.household !== player).length;
  if (neighbours > NEIGHBOUR_CAP) out.push(`${neighbours} neighbours (the cap is ${NEIGHBOUR_CAP})`);
  return out;
}

/** What a backdrop building may be, and its roofs; the places' kinds. */
const USES = ['flats', 'terrace', 'house', 'shop', 'cafe', 'restaurant', 'office', 'school', 'church', 'hall', 'workshop', 'farm', 'barn'];
const ROOFS = ['gable', 'hip', 'flat', 'mansard'];
const PLACE_KINDS = ['shop', 'cafe', 'restaurant', 'office', 'school', 'church', 'park', 'outdoor', 'workshop', 'farm'];
/** A rectangle's corners, grown by `by` m. */
const corners = (r: Rect, by = 0): Pt[] => [
  [r.x - by, r.z - by],
  [r.x + r.w + by, r.z - by],
  [r.x + r.w + by, r.z + r.d + by],
  [r.x - by, r.z + r.d + by],
];
/**
 * The living town's world data (`game/backdrop.ts`): every
 * backdrop building a rectangle (4 to 8 corners, counter-clockwise seen from above), 1 to 6 storeys,
 * a roof (a ridge for a gable or mansard), a use, its area and its place; on no lot (and
 * `LOT_CLEAR` m off one), road or trail with its sidewalks (more than half its width off its
 * middle: a junction's pad too), parking, water or other backdrop building; a side along the road
 * nearest it (`facesRoad`); `MAIN_LEAST` or more in the main town and `HAMLET_LEAST` in every hamlet,
 * the centre's open land built over more densely than the rest of the main town's box's. Every public lot a
 * place, every backdrop place a place of its kind; each place named (none twice), its hours within
 * the day, its work careers that are, and every career a resident may hold working somewhere in
 * the main town. The main town has its centre.
 */
function livingProblems(town: TownFile, careers: readonly string[]): string[] {
  const out: string[] = [];
  const backdrop = town.meta.backdrop;
  const places = town.meta.places;
  if (!backdrop || !places) return ['no backdrop or places'];
  const areas = town.areas ?? [];
  const roads = (town.roads ?? town.meta.roads ?? []) as RoadPlan[];
  const index = new RoadIndex(roads);
  const water = town.terrain?.water ? unrle(town.terrain.water, town.width * town.depth) : null;
  const lots = town.plots.map((p) => corners(p, LOT_CLEAR - 0.01));
  const parking = (town.parking ?? []).map((p) => corners(p.rot % 2 ? { x: p.x - 3, z: p.z - 1.5, w: 6, d: 3 } : { x: p.x - 1.5, z: p.z - 3, w: 3, d: 6 }));
  const say = (i: number, what: string) => out.push(`backdrop ${i} (${backdrop[i].use}) ${what}`);
  backdrop.forEach((b, i) => {
    const { poly } = b;
    if (poly.length < 4 || poly.length > 8) say(i, `has ${poly.length} corners`);
    if (signedArea2(poly) >= 0) say(i, 'goes clockwise seen from above');
    if (!Number.isInteger(b.storeys) || b.storeys < 1 || b.storeys > 6) say(i, `has ${b.storeys} storeys`);
    if (!ROOFS.includes(b.roof)) say(i, `has a ${b.roof} roof`);
    if ((b.roof === 'gable' || b.roof === 'mansard') && b.ridge === undefined) say(i, 'has no ridge');
    if (!USES.includes(b.use)) say(i, 'has no use');
    if (!Number.isInteger(b.area) || b.area < 0 || b.area >= Math.max(1, areas.length)) say(i, `is in area ${b.area}`);
    if (PLACE_KINDS.includes(b.use) !== (b.place !== undefined)) say(i, b.place === undefined ? 'is a place with no place' : 'has a place');
    if (b.place !== undefined && places[b.place]?.kind !== b.use) say(i, `is place ${b.place}, a ${places[b.place]?.kind}`);
    for (const [k, lot] of lots.entries()) if (overlaps(poly, lot, 0)) say(i, `is within ${LOT_CLEAR} m of plot ${k}`);
    for (const [k, spot] of parking.entries()) if (overlaps(poly, spot, 0)) say(i, `is on parking space ${k}`);
    for (let k = 0; k < poly.length; k++) {
      const [a, c] = [poly[k], poly[(k + 1) % poly.length]];
      const n = Math.max(1, Math.ceil(Math.hypot(c[0] - a[0], c[1] - a[1]) * 2));
      for (let j = 0; j <= n; j++) {
        const [x, z] = [a[0] + ((c[0] - a[0]) * j) / n, a[1] + ((c[1] - a[1]) * j) / n];
        if (index.onRoad(x, z)) {
          say(i, `stands on a road at ${x.toFixed(1)}, ${z.toFixed(1)}`);
          j = n;
          k = poly.length;
        }
      }
    }
    if (roads.some((r) => r.points.some(([x, z]) => inside(poly, x, z)))) say(i, 'has a road through it');
    if (water) {
      const xs = poly.map((p) => p[0]);
      const zs = poly.map((p) => p[1]);
      let wet = 0;
      for (let z = Math.floor(Math.min(...zs)); z < Math.ceil(Math.max(...zs)); z++) for (let x = Math.floor(Math.min(...xs)); x < Math.ceil(Math.max(...xs)); x++) if (inside(poly, x + 0.5, z + 0.5) && water[z * town.width + x]) wet++;
      if (wet) say(i, `stands on ${wet} tiles of water`);
    }
    // Along the road nearest it: one of its sides within `FACING` of the road's way (or square to it).
    if (!facesRoad(poly, roads, index)) say(i, "doesn't face the road nearest it");
  });
  // (Two touching is a row: overlapping is not.)
  const cell = new Map<string, number[]>();
  backdrop.forEach((b, i) => {
    const xs = b.poly.map((p) => p[0]);
    const zs = b.poly.map((p) => p[1]);
    for (let cz = Math.floor(Math.min(...zs) / 32); cz <= Math.floor(Math.max(...zs) / 32); cz++) {
      for (let cx = Math.floor(Math.min(...xs) / 32); cx <= Math.floor(Math.max(...xs) / 32); cx++) {
        const list = cell.get(`${cx},${cz}`) ?? [];
        for (const j of list) if (overlaps(b.poly, backdrop[j].poly)) say(i, `overlaps backdrop ${j}`);
        list.push(i);
        cell.set(`${cx},${cz}`, list);
      }
    }
  });
  areas.forEach((a, k) => {
    const n = backdrop.filter((b) => b.area === k).length;
    const least = k === 0 ? MAIN_LEAST : HAMLET_LEAST;
    if (n < least) out.push(`${a.name} has ${n} backdrop buildings (${least} or more)`);
  });
  // The centre: the main town's, its open land (not a road, a lot, a green or water) built over more
  // densely than the rest of its box's.
  const centre = town.meta.centre?.find((c) => c.area === 0);
  if (!centre || centre.poly.length < 3) out.push('the main town has no centre');
  else if (areas[0]) {
    const box = areas[0];
    const taken = roadMask(roads.filter((r) => r.kind !== 'trail'), town.width, town.depth, 0);
    for (const p of town.plots) fillPoly(corners(p), town.width, town.depth, (i) => (taken[i] = 1));
    // (Nor the water: a lakeside town's high street runs along its shore.)
    if (water) for (let i = 0; i < taken.length; i++) if (water[i]) taken[i] = 1;
    for (const g of town.meta.greens ?? []) fillPoly(g, town.width, town.depth, (i) => (taken[i] = 1));
    const { centre: inC, rest: inRest } = centreDensity(town.width, town.depth, taken, box, centre.poly, backdrop.map((b) => b.poly));
    if (inC <= inRest) out.push(`the centre's open land is ${(inC * 100).toFixed(0)}% built over, the rest of the main town's ${(inRest * 100).toFixed(0)}% (the centre the densest)`);
  }
  // Places: every public lot one, named once, open within the day, careers that are.
  town.plots.forEach((p, i) => {
    if (!p.public) return;
    const n = places.filter((q) => q.plot === i).length;
    if (n !== 1) out.push(`plot ${i} (${p.kind}) has ${n} places`);
  });
  const names = new Set<string>();
  places.forEach((p, i) => {
    if (!p.name) out.push(`place ${i} has no name`);
    else if (names.has(p.name)) out.push(`two places called ${p.name}`);
    names.add(p.name);
    if (!PLACE_KINDS.includes(p.kind)) out.push(`place ${p.name} is a ${p.kind}`);
    if (p.plot === undefined && !backdrop.some((b) => b.place === i)) out.push(`place ${p.name} is neither a lot nor a backdrop building`);
    if (p.hours && !(p.hours.open >= 0 && p.hours.open < p.hours.close && p.hours.close <= 1440 && p.hours.days > 0 && p.hours.days < 128)) out.push(`place ${p.name} is open ${JSON.stringify(p.hours)}`);
    if (!p.hours && p.kind !== 'park' && p.kind !== 'outdoor') out.push(`place ${p.name} (${p.kind}) has no hours`);
    for (const w of p.work ?? []) if (!careers.includes(w)) out.push(`place ${p.name}'s workers are ${w}, no career`);
    if (!Number.isInteger(p.area) || p.area < 0 || p.area >= Math.max(1, areas.length)) out.push(`place ${p.name} is in area ${p.area}`);
  });
  for (const c of careers) if (!places.some((p) => p.area === 0 && p.work?.includes(c))) out.push(`no place in the main town where ${c} workers work`);
  return out;
}

/**
 * How long a country road may run alongside another road a few metres off (m): where it leaves a
 * port beside a street's end, or joins a road at a slant (`layoutProblems`). Two that follow the
 * same way for longer share it.
 */
const SIDE_BY_SIDE = 60;
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
  // No country road alongside another road a few metres off for long (`roads.ts` `sideBySide`: cars
  // on the two meet nose to nose): one that follows another's way shares it (`macro.ts` `alongExisting`).
  const all = (town.meta.roads ?? []) as RoadPlan[];
  for (const { a, b, length, at } of sideBySide(all, SIDE_BY_SIDE, 'country')) out.push(`${all[a].name} runs alongside ${all[b].name} a few metres off for ${length} m by ${at.map(Math.round).join(', ')}`);
  for (const [x, z] of town.exits.map(exitAt)) if (Math.min(x, z, town.width - x, town.depth - z) > 1) out.push(`a way out of town at ${x}, ${z}, not on its edge`);
  if (!town.exits.length) out.push('no way out of town');
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
/** Which worlds of each kind (0: CI's); each world's seed moves on `SEED_STEP` per one. */
const seedBase = Number(flag('--seed-base') ?? process.env.WORLDS_SEED_BASE ?? 0);
if (!Number.isInteger(seedBase) || seedBase < 0) throw new Error(`no seed base ${seedBase}`);
const SEED_STEP = 7919;
/** `--no-timing`: the layout's and the file's time not checked against their budgets (a run beside others, on a slow runner: `web/tests/sampleWorlds.test.ts`). */
const timing = !process.argv.includes('--no-timing');
if (!timing) process.argv.splice(process.argv.indexOf('--no-timing'), 1);
const kinds = (flag('--kinds')?.split(',') ?? [...WORLD_KINDS]) as (typeof WORLD_KINDS)[number][];
for (const kind of kinds) if (!WORLD_KINDS.includes(kind)) throw new Error(`no kind ${kind}`);
const out = process.argv[2] ?? 'worlds.json';
const perKind = Number(process.argv[3] ?? 3);
// (A flag this script no longer takes: a call site that still passes one, rather than one silently
// ignored and the file it wanted never written.)
const leftover = process.argv.slice(2).filter((a) => a.startsWith('--'));
if (leftover.length) throw new Error(`no flag ${leftover.join(' ')}`);
const worlds: unknown[] = [];
/** The slowest world (ms): the layout (without houses), the houses, the file. */
const slowest: [number, number, number] = [0, 0, 0];
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
for (const kind of WORLD_KINDS.filter((k) => kinds.includes(k))) {
  for (let i = 1; i <= perKind; i++) {
    const seed = i * 104729 + kind.length + seedBase * SEED_STEP;
    const tag = `${kind} ${seed}`;
    // (The households, the player's and the neighbours', drawn with `Math.random`: seeded here
    // by the world's seed, the same seeds make the same files, so a failure in a
    // check of them can be run again.)
    Math.random = mulberry32(seed);
    generateStats.houseMs = 0;
    let t0 = performance.now();
    const world = generateWorld(content, templates, kind, seed);
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
    [slowest[0], slowest[1], slowest[2]] = [Math.max(slowest[0], layout), Math.max(slowest[1], houses), Math.max(slowest[2], assemble)];
    for (const p of [...layoutProblems(file), ...livingProblems(file, content.careerCategories.map((c) => c.id)), ...(world.diagnostics?.roads ?? [])]) problems.push(`${tag}: ${p}`);
    // (A terrace lot's house is a generated terraced house, its lot cut to it: a fixed layout is wider than the lot.)
    for (const s of town.slots) if (s.lot === 'terrace' && houseTemplate(templates, s.template)?.class !== 'terraced house') problems.push(`${tag}: terrace lot ${s.index} has a ${s.template}`);
    if (text.length > MAX_FILE) problems.push(`${tag}: the town file is ${(text.length / 1e6).toFixed(1)} MB`);
    if (timing && layout > LAYOUT_BUDGET * 2) problems.push(`${tag}: laid out in ${layout.toFixed(0)} ms (budget ${LAYOUT_BUDGET})`);
    if (timing && assemble > ASSEMBLE_BUDGET * 2) problems.push(`${tag}: assembled in ${assemble.toFixed(0)} ms (budget ${ASSEMBLE_BUDGET})`);
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
    worlds.push({ kind, seed, name: town.name, town: file });
    const farms = town.slots.filter((s) => s.lot === 'farm').length;
    const basements = town.slots.filter((s) => s.kind === 'house' && houseTemplate(templates, s.template)?.basement).length;
    const spots = town.slots.filter((s) => s.kind === 'outdoor').length;
    const dropped = world.diagnostics?.dropped.length ?? 0;
    const courts = town.roads.filter((r) => r.kind !== 'trail').reduce((n, r) => n + r.ends.filter((e) => e === 'dead').length, 0);
    [courtsOf[0], courtsOf[1], courtsOf[2]] = [courtsOf[0] + 1, courtsOf[1] + courts, Math.max(courtsOf[2], courts)];
    console.log(
      `${tag}: ${town.name} (${town.world?.tagline}) ${town.width}×${town.depth}, ${town.roads.length} road(s) (${courts} courts), ` +
        `${town.open ? `${town.open.areas.length} areas, ${spots} outdoor places, ${((f) => `${(f.built * 100).toFixed(0)}% built and farmed, ${(f.empty * 100).toFixed(0)}% squares empty, `)(fillOf(file))}` : ''}` +
        `${town.slots.filter((s) => s.kind === 'house').length} houses${farms ? ` (${farms} farms)` : ''} (${town.slots.filter((s) => s.lot === 'terrace').length} terraced)${dropped ? `, ${dropped} lots left out` : ''}, ${basements} with a basement, ${town.households.length} households, ` +
        `${town.open?.backdrop ? `${town.open.backdrop.length} backdrop buildings (${town.open.backdrop.filter((b) => b.area === 0).length} in the main town), ${town.open.places?.length ?? 0} places, ` : ''}` +
        `${file.sims.length} residents, ${file.objects.length} things, ${(text.length / 1e6).toFixed(2)} MB; ${layout.toFixed(0)} ms + ${houses.toFixed(0)} ms houses, file ${assemble.toFixed(0)} ms`,
    );
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
const [n, all, most] = courtsOf;
console.log(`Courts: ${all} in ${n} worlds (${(all / n).toFixed(1)} a world, ${most} at most)`);
console.log(`Slowest: layout ${slowest[0].toFixed(0)} ms, houses ${slowest[1].toFixed(0)} ms, file ${slowest[2].toFixed(0)} ms`);
writeFileSync(out, JSON.stringify(worlds));
console.log(`${worlds.length} worlds → ${out}`);
if (problems.length) {
  console.log(`${problems.length} problems:`);
  for (const p of problems.slice(0, 60)) console.log(`  ${p}`);
  process.exitCode = 1;
}
