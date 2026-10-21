/**
 * Games and sport at things (characters.ts `ACTIONS.play`, `ACTIONS.exercise` and `STEP_IN` pick them by
 * object): darts thrown at the board, a pool shot bent over the rail, a foosball table's rods turned, a
 * table-tennis rally, plates smashed into the bin, a bouldering problem climbed on the wall and a yoga flow on
 * the mat. Things at fixed heights (the pool table's cloth, the foosball rods, the plate stack, the wall's
 * holds) have their clips in sizes, as the climbing frame's (`street.ts`): each is fitted to a body of its
 * share `s` of the reference skeleton's size, standing `to` m from the object's middle; the object's numbers
 * in metres divided by `s` give the clip's. The darts, the rally and the yoga are body-relative: one size.
 *
 * The cue and the bats aren't in the hands yet (props in hand, a later item): the hands mime them, closed.
 */

import { cycle, type ClipDef, type PoseSpec, type Rot, type Vec } from './dsl';
import { breathe, hangArms, TAU } from './idle';
import { addV, armIK, footIK, lerpV, norm, pointOn, smooth, sub, wrist, type ArmOptions } from './ik';
import { sizes, type Fit, type Size } from './street';

// --- tracks: values keyed in time ---------------------------------------------------------------

type Ease = 'smooth' | 'out' | 'in' | 'linear';
/** A key: at time `t` the value `v`, eased into from the key before. */
type Key<T> = readonly [t: number, v: T, ease?: Ease];

function eased(e: Ease | undefined, u: number): number {
  return e === 'linear' ? u : e === 'out' ? 1 - (1 - u) * (1 - u) : e === 'in' ? u * u : smooth(u);
}

/** The value of `keys` (in time order) at `t`: eased from key to key, held before the first and after the last. */
function track<T extends number | Vec>(keys: readonly Key<T>[], t: number): T {
  if (t <= keys[0][0]) return keys[0][1];
  for (let i = 1; i < keys.length; i++) {
    const [t1, b, e] = keys[i];
    if (t > t1) continue;
    const [t0, a] = keys[i - 1];
    const u = eased(e, t1 > t0 ? (t - t0) / (t1 - t0) : 1);
    return (typeof a === 'number' ? a + ((b as number) - a) * u : lerpV(a as Vec, b as Vec, u)) as T;
  }
  return keys[keys.length - 1][1];
}

/** A foot (or a hand) going from place to place (`keys`): as `track`, lifted clear by `lift` (m, at the middle of each move) between two places. */
function moved(keys: readonly Key<Vec>[], t: number, lift: Vec): Vec {
  const at = track(keys, t);
  for (let i = 1; i < keys.length; i++) {
    const [t0, a] = keys[i - 1];
    const [t1, b] = keys[i];
    if (t <= t0 || t >= t1 || Math.hypot(b[0] - a[0], b[1] - a[1], b[2] - a[2]) < 1e-3) continue;
    const k = Math.sin((Math.PI * (t - t0)) / (t1 - t0));
    return addV(at, lift, k);
  }
  return at;
}

// --- the body -----------------------------------------------------------------------------------

interface Body {
  /** The pelvis moved from rest (m), pitched forward `hip` and turned `yaw` (deg, + to the right). */
  at?: Vec;
  hip?: number;
  yaw?: number;
  /** The back bent forward `bend`, turned `twist` (+ to the right) and bent `side` (+ to the left), deg, spread over the spine. */
  bend?: number;
  twist?: number;
  side?: number;
  /** The head on the chest: nod (+ down), turn (+ right), tilt (deg). */
  head?: Vec;
  breath?: number;
  /** The ankles from their rest places (m), the feet's world turn (DSL Euler: + toes down, + toes right) and where the knees point. */
  l?: Vec;
  r?: Vec;
  fl?: Rot;
  fr?: Rot;
  kl?: Vec;
  kr?: Vec;
}

/** A body pose: pelvis, spine and head, the feet put where `o` says (leg IK). It breathes in the chest only: the pelvis keeps still over planted feet. */
function body(o: Body): PoseSpec {
  const b = breathe(o.breath ?? 0, 0.7);
  const bend = o.bend ?? 0;
  const tw = o.twist ?? 0;
  const sd = o.side ?? 0;
  const h = o.head ?? [0, 0, 0];
  const at = o.at ?? [0, 0, 0];
  const spec: PoseSpec = {
    move: at,
    pelvis: [o.hip ?? 0, o.yaw ?? 0, 0],
    spine_01: [bend * 0.35, tw * 0.25, sd * 0.3],
    spine_02: [bend * 0.35 + (b.spine_02?.[0] ?? 0), tw * 0.35, sd * 0.35],
    spine_03: [bend * 0.3 + (b.spine_03?.[0] ?? 0), tw * 0.4, sd * 0.35],
    clavicle_l: b.clavicle,
    clavicle_r: [0, 0, -(b.clavicle?.[2] ?? 0)],
    neck_01: [h[0] * 0.4, h[1] * 0.35, h[2] * 0.3],
    Head: [h[0] * 0.6, h[1] * 0.65, h[2] * 0.7],
  };
  return {
    ...spec,
    ...footIK('l', o.l ?? [0, 0, 0], spec, { foot: o.fl ?? [0, -4, 0], pole: o.kl }),
    ...footIK('r', o.r ?? [0, 0, 0], spec, { foot: o.fr ?? [0, 4, 0], pole: o.kr }),
  };
}

type Hand = ArmOptions & { at: Vec };

/** The arms placed by IK on `b` (a side null: hanging). */
function arms(b: PoseSpec, l: Hand | null, r: Hand | null): PoseSpec {
  return { ...b, ...hangArms(), ...(l ? armIK('l', l.at, b, l) : {}), ...(r ? armIK('r', r.at, b, r) : {}) };
}

/** A hand held open (the clip's own fingers: the rest pose, not `GRIP`). */
function open(side: 'l' | 'r'): PoseSpec {
  const z: Rot = [0, 0, 0];
  return { [`fingers_01_${side}`]: z, [`fingers_02_${side}`]: z, [`thumb_01_${side}`]: [0, 0, 6], [`thumb_02_${side}`]: z } as PoseSpec;
}

/** The ankle raised on tiptoe by `tip` m (the heel up about the ball of the foot, 0.17 m ahead), and the foot's pitch for it (deg). */
function tiptoe(tip: number): [Vec, number] {
  const a = Math.asin(Math.min(0.9, tip / 0.17));
  return [[0, tip, -0.17 * (1 - Math.cos(a))], (a * 180) / Math.PI];
}

// --- darts --------------------------------------------------------------------------------------

/**
 * Darts (`dartboard`): side-on at the line, the right foot forward and the weight on it, the dart up by
 * the eye; three throws (aim, the forearm back, through and let go, the hand following to the board),
 * then a step back off the line to read the score, and in again. The board's three darts grow in with
 * the first throw (its `darts` clip, 0.6 s from the use's start).
 */
const DART_S = 7.2;
const THROWS = [0.1, 1.85, 3.6];
const AIM: Vec = [0.12, 1.6, 0.31];
const DRAW: Vec = [0.14, 1.63, 0.15];
const LET_GO: Vec = [0.12, 1.57, 0.62];
const FOLLOW: Vec = [0.12, 1.47, 0.62];
const LOW: Vec = [0.17, 1.0, 0.16];
const DART_HAND: Key<Vec>[] = [[0, AIM]];
const DART_FLEX: Key<number>[] = [[0, -35]];
for (const [i, t0] of THROWS.entries()) {
  DART_HAND.push([t0 + 0.12, addV(AIM, [0, 0, 0.025])], [t0 + 0.28, DRAW], [t0 + 0.4, LET_GO, 'out'], [t0 + 0.6, FOLLOW], [t0 + 0.85, FOLLOW]);
  DART_FLEX.push([t0 + 0.28, -45], [t0 + 0.4, 10, 'out'], [t0 + 0.6, 30], [t0 + 0.85, 30]);
  if (i < THROWS.length - 1) {
    DART_HAND.push([t0 + 1.45, AIM]);
    DART_FLEX.push([t0 + 1.45, -35]);
  }
}
DART_HAND.push([5.0, LOW], [6.75, LOW], [DART_S, AIM]);
DART_FLEX.push([5.0, 10], [6.75, 10], [DART_S, -35]);
/** The feet: the right on the line, back a step to read the score, and in again. */
const ON_LINE: Vec = [-0.02, 0, 0.14];
const STEPPED: Vec = [0.02, 0, -0.1];
const DART_R: Key<Vec>[] = [[0, ON_LINE], [5.05, ON_LINE], [5.5, STEPPED], [6.3, STEPPED], [6.75, ON_LINE]];
const DART_L: Vec = [-0.06, 0, -0.16];
/** The pelvis: over the front foot throwing, over the back one while the front steps. */
const DART_AT: Key<Vec>[] = [
  [0, [0.02, -0.012, 0.05]],
  [4.6, [0.02, -0.012, 0.05]],
  [5.05, [-0.06, -0.008, -0.1]],
  [5.5, [-0.03, -0.004, -0.12]],
  [6.3, [-0.06, -0.008, -0.11]],
  [6.75, [0.02, -0.012, 0.05]],
];
const DART_YAW: Key<number>[] = [[0, -30], [4.6, -30], [5.5, -14], [6.3, -14], [6.75, -30]];

function dartPose(p: number): PoseSpec {
  const t = p * DART_S;
  const yaw = track(DART_YAW, t);
  // Leaning in a little more as each dart goes.
  const lean = 4 + THROWS.reduce((n, t0) => n + 5 * Math.max(0, 1 - Math.abs(t - t0 - 0.42) / 0.3), 0);
  const twist = -8;
  const b = body({ breath: p * 2, at: track(DART_AT, t), yaw, twist, bend: lean, head: [-6 - lean * 0.5, -(yaw + twist), 0], l: DART_L, fl: [0, -55, 0], r: moved(DART_R, t, [0, 0.06, 0]), fr: [0, -15, 0] });
  return arms(
    b,
    { at: [-0.03, 1.1, 0.25], palm: true, pole: [-0.6, -1, -0.1], pronate: 30, hand: wrist('l', 20, 0) },
    { at: track(DART_HAND, t), palm: true, pole: [0.1, -0.6, 0.8], pronate: 0, hand: wrist('r', track(DART_FLEX, t), 0) },
  );
}

// --- pool ---------------------------------------------------------------------------------------

/**
 * The pool table (`model.poolTable`): the cloth 0.785 m up, the long rails' outer faces 0.7 m out from the
 * middle. A shot from the long side: bent over the rail (as far as the body's height needs, the chin a hand
 * over the cue), the left hand bridging on the cloth, the right forearm hanging from the elbow; two practice
 * strokes, a slow back swing and the stroke through, then up to watch the balls and chalk the cue.
 */
type PoolFit = Fit;
export const POOL: Size<PoolFit>[] = sizes('poolShot', [0.97, 0.83, 0.63], (s) => ({ s, to: 0.72 + 0.32 * s }));
const BED = 0.785;
const POOL_S = 8;
/** The stroke (m along the cue, + through): two practice strokes, the back swing, the shot. */
const STROKE: Key<number>[] = [
  [0, 0],
  [1.2, 0],
  [1.55, -0.1],
  [1.9, 0.05],
  [2.25, -0.1],
  [2.6, 0.03],
  [2.9, 0],
  [3.3, -0.15],
  [3.45, 0.17, 'out'],
  [4.2, 0.17],
  [4.9, 0.1],
];
/** How far down into the stance (0 up, 1 down over the cue). */
const DOWN: Key<number>[] = [[0, 0], [1.2, 1], [4.2, 1], [5.0, 0]];

function poolPose(fit: PoolFit): (p: number) => PoseSpec {
  const { s, to } = fit;
  const bridge: Vec = [-0.02, (BED + 0.025) / s, (to - (0.62 - 0.2 * s)) / s];
  const goal = (BED + 0.27) / s;
  const stance = (lean: number, down: number, p: number): PoseSpec =>
    body({
      breath: p * 3,
      at: [0.02, -0.03 * down, -0.1 * Math.sin((lean * Math.PI) / 180)],
      hip: 0.5 * lean,
      bend: 0.5 * lean + 3,
      yaw: -28,
      twist: 10 * down,
      head: [-0.8 * lean + 6, 18 + 6 * down, 0],
      l: [-0.04, 0, 0.1],
      fl: [0, -22, 0],
      r: [0.1, 0, -0.22],
      fr: [0, 40, 0],
    });
  // How far over the cue (found on the first frame baked: the reference skeleton is there only then).
  let lean = -1;
  const solve = () => {
    lean = 20;
    for (let k = 0; k < 40; k++) lean = Math.min(72, Math.max(0, lean + (pointOn(stance(lean, 1, 0), 'Head', [0, 0, 0])[1] - goal) * 100));
  };
  return (p) => {
    if (lean < 0) solve();
    const t = p * POOL_S;
    const down = track(DOWN, t);
    const b = stance(4 + (lean - 4) * down, down, p);
    // The cue: from the grip by the right hip down to the bridge; the grip moves along it.
    const grip0: Vec = [0.13, bridge[1] + 0.16, bridge[2] - 0.78];
    const grip = addV(grip0, norm(sub(bridge, grip0)), track(STROKE, t));
    // Up: chalking the cue held upright in the left hand.
    const c = t * TAU * 1.5;
    const chalkL: Vec = [-0.04, 1.18, 0.26];
    const chalkR: Vec = [-0.01 + 0.02 * Math.cos(c), 1.34 + 0.01 * Math.sin(c), 0.27];
    const u = smooth(down);
    return {
      ...arms(
        b,
        { at: lerpV(chalkL, bridge, u), palm: true, pole: [-1, -0.4, -0.1], pronate: 10 + 80 * u, hand: wrist('l', 10 - 40 * u, 0) },
        { at: lerpV(chalkR, grip, u), palm: true, pole: lerpV([0.6, -1, -0.2], [0.3, 1, -0.6], u), pronate: 20, hand: wrist('r', 10, 0) },
      ),
      ...open('l'),
    };
  };
}

// --- foosball -----------------------------------------------------------------------------------

/**
 * The foosball table (`model.foosball`): the rods' handles 0.84 m up, at 0.46 m out from the middle, rods at
 * 0.45 and 0.15 m either side of it; a player at a pair of them (`STEP_IN` lanes 0.3 m either side). Hands
 * round the two handles, the wrists turning the rods (the right snapping kicks in time with the table's
 * moving rods, 0.8 s a round; the left blocking), the rods slid in and out; the head over the game.
 */
type FoosFit = Fit;
export const FOOS: Size<FoosFit>[] = sizes('foosball', [0.97, 0.83, 0.63], (s) => ({ s, to: 0.47 + 0.33 * s }));
export const FOOS_LANES = [-0.3, 0.3] as const;
const FOOS_S = 3.2;

function foosPose(fit: FoosFit): (p: number) => PoseSpec {
  const { s, to } = fit;
  const hy = 0.84 / s;
  const hz = (to - 0.47) / s;
  const hx = 0.15 / s;
  return (p) => {
    const t = p * FOOS_S;
    const kick = Math.sin((t / 0.8) * TAU);
    const block = Math.sin((t / 1.6) * TAU + 1);
    const follow = Math.sin(p * TAU);
    const b = body({ breath: p * 2, at: [0.015 * follow, -0.035, 0.02], bend: 14 + 2 * Math.abs(kick), twist: 4 * follow, head: [28, 6 * follow, 0], l: [-0.05, 0, 0], r: [0.05, 0, 0] });
    return arms(
      b,
      { at: [-hx, hy, hz - 0.025 * Math.sin(p * TAU * 2)], palm: true, pole: [-0.7, -0.6, -0.4], pronate: 70 + 30 * block, hand: wrist('l', 15, 0) },
      { at: [hx, hy, hz + 0.03 * Math.sin(p * TAU * 2 + 1)], palm: true, pole: [0.7, -0.6, -0.4], pronate: 70 + 45 * kick, hand: wrist('r', 15, 0) },
    );
  };
}

// --- table tennis -------------------------------------------------------------------------------

/**
 * A table-tennis rally (`pingPongTable`): the ready stance (knees bent, leant in, the bat hand in front), a
 * forehand (the chest turning back to the right, through and up across) and a backhand (the bat in front of
 * the belly, out to the right), the weight going with each. The bat is mimed in a closed hand.
 */
const PONG_S = 1.8;
const READY: Vec = [0.16, 1.0, 0.38];
const PONG_HAND: Key<Vec>[] = [
  [0, READY],
  [0.3, [0.44, 0.93, 0.04]],
  [0.42, [0.06, 1.2, 0.54], 'out'],
  [0.55, [-0.06, 1.3, 0.5]],
  [0.9, READY],
  [1.15, [-0.06, 0.98, 0.2]],
  [1.27, [0.25, 1.12, 0.5], 'out'],
  [1.4, [0.35, 1.18, 0.42]],
  [PONG_S, READY],
];
const PONG_TWIST: Key<number>[] = [[0, 0], [0.3, 32], [0.42, -14, 'out'], [0.55, -22], [0.9, 0], [1.15, -14], [1.27, 8, 'out'], [1.4, 14], [PONG_S, 0]];
const PONG_PRON: Key<number>[] = [[0, 20], [0.3, -30], [0.55, 10], [0.9, 20], [1.15, 75], [1.4, 60], [PONG_S, 20]];
const PONG_FLEX: Key<number>[] = [[0, 0], [0.3, -25], [0.42, 15], [0.9, 0], [1.15, 40], [1.27, -10], [PONG_S, 0]];
const PONG_SHIFT: Key<number>[] = [[0, 0], [0.3, 0.04], [0.5, -0.02], [0.9, 0], [1.15, -0.02], [1.35, 0.03], [PONG_S, 0]];

function pongPose(p: number): PoseSpec {
  const t = p * PONG_S;
  const tw = track(PONG_TWIST, t);
  const shift = track(PONG_SHIFT, t);
  const b = body({ breath: p, at: [shift, -0.07, 0.03], hip: 10, bend: 16, twist: tw, head: [12, -0.8 * tw, 0], l: [-0.09, 0, 0.03], fl: [0, -12, 0], r: [0.09, 0, -0.03], fr: [0, 12, 0] });
  return arms(
    b,
    { at: [-0.22 - 0.4 * shift, 1.02, 0.28 - 0.003 * tw], palm: true, pole: [-0.6, -1, -0.2], pronate: 20, hand: wrist('l', 10, 0) },
    { at: track(PONG_HAND, t), palm: true, pole: [0.7, -1, -0.3], pronate: track(PONG_PRON, t), hand: wrist('r', track(PONG_FLEX, t), 0) },
  );
}

// --- smashing plates ----------------------------------------------------------------------------

/**
 * The plate-smash bin (`model.hotHeaded.smashBin`): the crate 0.6 m high, its front 0.09 m out from the
 * middle and 0.08 m to the right of it (as the user faces it), the plates stacked to 0.58 m on the stool 0.28 m
 * to their left, 0.25 m out. Turned and bent to the stack, a plate taken in both hands, up and back over the
 * right shoulder, and thrown down into the crate overarm; a look in after it. The plate is mimed.
 */
type SmashFit = Fit;
export const SMASH: Size<SmashFit>[] = sizes('smashPlate', [0.97, 0.83, 0.63], (s) => ({ s, to: 0.48 + 0.15 * s }));
const SMASH_S = 3.4;

function smashPose(fit: SmashFit): (p: number) => PoseSpec {
  const { s, to } = fit;
  const plate: Vec = [-0.28 / s, 0.6 / s, (to - 0.25) / s];
  const crate: Vec = [0.08 / s, 0.62 / s, (to - 0.09) / s];
  // Bent to the stack as far as its height needs.
  const reach = Math.min(1, Math.max(0, (1.1 - plate[1]) / 0.5));
  const LOWER: Vec = [0.18, 0.98, 0.14];
  const HAND: Key<Vec>[] = [
    [0, LOWER],
    [0.75, addV(plate, [0.05, 0.03, 0])],
    [0.95, addV(plate, [0.05, 0.03, 0])],
    [1.5, [0.27, 1.72, -0.1]],
    [1.72, [crate[0] - 0.02, crate[1] + 0.35, crate[2] + 0.02], 'out'],
    [2.3, [crate[0] - 0.02, crate[1] + 0.3, crate[2]]],
    [SMASH_S, LOWER],
  ];
  const OTHER: Key<Vec>[] = [
    [0, [-0.18, 0.98, 0.14]],
    [0.75, addV(plate, [-0.07, 0.02, -0.02])],
    [0.95, addV(plate, [-0.07, 0.02, -0.02])],
    [1.5, [-0.1, 1.3, 0.42]],
    [1.72, [-0.24, 1.02, 0.02]],
    [2.3, [-0.22, 1.0, 0.06]],
    [SMASH_S, [-0.18, 0.98, 0.14]],
  ];
  const BEND: Key<number>[] = [[0, 2], [0.75, 8 + 40 * reach], [0.95, 8 + 40 * reach], [1.5, -6], [1.72, 26, 'out'], [2.3, 28], [SMASH_S, 2]];
  const TWIST: Key<number>[] = [[0, 0], [0.75, -26], [0.95, -26], [1.5, 30], [1.72, -16, 'out'], [2.3, -6], [SMASH_S, 0]];
  const SINK: Key<number>[] = [[0, 0], [0.75, 0.1 * reach], [0.95, 0.1 * reach], [1.5, 0], [1.72, 0.03], [2.3, 0.03], [SMASH_S, 0]];
  return (p) => {
    const t = p * SMASH_S;
    const bend = track(BEND, t);
    const tw = track(TWIST, t);
    const b = body({ breath: p, at: [-0.03 * Math.max(0, -tw / 26), -track(SINK, t), 0.02 - 0.08 * Math.max(0, (bend - 10) / 40)], hip: 0.4 * Math.max(0, bend), bend: 0.6 * bend, yaw: 0.15 * tw, twist: 0.85 * tw, head: [8 + 0.3 * bend, -0.6 * tw, 0], l: [-0.04, 0, 0.06], fl: [0, -14, 0], r: [0.05, 0, -0.08], fr: [0, 14, 0] });
    return arms(
      b,
      { at: track(OTHER, t), palm: true, pole: [-0.6, -1, -0.2], pronate: 40, hand: wrist('l', 10, 0) },
      { at: track(HAND, t), palm: true, pole: [0.8, -0.5, -0.4], pronate: 30, hand: wrist('r', 10, 0) },
    );
  };
}

// --- bouldering ---------------------------------------------------------------------------------

/**
 * The home bouldering wall (energetic pack, `model.energetic.boulderingWall`): a plywood panel from 0.02 m up
 * at 0.38 m behind the middle, leaning 12 degrees over the climber, its holds (`HOLDS`, along the panel from
 * its foot), and the crash mat in front 0.22 m high, where the climber stands (`STEP_IN` up). A problem up the
 * middle: hands to the start holds, feet onto the low ones, then a foot, a hand, a foot and a hand to the top, a look down, and a drop to the mat, landing on bent knees; up, and a shake of the arms. The route is
 * made for the reference body and scaled to each size, each place moved onto the nearest real hold in reach. The
 * wall is short (2.3 m), so the climb is too: from the start holds over the head to the top, the feet up to
 * about 0.75 m.
 */
type BoulderFit = Fit;
export const BOULDER: Size<BoulderFit>[] = sizes('boulder', [0.97, 0.83, 0.63], (s) => ({ s, to: 0.12 + 0.05 * s, up: 0.22 }));
const WALL = { lean: (12 * Math.PI) / 180, y: 0.02, z: -0.38 };
/** The holds: (m to the wall's right of its middle, m up the panel from its foot), as the model has them. */
const HOLDS: readonly (readonly [number, number])[] = [
  [-0.62, 0.35], [-0.18, 0.42], [0.35, 0.3], [0.7, 0.55], [-0.45, 0.75], [0.05, 0.8], [0.52, 0.95], [-0.75, 1.1], [-0.25, 1.2],
  [0.28, 1.3], [0.72, 1.42], [-0.55, 1.55], [0.0, 1.62], [0.45, 1.75], [-0.3, 1.95], [0.2, 2.05], [0.66, 2.1], [-0.7, 1.9],
];
/** The crash mat's top (m). */
const CRASH = 0.22;
/** A point `up` m up the panel from its foot and `out` m out of it, `x` to the wall's right (the object's metres). */
function onPanel(x: number, up: number, out = 0): Vec {
  return [x, WALL.y + up * Math.cos(WALL.lean) - out * Math.sin(WALL.lean), WALL.z + up * Math.sin(WALL.lean) + out * Math.cos(WALL.lean)];
}
/** The panel's face (m out from the middle) at height y. */
const faceZ = (y: number) => WALL.z + (y - WALL.y) * Math.tan(WALL.lean);
type Limb = 'lh' | 'rh' | 'lf' | 'rf';
/**
 * The problem (the reference body's): each move a limb onto a place, `x` m to the climber's right (scaled by
 * the size) and `up` m over the panel at the mat's top (in the body's size), from `t0` to `t1` of the round.
 */
const ROUTE: readonly { limb: Limb; x: number; up: number; t0: number; t1: number }[] = [
  { limb: 'lh', x: -0.45, up: 1.5, t0: 0.05, t1: 0.09 },
  { limb: 'rh', x: 0.55, up: 1.4, t0: 0.07, t1: 0.11 },
  { limb: 'lf', x: -0.35, up: 0.1, t0: 0.13, t1: 0.18 },
  { limb: 'rf', x: 0.18, up: 0.22, t0: 0.2, t1: 0.25 },
  { limb: 'rf', x: 0.45, up: 0.57, t0: 0.28, t1: 0.33 },
  { limb: 'rh', x: 0.3, up: 1.78, t0: 0.36, t1: 0.41 },
  { limb: 'lf', x: -0.68, up: 0.36, t0: 0.44, t1: 0.49 },
  { limb: 'lh', x: -0.2, up: 1.9, t0: 0.52, t1: 0.57 },
];
const BOULDER_S = 16;
/** The round after the top: the feet come off (`off`), the hands let go (`drop`), the landing (`land`), up again (`up`). */
const DROP = { off: 0.66, drop: 0.69, land: 0.72, up: 0.8 };

function boulderPose(fit: BoulderFit): (p: number) => PoseSpec {
  const { s, to } = fit;
  // The object's metres in the clip's (the climber at the wall's middle, the mat's top the clip's floor).
  const C = (v: Vec): Vec => [-v[0] / s, (v[1] - CRASH) / s, (to - v[2]) / s];
  const upMat = (CRASH - WALL.y) / Math.cos(WALL.lean);
  /** The nearest hold to a place (x the object's), if one is within reach of it. */
  const snap = (x: number, up: number): [number, number] => {
    let best: [number, number] = [x, up];
    let d = 0.17;
    for (const h of HOLDS) {
      const e = Math.hypot(h[0] - x, h[1] - up);
      if (e < d) [best, d] = [[h[0], h[1]], e];
    }
    return best;
  };
  // Found on the first frame baked (the reference skeleton is there only while baking).
  let made: { limbs: Record<Limb, Key<Vec>[]>; pelvis: Key<Vec>[] } | null = null;
  const make = () => {
    // The ankles' rest places, and the holds' in the clip's metres.
    const footRest = { lf: pointOn({}, 'foot_l', [0, 0, 0]), rf: pointOn({}, 'foot_r', [0, 0, 0]) };
    const hand = (x: number, up: number): Vec => C(onPanel(x, up, 0.05));
    const foot = (x: number, up: number): Vec => {
      const toe = onPanel(x, up + 0.06, 0.03);
      return C([toe[0], toe[1], toe[2] + 0.12 * s]);
    };
    const where: Record<Limb, Vec | null> = { lh: null, rh: null, lf: footRest.lf, rf: footRest.rf };
    const limbs: Record<Limb, Key<Vec>[]> = { lh: [], rh: [], lf: [[0, footRest.lf]], rf: [[0, footRest.rf]] };
    const restP: Vec = [0, 0.947, 0];
    const pelvis: Key<Vec>[] = [[0, restP], [0.05, restP]];
    /** The pelvis for the limbs' places: over the feet (bent legs) but up to the higher hand, out from the face. */
    const hips = (): Vec => {
      const feet = [where.lf!, where.rf!];
      const hands = [where.lh, where.rh].filter((h): h is Vec => !!h);
      const footY = (feet[0][1] + feet[1][1]) / 2;
      const lowFoot = Math.min(feet[0][1], feet[1][1]);
      const top = Math.max(...hands.map((h) => h[1]));
      // (Both feet still on the mat: standing, the knees soft.)
      const mat = feet[0] === footRest.lf && feet[1] === footRest.rf;
      const y = mat ? 0.92 : Math.max(footY + 0.55, Math.min(lowFoot + 0.84, top - 1.0));
      // Out from the face (the object's depth at that height, in the clip's metres: forward is towards it).
      const yObj = y * s + CRASH;
      const z = (to - faceZ(yObj)) / s - 0.26;
      const x = (feet[0][0] + feet[1][0] + hands.reduce((n, h) => n + h[0], 0) * (2 / Math.max(1, hands.length))) / 4;
      return [x, y, Math.min(z, 0.06)];
    };
    for (const m of ROUTE) {
      const [hx, hu] = snap(-m.x * s, upMat + m.up * s);
      const at = m.limb[1] === 'h' ? hand(hx, hu) : foot(hx, hu);
      const keys = limbs[m.limb];
      if (m.limb[1] === 'h' && !where[m.limb]) keys.push([m.t0, [NaN, NaN, NaN]]);
      else keys.push([m.t0, where[m.limb]!]);
      keys.push([m.t1, at]);
      where[m.limb] = at;
      pelvis.push([m.t1 + 0.01, hips()]);
    }
    // At the top: a look down; the feet off the holds, hanging; the hands let go; the drop onto the mat.
    const top = hips();
    pelvis.push([DROP.off, top]);
    const hang: Vec = [top[0], top[1] - 0.12, top[2] - 0.08];
    pelvis.push([DROP.drop, hang]);
    for (const f of ['lf', 'rf'] as const) {
      const sx = f === 'lf' ? -1 : 1;
      limbs[f].push([DROP.off, where[f]!], [DROP.drop, [sx * 0.12, hang[1] - 0.84, hang[2] - 0.05]], [DROP.land, footRest[f]], [1, footRest[f]]);
    }
    for (const h of ['lh', 'rh'] as const) limbs[h].push([DROP.drop, where[h]!], [DROP.drop + 0.02, [NaN, NaN, NaN]]);
    pelvis.push([DROP.land, [0, 0.62, -0.06], 'in'], [DROP.land + 0.015, [0, 0.6, -0.07]], [DROP.up, restP], [1, restP]);
    return { limbs, pelvis };
  };
  let restL: Vec = [0, 0, 0];
  let restR: Vec = [0, 0, 0];
  return (p) => {
    if (!made) {
      made = make();
      restL = pointOn({}, 'foot_l', [0, 0, 0]);
      restR = pointOn({}, 'foot_r', [0, 0, 0]);
    }
    const pel = track(made.pelvis, p);
    const climbing = p > 0.1 && p < DROP.land;
    const onWall = smooth((p - 0.05) / 0.08) * (1 - smooth((p - DROP.drop) / 0.03));
    // Looking up to the hand that goes, down to the foot.
    const move = ROUTE.find((m) => p >= m.t0 - 0.01 && p <= m.t1 + 0.01);
    const look = move ? (move.limb[1] === 'h' ? -32 : 30) : p > 0.6 && p < DROP.off ? 34 : -10 * onWall;
    const land = Math.sin(Math.PI * Math.min(1, Math.max(0, (p - DROP.land) / (DROP.up - DROP.land))));
    const shake = Math.sin(p * TAU * 24) * Math.max(0, Math.sin(Math.PI * Math.min(1, Math.max(0, (p - 0.82) / 0.12))));
    const spec = (l: Vec, r: Vec): PoseSpec =>
      body({
        breath: p * 5,
        at: [pel[0], pel[1] - 0.947, pel[2]],
        hip: 10 * onWall + 30 * land,
        bend: -6 * onWall + 20 * land,
        head: [look - 10 * land, 0, 0],
        l: sub(l, restL),
        r: sub(r, restR),
        fl: [climbing ? -10 : 0, -8, 0],
        fr: [climbing ? -10 : 0, 8, 0],
        kl: climbing ? [-0.6, 0, 1] : undefined,
        kr: climbing ? [0.6, 0, 1] : undefined,
      });
    const foot = (f: 'lf' | 'rf') => moved(made!.limbs[f], p, [0, 0.06, -0.08]);
    const b = spec(foot('lf'), foot('rf'));
    const arm = (side: 'l' | 'r'): Hand => {
      const sx = side === 'l' ? -1 : 1;
      const keys = made!.limbs[`${side}h`];
      const free: Vec = [pel[0] + sx * 0.24, pel[1] - 0.1 + 0.05 * shake, pel[2] + 0.06];
      // (A hand off the wall, NaN in its keys: by the side.)
      const k = keys.map(([t, v, e]) => [t, Number.isNaN(v[0]) ? free : v, e] as Key<Vec>);
      const at = k.length ? moved(k, p, [0, 0.04, -0.1]) : free;
      return { at, palm: true, pole: [0.9 * sx, -0.5, -0.2], pronate: 80, hand: wrist(side, -20, 0) };
    };
    return arms(b, arm('l'), arm('r'));
  };
}

// --- yoga ---------------------------------------------------------------------------------------

/**
 * A yoga flow on the mat (`yogaMat`, used from its front end facing along it: `STEP_IN`): mountain, the hands
 * to the heart, an upward salute, a forward fold, a half lift, the fold again, the hands walked out to a
 * downward dog (the heels pedalling), the right foot stepped up to a lunge and up into warrior I, back down
 * to the dog, the knees down, a child's pose, sitting up on the heels a while, then up through a squat. The
 * mat is short for the dog and the warrior (0.9 m: its length is a model fix, G-40): the dog is a short one.
 */
const YOGA_S = 48;
/** Where the yogi stands (`STEP_IN`): the ankles 0.3 m out from the mat's middle, its front end, on its top. */
export const YOGA_AT = { to: 0.3, up: 0.012 };
interface YogaPose extends Body {
  hl: Vec;
  hr: Vec;
  /** Forearm roll (both hands, + palm down) and the hands' flex (+ to the palm). */
  pron: number;
  flex: number;
}
const MOUNTAIN: YogaPose = { hl: [-0.21, 0.88, 0.05], hr: [0.21, 0.88, 0.05], pron: -10, flex: 5 };
const HEART: YogaPose = { ...MOUNTAIN, hl: [-0.03, 1.22, 0.2], hr: [0.03, 1.22, 0.2], pron: -5, flex: -70, head: [6, 0, 0] };
const SALUTE: YogaPose = { ...MOUNTAIN, hl: [-0.11, 2.1, 0.06], hr: [0.11, 2.1, 0.06], pron: -10, flex: -10, bend: -9, head: [-22, 0, 0] };
const FLOOR = 0.03;
const FOLD: YogaPose = { at: [0, -0.04, -0.13], hip: 78, bend: 42, head: [12, 0, 0], hl: [-0.17, FLOOR, 0.3], hr: [0.17, FLOOR, 0.3], pron: 85, flex: -55 };
const HALF: YogaPose = { at: [0, -0.03, -0.12], hip: 70, bend: 2, head: [-28, 0, 0], hl: [-0.12, 0.47, 0.2], hr: [0.12, 0.47, 0.2], pron: 85, flex: -10 };
const [TIP, TOES] = tiptoe(0.07);
const DOG: YogaPose = { at: [0, -0.08, 0.2], hip: 108, bend: -12, head: [8, 0, 0], hl: [-0.2, FLOOR, 0.84], hr: [0.2, FLOOR, 0.84], pron: 85, flex: -60, l: TIP, r: TIP, fl: [TOES, -4, 0], fr: [TOES, 4, 0] };
const FRONT: Vec = [0, 0, 0.6];
const LUNGE: YogaPose = { ...DOG, at: [0, -0.5, 0.3], hip: 62, bend: 18, head: [-16, 0, 0], hl: [-0.21, FLOOR, 0.74], hr: [0.21, FLOOR, 0.74], r: FRONT, fr: [0, 4, 0], kr: [0, 0, 1] };
/** Warrior I: the hips square to the front of the mat, the back heel down and turned out, the arms up. */
const WARRIOR: YogaPose = {
  at: [0, -0.21, 0.3],
  yaw: -6,
  hip: -4,
  bend: -8,
  head: [-14, 6, 0],
  l: [0, 0, 0],
  fl: [0, -45, 0],
  kl: [-0.5, 0, 1],
  r: FRONT,
  fr: [0, 4, 0],
  kr: [0, 0, 1],
  hl: [-0.1, 1.86, 0.36],
  hr: [0.1, 1.86, 0.36],
  pron: -10,
  flex: -10,
};
/** Kneeling: on the knees and the tops of the feet, sitting on the heels (`SEATED`), the hands on the thighs. */
const KNEEL: Vec = [0, -0.035, -0.02];
const FLAT: Rot = [170, 0, 0];
const TABLE: YogaPose = { at: [0, -0.48, 0.32], hip: 85, bend: -6, head: [-6, 0, 0], hl: [-0.2, FLOOR, 0.8], hr: [0.2, FLOOR, 0.8], pron: 85, flex: -60, l: KNEEL, r: KNEEL, fl: FLAT, fr: FLAT, kl: [0, -0.3, 1], kr: [0, -0.3, 1] };
const CHILD: YogaPose = { ...TABLE, at: [0, -0.62, 0.0], hip: 112, bend: 30, head: [20, 0, 0], hl: [-0.16, FLOOR, 0.78], hr: [0.16, FLOOR, 0.78] };
const SEATED: YogaPose = { ...TABLE, at: [0, -0.62, -0.02], hip: -2, bend: 2, head: [6, 0, 0], hl: [-0.13, 0.42, 0.3], hr: [0.13, 0.42, 0.3], pron: 85, flex: 10 };
const TUCK_PITCH = 80;
const TUCKED: YogaPose = { ...SEATED, at: [0, -0.56, -0.02], l: [0, 0.07, -0.02], r: [0, 0.07, -0.02], fl: [TUCK_PITCH, -4, 0], fr: [TUCK_PITCH, 4, 0], hl: [-0.15, 0.45, 0.35], hr: [0.15, 0.45, 0.35] };
const SQUAT: YogaPose = { at: [0, -0.52, -0.14], hip: 40, bend: 22, head: [-14, 0, 0], hl: [-0.14, 0.62, 0.42], hr: [0.14, 0.62, 0.42], pron: 30, flex: 0, fl: [0, -18, 0], fr: [0, 18, 0], kl: [-0.4, 0, 1], kr: [0.4, 0, 1] };
/** The flow: (time, pose) a hold from each key to the next one's start, moved between them over `go` s. */
const FLOW: readonly (readonly [t: number, pose: YogaPose, go: number])[] = [
  [0, MOUNTAIN, 0],
  [4.5, HEART, 1.5],
  [8, SALUTE, 1.5],
  [12, FOLD, 2],
  [15.5, HALF, 1.5],
  [18.5, FOLD, 1.5],
  [21, DOG, 2.5],
  [27.5, LUNGE, 1.5],
  [29.5, WARRIOR, 2],
  [35.5, LUNGE, 1.5],
  [37, DOG, 1.5],
  [38.5, TABLE, 1.5],
  [40, CHILD, 1.5],
  [43, SEATED, 1.5],
  [44.5, TUCKED, 1.0],
  [46, SQUAT, 1.5],
  [YOGA_S, MOUNTAIN, 2],
];

/** The pose at time `t`: held, or moving from one to the next (numbers and places eased; feet and hands lifted on the way). */
function yogaAt(t: number): { pose: YogaPose; u: number; from: YogaPose; to: YogaPose } {
  for (let i = 1; i < FLOW.length; i++) {
    const [t1, b, go] = FLOW[i];
    if (t > t1) continue;
    const a = FLOW[i - 1][1];
    const u = smooth(Math.max(0, (t - (t1 - go)) / go));
    const n = (x: number | undefined, y: number | undefined) => (x ?? 0) + ((y ?? 0) - (x ?? 0)) * u;
    const v = (x: Vec | undefined, y: Vec | undefined, d: Vec = [0, 0, 0]) => lerpV(x ?? d, y ?? d, u);
    const pose: YogaPose = {
      at: v(a.at, b.at),
      hip: n(a.hip, b.hip),
      yaw: n(a.yaw, b.yaw),
      bend: n(a.bend, b.bend),
      twist: n(a.twist, b.twist),
      head: v(a.head, b.head),
      l: v(a.l, b.l),
      r: v(a.r, b.r),
      fl: v(a.fl, b.fl, [0, -4, 0]),
      fr: v(a.fr, b.fr, [0, 4, 0]),
      kl: a.kl || b.kl ? v(a.kl, b.kl, [-0.12, 0, 1]) : undefined,
      kr: a.kr || b.kr ? v(a.kr, b.kr, [0.12, 0, 1]) : undefined,
      hl: v(a.hl, b.hl),
      hr: v(a.hr, b.hr),
      pron: n(a.pron, b.pron),
      flex: n(a.flex, b.flex),
    };
    return { pose, u, from: a, to: b };
  }
  return { pose: MOUNTAIN, u: 0, from: MOUNTAIN, to: MOUNTAIN };
}

function yogaPose(p: number): PoseSpec {
  const t = p * YOGA_S;
  const { pose: y, u, from, to } = yogaAt(t);
  // (Lifted clear until it is all but there.)
  const lifting = u > 0 && u < 1 ? Math.sqrt(Math.sin(Math.PI * u)) : 0;
  // A foot that moves on along the mat steps (lifted); in the dog the heels pedal, in turn.
  const step = (side: 'l' | 'r'): Vec => {
    const a = from[side] ?? [0, 0, 0];
    const b = to[side] ?? [0, 0, 0];
    return Math.hypot(b[0] - a[0], b[2] - a[2]) > 0.2 ? addV(y[side] ?? [0, 0, 0], [0, 0.1, 0], lifting) : y[side] ?? [0, 0, 0];
  };
  const pedal = t > 21 && t < 26 ? Math.sin(((t - 21) / 2.5) * TAU) : 0;
  const l = addV(step('l'), [0, -0.035 * Math.max(0, pedal), 0]);
  const r = addV(step('r'), [0, -0.035 * Math.max(0, -pedal), 0]);
  const b = body({ ...y, breath: t / 5, l, r });
  // On the floor from place to place (walked out to the dog and back), the hands go in turn, lifted.
  const hand = (side: 'l' | 'r'): Vec => {
    const a = from[`h${side}`];
    const c = to[`h${side}`];
    if (a[1] !== FLOOR || c[1] !== FLOOR || Math.abs(c[2] - a[2]) < 0.2) return y[`h${side}`];
    const k = Math.min(1, Math.max(0, 2 * u - (side === 'l' ? 0 : 1)));
    return addV(lerpV(a, c, smooth(k)), [0, 0.05, 0], Math.sin(Math.PI * k));
  };
  return arms(
    b,
    { at: hand('l'), palm: true, pole: [-0.8, -0.5, -0.3], pronate: y.pron, hand: wrist('l', y.flex, 0) },
    { at: hand('r'), palm: true, pole: [0.8, -0.5, -0.3], pronate: y.pron, hand: wrist('r', y.flex, 0) },
  );
}

export function gameClips(): ClipDef[] {
  return [
    cycle('dartThrow', { seconds: DART_S, loop: true }, dartPose),
    cycle('pingPong', { seconds: PONG_S, loop: true }, pongPose),
    cycle('yoga', { seconds: YOGA_S, loop: true, hands: 'open' }, yogaPose),
    ...POOL.map(({ clip, fit }) => cycle(clip, { seconds: POOL_S, loop: true }, poolPose(fit))),
    ...FOOS.map(({ clip, fit }) => cycle(clip, { seconds: FOOS_S, loop: true }, foosPose(fit))),
    ...SMASH.map(({ clip, fit }) => cycle(clip, { seconds: SMASH_S, loop: true }, smashPose(fit))),
    ...BOULDER.map(({ clip, fit }) => cycle(clip, { seconds: BOULDER_S, loop: true }, boulderPose(fit))),
  ];
}
