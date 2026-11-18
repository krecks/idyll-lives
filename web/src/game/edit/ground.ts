/**
 * The generator's `Ground` (`town.ts`: what is taken on a world, where a lot may go) made again
 * from a draft, for the town edits: its roads, greens, water with its shore strip, every lot, the
 * backdrop, the parking, the squares, the trails and the orchards, each taken as the generator took
 * it when it laid them (`macro.ts` `generateFromSpec`). And where each lot meets its road.
 *
 * What the lots do not change (the roads, greens, water, parking, squares, trails and orchards) is
 * worked out once per set of roads and kept; the lots and the backdrop are put on a copy of it.
 */

import { unrle } from '../landMasks';
import { grown, SHORE_GAP } from '../macro';
import { inside, nearest, pointAt, roadIndex, type Pt, type RoadPlan } from '../roads';
import { boundsOf, Ground, plotRect, turner, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates } from '../town';

/** A place along a road: which road, how far along it (m), and which side (+1: its left, looking along it; -1: its right). */
export interface RoadSpot {
  road: number;
  s: number;
  side: 1 | -1;
}

/**
 * A place's lots and buildings keep a strip of bank (`macro.ts` `SHORE_GAP`) between them and the
 * water. A classic world has no strip there; its towns are not edited in step 2, so the strip is kept
 * here for every world.
 */

/** What a set of roads' fixed ground was made from, and the mask of what it takes besides the roads and greens. */
interface Base {
  w: number;
  d: number;
  greens: unknown;
  water: unknown;
  parking: unknown;
  squares: unknown;
  zones: unknown;
  mask: Uint8Array;
}

/** Each set of roads' fixed ground (see `Base`). */
const bases = new WeakMap<readonly RoadPlan[], Base>();

/** The tiles the water (with its shore strip), the parking, the squares, the trails and the orchards take on `draft`'s world. */
function baseOf(draft: NeighbourhoodDraft): Uint8Array {
  const { width: W, depth: D, roads, greens, open } = draft;
  const had = bases.get(roads);
  if (had && had.w === W && had.d === D && had.greens === greens && had.water === open?.water && had.parking === open?.parking && had.squares === open?.squares && had.zones === open?.zones) return had.mask;
  const mask = new Uint8Array(W * D);
  const take = (r: Rect, grow = 0) => {
    for (let z = Math.max(0, r.z - grow); z < Math.min(D, r.z + r.d + grow); z++) for (let x = Math.max(0, r.x - grow); x < Math.min(W, r.x + r.w + grow); x++) mask[z * W + x] = 1;
  };
  if (open?.water) mask.set(grown(unrle(open.water, W * D), W, D, SHORE_GAP));
  // A parking space as the generator laid it: 6 m along the verge, 3 m deep, and a metre round it.
  for (const p of open?.parking ?? []) take(p.rot % 2 ? { x: Math.round(p.x - 3), z: Math.round(p.z - 1.5), w: 6, d: 3 } : { x: Math.round(p.x - 1.5), z: Math.round(p.z - 3), w: 3, d: 6 }, 1);
  // A square's own tiles (those whose middle is in it), as the generator keeps them clear (`chooseCentre`).
  for (const q of open?.squares ?? []) {
    const b = boundsOf(q);
    for (let z = Math.max(0, Math.floor(b.z)); z < Math.min(D, Math.ceil(b.z + b.d)); z++) for (let x = Math.max(0, Math.floor(b.x)); x < Math.min(W, Math.ceil(b.x + b.w)); x++) if (inside(q, x + 0.5, z + 0.5)) mask[z * W + x] = 1;
  }
  // Nothing goes down on a trail (a tile round each of its points), nor in an orchard (and 2 m round it).
  for (const r of roads) if (r.kind === 'trail') for (const [x, z] of r.points) take({ x: Math.floor(x) - 1, z: Math.floor(z) - 1, w: 3, d: 3 });
  for (const zone of open?.zones ?? []) if (zone.kind === 'orchard' && zone.poly.length) take(boundsOf(zone.poly), 2);
  bases.set(roads, { w: W, d: D, greens, water: open?.water, parking: open?.parking, squares: open?.squares, zones: open?.zones, mask });
  return mask;
}

/**
 * The ground of `draft` as the generator would have it with everything on it laid already: what
 * its roads, greens, water and shore strip, parking, squares, trails and orchards take, every lot
 * (a terrace lot with a metre round it, as `LotPlacer.placeRow` takes a row), and every backdrop
 * building with a metre round it (as `withGarden` keeps off them). `except`: that lot left off
 * (one being moved or refitted). A new `Ground` each call, the caller's to take more of.
 */
export function townGround(draft: NeighbourhoodDraft, t: Templates, except = -1): Ground {
  const ground = new Ground(draft.width, draft.depth, draft.roads, draft.greens);
  ground.block(baseOf(draft));
  for (const s of draft.slots) if (s.index !== except) ground.take(plotRect(t, s), s.lot === 'terrace' ? 1 : 0);
  for (const b of draft.open?.backdrop ?? []) ground.take(boundsOf(b.poly), 1);
  return ground;
}

/** Each draft's ground (`townGround`), and the last one asked for with a lot left off. */
const kept = new WeakMap<NeighbourhoodDraft, { all?: Ground; except?: { slot: number; ground: Ground } }>();

/** `townGround(draft, t, except)` made once per draft (and per lot left off) and kept, for the checks made on every pointer move: only to be read, never taken from. */
export function groundOf(draft: NeighbourhoodDraft, t: Templates, except = -1): Ground {
  let had = kept.get(draft);
  if (!had) kept.set(draft, (had = {}));
  if (except < 0) return (had.all ??= townGround(draft, t));
  if (had.except?.slot !== except) had.except = { slot: except, ground: townGround(draft, t, except) };
  return had.except.ground;
}

/** The middle of a lot's front edge on the town: where it meets its road. */
export function frontOf(t: Templates, s: Pick<PlotSlot, 'lot' | 'w' | 'd' | 'x' | 'z' | 'facing'>): Pt {
  const r = plotRect(t, s);
  // (Its width along its street and its depth: turned a quarter, it lies across.)
  const [lw, ld] = s.facing % 2 ? [r.d, r.w] : [r.w, r.d];
  return turner(r.x, r.z, lw, ld, s.facing).point([lw / 2, 0]);
}

/** Each set of slots' entries (`entriesOf`), and the roads they were measured on. */
const entries = new WeakMap<readonly PlotSlot[], { roads: readonly RoadPlan[]; spots: RoadSpot[] }>();

/**
 * Where each lot of `draft` meets its road (by slot index): the road nearest the middle of its
 * front (not a trail, but for an outdoor place, which its trail leads to), how far along it and
 * which side. A park on a green meets the road nearest it. `road` is -1 in a town with no roads.
 */
export function entriesOf(draft: NeighbourhoodDraft, t: Templates): RoadSpot[] {
  const had = entries.get(draft.slots);
  if (had && had.roads === draft.roads) return had.spots;
  const spots = draft.slots.map((s) => entryOf(draft, t, s));
  entries.set(draft.slots, { roads: draft.roads, spots });
  return spots;
}

/** Where lot `s` meets its road (see `entriesOf`). */
export function entryOf(draft: NeighbourhoodDraft, t: Templates, s: Pick<PlotSlot, 'kind' | 'lot' | 'w' | 'd' | 'x' | 'z' | 'facing'>): RoadSpot {
  const { roads } = draft;
  const [fx, fz] = frontOf(t, s);
  const index = roadIndex(roads);
  const may = (r: number) => (roads[r].kind === 'trail') === (s.kind === 'outdoor');
  // The way out of its front (the way it faces): a road straight ahead of it before one off to the
  // side at a corner, each the further off the further it is from straight ahead.
  const [ox, oz] = ([[0, -1], [-1, 0], [0, 1], [1, 0]] as const)[s.facing];
  let best = -1;
  let score = Infinity;
  for (const [road, hit] of index.around(fx, fz, 40)) {
    if (!may(road)) continue;
    const [px, pz] = pointAt(roads[road].points, hit.s).p;
    const ahead = hit.dist > 0.01 ? ((px - fx) * ox + (pz - fz) * oz) / hit.dist : 1;
    const by = hit.dist / Math.max(0.2, ahead);
    if (by < score) [best, score] = [road, by];
  }
  // (Far from any of its kind: the nearest of all.)
  let dist = Infinity;
  if (best < 0) {
    roads.forEach((r, i) => {
      const d = r.points.length ? nearest(r.points, fx, fz).dist : Infinity;
      if (d < dist) [best, dist] = [i, d];
    });
  }
  if (best < 0) return { road: -1, s: 0, side: 1 };
  const { s: along, side } = nearest(roads[best].points, fx, fz);
  return { road: best, s: along, side };
}

/** Whether point `p` is inside one of the town centres' outlines (`open.centre`). */
export function inCentre(draft: NeighbourhoodDraft, [x, z]: Pt): boolean {
  return (draft.open?.centre ?? []).some((c) => c.poly.length >= 3 && inside(c.poly, x, z));
}
