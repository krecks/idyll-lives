/**
 * What step 2's tools point at, found on the town's plan draft (not on the town on screen, which
 * lags it until the town is built again): a lot (a home, a building of flats, a place...), a road, a
 * village or the main town, from a point on the ground (`menuScene.landAt`). And what is picked kept across edits, by
 * what it is (`keyOf`, `find`) rather than by its index, which moves as lots and roads go.
 */

import type { RoadPlan } from '../../core/protocol';
import type { RoadSpot } from '../../game/edit/ground';
import { lotKey, slotOf } from '../../game/edit/lots';
import { nearest, roadIndex } from '../../game/roads';
import { plotRect, type NeighbourhoodDraft, type Templates } from '../../game/town';

/** What a tool may point at: a lot (its slot), a road, a village (an area past the main town), the main town. */
export type Picked = { kind: 'lot'; slot: number } | { kind: 'road'; road: number } | { kind: 'village'; area: number } | { kind: 'town' };

/** A narrow road (a footpath) is pointed at this near its middle (m), as `menuScene.roadAt` has it. */
const ROAD_PICK = 1.5;
/** A lot is placed along the nearest road this near (m). */
const SPOT_REACH = 40;

/** What `pick` may find. */
export interface PickWhat {
  lots?: boolean;
  roads?: boolean;
  areas?: boolean;
}

/** The road at (`x`, `z`), if the point is on one (out to its edges; a footpath within `ROAD_PICK` m), the one whose middle is nearest. */
export function roadAtPoint(roads: readonly RoadPlan[], x: number, z: number): number | null {
  if (!roads.length) return null;
  let reach = ROAD_PICK;
  for (const r of roads) reach = Math.max(reach, r.width / 2);
  let best: { road: number; dist: number } | null = null;
  for (const [road, m] of roadIndex(roads).around(x, z, reach)) {
    if (m.dist > Math.max(roads[road].width / 2, ROAD_PICK) || (best && best.dist <= m.dist)) continue;
    best = { road, dist: m.dist };
  }
  return best?.road ?? null;
}

/** The lot at (`x`, `z`) on `draft`, if any. */
export function lotAtPoint(draft: NeighbourhoodDraft, t: Templates, x: number, z: number): number | null {
  for (const s of draft.slots) {
    const r = plotRect(t, s);
    if (x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d) return s.index;
  }
  return null;
}

/**
 * What is at ground point (`x`, `z`) on `draft`, of what `what` allows: a lot before a road (a
 * lot's edge may touch a road's; the lot is what was meant), then the village or the main town
 * whose box holds the point.
 */
export function pick(draft: NeighbourhoodDraft, t: Templates, x: number, z: number, what: PickWhat = { lots: true, roads: true, areas: true }): Picked | null {
  if (what.lots) {
    const slot = lotAtPoint(draft, t, x, z);
    if (slot !== null) return { kind: 'lot', slot };
  }
  if (what.roads) {
    const road = roadAtPoint(draft.roads, x, z);
    if (road !== null) return { kind: 'road', road };
  }
  if (what.areas) {
    const areas = draft.open?.areas ?? [];
    // (A village before the main town: their boxes are apart, but the main town's is the larger.)
    for (let a = areas.length - 1; a >= 0; a--) {
      const b = areas[a];
      if (x >= b.x && x <= b.x + b.w && z >= b.z && z <= b.z + b.d) return a === 0 ? { kind: 'town' } : { kind: 'village', area: a };
    }
  }
  return null;
}

/**
 * The place along a road a lot at (`x`, `z`) would front: the nearest road a car drives (not a
 * footpath) within `SPOT_REACH` m, how far along it, and on which side the point is. Null: none near.
 */
export function spotNear(draft: NeighbourhoodDraft, x: number, z: number): RoadSpot | null {
  const roads = draft.roads;
  let best: { road: number; dist: number } | null = null;
  for (const [road, m] of roadIndex(roads).around(x, z, SPOT_REACH)) {
    if (roads[road].kind === 'trail' || (best && best.dist <= m.dist)) continue;
    best = { road, dist: m.dist };
  }
  if (!best) return null;
  const { s, side } = nearest(roads[best.road].points, x, z);
  return { road: best.road, s, side };
}

/** What a picked thing is, kept across edits: a lot by where it stands, a road by where it starts, a village by its area. */
export function keyOf(draft: NeighbourhoodDraft, p: Picked): string | null {
  switch (p.kind) {
    case 'lot': {
      const s = draft.slots[p.slot];
      return s ? `lot:${lotKey(s)}` : null;
    }
    case 'road': {
      const r = draft.roads[p.road];
      return r ? `road:${roadStart(r)}|${r.name ?? ''}` : null;
    }
    case 'village':
      return draft.open?.areas[p.area] ? `village:${p.area}` : null;
    case 'town':
      return 'town';
  }
}

const roadStart = (r: RoadPlan) => `${Math.round(r.points[0][0] * 2) / 2},${Math.round(r.points[0][1] * 2) / 2}`;

/** The thing `key` (`keyOf`) names on `draft`, if it is still there. A road is found by where it starts, else by its name (one renamed keeps its start; one carried on keeps its name). */
export function find(draft: NeighbourhoodDraft, key: string): Picked | null {
  const [kind, rest] = [key.slice(0, key.indexOf(':') < 0 ? key.length : key.indexOf(':')), key.slice(key.indexOf(':') + 1)];
  switch (kind) {
    case 'lot': {
      const slot = slotOf(draft, rest);
      return slot >= 0 ? { kind: 'lot', slot } : null;
    }
    case 'road': {
      const [start, name] = [rest.slice(0, rest.indexOf('|')), rest.slice(rest.indexOf('|') + 1)];
      let road = draft.roads.findIndex((r) => roadStart(r) === start);
      if (road < 0 && name) road = draft.roads.findIndex((r) => r.name === name);
      return road >= 0 ? { kind: 'road', road } : null;
    }
    case 'village': {
      const area = Number(rest);
      return draft.open?.areas[area] ? { kind: 'village', area } : null;
    }
    case 'town':
      return { kind: 'town' };
  }
  return null;
}
