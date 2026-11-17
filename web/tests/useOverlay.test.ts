// Step 2's painted uses over the town (`render/pc/useOverlay.ts`): a row's cells of one use drawn as one run (a new
// one where the use or the hatching changes), and a new layer compared with the last so only the tiles round the
// changed cells are made again. On the engine's null device: no GPU.
import { afterAll, describe, expect, mock, test } from 'bun:test';
// (PlayCanvas as the renderer imports it: render/pc/engine.ts.)
import * as pc from '../src/render/pc/engine';
import { packUses, unpackUses, USE_CODE } from '../src/game/uses';

// (The renderer's modules read the page's address as they load: one put in for them, the real one back after.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (Only the front face comes from the materials, whose module wants the game's settings.)
mock.module('../src/render/pc/materials', () => ({ GAME_FRONT_FACE: pc.FRONTFACE_CW }));
const { dirtyRect, rowRuns, tilesOf, TILE, UseOverlayDraw } = await import('../src/render/pc/useOverlay');

function scene() {
  const canvas = { width: 1, height: 1, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1, height: 1 }) } as unknown as HTMLCanvasElement;
  const device = new pc.NullGraphicsDevice(canvas);
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.RenderComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const root = new pc.Entity('root', app);
  app.root.addChild(root);
  return { app, root };
}

/** A world of `cols` × `rows` cells with `paint` (column, row → code) on it. */
function layer(cols: number, rows: number, paint: (i: number, j: number) => number) {
  const codes = new Uint8Array(cols * rows);
  for (let j = 0; j < rows; j++) for (let i = 0; i < cols; i++) codes[j * cols + i] = paint(i, j);
  return packUses(codes, cols, rows);
}

const { homes, shops, green } = USE_CODE;
const flat = () => 0;

describe("a row's runs", () => {
  test('one run for each stretch of one use, none for the cells with none, split where the hatching changes', () => {
    const l = layer(12, 1, (i) => [0, homes, homes, homes, shops, shops, 0, 0, green, green, green, green][i]);
    const hatch = new Uint8Array(12);
    hatch[10] = hatch[11] = 1;
    expect(rowRuns(unpackUses(l), hatch, 12, 0, 0, 12)).toEqual([
      { i: 1, n: 3, code: homes, hatched: false },
      { i: 4, n: 2, code: shops, hatched: false },
      { i: 8, n: 2, code: green, hatched: false },
      { i: 10, n: 2, code: green, hatched: true },
    ]);
    // (Within a tile's columns only.)
    expect(rowRuns(unpackUses(l), null, 12, 0, 2, 9)).toEqual([
      { i: 2, n: 2, code: homes, hatched: false },
      { i: 4, n: 2, code: shops, hatched: false },
      { i: 8, n: 1, code: green, hatched: false },
    ]);
  });
});

describe('the changed cells', () => {
  test('the rectangle round the cells whose use or hatching changed; none when nothing did', () => {
    const before = layer(20, 10, (i, j) => (i < 10 && j < 5 ? homes : 0));
    const after = layer(20, 10, (i, j) => (i < 10 && j < 5 ? homes : i === 15 && j === 8 ? shops : 0));
    const [a, b] = [unpackUses(before), unpackUses(after)];
    expect(dirtyRect(a, a, null, null, 20)).toBeNull();
    expect(dirtyRect(a, b, null, null, 20)).toEqual({ i0: 15, j0: 8, i1: 16, j1: 9 });
    const hatch = new Uint8Array(200);
    hatch[2 * 20 + 3] = 1;
    // (Hatching where there is no use changes nothing.)
    hatch[9 * 20 + 19] = 1;
    expect(dirtyRect(a, a, null, hatch, 20)).toEqual({ i0: 3, j0: 2, i1: 4, j1: 3 });
    // (Nothing drawn before: all of it.)
    expect(dirtyRect(null, a, null, null, 20)).toEqual({ i0: 0, j0: 0, i1: 20, j1: 10 });
  });

  test('the tiles a rectangle touches', () => {
    expect(tilesOf({ i0: 0, j0: 0, i1: 1, j1: 1 }, 240, 240)).toEqual([0]);
    expect(tilesOf({ i0: TILE - 1, j0: TILE - 1, i1: TILE + 1, j1: TILE + 1 }, 240, 240)).toEqual([0, 1, 4, 5]);
    // (Grown past the grid's edges: within it.)
    expect(tilesOf({ i0: -1, j0: -1, i1: 241, j1: 2 }, 240, 240)).toEqual([0, 1, 2, 3]);
  });
});

describe('the overlay', () => {
  test("a stamp makes only its tiles again; the same layer again, none; another land or scale, every tile with a use", () => {
    const { app, root } = scene();
    const draw = new UseOverlayDraw(app, root);
    const [cols, rows] = [240, 240];
    // (Homes over the world's first two rows of tiles, shops in the far corner.)
    const base = (i: number, j: number) => (j < TILE * 2 ? homes : i > 200 && j > 200 ? shops : 0);
    draw.set(layer(cols, rows, base), null, flat, 0, 'land');
    expect(draw.remade).toBe(9);
    draw.set(layer(cols, rows, base), null, flat, 0, 'land');
    expect(draw.remade).toBe(0);
    // A stamp in the middle of a tile.
    const stamped = (i: number, j: number) => (i >= 150 && i < 154 && j >= 150 && j < 154 ? green : base(i, j));
    draw.set(layer(cols, rows, stamped), null, flat, 0, 'land');
    expect(draw.remade).toBe(1);
    // One on a tile's corner: the four round it.
    const corner = (i: number, j: number) => (i >= TILE * 3 - 1 && i <= TILE * 3 && j >= TILE * 3 - 1 && j <= TILE * 3 ? green : stamped(i, j));
    draw.set(layer(cols, rows, corner), null, flat, 0, 'land');
    expect(draw.remade).toBe(4);
    // Hatching a painted cell.
    const hatch = new Uint8Array(cols * rows);
    hatch[10 * cols + 10] = 1;
    draw.set(layer(cols, rows, corner), hatch, flat, 0, 'land');
    expect(draw.remade).toBe(1);
    // The land drawn again: every tile with a use on it.
    draw.set(layer(cols, rows, corner), hatch, flat, 0, 'land again');
    expect(draw.remade).toBe(12);
    expect(draw.setScale(2)).toBe(true);
    draw.set(layer(cols, rows, corner), hatch, flat, 0, 'land again');
    expect(draw.remade).toBe(12);
    draw.destroy();
  });

  test('drapes its cells over the land, a little over it', () => {
    const { app, root } = scene();
    const draw = new UseOverlayDraw(app, root);
    draw.set(
      layer(8, 8, (i, j) => (i === 2 && j === 3 ? homes : 0)),
      null,
      (x, z) => x + z,
      0,
      'land',
    );
    const mesh = (root.findByName('useOverlay.0') as pc.Entity).render!.meshInstances[0].mesh!;
    const pos: number[] = [];
    mesh.getPositions(pos);
    // (The cell's fill and its four edge bands: 4 vertices each; the buffer is longer, kept for later layers.)
    expect(pos.length).toBeGreaterThanOrEqual(20 * 3);
    for (let v = 0; v < 20 * 3; v += 3) {
      const [x, y, z] = [pos[v], pos[v + 1], pos[v + 2]];
      expect(x).toBeGreaterThanOrEqual(8);
      expect(x).toBeLessThanOrEqual(12);
      expect(z).toBeGreaterThanOrEqual(12);
      expect(z).toBeLessThanOrEqual(16);
      expect(y - (x + z)).toBeCloseTo(0.12, 5);
    }
    draw.destroy();
  });
});
