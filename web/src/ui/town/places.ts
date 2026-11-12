/**
 * What the town tags know of the town itself (docs/design/living-town.md, Town tags): its places (the town file's
 * `places`, else made from the public lots), whether a place is open, the homes for sale and what they cost, and
 * the street and area names. Pure: the tag layer (`TownTags.svelte`) gives it the game's views.
 */

import type { PlotInfo, RoadPlan, TownArea, TownPlace, WorldMeta } from '../../core/protocol';

type Rates = { perTile: number; perFloorTile: number; round: number; sizeRef?: number; sizePremium?: number };

/** A public lot's kind, from its name (the town file's own places say theirs). */
function kindFromName(name: string, outdoor: boolean): TownPlace['kind'] {
  const n = name.toLowerCase();
  if (/caf[eé]|coffee|tea room/.test(n)) return 'cafe';
  if (/bakery|grocer|shop|store|market hall|butcher/.test(n)) return 'shop';
  if (/restaurant|bistro|diner|pub|inn\b/.test(n)) return 'restaurant';
  if (outdoor) return 'outdoor';
  return 'park';
}

/** Who works at a public lot made into a place (a café's cooks and waiters, a shop's sellers). */
const FALLBACK_WORK: Partial<Record<TownPlace['kind'], string[]>> = {
  cafe: ['culinary', 'hospitality'],
  restaurant: ['culinary', 'hospitality'],
  shop: ['retail'],
};

/** The index of the area (`areas`) round a point, else 0 (the main town). */
export function areaAt(x: number, z: number, areas: readonly Pick<TownArea, 'x' | 'z' | 'w' | 'd'>[]): number {
  const i = areas.findIndex((a) => x >= a.x && x <= a.x + a.w && z >= a.z && z <= a.z + a.d);
  return i < 0 ? 0 : i;
}

/**
 * The town's places: the town file's own (`meta.places`, worldgen), else one for every public lot (a town from
 * before them, or an old save), its kind from its name, its tag at its entrance, open always (no hours known), and
 * a café's or shop's workers there (`FALLBACK_WORK`).
 */
export function townPlaces(meta: Pick<WorldMeta, 'places' | 'areas' | 'pads'> | null | undefined, plots: readonly PlotInfo[]): TownPlace[] {
  if (meta?.places?.length) return meta.places;
  const areas = meta?.areas ?? [];
  const outdoor = new Set(Object.keys(meta?.pads ?? {}).map(Number));
  const out: TownPlace[] = [];
  for (const p of plots) {
    if (!p.public) continue;
    const kind = kindFromName(p.name, outdoor.has(p.id));
    const [x, z] = p.entry ?? [p.x + p.w / 2, p.z + p.d / 2];
    out.push({ name: p.name, kind, x, z, plot: p.id, work: FALLBACK_WORK[kind], area: areaAt(p.x + p.w / 2, p.z + p.d / 2, areas) });
  }
  return out;
}

/** Whether a place is open at `minute` of `weekday` (0 = Monday), and till when; or when it opens next (in days from today). */
export type OpenState = { open: true; until: number | null } | { open: false; opens: { inDays: number; minute: number } | null };

/** Is a place with these `hours` open now (hours past midnight when `close` isn't after `open`)? Absent hours: always. */
export function openState(hours: TownPlace['hours'] | undefined, minute: number, weekday: number): OpenState {
  if (!hours) return { open: true, until: null };
  const on = (d: number) => ((hours.days >> (((d % 7) + 7) % 7)) & 1) === 1;
  const late = hours.close <= hours.open;
  if (!late && on(weekday) && minute >= hours.open && minute < hours.close) return { open: true, until: hours.close };
  if (late && on(weekday) && minute >= hours.open) return { open: true, until: hours.close };
  if (late && on(weekday - 1) && minute < hours.close) return { open: true, until: hours.close };
  for (let k = 0; k <= 7; k++) {
    if (!on(weekday + k)) continue;
    if (k === 0 && minute >= hours.open) continue;
    return { open: false, opens: { inDays: k, minute: hours.open } };
  }
  return { open: false, opens: null };
}

/** "open till 18:00", "closed · opens 09:00", "closed · opens Mon 09:00"; '' for a place always open. */
export function openText(state: OpenState, weekday: number, time: (minute: number) => string, days: readonly string[]): string {
  if (state.open) return state.until === null ? '' : `open till ${time(state.until)}`;
  if (!state.opens) return 'closed';
  const day = state.opens.inDays === 0 ? '' : state.opens.inDays === 1 ? 'tomorrow ' : `${days[(weekday + state.opens.inDays) % 7]} `;
  return `closed · opens ${day}${time(state.opens.minute)}`;
}

/** A home for sale: its lot, what it costs and how big it is. */
export interface HomeForSale {
  plot: number;
  /** The land and the house as built (the furniture comes on top): `game/price.ts` without the things in it. */
  price: number;
  /** Floor (m², every storey), 0 for an empty lot. */
  floor: number;
  storeys: number;
  /** The lot (m). */
  w: number;
  d: number;
}

/**
 * The homes nobody lives in (private lots no household has), priced as `game/price.ts` prices a home from what the
 * views carry: the land by its size and the house by its floor (the walls' box, every storey). The things in it
 * aren't in the views, so the price is a "from".
 */
export function homesForSale(plots: readonly PlotInfo[], households: readonly { plot: number | null }[], rates: Rates | undefined): HomeForSale[] {
  const taken = new Set(households.map((h) => h.plot));
  const out: HomeForSale[] = [];
  for (const p of plots) {
    if (p.public || taken.has(p.id)) continue;
    const storeys = p.house ? (p.storeys ?? 1) : 0;
    const floor = p.house ? (p.house[2] - p.house[0]) * (p.house[3] - p.house[1]) * storeys : 0;
    let price = 0;
    if (rates) {
      const land = rates.perTile * p.w * p.d;
      const built = rates.perFloorTile * floor * (floor / (rates.sizeRef ?? 100)) ** (rates.sizePremium ?? 0);
      const step = Math.max(1, rates.round);
      price = Math.round((land + built) / step) * step;
    }
    out.push({ plot: p.id, price, floor, storeys, w: p.w, d: p.d });
  }
  return out;
}

/** A name to put on the map: a street's (at its middle) or an area's. */
export interface NameLabel {
  key: string;
  text: string;
  x: number;
  z: number;
  /** An area (a hamlet, the main town) rather than a street. */
  area: boolean;
}

/** The point halfway along a polyline. */
function midway(points: readonly [number, number][]): [number, number] | null {
  if (!points.length) return null;
  let total = 0;
  for (let i = 1; i < points.length; i++) total += Math.hypot(points[i][0] - points[i - 1][0], points[i][1] - points[i - 1][1]);
  let left = total / 2;
  for (let i = 1; i < points.length; i++) {
    const [ax, az] = points[i - 1];
    const [bx, bz] = points[i];
    const len = Math.hypot(bx - ax, bz - az);
    if (len >= left && len > 0) return [ax + ((bx - ax) * left) / len, az + ((bz - az) * left) / len];
    left -= len;
  }
  return points[points.length - 1];
}

function polyLength(points: readonly [number, number][]): number {
  let n = 0;
  for (let i = 1; i < points.length; i++) n += Math.hypot(points[i][0] - points[i - 1][0], points[i][1] - points[i - 1][1]);
  return n;
}

/** The middle of a polygon's corners. */
function centroid(poly: readonly [number, number][]): [number, number] {
  let x = 0;
  let z = 0;
  for (const [px, pz] of poly) {
    x += px;
    z += pz;
  }
  return [x / Math.max(1, poly.length), z / Math.max(1, poly.length)];
}

/**
 * The names the map shows: each named street once (on its longest stretch, halfway along), trails and country
 * roads without a name left out, and each hamlet's (its centre's middle when the town file has one, else the middle
 * of its area). Countryside areas aren't named.
 */
export function nameLabels(meta: Pick<WorldMeta, 'roads' | 'areas' | 'centre' | 'name'> | null | undefined): NameLabel[] {
  const out: NameLabel[] = [];
  const longest = new Map<string, RoadPlan>();
  for (const r of meta?.roads ?? []) {
    if (!r.name || r.kind === 'trail') continue;
    const had = longest.get(r.name);
    if (!had || polyLength(r.points) > polyLength(had.points)) longest.set(r.name, r);
  }
  for (const [name, r] of longest) {
    const at = midway(r.points);
    if (at) out.push({ key: `street:${name}`, text: name, x: at[0], z: at[1], area: false });
  }
  (meta?.areas ?? []).forEach((a, i) => {
    if (a.kind !== 'hamlet' && i !== 0) return;
    const centre = meta?.centre?.find((c) => c.area === i);
    const [x, z] = centre ? centroid(centre.poly) : [a.x + a.w / 2, a.z + a.d / 2];
    out.push({ key: `area:${i}`, text: a.name, x, z, area: true });
  });
  return out;
}
