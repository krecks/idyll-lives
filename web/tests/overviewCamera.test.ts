/**
 * The menus' town camera (`render/shared/overviewCamera.ts`) for the new game's shots: the whole land framed into the
 * stage beside the side panel at every land size, within the zoom limits; an area's shot round its lots (its box while
 * it has none, the whole land for an area a bare land doesn't have); the player's pan kept over the land; the shot
 * framed again when asked.
 */
import { describe, expect, test } from 'bun:test';
import { areaFocus, EYE_OUT, OVERVIEW_CLOSE, OVERVIEW_FARTHEST, OVERVIEW_NEAREST, OverviewCamera, TILT_MAX, TILT_MIN, viewShift, type AreaWorld } from '../src/render/shared/overviewCamera';
import { LENS } from '../src/render/shared/lens';
import type { TownShot } from '../src/render/types';

const W = 1920;
const H = 1080;
/** The stage beside a side panel a quarter of the window wide. */
const STAGE = { left: 460, top: 0, right: W, bottom: H };

function camera() {
  const pose = { alpha: 0, beta: 0.9, radius: 60, target: { x: 0, y: 0, z: 0 }, targetScreenOffset: { x: 0, y: 0 }, fov: LENS.fov };
  const rig = new OverviewCamera(pose);
  rig.setViewport(W, H);
  rig.frame(0, W / H);
  return { rig, pose };
}

/** Where a ground point shows on the window (CSS pixels) through the pose, as `PlayCanvasRenderer`'s camera draws it. */
function project(pose: ReturnType<typeof camera>['pose'], x: number, y: number, z: number): [number, number] {
  const { alpha, beta, radius } = pose;
  const e = [Math.cos(alpha) * Math.sin(beta), Math.cos(beta), Math.sin(alpha) * Math.sin(beta)];
  const rl = Math.hypot(e[2], e[0]);
  const r = [-e[2] / rl, 0, e[0] / rl];
  const u = [-e[1] * r[2], -e[2] * r[0] + e[0] * r[2], e[1] * r[0]];
  const d = [x - pose.target.x - e[0] * radius, y - e[1] * radius, z - pose.target.z - e[2] * radius];
  const depth = -(d[0] * e[0] + d[1] * e[1] + d[2] * e[2]);
  const tanY = Math.tan(pose.fov / 2);
  const tanX = tanY * (W / H);
  const nx = (d[0] * r[0] + d[2] * r[2] + pose.targetScreenOffset.x) / (depth * tanX);
  const ny = (d[0] * u[0] + d[1] * u[1] + d[2] * u[2] + pose.targetScreenOffset.y) / (depth * tanY);
  return [((nx + 1) / 2) * W, ((1 - ny) / 2) * H];
}

const whole = (size: number): TownShot => ({ focus: { x: 0, z: 0, w: size, d: size }, frame: STAGE, alpha: -Math.PI / 2, beta: 0.62, zoom: 1, duration: 0 });

describe('the whole land', () => {
  for (const size of [640, 832, 960]) {
    test(`a ${size} m land fits the stage, its corners inside it, within the zoom limits`, () => {
      const { rig, pose } = camera();
      rig.setShot(whole(size), true);
      rig.frame(0, W / H);
      expect(pose.radius).toBeGreaterThan(OVERVIEW_NEAREST);
      expect(pose.radius).toBeLessThan(OVERVIEW_FARTHEST);
      for (const [x, z] of [
        [0, 0],
        [size, 0],
        [0, size],
        [size, size],
      ]) {
        const [sx, sy] = project(pose, x, 0, z);
        expect(sx).toBeGreaterThanOrEqual(STAGE.left - 1);
        expect(sx).toBeLessThanOrEqual(STAGE.right + 1);
        expect(sy).toBeGreaterThanOrEqual(STAGE.top - 1);
        expect(sy).toBeLessThanOrEqual(STAGE.bottom + 1);
      }
    });
  }

  test("the player's zoom goes out no further than the farthest", () => {
    const { rig, pose } = camera();
    rig.setShot(whole(960), true);
    for (let i = 0; i < 40; i++) rig.nudge(0, 0.8);
    for (let i = 0; i < 200; i++) rig.frame(0.05, W / H);
    expect(pose.radius).toBeLessThanOrEqual(OVERVIEW_FARTHEST + 1e-6);
  });
});

describe('the pan', () => {
  test('keeps the target over the land, however far it is dragged', () => {
    const { rig, pose } = camera();
    rig.setBounds({ x0: 0, z0: 0, x1: 640, z1: 640 });
    rig.setShot(whole(640), true);
    rig.pan(-5000, 9000);
    for (let i = 0; i < 400; i++) rig.frame(0.05, W / H);
    expect(pose.target.x).toBeCloseTo(0, 3);
    expect(pose.target.z).toBeCloseTo(640, 3);
    rig.pan(200, -100);
    for (let i = 0; i < 400; i++) rig.frame(0.05, W / H);
    expect(pose.target.x).toBeCloseTo(200, 3);
    expect(pose.target.z).toBeCloseTo(540, 3);
  });

  test('is let go by a reframe, and the camera is back on the shot', () => {
    const { rig, pose } = camera();
    rig.setBounds({ x0: 0, z0: 0, x1: 960, z1: 960 });
    rig.setShot(whole(960), true);
    rig.frame(0, W / H);
    const framed = { ...pose, target: { ...pose.target } };
    rig.pan(150, 80);
    rig.nudge(0.4, 1.5);
    for (let i = 0; i < 400; i++) rig.frame(0.05, W / H);
    expect(pose.target.x).not.toBeCloseTo(framed.target.x, 1);
    rig.reframe(0);
    rig.frame(0, W / H);
    expect(pose.target.x).toBeCloseTo(framed.target.x, 6);
    expect(pose.target.z).toBeCloseTo(framed.target.z, 6);
    expect(pose.radius).toBeCloseTo(framed.radius, 6);
    expect(pose.alpha).toBeCloseTo(framed.alpha, 6);
  });

  test('is kept over a new shot of the same land, pulled in to it', () => {
    const { rig, pose } = camera();
    rig.setBounds({ x0: 0, z0: 0, x1: 960, z1: 960 });
    rig.setShot(whole(960), true);
    rig.pan(300, 0);
    // (The same focus again: the pan stays.)
    rig.setShot({ ...whole(960), beta: 0.7 }, true);
    rig.frame(0, W / H);
    expect(pose.target.x).toBeCloseTo(780, 6);
  });
});

describe("an area's shot", () => {
  const world = (plots: AreaWorld['plots'], areas?: NonNullable<AreaWorld['meta']>['areas']): AreaWorld => ({ width: 960, depth: 960, plots, meta: areas ? { areas } : null });
  const village = { x: 600, z: 100, w: 200, d: 160 };

  test('frames the lots in its box, with a margin, and none outside it', () => {
    const f = areaFocus(
      world(
        [
          { x: 640, z: 120, w: 20, d: 30 },
          { x: 700, z: 180, w: 24, d: 30 },
          { x: 100, z: 100, w: 20, d: 30 },
        ],
        [{ x: 0, z: 0, w: 400, d: 400 }, village],
      ),
      1,
    );
    expect(f).toEqual({ x: 624, z: 104, w: 116, d: 122 });
  });

  test('frames its box while it has no lots (a village just placed)', () => {
    expect(areaFocus(world([], [{ x: 0, z: 0, w: 400, d: 400 }, village]), 1)).toEqual(village);
  });

  test("frames the whole land for an area the world doesn't have (a bare land)", () => {
    expect(areaFocus(world([]), 0)).toEqual({ x: 0, z: 0, w: 960, d: 960 });
  });

  test('stays within the land', () => {
    const f = areaFocus(world([{ x: 2, z: 940, w: 20, d: 20 }], [{ x: 0, z: 900, w: 100, d: 60 }]), 0);
    expect(f.x).toBe(0);
    expect(f.z + f.d).toBe(960);
  });
});

describe("the player's free camera", () => {
  const settle = (rig: OverviewCamera, steps = 400) => {
    for (let i = 0; i < steps; i++) rig.frame(0.05, W / H);
  };

  test('tilts within its limits, from the shot and back with a reframe', () => {
    const { rig, pose } = camera();
    rig.setShot(whole(960), true);
    rig.nudge(0, 1, 0.3);
    settle(rig);
    expect(pose.beta).toBeCloseTo(0.92, 6);
    rig.nudge(0, 1, 5);
    settle(rig);
    expect(pose.beta).toBeCloseTo(TILT_MAX, 6);
    rig.nudge(0, 1, -9);
    settle(rig);
    expect(pose.beta).toBeCloseTo(TILT_MIN, 6);
    rig.reframe(0);
    rig.frame(0, W / H);
    expect(pose.beta).toBeCloseTo(0.62, 6);
  });

  test('comes in to 10 m in the steps that shape the town, and to 50 m in the menus', () => {
    const { rig, pose } = camera();
    rig.setShot(whole(960), true);
    for (let i = 0; i < 80; i++) rig.nudge(0, 1.25);
    settle(rig);
    expect(pose.radius).toBeCloseTo(OVERVIEW_NEAREST, 3);
    rig.setNearest(OVERVIEW_CLOSE);
    for (let i = 0; i < 80; i++) rig.nudge(0, 1.25);
    settle(rig);
    expect(pose.radius).toBeCloseTo(OVERVIEW_CLOSE, 3);
    // (Back in the menus: pulled out again to their nearest.)
    rig.setNearest(OVERVIEW_NEAREST);
    settle(rig);
    expect(pose.radius).toBeCloseTo(OVERVIEW_NEAREST, 3);
  });

  test('keeps clear of the land by a share of its distance when close, and looks low along a flat street', () => {
    const { rig, pose } = camera();
    rig.ground = () => 20;
    rig.setNearest(OVERVIEW_CLOSE);
    rig.setShot(whole(960), true);
    for (let i = 0; i < 80; i++) rig.nudge(0, 1.25);
    rig.nudge(0, 1, 0.55);
    settle(rig);
    // (On the land: the target lifted onto it, the camera 3.5 m over it at most, here 2.2 m.)
    expect(pose.target.y).toBeCloseTo(20, 6);
    expect(pose.beta).toBeCloseTo(1.17, 6);
    const eyeY = pose.target.y + Math.cos(pose.beta) * pose.radius;
    expect(eyeY - 20).toBeGreaterThanOrEqual(Math.min(12, OVERVIEW_CLOSE * 0.35) - 1e-9);
    // (From far out the target stays at the land's datum, as the shot is framed.)
    rig.reframe(0);
    rig.frame(0, W / H);
    expect(pose.target.y).toBe(0);
  });

  test('not down a gorge: low along one, the eye rises over the land round it', () => {
    const { rig, pose } = camera();
    // A slot 18 m wide and 104 m deep along x through high ground (a river's way out past a river valley's edge).
    const gorge = (_x: number, z: number) => (Math.abs(z - 480) < 9 ? -4 : 100);
    rig.ground = gorge;
    rig.setShot(whole(960), true);
    settle(rig);
    // Looking along it (the eye off along x), low, 300 m out, the target in it.
    rig.nudge(-pose.alpha, 1, TILT_MAX);
    rig.nudge(0, pose.radius / 300);
    rig.pan(480 - pose.target.x, 480 - pose.target.z);
    settle(rig);
    expect(pose.radius).toBeCloseTo(300, 3);
    const flat = Math.sin(pose.beta) * pose.radius;
    const eye = [pose.target.x + Math.cos(pose.alpha) * flat, pose.target.y + Math.cos(pose.beta) * pose.radius, pose.target.z + Math.sin(pose.alpha) * flat];
    expect(Math.abs(eye[2] - 480)).toBeLessThan(9);
    // (In the gorge, 65 m up, before: its sides over it on both hands.)
    expect(eye[1]).toBeGreaterThan(100);
    expect(pose.beta).toBeLessThan(TILT_MAX);
    // The camera as drawn, moved by the shot's view offset (here tens of metres), over the land round it as well.
    const [sx, sy, sz] = viewShift(pose.alpha, pose.beta, pose.targetScreenOffset.x, pose.targetScreenOffset.y, [0, 0, 0]);
    expect(Math.hypot(sx, sy, sz)).toBeGreaterThan(10);
    for (const [ax, az] of [
      [0, 0],
      [12, 0],
      [-12, 0],
      [0, 12],
      [0, -12],
    ])
      expect(eye[1] + sy).toBeGreaterThanOrEqual(gorge(eye[0] + sx + ax, eye[2] + sz + az));
  });

  test('from far out, tilted low, it keeps within reach of a land with a landscape round it; nearer it tilts as it will', () => {
    const size = 832;
    /** How far (m) the camera as drawn stands off the land. */
    const off = (pose: ReturnType<typeof camera>['pose']) => {
      const [sx, , sz] = viewShift(pose.alpha, pose.beta, pose.targetScreenOffset.x, pose.targetScreenOffset.y, [0, 0, 0]);
      const flat = Math.sin(pose.beta) * pose.radius;
      const [x, z] = [pose.target.x + Math.cos(pose.alpha) * flat + sx, pose.target.z + Math.sin(pose.alpha) * flat + sz];
      return Math.hypot(Math.max(-x, 0, x - size), Math.max(-z, 0, z - size));
    };
    for (const alpha of [-Math.PI / 2, 0, 0.8]) {
      const { rig, pose } = camera();
      rig.setBounds({ x0: 0, z0: 0, x1: size, z1: size }, true);
      rig.setShot({ ...whole(size), alpha }, true);
      settle(rig);
      // (The whole land's shot as framed: as it was.)
      expect(pose.beta).toBeCloseTo(0.62, 6);
      // Tilted as low as it goes, 1,300 m out: no lower than keeps it within reach (well past the landscape, before:
      // 600 m and more off the land, 290 m up).
      rig.nudge(0, pose.radius / 1300, TILT_MAX);
      settle(rig);
      expect(pose.radius).toBeCloseTo(1300, 3);
      expect(pose.beta).toBeLessThan(1.1);
      expect(pose.beta).toBeGreaterThan(0.8);
      expect(off(pose)).toBeLessThanOrEqual(EYE_OUT + 0.5);
      expect(off(pose)).toBeGreaterThan(EYE_OUT - 1);
      // Further out, higher still; nearer, as low as it goes.
      rig.nudge(0, pose.radius / 2100);
      settle(rig);
      expect(pose.beta).toBeLessThan(0.6);
      expect(off(pose)).toBeLessThanOrEqual(EYE_OUT + 0.5);
      rig.nudge(0, pose.radius / 300);
      settle(rig);
      expect(pose.beta).toBeCloseTo(TILT_MAX, 6);
    }
    // Without a landscape round the land (none given), as low as it goes from anywhere.
    const { rig, pose } = camera();
    rig.setBounds({ x0: 0, z0: 0, x1: size, z1: size });
    rig.setShot(whole(size), true);
    rig.nudge(0, pose.radius / 1300, TILT_MAX);
    settle(rig);
    expect(pose.beta).toBeCloseTo(TILT_MAX, 6);
  });

  test("pans along the view: right is the picture's right, forward is into it", () => {
    const { rig, pose } = camera();
    rig.setBounds({ x0: 0, z0: 0, x1: 960, z1: 960 });
    rig.setShot({ ...whole(960), alpha: 0.7 }, true);
    rig.nudge(0, 3);
    settle(rig);
    const [cx, cz] = [pose.target.x, pose.target.z];
    const [sx, sy] = project(pose, cx, 0, cz);
    rig.panView(40, 0);
    settle(rig);
    // (The point that was in the middle now shows to the left of it, at the same height.)
    const [ax, ay] = project(pose, cx, 0, cz);
    expect(ax).toBeLessThan(sx - 50);
    expect(Math.abs(ay - sy)).toBeLessThan(1);
    rig.panView(-40, 40);
    settle(rig);
    // (Moved into the picture: the old middle is lower down, about straight below; the shot's frame is off the
    // window's middle, so a little to the side in perspective.)
    const [bx, by] = project(pose, cx, 0, cz);
    expect(by).toBeGreaterThan(sy + 50);
    expect(Math.abs(bx - sx)).toBeLessThan((by - sy) * 0.2);
  });

  test('a zoom towards a point keeps it where it shows', () => {
    const { rig, pose } = camera();
    rig.setBounds({ x0: 0, z0: 0, x1: 960, z1: 960 });
    rig.setShot(whole(960), true);
    rig.frame(0, W / H);
    const [px, pz] = [700, 300];
    const [sx, sy] = project(pose, px, 0, pz);
    rig.zoomAt(px, pz, 2);
    settle(rig);
    const [ax, ay] = project(pose, px, 0, pz);
    expect(Math.abs(ax - sx)).toBeLessThan(1);
    expect(Math.abs(ay - sy)).toBeLessThan(1);
  });

  test('its drift and sway stop once the player moves it, and come back with a reframe', () => {
    const { rig, pose } = camera();
    rig.setShot({ ...whole(960), drift: 0.05, sway: 0.3 }, true);
    settle(rig, 20);
    rig.nudge(0.1, 1);
    settle(rig, 40);
    const still = pose.alpha;
    settle(rig, 200);
    expect(pose.alpha).toBeCloseTo(still, 6);
    rig.reframe(0);
    settle(rig, 200);
    expect(Math.abs(pose.alpha - still)).toBeGreaterThan(0.05);
  });
});
