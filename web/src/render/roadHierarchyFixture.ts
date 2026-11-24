/**
 * The road hierarchy's fixture: a small world of every kind of road, the same layout as the simulation's
 * (`crates/sim-core/tests/sim/travel_hierarchy.rs` `hierarchy()`), road for road in the same order:
 *
 * - a motorway edge to edge along z = 200 to 209: two one-way carriageways 9 m apart, laid for traffic on the right as
 *   the game draws it (z north; eastbound the southern, z = 200, its points running east; westbound the northern, z = 209, its points running east
 *   too and one-way against them: each has the other on its left, and its ramps leave to its right), `raised` over a street (`BRIDGE_STREET`,
 *   north from the avenue to the world's edge) that it `crosses` without meeting, and a `bridge` over a river;
 * - a half-diamond west of the street: an off-ramp from the eastbound carriageway (south of it) and an on-ramp onto
 *   the westbound one (north of it), each merging into its carriageway at one end (`merge`) and meeting the street at a T 25 m from the motorway;
 * - an avenue along z = 80 with two cross streets;
 * - a one-way loop round a block, joined to the avenue's east end by a two-way street.
 *
 * Tests only, for now: nothing in the game lays it.
 */

import type { RoadEnd, RoadPlan } from '../core/protocol';

type Pt = [number, number];

/** The world's size (m). */
export const WIDTH = 480;
export const DEPTH = 320;
/** The carriageways' middles (z): eastbound, westbound (north is +z: eastbound's right, -z, is the south). */
export const EAST_Z = 200;
export const WEST_Z = 209;
/** The carriageways' ends (x): `s` along either is `x - MOTORWAY_X0`. */
export const MOTORWAY_X0 = 0.5;
export const MOTORWAY_X1 = 479.5;
/** The street under the motorway (x), from the avenue (z) to the world's edge. */
export const STREET_X = 140;
export const AVENUE_Z = 80;
/** The river: water from `RIVER_X` (x) on, `RIVER_W` wide, from `RIVER_Z0` (z) to the edge. */
export const RIVER_X = 330;
export const RIVER_W = 18;
export const RIVER_Z0 = 130;

/** The roads, by index (`roadHierarchy().roads`). */
export const MOTORWAY_EAST = 0;
export const MOTORWAY_WEST = 1;
export const BRIDGE_STREET = 2;
export const OFF_RAMP = 3;
export const ON_RAMP = 4;
export const AVENUE = 5;
export const CROSS_WEST = 6;
export const CROSS_EAST = 7;
export const LOOP_LINK = 8;
/** The one-way loop's four sides: west (cars go south), south (east), east (north) and north (west, its points running east). */
export const LOOP = [9, 10, 11, 12] as const;

/** A line from `a` to `b` with a point about every metre (as the simulation's tests' `hamlets::line`). */
function line(a: Pt, b: Pt): Pt[] {
  const n = Math.max(1, Math.ceil(Math.sqrt((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2)));
  const out: Pt[] = [];
  for (let i = 0; i <= n; i++) {
    const t = i / n;
    out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
  }
  return out;
}

/** A line through `pts` with a point about every metre, to the millimetre. */
function polyline(pts: readonly Pt[]): Pt[] {
  const mm = (v: number) => Math.round(v * 1000) / 1000;
  const out: Pt[] = [pts[0]];
  for (let i = 1; i < pts.length; i++) out.push(...line(pts[i - 1], pts[i]).slice(1));
  return out.map(([x, z]) => [mm(x), mm(z)]);
}

/** The fixture's roads and its river (the water's outline). */
export function roadHierarchy(): { width: number; depth: number; roads: RoadPlan[]; river: Pt[] } {
  const s = (x: number) => x - MOTORWAY_X0;
  const motorway = (z: number, oneway: 1 | -1, name: string): RoadPlan => ({
    points: polyline([
      [MOTORWAY_X0, z],
      [MOTORWAY_X1, z],
    ]),
    width: 9,
    ends: ['out', 'out'],
    kind: 'motorway',
    oneway,
    name,
    // (Over the street, a 30 m deck; over the river, 4 m past its banks.)
    raised: [[s(STREET_X - 15), s(STREET_X + 15)]],
    bridge: [s(RIVER_X - 4), s(RIVER_X + RIVER_W + 4)],
    crosses: [{ road: BRIDGE_STREET, s: s(STREET_X) }],
  });
  const street = (pts: Pt[], ends: [RoadEnd, RoadEnd], name: string): RoadPlan => ({ points: polyline(pts), width: 7, ends, kind: 'street', name });
  const oneWay = (pts: Pt[], oneway: 1 | -1, name: string): RoadPlan => ({ points: polyline(pts), width: 7, ends: ['join', 'join'], kind: 'street', oneway, name });
  const ramp = (pts: Pt[], ends: [RoadEnd, RoadEnd], name: string): RoadPlan => ({ points: polyline(pts), width: 5, ends, kind: 'ramp', oneway: 1, name });
  const bridgeStreet: RoadPlan = {
    ...street(
      [
        [STREET_X, AVENUE_Z],
        [STREET_X, DEPTH - 0.5],
      ],
      ['join', 'out'],
      'Bridge Street',
    ),
    crosses: [
      { road: MOTORWAY_EAST, s: EAST_Z - AVENUE_Z },
      { road: MOTORWAY_WEST, s: WEST_Z - AVENUE_Z },
    ],
  };
  const [x0, x1, y0, y1] = [300, 360, 20, 60];
  const roads: RoadPlan[] = [
    motorway(EAST_Z, 1, 'Motorway East'),
    motorway(WEST_Z, -1, 'Motorway West'),
    bridgeStreet,
    // (Off the eastbound carriageway's middle, out beside it, and up to the street 25 m short.)
    ramp(
      [
        [20, EAST_Z],
        [40, EAST_Z - 7],
        [110, EAST_Z - 7],
        [STREET_X, EAST_Z - 25],
      ],
      ['merge', 'join'],
      'Off-ramp',
    ),
    ramp(
      [
        [STREET_X, WEST_Z + 25],
        [110, WEST_Z + 7],
        [40, WEST_Z + 7],
        [20, WEST_Z],
      ],
      ['join', 'merge'],
      'On-ramp',
    ),
    {
      points: polyline([
        [0.5, AVENUE_Z],
        [x0, AVENUE_Z],
      ]),
      width: 14,
      ends: ['out', 'join'],
      kind: 'avenue',
      name: 'The Avenue',
    },
    street(
      [
        [90, 40],
        [90, 120],
      ],
      ['dead', 'dead'],
      'West Cross',
    ),
    street(
      [
        [210, 40],
        [210, 120],
      ],
      ['dead', 'dead'],
      'East Cross',
    ),
    street(
      [
        [x0, AVENUE_Z],
        [x0, y1],
      ],
      ['join', 'join'],
      'Loop Link',
    ),
    oneWay(
      [
        [x0, y1],
        [x0, y0],
      ],
      1,
      'Loop West',
    ),
    oneWay(
      [
        [x0, y0],
        [x1, y0],
      ],
      1,
      'Loop South',
    ),
    oneWay(
      [
        [x1, y0],
        [x1, y1],
      ],
      1,
      'Loop East',
    ),
    oneWay(
      [
        [x0, y1],
        [x1, y1],
      ],
      -1,
      'Loop North',
    ),
  ];
  const river: Pt[] = [
    [RIVER_X, RIVER_Z0],
    [RIVER_X + RIVER_W, RIVER_Z0],
    [RIVER_X + RIVER_W, DEPTH],
    [RIVER_X, DEPTH],
  ];
  return { width: WIDTH, depth: DEPTH, roads, river };
}
