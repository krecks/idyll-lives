// The pieces of dry land a shaped land's water cuts it into, and where a bridge could cross between them (`landPieces.ts`).
import { describe, expect, test } from 'bun:test';
import { bridgeSites, dryPieces } from '../src/game/landPieces';

const CELL = 8;

/** A `W` × `W` world's 1 m tiles, wet within `width / 2` of each line through its middle at the angles given (degrees, from the x axis). */
function rivers(W: number, width: number, angles: readonly number[], at: [number, number] = [W / 2, W / 2]): Uint8Array {
  const wet = new Uint8Array(W * W);
  for (let z = 0; z < W; z++) {
    for (let x = 0; x < W; x++) {
      for (const deg of angles) {
        const a = (deg * Math.PI) / 180;
        // (How far the tile's middle is off the line.)
        const off = Math.abs(-(x + 0.5 - at[0]) * Math.sin(a) + (z + 0.5 - at[1]) * Math.cos(a));
        if (off < width / 2) wet[z * W + x] = 1;
      }
    }
  }
  return wet;
}

/** The 8 m cells over 1 m tiles, wet if any of their tiles is (as the generator reads a land: `macro.ts` `fillFrom`). */
function cells(wet: Uint8Array, W: number) {
  const gw = Math.ceil(W / CELL);
  const water = new Uint8Array(gw * gw);
  for (let z = 0; z < W; z++) for (let x = 0; x < W; x++) if (wet[z * W + x]) water[Math.floor(z / CELL) * gw + Math.floor(x / CELL)] = 1;
  return { gw, gd: gw, water };
}

/** The angle (degrees, 0..90) between a way and a line at `deg`. */
const offSquare = (u: readonly [number, number], deg: number) => {
  const a = (deg * Math.PI) / 180;
  // (Square to the line: no part of it along the line.)
  return (Math.asin(Math.min(1, Math.abs(u[0] * Math.cos(a) + u[1] * Math.sin(a)))) * 180) / Math.PI;
};

describe('dry pieces', () => {
  test('a river cross cuts the land into four pieces, each told apart', () => {
    const W = 480;
    const land = cells(rivers(W, 30, [0, 90]), W);
    const pieces = dryPieces(land);
    expect(pieces.sizes.length).toBe(4);
    const quarters = [
      [100, 100],
      [380, 100],
      [100, 380],
      [380, 380],
    ] as const;
    const ids = quarters.map((p) => pieces.of([p[0], p[1]]));
    expect(new Set(ids).size).toBe(4);
    expect(ids.every((id) => id >= 0)).toBe(true);
    expect(pieces.of([240, 240])).toBe(-1);
    expect(pieces.bySize.length).toBe(4);
    // A box on one quarter stands on its piece, not on another's; one across the river on none.
    const box = { x: 60, z: 60, w: 120, d: 100 };
    expect(pieces.holds(ids[0], box, 8)).toBe(true);
    expect(pieces.holds(ids[1], box, 8)).toBe(false);
    expect(pieces.holds(ids[0], { ...box, x: 180 }, 8)).toBe(false);
    // The nearest place a box fits on a piece, and on whichever piece is nearest.
    const at = pieces.nearest(ids[3], box, [0, 0, W - box.w, W - box.d]);
    expect(at).not.toBeNull();
    expect(pieces.holds(ids[3], { ...box, x: at![0], z: at![1] }, 8)).toBe(true);
    const any = pieces.nearestAny({ ...box, x: 300, z: 70 }, [0, 0, W - box.w, W - box.d]);
    expect(any?.id).toBe(ids[1]);
  });
});

describe('bridge sites', () => {
  for (const deg of [0, 30]) {
    test(`a 30 m river at ${deg}°: crossings square to it, about its width`, () => {
      const W = 400;
      const wet = rivers(W, 30, [deg]);
      const sites = bridgeSites(wet, W, W);
      expect(sites.length).toBeGreaterThan(10);
      for (const s of sites) {
        expect(offSquare(s.u, deg)).toBeLessThan(6);
        expect(s.across).toBeGreaterThan(29);
        expect(s.across).toBeLessThan(33);
        // Its banks dry, the water between them.
        expect(wet[Math.floor(s.a[1]) * W + Math.floor(s.a[0])]).toBe(0);
        expect(wet[Math.floor(s.b[1]) * W + Math.floor(s.b[0])]).toBe(0);
        const mid = [(s.a[0] + s.b[0]) / 2, (s.a[1] + s.b[1]) / 2];
        expect(wet[Math.floor(mid[1]) * W + Math.floor(mid[0])]).toBe(1);
      }
    });
  }

  test('none over a river 60 m across', () => {
    const W = 400;
    expect(bridgeSites(rivers(W, 60, [20]), W, W)).toEqual([]);
  });

  test('across a cross, between the quarters; the same every time', () => {
    const W = 480;
    const wet = rivers(W, 30, [0, 90]);
    const sites = bridgeSites(wet, W, W);
    const pieces = dryPieces(cells(wet, W));
    // (Each end 12 m inland of its bank, where a bridge's end stands.)
    const inland = (s: (typeof sites)[number]) => [pieces.of([s.a[0] - s.u[0] * 12, s.a[1] - s.u[1] * 12]), pieces.of([s.b[0] + s.u[0] * 12, s.b[1] + s.u[1] * 12])];
    expect(sites.every((s) => inland(s).every((id) => id >= 0))).toBe(true);
    const pairs = new Set(sites.map((s) => inland(s).sort().join('-')));
    // (Each quarter to the two beside it; none over the crossing itself.)
    expect(pairs.size).toBe(4);
    expect(JSON.stringify(bridgeSites(wet, W, W))).toBe(JSON.stringify(sites));
  });
});
