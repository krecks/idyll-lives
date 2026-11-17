/**
 * What is wrong with a town, read from its town file (`assembleTown`): the layout checks the
 * sample tool runs over every place's worlds (`tools/worlds/sample.ts`), and step 2 over a town
 * edited by hand. `townProblems` lists them: `hard` (a town the sim may not run well, or at all),
 * `soft` (worth knowing: a career with nowhere to work in the main town).
 *
 * For every world: the roads (country roads and trails too) one network as the sim joins them (an
 * end within half the wider road's width and a metre of the other's middle), every way out on it,
 * no more courts (dead ends) than an area keeps, none reaching more than `COURT` m past the last
 * road it meets; every area reaching a way out, nothing on the water, every outdoor place at a
 * trail's end and near its area, no trail through a lot, no country road turning further than 70°
 * over 40 m (no hairpins: `COUNTRY_TURN`), none running alongside another road a few metres off for
 * more than `SIDE_BY_SIDE` m, areas apart, hamlets of four lots or more, residents within the cap,
 * every road named (no two alike); lots inside the world, apart from each other and off the roads,
 * each by its road (a park may stand on a green), the ways out on its edge. Over the water only
 * bridges and causeways, each over its deck (`bridge`) alone, straight and not too long; a river
 * world (`river`) has a bridge. And the living town's backdrop, centre and places (`livingProblems`).
 *
 * With `expect` (`world.ts` `expectations`), the world is filled and its homes as many as its place
 * and options want: so much of its land less the water built and farmed, at most so many of its
 * 100 m squares with nothing in them, so many of its quarters with an area or homes out in the
 * country, so many homes, so many ways out, the backdrop buildings each area wants, and (in a town
 * or a city) every career working somewhere in the main town.
 */

import type { BackdropBuilding, LandFile, RoadPlan, TownPlace } from '../core/protocol';
import { centreDensity, facesRoad, fillPoly, HAMLET_LEAST, LOT_CLEAR, overlaps, signedArea2 } from './backdrop';
import { AREA_APART, COUNTRY_TURN, unrle } from './macro';
import { COURT, inside, lengths, nearest, pointAt, RoadIndex, roadMask, sharpestTurn, sideBySide, type Pt } from './roads';
import { BIG_COURTS, COURTS, NEIGHBOUR_CAP, turner, type Facing } from './town';
import type { Expected } from './world';

type Rect = { x: number; z: number; w: number; d: number };
type Exit = Pt | { x: number; z: number; kind: 'road' };
/** A town file as the checks read it (`assembleTown`'s JSON; only what they look at). */
export type CheckedTown = {
  size?: 'big';
  width: number;
  depth: number;
  plots: (Rect & { public: boolean; area?: number; kind?: string })[];
  exits: Exit[];
  sims: { household: number }[];
  households: { player: boolean }[];
  roads?: RoadPlan[];
  areas?: { name: string; x: number; z: number; w: number; d: number }[];
  terrain?: { water?: number[] };
  /**
   * Whether the land itself is under water at (x, z) (a step 2 draft's, `edit/checked.ts`; none in a town file). A
   * bridge the Street tool lays over the water as the land has it (its tiles) may cross a pond or a corner of a shore
   * the water zones' outlines leave out (pieces under 4 samples dropped, corners rounded): its deck is over water.
   */
  landWet?: (x: number, z: number) => boolean;
  meta: {
    roads?: RoadPlan[];
    paths?: Rect[];
    land?: LandFile;
    world?: { landscape?: { slope?: number } };
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

/** A square holds something if this much of it does (m²). */
const HOLDS = 200;
/** The longest deck a bridge has over the water, a causeway (a road out of the world) (m). */
const SPAN = 45;
const CAUSEWAY = 140;
/** A bridge's deck runs straight: it turns no further than this (radians) end to end. */
const DECK_TURN = 0.05;

/**
 * How filled a big world is: the share of its land (less the water) built and farmed, the share of
 * its 100 m squares with nothing in them (with `meadow`, a heath's: its meadows, the heather, count).
 */
export function fillOf(town: CheckedTown, meadow = false): { built: number; empty: number } {
  const had = fills.get(town);
  if (had) return had;
  const fill = measureFill(town, meadow);
  fills.set(town, fill);
  return fill;
}
/** Each town file's `fillOf`, measured once (the checks and the log line both ask). */
const fills = new WeakMap<CheckedTown, { built: number; empty: number }>();
function measureFill(town: CheckedTown, meadow: boolean): { built: number; empty: number } {
  const { width: W, depth: D } = town;
  const C = 2;
  const [gw, gd] = [Math.ceil(W / C), Math.ceil(D / C)];
  const built = new Uint8Array(gw * gd);
  const some = new Uint8Array(gw * gd);
  const wet = new Uint8Array(gw * gd);
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
    } else if (z.kind === 'woods' || z.kind === 'water' || (meadow && z.kind === 'meadow')) poly(some, z.poly);
    if (z.kind === 'water') poly(wet, z.poly);
  }
  const roads = roadMask(town.roads ?? town.meta.roads ?? [], W, D, 0);
  for (let z = 0; z < D; z += C) for (let x = 0; x < W; x += C) if (roads[z * W + x]) some[(z / C) * gw + x / C] = 1;
  let [filled, land] = [0, 0];
  for (let i = 0; i < built.length; i++) {
    if (wet[i]) continue;
    filled += built[i];
    land++;
  }
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
  return { built: filled / Math.max(1, land), empty: empty / squares };
}

/** What's wrong with how a town's roads join up, its areas, water, bridges, trails and names (the open-world checks), and how filled it is (`expect`, if given). */
function openProblems(town: CheckedTown, expect: Expected | undefined, river: boolean, index: RoadIndex): string[] {
  const out: string[] = [];
  const roads = index.roads;
  // Joined as the sim joins them (travel.rs `Graph::build`): an end within half the wider road's width and a metre of the other's middle.
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  /** Per road, where along it the other roads' ends meet it (m). */
  const joins: number[][] = roads.map(() => []);
  const widest = Math.max(0, ...roads.map((r) => r.width));
  roads.forEach((r, i) => {
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      // (Only the roads passing near the end: those further can't meet it.)
      for (const j of index.roadsNear(end[0], end[1], Math.max(widest, r.width) / 2 + 1.01)) {
        const o = roads[j];
        const near = j === i ? null : nearest(o.points, end[0], end[1]);
        if (!near || near.dist > Math.max(o.width, r.width) / 2 + 1) continue;
        parent[find(i)] = find(j);
        joins[j].push(near.s);
      }
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
  out.push(...bridgeProblems(town, roads, river));
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
  // (How filled it is only where something is wanted of it: an edited town's checks ask nothing.)
  if (expect && (expect.built > 0 || expect.empty !== null)) {
    const fill = fillOf(town, expect.meadow);
    const pc = (v: number) => `${(v * 100).toFixed(0)}%`;
    if (fill.built < expect.built) out.push(`${pc(fill.built)} of the land built and farmed (${pc(expect.built)} or more wanted)`);
    if (expect.empty !== null && fill.empty > expect.empty) out.push(`${pc(fill.empty)} of its 100 m squares hold nothing (${pc(expect.empty)} at most)`);
  }
  if (expect) {
    // Spread over the whole world: each quarter of it an area's middle, or a farmstead or cottage out in the country.
    const inArea = (p: Rect) => areas.some((a) => offRect(a, p.x + p.w / 2, p.z + p.d / 2) === 0);
    const country = town.plots.filter((p) => (p.kind === 'home' || p.kind === 'farm') && !inArea(p)).map((p): Pt => [p.x + p.w / 2, p.z + p.d / 2]);
    const quarter = ([x, z]: Pt) => (x < town.width / 2 ? 0 : 1) + (z < town.depth / 2 ? 0 : 2);
    const held = new Set([...centres, ...country].map(quarter));
    if (held.size < expect.quarters) out.push(`only the ${[...held].map((q) => ['north-west', 'north-east', 'south-west', 'south-east'][q]).join(', ')} quarters have an area, farmstead or cottage (${expect.quarters} wanted)`);
  }
  areas.forEach((a, k) => {
    const lots = town.plots.filter((p) => (p.area ?? 0) === k && p.kind !== 'countryside').length;
    if (k > 0 && lots < 4) out.push(`hamlet ${a.name} has ${lots} lots`);
  });
  const homes = town.plots.filter((p) => p.kind === 'home' || p.kind === 'farm');
  if (expect && homes.length < expect.homes) out.push(`${homes.length} homes (${expect.homes} or more wanted)`);
  // Neighbours within the cap (the player's household aside).
  const player = town.households.findIndex((h) => h.player);
  const neighbours = town.sims.filter((s) => s.household !== player).length;
  if (neighbours > NEIGHBOUR_CAP) out.push(`${neighbours} neighbours (the cap is ${NEIGHBOUR_CAP})`);
  return out;
}

/**
 * What's wrong with a world's bridges and causeways: a road over the water (a water zone) but on
 * its deck (`bridge`, its arc lengths); a deck longer than `SPAN` (a causeway's, a road out of the
 * world, `CAUSEWAY`) or not straight; a deck over no water; a river (`river`) with no bridge.
 */
function bridgeProblems(town: CheckedTown, roads: readonly RoadPlan[], river: boolean): string[] {
  const out: string[] = [];
  const water = (town.meta.zones ?? []).filter((z) => z.kind === 'water').map((z) => z.poly);
  // (At the 1 m tile's middle, as the water's tiles are made from the zones: `macro.ts` `rasterize`.)
  const wet = (x: number, z: number) => water.some((poly) => inside(poly, Math.floor(x) + 0.5, Math.floor(z) + 0.5));
  for (const [i, r] of roads.entries()) {
    const name = r.name ?? `road ${i}`;
    const ls = lengths(r.points);
    if (!r.bridge) {
      const at = r.points.find(([x, z]) => wet(x, z));
      if (at) out.push(`${name} runs over the water at ${at[0].toFixed(0)}, ${at[1].toFixed(0)} with no bridge`);
      continue;
    }
    const [s0, s1] = r.bridge;
    if (!(s0 >= 0 && s1 > s0 && s1 <= ls[ls.length - 1] + 0.01)) {
      out.push(`${name}'s deck runs ${s0} to ${s1} m along it (${ls[ls.length - 1].toFixed(0)} m long)`);
      continue;
    }
    const off = r.points.findIndex(([x, z], k) => wet(x, z) && (ls[k] < s0 || ls[k] > s1));
    if (off >= 0) out.push(`${name} runs over the water off its deck at ${r.points[off][0].toFixed(0)}, ${r.points[off][1].toFixed(0)}`);
    if (!r.points.some(([x, z], k) => ls[k] >= s0 && ls[k] <= s1 && (wet(x, z) || !!town.landWet?.(x, z)))) out.push(`${name}'s deck is over no water`);
    const most = r.ends.includes('out') ? CAUSEWAY : SPAN;
    if (s1 - s0 > most) out.push(`${name}'s deck is ${(s1 - s0).toFixed(0)} m long (${most} m at most)`);
    const turn = Math.abs(headingAt(r.points, s1, ls) - headingAt(r.points, s0, ls));
    if (Math.min(turn, 2 * Math.PI - turn) > DECK_TURN) out.push(`${name}'s deck turns ${((turn * 180) / Math.PI).toFixed(0)}°`);
  }
  if (river && !roads.some((r) => r.bridge && !r.ends.includes('out'))) out.push('a river with no bridge over it');
  return out;
}
/** Which way a polyline heads at arc length `s` (radians). */
const headingAt = (pts: readonly Pt[], s: number, ls: number[]) => {
  const { t } = pointAt(pts, s, ls);
  return Math.atan2(t[1], t[0]);
};

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
 * nearest it (`facesRoad`); as many as its size wants in the main town (`expect.mainLeast`) and `HAMLET_LEAST` in every hamlet,
 * the centre's open land built over more densely than the rest of the main town's box's. Every public lot a
 * place, every backdrop place a place of its kind; each place named (none twice), its hours within
 * the day, its work careers that are, and (in a town or a city: `expect.careers`) every career a
 * resident may hold working somewhere in the main town. The main town has its centre.
 */
function livingProblems(town: CheckedTown, careers: readonly string[], expect: Expected | undefined, given: Record<number, number>, soft: string[], index: RoadIndex, generated: boolean): string[] {
  const out: string[] = [];
  const backdrop = town.meta.backdrop;
  const places = town.meta.places;
  if (!backdrop || !places) return ['no backdrop or places'];
  const areas = town.areas ?? [];
  const roads = index.roads;
  // (The roads' points by the 16 m cell they are in: a building looks only at those round it.)
  const points = new Map<number, Pt[]>();
  for (const r of roads) {
    for (const p of r.points) {
      const k = Math.floor(p[0] / 16) * 65536 + Math.floor(p[1] / 16);
      const list = points.get(k);
      if (list) list.push(p);
      else points.set(k, [p]);
    }
  }
  const pointsIn = (poly: readonly Pt[]) => {
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    for (let cz = Math.floor(Math.min(...zs) / 16); cz <= Math.floor(Math.max(...zs) / 16); cz++) {
      for (let cx = Math.floor(Math.min(...xs) / 16); cx <= Math.floor(Math.max(...xs) / 16); cx++) if (points.get(cx * 65536 + cz)?.some(([x, z]) => inside(poly, x, z))) return true;
    }
    return false;
  };
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
    if (pointsIn(poly)) say(i, 'has a road through it');
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
    // (A town checked with no expectations, one edited by hand: as many as it has.)
    if (!expect) return;
    const n = backdrop.filter((b) => b.area === k).length;
    // (Less those the slope limits gave up there: `landFollow.ts` `settleLand`.)
    const least = (k === 0 ? expect.mainLeast : HAMLET_LEAST) - (given[k] ?? 0);
    if (n < least) out.push(`${a.name} has ${n} backdrop buildings (${least} or more)`);
  });
  // The centre: the main town's, its open land (not a road, a lot, a green or water) built over more
  // densely than the rest of its box's (how the generator fills a town: not asked of an edited one).
  const centre = town.meta.centre?.find((c) => c.area === 0);
  if (!generated) {
    // (Not checked.)
  } else if (!centre || centre.poly.length < 3) out.push('the main town has no centre');
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
  if (expect?.careers) for (const c of careers) if (!places.some((p) => p.area === 0 && p.work?.includes(c))) soft.push(`no place in the main town where ${c} workers work`);
  return out;
}

/**
 * How long a country road may run alongside another road a few metres off (m): where it leaves a
 * port beside a street's end, or joins a road at a slant (`layoutProblems`). Two that follow the
 * same way for longer share it.
 */
const SIDE_BY_SIDE = 60;
function layoutProblems(town: CheckedTown, expect: Expected | undefined, river: boolean, index: RoadIndex): string[] {
  const out: string[] = openProblems(town, expect, river, index);
  // (Trails are on foot, between the lots: `openProblems` checks they keep off them.)
  const roads = (town.meta.roads ?? []).filter((r) => r.kind !== 'trail');
  const mask = roadMask(roads, town.width, town.depth, 0);
  // (The roads near a lot's front, found through the index when it is of the same roads.)
  const ways = index.roads === town.meta.roads ? (town.meta.roads ?? []).flatMap((r, i) => (r.kind !== 'trail' ? [i] : [])) : null;
  const wayOf = ways && new Map(ways.map((i, k) => [i, k]));
  const widest = Math.max(0, ...roads.map((r) => r.width));
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
    // (The roads near it first: one within 13 m of it settles it; else every road, for how far it is.)
    const gapOf = (list: readonly RoadPlan[]) => Math.min(...list.map((r) => nearest(r.points, fx, fz).dist - r.width / 2));
    let gap = wayOf ? gapOf(index.roadsNear(fx, fz, 13 + widest / 2 + 0.01).flatMap((i) => (wayOf.has(i) ? [roads[wayOf.get(i)!]] : []))) : Infinity;
    if (!(gap <= 13)) gap = gapOf(roads);
    // (A park may stand in the middle of a green.)
    if (roads.length && gap > 13 && !p.public) out.push(`plot ${i} ${gap.toFixed(1)} m from its road`);
  });
  // No country road alongside another road a few metres off for long (`roads.ts` `sideBySide`: cars
  // on the two meet nose to nose): one that follows another's way shares it (`macro.ts` `alongExisting`).
  const all = town.meta.roads ?? [];
  for (const { a, b, length, at } of sideBySide(all, SIDE_BY_SIDE, 'country')) out.push(`${all[a].name} runs alongside ${all[b].name} a few metres off for ${length} m by ${at.map(Math.round).join(', ')}`);
  for (const [x, z] of town.exits.map(exitAt)) if (Math.min(x, z, town.width - x, town.depth - z) > 1) out.push(`a way out of town at ${x}, ${z}, not on its edge`);
  if (!town.exits.length) out.push('no way out of town');
  if (expect?.exits && (town.exits.length < expect.exits[0] || town.exits.length > expect.exits[1])) out.push(`${town.exits.length} ways out of the world (${expect.exits[0]} to ${expect.exits[1]} wanted)`);
  return out;
}

/**
 * What is wrong with `town` (see the top of this file): `river`, a river world's (it wants a bridge);
 * `careers`, the careers a resident may hold; `given`, per area, the backdrop buildings the slope
 * limits gave up there (`landFollow.ts` `settleLand`: fewer are wanted); `generated` false: not the
 * checks of how the generator fills a town (the centre the densest), which an edited town needn't keep.
 */
export function townProblems(town: CheckedTown, expect?: Expected, { river = false, careers = [], given = {}, generated = true }: { river?: boolean; careers?: readonly string[]; given?: Record<number, number>; generated?: boolean } = {}): { hard: string[]; soft: string[] } {
  const soft: string[] = [];
  // (One index of the roads for every check that asks which are near a point.)
  const index = new RoadIndex(town.roads ?? town.meta.roads ?? []);
  const hard = [...layoutProblems(town, expect, river, index), ...livingProblems(town, careers, expect, given, soft, index, generated)];
  return { hard, soft };
}
