/**
 * Where a car is drawn between two snapshots (`render/shared/carWay.ts` `poseOnWay`, `render/pc/cars.ts` `pose`): along
 * the way it went (sim-core `snapshot::car::TRAIL`), so at super speed, where a step runs up to 60 ticks (some 50 m of
 * driving), it follows the bends and turns it took, on its side of the road, never into the car in front; at normal
 * speed (a tick a step: no point of its way between two snapshots) on the straight line between them as it always was.
 *
 * (What this caught when written: on the straight line, two cars turning a corner one behind the other at super speed
 * were drawn across the inside of the turn, metres off their lane and into each other, though the sim kept them a
 * car's length apart; and on past the newest snapshot a late one let them drive 0.9 of a step, some 45 m, straight on.)
 */

import { describe, expect, test } from 'bun:test';
import { maxAlpha } from '../src/core/bridge';
import { NO_WAY, poseOnWay, wayOf, type Way, type WayPose } from '../src/render/shared/carWay';
import { lerpAngle } from '../src/render/shared/characters/rig';
import { layout, LAYOUT } from './snapshotLayout';

const F = LAYOUT.trail;

/** A way of points `[t, x, z, yaw]` (oldest first) as a snapshot lays it out. */
function wayFrom(points: readonly (readonly number[])[]): Way {
  const snap = new Float32Array(points.length * LAYOUT.trailStride);
  points.forEach((p, i) => {
    snap[i * LAYOUT.trailStride + F.t] = p[0];
    snap[i * LAYOUT.trailStride + F.x] = p[1];
    snap[i * LAYOUT.trailStride + F.z] = p[2];
    snap[i * LAYOUT.trailStride + F.yaw] = p[3];
  });
  return { snap, at: 0, n: points.length, stride: LAYOUT.trailStride, fields: F };
}

const pose = (x: number, z: number, yaw = 0): WayPose => ({ x, z, yaw });
const out = (): WayPose => pose(0, 0, 0);

/**
 * A car's poses tick by tick through a right-angled turn (right-hand traffic, turning left: up +z, then along -x
 * round a quarter circle of radius `R`), at `speed` m a tick, from `s0` m along its way: `[x, z, yaw]`.
 */
const R = 9;
const STRAIGHT = 40;
function along(s: number): [number, number, number] {
  if (s < STRAIGHT) return [0, s, 0];
  const arc = (Math.PI / 2) * R;
  if (s < STRAIGHT + arc) {
    const a = (s - STRAIGHT) / R;
    return [-R + R * Math.cos(a), STRAIGHT + R * Math.sin(a), -a];
  }
  return [-R - (s - STRAIGHT - arc), STRAIGHT + R, -Math.PI / 2];
}

/** The way as the sim writes it into the snapshot taken after tick `tick` (`ticks` a step): every tick of the last `back` steps. */
function wayAt(s0: number, speed: number, tick: number, ticks: number, back = 6): { prev: WayPose; curr: WayPose; way: Way } {
  const at = (k: number) => along(s0 + k * speed);
  const points: number[][] = [];
  for (let k = Math.max(0, tick - back * ticks); k < tick; k++) points.push([(k - tick) / ticks, ...at(k)]);
  const [px, pz, pyaw] = at(tick - ticks);
  const [cx, cz, cyaw] = at(tick);
  return { prev: pose(px, pz, pyaw), curr: pose(cx, cz, cyaw), way: wayFrom(points) };
}

/** The corners of a car `len` × `wid` at `p` (yaw 0 along +z). */
function corners(p: WayPose, len = 4.4, wid = 1.8): [number, number][] {
  const [hx, hz] = [Math.sin(p.yaw), Math.cos(p.yaw)];
  const [l, w] = [len / 2, wid / 2];
  return [
    [p.x + hx * l + hz * w, p.z + hz * l - hx * w],
    [p.x + hx * l - hz * w, p.z + hz * l + hx * w],
    [p.x - hx * l - hz * w, p.z - hz * l + hx * w],
    [p.x - hx * l + hz * w, p.z - hz * l - hx * w],
  ];
}

/** Whether two cars' boxes overlap (separating axes). */
function overlap(a: WayPose, b: WayPose): boolean {
  const [A, B] = [corners(a), corners(b)];
  for (const P of [A, B])
    for (let k = 0; k < 4; k++) {
      const [p, q] = [P[k], P[(k + 1) % 4]];
      const [nx, nz] = [q[1] - p[1], p[0] - q[0]];
      const pa = A.map((c) => c[0] * nx + c[1] * nz);
      const pb = B.map((c) => c[0] * nx + c[1] * nz);
      if (Math.max(...pa) < Math.min(...pb) || Math.max(...pb) < Math.min(...pa)) return false;
    }
  return true;
}

/** How far `p` is off the way (`along`) the car drives: from the nearest point of it, to a centimetre. */
function offWay(p: WayPose): number {
  let best = Infinity;
  for (let s = 0; s < 200; s += 0.01) {
    const [x, z] = along(s);
    best = Math.min(best, Math.hypot(p.x - x, p.z - z));
  }
  return best;
}

describe('a car between two snapshots', () => {
  test('at normal speed goes on the straight line between them, and on past the newest, as it always did', () => {
    // (A tick a step: its way's points are at whole steps, none after the snapshot before.)
    const prev = pose(0, 10, 0.1);
    const curr = pose(0.5, 10.9, 0.2);
    const way = wayFrom([
      [-3, 0, 8.2, 0],
      [-2, 0, 9.1, 0.05],
      [-1, 0, 10, 0.1],
    ]);
    const ahead = maxAlpha(1000 / LAYOUT.ticksPerSecond) - 1;
    for (const alpha of [0, 0.25, 0.5, 0.99, 1, 1.4, 1 + ahead]) {
      const p = poseOnWay(prev, curr, way, 1, alpha - 1, ahead, out());
      expect(p.x).toBeCloseTo(prev.x + (curr.x - prev.x) * alpha, 5);
      expect(p.z).toBeCloseTo(prev.z + (curr.z - prev.z) * alpha, 5);
      expect(p.yaw).toBeCloseTo(lerpAngle(prev.yaw, curr.yaw, alpha), 5);
    }
    // (No way at all: the same.)
    const p = poseOnWay(prev, curr, NO_WAY, 1, -0.5, ahead, out());
    expect([p.x, p.z]).toEqual([0.25, 10.45]);
  });

  test('goes no further past the newest than `ahead`, and not at all when parking', () => {
    const prev = pose(0, 0);
    const curr = pose(0, 10);
    expect(poseOnWay(prev, curr, NO_WAY, 1, 0.9, 0.5, out()).z).toBeCloseTo(15, 5);
    expect(poseOnWay(prev, curr, NO_WAY, 1, 0.9, 0, out()).z).toBe(10);
    // (And no further back than the snapshot before.)
    expect(poseOnWay(prev, curr, NO_WAY, 1, -3, 0.5, out()).z).toBe(0);
  });

  test('on past the newest goes on along the last stretch of its way, at its speed there', () => {
    // Slowing to a stop: 10 m in the step's first nine tenths, 0.1 m in its last tenth.
    const way = wayFrom([[-0.1, 0, 9.9, 0]]);
    const p = poseOnWay(pose(0, 0), pose(0, 10), way, 1, 0.05, 0.05, out());
    expect(p.z).toBeCloseTo(10.05, 4);
  });

  test('follows the points of its way after the snapshot before, and leaves out those before it', () => {
    // A corner: from (0, 0) up to (0, 10) by half the step, then over to (10, 10).
    const way = wayFrom([
      [-3, 50, 50, 0],
      [-0.5, 0, 10, 0],
    ]);
    const prev = pose(0, 0, 0);
    const curr = pose(10, 10, Math.PI / 2);
    expect(poseOnWay(prev, curr, way, 1, -0.75, 0, out())).toMatchObject({ x: 0, z: 5 });
    expect(poseOnWay(prev, curr, way, 1, -0.5, 0, out())).toMatchObject({ x: 0, z: 10 });
    const p = poseOnWay(prev, curr, way, 1, -0.25, 0, out());
    expect([p.x, p.z]).toEqual([5, 10]);
    expect(p.yaw).toBeCloseTo(Math.PI / 4, 5);
    // (Frames that missed steps: from a snapshot three steps back, which stands for the point there, to the next.)
    expect(poseOnWay(pose(0, -20), curr, way, 3, -2.75, 0, out()).z).toBeCloseTo(-17, 4);
  });

  test('reads its way from the snapshot as the sim lays it out, and none from an older layout', () => {
    const o = LAYOUT.carBase;
    const snap = new Float32Array(LAYOUT.capacity);
    snap[o + LAYOUT.car.trail] = 5;
    snap[o + LAYOUT.car.trailLen] = 2;
    const way = wayOf(snap, layout, o, { ...NO_WAY });
    expect(way.n).toBe(2);
    expect(way.at).toBe(LAYOUT.trailBase + 5 * LAYOUT.trailStride);
    expect(wayOf(snap, { ...layout, trail: undefined }, o, { ...NO_WAY })).toBe(NO_WAY);
  });

  describe('at super speed (60 ticks a step), two cars turning a corner one behind the other', () => {
    // A car's length and 2 m apart, 0.9 m a tick (as cars go at super speed: 54 m a step).
    const TICKS = 60;
    const SPEED = 0.9;
    const GAP = 4.4 + 2;
    // And one standing on the road they come up, 5.7 m to the left of their way (in the other lane, and its kerb).
    const STANDING = pose(-5.7, 21.2, 0);
    /** Every frame's pose of the two (64 frames a step, a step a frame), drawn the way `draw` says. */
    function frames(draw: (front: boolean, tick: number, r: number) => WayPose): WayPose[][] {
      const all: WayPose[][] = [];
      for (let tick = TICKS; tick <= 2 * TICKS; tick += TICKS)
        for (let r = -1; r <= 0; r += 1 / 64) all.push([draw(true, tick, r), draw(false, tick, r)]);
      return all;
    }
    // (The sim never has them into each other, nor into the one standing.)
    for (let k = 0; k <= 2 * TICKS; k++) {
      const [a, b] = [along(10 + GAP + k * SPEED), along(10 + k * SPEED)].map(([x, z, yaw]) => pose(x, z, yaw));
      if (overlap(a, b) || overlap(a, STANDING) || overlap(b, STANDING)) throw new Error(`the sim's cars overlap at tick ${k}`);
    }
    const alongWay = (front: boolean, tick: number, r: number) => {
      const { prev, curr, way } = wayAt(front ? 10 + GAP : 10, SPEED, tick, TICKS);
      return poseOnWay(prev, curr, way, 1, r, 0, out());
    };
    const straight = (front: boolean, tick: number, r: number) => {
      const { prev, curr } = wayAt(front ? 10 + GAP : 10, SPEED, tick, TICKS);
      return poseOnWay(prev, curr, NO_WAY, 1, r, 0, out());
    };

    test('on the straight line cut across the turn, into the car standing there (as they were drawn)', () => {
      const drawn = frames(straight);
      expect(Math.max(...drawn.flat().map(offWay))).toBeGreaterThan(5);
      expect(drawn.some((two) => two.some((c) => overlap(c, STANDING)))).toBe(true);
    });

    test('along their ways keep to their lane, and apart', () => {
      const drawn = frames(alongWay);
      // (Between ticks on the chord of a tick's 0.9 m of arc: a centimetre off at most.)
      expect(Math.max(...drawn.flat().map(offWay))).toBeLessThan(0.02);
      expect(drawn.some(([a, b]) => overlap(a, b) || overlap(a, STANDING) || overlap(b, STANDING))).toBe(false);
    });

    test('along their ways, a frame that missed steps still follows the turn', () => {
      // (A frame every three steps, at 20 frames a second: the snapshot before is three steps back.)
      const { curr, way } = wayAt(10, SPEED, 3 * TICKS, TICKS);
      const [px, pz, pyaw] = along(10);
      for (let r = -3; r <= 0; r += 1 / 16) expect(offWay(poseOnWay(pose(px, pz, pyaw), curr, way, 3, r, 0, out()))).toBeLessThan(0.02);
    });

    test('a late snapshot takes them a tick’s way on at most', () => {
      const ahead = (maxAlpha(1000 / LAYOUT.ticksPerSecond) - 1) / TICKS;
      const { prev, curr, way } = wayAt(10, SPEED, TICKS, TICKS);
      const p = poseOnWay(prev, curr, way, 1, 0.9, ahead, out());
      expect(Math.hypot(p.x - curr.x, p.z - curr.z)).toBeLessThan(SPEED);
    });
  });
});
