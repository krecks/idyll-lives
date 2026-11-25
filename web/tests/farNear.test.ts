import { describe, expect, test } from 'bun:test';
import { Geo } from '../src/render/shared/geometry';
import { asDrawn, HOUSE_MATERIALS, HousePlanner, houseScheme, type SilhouetteSpec } from '../src/render/shared/house';
import { WALL_HEIGHT } from '../src/render/shared/walls';
import type { WallEdge } from '../src/core/protocol';
import { realTown, STOREY, W } from './fixtures/realTown';

/**
 * A house's far versions (the street's silhouette of it, the far LOD's box, the town view's) stand for the house the
 * game draws on its lot: the same kind of roof, the colours its textured walls and roof show from far off, the
 * player's own paint, and nothing on its roof that the house has not. The town's paving far off (the town view's
 * map, `groundTiles.ts` `paintPaving`) covers the tiles the street lays near (`streetGround.ts`).
 */

const COVERINGS = [
  { id: 'paint.blue', label: 'Blue', finish: 'plaster' as const, color: '#3050A0', price: 0 },
  { id: 'brick.red', label: 'Red brick', finish: 'brick' as const, color: '#C06040', price: 0 },
];
const planner = () => {
  const p = new HousePlanner();
  p.setLooks({ coverings: COVERINGS, floors: [], doors: [], windows: [] });
  return p;
};
/** The first lot along x whose own look is `look`. */
const lotWith = (look: (s: ReturnType<typeof houseScheme>) => boolean): number => {
  for (let x = 0; x < 4000; x++) if (look(houseScheme(x, 10))) return x;
  throw new Error('no such lot');
};
const plotAt = (x: number) => ({ x, z: 10, house: [x + 3, 14, x + 12, 20] as [number, number, number, number] });
/** A geometry's triangles: their corners' positions, their normal's y and their first corner's colour (bytes). */
function triangles(geo: Geo): { p: number[][]; ny: number; colour: number[] }[] {
  const { pos, idx, col } = geo;
  const out = [];
  for (let i = 0; i < idx.length; i += 3) {
    const p = [0, 1, 2].map((k) => [pos[idx[i + k] * 3], pos[idx[i + k] * 3 + 1], pos[idx[i + k] * 3 + 2]]);
    const [u, v] = [p[1].map((c, k) => c - p[0][k]), p[2].map((c, k) => c - p[0][k])];
    // (The winding `Geo` writes: the normal is (c − a) × (b − a).)
    const n = [v[1] * u[2] - v[2] * u[1], v[2] * u[0] - v[0] * u[2], v[0] * u[1] - v[1] * u[0]];
    out.push({ p, ny: n[1] / (Math.hypot(...n) || 1), colour: [...col.subarray(idx[i] * 4, idx[i] * 4 + 3)] });
  }
  return out;
}
const bytes = (hex: string) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));

describe('a house far off', () => {
  test('a hip roof is hipped far off too: no gable end stands over its walls, and it is as many triangles as a gable', () => {
    const [hip, gable] = [lotWith((s) => s.hip), lotWith((s) => !s.hip)];
    const [far, other] = [new Geo(), new Geo()];
    planner().farHouse(far, plotAt(hip));
    planner().farHouse(other, plotAt(gable));
    const upright = (g: Geo) => triangles(g).filter((t) => Math.abs(t.ny) < 0.01 && t.p.some((p) => p[1] > WALL_HEIGHT + 0.01));
    expect(upright(far)).toHaveLength(0);
    expect(upright(other)).toHaveLength(2);
    expect(far.idx.length / 3).toBe(14);
    expect(other.idx.length / 3).toBe(14);
    // Its four slopes all rise to the ridge the silhouette's roof has.
    const top = planner().roofTop(plotAt(hip));
    const slopes = triangles(far).filter((t) => t.ny > 0.2 && t.ny < 0.99);
    expect(slopes).toHaveLength(6);
    expect(Math.max(...slopes.flatMap((t) => t.p.map((p) => p[1])))).toBeCloseTo(top, 6);
  });

  test('stands its storeys as the house does: a storey on part of the one under it a box of its own, the roofs at their own heights', () => {
    // Two storeys of 12 rows: rooms over 10 × 8 tiles on the ground, 4 × 4 of them upstairs.
    const [W, SD] = [20, 12];
    const rooms = new Array<number>(W * SD * 2).fill(0);
    for (let z = 2; z < 10; z++) for (let x = 2; x < 12; x++) rooms[z * W + x] = 1;
    for (let z = 4; z < 8; z++) for (let x = 4; x < 8; x++) rooms[(z + SD) * W + x] = 2;
    const world = { rooms, width: W, depth: SD * 2, storeyDepth: SD, storeys: 2 };
    const p = planner();
    const house = [2, 2, 12, 10] as const;
    const boxes = p.storeyBoxes(world, house, 2);
    expect(boxes).toEqual([[[2, 2, 12, 10]], [[4, 4, 8, 8]]]);
    const plot = { x: lotWith((s) => !s.hip), z: 10, house, storeys: 2, boxes };
    expect(p.farLook(plot).parts.map((q) => [q.from, q.to])).toEqual([
      [0, WALL_HEIGHT],
      [WALL_HEIGHT, 2 * WALL_HEIGHT],
    ]);
    // (Its top the upper storey's narrower roof's, under the one a box of both storeys had.)
    expect(p.roofTop(plot)).toBeLessThan(p.roofTop({ ...plot, boxes: undefined }));
    const far = new Geo();
    p.farHouse(far, plot);
    expect(far.idx.length / 3).toBe(28);
  });

  test("its walls and roof are the colours their textures show from far off, not their tints", () => {
    const x = lotWith((s) => s.brick && !s.hip);
    const s = houseScheme(x, 10);
    const far = new Geo();
    planner().farHouse(far, plotAt(x));
    const walls = triangles(far).filter((t) => Math.abs(t.ny) < 0.01);
    expect(walls[0].colour).toEqual(bytes(asDrawn(s.wall, HOUSE_MATERIALS.brick)));
    // (Brick under a near-white tint is red-brown, not cream.)
    expect(walls[0].colour[0]).toBeLessThan(130);
    expect(walls[0].colour[0] - walls[0].colour[2]).toBeGreaterThan(15);
    const roof = triangles(far).find((t) => t.ny > 0.2)!;
    expect(roof.colour).toEqual(bytes(asDrawn(s.roof, HOUSE_MATERIALS.roof)));
    const look = planner().farLook(plotAt(x));
    expect(look.wall).toBe(asDrawn(s.wall, HOUSE_MATERIALS.brick));
    expect(look.roof).toBe(asDrawn(s.roof, HOUSE_MATERIALS.roof));
  });

  test('a colour through a surface and back out of it is itself; onto a lighter surface it is darker', () => {
    expect(asDrawn('#806040', HOUSE_MATERIALS.siding, HOUSE_MATERIALS.siding)).toBe('#806040');
    const onSiding = bytes(asDrawn('#C06040', HOUSE_MATERIALS.brick, HOUSE_MATERIALS.siding));
    expect(onSiding[0]).toBeLessThan(0xc0);
    expect(asDrawn('#FFFFFF', 'no such surface')).toBe('#ffffff');
  });
});

describe("a player's painted exterior", () => {
  const x = lotWith((s) => !s.brick && !s.hip);
  const plot = plotAt(x);
  const [x0, z0, x1, z1] = plot.house;
  /** The house's walls, their outside faces in covering `cover` on the sides `painted` picks. */
  const walls = (cover: number, painted: (side: number) => boolean = () => true): WallEdge[] => {
    const out: WallEdge[] = [];
    const face = (side: number, at: 0 | 1): [number, number] | undefined => (painted(side) ? (at === 0 ? [cover, 0] : [0, cover]) : undefined);
    for (let u = x0; u < x1; u++) out.push({ axis: 'h', x: u, z: z0, faces: face(0, 0) }, { axis: 'h', x: u, z: z1, faces: face(1, 1) });
    for (let u = z0; u < z1; u++) out.push({ axis: 'v', x: x0, z: u, faces: face(2, 0) }, { axis: 'v', x: x1, z: u, faces: face(3, 1) });
    // (A wall inside, papered: no outside face.)
    out.push({ axis: 'v', x: x0 + 4, z: z0 + 1, faces: [1, 1] });
    return out;
  };
  const world = (list: WallEdge[]) => ({ walls: list, depth: 60 });

  test('is the covering most of its outside wears, and none where less than half of it is covered', () => {
    const p = planner();
    expect(p.exteriorCover(world(walls(2)), plot.house)).toBe(2);
    expect(p.exteriorCover(world(walls(1, (side) => side !== 0)), plot.house)).toBe(1);
    expect(p.exteriorCover(world(walls(2, (side) => side === 0)), plot.house)).toBe(0);
    expect(p.exteriorCover(world(walls(0)), plot.house)).toBe(0);
  });

  test('shows far off and from across the street', () => {
    const p = planner();
    const cover = p.exteriorCover(world(walls(2)), plot.house);
    const far = new Geo();
    p.farHouse(far, { ...plot, cover });
    expect(triangles(far).filter((t) => Math.abs(t.ny) < 0.01 && t.p.every((q) => q[1] <= WALL_HEIGHT + 0.01))[0].colour).toEqual(bytes(asDrawn('#C06040', HOUSE_MATERIALS.brick)));
    expect(p.farLook({ ...plot, cover }).wall).toBe(asDrawn('#C06040', HOUSE_MATERIALS.brick));
    const spec: SilhouetteSpec = { x0, z0, x1, z1, lot: { x, z: 10 }, front: [0, -1], door: null, garage: null, cover };
    const part = (plan: ReturnType<HousePlanner['silhouettes']>, name: string) => plan.parts.find((q) => q.name === name)!.geo;
    // Red brick: its walls in the street's brick, in the covering's colour (its own siding has none left but the gable ends).
    const brick = p.silhouettes([spec], []);
    expect(triangles(part(brick, 'silhouetteBrick')).some((t) => t.colour.join() === bytes('#C06040').join())).toBe(true);
    expect(triangles(part(brick, 'silhouetteWalls')).every((t) => t.p.some((q) => q[1] > WALL_HEIGHT + 0.01))).toBe(true);
    // Blue plaster: on the siding, in the colour that shows there as the paint does on plaster.
    const blue = p.silhouettes([{ ...spec, cover: 1 }], []);
    const want = bytes(asDrawn('#3050A0', HOUSE_MATERIALS.plaster, HOUSE_MATERIALS.siding)).join();
    expect(triangles(part(blue, 'silhouetteWalls')).some((t) => t.colour.join() === want)).toBe(true);
    expect(part(blue, 'silhouetteBrick').empty).toBe(true);
  });
});

describe("a neighbour's silhouette", () => {
  test('has nothing over its roof that the house on its lot has not: no chimney on any lot', () => {
    const p = planner();
    for (let x = 0; x < 60; x++) {
      const plot = plotAt(x);
      const [x0, z0, x1, z1] = plot.house;
      const plan = p.silhouettes([{ x0, z0, x1, z1, lot: { x, z: 10 }, front: [0, -1], door: null, garage: null }], []);
      const top = p.roofTop(plot);
      for (const part of plan.parts) for (let i = 1; i < part.geo.pos.length; i += 3) expect(part.geo.pos[i]).toBeLessThanOrEqual(top + 1e-6);
    }
  });
});

describe('the town’s paving far and near', () => {
  /** What a 2D canvas is asked to fill, in town metres: rectangles and outlines (even-odd), each in its colour. */
  class Recorder {
    fillStyle: string | CanvasGradient | CanvasPattern = '#000';
    strokeStyle: string | CanvasGradient | CanvasPattern = '#000';
    lineWidth = 1;
    lineCap: CanvasLineCap = 'butt';
    readonly fills: { colour: string; polys: [number, number][][] }[] = [];
    private path: [number, number][][] = [];
    fillRect(x: number, z: number, w: number, d: number): void {
      this.fills.push({ colour: String(this.fillStyle), polys: [[[x, z], [x + w, z], [x + w, z + d], [x, z + d]]] });
    }
    beginPath(): void {
      this.path = [];
    }
    moveTo(x: number, z: number): void {
      this.path.push([[x, z]]);
    }
    lineTo(x: number, z: number): void {
      this.path[this.path.length - 1].push([x, z]);
    }
    closePath(): void {}
    rect(x: number, z: number, w: number, d: number): void {
      this.path.push([[x, z], [x + w, z], [x + w, z + d], [x, z + d]]);
    }
    fill(): void {
      this.fills.push({ colour: String(this.fillStyle), polys: this.path.map((p) => [...p]) });
    }
    stroke(): void {}
  }
  /** Whether (`x`, `z`) is inside the outlines `polys`, even-odd. */
  const inside = (polys: [number, number][][], x: number, z: number) => {
    let n = 0;
    for (const poly of polys) {
      for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
        const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
        if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) n++;
      }
    }
    return n % 2 === 1;
  };

  test('cover the same tiles: what the far map paints is what the street lays near', async () => {
    Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: { addEventListener: () => {}, removeEventListener: () => {} } });
    const { paintPaving } = await import('../src/render/pc/groundTiles');
    const { streetGround } = await import('../src/render/shared/streetGround');
    const { townPaving, PAVING_LOOK } = await import('../src/render/shared/townPaving');
    const { chunkAt } = await import('../src/render/shared/chunks');
    const t = realTown();
    // (A car park turned off the grid, so its edges cross the tiles anywhere, and a yard beside it.)
    const turn = (x: number, z: number): [number, number] => [30 + (x - 30) * Math.cos(0.3) - (z - 27) * Math.sin(0.3), 27 + (x - 30) * Math.sin(0.3) + (z - 27) * Math.cos(0.3)];
    t.carParks = [{ area: 0, poly: [turn(26.1, 24.6), turn(26.1, 30.4), turn(37.9, 30.4), turn(37.9, 24.6)], sign: [25.5, 30.5, 0] }];
    t.meta!.paving!.push({ kind: 'yard', poly: [[0, 0], [0, 4], [3, 4], [3, 0]] });
    // (And a centre whose west side runs slantwise, its edge tiles laid in part.)
    t.meta!.paving!.push({ kind: 'centre', poly: [[26, 2], [39, 2], [39, 15], [30, 15], [26, 8]] });
    const g = new Recorder();
    paintPaving(g as unknown as Parameters<typeof paintPaving>[0], townPaving(t), W, STOREY);
    const colours = new Set<string>([PAVING_LOOK.stone, PAVING_LOOK.joint, PAVING_LOOK.yard, PAVING_LOOK.asphalt]);
    const far = new Uint8Array(W * STOREY);
    for (let z = 0; z < STOREY; z++) for (let x = 0; x < W; x++) if (g.fills.some((f) => colours.has(f.colour) && inside(f.polys, x + 0.5, z + 0.5))) far[z * W + x] = 1;
    // Near: every chunk's layers, as the street plans them.
    const near = new Uint8Array(W * STOREY);
    const plan = { ground: [] as { name: string; geo: Geo }[], batches: new Map() };
    streetGround(plan as unknown as Parameters<typeof streetGround>[0], t, chunkAt(0, 0), undefined);
    const layers = plan.ground.filter((l) => l.name !== 'bayLines');
    expect(layers.map((l) => l.name).sort()).toEqual(['carParks', 'townPaving', 'townYards']);
    for (const { geo } of layers) {
      for (const tri of triangles(geo)) {
        const poly = tri.p.map((q) => [q[0], q[2]] as [number, number]);
        for (let z = 0; z < STOREY; z++) for (let x = 0; x < W; x++) if (inside([poly], x + 0.5, z + 0.5)) near[z * W + x] = 1;
      }
    }
    expect(far.some((v) => v)).toBe(true);
    const differ = [...far.keys()].filter((i) => far[i] !== near[i]).map((i) => [i % W, Math.floor(i / W)]);
    expect(differ).toEqual([]);
    // And both are the mask the lawn reads.
    expect([...far]).toEqual([...townPaving(t).mask(W, STOREY)]);
    // The P sign by the car park, standing where the town file says.
    expect(plan.batches.get('model.street.parkingSign')).toEqual([[25.5, 30.5, 1, 1, 0]]);
  });
});
