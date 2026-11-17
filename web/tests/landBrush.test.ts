// The land brushes and their editor (`game/landBrush.ts`, `game/landEdit.ts`): each tool does what it says
// (raise, lower, smooth, flatten, level; a lake that sits level, the sea pushed inland, a river that never
// rises and stops where it would cut too deep, dry; the cover tools; a peak and a range placed on release);
// undo and redo give back the bytes exactly; the built land is held off and water kept off lots and roads;
// what the land makes of the world (the sim's water tiles with the bridge decks dry, the zones); the price
// of a stroke; the point brush (4 m and under: one sample, or one cover cell, wherever the pointer is
// within it, an unbroken trail when dragged); and a stamp's time (printed). `bun test tests/landBrush.test.ts`.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { blankLand, COVER, coverSpan, heightAt, LAND_CELL, NONE, SEA_CM, wetAt, type Land } from '../src/game/land';
import { aimOf, isPoint, mountainGhost, POINT, settleWater, stamp, strokeOf, type Brush, type LandTool } from '../src/game/landBrush';
import { applyLand, LandEditor, landZones, Protection, wetTiles, type BuiltLand, type LandPrices } from '../src/game/landEdit';
import { rle, unrle } from '../src/game/landMasks';
import { fbm } from '../src/game/noise';
import { inside } from '../src/game/roads';

const FREE = () => 1;
const PRICES: LandPrices = { earth: 3, mountain: 0.02, water: 2, woods: 6, meadow: 1, fields: 2, clear: 1.5 };

/** A `size` m land whose height (cm) at each sample is `fill(x, z)` (m). */
function landOf(size: number, fill: (x: number, z: number) => number = () => 0): Land {
  const land = blankLand(size, size);
  for (let j = 0; j < land.rows; j++) for (let i = 0; i < land.cols; i++) land.h[j * land.cols + i] = Math.round(fill(i * LAND_CELL, j * LAND_CELL) * 100);
  return land;
}

/** A copy of a land's bytes. */
const bytes = (land: Land) => ({ h: land.h.slice(), wl: land.wl.slice(), cover: land.cover.slice() });
const sameBytes = (land: Land, b: ReturnType<typeof bytes>) => {
  expect(Buffer.from(land.h.buffer).equals(Buffer.from(b.h.buffer))).toBe(true);
  expect(Buffer.from(land.wl.buffer).equals(Buffer.from(b.wl.buffer))).toBe(true);
  expect(Buffer.from(land.cover).equals(Buffer.from(b.cover))).toBe(true);
};
const at = (land: Land, x: number, z: number) => Math.round(z / LAND_CELL) * land.cols + Math.round(x / LAND_CELL);

/** One stroke of `brush` along `points` with the editor, `dt` s between points. */
function stroke(editor: LandEditor, brush: Brush, points: [number, number][], dt = 1 / 60) {
  editor.begin(brush, ...points[0]);
  for (const p of points.slice(1)) editor.move(...p, dt);
  return editor.end();
}
/** `n` points from `a` to `b`. */
const line = (a: [number, number], b: [number, number], n = 40): [number, number][] => Array.from({ length: n + 1 }, (_, k) => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n]);
/** Held still at `p` for `seconds`. */
const hold = (p: [number, number], seconds: number): [number, number][] => Array.from({ length: Math.round(seconds * 60) }, () => p);

describe('shape tools', () => {
  test('raise lifts the middle most and leaves the land past the radius; lower takes it down', () => {
    const land = landOf(256);
    const editor = new LandEditor(land, FREE);
    stroke(editor, { tool: 'raise', radius: 20, strength: 0.5 }, hold([128, 128], 1));
    const middle = land.h[at(land, 128, 128)];
    expect(middle).toBeGreaterThan(300);
    expect(land.h[at(land, 138, 128)]).toBeGreaterThan(0);
    expect(land.h[at(land, 138, 128)]).toBeLessThan(middle);
    expect(land.h[at(land, 150, 128)]).toBe(0);
    stroke(editor, { tool: 'lower', radius: 20, strength: 0.5 }, hold([128, 128], 2));
    expect(land.h[at(land, 128, 128)]).toBeLessThan(-200);
  });

  test('smooth evens rough land; flatten brings it to the height at the press; level takes a peak down', () => {
    const rough = landOf(256, (x, z) => fbm(x / 6, z / 6, 3, 3) * 8);
    const spread = (land: Land) => {
      let [sum, sq, n] = [0, 0, 0];
      for (let z = 110; z <= 146; z += 2) for (let x = 110; x <= 146; x += 2) [sum, sq, n] = [sum + land.h[at(land, x, z)], sq + land.h[at(land, x, z)] ** 2, n + 1];
      return Math.sqrt(sq / n - (sum / n) ** 2);
    };
    const before = spread(rough);
    stroke(new LandEditor(rough, FREE), { tool: 'smooth', radius: 30, strength: 1 }, hold([128, 128], 1));
    expect(spread(rough)).toBeLessThan(before * 0.5);

    const slope = landOf(256, (x) => x / 20);
    stroke(new LandEditor(slope, FREE), { tool: 'flatten', radius: 24, strength: 1 }, [[128, 128], ...line([128, 128], [140, 128], 20), ...hold([140, 128], 1)]);
    // (The height at the press, 6.4 m, all along.)
    for (const x of [124, 128, 134, 140]) expect(Math.abs(slope.h[at(slope, x, 128)] - 640)).toBeLessThanOrEqual(2);

    const peak = landOf(256, (x, z) => Math.max(0, 20 - Math.hypot(x - 128, z - 128) / 2));
    stroke(new LandEditor(peak, FREE), { tool: 'level', radius: 40, strength: 1 }, hold([128, 128], 2));
    expect(peak.h[at(peak, 128, 128)]).toBeLessThan(1400);
  });
});

describe('water tools', () => {
  test('a lake sits level at the ground at the press less 10 cm, its bed dug, a bank round it', () => {
    const land = landOf(256, (x) => 10 + x / 25);
    const editor = new LandEditor(land, FREE);
    const press = Math.round(heightAt(land, 128, 128) * 100);
    stroke(editor, { tool: 'lake', radius: 16, strength: 0.5 }, line([128, 128], [150, 128], 10));
    const levels = new Set<number>();
    let wet = 0;
    for (let k = 0; k < land.h.length; k++) {
      if (land.wl[k] === NONE) continue;
      levels.add(land.wl[k]);
      expect(land.wl[k]).toBeGreaterThan(land.h[k]);
      wet++;
    }
    expect(levels).toEqual(new Set([press - 10]));
    expect(wet).toBeGreaterThan(150);
    // Deepest in the middle (the sea sheet's bed: 0.22e + 0.004e², e m in).
    // (At strength 0.5, 2.25 m at most.)
    expect(land.wl[at(land, 128, 128)] - land.h[at(land, 128, 128)]).toBe(225);
    // The downhill bank stands over the water.
    expect(land.h[at(land, 128 - 18, 128)]).toBeGreaterThan(press - 10);
  });

  test('a lake begun on the sea pushes the sea inland', () => {
    const land = landOf(256, (x) => (x < 60 ? -2 : 0.5));
    for (let k = 0; k < land.h.length; k++) if (land.h[k] < SEA_CM) land.wl[k] = SEA_CM;
    const editor = new LandEditor(land, FREE);
    stroke(editor, { tool: 'lake', radius: 12, strength: 0.5 }, line([56, 128], [100, 128], 20));
    expect(wetAt(land, 45, 64)).toBe(true);
    expect(land.wl[at(land, 90, 128)]).toBe(SEA_CM);
  });

  test("a river never rises, takes the sea's level where it meets it, and stops where it would cut more than 6 m", () => {
    // Down a hill to the sea at x < 40.
    const land = landOf(512, (x) => (x < 40 ? -3 : 0.5 + (x - 40) / 30 + fbm(x / 20, 3, 5, 2) * 2));
    for (let k = 0; k < land.h.length; k++) if (land.h[k] < SEA_CM) land.wl[k] = SEA_CM;
    const editor = new LandEditor(land, FREE);
    const path = line([400, 256], [20, 260], 120);
    stroke(editor, { tool: 'river', radius: 6, strength: 0.4 }, path);
    let last = Infinity;
    for (const [x, z] of path) {
      const k = at(land, x, z);
      expect(wetAt(land, k % land.cols, Math.floor(k / land.cols))).toBe(true);
      expect(land.wl[k]).toBeLessThanOrEqual(last);
      last = land.wl[k];
    }
    expect(last).toBe(SEA_CM);
    expect(editor.hint).toBeNull();

    // Up the hill: it stops once the hill stands more than 6 m over it, and says so.
    const up = landOf(512, (x) => x / 10);
    const climb = new LandEditor(up, FREE);
    stroke(climb, { tool: 'river', radius: 6, strength: 0.4 }, line([100, 256], [300, 256], 80));
    expect(climb.hint).toContain('6 m');
    expect(up.wl[at(up, 120, 256)]).not.toBe(NONE);
    expect(up.wl[at(up, 200, 256)]).toBe(NONE);
  });

  test('dry takes the water away and fills its bed', () => {
    const land = landOf(256, () => 5);
    const editor = new LandEditor(land, FREE);
    stroke(editor, { tool: 'lake', radius: 16, strength: 0.5 }, [[128, 128]]);
    expect(wetAt(land, 64, 64)).toBe(true);
    stroke(editor, { tool: 'dry', radius: 24, strength: 1 }, [[128, 128]]);
    for (let k = 0; k < land.h.length; k++) expect(land.wl[k]).toBe(NONE);
    expect(land.h[at(land, 128, 128)]).toBeGreaterThanOrEqual(490);
  });

  test('the water settles: raised sea dries out, ground lowered by the sea fills from it', () => {
    const land = landOf(128, (x) => (x < 40 ? -2 : 0.2));
    for (let k = 0; k < land.h.length; k++) if (land.h[k] < SEA_CM) land.wl[k] = SEA_CM;
    const editor = new LandEditor(land, FREE);
    stroke(editor, { tool: 'raise', radius: 10, strength: 1 }, hold([20, 64], 1));
    expect(wetAt(land, 10, 32)).toBe(false);
    expect(land.wl[at(land, 20, 64)]).toBe(NONE);
    stroke(editor, { tool: 'lower', radius: 10, strength: 1 }, hold([46, 100], 1));
    expect(land.wl[at(land, 46, 100)]).toBe(SEA_CM);
    // (On its own, away from the sea, a hollow stays dry.)
    stroke(editor, { tool: 'lower', radius: 10, strength: 1 }, hold([100, 100], 1));
    expect(land.wl[at(land, 100, 100)]).toBe(NONE);
    expect(settleWater(land, { i0: 0, j0: 0, i1: land.cols, j1: land.rows })).toEqual({ i0: 0, j0: 0, i1: land.cols, j1: land.rows });
  });
});

describe('cover and mountains', () => {
  test('the cover tools paint 4 m cells where the falloff is over a half', () => {
    const land = landOf(256);
    const editor = new LandEditor(land, FREE);
    stroke(editor, { tool: 'woods', radius: 20, strength: 1 }, [[128, 128]]);
    const cw = coverSpan(land.cols);
    const cell = (x: number, z: number) => land.cover[Math.floor(z / 4) * cw + Math.floor(x / 4)];
    expect(cell(128, 128)).toBe(COVER.woods);
    expect(cell(138, 128)).toBe(COVER.woods);
    expect(cell(146, 128)).toBe(COVER.open);
    stroke(editor, { tool: 'fields', radius: 8, strength: 1 }, [[128, 128]]);
    expect(cell(128, 128)).toBe(COVER.fields);
    stroke(editor, { tool: 'clear', radius: 40, strength: 1 }, [[128, 128]]);
    expect(land.cover.every((c) => c === COVER.open)).toBe(true);
  });

  test('a peak and a range go up on release, with a ghost while dragging', () => {
    const land = landOf(512);
    const editor = new LandEditor(land, FREE);
    editor.begin({ tool: 'peak', radius: 60, strength: 1, height: 50 }, 200, 200);
    editor.move(256, 256, 0.1);
    const ghost = editor.ghost()!;
    expect(ghost).not.toBeNull();
    expect(Math.max(...ghost.lift)).toBeGreaterThan(25);
    expect(land.h.every((h) => h === 0)).toBe(true);
    const patch = editor.end()!;
    expect(patch).not.toBeNull();
    // (At the last point, between 0.6 and 1 of its height.)
    expect(land.h[at(land, 256, 256)]).toBeGreaterThanOrEqual(2990);
    expect(land.h[at(land, 256, 256)]).toBeLessThanOrEqual(5000);
    expect(land.h[at(land, 256 + 62, 256)]).toBe(0);

    const ridge = landOf(512);
    const e2 = new LandEditor(ridge, FREE);
    stroke(e2, { tool: 'range', radius: 40, strength: 1, height: 30 }, line([100, 256], [400, 256], 30));
    for (const x of [120, 200, 300, 380]) expect(ridge.h[at(ridge, x, 256)]).toBeGreaterThan(1500);
    expect(ridge.h[at(ridge, 250, 256 + 45)]).toBe(0);
    // (The editor strokes along a list of points in one go.)
    expect(e2.stroke({ tool: 'raise', radius: 10, strength: 1 }, [[50, 50], [60, 50], [70, 50]])).not.toBeNull();
    expect(e2.stroke({ tool: 'raise', radius: 10, strength: 1 }, [])).toBeNull();
    // (A bare stroke has a ghost too.)
    const s = strokeOf({ tool: 'range', radius: 40, strength: 1 });
    stamp(ridge, s.brush, { x: 10, z: 10, dt: 0 }, s, FREE);
    expect(mountainGhost(s)?.cols).toBeGreaterThan(10);
  });
});

describe('undo', () => {
  test('undo gives back every byte, redo puts the stroke back byte for byte, reset goes back to the survey', () => {
    const land = landOf(512, (x, z) => fbm(x / 80, z / 80, 9, 3) * 20);
    for (let k = 0; k < land.h.length; k++) if (land.h[k] < 600) land.wl[k] = 600;
    const editor = new LandEditor(land, FREE);
    const states = [bytes(land)];
    const strokes: [Brush, [number, number][]][] = [
      [{ tool: 'raise', radius: 30, strength: 0.7 }, line([100, 100], [300, 200])],
      [{ tool: 'lake', radius: 20, strength: 0.5 }, line([300, 300], [340, 320])],
      [{ tool: 'river', radius: 5, strength: 0.5 }, line([450, 50], [200, 400], 60)],
      [{ tool: 'woods', radius: 40, strength: 1 }, line([50, 400], [200, 450])],
      [{ tool: 'smooth', radius: 25, strength: 1 }, line([100, 100], [400, 400])],
      [{ tool: 'peak', radius: 60, strength: 1, height: 40 }, [[250, 250]]],
      [{ tool: 'dry', radius: 30, strength: 1 }, line([300, 300], [320, 320])],
    ];
    for (const [brush, points] of strokes) {
      stroke(editor, brush, points);
      states.push(bytes(land));
    }
    expect(land.edited).toBe(true);
    for (let n = strokes.length - 1; n >= 0; n--) {
      expect(editor.undo()).not.toBeNull();
      sameBytes(land, states[n]);
    }
    expect(editor.undo()).toBeNull();
    expect(land.edited).toBe(false);
    for (let n = 1; n <= strokes.length; n++) {
      editor.redo();
      sameBytes(land, states[n]);
    }
    expect(editor.reset()).not.toBeNull();
    sameBytes(land, states[0]);
    expect(land.edited).toBe(false);
    editor.undo();
    sameBytes(land, states[strokes.length]);
    // A new stroke drops what there was to redo.
    editor.undo();
    stroke(editor, { tool: 'raise', radius: 10, strength: 1 }, [[10, 10]]);
    expect(editor.canRedo).toBe(false);
  });

  test('it keeps the last 32 strokes, and no more than 16 MB of them', () => {
    const land = landOf(1024);
    const editor = new LandEditor(land, FREE);
    for (let n = 0; n < 40; n++) stroke(editor, { tool: 'raise', radius: 10, strength: 1 }, [[20 + n * 20, 500]]);
    expect(editor.stats().undo).toBe(32);
    // Whole-land strokes, each a megabyte of undo.
    for (let n = 0; n < 20; n++) stroke(editor, { tool: 'peak', radius: 2000, strength: 1, height: 10 }, [[512, 512]]);
    expect(editor.stats().bytes).toBeLessThanOrEqual(16 * 1024 * 1024);
    expect(editor.stats().undo).toBeLessThan(20);
  });
});

describe('the built land', () => {
  // A 256 m world: a lot at (100, 100) 20 × 30 m, a road across it at z = 200, a bridge on it from x = 40 to 80.
  const road: RoadPlan = { points: line([0, 200], [256, 200], 32), width: 7, ends: ['out', 'out'], kind: 'country', bridge: [40, 80] };
  const built: BuiltLand = { width: 256, depth: 256, plots: [{ x: 100, z: 100, w: 20, d: 30 }], roads: [road] };

  test('the brushes leave the lots and roads, and fade in clear of them', () => {
    const land = landOf(256);
    const editor = LandEditor.forWorld(land, built);
    stroke(editor, { tool: 'raise', radius: 40, strength: 1 }, hold([110, 115], 1));
    for (let z = 100; z <= 130; z += 2) for (let x = 100; x <= 120; x += 2) expect(land.h[at(land, x, z)]).toBe(0);
    expect(land.h[at(land, 110, 145)]).toBeGreaterThan(0);
    stroke(editor, { tool: 'raise', radius: 30, strength: 1 }, hold([150, 200], 1));
    expect(land.h[at(land, 150, 200)]).toBe(0);
    expect(land.h[at(land, 150, 186)]).toBeGreaterThan(0);
    // Free: 0 within 6 m of the lot, all the way 14 m off.
    const guard = new Protection(land.cols, land.rows, built);
    expect(guard.free(62, 57)).toBe(0);
    expect(guard.free(Math.round(136 / 2), Math.round(115 / 2))).toBe(1);
  });

  test("water keeps 3 m off a lot and clear of a road, and says why", () => {
    const land = landOf(256, () => 1);
    const guard = new Protection(land.cols, land.rows, built);
    const editor = new LandEditor(land, () => 1, { wettable: guard.wettable, world: built });
    stroke(editor, { tool: 'lake', radius: 20, strength: 0.5 }, line([110, 150], [110, 190], 10));
    expect(editor.hint).toContain('Water keeps');
    for (let k = 0; k < land.h.length; k++) if (wetAt(land, k % land.cols, Math.floor(k / land.cols))) expect(guard.wettable(k % land.cols, Math.floor(k / land.cols))).toBe(true);
    expect(wetAt(land, 55, 85)).toBe(true);
    // (Within 3 m of the lot's side at z = 130, and within 5.5 m of the road's middle: dry.)
    expect(guard.wettable(55, 66)).toBe(false);
    expect(guard.wettable(55, 98)).toBe(false);
    expect(guard.wettable(55, 96)).toBe(true);
  });

  test("the sim's water: the generator's, with the land's changes, the bridge's deck dry", () => {
    const land = landOf(256, () => 1);
    // The generator's water: a river under the bridge, x 50..70, all down the world.
    const base = new Uint8Array(256 * 256);
    for (let z = 0; z < 256; z++) for (let x = 50; x < 70; x++) base[z * 256 + x] = 1;
    for (let j = 0; j < land.rows; j++) for (let i = 25; i <= 35; i++) [land.h[j * land.cols + i], land.wl[j * land.cols + i]] = [-100, SEA_CM];
    const editor = new LandEditor(land, () => 1, { world: built, water: rle(base) });
    stroke(editor, { tool: 'lake', radius: 10, strength: 0.5 }, [[180, 60]]);
    const mask = unrle(editor.water(), 256 * 256);
    // The lake is wet, the river as the generator had it, the deck dry.
    expect(mask[60 * 256 + 180]).toBe(1);
    expect(mask[60 * 256 + 60]).toBe(1);
    expect(mask[100 * 256 + 51]).toBe(1);
    expect(mask[200 * 256 + 60]).toBe(0);
    const tiles = editor.waterTiles({ i0: 20, j0: 90, i1: 40, j1: 110 });
    expect(tiles.length).toBe(40 * 40);
    // (Row z = 200 is the tiles' 20th row; x = 60 their 20th column: the deck.)
    expect(tiles[20 * 40 + 20]).toBe(0);
    expect(tiles[0 * 40 + 20]).toBe(1);
    expect(editor.tileRect({ i0: 20, j0: 90, i1: 40, j1: 110 })).toEqual({ x0: 40, z0: 180, x1: 80, z1: 220 });
  });

  test("the zones: the land's own water and cover where they changed, the rest the generator's", () => {
    const land = landOf(512, () => 2);
    const hill = { kind: 'hill' as const, poly: [[0, 0], [50, 0], [50, 50]] as [number, number][] };
    const woods = { kind: 'woods' as const, poly: [[300, 300], [350, 300], [350, 350]] as [number, number][] };
    const editor = new LandEditor(land, FREE, { zones: [hill, woods] });
    stroke(editor, { tool: 'meadow', radius: 30, strength: 1 }, [[100, 100]]);
    let zones = editor.zonesOf();
    expect(zones.filter((z) => z.kind === 'meadow').some((z) => inside(z.poly, 100, 100))).toBe(true);
    // (The cover changed: the woods are the land's, of which there are none.)
    expect(zones.some((z) => z.kind === 'woods')).toBe(false);
    expect(zones.filter((z) => z.kind === 'hill')).toEqual([hill]);
    stroke(editor, { tool: 'lake', radius: 20, strength: 0.5 }, [[400, 100]]);
    zones = editor.zonesOf();
    const water = zones.filter((z) => z.kind === 'water');
    expect(water.length).toBe(1);
    expect(inside(water[0].poly, 400, 100)).toBe(true);
    expect(inside(water[0].poly, 400, 125)).toBe(false);
    // The sea at the world's edge runs on past it.
    const sea = landOf(256, (x) => (x < 30 ? -2 : 1));
    for (let k = 0; k < sea.h.length; k++) if (sea.h[k] < 0) sea.wl[k] = SEA_CM;
    const [shore] = landZones(sea, { water: true });
    expect(Math.min(...shore.poly.map((p) => p[0]))).toBeLessThan(-40);
    expect(wetTiles(sea, 256, 256)[100 * 256 + 10]).toBe(1);
  });

  test('a draft takes the land once it is shaped', () => {
    const land = landOf(256, () => 1);
    const draft = { name: 'x', width: 256, depth: 256, slots: [], roads: [], greens: [], households: [], open: { areas: [], zones: [], water: [256 * 256, 0], pads: {}, parking: [] } };
    const editor = new LandEditor(land, FREE);
    expect(applyLand(draft, editor).open!.land).toBeUndefined();
    stroke(editor, { tool: 'lake', radius: 20, strength: 0.5 }, [[128, 128]]);
    const shaped = applyLand(draft, editor);
    expect(shaped.open!.land).toBe(land);
    expect(unrle(shaped.open!.water, 256 * 256)[128 * 256 + 128]).toBe(1);
    expect(shaped.open!.zones.some((z) => z.kind === 'water')).toBe(true);
    expect(draft.open.water).toEqual([256 * 256, 0]);
  });
});

describe('cost', () => {
  test('earth by the cubic metre, water and cover by the square metre, mountains cheap, undo a refund', () => {
    const land = landOf(256);
    const editor = new LandEditor(land, FREE);
    const raised = stroke(editor, { tool: 'raise', radius: 20, strength: 1 }, hold([128, 128], 0.5))!;
    let moved = 0;
    for (let k = 0; k < land.h.length; k++) moved += land.h[k];
    const volume = (moved / 100) * 4;
    expect(editor.cost(raised, PRICES)).toBe(Math.round(volume * 3));
    const back = editor.undo()!;
    expect(editor.cost(back, PRICES)).toBe(-Math.round(volume * 3));
    expect(editor.cost(editor.redo()!, PRICES)).toBe(Math.round(volume * 3));

    const lake = stroke(editor, { tool: 'lake', radius: 10, strength: 0.5 }, [[60, 60]])!;
    let wet = 0;
    for (let k = 0; k < land.h.length; k++) if (land.wl[k] !== NONE && land.wl[k] > land.h[k]) wet++;
    expect(editor.cost(lake, PRICES)).toBe(Math.round(wet * 4 * 2));

    const woods = stroke(editor, { tool: 'woods', radius: 20, strength: 1 }, [[200, 200]])!;
    const cells = land.cover.filter((c) => c === COVER.woods).length;
    expect(editor.cost(woods, PRICES)).toBe(Math.round(cells * 16 * 6));

    // While a stroke is under way, what it has cost so far; a peak, what its ghost would.
    editor.begin({ tool: 'woods', radius: 10, strength: 1 }, 40, 200);
    editor.move(60, 200, 0.1);
    const sofar = editor.running(PRICES);
    expect(sofar).toBeGreaterThan(0);
    expect(editor.cost(editor.end()!, PRICES)).toBe(sofar);
    editor.begin({ tool: 'peak', radius: 60, strength: 1, height: 40 }, 128, 128);
    const ghost = editor.running(PRICES);
    const peak = editor.end()!;
    expect(Math.abs(editor.cost(peak, PRICES) - ghost)).toBeLessThanOrEqual(2);
    expect(editor.cost(peak, PRICES)).toBeGreaterThan(0);
    expect(editor.cost(peak, PRICES)).toBeLessThan(editor.cost(raised, PRICES));
  });
});

describe('the point brush', () => {
  /** Where the samples of `land` differ from `before`'s heights, water levels and cover (sample and cover cell indices). */
  const changes = (land: Land, before: ReturnType<typeof bytes>) => ({
    h: [...land.h.keys()].filter((k) => land.h[k] !== before.h[k] || land.wl[k] !== before.wl[k]),
    wet: [...land.wl.keys()].filter((k) => (land.wl[k] !== NONE) !== (before.wl[k] !== NONE)),
    cover: [...land.cover.keys()].filter((k) => land.cover[k] !== before.cover[k]),
  });
  /** The pointer a little way off a sample (and off a cover cell's middle), every way. */
  const OFFSETS: [number, number][] = [
    [0, 0],
    [0.3, 0],
    [-0.7, 0.4],
    [0.99, -0.99],
    [-0.49, 0.95],
    [0.6, 0.6],
  ];

  test('4 m and under is a point brush; it aims at the nearest sample, a cover tool at the cell under the pointer', () => {
    expect(isPoint({ radius: POINT / 2 })).toBe(true);
    expect(isPoint({ radius: POINT / 2 + 0.01 })).toBe(false);
    expect(aimOf({ tool: 'raise', radius: 0.5, strength: 1 }, 101, 98.9)).toEqual([102, 98]);
    expect(aimOf({ tool: 'woods', radius: 2, strength: 1 }, 101, 99.9)).toEqual([102, 98]);
    expect(aimOf({ tool: 'woods', radius: 2, strength: 1 }, 100.1, 100.1)).toEqual([102, 102]);
    expect(aimOf({ tool: 'raise', radius: 3, strength: 1 }, 101, 98.9)).toEqual([101, 98.9]);
  });

  test('a shape tool at 1 to 4 m changes exactly the sample nearest the pointer, at full strength, wherever it is pointed', () => {
    const rough = (x: number, z: number) => fbm(x / 6, z / 6, 3, 3) * 4;
    for (const tool of ['raise', 'lower', 'smooth', 'flatten'] as LandTool[]) {
      for (const size of [1, 2, 3, 4]) {
        for (const [dx, dz] of OFFSETS) {
          const land = landOf(128, rough);
          // (Flatten pulls towards the height at the press: pressed beside, then dragged on.)
          const press: [number, number] = tool === 'flatten' ? [60 + dx, 64 + dz] : [64 + dx, 64 + dz];
          const editor = new LandEditor(land, FREE);
          editor.begin({ tool, radius: size / 2, strength: 1 }, ...press);
          const before = bytes(land);
          const target = [64 + dx, 64 + dz] as const;
          editor.move(...target, 0.05);
          editor.move(...target, 0.05);
          editor.end();
          const changed = changes(land, before).h;
          if (tool === 'flatten') {
            // (Each sample on the way to it changed, each brought to the height pressed: 60, 62, 64.)
            expect(changed).toEqual([at(land, 62, 64), at(land, 64, 64)]);
            continue;
          }
          expect(changed).toEqual([at(land, 64, 64)]);
        }
      }
    }
    // Raised at full strength for a second: 8 m, all of it on that one sample.
    const land = landOf(128);
    stroke(new LandEditor(land, FREE), { tool: 'raise', radius: 0.5, strength: 1 }, hold([64.6, 63.5], 1));
    expect(land.h[at(land, 64, 64)]).toBeGreaterThan(700);
    expect(land.h.filter((v) => v !== 0).length).toBe(1);
  });

  test('a 2 m lake is one sample of water (its bank round it); dry at 2 m takes one away', () => {
    for (const [dx, dz] of OFFSETS) {
      const land = landOf(128, () => 3);
      const before = bytes(land);
      stroke(new LandEditor(land, FREE), { tool: 'lake', radius: 1, strength: 0.5 }, [[64 + dx, 64 + dz]]);
      const { h, wet } = changes(land, before);
      expect(wet).toEqual([at(land, 64, 64)]);
      expect(land.wl[at(land, 64, 64)]).toBe(290);
      expect(land.h[at(land, 64, 64)]).toBeLessThan(290);
      // (Its bank: the eight samples round it, raised to the level and a little over.)
      expect(h.length).toBe(9);
      for (const k of h) if (k !== at(land, 64, 64)) expect(land.h[k]).toBe(315);
    }
    const land = landOf(128, () => 3);
    const editor = new LandEditor(land, FREE);
    stroke(editor, { tool: 'lake', radius: 12, strength: 0.5 }, [[64, 64]]);
    const before = bytes(land);
    stroke(editor, { tool: 'dry', radius: 1, strength: 1 }, [[63.2, 64.9]]);
    expect(changes(land, before).wet).toEqual([at(land, 64, 64)]);
  });

  test('cover at 4 m paints the one 4 m cell under the pointer; a little over 4 m never paints nothing', () => {
    const cw = coverSpan(64);
    for (const [dx, dz] of [...OFFSETS, [1.9, 1.9], [-1.9, -1.9]] as [number, number][]) {
      const land = landOf(128);
      const before = bytes(land);
      // (The cell from 64 to 68 m, its middle at 66.)
      stroke(new LandEditor(land, FREE), { tool: 'woods', radius: 2, strength: 1 }, [[66 + dx, 66 + dz]]);
      expect(changes(land, before).cover).toEqual([16 * cw + 16]);
      expect(land.cover[16 * cw + 16]).toBe(COVER.woods);
    }
    // 5 m across, pointed at a cell's corner (no cell's middle within reach of a half): the cell under the pointer still.
    const land = landOf(128);
    stroke(new LandEditor(land, FREE), { tool: 'meadow', radius: 2.5, strength: 1 }, [[64.1, 64.1]]);
    expect(land.cover[16 * cw + 16]).toBe(COVER.meadow);
  });

  test('dragged, it stamps each sample it crosses once, side by side so the trail is unbroken; held still, it goes on', () => {
    const land = landOf(128);
    const editor = new LandEditor(land, FREE);
    // A diagonal, a frame a long way at a time.
    editor.begin({ tool: 'lake', radius: 1, strength: 0.5 }, 20.3, 20.4);
    editor.move(60.2, 40.1, 1 / 60);
    editor.move(100.9, 61, 1 / 60);
    const s = editor.current!;
    // (From sample (10, 10) to (50, 31): 40 across and 21 along, one stamp each, after the press.)
    expect(s.stamps).toBe(1 + 40 + 21);
    const points = s.path.map(([x, z]) => `${x},${z}`);
    expect(new Set(points).size).toBe(points.length);
    for (let k = 1; k < s.path.length; k++) expect(Math.abs(s.path[k][0] - s.path[k - 1][0]) + Math.abs(s.path[k][1] - s.path[k - 1][1])).toBe(LAND_CELL);
    editor.end();
    // (The water joins up side to side all the way.)
    const wet = new Set([...land.wl.keys()].filter((k) => land.wl[k] !== NONE));
    expect(wet.size).toBe(62);
    const seen = new Set([at(land, 20, 20)]);
    for (const k of seen) for (const n of [k - 1, k + 1, k - land.cols, k + land.cols]) if (wet.has(n) && !seen.has(n)) seen.add(n);
    expect(seen.size).toBe(62);

    // A cover point brush steps cell to cell.
    const covered = landOf(128);
    const e2 = new LandEditor(covered, FREE);
    e2.begin({ tool: 'woods', radius: 2, strength: 1 }, 10, 10);
    e2.move(50, 10, 1 / 60);
    expect(e2.current!.stamps).toBe(1 + 10);
    e2.end();
    expect(covered.cover.filter((c) => c === COVER.woods).length).toBe(11);

    // Held still, a raise keeps raising the one sample.
    const held = landOf(128);
    const e3 = new LandEditor(held, FREE);
    e3.begin({ tool: 'raise', radius: 0.5, strength: 0 }, 64, 64);
    for (let k = 0; k < 30; k++) e3.move(64.4, 63.8, 1 / 30);
    e3.end();
    expect(held.h[at(held, 64, 64)]).toBeGreaterThan(40);
    expect(held.h.filter((v) => v !== 0).length).toBe(1);
  });

  test('undo gives back every byte of point strokes', () => {
    const land = landOf(256, (x, z) => fbm(x / 40, z / 40, 9, 3) * 10);
    for (let k = 0; k < land.h.length; k++) if (land.h[k] < 300) land.wl[k] = 300;
    const editor = new LandEditor(land, FREE);
    const states = [bytes(land)];
    const strokes: [Brush, [number, number][]][] = [
      [{ tool: 'raise', radius: 0.5, strength: 1 }, line([30.3, 40.7], [90.1, 70.2], 30)],
      [{ tool: 'lower', radius: 2, strength: 0.4 }, line([50.5, 50.5], [52, 90], 10)],
      [{ tool: 'smooth', radius: 1.5, strength: 1 }, line([30, 30], [120, 120], 20)],
      [{ tool: 'lake', radius: 1, strength: 0.5 }, line([140.4, 140.6], [180.3, 150.9], 12)],
      [{ tool: 'river', radius: 2, strength: 0.5 }, line([200, 40], [120, 200], 50)],
      [{ tool: 'dry', radius: 1, strength: 1 }, line([140.4, 140.6], [160, 146], 8)],
      [{ tool: 'fields', radius: 2, strength: 1 }, line([10, 200], [90, 230], 20)],
    ];
    for (const [brush, points] of strokes) {
      stroke(editor, brush, points);
      states.push(bytes(land));
    }
    for (let n = strokes.length - 1; n >= 0; n--) {
      editor.undo();
      sameBytes(land, states[n]);
    }
    for (let n = 1; n <= strokes.length; n++) {
      editor.redo();
      sameBytes(land, states[n]);
    }
  });
});

describe('time', () => {
  test('a 30 m brush stamps on a 1024 m land in under 2 ms', () => {
    const land = landOf(1024, (x, z) => fbm(x / 120, z / 120, 4, 4) * 30);
    for (let k = 0; k < land.h.length; k++) if (land.h[k] < 800) land.wl[k] = 800;
    const editor = new LandEditor(land, FREE);
    const times: Record<string, number> = {};
    for (const tool of ['raise', 'smooth', 'flatten', 'level', 'lake', 'river', 'dry', 'woods'] as const) {
      // Each stamp on its own (`stamp`, with the water settling), then a stroke's end through the editor.
      const brush = { tool, radius: 30, strength: 0.6 };
      const s = strokeOf(brush, land);
      const n = 100;
      const t0 = performance.now();
      for (let k = 0; k < n; k++) stamp(land, brush, { x: 200 + k * 6, z: 200 + k * 5, dt: 1 / 60 }, s, FREE);
      times[tool] = Math.round(((performance.now() - t0) / n) * 1000) / 1000;
      editor.begin(brush, 200, 600);
      for (let k = 1; k <= n; k++) editor.move(200 + k * 6, 600 - k * 3, 1 / 60);
      const t1 = performance.now();
      editor.end();
      times[`${tool} end`] = Math.round((performance.now() - t1) * 100) / 100;
    }
    const t2 = performance.now();
    editor.undo();
    times.undo = Math.round((performance.now() - t2) * 100) / 100;
    const t3 = performance.now();
    editor.zonesOf();
    times.zones = Math.round(performance.now() - t3);
    const t4 = performance.now();
    LandEditor.forWorld(land, { width: 1024, depth: 1024, plots: [], roads: [] });
    times.editor = Math.round(performance.now() - t4);
    console.log('[land brush] ms (a 30 m radius stamp; a 600 m stroke\'s end, its undo; the zones; a new editor):', JSON.stringify(times));
    for (const tool of ['raise', 'smooth', 'flatten', 'lake', 'river', 'dry', 'woods']) expect(times[tool]).toBeLessThan(2);
  });
});
