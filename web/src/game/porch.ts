/**
 * Build mode's Porch tool, as far as it can be worked out without the simulation (which checks
 * everything again, and that nobody is shut in: sim-core `porches.rs`): the porch over a door, the
 * porch pieces under the pointer, whether they fit, and the boxes their previews are drawn as.
 *
 * A porch stands outdoors in front of a door on the ground storey, three tiles across (the door's
 * and one either side) and two out (`porchTiles`). A piece covers whole tiles: a post one, a canopy
 * or a step platform `across` × `deep` (`PIECE_MOST`), turned to face `rot` (as a thing's: 0 towards
 * +z, 1 +x, 2 -z, 3 -x). Nothing of either may stand across a wall or a fence; pieces of a kind don't
 * overlap; a post or a platform doesn't go where the car parks, a post not on a thing.
 */

import type { PorchInfo, PorchPieceInfo, PorchPieceKind } from '../core/protocol';

/** What the Porch tool builds: a porch over a door, or a piece on its own. */
export type PorchMode = 'door' | PorchPieceKind;

/** Tiles across and deep a piece may be at most (sim-core `CANOPY_MOST`, `STEP_MOST`). */
export const PIECE_MOST: Record<PorchPieceKind, readonly [number, number]> = { post: [1, 1], canopy: [3, 6], step: [4, 4] };

/** What each piece is called (as the simulation names them). */
export const PIECE_NAME: Record<PorchPieceKind, string> = { post: 'porch post', canopy: 'porch canopy', step: 'step platform' };

/** A box to draw a preview with: `[x0, y0, z0, x1, y1, z1]`. */
export type PreviewBox = [number, number, number, number, number, number];

interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** What a porch or a piece on the home lot has to keep clear of (the lot as the view has it). */
export interface PorchSite {
  /** The home plot (its ground rows). */
  plot: Rect;
  /** Whether a tile is indoors (in a room). */
  indoors(x: number, z: number): boolean;
  /** Whether a wall or a fence stands on an edge (`h`: along x at line z; `v`: along z at line x), or a diagonal (`d`) across tile (x, z). */
  walled(axis: 'h' | 'v' | 'd', x: number, z: number): boolean;
  /** The name (lower case) of what stands on a tile, if anything does. */
  thingAt(x: number, z: number): string | undefined;
  /** Whether the garage stands on a tile. */
  garage(x: number, z: number): boolean;
  /** The home's porch pieces. */
  pieces: readonly PorchPieceInfo[];
  /** The driveway, where the car parks (null: none). */
  drive: Rect | null;
  /** Whether the household can pay for it. */
  affords(price: number): boolean;
}

/** The tile `k` across (-1, 0, 1) and `row` out (0, 1) of a porch. */
function porchTile(p: Omit<PorchInfo, 'plot'>, k: number, row: number): [number, number] {
  const out = p.out > 0;
  if (p.axis === 'v') return [out ? p.x + row : p.x - 1 - row, p.z + k];
  return [p.x + k, out ? p.z + row : p.z - 1 - row];
}

/** The tiles a porch's landing, step and canopy are over: three across, two out. */
export function porchTiles(p: Omit<PorchInfo, 'plot'>): [number, number][] {
  const out: [number, number][] = [];
  for (const row of [0, 1]) for (const k of [-1, 0, 1]) out.push(porchTile(p, k, row));
  return out;
}

/** The tiles a porch's posts stand at the edges of. */
export function porchPostTiles(p: Omit<PorchInfo, 'plot'>): [number, number][] {
  return [porchTile(p, -1, 1), porchTile(p, 1, 1)];
}

/** The edges inside a block of tiles (between each and the next across and down), as `['h' | 'v', x, z]`. */
export function innerEdges(tiles: readonly [number, number][]): ['h' | 'v', number, number][] {
  const has = new Set(tiles.map(([x, z]) => `${x}:${z}`));
  const out: ['h' | 'v', number, number][] = [];
  for (const [x, z] of tiles) {
    if (has.has(`${x + 1}:${z}`)) out.push(['v', x + 1, z]);
    if (has.has(`${x}:${z + 1}`)) out.push(['h', x, z + 1]);
  }
  return out;
}

/** The tiles a piece covers. */
export function pieceTiles(p: Pick<PorchPieceInfo, 'x' | 'z' | 'w' | 'd'>): [number, number][] {
  const out: [number, number][] = [];
  for (let z = p.z; z < p.z + p.d; z++) for (let x = p.x; x < p.x + p.w; x++) out.push([x, z]);
  return out;
}

/** Whether a piece covers tile (x, z). */
export function covers(p: Pick<PorchPieceInfo, 'x' | 'z' | 'w' | 'd'>, x: number, z: number): boolean {
  return x >= p.x && z >= p.z && x < p.x + p.w && z < p.z + p.d;
}

/** The piece of `kind`, `across` × `deep` tiles facing `rot`, whose tiles' middle is nearest the pointer at (cx, cz). */
export function piecePlan(kind: PorchPieceKind, cx: number, cz: number, across: number, deep: number, rot: number): Omit<PorchPieceInfo, 'plot'> {
  const [a, b] = kind === 'post' ? [1, 1] : [Math.max(1, Math.min(across, PIECE_MOST[kind][0])), Math.max(1, Math.min(deep, PIECE_MOST[kind][1]))];
  const [w, d] = rot % 2 === 0 ? [a, b] : [b, a];
  return { kind, x: Math.floor(cx - w / 2 + 0.5), z: Math.floor(cz - d / 2 + 0.5), w, d, rot: ((rot % 4) + 4) % 4 };
}

/** What a piece costs (`prices`: the catalog's build prices). */
export function piecePrice(kind: PorchPieceKind, tiles: number, prices: { porchPost?: number; porchCanopy?: number; porchStep?: number } | undefined): number {
  if (kind === 'post') return prices?.porchPost ?? 0;
  return (kind === 'canopy' ? (prices?.porchCanopy ?? 0) : (prices?.porchStep ?? 0)) * tiles;
}

/** Whether tile (x, z) is open ground on the home lot (on it, outdoors, no diagonal across it). */
function open(site: PorchSite, x: number, z: number): boolean {
  const p = site.plot;
  return x >= p.x && z >= p.z && x < p.x + p.w && z < p.z + p.d && !site.indoors(x, z) && !site.walled('d', x, z);
}

/**
 * The porch over door (`axis`, x, z) of the home, on its outdoor side (`door`: whether a door stands
 * on that edge): the porch (null while it has no side to stand on) and why it can't stand there
 * (a short hint; null: it can).
 */
export function porchAt(axis: 'h' | 'v', x: number, z: number, door: boolean, site: PorchSite, price: number): { porch: Omit<PorchInfo, 'plot'> | null; reason: string | null } {
  if (!door) return { porch: null, reason: 'Goes over a door' };
  const front = (out: number) => {
    const [tx, tz] = porchTile({ axis, x, z, out }, 0, 0);
    return !site.indoors(tx, tz);
  };
  const [a, b] = [front(1), front(-1)];
  if (a && b) return { porch: null, reason: 'Goes on a door into the house' };
  if (!a && !b) return { porch: null, reason: 'That door is indoors' };
  const porch = { axis, x, z, out: a ? 1 : -1 };
  const no = (reason: string) => ({ porch, reason });
  const tiles = porchTiles(porch);
  if (!tiles.every(([tx, tz]) => open(site, tx, tz))) return no('Needs open ground on your lot in front of the door');
  if (innerEdges(tiles).some(([ax, ex, ez]) => site.walled(ax, ex, ez))) return no('A wall or a fence is in the way');
  if (tiles.some(([tx, tz]) => site.garage(tx, tz))) return no('The garage is in the way');
  for (const [tx, tz] of porchPostTiles(porch)) {
    const thing = site.thingAt(tx, tz);
    if (thing) return no(`The ${thing} is in the way`);
  }
  if (!site.affords(price)) return no('Not enough money');
  return { porch, reason: null };
}

/** Whether piece `p` can go on the home lot (`site`): true, or why not (a short hint). */
export function pieceFits(p: Omit<PorchPieceInfo, 'plot'>, site: PorchSite, price: number): true | string {
  const tiles = pieceTiles(p);
  const pl = site.plot;
  if (!tiles.every(([x, z]) => x >= pl.x && z >= pl.z && x < pl.x + pl.w && z < pl.z + pl.d)) return 'Has to go on your lot';
  if (!tiles.every(([x, z]) => open(site, x, z))) return 'Goes outdoors, not in a room';
  if (innerEdges(tiles).some(([ax, ex, ez]) => site.walled(ax, ex, ez))) return 'A wall or a fence is in the way';
  if (tiles.some(([x, z]) => site.garage(x, z))) return 'The garage is in the way';
  if (site.pieces.some((q) => q.kind === p.kind && tiles.some(([x, z]) => covers(q, x, z)))) return `There's a ${PIECE_NAME[p.kind]} there already`;
  const d = site.drive;
  if (p.kind !== 'canopy' && d && tiles.some(([x, z]) => x + 1 > d.x && x < d.x + d.w && z + 1 > d.z && z < d.z + d.d)) return "That's where the car parks";
  if (p.kind === 'post') {
    const thing = site.thingAt(p.x, p.z);
    if (thing) return `The ${thing} is in the way`;
  }
  if (!site.affords(price)) return 'Not enough money';
  return true;
}

/** How high a porch's posts reach and its canopy's eaves (house.ts `PORCH_TOP`), and how deep the canopy's preview is. */
const TOP = 2.34;
const ROOF = 0.5;
/** Half a wall (house.ts `T`). */
const T = 0.07;

/** A porch's preview: its landing, its posts and its canopy (as `HouseBuilder.porch` lays them out). */
export function porchBoxes(p: Omit<PorchInfo, 'plot'>): PreviewBox[] {
  const s = p.out > 0 ? 1 : -1;
  const [line, mid] = p.axis === 'h' ? [p.z, p.x + 0.5] : [p.x, p.z + 0.5];
  const plane = line + s * T;
  const half = 0.95;
  const depth = 1.3;
  const [v0, v1] = [Math.min(plane, plane + s * depth), Math.max(plane, plane + s * depth)];
  const box = (u0: number, y0: number, a: number, u1: number, y1: number, b: number): PreviewBox => {
    const [va, vb] = [Math.min(a, b), Math.max(a, b)];
    return p.axis === 'h' ? [u0, y0, va, u1, y1, vb] : [va, y0, u0, vb, y1, u1];
  };
  const post = plane + s * (depth - 0.08);
  return [
    box(mid - half, 0, v0, mid + half, 0.06, v1),
    box(mid - half + 0.03, 0.06, post - 0.055, mid - half + 0.14, TOP, post + 0.055),
    box(mid + half - 0.14, 0.06, post - 0.055, mid + half - 0.03, TOP, post + 0.055),
    box(mid - half - 0.1, TOP, v0, mid + half + 0.1, TOP + ROOF, v1),
  ];
}

/** A piece's preview: a post's column, a canopy's slab at the porch's height, a platform's slab. */
export function pieceBoxes(p: Omit<PorchPieceInfo, 'plot'>): PreviewBox[] {
  const [x0, z0, x1, z1] = [p.x, p.z, p.x + p.w, p.z + p.d];
  if (p.kind === 'post') return [[p.x + 0.42, 0, p.z + 0.42, p.x + 0.58, TOP, p.z + 0.58]];
  if (p.kind === 'canopy') return [[x0 - 0.05, TOP, z0 - 0.05, x1 + 0.05, TOP + ROOF, z1 + 0.05]];
  return [[x0, 0, z0, x1, 0.08, z1]];
}
