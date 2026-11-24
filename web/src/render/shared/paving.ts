/**
 * How the street's and the garden's hard ground is toned, by vertex (no texture, no read: the same on
 * every preset): garden paths and drives darker and mossy along their sides, so paving does not end on
 * the lawn with a ruler's edge, and a little lighter and darker along their length; the road's asphalt
 * worn pale where wheels run and dark between them and along the kerb, with a patch of newer asphalt
 * here and there (`roads.ts` lays it out). The tones are shares of the light reflected (linear), written
 * into the vertex colour (sRGB) the surface's texture is multiplied by.
 */

import { hash, smoothstep, valueNoise } from '../../game/noise';
import type { PathLook } from '../../game/pathLooks';
import { tuned } from '../tune';
import type { Geo, V3 } from './geometry';
import { terrainPieces } from './townPaving';

interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
  /** A garden path's or drive's: what it is laid in (`game/pathLooks.ts`; none: hexagonal pavers). */
  look?: string;
}

/**
 * How each look of a garden path or drive is drawn (`game/pathLooks.ts`): its material, and its colour on the town
 * view's map (sRGB, `pc/groundTiles.ts`), its texture's mean as the paving's tones leave it (`pavedRects`). An asphalt
 * drive is the road's in both.
 */
export const PATH_LOOK: Record<PathLook, { material: string; map: string }> = {
  hex: { material: 'material.path', map: '#A99C86' },
  concrete: { material: 'material.path.concrete', map: '#97938A' },
  brick: { material: 'material.path.brick', map: '#834D3D' },
  flagstone: { material: 'material.path.flagstone', map: '#938D80' },
  gravel: { material: 'material.path.gravel', map: '#A89B88' },
  asphalt: { material: 'material.road', map: '#55544F' },
};

/** The look a path is drawn in: its own, or (none, or one this game doesn't know: a save's are plain strings) the hexagonal pavers. */
export const pathLook = (look: string | undefined): PathLook => (look && Object.hasOwn(PATH_LOOK, look) ? (look as PathLook) : 'hex');

/** How high garden paths lie over the ground (m). */
const PATH_Y = 0.014;

/** Ground layer `n`'s mesh name, after `prefix` (`town`: `townPaths`; none: `n`). */
export const layerName = (prefix: string, n: string): string => (prefix ? `${prefix}${n[0].toUpperCase()}${n.slice(1)}` : n);

/**
 * Garden paths and drives as ground layers, one for each material they are laid in: its name (`paths` for the
 * hexagonal pavers, as ever, `paths.<look>` for the others, after `prefix`: `layerName`), its rectangles, its material
 * and its height.
 */
export function pathLayers<R extends Rect>(rects: readonly R[], prefix: string): [string, R[], string, number][] {
  const by = new Map<PathLook, R[]>();
  for (const r of rects) {
    const look = pathLook(r.look);
    const list = by.get(look);
    if (list) list.push(r);
    else by.set(look, [r]);
  }
  return [...by].map(([look, list]) => [layerName(prefix, look === 'hex' ? 'paths' : `paths.${look}`), list, PATH_LOOK[look].material, PATH_Y]);
}

/** Whether ground layer `name` is garden paths (`pathLayers`, after `prefix`). */
export function isPaths(name: string, prefix: string): boolean {
  const paths = layerName(prefix, 'paths');
  return name === paths || name.startsWith(`${paths}.`);
}

/** `?tune=street.…` on the dev server. */
export const STREET_LOOK = tuned('street', {
  /**
   * Paving: its tone at a side (and the colour it takes there, moss and earth), how far in that fades (m), and its
   * patches' depth. (Darker and greener, 0.38 of its light and `[0.8, 1, 0.62]` over half a metre, a narrow front
   * path was olive from side to side and read as dirty, not as worn stone.)
   */
  pathEdge: 0.55,
  pathEdgeTint: [0.9, 1, 0.78],
  pathReach: 0.35,
  pathVary: 0.1,
  /** A drive: how much darker it is where a car's wheels run. */
  driveTrack: 0.22,
  /**
   * Asphalt: the tone where wheels run (the rest is under it), between them, on the crown, along the kerb, and the
   * one tone it has where roads meet (`junction`); its patches' depth; a repair's tone and how many of a road's
   * 4 m stretches have one (none: a rectangle of darker asphalt with ruled edges read as a fault in the picture,
   * not as a repair).
   */
  wheel: 1,
  between: 0.68,
  crown: 0.8,
  gutter: 0.55,
  junction: 0.8,
  roadVary: 0.1,
  patch: 0.45,
  patches: 0,
  /** The pavement: its slabs, the kerb and the course at its back (colours), its patches' depth, and the gutter's shade. */
  walk: '#E8E4DA',
  kerb: '#A9A7A2',
  walkVary: 0.07,
  gutterShade: 0.5,
});

/** Paths at least this wide are drives (`game/town.ts` `DRIVEWAY_MIN_WIDTH`), and half the width of a wheel's track on one (m). */
const DRIVE = 1.8;
const TRACK = 0.3;
/**
 * A concrete path's slabs: about this long along it (a path's, a drive's, m), and the joint cut between two (half its
 * width, and half the width it fades over, m: wider than a real one's centimetre, which is under a pixel from the
 * play camera) and its tone. Drawn here, square to the path's run, where its texture,
 * laid by world position, could not.
 */
const SLAB = { path: 1.2, drive: 3, half: 0.01, soft: 0.03, tone: 0.45 };

/** A tone (linear), in a colour (linear multipliers), as a vertex colour's multiplier (sRGB). */
export function tint(lin: number, colour: readonly number[] = [1, 1, 1]): V3 {
  const v = (c: number) => Math.pow(Math.min(1, Math.max(0, lin * c)), 1 / 2.2);
  return [v(colour[0]), v(colour[1]), v(colour[2])];
}

/**
 * The ground garden paths are laid on where the town follows its land: a lot's pad on a lot (null: on none), else the
 * terrain as its tiles draw it (`streetGround.ts` `pathGround`).
 */
export interface PathGround {
  pad(x: number, z: number): number | null;
  terrain(x: number, z: number): number;
}

/**
 * Garden paths and drives (`rects`, `y` up) into `g`: each a grid of quads, darker and mossy at the sides
 * the lawn is on (not where another of `all` goes on), lighter and darker in patches along it. On `ground`: one
 * on a lot at its pad; one off it (across the verge to its road) cut into the terrain's triangles and laid on
 * each, so it is as flat as the lawn beside it (laid on the land between its corners, `BuiltSurface.at`, the
 * lawn came up through a drive in patches where the land creases).
 */
export function pavedRects(g: Geo, rects: readonly Rect[], all: readonly Rect[], y: number, ground?: PathGround | null): void {
  const L = STREET_LOOK;
  const paved = (x: number, z: number) => all.some((r) => x > r.x && x < r.x + r.w && z > r.z && z < r.z + r.d);
  const stations = (from: number, len: number, lo: boolean, hi: boolean): number[] => {
    const out = [0];
    // (A narrow path's middle is as far in as its sides' tone reaches: clean down its middle.)
    const edge = [0.15, Math.min(L.pathReach, len / 2)].filter((e) => e <= len / 2);
    if (lo) out.push(...edge);
    for (let s = Math.ceil(from + 0.45) - from; s < len - 0.45; s++) if (out.every((o) => Math.abs(o - s) > 0.2)) out.push(s);
    if (hi) out.push(...edge.map((e) => len - e));
    out.push(len);
    return out.sort((a, b) => a - b).filter((o, i, list) => i === 0 || o - list[i - 1] > 1e-6);
  };
  for (const r of rects) {
    // The sides the lawn is on: a side another path (or a drive) goes on from is none.
    const [x0, x1, z0, z1] = [!paved(r.x - 0.2, r.z + r.d / 2), !paved(r.x + r.w + 0.2, r.z + r.d / 2), !paved(r.x + r.w / 2, r.z - 0.2), !paved(r.x + r.w / 2, r.z + r.d + 0.2)];
    // (A path's ends meet a door or the pavement: only its long sides; a square's every side.)
    const [alongX, alongZ] = [r.w > r.d * 1.5, r.d > r.w * 1.5];
    const sides = { x0: x0 && !alongX, x1: x1 && !alongX, z0: z0 && !alongZ, z1: z1 && !alongZ };
    // A drive (a path a car fits on): darker where its wheels run, along it, 0.75 m either side of its middle.
    const drive = Math.min(r.w, r.d) >= DRIVE;
    const runsZ = r.d >= r.w;
    const [across, along] = runsZ ? [r.w, r.d] : [r.d, r.w];
    const tracks = drive ? [-0.75, 0.75].map((t) => across / 2 + t) : [];
    const marks = tracks.flatMap((t) => [t - TRACK, t, t + TRACK]).filter((m) => m > 0.05 && m < across - 0.05);
    const merge = (list: number[], extra: number[]) => [...list, ...extra.filter((m) => list.every((o) => Math.abs(o - m) > 0.08))].sort((a, b) => a - b);
    // A concrete one's joints across it, its slabs alike: what the stations along it leave room for.
    const cuts = r.look === 'concrete' ? slabJoints(along, drive ? SLAB.drive : SLAB.path) : [];
    const cut = (list: number[]) =>
      cuts.length ? [...list.filter((o) => cuts.every((j) => Math.abs(o - j) > SLAB.soft + 0.01)), ...cuts.flatMap((j) => [j - SLAB.soft, j - SLAB.half, j + SLAB.half, j + SLAB.soft])].sort((a, b) => a - b) : list;
    const [xs0, zs0] = [stations(r.x, r.w, sides.x0, sides.x1), stations(r.z, r.d, sides.z0, sides.z1)];
    const [xs, zs] = runsZ ? [merge(xs0, marks), cut(zs0)] : [cut(xs0), merge(zs0, marks)];
    const tone = (u: number, v: number): V3 => {
      const edge = Math.min(sides.x0 ? u : 9, sides.x1 ? r.w - u : 9, sides.z0 ? v : 9, sides.z1 ? r.d - v : 9);
      const e = 1 - smoothstep(0, L.pathReach, edge);
      const vary = (valueNoise((r.x + u) / 2.1, (r.z + v) / 2.1, 431) - 0.5) * 2 * L.pathVary;
      const [c, a] = runsZ ? [u, v] : [v, u];
      const wheel = Math.max(0, ...tracks.map((t) => 1 - smoothstep(0, TRACK, Math.abs(c - t))));
      let joint = 0;
      for (const j of cuts) joint = Math.max(joint, 1 - smoothstep(SLAB.half, SLAB.soft, Math.abs(a - j)));
      const lin = (1 - L.pathVary + vary) * (1 - (1 - L.pathEdge) * e) * (1 - L.driveTrack * wheel) * (1 - (1 - SLAB.tone) * joint);
      return tint(lin, L.pathEdgeTint.map((c) => 1 + (c - 1) * e));
    };
    const put = (c: readonly (readonly [number, number])[], at: (x: number, z: number) => number) =>
      g.poly(c.map(([u, v]) => [r.x + u, y + at(r.x + u, r.z + v), r.z + v] as V3), [0, 1, 0], c.map(([u, v]) => [u / r.w, v / r.d] as [number, number]), undefined, c.map(([u, v]) => tone(u, v)));
    const pad = ground ? ground.pad(r.x + r.w / 2, r.z + r.d / 2) : 0;
    for (let j = 0; j + 1 < zs.length; j++) {
      for (let i = 0; i + 1 < xs.length; i++) {
        const c: [number, number][] = [[xs[i], zs[j]], [xs[i + 1], zs[j]], [xs[i + 1], zs[j + 1]], [xs[i], zs[j + 1]]];
        if (pad !== null) put(c, () => pad);
        else if (ground) for (const part of terrainPieces(c.map(([u, v]) => [r.x + u, r.z + v]))) put(part.map(([x, z]) => [x - r.x, z - r.z]), ground.terrain);
      }
    }
  }
}

/** Where the joints go along a concrete path `len` m long, its slabs about `slab` m and all alike (none: one slab). */
function slabJoints(len: number, slab: number): number[] {
  const n = Math.max(1, Math.round(len / slab));
  return Array.from({ length: n - 1 }, (_, k) => ((k + 1) * len) / n);
}

/**
 * Where a carriageway `half` m wide either side of its middle is toned across (offsets from the middle,
 * both sides), and its tone at each: pale where wheels run, darker between them, on the crown and along
 * the kerb. A lane's wheels run 0.8 m either side of its middle; a road under 5.2 m between its kerbs is
 * one lane.
 */
export function carriageway(half: number): { at: number[]; tone: (off: number) => number } {
  const L = STREET_LOOK;
  const lanes = half >= 2.6 ? [-half / 2, half / 2] : [0];
  const marks: [number, number][] = [[-half, L.gutter], [half, L.gutter], [-half + 0.45, L.between], [half - 0.45, L.between]];
  for (const m of lanes) marks.push([m - 0.8, L.wheel], [m, L.between], [m + 0.8, L.wheel]);
  if (lanes.length === 2) marks.push([0, L.crown]);
  marks.sort((a, b) => a[0] - b[0]);
  // (Marks closer than 0.2 m are one: a narrow lane's wheels run by its kerbs.)
  const kept = marks.filter((m, i) => i === 0 || m[0] - marks[i - 1][0] > 0.2);
  const tone = (off: number): number => {
    if (off <= kept[0][0]) return kept[0][1];
    for (let i = 0; i + 1 < kept.length; i++) {
      const [a, b] = [kept[i], kept[i + 1]];
      if (off <= b[0]) return a[1] + (b[1] - a[1]) * smoothstep(a[0], b[0], off);
    }
    return kept[kept.length - 1][1];
  };
  return { at: kept.map((m) => m[0]), tone };
}

/** The asphalt's patches at a point: lighter and darker a few metres across, more along the road than across it. */
export function asphaltVary(x: number, z: number): number {
  const L = STREET_LOOK;
  return 1 - L.roadVary + (valueNoise(x / 3.7, z / 3.7, 733) - 0.5) * 2 * L.roadVary;
}

/**
 * A repair in stretch `k` of road `ri` (each 4 m of it), if it has one: where across the carriageway
 * (-1..1 of its half width), how wide (of its half width) and how long (of the stretch), and its tone.
 */
export function repair(ri: number, k: number): { across: number; wide: number; from: number; long: number; tone: number } | null {
  const L = STREET_LOOK;
  if (hash(ri * 131 + 7, k, 811) >= L.patches) return null;
  const wide = 0.3 + hash(ri, k, 812) * 0.3;
  return { across: (hash(ri, k, 813) * 2 - 1) * (0.92 - wide), wide, from: hash(ri, k, 814) * 0.3, long: 0.45 + hash(ri, k, 815) * 0.5, tone: L.patch * (0.85 + hash(ri, k, 816) * 0.3) };
}
