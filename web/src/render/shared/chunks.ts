/**
 * The renderer's stream chunks (`docs/design/open-world.md`): the town cut into 64 m squares in
 * town coordinates, chunk (cx, cz) covering x in [cx·64, cx·64 + 64) and z in [cz·64, cz·64 + 64)
 * on the ground storey. What the street plans (`shared/street.ts`) is planned per chunk, and drawn
 * merged per draw group: a far-LOD superchunk of 2 × 2 chunks (128 m), one mesh per material, so a
 * town far off the screen costs nothing and a change rebuilds one group's meshes.
 *
 * Engine-neutral: ids, rectangles and a small grid index of rectangles (`RectGrid`).
 */

/** Side of a stream chunk (m). */
export const CHUNK = 64;
/** Side of a far-LOD superchunk, the street's draw group (m). */
export const SUPERCHUNK = 128;

/** Offset that keeps chunk coordinates of things a little outside the town (roads running out of it) positive in an id. */
const BIAS = 4096;
const SPAN = 8192;

/** The id of chunk (cx, cz): one number, stable, for maps and sets. */
export const chunkId = (cx: number, cz: number): number => (cx + BIAS) * SPAN + (cz + BIAS);
/** Chunk (cx, cz) of an id. */
export const chunkCoords = (id: number): [number, number] => [Math.floor(id / SPAN) - BIAS, (id % SPAN) - BIAS];
/** The chunk a ground point (m) is in. */
export const chunkAt = (x: number, z: number): number => chunkId(Math.floor(x / CHUNK), Math.floor(z / CHUNK));
/** The draw group (superchunk) a chunk belongs to, as a chunk id of its own grid. */
export const groupOf = (id: number): number => {
  const [cx, cz] = chunkCoords(id);
  const k = SUPERCHUNK / CHUNK;
  return chunkId(Math.floor(cx / k), Math.floor(cz / k));
};

/** How far (m) a ground point (`x`, `z`) is from the rectangle from (`x0`, `z0`), `w` by `d` m (0 on it). */
export function distanceToRect(x: number, z: number, x0: number, z0: number, w: number, d: number): number {
  return Math.hypot(Math.max(x0 - x, 0, x - (x0 + w)), Math.max(z0 - z, 0, z - (z0 + d)));
}

/** A chunk's rectangle (m). */
export function chunkRect(id: number): { x: number; z: number; w: number; d: number } {
  const [cx, cz] = chunkCoords(id);
  return { x: cx * CHUNK, z: cz * CHUNK, w: CHUNK, d: CHUNK };
}

/** The ids of the chunks a rectangle (m) touches. */
export function chunksIn(x0: number, z0: number, x1: number, z1: number): number[] {
  const out: number[] = [];
  for (let cx = Math.floor(x0 / CHUNK); cx <= Math.floor(x1 / CHUNK); cx++) for (let cz = Math.floor(z0 / CHUNK); cz <= Math.floor(z1 / CHUNK); cz++) out.push(chunkId(cx, cz));
  return out;
}

/** 32-bit FNV-1a over a string, for chunk keys. */
export function hashString(s: string, h = 0x811c9dc5): number {
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return h >>> 0;
}

/** 32-bit FNV-1a over numbers (rounded to millimetres), for what a chunk drew. */
export function hashNumbers(values: readonly number[], h = 0x811c9dc5): number {
  for (let i = 0; i < values.length; i++) h = Math.imul(h ^ Math.round(values[i] * 1000), 16777619);
  return h >>> 0;
}

/**
 * Rectangles found by where they are: each in the cells of `cell` m it touches. `near` lists those
 * that may overlap a query rectangle (with a margin), each once.
 */
export class RectGrid<T extends { x: number; z: number; w: number; d: number }> {
  private readonly cells = new Map<number, T[]>();
  private stamp = 0;
  private readonly seen = new Map<T, number>();
  constructor(private readonly cell = 4) {}

  private key(cx: number, cz: number): number {
    return (cx + BIAS * 16) * SPAN * 16 + (cz + BIAS * 16);
  }

  add(item: T): void {
    const c = this.cell;
    for (let cx = Math.floor(item.x / c); cx <= Math.floor((item.x + item.w) / c); cx++) {
      for (let cz = Math.floor(item.z / c); cz <= Math.floor((item.z + item.d) / c); cz++) {
        const k = this.key(cx, cz);
        let list = this.cells.get(k);
        if (!list) this.cells.set(k, (list = []));
        list.push(item);
      }
    }
  }

  /** The items in the cells round rectangle `r` grown by `m`, each once. */
  near(r: { x: number; z: number; w: number; d: number }, m = 0): T[] {
    const c = this.cell;
    const out: T[] = [];
    const stamp = ++this.stamp;
    for (let cx = Math.floor((r.x - m) / c); cx <= Math.floor((r.x + r.w + m) / c); cx++) {
      for (let cz = Math.floor((r.z - m) / c); cz <= Math.floor((r.z + r.d + m) / c); cz++) {
        const list = this.cells.get(this.key(cx, cz));
        if (!list) continue;
        for (const item of list) {
          if (this.seen.get(item) === stamp) continue;
          this.seen.set(item, stamp);
          out.push(item);
        }
      }
    }
    return out;
  }
}
