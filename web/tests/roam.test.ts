/**
 * The roaming camera (`render/shared/focus.ts`, `orbit.ts`'s pan bounds): the chunks streamed in
 * round the camera with hysteresis, the focus lot reported once the camera settles on it, a
 * flight across the world, and the target kept on the land.
 */
import { describe, expect, test } from 'bun:test';
import { chunkAt, CHUNK } from '../src/render/shared/chunks';
import { Flight, HYSTERESIS, panBounds, Roam, SETTLE_MS, streamRadius, WARM_MIN_GAIN, warmLots, type FocusPlot } from '../src/render/shared/focus';
import { OrbitControls, pointerInteraction } from '../src/render/shared/orbit';

const cam = (x: number, z: number, radius = 30) => ({ x, z, radius, alpha: 0, fovH: 1.1 });
const none = () => null;
const take = () => true;

describe('roaming', () => {
  test('the stream radius grows with the camera’s distance, within 120..260 m', () => {
    expect(streamRadius(6)).toBe(120);
    expect(streamRadius(60)).toBe(152);
    expect(streamRadius(400)).toBe(260);
  });

  test('chunks come in round the camera and go only past the hysteresis', () => {
    const roam = new Roam();
    expect(roam.update(cam(500, 500), 0, none, take)).toBe(true);
    const here = chunkAt(500, 500);
    expect(roam.chunks.has(here)).toBe(true);
    const r = streamRadius(30);
    // A chunk just inside the radius behind the camera (it looks along -x: alpha 0), then the camera moves a little away from it.
    const behindX = 500 + r - 1;
    const behind = chunkAt(behindX + CHUNK / 2, 500);
    const edge = Math.floor((behindX + 1) / CHUNK) * CHUNK;
    const near = new Roam();
    near.update(cam(edge - r + 1, 500), 0, none, take);
    expect(near.chunks.has(chunkAt(edge + 1, 500))).toBe(true);
    // Moved back by less than the hysteresis: it stays; by more: it goes.
    near.update(cam(edge - r - HYSTERESIS / 2, 500), 16, none, take);
    expect(near.chunks.has(chunkAt(edge + 1, 500))).toBe(true);
    near.update(cam(edge - r - HYSTERESIS - 4, 500), 32, none, take);
    expect(near.chunks.has(chunkAt(edge + 1, 500))).toBe(false);
    expect(behind).toBeGreaterThan(0);
  });

  test("a preset's share of the stream radius streams fewer chunks", () => {
    expect(streamRadius(60, 0.7)).toBeCloseTo(152 * 0.7);
    const full = new Roam();
    const low = new Roam();
    low.streamScale = 0.7;
    full.update(cam(1000, 1000, 60), 0, none, take);
    low.update(cam(1000, 1000, 60), 0, none, take);
    expect(low.chunks.size).toBeLessThan(full.chunks.size);
    for (const id of low.chunks) expect(full.chunks.has(id)).toBe(true);
  });

  test('a still camera keeps the same set, and a camera that moved works it out again', () => {
    const roam = new Roam();
    roam.update(cam(500, 500), 0, none, take);
    const first = roam.chunks;
    // (Unchanged: not even a new set.)
    expect(roam.update(cam(500, 500), 16, none, take)).toBe(false);
    expect(roam.update(cam(500.001, 500), 32, none, take)).toBe(false);
    expect(roam.chunks).toBe(first);
    // Creeping by less than the threshold a frame still gets there: the comparison is with where it was last worked out.
    let x = 500;
    for (let t = 48; x < 500 + CHUNK * 3; t += 16) {
      x += 0.004;
      roam.update(cam(x, 500), t, none, take);
      if (roam.chunks !== first) break;
    }
    expect(roam.chunks).not.toBe(first);
    const moved = new Roam();
    moved.update(cam(500, 500), 0, none, take);
    moved.update(cam(x, 500), 16, none, take);
    expect([...roam.chunks].sort()).toEqual([...moved.chunks].sort());
    // A big move: the chunks follow.
    expect(roam.update(cam(900, 500), 2000, none, take)).toBe(true);
    expect(roam.chunks.has(chunkAt(900, 500))).toBe(true);
  });

  test('more is streamed ahead of the camera than behind it', () => {
    const roam = new Roam();
    roam.update(cam(1000, 1000, 60), 0, none, take);
    const r = streamRadius(60);
    // Looking along -x (alpha 0): a chunk past the radius ahead is in, the same behind is not.
    expect(roam.chunks.has(chunkAt(1000 - r - 66, 1000))).toBe(true);
    expect(roam.chunks.has(chunkAt(1000 + r + 66, 1000))).toBe(false);
  });

  test('the focus lot is reported once the camera has settled on it, and only if the game takes it', () => {
    const plot: FocusPlot = { id: 7, x: 0, z: 0, w: 20, d: 20 };
    const at = (x: number, z: number) => (x >= 0 && x < 20 && z >= 0 && z < 20 ? plot : null);
    const roam = new Roam();
    const seen: number[] = [];
    let taking = false;
    const report = (p: FocusPlot) => {
      seen.push(p.id);
      return taking;
    };
    roam.update(cam(10, 10), 0, at, report);
    roam.update(cam(10, 10), SETTLE_MS / 2, at, report);
    expect(seen).toEqual([]);
    roam.update(cam(10, 10), SETTLE_MS + 1, at, report);
    expect(seen).toEqual([7]);
    expect(roam.focus).toBe(null);
    taking = true;
    roam.update(cam(10, 10), SETTLE_MS + 20, at, report);
    expect(roam.focus).toBe(7);
    roam.update(cam(10, 10), SETTLE_MS + 40, at, report);
    expect(seen).toEqual([7, 7]);
    // Pinned (following someone): nothing.
    roam.pinned = true;
    roam.setFocus(null);
    roam.update(cam(10, 10), 2000, at, report);
    expect(seen).toEqual([7, 7]);
  });

  test('a flight eases to its end, rising on the way when far', () => {
    const f = new Flight();
    f.start(0, 0, 30, 600, 0, 30);
    const out = { x: 0, z: 0, radius: 0 };
    let peak = 0;
    for (let i = 0; i < 400 && f.active; i++) {
      f.step(1 / 60, out);
      peak = Math.max(peak, out.radius);
    }
    expect(out.x).toBeCloseTo(600, 5);
    expect(out.radius).toBeCloseTo(30, 5);
    expect(peak).toBeGreaterThan(60);
  });

  test('with Reduce motion a flight is a cut: there on the next step, without rising', () => {
    const f = new Flight();
    f.start(0, 0, 30, 600, 0, 40, true);
    const out = { x: 0, z: 0, radius: 0 };
    expect(f.step(1 / 60, out)).toBe(true);
    expect(out).toEqual({ x: 600, z: 0, radius: 40 });
    expect(f.active).toBe(false);
  });

  test('the middle button turns the camera (as while the left draws walls), the right pans', () => {
    expect(pointerInteraction(1, false)).toBe('rotate');
    expect(pointerInteraction(0, false)).toBe('rotate');
    expect(pointerInteraction(0, true)).toBe('pan');
    expect(pointerInteraction(2, false)).toBe('pan');
    expect(pointerInteraction(3, false)).toBe(null);
  });

  test('the target stays on the land: the town and 60 m round it, a big world’s land and 20 m', () => {
    const o = new OrbitControls({ alpha: 0, beta: 1, radius: 30 }, { radius: [6, 95], beta: [0.3, 1.35] });
    o.panBounds = panBounds({ width: 200, depth: 600, storeyDepth: 150 });
    o.target.x = -500;
    o.target.z = 900;
    o.checkLimits();
    expect([o.target.x, o.target.z]).toEqual([-60, 210]);
    expect(panBounds({ width: 1000, depth: 1000, meta: { world: { size: 'big' } } })).toEqual({ x0: -20, z0: -20, x1: 1020, z1: 1020 });
  });
});

describe('warm-up lots', () => {
  const plot = (x: number) => ({ x, z: 0, w: 10, d: 10, house: [x, 0, x + 5, 5] });
  test('the lots adding the most not yet seen, the home and houseless lots left out, none that add little', () => {
    const world = {
      plots: [plot(0), plot(10), plot(20), { ...plot(30), house: null }, plot(40)],
      depth: 10,
      walls: [
        { x: 1, z: 1, faces: [1, 2] },
        { x: 11, z: 1, faces: [1, 2] },
        { x: 21, z: 1, faces: [3, 4] },
        { x: 41, z: 11, faces: [5, 5] },
      ],
      openings: [
        { x: 22, z: 1, kind: 'window', style: 2 },
        { x: 23, z: 1, kind: 'door' },
        { x: 42, z: 11, kind: 'window', style: 3 },
      ],
      floors: [[12, 2, 7]],
      objects: [{ x: 31, z: 1, def: 'sofa' }],
    };
    // Lot 20 adds 4 (two faces, a window style, a door), lot 40 (upstairs, row 11) three (two faces, a window
    // style), lot 10 one (its floor: its faces are the home's, lot 0, seen already): fewer than the least worth
    // building.
    expect(WARM_MIN_GAIN).toBeGreaterThan(1);
    expect(WARM_MIN_GAIN).toBeLessThanOrEqual(3);
    const gains: number[] = [];
    expect(warmLots(world, { x: 0, z: 0 }, 5, gains).map((p) => p.x)).toEqual([20, 40]);
    expect(gains).toEqual([4, 3]);
    expect(warmLots(world, { x: 0, z: 0 }, 1).map((p) => p.x)).toEqual([20]);
  });
});
