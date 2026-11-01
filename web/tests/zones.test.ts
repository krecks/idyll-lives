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

  test('a zone along the world’s edge goes on past it, one away from it stays away', () => {
    const f = new ZoneField([{ kind: 'water', poly: square(0, 0, 400, 60) }], 400, 400);
    expect(f.edge('water', 200, -100)).toBeGreaterThan(100);
    expect(f.edge('water', 200, 500)).toBeLessThan(-400);
  });
});
