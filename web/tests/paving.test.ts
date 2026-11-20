// The street's and the garden's hard ground toned by vertex, engine-neutral (`render/shared/paving.ts`):
// paving darker at the sides the lawn is on and clean down a narrow path's middle, a drive darker where
// wheels run, the asphalt pale in the wheels' tracks and darkest along the kerbs, the same road always the same.
// And the town's paving (`render/shared/townPaving.ts`): what is paved is the union of its pieces, its bays'
// lines are shared between bays side by side, a car park has its P sign.
import { describe, expect, test } from 'bun:test';
import { Geo } from '../src/render/shared/geometry';
import { carriageway, pavedRects, repair, STREET_LOOK } from '../src/render/shared/paving';
import { roadMeshes } from '../src/render/shared/roads';
import { carriageway as kerbToKerb } from '../src/render/shared/junctions';
import { townPaving, type Rect } from '../src/render/shared/townPaving';
import { realTown, STOREY, W } from './fixtures/realTown';

/** The red of the vertex nearest `(x, z)` in `g` (0..1, sRGB). */
function redAt(g: Geo, x: number, z: number): number {
  let [best, at] = [Infinity, 0];
  for (let i = 0; i < g.pos.length / 3; i++) {
    const d = Math.hypot(g.pos[i * 3] - x, g.pos[i * 3 + 2] - z);
    if (d < best) [best, at] = [d, i];
  }
  return g.col[at * 4] / 255;
}

describe('paving', () => {
  test('a path is darker and greener at its sides, clean down its middle and at its ends', () => {
    const path = { x: 4, z: 2, w: 1, d: 6 };
    const g = new Geo();
    pavedRects(g, [path], [path], 0.014);
    const [side, middle] = [redAt(g, 4, 5), redAt(g, 4.5, 5)];
    expect(side).toBeLessThan(middle * 0.75);
    expect(middle).toBeGreaterThan(0.88);
    // Greener at the side: its red under its green there.
    const i = g.pos.findIndex((v, k) => k % 3 === 0 && v === 4 && g.pos[k + 2] === 5) / 3;
    expect(g.col[i * 4]).toBeLessThan(g.col[i * 4 + 1]);
    // The end, at a door or the pavement, is as its middle is.
    expect(redAt(g, 4.5, 2)).toBeGreaterThan(0.88);
    // (A byte a channel: 255 is 1.)
    for (const c of g.col) expect(c).toBeLessThanOrEqual(255);
  });

  test('a side another path goes on from is not an edge', () => {
    const [a, b] = [{ x: 0, z: 0, w: 3, d: 3 }, { x: 3, z: 0, w: 3, d: 3 }];
    const g = new Geo();
    pavedRects(g, [a], [a, b], 0.014);
    expect(redAt(g, 3, 1.5)).toBeGreaterThan(redAt(g, 0, 1.5) * 1.2);
  });

  test('a drive is darker where wheels run', () => {
    const drive = { x: 0, z: 0, w: 3, d: 6 };
    const g = new Geo();
    pavedRects(g, [drive], [drive], 0.014);
    // (Compared a metre apart along one line across it: the patches differ little there.)
    expect(redAt(g, 0.75, 3)).toBeLessThan(redAt(g, 1.5, 3));
    expect(redAt(g, 2.25, 3)).toBeLessThan(redAt(g, 1.5, 3));
  });

  test('a carriageway is pale where wheels run and darkest along its kerbs, the same on both sides', () => {
    for (const half of [1.65, 3, 4]) {
      const { at, tone } = carriageway(half);
      expect(at[0]).toBeCloseTo(-half);
      expect(at[at.length - 1]).toBeCloseTo(half);
      for (const o of at) expect(tone(o)).toBeCloseTo(tone(-o));
      expect(tone(half)).toBe(STREET_LOOK.gutter);
      const wheel = half >= 2.6 ? half / 2 + 0.8 : 0.8;
      expect(tone(wheel)).toBe(STREET_LOOK.wheel);
      for (let o = -half; o <= half; o += 0.1) expect(tone(o)).toBeGreaterThanOrEqual(STREET_LOOK.gutter - 1e-9);
    }
  });

  test('a road has the same repairs every time, in as many stretches as the look says (none now)', () => {
    let n = 0;
    for (let k = 0; k < 2000; k++) {
      const fix = repair(3, k);
      expect(repair(3, k)).toEqual(fix);
      if (!fix) continue;
      n++;
      expect(Math.abs(fix.across) + fix.wide).toBeLessThanOrEqual(0.92 + 1e-9);
    }
    expect(n / 2000).toBeGreaterThanOrEqual(STREET_LOOK.patches * 0.7);
    expect(n / 2000).toBeLessThanOrEqual(STREET_LOOK.patches * 1.3);
  });

  test('a worn road covers what the plain one does, toned between its kerbs', () => {
    const road = { points: [[0, 10], [40, 10]] as [number, number][], width: 9, ends: ['out', 'out'] as ['out', 'out'] };
    const plain = roadMeshes([road], null, 0, {}, 'surface').surface;
    const worn = roadMeshes([road], null, 0, {}, 'surface', undefined, () => 9 / 2 - 1.35).surface;
    const span = (g: Geo, k: number) => [Math.min(...g.pos.filter((_, i) => i % 3 === k)), Math.max(...g.pos.filter((_, i) => i % 3 === k))];
    expect(span(worn, 0)).toEqual(span(plain, 0));
    expect(span(worn, 2)).toEqual(span(plain, 2));
    expect(plain.col.every((c) => c === 255)).toBe(true);
    expect(worn.col.some((c) => c < 0.9 * 255)).toBe(true);
    expect(worn.idx.length).toBeGreaterThan(plain.idx.length);
  });
});

describe('the town’s paving (`townPaving`)', () => {
  /** Whether (`x`, `z`) is inside `poly` (even-odd, a ray along +x). */
  const inPoly = (poly: readonly (readonly [number, number])[], x: number, z: number) => {
    let inside = false;
    for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
      if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) inside = !inside;
    }
    return inside;
  };
  const inRect = (r: Rect, x: number, z: number) => x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d;
  /** What each piece paves, worked out tile by tile (its middle in it, not in a hole or a cut), less the tiles wholly under a carriageway. */
  const union = (world: ReturnType<typeof realTown>) => {
    const p = townPaving(world);
    const out = new Uint8Array(W * STOREY);
    const road = world.meta!.roads![0];
    const kerb = kerbToKerb(road) / 2;
    for (let z = 0; z < STOREY; z++) {
      for (let x = 0; x < W; x++) {
        const [cx, cz] = [x + 0.5, z + 0.5];
        // (The fixture's street runs along z = 34.5.)
        const under = Math.abs(cz - road.points[0][1]) < kerb - Math.SQRT1_2;
        for (const piece of p.pieces) {
          if (!inPoly(piece.poly, cx, cz) || piece.holes.some((r) => inRect(r, cx, cz)) || (piece.cuts ?? []).some((c) => inPoly(c, cx, cz))) continue;
          if (piece.edge === 'tiles' && under) continue;
          out[z * W + x] = 1;
        }
      }
    }
    return out;
  };

  test('the mask is the union of the pieces: the centre less its lots, the court on its lot, the car park', () => {
    const t = realTown();
    const p = townPaving(t);
    expect(p.pieces.map((q) => q.kind).sort()).toEqual(['carpark', 'centre', 'court', 'square']);
    expect(p.mask(W, STOREY)).toEqual(union(t));
    const at = (x: number, z: number) => p.mask(W, STOREY)[z * W + x];
    // The centre round the building's lot; the court in front of the building, on its lot; the lot behind the court is not paved.
    expect(at(30, 20)).toBe(1);
    expect(at(10, 20)).toBe(1);
    expect(at(10, 10)).toBe(0);
    // The street's middle is the road's, its edge under the pavement paved.
    expect(at(10, 34)).toBe(0);
    expect(at(10, 31)).toBe(1);
    // The same object for the same world, and a lot moved draws it again.
    expect(townPaving(t)).toBe(p);
    expect(townPaving({ ...t, plots: t.plots.map((q) => ({ ...q, x: q.x + 1 })) })).not.toBe(p);
  });

  test('a green in the centre is not paved; what the town lays replaces the centre worked out from its outline', () => {
    const t = realTown();
    t.meta!.greens = [[[28, 20], [28, 23], [31, 23], [31, 20]]];
    const p = townPaving(t);
    expect(p.mask(W, STOREY)[21 * W + 29]).toBe(0);
    expect(p.mask(W, STOREY)).toEqual(union(t));
    const laid = realTown();
    laid.meta!.paving = [...laid.meta!.paving!, { kind: 'centre', poly: [[26, 18], [26, 24], [38, 24], [38, 18]] }];
    const q = townPaving(laid);
    expect(q.pieces.filter((r) => r.kind === 'centre')).toHaveLength(1);
    expect(q.mask(W, STOREY)[20 * W + 2]).toBe(0);
    expect(q.mask(W, STOREY)[20 * W + 30]).toBe(1);
  });

  test('a slanted outline is paved along its line, not in steps of a tile: its edge tiles in part, cut where it runs', () => {
    const t = realTown();
    // (A centre whose west side runs slantwise from (26, 8) to (30, 15), clear of the lot, the square, the car park and
    // the road.)
    const poly: [number, number][] = [[26, 2], [39, 2], [39, 15], [30, 15], [26, 8]];
    t.meta!.paving = [...t.meta!.paving!, { kind: 'centre', poly }];
    const p = townPaving(t);
    const [tiles, parts] = [p.tiles(W, STOREY), p.parts(W, STOREY)];
    const area = (q: readonly (readonly [number, number])[]) => Math.abs(q.reduce((a, c, k) => a + c[0] * q[(k + 1) % q.length][1] - q[(k + 1) % q.length][0] * c[1], 0)) / 2;
    // The tiles the line crosses are laid in part, and each part's corners off its tile's are on the line.
    const off = (x: number, z: number) => Math.abs((x - 26) * 7 - (z - 8) * 4) / Math.hypot(7, 4);
    let cut = 0;
    for (let z = 8; z < 15; z++) {
      // Row by row, as much is paved west of x 31 as the outline holds there (to within a few hundredths of a m²).
      let paved = 0;
      for (let x = 24; x < 31; x++) {
        const i = z * W + x;
        if (!tiles[i]) continue;
        const q = parts.get(i);
        paved += q ? area(q[0]) : 1;
        if (!q) continue;
        cut++;
        for (const [px, pz] of q[0]) if (!((px === x || px === x + 1) && (pz === z || pz === z + 1))) expect(off(px, pz)).toBeLessThan(1e-6);
        // The lawn's mask: a tile paved at all (`any`); without, one whose middle is.
        expect(p.mask(W, STOREY, true)[i]).toBe(1);
        expect(p.mask(W, STOREY)[i]).toBe((x + 0.5 - 26) * 7 - (z + 0.5 - 8) * 4 > 0 ? 1 : 0);
      }
      const x0 = 26 + ((z + 0.5 - 8) * 4) / 7;
      expect(Math.abs(paved - (31 - x0))).toBeLessThan(0.02);
    }
    expect(cut).toBeGreaterThanOrEqual(7);
  });

  test('bays: their lines, one between two side by side, the P sign; a lone bay its own apron', () => {
    const t = realTown();
    const p = townPaving(t);
    expect(p.bays).toHaveLength(4);
    // Five lines along the four bays' sides (three shared), four across their far ends.
    const lines = p.bays.flatMap((b) => b.lines);
    expect(lines).toHaveLength(9);
    const along = lines.filter(([a, b]) => Math.abs(a[0] - b[0]) < 1e-3).map(([a]) => a[0]).sort((a, b) => a - b);
    expect(along.map((x) => Math.round(x * 100) / 100)).toEqual([26.05, 29, 32, 35, 37.95]);
    // The far ends away from the street (nose in): at z = 27.5 - 2.75.
    const across = lines.filter(([a, b]) => Math.abs(a[1] - b[1]) < 1e-3);
    expect(across).toHaveLength(4);
    for (const [a] of across) expect(a[1]).toBeCloseTo(24.75, 2);
    expect(p.signs).toEqual([[25.5, 30.5, 0]]);
    const lone = realTown();
    lone.parking = [{ x: 10, z: 36.5, yaw: 0, area: 0 }];
    lone.carParks = [];
    const q = townPaving(lone);
    expect(q.pieces.filter((r) => r.kind === 'carpark')).toHaveLength(1);
    expect(q.bays[0].lines).toHaveLength(3);
    expect(q.signs).toEqual([]);
  });
});
