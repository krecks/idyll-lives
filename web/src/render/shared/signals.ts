/**
 * The traffic lights' signal heads as the snapshot has them (sim-core `traffic::Signal`, the
 * snapshot's `light` rows: one per arm of each lit junction, the same rows from tick to tick): where
 * each pole stands on the kerb, which way its head faces (a resident's yaw: towards those coming in
 * along its arm), which lamp it shows, and whether its pedestrians' head shows walk (the junction's
 * walk phase: every car head red, every crossing walk). Read into arrays made once (`SignalRows.read`), so the
 * renderer (`pc/signals.ts`) reads them every frame without allocating, and knows whether the poles
 * moved (a new world) or only the lamps changed.
 */

import type { WorldStructure } from '../../core/protocol';
import type { SnapshotLayout } from '../../core/snapshot';
import { Geo, type V3 } from './geometry';
import { hasSidewalks } from './junctions';
import { townRoads } from './roads';

/** The lamp a head shows: its lens, top to bottom (`SIGNAL_LAMPS`). */
export const RED = 0;
export const AMBER = 1;
export const GREEN = 2;
export type Lamp = typeof RED | typeof AMBER | typeof GREEN;

/** The lenses top to bottom, as the model's lens materials are named (`lens_<name>`, tools/art/blender/families/street_furniture.py). */
export const SIGNAL_LAMPS = ['red', 'amber', 'green'] as const;

/** Signal heads read at most (the snapshot's room, sim-core `snapshot::MAX_LIGHTS`, when the layout doesn't say). */
export const MOST_SIGNALS = 256;

/** What changed in the rows since the last read. */
export interface SignalChange {
  /** Heads came or went, or one stands elsewhere (a new world). */
  placed: boolean;
  /** A head shows another lamp. */
  lit: boolean;
}

/** The heads to be seen in the newest snapshot (rows not shown are left out), read again each frame. */
export class SignalRows {
  /** Heads read. */
  count = 0;
  readonly x: Float32Array;
  readonly z: Float32Array;
  readonly yaw: Float32Array;
  /** Each head's `Lamp`. */
  readonly lamp: Uint8Array;
  /** Each head's pedestrians' lamp: 1 walk, 0 don't walk (`walkOf`). */
  readonly walk: Uint8Array;
  /** Each head's row in the snapshot (`WorldStructure.junctions` arms' `light`: rows not shown are left out). */
  readonly row: Uint16Array;
  private readonly change: SignalChange = { placed: false, lit: false };

  constructor(readonly most = MOST_SIGNALS) {
    this.x = new Float32Array(most);
    this.z = new Float32Array(most);
    this.yaw = new Float32Array(most);
    this.lamp = new Uint8Array(most);
    this.walk = new Uint8Array(most);
    this.row = new Uint16Array(most);
  }

  /**
   * The heads in `snap` (laid out as `layout`; an older layout without them: none). What changed
   * since the last read (the same object each time: read it before the next).
   */
  read(snap: ArrayLike<number>, layout: SnapshotLayout): SignalChange {
    const change = this.change;
    change.placed = false;
    change.lit = false;
    const f = layout.light;
    const base = layout.lightBase;
    const stride = layout.lightStride;
    const at = layout.header.lightCount;
    const room = Math.min(this.most, layout.maxLights ?? MOST_SIGNALS);
    const rows = !f || base === undefined || stride === undefined || at === undefined ? 0 : Math.max(0, Math.min(room, snap[at] | 0));
    let n = 0;
    for (let k = 0; k < rows; k++) {
      const o = base! + k * stride!;
      if (!snap[o + f!.shown]) continue;
      const x = snap[o + f!.x];
      const z = snap[o + f!.z];
      const yaw = snap[o + f!.yaw];
      const state = snap[o + f!.state];
      const lamp = lampOf(state, f!.states);
      const walk = walkOf(state, f!.states) ? 1 : 0;
      // (Compared as stored: a Float32Array's own values.)
      if (n >= this.count || this.x[n] !== Math.fround(x) || this.z[n] !== Math.fround(z) || this.yaw[n] !== Math.fround(yaw)) change.placed = true;
      else if (this.lamp[n] !== lamp || this.walk[n] !== walk) change.lit = true;
      this.x[n] = x;
      this.z[n] = z;
      this.yaw[n] = yaw;
      this.lamp[n] = lamp;
      this.walk[n] = walk;
      if (this.row[n] !== k) change.placed = true;
      this.row[n] = k;
      n++;
    }
    if (n !== this.count) change.placed = true;
    this.count = n;
    // (Placed again, every head's lamp is drawn again with it.)
    if (change.placed) change.lit = true;
    return change;
  }

  /** None read (a new world: the next read places them all). */
  clear(): void {
    this.count = 0;
  }
}

/**
 * Per snapshot row (`SnapshotLayout.light`), whether its head's arm has a crosswalk, so its pole
 * gets the pedestrians' head: an arm into a street or a lane, as its zebra is drawn (`roads.ts`
 * `junctionMarks`), not a country road's (no sidewalks to cross between) nor a roundabout's (no walk
 * phase). Null: the structure doesn't say which arm a row is (one from before `junctions`): every
 * head has it.
 */
export function crosswalkRows(world: WorldStructure): Uint8Array | null {
  const list = world.junctions;
  if (!list) return null;
  const roads = townRoads(world);
  let rows = 0;
  for (const j of list) for (const a of j.arms) if (a.light !== null) rows = Math.max(rows, a.light + 1);
  const out = new Uint8Array(rows);
  for (const j of list) {
    if (j.roundabout) continue;
    for (const a of j.arms) if (a.light !== null && hasSidewalks(roads[a.road])) out[a.light] = 1;
  }
  return out;
}

/** A row's `state` values (`SnapshotLayout.light.states`). */
type States = { red: number; amber: number; green: number; walk: number };

/**
 * The lamp a row's `state` shows to traffic: amber or green as they are; red otherwise: while those
 * on foot cross (walk: every head of the junction red), and for a value this renderer doesn't know.
 */
export function lampOf(state: number, states: States): Lamp {
  if (state === states.green) return GREEN;
  if (state === states.amber) return AMBER;
  return RED;
}

/** Whether a row's `state` shows walk to those on foot (the junction's walk phase); don't walk otherwise. */
export function walkOf(state: number, states: States): boolean {
  return state === states.walk;
}

/**
 * A pole's pedestrians' head (local to the pole as `model.trafficLight`: its axis the origin, its
 * traffic head facing +z): a small housing on the pole below the traffic head, facing across the
 * road (-x: the pole stands on the kerb to the right of those coming in), two lenses one over the
 * other, and the figures that glow on them: don't walk (red, standing) above, walk (green,
 * striding) below. The housing in the plain finish (vertex alpha: its roughness and metalness), the
 * figures in the glow's colours.
 */
export interface PedestrianHead {
  housing: Geo;
  dontWalk: Geo;
  walk: Geo;
}

/** The pedestrians' head's front (m out from the pole's axis, towards -x), its lenses' middles (m up) and half their size. */
const PED_FRONT = 0.26;
const PED_LENSES = { dontWalk: 1.79, walk: 1.55 } as const;
const PED_LENS = 0.1;
/** The glow's colours, as the traffic head's lenses (tools/art/blender/families/street_furniture.py). */
const PED_RED = '#FF4A2E';
const PED_GREEN = '#1FE58A';

let pedHead: PedestrianHead | null = null;

/** The pedestrians' head's geometry (made once). */
export function pedestrianHead(): PedestrianHead {
  if (pedHead) return pedHead;
  const housing = new Geo();
  // The housing (cast iron, as the pole: metal) on a bracket round the pole; the dark backs of the lenses.
  housing.color('#2B302E', 0.75);
  housing.box(-0.08, 1.71, -0.03, -0.03, 1.75, 0.03);
  housing.box(-0.08, 1.47, -0.03, -0.03, 1.51, 0.03);
  housing.box(-PED_FRONT + 0.005, 1.42, -0.13, -0.08, 1.92, 0.13);
  // Visors over each lens, open below.
  for (const y of Object.values(PED_LENSES)) housing.box(-PED_FRONT - 0.09, y + PED_LENS + 0.005, -0.12, -PED_FRONT + 0.005, y + PED_LENS + 0.02, 0.12);
  housing.color('#171A19', 0.45);
  for (const y of Object.values(PED_LENSES)) housing.poly(lens(y, 0), [-1, 0, 0]);
  // The figures, a little in front of their lens: standing (a hand's stop), striding.
  const dontWalk = new Geo().color(PED_RED);
  figure(dontWalk, PED_LENSES.dontWalk, false);
  const walk = new Geo().color(PED_GREEN);
  figure(walk, PED_LENSES.walk, true);
  pedHead = { housing, dontWalk, walk };
  return pedHead;
}

/** A lens's square at height `y`, `out` m in front of the housing. */
function lens(y: number, out: number): V3[] {
  const x = -PED_FRONT - out;
  return [
    [x, y - PED_LENS, -PED_LENS],
    [x, y - PED_LENS, PED_LENS],
    [x, y + PED_LENS, PED_LENS],
    [x, y + PED_LENS, -PED_LENS],
  ];
}

/** A little figure on the lens at height `y` (its across is z, its up y): standing still, or striding. */
function figure(g: Geo, y: number, striding: boolean): void {
  const x = -PED_FRONT - 0.004;
  const n: V3 = [-1, 0, 0];
  /** A quad from (z0, y0)–(z1, y0) at its foot to (z2, y1)–(z3, y1) at its top. */
  const quad = (z0: number, z1: number, y0: number, z2: number, z3: number, y1: number) =>
    g.poly([[x, y + y0, z0], [x, y + y0, z1], [x, y + y1, z3], [x, y + y1, z2]], n);
  // The head.
  quad(-0.012, 0.012, 0.05, -0.012, 0.012, 0.074);
  if (striding) {
    quad(-0.016, 0.014, -0.005, -0.008, 0.02, 0.045);
    quad(-0.016, -0.002, -0.005, -0.05, -0.034, -0.075);
    quad(0.0, 0.014, -0.005, 0.034, 0.048, -0.075);
    quad(-0.012, -0.002, 0.04, -0.04, -0.03, 0.0);
    quad(0.008, 0.018, 0.04, 0.036, 0.046, 0.005);
  } else {
    quad(-0.018, 0.018, -0.005, -0.018, 0.018, 0.045);
    quad(-0.016, -0.004, -0.075, -0.016, -0.004, -0.005);
    quad(0.004, 0.016, -0.075, 0.004, 0.016, -0.005);
    quad(-0.03, -0.02, 0.0, -0.03, -0.02, 0.042);
    quad(0.02, 0.03, 0.0, 0.02, 0.03, 0.042);
  }
}
