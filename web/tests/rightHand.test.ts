// Traffic keeps to the right as the game shows it (the owner's choice, 2026-11-24: German rules), pinned end to end
// with sim-core's `travel_hierarchy::traffic_keeps_to_the_right_as_the_game_draws_it`: the simulation's one right
// (`cars::right_of`, read from its source here), taken through the world root's mirror (`space.ts`) and the game's own
// camera (`GameCamera`) to the screen. A camera behind a car sees its lane on the screen's right half, one in front of
// it on the left; the driver's door (door 0) on the car's left; an avenue's lanes and a motorway's pair as the
// renderer lays them on the same sides.
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import * as pc from '../src/render/pc/engine';
import type { RoadPlan } from '../src/core/protocol';
import { constant } from './snapshotLayout';

// (The renderer's modules read the page's address as they load: one put in for them, the real one back after.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
const { GameCamera } = await import('../src/render/pc/camera');
const { createWorldRoot } = await import('../src/render/pc/space');
const { carDoor, CARS } = await import('../src/render/shared/cars');
const { laneLayout } = await import('../src/render/shared/markings');
const { insideOf } = await import('../src/render/shared/motorway');

type Pt = [number, number];

/** sim-core's `cars::right_of`, as its source writes it: the right of going `d` (x, z). */
function simRightOf(): (d: Pt) => Pt {
  const src = readFileSync(join(import.meta.dir, '../../crates/sim-core/src/cars/lane.rs'), 'utf8');
  const body = /pub fn right_of\(d: \[f32; 2\]\) -> \[f32; 2\] \{\s*\[([^\]]*\]), ([^\]]*\])\]\s*\}/.exec(src);
  if (!body) throw new Error('no right_of in cars/lane.rs');
  const term = (t: string) => {
    const m = /^(-?)d\[([01])\]$/.exec(t.trim());
    if (!m) throw new Error(`right_of: ${t}`);
    const [sign, k] = [m[1] ? -1 : 1, Number(m[2])];
    return (d: Pt) => sign * d[k];
  };
  const [x, z] = [term(body[1]), term(body[2])];
  return (d) => [x(d), z(d)];
}

/** The game's view: a 1920 × 1080 picture through the game's camera under a world root mirrored as the game's is. */
function view() {
  const canvas = { width: 1920, height: 1080, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 1920, height: 1080 }) } as unknown as HTMLCanvasElement;
  const device = new pc.NullGraphicsDevice(canvas);
  const app = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.CameraComponentSystem];
  options.resourceHandlers = [];
  app.init(options);
  const root = createWorldRoot(app);
  const camera = new GameCamera(app.root, { fov: Math.PI / 4, near: 0.1, far: 1000 });
  camera.camera.aspectRatio = 1920 / 1080;
  /** Where game-space point (x, z) on the ground is on the screen (px across; null: not in view). */
  const screenX = (x: number, z: number): number | null => {
    // (Through the world root: a thing the game puts at (x, z) stands where the engine draws it.)
    const node = new pc.GraphNode();
    root.addChild(node);
    node.setLocalPosition(x, 0, z);
    const at = node.getPosition();
    root.removeChild(node);
    const out = { x: 0, y: 0 };
    // (`project` takes game space: the engine's point back through the mirror.)
    return camera.project(at.x, at.y, -at.z, 1920, 1080, out) ? out.x : null;
  };
  return { camera, screenX };
}

describe('traffic on the right as the game shows it', () => {
  const right = simRightOf();
  const LANE = constant('LANE');
  const { camera, screenX } = view();
  // A car at the origin going +x (the sim's yaw a quarter turn: forward (sin, cos)).
  const yaw = Math.PI / 2;
  const forward: Pt = [Math.sin(yaw), Math.cos(yaw)];
  const r = right(forward);
  /** The camera `radius` m behind the car (`behind` 1) or in front of it (-1), looking at it: the arc camera's eye is target + radius (cos α sin β, cos β, sin α sin β). */
  const look = (behind: 1 | -1) => camera.setPose(Math.atan2(-forward[1] * behind, -forward[0] * behind), 1.1, 20, 0, 0, 0);

  test("the sim's right of going +x is -z", () => {
    expect(r[0]).toBeCloseTo(0, 9);
    expect(r[1]).toBeCloseTo(-1, 9);
  });

  test('seen from behind, a car keeps to the right half of the road; seen from in front, to the left', () => {
    look(1);
    const lane = screenX(r[0] * LANE, r[1] * LANE)!;
    const oncoming = screenX(-r[0] * LANE, -r[1] * LANE)!;
    expect(lane).toBeGreaterThan(960 + 10);
    expect(oncoming).toBeLessThan(960 - 10);
    look(-1);
    expect(screenX(r[0] * LANE, r[1] * LANE)!).toBeLessThan(960 - 10);
  });

  test("the driver sits on the car's left: door 0 on the left half of the picture from behind", () => {
    look(1);
    const body = CARS[0];
    for (const d of [0, 1]) {
      // (A door's side across the car, +x as the car is made; turned by its yaw, +x is (cos, -sin).)
      const s = carDoor(body, d).side * (body.wid / 2);
      const at = screenX(Math.cos(yaw) * s, -Math.sin(yaw) * s)!;
      if (d === 0) expect(at).toBeLessThan(960 - 10);
      else expect(at).toBeGreaterThan(960 + 10);
    }
  });

  test("an avenue's lanes, and a motorway's pair, on the sides the sim drives them", () => {
    // (Along +x; a lane's offset `off` lies at (t.z, -t.x) · off of the way its points run: `markings.ts`.)
    const line = (z: number): Pt[] => Array.from({ length: 101 }, (_, k) => [k, z]);
    const avenue: RoadPlan = { points: line(0), width: 14, ends: ['out', 'out'], kind: 'avenue' };
    const lay = laneLayout([avenue], 0)!;
    look(1);
    for (const l of lay.lanes) {
      const at = screenX(0, -l.off)!;
      // (Going along its points, +x, as the car: on the right; the other way, on the left.)
      if (l.way === 1) expect(at).toBeGreaterThan(960 + 10);
      else expect(at).toBeLessThan(960 - 10);
    }
    // (Two carriageways along +x: going +x at z = 0, its pair going -x at z = 9, on its left.)
    const east: RoadPlan = { points: line(0), width: 9, ends: ['out', 'out'], kind: 'motorway', oneway: 1 };
    const west: RoadPlan = { points: line(9), width: 9, ends: ['out', 'out'], kind: 'motorway', oneway: -1 };
    const roads = [east, west];
    // (Its inside, towards the pair: -1, the other side from (t.z, -t.x), +z.)
    expect(insideOf(roads, 0)).toBe(-1);
    expect(screenX(0, 9)!).toBeLessThan(960 - 10);
  });
});
