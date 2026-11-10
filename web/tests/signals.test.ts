/**
 * The traffic lights' signal-head rows as the renderer reads them (`render/shared/signals.ts`
 * `SignalRows`, drawn by `render/pc/signals.ts`): laid out as sim-core's snapshot writes them
 * (`snapshot.rs` `LIGHT_*`, `header::LIGHT_COUNT`, `light::*`, read from its source: `snapshotLayout.ts`), the rows
 * shown, where each pole stands and which way it faces, the lamp each shows to traffic and whether
 * its pedestrians' head shows walk, and what changed from one read to the next; the pedestrians' head
 * as drawn.
 */

import { describe, expect, test } from 'bun:test';
import type { SnapshotLayout } from '../src/core/snapshot';
import { AMBER, GREEN, lampOf, pedestrianHead, RED, SignalRows, walkOf } from '../src/render/shared/signals';
import { BASES, constant, layout, LAYOUT, SNAPSHOT_RS } from './snapshotLayout';

/** The light rows' part of the layout, as sim-core's `layout_json` writes it (worked out from snapshot.rs: `snapshotLayout.ts`). */
const STRIDE = LAYOUT.lightStride;
const BASE = LAYOUT.lightBase;
const F = LAYOUT.light;
const COUNT = LAYOUT.header.lightCount;

/** A head's row: shown, its pole and facing, its state (the sim's value). */
type Row = { shown?: number; x: number; z: number; yaw: number; state: number };

function snapshot(rows: readonly Row[]): Float32Array {
  const snap = new Float32Array(layout.capacity);
  snap[COUNT] = rows.length;
  rows.forEach((r, k) => {
    const o = BASE + k * STRIDE;
    snap[o + F.shown] = r.shown ?? 1;
    snap[o + F.x] = r.x;
    snap[o + F.z] = r.z;
    snap[o + F.yaw] = r.yaw;
    snap[o + F.state] = r.state;
  });
  return snap;
}

const S = F.states;
/** A lit junction's four arms: one green, the rest red. */
const JUNCTION: Row[] = [
  { x: 101.2, z: 47.5, yaw: 0, state: S.green },
  { x: 108.5, z: 51.2, yaw: Math.PI / 2, state: S.red },
  { x: 104.8, z: 58.5, yaw: Math.PI, state: S.red },
  { x: 97.5, z: 54.8, yaw: -Math.PI / 2, state: S.red },
];

describe('signal-head rows', () => {
  test('are laid out as sim-core writes them', () => {
    // (Right after the cars' rows, as snapshot.rs works it out; the fields fit the row; the header's count is in the header.)
    expect(SNAPSHOT_RS).toContain(BASES.light);
    expect(SNAPSHOT_RS).toContain(BASES.capacity);
    expect(BASE).toBe(LAYOUT.carBase + LAYOUT.maxCars * LAYOUT.carStride);
    expect(Math.max(F.shown, F.x, F.z, F.yaw, F.state)).toBeLessThan(STRIDE);
    expect(COUNT).toBeLessThan(constant('HEADER_LEN'));
    expect(new Set(Object.values(S)).size).toBe(4);
  });

  test('reads each head: where its pole stands, which way it faces, the lamp it shows', () => {
    const rows = new SignalRows();
    rows.read(snapshot(JUNCTION), layout);
    expect(rows.count).toBe(4);
    JUNCTION.forEach((r, k) => {
      expect(rows.x[k]).toBeCloseTo(r.x, 4);
      expect(rows.z[k]).toBeCloseTo(r.z, 4);
      expect(rows.yaw[k]).toBeCloseTo(r.yaw, 5);
    });
    expect([...rows.lamp.subarray(0, 4)]).toEqual([GREEN, RED, RED, RED]);
  });

  test('shows traffic red for walk and for a state it does not know, amber and green as they are', () => {
    expect(lampOf(S.red, S)).toBe(RED);
    expect(lampOf(S.amber, S)).toBe(AMBER);
    expect(lampOf(S.green, S)).toBe(GREEN);
    expect(lampOf(S.walk, S)).toBe(RED);
    expect(lampOf(7, S)).toBe(RED);
  });

  test('shows those on foot walk only for walk, else do not walk', () => {
    expect(walkOf(S.walk, S)).toBe(true);
    for (const state of [S.red, S.amber, S.green, 7]) expect(walkOf(state, S)).toBe(false);
  });

  test('reads the walk phase: every head red to traffic and walk to those on foot, and back', () => {
    const rows = new SignalRows();
    const snap = snapshot(JUNCTION);
    rows.read(snap, layout);
    expect([...rows.walk.subarray(0, 4)]).toEqual([0, 0, 0, 0]);
    for (let k = 0; k < 4; k++) snap[BASE + k * STRIDE + F.state] = S.walk;
    expect(rows.read(snap, layout)).toEqual({ placed: false, lit: true });
    expect([...rows.lamp.subarray(0, 4)]).toEqual([RED, RED, RED, RED]);
    expect([...rows.walk.subarray(0, 4)]).toEqual([1, 1, 1, 1]);
    // From walk to all red: traffic's lamps the same, those on foot's changed.
    for (let k = 0; k < 4; k++) snap[BASE + k * STRIDE + F.state] = S.red;
    expect(rows.read(snap, layout)).toEqual({ placed: false, lit: true });
    expect([...rows.walk.subarray(0, 4)]).toEqual([0, 0, 0, 0]);
  });

  test("the pedestrians' head hangs on the pole below the traffic head, its figures facing across the road", () => {
    const { housing, walk, dontWalk } = pedestrianHead();
    for (const g of [housing, walk, dontWalk]) {
      expect(g.empty).toBe(false);
      for (let i = 0; i < g.pos.length; i += 3) {
        // To the -x side of the pole (towards the road's middle), under the traffic head (its board from 1.97 m up).
        expect(g.pos[i]).toBeLessThanOrEqual(0);
        expect(g.pos[i + 1]).toBeGreaterThan(1.3);
        expect(g.pos[i + 1]).toBeLessThan(1.96);
      }
    }
    // The figures face -x, don't walk above walk.
    for (const g of [walk, dontWalk]) for (let i = 0; i < g.nor.length; i += 3) expect(g.nor[i]).toBe(-1);
    const top = (g: typeof walk) => Math.max(...g.pos.filter((_, i) => i % 3 === 1));
    expect(top(dontWalk)).toBeGreaterThan(top(walk));
  });

  test('leaves out the rows not shown', () => {
    const rows = new SignalRows();
    rows.read(snapshot([JUNCTION[0], { ...JUNCTION[1], shown: 0 }, JUNCTION[2]]), layout);
    expect(rows.count).toBe(2);
    expect(rows.x[1]).toBeCloseTo(JUNCTION[2].x, 4);
  });

  test('says what changed: the heads placed, a lamp changed, or nothing', () => {
    const rows = new SignalRows();
    const snap = snapshot(JUNCTION);
    expect(rows.read(snap, layout)).toEqual({ placed: true, lit: true });
    // The same rows the next tick: nothing to draw again.
    expect(rows.read(snap, layout)).toEqual({ placed: false, lit: false });
    // Green to amber on the first arm: only the lamps.
    snap[BASE + F.state] = S.amber;
    expect(rows.read(snap, layout)).toEqual({ placed: false, lit: true });
    expect(rows.lamp[0]).toBe(AMBER);
    // Then red all round, the next arm green.
    snap[BASE + F.state] = S.red;
    snap[BASE + STRIDE + F.state] = S.green;
    expect(rows.read(snap, layout)).toEqual({ placed: false, lit: true });
    expect([...rows.lamp.subarray(0, 4)]).toEqual([RED, GREEN, RED, RED]);
    // A pole elsewhere (a new world), and a head fewer: placed again.
    snap[BASE + F.x] += 3;
    expect(rows.read(snap, layout).placed).toBe(true);
    snap[COUNT] = 3;
    expect(rows.read(snap, layout).placed).toBe(true);
    expect(rows.count).toBe(3);
    // Cleared (a world built again): the next read places them all.
    rows.clear();
    expect(rows.read(snap, layout)).toEqual({ placed: true, lit: true });
  });

  test('reads no heads from a layout without them, nor past the room for them', () => {
    const rows = new SignalRows();
    const old = { ...layout, light: undefined, lightBase: undefined, lightStride: undefined, header: { ...layout.header, lightCount: undefined } } as SnapshotLayout;
    expect(rows.read(snapshot(JUNCTION), old)).toEqual({ placed: false, lit: false });
    expect(rows.count).toBe(0);
    // A count past the snapshot's room (a stray value) reads only what there's room for.
    const snap = snapshot(JUNCTION);
    snap[COUNT] = 10_000;
    const few = new SignalRows(8);
    few.read(snap, layout);
    expect(few.count).toBeLessThanOrEqual(8);
    expect(few.count).toBe(4);
  });
});
