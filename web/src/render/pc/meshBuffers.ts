/**
 * A growing vertex and index buffer for the overlays drawn over the town as it is shaped (`townGhosts.ts`,
 * `useOverlay.ts`), written again from the start for each mesh: no allocation once it has room.
 */

/** A growing vertex (position, RGBA bytes) and index buffer, written from the start again after each `reset`; `start` vertices' room to begin with. */
export class MeshBuffers {
  pos: Float32Array;
  col: Uint8Array;
  idx: Uint32Array;
  nv = 0;
  ni = 0;

  constructor(start: number) {
    this.pos = new Float32Array(3 * start);
    this.col = new Uint8Array(4 * start);
    this.idx = new Uint32Array(6 * start);
  }

  reset(): void {
    this.nv = this.ni = 0;
  }

  /** Makes room for `v` more vertices and `i` more indices. */
  reserve(v: number, i: number): void {
    if ((this.nv + v) * 3 > this.pos.length) {
      const size = Math.max(this.pos.length * 2, (this.nv + v) * 3);
      const [pos, col] = [new Float32Array(size), new Uint8Array((size / 3) * 4)];
      pos.set(this.pos);
      col.set(this.col);
      [this.pos, this.col] = [pos, col];
    }
    if (this.ni + i > this.idx.length) {
      const idx = new Uint32Array(Math.max(this.idx.length * 2, this.ni + i));
      idx.set(this.idx);
      this.idx = idx;
    }
  }

  /** A vertex at (`x`, `y`, `z`) of colour `rgb` (bytes) and `alpha` (0..1, held there); its index. */
  vertex(x: number, y: number, z: number, rgb: readonly number[], alpha: number): number {
    const v = this.nv++;
    this.pos[v * 3] = x;
    this.pos[v * 3 + 1] = y;
    this.pos[v * 3 + 2] = z;
    this.col[v * 4] = rgb[0];
    this.col[v * 4 + 1] = rgb[1];
    this.col[v * 4 + 2] = rgb[2];
    this.col[v * 4 + 3] = Math.round(Math.min(1, Math.max(0, alpha)) * 255);
    return v;
  }

  tri(a: number, b: number, c: number): void {
    const i = this.ni;
    this.idx[i] = a;
    this.idx[i + 1] = b;
    this.idx[i + 2] = c;
    this.ni += 3;
  }

  quad(a: number, b: number, c: number, d: number): void {
    this.tri(a, b, c);
    this.tri(a, c, d);
  }
}
