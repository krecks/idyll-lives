/**
 * Hands on low tops: seated at a coffee table on the kids' chair (characters.ts picks `sitHandsLow` for a top
 * less than `LOW_TOP` over the seat, `tuckIn`), and kneeling at the train table (`STEP_IN`). Both in sizes,
 * as the street's (`street.ts`): the top's height and distance are fixed, so each size is fitted to a body of
 * its share `s` of the reference skeleton's size, the object's metres divided by `s` giving the clip's; leant
 * in as far as the hands need (`leanFor`).
 */

import { cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { kneelBody } from './garden';
import { bump, TAU } from './idle';
import { armIK, len, pointOn, sub, wrist } from './ik';
import { SEAT_MOVE, seatedBody } from './seated';
import { sizes, type Fit, type Size } from './street';

/** From the shoulder to the palm's centre, a little short of straight (m, reference skeleton). */
const ARM = 0.56;
/** The sizes' shares of the reference body's size (1.77 m, 1.47 m, 1.12 m). */
const SHARES = [1, 0.83, 0.63];

/** The least lean (deg, up to `most`) of `make`'s body that brings each of `goals` (side, palm place) within reach, all of it if they are out of reach. */
function leanFor(make: (lean: number) => PoseSpec, goals: readonly (readonly ['l' | 'r', Vec])[], most: number): number {
  const short = (lean: number) => {
    const b = make(lean);
    return Math.max(...goals.map(([s, g]) => len(sub(g, pointOn(b, `upperarm_${s}`, [0, 0, 0]))) - ARM));
  };
  if (short(0) <= 0) return 0;
  if (short(most) > 0) return most;
  let lo = 0;
  let hi = most;
  for (let k = 0; k < 14; k++) [lo, hi] = short((lo + hi) / 2) > 0 ? [(lo + hi) / 2, hi] : [lo, (lo + hi) / 2];
  return hi;
}

// --- seated at a low top -------------------------------------------------------------------------

/**
 * A top this low over the seat (m, for a body of the reference height: the desk-hands clips hold the hands
 * about 0.28 m over it) is reached with `sitHandsLow` (characters.ts). The one such: the kids' chair (0.31 m) at
 * the coffee table (0.42 m), its edge, with the chair pulled in until the knees meet it (`TUCK`: 0.56 m ahead of
 * the hips for a body of the reference height, and 0.02 m clear), and 0.11 m over the seat.
 */
export const LOW_TOP = 0.2;
const LOW = { over: 0.11, knees: 0.56, clear: 0.02, on: 0.15 };
type LowFit = Fit;
/** The seated clip's seat (m up; the runtime fits it to each body and seat) and its hips' place ahead (`SEAT_MOVE`). */
const SEAT_Y = 0.45;
const LOW_S = 5;
export const SIT_LOW: Size<LowFit>[] = sizes('sitHandsLow', SHARES, (s) => ({ s, to: 0 }));

/**
 * At a low top: bent well forward over it from the hips, the left hand flat on it holding the paper, the right
 * drawing (small strokes back and forth, now and then a longer one), the head down over them, now and then up
 * for a look round. The seat's pelvis is `sit`'s.
 */
function sitLowPose(fit: LowFit): (p: number) => PoseSpec {
  const { s } = fit;
  // The top's height and its edge's place in the clip's frame, the hands `on` m in from the edge.
  const y = SEAT_Y + (LOW.over + 0.02) / s;
  const z = SEAT_MOVE[2] + LOW.knees + (LOW.clear + LOW.on) / s;
  const right: Vec = [0.1, y, z];
  const left: Vec = [-0.14, y - 0.005, z - 0.02];
  let lean = -1;
  const make = (l: number, p = 0, head: Vec = [0, 0, 0]) => seatedBody({ breath: p, depth: 0.6, lean: l, tilt: -6 + 0.35 * l, head });
  return (p) => {
    if (lean < 0) lean = leanFor((l) => make(l), [['l', left], ['r', right]], 55);
    const look = bump(p, 0.62, 0.88, 0.1);
    const body = make(lean, p, [18 - 30 * look, 5 * look, 0]);
    // Drawing: small strokes, a longer sweep once a loop.
    const stroke = 0.03 * Math.sin(p * TAU * 15) * (1 - look);
    const sweep = 0.06 * bump(p, 0.2, 0.45, 0.1) * Math.sin(p * TAU * 2);
    return {
      ...body,
      ...armIK('l', left, body, { palm: true, pole: [-0.6, -0.8, -0.3], pronate: 85, hand: wrist('l', 10, 0) }),
      ...armIK('r', [right[0] + stroke + sweep, right[1] + 0.008 * (1 - look) * Math.abs(Math.sin(p * TAU * 15)), right[2] + 0.02 * Math.cos(p * TAU * 15)], body, {
        palm: true,
        pole: [0.6, -0.8, -0.3],
        pronate: 60,
        hand: wrist('r', -5, 0),
      }),
    };
  };
}

// --- kneeling at the train table ----------------------------------------------------------------

/**
 * The train table (`kids.py`): its top 0.44 m up (the track 0.012 m on it), its front edge 0.42 m out from the
 * middle. Kneeling, the knees `KNEE` m short of the edge; the knee joint is 0.123 m ahead of the feet's place
 * (the kneel's, reference skeleton), so `to` puts them there for each size. The track's ring runs round its
 * right half, 0.3 m out at its front (a lane there), the little town on the left half (the other lane).
 */
const TRAIN = { top: 0.44, track: 0.012, edge: 0.42, front: 0.3 };
const KNEE = 0.1;
type KneelFit = Fit;
export const KNEEL_TABLE: Size<KneelFit>[] = sizes('kneelTable', SHARES, (s) => ({ s, to: TRAIN.edge + KNEE + 0.123 * s }));
export const KNEEL_LANES = [0.42, -0.45] as const;
const KNEEL_S = 6;

/**
 * Kneeling up at the table's edge, leant in over it: the right hand pushing an engine along the track's front
 * and back, the left on the top beside it (now and then moving a tree or a house); the head down at them, once a
 * loop sitting back a little for a look along the table.
 */
function kneelTablePose(fit: KneelFit): (p: number) => PoseSpec {
  const { s, to } = fit;
  const o = (x: number, y: number, z: number): Vec => [x / s, y / s, (to - z) / s];
  // The engine's way: along the ring's front arc, 0.12 m either side of its middle.
  const engine = (x: number): Vec => o(x, TRAIN.top + TRAIN.track + 0.03, 0.02 + Math.sqrt(TRAIN.front * TRAIN.front - x * x) - 0.01);
  const left = o(-0.2, TRAIN.top + 0.025, TRAIN.edge - 0.1);
  let lean = -1;
  const make = (l: number, p = 0, twist = 0, head: Vec = [0, 0, 0]) => kneelBody(p, l, twist, head);
  return (p) => {
    if (lean < 0) lean = Math.max(10, leanFor((l) => make(l), [['l', left], ['r', engine(0.12)], ['r', engine(-0.12)]], 50));
    const back = bump(p, 0.7, 0.95, 0.08);
    const x = 0.12 * Math.sin(p * TAU * 2);
    const move = bump(p, 0.3, 0.5, 0.06);
    const body = make(lean - 14 * back, p, -6 * (x / 0.12) * (1 - back), [16 - 20 * back, -10 * (x / 0.12) - 14 * back, 0]);
    const r = engine(x);
    // Lifted off the track while sitting back, the left hand moving a tree further along and back.
    const rAt: Vec = [r[0], r[1] + 0.12 * back, r[2] - 0.15 * back];
    const lAt: Vec = [left[0] - 0.06 * move, left[1] + 0.05 * Math.sin(Math.PI * move), left[2] + 0.04 * move - 0.15 * back];
    return {
      ...body,
      ...armIK('l', lAt, body, { palm: true, pole: [-0.6, -0.8, -0.3], pronate: 80, hand: wrist('l', 10, 0) }),
      ...armIK('r', rAt, body, { palm: true, pole: [0.6, -0.8, -0.3], pronate: 80, hand: wrist('r', 5, 0) }),
    };
  };
}

export function lowClips(): ClipDef[] {
  return [
    ...SIT_LOW.map(({ clip, fit }) => cycle(clip, { seconds: LOW_S, loop: true }, sitLowPose(fit))),
    ...KNEEL_TABLE.map(({ clip, fit }) => cycle(clip, { seconds: KNEEL_S, loop: true }, kneelTablePose(fit))),
  ];
}
