/**
 * The garden fences, hedges and farm field fences a town puts up, on the grid: in a new town real
 * fences (`LotFile.fences` for the simulation, `WorldStructure.fences` for the renderer) that block
 * walking, make no rooms, and that the player can take down or change in Build mode; in a town from
 * before them the same plan is only drawn (`render/shared/fences.ts` `shownFences`).
 *
 * Planned once for the whole town, whichever lot is in view, as tidy enclosures round each back
 * garden, on the plot edges:
 * - a garden's style by the same hash of its plot as the street's dressing had: a white picket
 *   fence, a clipped hedge or none; rail fences round a farm;
 * - a fenced garden is closed off behind the house: a short cross fence from each side wall of the
 *   house to the plot's side, with a gate in it, then the sides back to the back edge, and the back
 *   edge; corners meet, and a side or back shared with another plot gets one fence (the left plot's,
 *   as laid out with its street along x, or the right one's when the left has none);
 * - the cross fences stand at the house front line, or behind the inner end of a driveway (and its
 *   garage) beside the house or beside it next door, so a car's doors open clear of them; on a
 *   shared side both plots' at the same depth, so the side fence meets both;
 * - a farm's sides and back are fenced (its land is open in front of them), and its fields all
 *   round, a gate in the middle of the side towards the farmhouse;
 * - none on public places, empty lots or the countryside.
 *
 * Kept clear of what residents need: front gardens stay open, and no edge goes on a wall, across a
 * path or driveway, along a driveway's side within a metre of it (a car's doors open there), through
 * a thing or a garage, or beside a thing residents use (its tile fills the gap); every garden
 * is reached through a gate or past the end of its house.
 */

import type { Facing, Rect } from './town';
import { hash } from '../render/shared/cars';

/** A fence or gate on a grid edge, as the town puts it up (style: a `fenceStyles` id). */
export interface TownFence {
  axis: 'h' | 'v';
  x: number;
  z: number;
  kind: 'fence' | 'gate';
  style: string;
}

/** A plot as the fences see it (town tiles). */
export interface FencePlot {
  rect: Rect;
  facing: Facing;
  /** A house's walls' bounding box on the ground storey (x0, z0, x1, z1); null: no house. */
  house: [number, number, number, number] | null;
  /** A house plot's (not a public place's). */
  home: boolean;
  /** Its driveway and garage (town tiles), if it has them. */
  drive: Rect | null;
  garage?: Rect | null;
  /** Whether it gets fences of its own (an old town's player lot doesn't: the player builds there). */
  fenced: boolean;
}

export interface FenceInput {
  plots: FencePlot[];
  /** Farm fields (town tiles; `TownLayout.fields`). */
  fields: Rect[];
  /** Ground-storey wall edges and the edges in front of things (`h:x:z`, `v:x:z`): no fence goes on one. */
  walls: Set<string>;
  /** Things standing on the ground and garages: no fence through one, nor beside one residents use (`use`). */
  blockers: (Rect & { use?: boolean })[];
  /** Garden paths and driveways: no fence across one. */
  paths: Rect[];
  /** Whether a point is on a road or a trail (a gap between two gardens with one in it stays open). */
  onRoad?: (x: number, z: number) => boolean;
}

/** Gardens this far apart or nearer (m) share one fence: the gap goes to the one on its lower side. */
const NARROW_GAP = 3;
/** Fences keep this far (m) from beside another plot's driveway or garage, and start behind it (room for a car's doors). */
const BESIDE_DRIVE = 1.5;
/** No fence along a driveway's side nearer than this (m), whosever it is (a back fence beside the next plot's): a car parked on it has room to open its doors (sim-core `cars::SWING`). */
const DOOR_ROOM = 1;

/** The fence styles the town uses (content `fenceStyles` ids). */
export const TOWN_FENCE = { picket: 'picket', hedge: 'hedge', rail: 'rails' } as const;

/** As `shared/street.ts` lays a plot out: turned a quarter when its street runs along z. */
const localRect = (turned: boolean, r: Rect): Rect => (turned ? { x: r.z, z: -(r.x + r.w), w: r.d, d: r.w } : r);
const townPoint = (turned: boolean, x: number, z: number): [number, number] => (turned ? [-z, x] : [x, z]);
const overlaps = (a: Rect, b: Rect, m = 0) => a.x - m < b.x + b.w && a.x + a.w + m > b.x && a.z - m < b.z + b.d && a.z + a.d + m > b.z;

interface Laid {
  p: FencePlot;
  plot: Rect;
  turned: boolean;
  front: 1 | -1;
  edge: number;
  /** House box as laid out. */
  house: { x0: number; z0: number; x1: number; z1: number } | null;
  drive: Rect | null;
  garage: Rect | null;
  style: string | null;
}

/** Every fence and gate the town puts up (see the rules at the top). */
export function townFences(input: FenceInput): TownFence[] {
  const out = new Map<string, TownFence>();
  // Blockers by where they are (cells of 4 m).
  const cells = new Map<string, (Rect & { use?: boolean })[]>();
  for (const b of input.blockers) {
    for (let cx = Math.floor(b.x / 4); cx <= Math.floor((b.x + b.w) / 4); cx++) {
      for (let cz = Math.floor(b.z / 4); cz <= Math.floor((b.z + b.d) / 4); cz++) {
        const k = `${cx}:${cz}`;
        const list = cells.get(k);
        if (list) list.push(b);
        else cells.set(k, [b]);
      }
    }
  }
  /** Whether a thing stands across thin box `r` round an edge (on both sides of it), or beside it and residents use it. */
  const blockedNear = (r: Rect, axis: 'h' | 'v') => {
    const across = (b: Rect) => (axis === 'h' ? b.z < r.z + 0.05 && b.z + b.d > r.z + r.d - 0.05 : b.x < r.x + 0.05 && b.x + b.w > r.x + r.w - 0.05);
    for (let cx = Math.floor(r.x / 4); cx <= Math.floor((r.x + r.w) / 4); cx++) {
      for (let cz = Math.floor(r.z / 4); cz <= Math.floor((r.z + r.d) / 4); cz++) if (cells.get(`${cx}:${cz}`)?.some((b) => overlaps(r, b) && (b.use || across(b)))) return true;
    }
    return false;
  };
  /** Whether unit edge (axis, x, z) may take a fence: free, not a wall, nothing beside it, no path across it. */
  const allowed = (axis: 'h' | 'v', x: number, z: number) => {
    const key = `${axis}:${x}:${z}`;
    if (out.has(key) || input.walls.has(key)) return false;
    const thin: Rect = axis === 'h' ? { x: x + 0.05, z: z - 0.1, w: 0.9, d: 0.2 } : { x: x - 0.1, z: z + 0.05, w: 0.2, d: 0.9 };
    if (blockedNear(thin, axis)) return false;
    const across = (r: Rect) => (axis === 'h' ? r.x < x + 1 && r.x + r.w > x && r.z < z - 0.01 && r.z + r.d > z + 0.01 : r.z < z + 1 && r.z + r.d > z && r.x < x - 0.01 && r.x + r.w > x + 0.01);
    return !input.paths.some(across) && !besideDrive(axis, x, z);
  };
  /** Whether unit edge (axis, x, z) runs along a driveway's side within `DOOR_ROOM` of it: the car parked on it opens its doors there. */
  const besideDrive = (axis: 'h' | 'v', x: number, z: number) =>
    input.plots.some(({ drive: r }) => {
      if (!r) return false;
      // (Along its length: a driveway runs from the street into the plot, its longer way.)
      if (axis === 'v') return r.d >= r.w && z < r.z + r.d && z + 1 > r.z && (x <= r.x ? r.x - x : x - (r.x + r.w)) <= DOOR_ROOM;
      return r.w > r.d && x < r.x + r.w && x + 1 > r.x && (z <= r.z ? r.z - z : z - (r.z + r.d)) <= DOOR_ROOM;
    });
  const put = (axis: 'h' | 'v', x: number, z: number, style: string, kind: 'fence' | 'gate' = 'fence') => {
    if (!allowed(axis, x, z)) return false;
    out.set(`${axis}:${x}:${z}`, { axis, x, z, kind, style });
    return true;
  };
  /** The unit edges of a straight line between grid points (town tiles). */
  const edgesOf = (ax: number, az: number, bx: number, bz: number): ['h' | 'v', number, number][] => {
    [ax, az, bx, bz] = [ax, az, bx, bz].map(Math.round);
    const e: ['h' | 'v', number, number][] = [];
    if (az === bz) for (let x = Math.min(ax, bx); x < Math.max(ax, bx); x++) e.push(['h', x, az]);
    else if (ax === bx) for (let z = Math.min(az, bz); z < Math.max(az, bz); z++) e.push(['v', ax, z]);
    return e;
  };
  const line = (ax: number, az: number, bx: number, bz: number, style: string) => {
    for (const [axis, x, z] of edgesOf(ax, az, bx, bz)) put(axis, x, z, style);
  };
  /** A line with a gate in it, in the middle of what of it can stand. */
  const gated = (ax: number, az: number, bx: number, bz: number, style: string) => {
    const edges = edgesOf(ax, az, bx, bz).filter(([axis, x, z]) => allowed(axis, x, z));
    if (!edges.length) return;
    const g = Math.floor((edges.length - 1) / 2);
    edges.forEach(([axis, x, z], i) => put(axis, x, z, style, i === g ? 'gate' : 'fence'));
  };

  const fields = input.fields;
  // Gardens a narrow gap apart, with no road through it, share one fence: the plot on the gap's lower
  // side (smaller x or z) fences it in (as far as the fences go: its rectangle grows over the gap).
  const fenced = (p: FencePlot) => p.home && !!p.house;
  const grown = new Map<FencePlot, Rect>(input.plots.map((p) => [p, { ...p.rect }]));
  for (const a of input.plots) {
    if (!fenced(a)) continue;
    const r = grown.get(a)!;
    for (const b of input.plots) {
      if (b === a || !fenced(b)) continue;
      const q = b.rect;
      const along = (lo: number, hi: number, lo2: number, hi2: number) => Math.min(hi, hi2) - Math.max(lo, lo2);
      const gapX = q.x - (r.x + r.w);
      const gapZ = q.z - (r.z + r.d);
      const clear = (x0: number, z0: number, x1: number, z1: number) => {
        for (let z = Math.floor(z0); z < z1; z++) for (let x = Math.floor(x0); x < x1; x++) if (input.onRoad?.(x + 0.5, z + 0.5)) return false;
        return true;
      };
      // (Never over another plot.)
      const free = (g: Rect) => input.plots.every((o) => o === a || !overlaps(g, o.rect, -0.01));
      if (gapX > 0.01 && gapX <= NARROW_GAP && along(r.z, r.z + r.d, q.z, q.z + q.d) > 3 && clear(r.x + r.w, Math.max(r.z, q.z), q.x, Math.min(r.z + r.d, q.z + q.d)) && free({ ...r, w: q.x - r.x })) r.w = q.x - r.x;
      else if (gapZ > 0.01 && gapZ <= NARROW_GAP && along(r.x, r.x + r.w, q.x, q.x + q.w) > 3 && clear(Math.max(r.x, q.x), r.z + r.d, Math.min(r.x + r.w, q.x + q.w), q.z) && free({ ...r, d: q.z - r.z })) r.d = q.z - r.z;
    }
  }
  // Plots as the street lays them out, and who is beside whom along each street edge.
  const laid: Laid[] = input.plots.map((p) => {
    const turned = p.facing % 2 === 1;
    const own = localRect(turned, p.rect);
    const plot = localRect(turned, grown.get(p)!);
    const front: 1 | -1 = p.facing === 2 || p.facing === 1 ? 1 : -1;
    const h = p.house && localRect(turned, { x: p.house[0], z: p.house[1], w: p.house[2] - p.house[0], d: p.house[3] - p.house[1] });
    const farm = fields.some((f) => overlaps(f, p.rect, -0.01));
    const style = !p.home || !p.house || !p.fenced ? null : farm ? TOWN_FENCE.rail : hash(own.x, own.z, 30) < 0.5 ? TOWN_FENCE.picket : hash(own.x, own.z, 31) < 0.75 ? TOWN_FENCE.hedge : null;
    return {
      p,
      plot,
      turned,
      front,
      edge: front > 0 ? plot.z + plot.d : plot.z,
      house: h && { x0: h.x, z0: h.z, x1: h.x + h.w, z1: h.z + h.d },
      drive: p.drive && localRect(turned, p.drive),
      garage: p.garage ? localRect(turned, p.garage) : null,
      style,
    };
  });
  const sides = new Map<Laid, { left?: Laid; right?: Laid }>();
  const rows = new Map<string, Laid[]>();
  for (const L of laid) {
    const k = `${L.turned}|${L.edge}`;
    let row = rows.get(k);
    if (!row) rows.set(k, (row = []));
    row.push(L);
  }
  for (const row of rows.values()) {
    row.sort((a, b) => a.plot.x - b.plot.x);
    row.forEach((L, i) => {
      const left = row[i - 1] && Math.abs(row[i - 1].plot.x + row[i - 1].plot.w - L.plot.x) < 0.01 ? row[i - 1] : undefined;
      const right = row[i + 1] && Math.abs(L.plot.x + L.plot.w - row[i + 1].plot.x) < 0.01 ? row[i + 1] : undefined;
      sides.set(L, { left, right });
    });
  }
  /** How far back from the street edge a depth `z` (as laid out) is. */
  const depth = (o: Laid, z: number) => (z - o.edge) * -o.front;
  /** How far back from the street edge a house's front and back stand (m). */
  const frontDepth = (o: Laid) => (o.house ? depth(o, o.front > 0 ? o.house.z1 : o.house.z0) : 0);
  const backDepth = (o: Laid) => (o.house ? depth(o, o.front > 0 ? o.house.z0 : o.house.z1) : 0);
  /** How far back a driveway (or its garage) beside the house on side `side` (-1 left, +1 right, as laid out) reaches (0: none there). */
  const driveDepth = (o: Laid, side: -1 | 1) => {
    const h = o.house;
    let v = 0;
    for (const r of [o.drive, o.garage]) {
      if (!r || !h || !(side > 0 ? r.x >= h.x1 - 0.01 : r.x + r.w <= h.x0 + 0.01)) continue;
      v = Math.max(v, depth(o, r.z), depth(o, r.z + r.d));
    }
    return v;
  };
  /** What side `side` of a house plot needs a cross fence behind: its house front, its driveway's end. */
  const needs = (o: Laid, side: -1 | 1) => {
    const drive = driveDepth(o, side);
    return Math.max(frontDepth(o), drive ? drive + 1 : 0);
  };
  /** Where the fences on side `side` of plot L start (m back): behind what both plots there need, and behind any driveway or garage beside it outside. */
  const startOf = (L: Laid, side: -1 | 1) => {
    const other = sides.get(L)?.[side > 0 ? 'right' : 'left'];
    let v = needs(L, side);
    if (other?.p.home && other.house) v = Math.max(v, needs(other, side > 0 ? -1 : 1));
    const line = side < 0 ? L.plot.x : L.plot.x + L.plot.w;
    for (const o of laid) {
      if (o === L) continue;
      for (const t of [o.p.drive, o.p.garage]) {
        if (!t) continue;
        const r = localRect(L.turned, t);
        const off = side < 0 ? line - (r.x + r.w) : r.x - line;
        if (off < -0.01 || off > BESIDE_DRIVE || !overlaps(r, L.plot, BESIDE_DRIVE + 0.01)) continue;
        v = Math.max(v, depth(L, r.z) + 1, depth(L, r.z + r.d) + 1);
      }
    }
    return Math.min(L.plot.d, Math.ceil(v - 0.01));
  };

  for (const L of laid) {
    const { plot, front, turned, style, house } = L;
    if (!style || !house) continue;
    const farm = style === TOWN_FENCE.rail;
    const { left } = sides.get(L) ?? {};
    const vz = (v: number) => L.edge - front * v; // depth into the plot -> z
    const at = (x: number, z: number) => townPoint(turned, x, z);
    const back = backDepth(L);
    for (const side of [-1, 1] as const) {
      // A shared side is the left plot's when it has a fence of its own.
      if (side < 0 && left?.style) continue;
      const x = side < 0 ? plot.x : plot.x + plot.w;
      line(...at(x, vz(startOf(L, side))), ...at(x, vz(plot.d)), style);
    }
    line(...at(plot.x, vz(plot.d)), ...at(plot.x + plot.w, vz(plot.d)), style);
    if (farm) continue;
    // The cross fences, from the house's side walls to the plot's sides (where they meet the house).
    // Behind a house shallower than its driveway, on along its side wall's line to meet it.
    for (const side of [-1, 1] as const) {
      const v = startOf(L, side);
      if (v >= plot.d) continue;
      const [wx, px] = side < 0 ? [house.x0, plot.x] : [house.x1, plot.x + plot.w];
      if (Math.abs(wx - px) < 0.5) continue;
      if (v > back) line(...at(wx, vz(back)), ...at(wx, vz(v)), style);
      gated(...at(wx, vz(v)), ...at(px, vz(v)), style);
    }
  }

  // A farm's fields, a gate on the side towards the farmhouse.
  for (const f of fields) {
    const farm = input.plots.find((p) => p.house && overlaps(f, p.rect, -0.01));
    if (farm && !farm.fenced) continue;
    const house = farm?.house;
    const gateWest = !house || (house[0] + house[2]) / 2 < f.x + f.w / 2;
    // In the middle of that side, else as near it as something beside it allows.
    const gx = gateWest ? f.x : f.x + f.w;
    for (let k = 0; k < f.d; k++) {
      const gz = f.z + Math.floor(f.d / 2) + (k % 2 ? -1 : 1) * Math.ceil(k / 2);
      if (gz >= f.z && gz < f.z + f.d && put('v', gx, gz, TOWN_FENCE.rail, 'gate')) break;
    }
    line(f.x, f.z, f.x + f.w, f.z, TOWN_FENCE.rail);
    line(f.x, f.z + f.d, f.x + f.w, f.z + f.d, TOWN_FENCE.rail);
    line(f.x, f.z, f.x, f.z + f.d, TOWN_FENCE.rail);
    line(f.x + f.w, f.z, f.x + f.w, f.z + f.d, TOWN_FENCE.rail);
  }
  joinPockets(out, input.plots);
  return [...out.values()];
}

/** A pocket of land no bigger than this (tiles), on no plot, shut in by fences, goes to a garden beside it. */
const POCKET = 64;

/**
 * Small pockets of land on no plot that the fences of gardens round them shut in (a narrow gap
 * between gardens that couldn't go to either): each goes to a garden beside it, the fences between
 * them taken away.
 */
function joinPockets(out: Map<string, TownFence>, plots: readonly FencePlot[]): void {
  if (!out.size) return;
  let W = 0;
  let D = 0;
  for (const f of out.values()) [W, D] = [Math.max(W, f.x + 2), Math.max(D, f.z + 2)];
  const onPlot = new Uint8Array(W * D);
  for (const { rect: r } of plots) {
    for (let z = Math.max(0, Math.floor(r.z)); z < Math.min(D, Math.ceil(r.z + r.d)); z++) for (let x = Math.max(0, Math.floor(r.x)); x < Math.min(W, Math.ceil(r.x + r.w)); x++) onPlot[z * W + x] = 1;
  }
  // Fences (not gates) by edge: `h` (x, z) between tiles (x, z - 1) and (x, z); `v` (x, z) between (x - 1, z) and (x, z).
  const h = new Uint8Array(W * D);
  const v = new Uint8Array(W * D);
  for (const f of out.values()) if (f.kind === 'fence' && f.x < W && f.z < D) (f.axis === 'h' ? h : v)[f.z * W + f.x] = 1;
  /** The neighbours of tile i not behind a fence, then those behind one (each with its edge), into `free` and `fenced`. */
  const look = (i: number, each: (j: number, fenced: boolean, axis: 'h' | 'v', e: number) => void) => {
    const x = i % W;
    if (x > 0) each(i - 1, !!v[i], 'v', i);
    if (x < W - 1) each(i + 1, !!v[i + 1], 'v', i + 1);
    if (i >= W) each(i - W, !!h[i], 'h', i);
    if (i + W < W * D) each(i + W, !!h[i + W], 'h', i + W);
  };
  const label = new Int32Array(W * D).fill(-1);
  const regions: { first: number; size: number; plot: boolean }[] = [];
  const stack: number[] = [];
  for (let s = 0; s < W * D; s++) {
    if (label[s] >= 0) continue;
    const id = regions.length;
    const region = { first: s, size: 0, plot: false };
    regions.push(region);
    label[s] = id;
    stack.push(s);
    while (stack.length) {
      const i = stack.pop()!;
      region.size++;
      region.plot ||= !!onPlot[i];
      look(i, (j, fenced) => {
        if (fenced || label[j] >= 0) return;
        label[j] = id;
        stack.push(j);
      });
    }
  }
  regions.forEach((r, id) => {
    if (r.plot || r.size > POCKET) return;
    // Its tiles again, and the garden beside it: the first plot's region across one of its fences; the fences between them go.
    const tiles: number[] = [];
    const seen = new Set([r.first]);
    for (let k = 0, q = [r.first]; k < q.length; k++) {
      tiles.push(q[k]);
      look(q[k], (j) => {
        if (label[j] === id && !seen.has(j)) {
          seen.add(j);
          q.push(j);
        }
      });
    }
    let into = -1;
    for (const i of tiles) look(i, (j, fenced) => void (into < 0 && fenced && regions[label[j]].plot && (into = label[j])));
    if (into < 0) return;
    for (const i of tiles) {
      look(i, (j, fenced, axis, e) => {
        if (!fenced || label[j] !== into) return;
        out.delete(`${axis}:${e % W}:${Math.floor(e / W)}`);
      });
    }
  });
}

/** Fences as the town file gives them (`LotFile.fences`): straight runs of one style, a gate on its own. */
export function fenceRuns(fences: readonly TownFence[]): { line: [number, number, number, number]; style: string; gate?: true }[] {
  const at = new Map(fences.map((f) => [`${f.axis}:${f.x}:${f.z}`, f]));
  const done = new Set<string>();
  const out: { line: [number, number, number, number]; style: string; gate?: true }[] = [];
  const sorted = [...fences].sort((a, b) => (a.axis === b.axis ? (a.axis === 'h' ? a.z - b.z || a.x - b.x : a.x - b.x || a.z - b.z) : a.axis === 'h' ? -1 : 1));
  for (const f of sorted) {
    const key = `${f.axis}:${f.x}:${f.z}`;
    if (done.has(key)) continue;
    done.add(key);
    if (f.kind === 'gate') {
      out.push({ line: f.axis === 'h' ? [f.x, f.z, f.x + 1, f.z] : [f.x, f.z, f.x, f.z + 1], style: f.style, gate: true });
      continue;
    }
    let n = 1;
    for (;;) {
      const k = f.axis === 'h' ? `h:${f.x + n}:${f.z}` : `v:${f.x}:${f.z + n}`;
      const next = at.get(k);
      if (!next || next.kind !== 'fence' || next.style !== f.style || done.has(k)) break;
      done.add(k);
      n++;
    }
    out.push({ line: f.axis === 'h' ? [f.x, f.z, f.x + n, f.z] : [f.x, f.z, f.x, f.z + n], style: f.style });
  }
  return out;
}
