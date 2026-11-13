/**
 * A big world's zones as the landscape reads them (`render/shared/zones.ts`): a signed distance to
 * each kind's edge, soft shares across it, holes cut by a ring's inner loop, and a zone along the
 * world's edge carried on past it.
 */
import { describe, expect, test } from 'bun:test';
import { ZONE_CELL, ZoneField } from '../src/render/shared/zones';

const square = (x0: number, z0: number, x1: number, z1: number): [number, number][] => [
  [x0, z0],
  [x1, z0],
  [x1, z1],
  [x0, z1],
];

describe('zones', () => {
  test('inside a zone the distance to its edge is above 0, outside below, within a cell', () => {
    const f = new ZoneField([{ kind: 'water', poly: square(100, 100, 200, 200) }], 400, 400);
    expect(f.has('water')).toBe(true);
    expect(f.has('woods')).toBe(false);
    expect(f.edge('woods', 150, 150)).toBe(-Infinity);
    expect(Math.abs(f.edge('water', 150, 150) - 50)).toBeLessThan(ZONE_CELL * 1.5);
    expect(Math.abs(f.edge('water', 150, 120) - 20)).toBeLessThan(ZONE_CELL * 1.5);
    expect(Math.abs(f.edge('water', 150, 60) + 40)).toBeLessThan(ZONE_CELL * 1.5);
    expect(Math.abs(f.edge('water', 200, 150))).toBeLessThan(ZONE_CELL);
  });

  test('the share goes from 0 to 1 across the soft edge', () => {
    const f = new ZoneField([{ kind: 'woods', poly: square(100, 0, 400, 400) }], 400, 400);
    expect(f.share('woods', 50, 200, 8)).toBe(0);
    expect(f.share('woods', 200, 200, 8)).toBe(1);
    const mid = f.share('woods', 100, 200, 8);
    expect(mid).toBeGreaterThan(0.2);
    expect(mid).toBeLessThan(0.8);
  });

  test('a ring with its hole bridged in (as the generator writes them) leaves the hole out', () => {
    // The outer square clockwise, then a bridge in to the inner square the other way round and back out.
    const poly: [number, number][] = [[0, 0], [200, 0], [200, 200], [0, 200], [0, 0], [50, 50], [50, 150], [150, 150], [150, 50], [50, 50]];
    const f = new ZoneField([{ kind: 'meadow', poly }], 200, 200);
    expect(f.edge('meadow', 100, 100)).toBeLessThan(0);
    expect(f.edge('meadow', 20, 100)).toBeGreaterThan(0);
  });

  test('farmland’s edge is evened out: along a staircase of 10 m steps it runs straight, where another zone’s steps', () => {
    // A slanted edge as the generator writes one: steps of 10 m from (100, 100) down to (300, 300), land below it.
    const stairs: [number, number][] = [];
    for (let k = 0; k < 20; k++) stairs.push([100 + k * 10, 100 + k * 10], [110 + k * 10, 100 + k * 10]);
    stairs.push([300, 300], [100, 300]);
    // How far the edge (where a zone's distance is 0) strays from a straight line, walking along the slant.
    const wobble = (f: ZoneField, kind: 'fields' | 'woods') => {
      const at: number[] = [];
      for (let t = 140; t <= 260; t += 1) at.push(f.edge(kind, t + 5, t));
      const mean = at.reduce((a, v) => a + v, 0) / at.length;
      return Math.max(...at.map((v) => Math.abs(v - mean)));
    };
    const fields = new ZoneField([{ kind: 'fields', poly: stairs }], 400, 400);
    const woods = new ZoneField([{ kind: 'woods', poly: stairs }], 400, 400);
    expect(wobble(woods, 'woods')).toBeGreaterThan(1.5);
    expect(wobble(fields, 'fields')).toBeLessThan(0.75);
    // Still inside well inside, and outside well outside.
    expect(fields.edge('fields', 150, 280)).toBeGreaterThan(15);
    expect(fields.edge('fields', 280, 150)).toBeLessThan(-15);
  });

  test('a zone along the world’s edge goes on past it, one away from it stays away', () => {
    const f = new ZoneField([{ kind: 'water', poly: square(0, 0, 400, 60) }], 400, 400);
    expect(f.edge('water', 200, -100)).toBeGreaterThan(100);
    expect(f.edge('water', 200, 500)).toBeLessThan(-400);
  });
});
