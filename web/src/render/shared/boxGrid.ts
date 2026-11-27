/**
 * Boxes (x0, z0, x1, z1) bucketed on a square grid, for finding the box a point is in without testing every box: the
 * night's scans of who is up in which house (`PlayCanvasRenderer` `findDarkRooms`, `findTownUp`), every resident
 * against every house before.
 */

/** A box: x0, z0, x1, z1 (m), edges in. */
export type Box = readonly [number, number, number, number];

export class BoxGrid {
  /** Each cell's boxes, by index, in the list's order. */
  private readonly cells = new Map<number, number[]>();

  /** `cell`: the grid's square (m). */
  constructor(
    boxes: readonly Box[],
    private readonly cell = 32,
  ) {
    boxes.forEach((b, i) => {
      for (let cx = Math.floor(b[0] / cell); cx <= Math.floor(b[2] / cell); cx++) {
        for (let cz = Math.floor(b[1] / cell); cz <= Math.floor(b[3] / cell); cz++) {
          const key = this.key(cx, cz);
          const list = this.cells.get(key);
          if (list) list.push(i);
          else this.cells.set(key, [i]);
        }
      }
    });
  }

  /** The first box in the list's order that holds (`x`, `z`), edges in (as testing them all in order finds), or -1. */
  find(boxes: readonly Box[], x: number, z: number): number {
    const list = this.cells.get(this.key(Math.floor(x / this.cell), Math.floor(z / this.cell)));
    if (!list) return -1;
    for (const i of list) {
      const b = boxes[i];
      if (x >= b[0] && x <= b[2] && z >= b[1] && z <= b[3]) return i;
    }
    return -1;
  }

  private key(cx: number, cz: number): number {
    // (Cells from -2^20 to 2^20 each way: a world is a few kilometres.)
    return (cx + 0x100000) * 0x200000 + (cz + 0x100000);
  }
}
