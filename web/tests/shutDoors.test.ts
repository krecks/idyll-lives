// Every door stands shut and swings open for someone going through it (`render/shared/house.ts`
// builds each leaf on its own, `PlayCanvasRenderer.swingDoors` turns it): the front door, a door
// between rooms and one in a diagonal wall alike. `bun test`.
//
// (Written for the owner's play of 0.7.0: an interior door stood wide open with nobody near. Only
// front doors and the doors of rooms the simulation keeps shut swung then; the rest were built
// into the house standing open.)
import { expect, test } from 'bun:test';
import type { DiagonalWall, Opening, WallEdge, WorldStructure } from '../src/core/protocol';
import { doorLead, goingThrough } from '../src/render/shared/doors';
import { HousePlanner, type DoorPlan } from '../src/render/shared/house';
import { WALL_HALF as T } from '../src/render/shared/walls';

const W = 12;
const D = 12;

/**
 * A box house 6 by 5 tiles from (2, 2): the front door at the back (z = 7), a wall at x = 5 with a
 * door between the rooms either side of it (room 1 west, 2 east), and a diagonal doorway in room 1.
 */
function house(): WorldStructure {
  const walls: WallEdge[] = [];
  const openings: Opening[] = [];
  const edge = (axis: 'h' | 'v', x: number, z: number, kind?: 'door' | 'window') => {
    walls.push({ axis, x, z });
    if (kind) openings.push({ axis, x, z, kind });
  };
  for (let x = 2; x < 8; x++) {
    edge('h', x, 2, x === 3 ? 'window' : undefined);
    edge('h', x, 7, x === 6 ? 'door' : undefined);
  }
  for (let z = 2; z < 7; z++) {
    edge('v', 2, z);
    edge('v', 8, z);
    edge('v', 5, z, z === 3 ? 'door' : undefined);
  }
  const rooms = new Array(W * D).fill(0);
  for (let z = 2; z < 7; z++) for (let x = 2; x < 8; x++) rooms[z * W + x] = x < 5 ? 1 : 2;
  const diagonals: DiagonalWall[] = [{ axis: 'dn', x: 3, z: 5, kind: 'door', rooms: [1, 1] }];
  return { width: W, depth: D, walls, openings, diagonals, rooms, objects: [], plots: [], households: [], sims: [], exits: [] } as unknown as WorldStructure;
}

function plan(preview = false) {
  const built = new HousePlanner().build(house(), { x: 0, z: 0, w: W, d: D }, preview ? { preview: true } : {});
  if (!built) throw new Error('no house');
  return built;
}

/** The unit direction along a door's doorway and across it (x, z), the way `goingThrough` counts across as +1. */
function frame(door: DoorPlan): { u: [number, number]; n: [number, number] } {
  const u: [number, number] = door.axis === 'h' ? [1, 0] : door.axis === 'v' ? [0, 1] : [door.axis[0], door.axis[1]];
  return { u, n: door.axis === 'v' ? [1, 0] : [-u[1], u[0]] };
}

/** Every vertex of a leaf turned about its pivot by `turn` (as the renderer turns its entity), as (along, across) its doorway from its middle. */
function leafAt(door: DoorPlan, turn: number): [number, number][] {
  const { u, n } = frame(door);
  const [c, s] = [Math.cos(turn), Math.sin(turn)];
  const out: [number, number][] = [];
  for (const part of door.parts) {
    const p = part.geo.pos;
    for (let i = 0; i < p.length; i += 3) {
      const [dx, dz] = [p[i] - door.pivot[0], p[i + 2] - door.pivot[1]];
      const x = door.pivot[0] + dx * c + dz * s - door.centre[0];
      const z = door.pivot[1] + dz * c - dx * s - door.centre[1];
      out.push([x * u[0] + z * u[1], x * n[0] + z * n[1]]);
    }
  }
  return out;
}

test('every door is a leaf of its own: the front door, the one between the rooms, the diagonal one', () => {
  const { doors } = plan();
  expect(doors.length).toBe(3);
  expect(doors.filter((d) => d.front).length).toBe(1);
  expect(doors.filter((d) => d.axis === 'v').length).toBe(1);
  expect(doors.filter((d) => typeof d.axis !== 'string').length).toBe(1);
});

test('at rest each stands shut in its doorway, and open it leaves the way through clear', () => {
  for (const door of plan().doors) {
    expect(door.slide).toBeUndefined();
    // Shut: in the wall (its handles 3.5 cm proud of the leaf), across the opening.
    const shut = leafAt(door, door.close);
    expect(Math.max(...shut.map(([, n]) => Math.abs(n)))).toBeLessThan(T);
    expect(Math.max(...shut.map(([a]) => Math.abs(a)))).toBeLessThan(0.5 - T + 1e-6);
    // Across most of the opening: no gap to see through.
    expect(Math.min(...shut.map(([a]) => a)) - -(0.5 - T)).toBeLessThan(0.03);
    expect(0.5 - T - Math.max(...shut.map(([a]) => a))).toBeLessThan(0.03);
    // Open all the way: against its jamb, standing out of the wall, the middle of the doorway clear.
    expect(door.opens).toBe(1);
    const open = leafAt(door, door.close * (1 - door.opens));
    expect(Math.min(...open.map(([a]) => Math.abs(a)))).toBeGreaterThan(0.3);
    expect(Math.max(...open.map(([, n]) => Math.abs(n)))).toBeGreaterThan(0.8);
  }
});

test('a door between rooms opens for someone walking through it, not for someone walking past', () => {
  const lead = doorLead(1);
  for (const door of plan().doors.filter((d) => !d.front)) {
    const { u, n } = frame(door);
    // (x, z) at `a` along the doorway and `b` across it, from its middle.
    const at = (a: number, b: number) => [door.centre[0] + u[0] * a + n[0] * b, door.centre[1] + u[1] * a + n[1] * b];
    const walker = (from: number[], to: number[]) => [...from, ...to, 0];
    // A hand's reach short, on their way across: open.
    expect(goingThrough(walker(at(0, -0.6), at(0, 0.9)), door.axis, door.centre, false, lead)).toBe(1);
    expect(goingThrough(walker(at(0.1, 0.6), at(0.1, -0.9)), door.axis, door.centre, false, lead)).toBe(-1);
    // Along the wall beside it, or off away from it: it stays shut.
    expect(goingThrough(walker(at(-1.5, -0.6), at(1.5, -0.6)), door.axis, door.centre, false, lead)).toBe(0);
    expect(goingThrough(walker(at(0, -0.8), at(0, -2.3)), door.axis, door.centre, false, lead)).toBe(0);
    // Through and on: it stays open while they are near, then shuts.
    expect(goingThrough(walker(at(0, 0.8), at(0, 2.3)), door.axis, door.centre, true, lead)).not.toBe(0);
    expect(goingThrough(walker(at(0, 1.5), at(0, 3)), door.axis, door.centre, true, lead)).toBe(0);
  }
});

test("a preview draws every leaf at rest, shut, in the house's own meshes", () => {
  expect(plan(true).doors).toEqual([]);
});
