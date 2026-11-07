/**
 * The snapshot's car rows as the renderer reads them (`render/shared/cars.ts` `carRowAt`,
 * `carRowIsTaxi`; `render/pc/cars.ts` draws them): laid out as sim-core writes them (`snapshot.rs`
 * `CAR_BASE`, `CAR_STRIDE`, `car::*`, read from its source: `snapshotLayout.ts`), six floats a row,
 * the households' cars first in the structure's order, then the work taxis' (`car.kind`); and the
 * taxi's own look (`paintTaxi`): yellow, its sign on its roof.
 */

import { describe, expect, test } from 'bun:test';
import type { SnapshotLayout } from '../src/core/snapshot';
import { CARS, carRowAt, carRowIsTaxi, paintTaxi, paintTaxiDoor, TAXI_BODY } from '../src/render/shared/cars';
import { Geo } from '../src/render/shared/geometry';
import { Painter } from '../src/render/shared/street';
import { BASES, layout, LAYOUT, SNAPSHOT_RS } from './snapshotLayout';

const F = LAYOUT.car;
const COUNT = LAYOUT.header.carCount;
const K = F.kinds;

/** A car's row: shown, where, which way, its kind (the sim's value). */
type Row = { shown: number; x: number; z: number; yaw: number; kind: number };

function snapshot(rows: readonly Row[]): Float32Array {
  const snap = new Float32Array(layout.capacity);
  snap[COUNT] = rows.length;
  rows.forEach((r, k) => {
    const o = carRowAt(layout, k);
    snap[o + F.shown] = r.shown;
    snap[o + F.x] = r.x;
    snap[o + F.z] = r.z;
    snap[o + F.yaw] = r.yaw;
    snap[o + F.kind] = r.kind;
  });
  return snap;
}

/** Two households' cars (one parked, one driving), then two taxis (one out, one unused: not shown). */
const ROWS: Row[] = [
  { shown: 1, x: 12.5, z: 40.2, yaw: 0, kind: K.household },
  { shown: 2, x: 30.1, z: 55.7, yaw: 1.2, kind: K.household },
  { shown: 2, x: 81.4, z: 20.9, yaw: -0.4, kind: K.taxi },
  { shown: 0, x: 0, z: 0, yaw: 0, kind: K.taxi },
];

describe('car rows', () => {
  test('are six floats each, after the residents and servers, as sim-core writes them', () => {
    expect(SNAPSHOT_RS).toContain(BASES.car);
    expect(LAYOUT.carStride).toBe(6);
    expect(F.kind).toBe(5);
    expect(Math.max(F.shown, F.x, F.z, F.yaw, F.doors, F.kind)).toBeLessThan(LAYOUT.carStride);
    expect(K.household).not.toBe(K.taxi);
    // (The lights' rows right after the room for every car.)
    expect(LAYOUT.lightBase).toBe(LAYOUT.carBase + LAYOUT.maxCars * LAYOUT.carStride);
    expect(LAYOUT.header.lightCount).toBe(11);
  });

  test('start a stride apart from the cars’ base', () => {
    for (const row of [0, 1, 7, 127]) expect(carRowAt(layout, row)).toBe(LAYOUT.carBase + row * 6);
    expect(carRowAt({ ...layout, carStride: undefined } as SnapshotLayout, 3)).toBe(-1);
  });

  test('read where each car is through the stride', () => {
    const snap = snapshot(ROWS);
    ROWS.forEach((r, k) => {
      const o = carRowAt(layout, k);
      expect(snap[o + F.shown]).toBe(r.shown);
      expect(snap[o + F.x]).toBeCloseTo(r.x, 4);
      expect(snap[o + F.z]).toBeCloseTo(r.z, 4);
      expect(snap[o + F.yaw]).toBeCloseTo(r.yaw, 5);
    });
  });

  test('say which are taxis: the households’ first, the taxis after them, an unused one too', () => {
    const snap = snapshot(ROWS);
    expect([0, 1, 2, 3].map((k) => carRowIsTaxi(snap, layout, k))).toEqual([false, false, true, true]);
    // Past the rows there are, none.
    expect(carRowIsTaxi(snap, layout, 4)).toBe(false);
    expect(carRowIsTaxi(snap, layout, -1)).toBe(false);
    snap[COUNT] = 2;
    expect(carRowIsTaxi(snap, layout, 2)).toBe(false);
  });

  test('are all households’ in a layout without kinds', () => {
    const snap = snapshot(ROWS);
    const old = { ...layout, car: { ...LAYOUT.car, kind: undefined, kinds: undefined } } as unknown as SnapshotLayout;
    expect([0, 1, 2, 3].some((k) => carRowIsTaxi(snap, old, k))).toBe(false);
  });
});

describe('the taxi', () => {
  /** Its geometry's highest point and its colours (vertex colours, as bytes). */
  function painted(paint: (p: Painter) => void): { top: number; count: number } {
    const p = new Painter(new Geo());
    paint(p.at(0, 0, 0, 0));
    const pos = p.geo.pos;
    let top = -Infinity;
    for (let i = 1; i < pos.length; i += 3) top = Math.max(top, pos[i]);
    return { top, count: pos.length / 3 };
  }

  test('wears a sign on its roof, above any household car of its body', () => {
    const roof = CARS[TAXI_BODY].cabin[5];
    const taxi = painted((p) => paintTaxi(p));
    expect(taxi.top).toBeGreaterThan(roof + 0.15);
    expect(taxi.top).toBeLessThan(roof + 0.3);
  });

  test('has its band on its doors when they are apart, and the body without them', () => {
    for (let d = 0; d < 4; d++) expect(painted((p) => paintTaxiDoor(p, d)).count).toBeGreaterThan(0);
    const whole = painted((p) => paintTaxi(p, false));
    const apart = painted((p) => paintTaxi(p, true));
    expect(whole.count).toBeGreaterThan(0);
    expect(apart.count).toBeGreaterThan(0);
  });
});
