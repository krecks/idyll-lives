/**
 * Rides and climbs at things: a child's round on the slide (`STEP_IN` in characters.ts: round to its ladder,
 * up it, a step up onto the deck and stood under its roof, sat at the top holding the arch, down the chute sat up,
 * off at the run-out and back round to the front), the upper bunk's ladder (characters.ts `BUNK_LADDER`: up it and
 * onto the mattress getting into bed, down it getting up), and the rides sat astride with the hands on the
 * grips (characters.ts `RIDES`): the rocking horse and the spring rider, the seesaw, the merry-go-round and the
 * swing (the legs pumping in time with it).
 *
 * The things are at fixed heights, so the clips come in sizes, fitted as the climbing frame's (`street.ts`) but
 * by the legs: each size is for a body whose legs are `s` of the reference skeleton's (the runtime moves the
 * pelvis by the body's `pelvisScale`, and a child's legs are shorter for their height than a grown-up's: 0.51
 * of it, a teen's and a grown-up's 0.56), so a place on the object `m` metres from where the clip's root is put
 * is `m / s` of the clip's metres from it.
 */

import { cycle, mix, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, hangArms, TAU } from './idle';
import { liePose } from './lie';
import { SEAT_MOVE, seatedBody } from './seated';
import { addV, armIK, footIK, lerpV, smooth, sub, wrist } from './ik';
import type { Fit, Size } from './street';

// --- tracks -------------------------------------------------------------------------------------

/**
 * A key: at `t` (a share of the clip) the value `v`, eased into from the key before (smoothly; `lin`: at an
 * even pace, for a slide; `lift`: a foot or a hand lifted clear on its way, `LIFT`).
 */
type K<T> = readonly [t: number, v: T, how?: 'lin' | 'lift'];
/** How far a foot or hand is lifted (the clip's metres) at the middle of a lifted move: up and back towards the body. */
const LIFT: Vec = [0, 0.07, -0.03];

function value<T extends number | Vec>(keys: readonly K<T>[], t: number, lift: Vec = LIFT): T {
  if (t <= keys[0][0]) return keys[0][1];
  for (let i = 1; i < keys.length; i++) {
    const [t1, b, how] = keys[i];
    if (t > t1) continue;
    const [t0, a] = keys[i - 1];
    const u = t1 > t0 ? (t - t0) / (t1 - t0) : 1;
    const e = how === 'lin' ? u : smooth(u);
    if (typeof a === 'number') return (a + ((b as number) - a) * e) as T;
    const v = lerpV(a as Vec, b as Vec, e);
    const moved = Math.hypot((b as Vec)[0] - (a as Vec)[0], (b as Vec)[1] - (a as Vec)[1], (b as Vec)[2] - (a as Vec)[2]) > 1e-3;
    return (how === 'lift' && moved ? addV(v, lift, Math.sin(Math.PI * u)) : v) as T;
  }
  return keys[keys.length - 1][1];
}

/** A direction in the clip's frame turned by `yaw` degrees (as the pelvis turns it: + towards the clip's +x, the body's right). */
function turned(v: Vec, yaw: number): Vec {
  const a = (yaw * Math.PI) / 180;
  return [v[0] * Math.cos(a) + v[2] * Math.sin(a), v[1], -v[0] * Math.sin(a) + v[2] * Math.cos(a)];
}

type Side = 'l' | 'r';
/**
 * A round of keys (the clip's metres, absolute: the pelvis joint, the ankles, the palms): the body's place,
 * turn (`yaw`, deg, + to the right), the pelvis pitched forward `hip` and the back bent `bend` (deg), the head's
 * nod (+ down); each foot's place and pitch (+ toes down) and where its knee points (`knee`: 0 forward, 1 up,
 * -1 down); each hand's palm and how much it holds there (`hold`: 0 hanging by the side, 1 at its place) and
 * where its elbow points (`elbow`: 0 down and out, 1 back).
 */
class Round {
  readonly pel: K<Vec>[] = [];
  readonly yaw: K<number>[] = [];
  readonly hip: K<number>[] = [];
  readonly bend: K<number>[] = [];
  readonly nod: K<number>[] = [];
  readonly foot: Record<Side, K<Vec>[]> = { l: [], r: [] };
  readonly pitch: Record<Side, K<number>[]> = { l: [], r: [] };
  /** Each foot's own turn (deg): planted, it keeps the turn it landed with while the body turns over it. */
  readonly turn: Record<Side, K<number>[]> = { l: [], r: [] };
  readonly knee: Record<Side, K<number>[]> = { l: [], r: [] };
  readonly hand: Record<Side, K<Vec>[]> = { l: [], r: [] };
  readonly hold: Record<Side, K<number>[]> = { l: [], r: [] };
  readonly elbow: Record<Side, K<number>[]> = { l: [], r: [] };

  /** The last value of a track. */
  static last<T>(keys: K<T>[]): T {
    return keys[keys.length - 1][1];
  }
  /** Keys `v` at `t`, held from the last key until `from` (a share; default: moving from the last key on). */
  static to<T>(keys: K<T>[], t: number, v: T, how?: 'lin' | 'lift', from?: number): void {
    if (from !== undefined && keys.length && from > keys[keys.length - 1][0]) keys.push([from, Round.last(keys)]);
    keys.push([t, v, how]);
  }
  /** The body at `t`: where and how (unset: as it was). */
  body(t: number, o: { at?: Vec; yaw?: number; hip?: number; bend?: number; nod?: number }, how?: 'lin'): void {
    Round.to(this.pel, t, o.at ?? Round.last(this.pel), how);
    Round.to(this.yaw, t, o.yaw ?? Round.last(this.yaw), how);
    Round.to(this.hip, t, o.hip ?? Round.last(this.hip), how);
    Round.to(this.bend, t, o.bend ?? Round.last(this.bend), how);
    Round.to(this.nod, t, o.nod ?? Round.last(this.nod), how);
  }
  /** Foot `side` at `at` by `t` (lifted on its way unless `how` says), its toes pitched `pitch`, from `from` (default the last key). */
  step(side: Side, t: number, at: Vec, o: { pitch?: number; knee?: number; yaw?: number; from?: number; how?: 'lin' | 'lift' | 'set' } = {}): void {
    const how = o.how === 'set' ? undefined : (o.how ?? 'lift');
    Round.to(this.foot[side], t, at, how, o.from);
    Round.to(this.turn[side], t, o.yaw ?? Round.last(this.yaw), undefined, o.from);
    Round.to(this.pitch[side], t, o.pitch ?? Round.last(this.pitch[side]), how === 'lin' ? 'lin' : undefined, o.from);
    Round.to(this.knee[side], t, o.knee ?? Round.last(this.knee[side]), undefined, o.from);
  }
  /** Hand `side` onto `at` by `t` (lifted on its way), holding (`hold` 1) or let go to hang (0). */
  grab(side: Side, t: number, at: Vec | null, o: { from?: number; elbow?: number; how?: 'lin' | 'lift' | 'set' } = {}): void {
    const how = o.how === 'set' ? undefined : (o.how ?? 'lift');
    const hand = this.hand[side];
    // (Taking hold from hanging: the arm swings up to the place itself, `posed` blending in the reach.)
    const hanging = !hand.length || Round.last(this.hold[side]) < 0.5;
    if (at && hanging && hand.length) Round.to(hand, o.from ?? hand[hand.length - 1][0], at);
    Round.to(hand, t, at ?? (hand.length ? Round.last(hand) : [0, 0, 0]), hanging ? undefined : how, hanging ? undefined : o.from);
    Round.to(this.hold[side], t, at ? 1 : 0, undefined, o.from);
    Round.to(this.elbow[side], t, o.elbow ?? Round.last(this.elbow[side]), undefined, o.from);
  }
  /** Both hands. */
  grabBoth(t: number, l: Vec | null, r: Vec | null, o: { from?: number; elbow?: number; how?: 'lin' | 'lift' | 'set' } = {}): void {
    this.grab('l', t, l, o);
    this.grab('r', t, r, o);
  }
  /** Every track. */
  tracks(): K<number | Vec>[][] {
    return [this.pel, this.yaw, this.hip, this.bend, this.nod, ...(['l', 'r'] as const).flatMap((s) => [this.foot[s], this.pitch[s], this.turn[s], this.knee[s], this.hand[s], this.hold[s], this.elbow[s]])];
  }
  /** Every track sorted (keys pushed out of order by `from`) and ended at `t = 1` with its value there. */
  done(): this {
    for (const keys of this.tracks()) {
      keys.sort((a, b) => a[0] - b[0]);
      if (keys[keys.length - 1][0] < 1) keys.push([1, keys[keys.length - 1][1]]);
    }
    return this;
  }
  /**
   * This round's part from `a` to `b` played backwards from `c0` to `c1` (climbing back down what it climbed up,
   * backing out the way it crawled in), as a new round to go on from. Each move keeps its own way of moving.
   */
  reversed(a: number, b: number, c0: number, c1: number): Round {
    const out = new Round();
    const into = out.tracks();
    const at = (t: number) => c0 + ((b - t) * (c1 - c0)) / (b - a);
    this.tracks().forEach((keys, n) => {
      const inner = keys.filter(([t]) => t > a && t < b);
      const all: K<number | Vec>[] = [[a, value(keys, a, [0, 0, 0])], ...inner, [b, value(keys, b, [0, 0, 0])]];
      // (A move's way, kept on the key it ends at: backwards it ends at the key it started from.)
      for (let k = all.length - 1; k >= 0; k--) into[n].push([at(all[k][0]), all[k][1], all[k + 1]?.[2]]);
    });
    return out;
  }
}

/** The reference skeleton's rest places (the clip's metres), found while baking. */
function rest(): { pelvis: Vec; ankle: Record<Side, Vec> } {
  const ref = reference();
  return { pelvis: ref.joint('pelvis'), ankle: { l: ref.joint('foot_l'), r: ref.joint('foot_r') } };
}

/**
 * Turning over a planted foot, it pivots on its ball (`ball` m ahead of the ankle): the heel lifts `rate` m
 * per radian the body has turned from the foot, `most` m at most. (A planted foot that stayed flat as the
 * body turned would twist on the floor; a child's wider hips would slide it round too.)
 */
const PIVOT = { ball: 0.17, rate: 0.25, most: 0.05 };

/** The pose of a round at `p`, breathing at `breath` (phase of a breath). */
function posed(r: Round, p: number, breath: number): PoseSpec {
  const at = value(r.pel, p);
  const yaw = value(r.yaw, p);
  const hip = value(r.hip, p);
  const bend = value(r.bend, p);
  const nod = value(r.nod, p);
  const b = breathe(breath, 0.6);
  const { pelvis, ankle } = rest();
  const spec: PoseSpec = {
    move: sub(at, pelvis),
    pelvis: [hip, yaw, 0],
    spine_01: [bend * 0.35, 0, 0],
    spine_02: [bend * 0.35 + (b.spine_02?.[0] ?? 0), 0, 0],
    spine_03: [bend * 0.3 + (b.spine_03?.[0] ?? 0), 0, 0],
    clavicle_l: b.clavicle,
    clavicle_r: [0, 0, -(b.clavicle?.[2] ?? 0)],
    neck_01: [nod * 0.4, 0, 0],
    Head: [nod * 0.6, 0, 0],
  };
  const out: PoseSpec = { ...spec };
  for (const side of ['l', 'r'] as const) {
    const sx = side === 'l' ? -1 : 1;
    const k = value(r.knee[side], p);
    // Forward and a touch out, up (sat with the legs out), or down (kneeling), turned with the body.
    const pole = k >= 0 ? lerpV([0.12 * sx, 0, 1], [0.15 * sx, 1, 0.25], k) : lerpV([0.12 * sx, 0, 1], [0.1 * sx, -1, 0.35], -k);
    // The body turning over a planted foot: up on its ball (`PIVOT`), the heel lifting with the turn.
    const fy = value(r.turn[side], p);
    const lift = Math.min(PIVOT.most, PIVOT.rate * Math.abs(((yaw - fy) * Math.PI) / 180));
    const a = Math.asin(lift / PIVOT.ball);
    const heel = turned([0, lift, -PIVOT.ball * (1 - Math.cos(a))], fy);
    const at = addV(value(r.foot[side], p), heel);
    Object.assign(out, footIK(side, sub(at, ankle[side]), spec, { foot: [value(r.pitch[side], p) + (a * 180) / Math.PI, fy + 4 * sx, 0], pole: turned(pole, yaw) }));
  }
  // The hands: hanging by the sides, blended into holding (arm IK) as they take hold.
  const hung = hangArms();
  for (const side of ['l', 'r'] as const) {
    const sx = side === 'l' ? -1 : 1;
    const w = value(r.hold[side], p);
    const e = value(r.elbow[side], p);
    const free: PoseSpec = { [`upperarm_${side}`]: hung[`upperarm_${side}`], [`lowerarm_${side}`]: hung[`lowerarm_${side}`], [`hand_${side}`]: hung[`hand_${side}`] } as PoseSpec;
    if (w <= 0.001) {
      Object.assign(out, free);
      continue;
    }
    const pole = turned(lerpV([0.6 * sx, -0.7, -0.3], [0.35 * sx, 0.1, -1], e), yaw);
    const held = armIK(side, value(r.hand[side], p), spec, { palm: true, pole, pronate: 60 + 20 * e, hand: wrist(side, -15, 0) });
    if (w >= 0.999) Object.assign(out, held);
    else for (const bone of Object.keys(held) as (keyof PoseSpec)[]) out[bone] = lerpV(free[bone] as Vec, held[bone] as Vec, w) as never;
  }
  return out;
}

/**
 * Walking `r` along `path` (the clip's metres, on the ground) from `t0` to `t1`: steps of about `stride` m, the
 * feet either side of the way, the body between them facing along it, then turning on the spot to `face`
 * (deg) if given. Starts with the foot `first`.
 */
function walk(r: Round, path: readonly Vec[], t0: number, t1: number, o: { first?: Side; face?: number; stride?: number } = {}): void {
  const ankleY = rest().ankle.l[1];
  const hipY = rest().pelvis[1] - 0.015;
  const stride = o.stride ?? 0.55;
  const segs = path.slice(1).map((b, i) => ({ a: path[i], b, len: Math.hypot(b[0] - path[i][0], b[2] - path[i][2]) }));
  const total = segs.reduce((n, s) => n + s.len, 0);
  // The direction of travel at a distance along the way (deg, unwrapped near the body's turn now).
  let yawNow = Round.last(r.yaw);
  const near = (y: number) => y + 360 * Math.round((yawNow - y) / 360);
  const where = (d: number): { at: Vec; yaw: number } => {
    let left = d;
    for (const s of segs) {
      if (left <= s.len + 1e-6 || s === segs[segs.length - 1]) {
        const u = s.len > 0 ? Math.min(1, left / s.len) : 1;
        return { at: lerpV(s.a, s.b, u), yaw: near((Math.atan2(s.b[0] - s.a[0], s.b[2] - s.a[2]) * 180) / Math.PI) };
      }
      left -= s.len;
    }
    return { at: path[path.length - 1], yaw: yawNow };
  };
  const turns = o.face === undefined ? 0 : Math.ceil(Math.abs(near(o.face) - where(total).yaw) / 60);
  const n = Math.max(2, Math.ceil(total / stride));
  const steps = n + 1 + 2 * turns;
  const dt = (t1 - t0) / steps;
  let side: Side = o.first ?? 'r';
  // (A foot at its rest place beside the body's: standing still at the end, the feet are where they rest.)
  const place = (at: Vec, yaw: number, s: Side): Vec => {
    const off = turned([rest().ankle[s][0], 0, rest().ankle[s][2]], yaw);
    return [at[0] + off[0], ankleY, at[2] + off[2]];
  };
  const put = (k: number, at: Vec, yaw: number, s: Side, body: Vec) => {
    const t = t0 + dt * k;
    r.step(s, t, place(at, yaw, s), { from: t - dt * 0.8, pitch: 0, knee: 0, yaw });
    r.body(t, { at: [body[0], hipY, body[2]], yaw, hip: 0, bend: 3, nod: 4 });
    yawNow = yaw;
  };
  r.body(t0, {});
  for (let k = 1; k <= n; k++) {
    const w = where((total * k) / n);
    const mid = where((total * (k - 0.5)) / n);
    put(k, w.at, w.yaw, side, k === n ? w.at : mid.at);
    side = side === 'l' ? 'r' : 'l';
  }
  // The other foot alongside, then turning on the spot a step each.
  const end = where(total);
  put(n + 1, end.at, end.yaw, side, end.at);
  side = side === 'l' ? 'r' : 'l';
  const target = o.face === undefined ? end.yaw : near(o.face);
  for (let k = 1; k <= 2 * turns; k++) {
    const yaw = end.yaw + ((target - end.yaw) * Math.ceil(k / 2)) / turns;
    put(n + 1 + k, end.at, yaw, side, end.at);
    side = side === 'l' ? 'r' : 'l';
  }
}

// --- the slide ----------------------------------------------------------------------------------

/**
 * The slide (playground, `model.slide`): a ladder up the back of its tower, its rails from the ground 0.94 m
 * behind the middle to the deck's back edge (1.42 m up, 0.7 m behind) 0.5 m apart, five rungs 0.237 m apart; the
 * deck's top 1.43 m up, 0.72 m to 0.28 m behind the middle, open at its back between the ladder's handrails, the
 * roof's eaves 2.46 m up; an arch over the chute's mouth (0.27 m either side of its middle, rising 0.55 m from
 * 1.43 m up, 0.27 m behind the middle); the
 * chute from the deck's front (1.43 m up, 0.29 m behind) down a smoothstep to its run-out, 0.28 m up and
 * level at 0.95 m in front, its floor 2.5 cm thick.
 */
const SLIDE_LADDER = { rail: 0.25, foot: -0.94, top: [1.42, -0.7] as const, rung: 0.2367, rungs: 5 };
const SLIDE_DECK = 1.43;
const SLIDE_HOOP = { y: 1.43, z: -0.27, r: 0.27, rise: 0.55 };
const CHUTE = { top: [1.43, -0.29] as const, end: [0.28, 0.95] as const, floor: 0.025 };
/** The chute's floor (m up) at a depth `z` (m in front of the middle). */
function chuteY(z: number): number {
  const t = Math.min(1, Math.max(0, (z - CHUTE.top[1]) / (CHUTE.end[1] - CHUTE.top[1])));
  return CHUTE.top[0] + (CHUTE.end[0] - CHUTE.top[0]) * t * t * (3 - 2 * t) + CHUTE.floor;
}
/** The chute's slope (deg down from level) at a depth `z`. */
function chuteSlope(z: number): number {
  return (Math.atan((chuteY(z - 0.01) - chuteY(z + 0.01)) / 0.02) * 180) / Math.PI;
}
type SlideFit = Fit;
/** Heights (m) the slide's round comes in for: children's (a teen and up isn't let on it). */
const SLIDE_HEIGHTS = [1.25, 1.12, 0.99];
/** A child's legs for their height (`pelvisScale` / height, rig.json). */
const CHILD_LEGS = 0.509;
/** The round in a size per height: the climber stands `to` m in front of the slide's middle, at the run-out's foot. */
export const SLIDE: Size<SlideFit>[] = SLIDE_HEIGHTS.map((h, i) => ({
  clip: `slide${Math.round(h * 100)}`,
  min: i < SLIDE_HEIGHTS.length - 1 ? (h + SLIDE_HEIGHTS[i + 1]) / 2 : 0,
  fit: { s: CHILD_LEGS * h, to: 1.25 },
}));
/** A round's length (s). */
const SLIDE_S = 14;

/**
 * A round on the slide, from standing at the run-out's foot facing it: round its right side to the ladder
 * (the walk on the left of the way, clear of the chute), hands onto the rails and up the five rungs, hand and
 * foot in turn; the hands up the handrails, a step up off the top rung onto the deck and the other foot after it,
 * stood up under the roof (M-community-07's headroom; A-community-02); the hands onto the arch's sides, sat down at
 * the top, the legs into the chute, a look down; a pull on the arch and a push, the hands let go onto the chute's
 * sides, down it sat up, the back leant back a little (25 deg down the steep middle), the legs out ahead along it;
 * upright on the run-out, the feet down, up off it, and a turn round back to where the round began.
 */
function slidePose(fit: SlideFit): (p: number) => PoseSpec {
  const { s, to } = fit;
  // The slide's metres (to its right of the middle, up, in front of it) in the clip's (from the start, facing it).
  const C = (x: number, y: number, z: number): Vec => [-x / s, y / s, (to - z) / s];
  let made: Round | null = null;
  const make = (): Round => {
    const r = new Round();
    const { pelvis, ankle } = rest();
    const hipY = pelvis[1];
    const aY = ankle.l[1];
    // (A child's pelvis joint over what it sits on, for the legs' length: rig.json `seatDrop`.)
    const sd = 0.174;
    // Standing at the start, facing the slide.
    r.body(0, { at: pelvis, yaw: 0, hip: 0, bend: 0, nod: 0 });
    for (const side of ['l', 'r'] as const) {
      r.step(side, 0, ankle[side], { pitch: 0, knee: 0, how: 'set' });
      r.grab(side, 0, null, { elbow: 0, how: 'set' });
    }
    r.body(0.03, {});
    // 1. Round its right side (the clip's left) to behind the ladder, and turn to face it.
    const zFoot = SLIDE_LADDER.foot - 0.05 - 0.14 * s;
    walk(r, [C(0, 0, to), C(0.62, 0, 0.95), C(0.62, 0, -1.02), C(0, 0, zFoot)], 0.03, 0.25, { first: 'l', face: 180 });
    const yaw = 180;
    // On the ladder (the slide's metres): a rung's middle, the rails at a height.
    const rung = (i: number): [number, number] => [SLIDE_LADDER.rung * i + 0.015, SLIDE_LADDER.foot + (0.24 * i) / 6];
    const railZ = (y: number) => (y <= SLIDE_LADDER.top[0] ? SLIDE_LADDER.foot + (0.24 * y) / SLIDE_LADDER.top[0] : SLIDE_LADDER.top[1] + (0.01 * (y - SLIDE_LADDER.top[0])) / 0.3);
    const foot = (side: Side, i: number): Vec => {
      const [y, z] = rung(i);
      return C((side === 'l' ? -1 : 1) * 0.1 * s, y + aY * s, z - 0.12 * s);
    };
    const palm = (side: Side, y: number): Vec => C((side === 'l' ? -1 : 1) * (SLIDE_LADDER.rail - 0.03), y, railZ(y) - 0.035);
    // The hips over the higher foot (on rung height `y`), back from the ladder's line, the knee bent.
    const climbHips = (y: number): Vec => C(0, y + 0.72 * s, railZ(y + 0.72 * s) - 0.25 * s);
    // 2. Hands onto the rails at the chest, then hand and foot in turn up the rungs.
    const c0 = 0.27;
    const c1 = 0.47;
    const reachY = (y: number) => y + 1.05 * s;
    r.grab('r', c0 + 0.01, palm('r', reachY(0) - 0.05 * s), { from: c0 - 0.02, elbow: 0 });
    r.grab('l', c0 + 0.02, palm('l', reachY(0)), { from: c0 - 0.01, elbow: 0 });
    r.body(c0 + 0.02, { at: [0, hipY - 0.03, Round.last(r.pel)[2] + 0.04], hip: 6, nod: -10 });
    const moves = 5;
    const dm = (c1 - c0 - 0.02) / moves;
    for (let i = 1; i <= moves; i++) {
      const t = c0 + 0.02 + dm * i;
      const side: Side = i % 2 ? 'r' : 'l';
      const other: Side = side === 'r' ? 'l' : 'r';
      r.step(side, t - dm * 0.35, foot(side, i), { from: t - dm, pitch: -8 });
      r.body(t - dm * 0.1, { at: climbHips(rung(i)[0]), yaw, hip: 12, nod: -18 });
      // The hand on the other side up a rung's height on the rail as the weight comes onto the foot.
      r.grab(other, t, palm(other, Math.min(1.7, reachY(rung(i)[0]) + 0.05 * s)), { from: t - dm * 0.3 });
      r.grab(side, t + dm * 0.4, palm(side, Math.min(1.68, reachY(rung(i)[0]) + 0.15 * s)), { from: t + dm * 0.05 });
    }
    // 3. Over the top: the hands up the handrails, a step up off the top rung onto the deck and the other foot after
    // it, stood up on the deck under the roof (its eaves 1.03 m over the deck: the head bowed a little), the hands
    // let go of the handrails.
    const o0 = c1;
    const o1 = 0.55;
    const ok = (k: number) => o0 + (o1 - o0) * k;
    const deckFoot = (side: Side, z: number): Vec => C((side === 'l' ? -1 : 1) * 0.096 * s, SLIDE_DECK + aY * s, z);
    const zStand = -0.6;
    r.grab('r', ok(0.15), palm('r', SLIDE_DECK + 0.5 * s), { from: ok(0.02) });
    r.grab('l', ok(0.25), palm('l', SLIDE_DECK + 0.45 * s), { from: ok(0.1) });
    r.step('r', ok(0.42), deckFoot('r', -0.66), { from: ok(0.2), pitch: 0, knee: 0 });
    r.body(ok(0.5), { at: C(0, SLIDE_DECK + 0.84 * hipY * s, -0.7), hip: 14, bend: 6, nod: -10 });
    r.step('l', ok(0.68), deckFoot('l', zStand), { from: ok(0.48), pitch: 0, knee: 0 });
    r.step('r', ok(0.82), deckFoot('r', zStand), { from: ok(0.66), pitch: 0, knee: 0 });
    r.body(ok(0.85), { at: C(0, SLIDE_DECK + 0.95 * hipY * s, zStand), hip: 4, bend: 4, nod: 12 });
    r.grabBoth(ok(1), null, null, { from: ok(0.82), elbow: 0 });
    // 4. Sat at the top: the hands onto the arch's sides, down onto the bottom at the deck's front, the legs out into
    // the chute (under the arch), a look down the slide.
    const t0 = o1;
    const t1 = 0.61;
    const tk = (k: number) => t0 + (t1 - t0) * k;
    const zSit = -0.5;
    const sitAt = C(0, SLIDE_DECK + sd * s, zSit);
    const legOut = (side: Side, z: number): Vec => C((side === 'l' ? -1 : 1) * 0.09 * s, chuteY(z) + 0.07 * s, z);
    const hoop = (side: Side, a: number): Vec => C((side === 'l' ? -1 : 1) * SLIDE_HOOP.r * Math.cos((a * Math.PI) / 180), SLIDE_HOOP.y + SLIDE_HOOP.rise * Math.sin((a * Math.PI) / 180), SLIDE_HOOP.z - 0.03);
    r.grab('l', tk(0.25), hoop('l', 55), { from: tk(0.02), elbow: 0 });
    r.grab('r', tk(0.3), hoop('r', 55), { from: tk(0.07), elbow: 0 });
    r.body(tk(0.45), { at: C(0, SLIDE_DECK + 0.5 * hipY * s, zSit - 0.04), hip: 35, bend: 10, nod: 5 });
    r.step('l', tk(0.6), legOut('l', zSit + 0.72 * s), { from: tk(0.3), pitch: -20, knee: 1 });
    r.step('r', tk(0.7), legOut('r', zSit + 0.72 * s), { from: tk(0.4), pitch: -20, knee: 1 });
    r.body(tk(0.75), { at: sitAt, hip: 10, bend: 6, nod: 10 });
    r.grab('l', tk(0.85), hoop('l', 35), { from: tk(0.6) });
    r.grab('r', tk(0.9), hoop('r', 35), { from: tk(0.65) });
    r.body(t1, { nod: 18 });
    // 5. A pull on the arch, the hands let go at the push and down onto the chute's sides; down it sat up, the back
    // leant back a little (more down the steep middle), the legs out ahead along it, the heels on it; upright on
    // the run-out.
    const s0 = t1 + 0.02;
    const s1 = 0.71;
    r.body(s0, { at: C(0, SLIDE_DECK + sd * s, -0.42), hip: -5, bend: 4, nod: 12 });
    const n = 12;
    const zEnd = CHUTE.end[1] - 0.12;
    const zFrom = -0.38;
    const side = (sx: number, z: number): Vec => C(sx * 0.235, chuteY(z) + 0.14, z + 0.02);
    for (let k = 1; k <= n; k++) {
      const u = k / n;
      // Quick down the steep middle, slowing on the level run-out.
      const e = 1 - (1 - u) * (1 - u);
      const t = s0 + (s1 - s0) * u;
      const z = zFrom + (zEnd - zFrom) * e;
      // Leant back 25 deg from upright down the steep part, 8 at the last on the level.
      const steep = Math.min(1, chuteSlope(z) / 30);
      const y = chuteY(z) + (sd + 0.01) * s;
      r.body(t, { at: C(0, y, z), hip: -8 - 17 * steep, bend: 2, nod: 10 + 6 * steep }, 'lin');
      // The legs out ahead along the chute (the heels on it, the knees a little bent), off its end at the last.
      let zf = z;
      while (zf < CHUTE.end[1] + 0.05 && Math.hypot(zf - z, chuteY(zf) - y) < 0.72 * s) zf += 0.01;
      for (const sd2 of ['l', 'r'] as const) r.step(sd2, t, legOut(sd2, zf), { pitch: -15, knee: 0.6, how: 'lin' });
      if (k === 1) r.grabBoth(t, side(-1, z), side(1, z), { elbow: 0.3 });
      else r.grabBoth(t, side(-1, z), side(1, z), { how: 'lin' });
    }
    r.grabBoth(s1 + 0.01, null, null, { elbow: 0 });
    // 6. The feet down off the end onto the ground, up off the run-out over them, a turn round and back.
    const g0 = s1;
    const g1 = 0.82;
    const gk = (k: number) => g0 + (g1 - g0) * k;
    const zLand = CHUTE.end[1] + 0.24 * s;
    r.step('l', gk(0.2), C(-0.096 * s, aY * s, zLand), { pitch: 0, knee: 0 });
    r.step('r', gk(0.28), C(0.096 * s, aY * s, zLand), { pitch: 0, knee: 0 });
    r.body(gk(0.4), { at: C(0, chuteY(zEnd) + 0.15 * s, zEnd + 0.02), hip: 30, bend: 15, nod: -10 });
    r.body(gk(0.75), { at: C(0, 0.6 * s, zLand - 0.12 * s), hip: 25, bend: 10, nod: -5 });
    r.body(g1, { at: C(0, hipY * s - 0.01, zLand), hip: 0, bend: 3, nod: 4 });
    walk(r, [C(0, 0, zLand), C(0, 0, to)], g1 + 0.01, 0.97, { first: 'l', face: 360, stride: 0.4 });
    r.body(1, { at: pelvis, yaw: 360, hip: 0, bend: 0, nod: 0 });
    return r.done();
  };
  return (p) => {
    made ??= make();
    return posed(made, p, p * 4);
  };
}

// --- the bunk bed's ladder ----------------------------------------------------------------------

/**
 * The bunk bed (bedroom, `model.bedBunk`; the clip's root at its middle, facing its foot end, the bed's metres
 * the clip's in its size): the ladder flat against its right side at the foot end, 0.465 m to the right of the
 * middle, its rails 0.42 m and 0.86 m towards the foot, rungs 0.32, 0.62, 0.92 and 1.22 m up; the upper mattress
 * 1.38 m up (characters.ts `BUNKS`), 0.84 m wide and 1.8 m long; its guard rail along the right side as far as
 * 0.38 m towards the foot, so the way in is between it and the foot end. The tile in front of the foot end is
 * 1.5 m from the middle.
 */
const BUNK = { x: 0.465, rails: [0.42, 0.86] as const, rungs: [0.32, 0.62, 0.92, 1.22], top: 1.38, tile: 1.5 };
/** The ladder's middle (m towards the foot end), where the climber goes up it. */
const BUNK_Z = 0.64;
/**
 * The sizes of the climb (heights, m, and the legs' share of the reference's): a grown-up's, a teen's and a
 * child's (an elder doesn't take the upper bunk: content `upperStages`). Each lies down as the runtime lies a
 * body of its height (characters.ts `lieCentre`, `lieBack`): the feet `h - 0.91` m towards the foot from the
 * middle, the back `0.1 h / 1.77` m over the mattress.
 */
const BUNK_SIZES: readonly (readonly [h: number, s: number])[] = [
  [1.72, 0.97],
  [1.47, 0.83],
  [1.12, 0.57],
];
type BunkFit = Fit & { h: number };
const bunkSizes = (name: string): Size<BunkFit>[] =>
  BUNK_SIZES.map(([h, s], i) => ({ clip: `${name}${Math.round(h * 100)}`, min: i < BUNK_SIZES.length - 1 ? (h + BUNK_SIZES[i + 1][0]) / 2 : 0, fit: { s, to: 0, h } }));
/** Up the ladder into the upper bunk and lain down (characters.ts `BUNK_LADDER`, getting into bed), and back down it getting up. */
export const BUNK_UP = bunkSizes('bunkUp');
export const BUNK_DOWN = bunkSizes('bunkDown');
/** Their lengths (s). */
const BUNK_UP_S = 7;
const BUNK_DOWN_S = 6.5;
/** Up: the walk round to the ladder ends (`walk`), the climb from the rails to sitting on the mattress (`sat`), then lain back. */
const BUNK_AT = { walk: 0.2, sat: 0.86 };
/** Down: lain to sitting up (`sat`), the climb back down to the ground (`down`), then the walk round to the tile. */
const BUNK_DOWN_AT = { sat: 0.12, down: 0.8 };

/**
 * Lain on the upper bunk as the runtime has the `lie` clip (turned flat by its root's -90 degree pitch about
 * the feet, `h - 0.91` m towards the foot): the same pose with the pelvis turned flat instead.
 */
function lainOnBunk(fit: BunkFit): PoseSpec {
  const lie = liePose(0);
  const root: Vec = [0, (BUNK.top + (0.1 * fit.h) / 1.77) / fit.s, (fit.h - 0.91) / fit.s];
  const { pelvis } = rest();
  const p = lie.pelvis ?? [0, 0, 0];
  // (Turned a whole way round, as the round ends sat facing the foot end: blended into, it doesn't turn back.)
  return { ...lie, pelvis: [p[0] - 90, p[1] + 360, p[2]], move: [0, root[1] - pelvis[1], root[2] - pelvis[1]] };
}

/**
 * Up the ladder into the upper bunk: from the tile in front of the foot end round the bed's right side to the
 * ladder, turned to face it; hands onto its rails and up the four rungs, hand and foot in turn; hands onto the
 * mattress, a knee up onto it and the other, in on hands and knees through the gap in the guard rail; sat down,
 * the legs swung round to lie along the bed, sitting up; lain back (into `lie`).
 */
function bunkUpRound(fit: BunkFit): Round {
  const { s } = fit;
  const C = (x: number, y: number, z: number): Vec => [x / s, y / s, z / s];
  const r = new Round();
  const { pelvis, ankle } = rest();
  const aY = ankle.l[1];
  // Standing on the tile, facing the bed.
  const start = C(0, 0, BUNK.tile);
  r.body(0, { at: [start[0], pelvis[1], start[2]], yaw: 180, hip: 0, bend: 0, nod: 0 });
  for (const side of ['l', 'r'] as const) {
    const a = turned([ankle[side][0], 0, ankle[side][2]], 180);
    r.step(side, 0, [start[0] + a[0], aY, start[2] + a[2]], { pitch: 0, knee: 0, yaw: 180, how: 'set' });
    r.grab(side, 0, null, { elbow: 0, how: 'set' });
  }
  // 1. Round the foot end's corner to the ladder, and a turn to face it (the body's right towards the foot).
  const xFoot = BUNK.x + 0.05 + 0.12 * s;
  walk(r, [start, C(0.72, 0, 1.25), C(xFoot + 0.1 * s, 0, BUNK_Z)], 0.01, BUNK_AT.walk, { first: 'r', face: 270 });
  const yaw = 270;
  // The ladder (the bed's metres): a foot on rung `i` (0 the ground), a palm on a rail at a height.
  const foot = (side: Side, i: number): Vec => C(BUNK.x + 0.12 * s, (i ? BUNK.rungs[i - 1] + 0.016 : 0) + aY * s, BUNK_Z + (side === 'l' ? -1 : 1) * 0.1 * s);
  const palm = (side: Side, y: number): Vec => C(BUNK.x + 0.035, Math.min(1.6, y), side === 'l' ? BUNK.rails[0] + 0.03 : BUNK.rails[1] - 0.03);
  const hips = (y: number): Vec => C(BUNK.x + 0.3 * s, y + 0.74 * s, BUNK_Z);
  // 2. Hands onto the rails at the chest, then up the rungs, hand and foot in turn.
  const c0 = BUNK_AT.walk + 0.02;
  const c1 = 0.52;
  r.grab('r', c0 + 0.02, palm('r', 1.0 * s), { from: c0 - 0.02 });
  r.grab('l', c0 + 0.03, palm('l', 1.08 * s), { from: c0 - 0.01 });
  r.body(c0 + 0.03, { at: hips(0), yaw, hip: 6, nod: -12 });
  const n = BUNK.rungs.length;
  const dm = (c1 - c0 - 0.03) / n;
  for (let i = 1; i <= n; i++) {
    const t = c0 + 0.03 + dm * i;
    const side: Side = i % 2 ? 'r' : 'l';
    const other: Side = side === 'r' ? 'l' : 'r';
    r.step(side, t - dm * 0.35, foot(side, i), { from: t - dm, pitch: -8 });
    r.body(t - dm * 0.1, { at: hips(BUNK.rungs[i - 1]), yaw, hip: 10, nod: -15 });
    r.grab(other, t, palm(other, BUNK.rungs[i - 1] + 1.05 * s), { from: t - dm * 0.3 });
    // (The trailing foot up beside it at the top.)
    if (i === n) r.step(other, t + dm * 0.4, foot(other, i), { from: t + dm * 0.05, pitch: -8 });
    else r.grab(side, t + dm * 0.4, palm(side, BUNK.rungs[i - 1] + 1.12 * s), { from: t + dm * 0.05 });
  }
  // 3. Hands onto the mattress, leant in over it, a knee up onto it and the other, in on hands and knees
  // towards the head end.
  const o0 = c1;
  const o1 = 0.7;
  const ok = (k: number) => o0 + (o1 - o0) * k;
  const top = BUNK.top;
  /** On hands and knees on the mattress, the hips over the knees at (x, z) of the bed, facing `y` (deg). */
  const kneel = (x: number, z: number, y: number) => {
    const at = C(x, top + 0.5 * s, z);
    const fwd = turned([0, 0, 1], y);
    const right = turned([1, 0, 0], y);
    const on = (side: Side, ahead: number, out: number, up: number): Vec => {
      const k = side === 'l' ? -1 : 1;
      return [at[0] + fwd[0] * ahead + right[0] * k * out, (top + up * s) / s, at[2] + fwd[2] * ahead + right[2] * k * out];
    };
    return { at, l: on('l', -0.34, 0.1, 0.06), r: on('r', -0.34, 0.1, 0.06), lh: on('l', 0.45, 0.13, 0.02 / s), rh: on('r', 0.45, 0.13, 0.02 / s) };
  };
  const edge = kneel(0.42, BUNK_Z, yaw);
  r.grab('l', ok(0.2), edge.lh, { from: ok(0.02), elbow: 1 });
  r.grab('r', ok(0.28), edge.rh, { from: ok(0.1), elbow: 1 });
  r.body(ok(0.3), { at: hips(BUNK.rungs[n - 1] + 0.05), hip: 50, bend: 10, nod: -30 });
  r.step('r', ok(0.55), edge.r, { from: ok(0.3), pitch: 140, knee: -1 });
  r.body(ok(0.6), { at: kneel(0.5, BUNK_Z, yaw).at, hip: 82, bend: 5, nod: -40 });
  r.step('l', ok(0.72), edge.l, { from: ok(0.55), pitch: 140, knee: -1 });
  r.body(ok(0.75), { at: edge.at });
  const inside = kneel(0.2, 0.48, 250);
  r.grab('l', ok(0.82), inside.lh, { from: ok(0.7) });
  r.grab('r', ok(0.9), inside.rh, { from: ok(0.78) });
  r.step('r', ok(0.95), inside.r, { from: ok(0.82), yaw: 250 });
  r.step('l', ok(1), inside.l, { from: ok(0.88), yaw: 250 });
  r.body(o1, { at: inside.at, yaw: 250, hip: 80, bend: 5, nod: -40 });
  // 4. Sat down onto the hip, a turn to face the foot end with the legs swung round along the bed, sitting up
  // where the hips will lie (near enough: lying back takes them the rest of the way).
  const t0 = o1;
  const t1 = BUNK_AT.sat;
  const tk = (k: number) => t0 + (t1 - t0) * k;
  const lieZ = fit.h - 0.91 - 0.947 * s + 0.15;
  const sat = C(0, top + 0.174 * s, lieZ);
  r.grabBoth(tk(0.35), null, null, { from: tk(0.05) });
  r.body(tk(0.45), { at: [(sat[0] + inside.at[0]) / 2, sat[1] + 0.1, (sat[2] + inside.at[2]) / 2], yaw: 315, hip: 30, bend: 5, nod: 0 });
  const out = (side: Side): Vec => C((side === 'l' ? -1 : 1) * 0.1 * s, top + 0.07 * s, lieZ + 0.78 * s);
  r.step('l', tk(0.75), out('l'), { from: tk(0.3), pitch: -30, knee: 1, yaw: 360 });
  r.step('r', tk(0.85), out('r'), { from: tk(0.4), pitch: -30, knee: 1, yaw: 360 });
  r.body(t1, { at: sat, yaw: 360, hip: -12, bend: 0, nod: 5 });
  return r.done();
}

/** Getting into the upper bunk: the round up to sitting on the mattress, then lain back into the `lie` pose. */
function bunkUpPose(fit: BunkFit): (p: number) => PoseSpec {
  let made: { r: Round; lain: PoseSpec } | null = null;
  return (p) => {
    made ??= { r: bunkUpRound(fit), lain: lainOnBunk(fit) };
    const at = BUNK_AT.sat;
    return p <= at ? posed(made.r, p, p * 2) : mix(posed(made.r, at, at * 2), made.lain, smooth((p - at) / (1 - at)));
  };
}

/**
 * Getting up from the upper bunk: sitting up from lying, then the way up backwards (the legs swung round, onto
 * hands and knees facing the bed's far side, backing out to the ladder and down it facing it), and a walk round
 * the foot end's corner to the tile in front, facing away from the bed.
 */
function bunkDownPose(fit: BunkFit): (p: number) => PoseSpec {
  let made: { r: Round; lain: PoseSpec } | null = null;
  const make = () => {
    const up = bunkUpRound(fit);
    const r = up.reversed(BUNK_AT.walk, BUNK_AT.sat, BUNK_DOWN_AT.sat, BUNK_DOWN_AT.down);
    const { s } = fit;
    const C = (x: number, y: number, z: number): Vec => [x / s, y / s, z / s];
    walk(r, [Round.last(r.pel), C(0.72, 0, 1.2), C(0, 0, BUNK.tile)], BUNK_DOWN_AT.down, 0.99, { first: 'l', face: 360 });
    return { r: r.done(), lain: lainOnBunk(fit) };
  };
  return (p) => {
    made ??= make();
    const at = BUNK_DOWN_AT.sat;
    return p >= at ? posed(made.r, p, p * 2) : mix(made.lain, posed(made.r, at, at * 2), smooth(p / at));
  };
}

// --- the rides, sat astride ---------------------------------------------------------------------

/**
 * Sat on a ride (characters.ts `RIDES`): the pelvis where `sit` has it (the runtime fits the seat to it), the
 * hands on the ride's grips and the legs astride it, placed from the ride's measures: metres from the seat's
 * point under the hips, in the rider's frame (to their right, up, ahead). A size is for a body whose legs are
 * `s` of the reference's, its pelvis joint `drop` m over the seat (rig.json `seatDrop`).
 */
interface RideFit {
  s: number;
  drop: number;
}
/** A grown-up's, a teen's and a child's (`h` m tall: the size's name, and the shortest it is for). */
const RIDE_SIZES: readonly (RideFit & { h: number })[] = [
  { h: 1.72, s: 0.97, drop: 0.146 },
  { h: 1.47, s: 0.83, drop: 0.123 },
  { h: 1.12, s: 0.57, drop: 0.099 },
];
type RideSize = { clip: string; min: number; fit: RideFit };
/** A ride's clips in the sizes of `RIDE_SIZES` from `from` down to `to` (indices), named `name` and the height (cm). */
function rideSizes(name: string, from: number, to: number): RideSize[] {
  const own = RIDE_SIZES.slice(from, to + 1);
  return own.map((f, i) => ({ clip: `${name}${Math.round(f.h * 100)}`, min: i < own.length - 1 ? (f.h + own[i + 1].h) / 2 : 0, fit: f }));
}
/** The seated pelvis (the clip's metres): `sit`'s. */
const SAT: Vec = [0, 0.947 + SEAT_MOVE[1], SEAT_MOVE[2]];
/** A place on the ride (m from the seat's point, the rider's frame) in the clip's metres. */
const onRide = (f: RideFit, p: Vec): Vec => [p[0] / f.s, SAT[1] + (p[1] - f.drop) / f.s, SAT[2] + p[2] / f.s];

/**
 * A rider's pose: sat, leant forward `lean` deg, the head's nod; the ankles at `feet` (the left; the right
 * mirrored; the clip's metres, absolute), the knees pointing forward (`knees` 0) to out to the side (1), raised
 * by `up`; the palms at `hands` (the left and the right, absolute) on the grips, rolled by `pronate`.
 */
function astride(o: { lean: number; nod?: number; breath: number; feet: Vec; knees: number; up?: number; toes?: number; hands: [Vec, Vec]; pronate?: number }): PoseSpec {
  const body = seatedBody({ breath: o.breath, depth: 0.7, lean: o.lean, head: [o.nod ?? 0, 0, 0], tilt: -6 + o.lean * 0.3 });
  const { ankle } = rest();
  const out: PoseSpec = { ...body };
  for (const side of ['l', 'r'] as const) {
    const sx = side === 'l' ? -1 : 1;
    const at: Vec = side === 'l' ? o.feet : [-o.feet[0], o.feet[1], o.feet[2]];
    const pole: Vec = [sx * (0.15 + 0.85 * o.knees), o.up ?? 0.2, 1 - 0.5 * o.knees];
    Object.assign(out, footIK(side, sub(at, ankle[side]), body, { foot: [o.toes ?? 10, sx * (6 + 20 * o.knees), 0], pole }));
  }
  const [l, r] = o.hands;
  return {
    ...out,
    ...armIK('l', l, body, { palm: true, pole: [-0.8, -0.6, -0.2], pronate: o.pronate ?? 70, hand: wrist('l', -10, 0) }),
    ...armIK('r', r, body, { palm: true, pole: [0.8, -0.6, -0.2], pronate: o.pronate ?? 70, hand: wrist('r', -10, 0) }),
  };
}

/**
 * The rocking horse (kids, `model.rockingHorse`, a child's): its saddle 0.56 m up over the middle, the handle's
 * grips 0.115 m either side, 0.12 m over it and 0.28 m ahead, at the head; the barrel 0.2 m across under the
 * saddle. Astride, the thighs either side of the barrel, the feet down by the rockers, leant to the handle;
 * the horse rocks the rider with it (its `ride` part).
 */
export const RIDE_HORSE = rideSizes('rideHorse', 2, 2);
function horsePose(f: RideFit): (p: number) => PoseSpec {
  return (p) =>
    astride({
      lean: 16 + 4 * Math.sin(p * TAU),
      nod: -10,
      breath: p,
      feet: onRide(f, [-0.17, -0.36, 0.06]),
      knees: 0.8,
      hands: [onRide(f, [-0.115, 0.12, 0.28]), onRide(f, [0.115, 0.12, 0.28])],
    });
}

/**
 * The spring rider (playground, `model.springRider`, a child's): its saddle 0.62 m up, the handles' ends 0.16 m
 * either side, 0.15 m over it and 0.28 m ahead, the foot pegs 0.19 m either side, 0.28 m under it and 0.1 m
 * ahead. Astride, the feet on the pegs, the hands on the handles.
 */
export const RIDE_SPRING = rideSizes('rideSpring', 2, 2);
function springPose(f: RideFit): (p: number) => PoseSpec {
  return (p) =>
    astride({
      lean: 12 + 3 * Math.sin(p * TAU),
      nod: -6,
      breath: p,
      feet: onRide(f, [-0.19, -0.23, 0.05]),
      knees: 0.7,
      toes: 0,
      hands: [onRide(f, [-0.15, 0.15, 0.28]), onRide(f, [0.15, 0.15, 0.28])],
    });
}

/**
 * The seesaw (playground, `model.seesaw`, a child's or a teen's): a seat at each end of the plank, 0.56 m up, the
 * rider at its slot's place 0.78 m out facing along it to the middle (content `places`: the two face each other),
 * its handle bar 0.31 m ahead, 0.26 m over the seat, the grips 0.1 m either side; the plank 0.24 m wide. Astride
 * the plank, the feet down either side of it, tucked up clear of the ground at the low end (the plank tips 12 deg).
 */
export const RIDE_SEESAW = rideSizes('rideSeesaw', 1, 2);
function seesawPose(f: RideFit): (p: number) => PoseSpec {
  // (A teen's longer legs tucked further back.)
  const feet: Vec = f.s > 0.7 ? [-0.2, -0.38, 0.05] : [-0.17, -0.34, 0.12];
  return (p) =>
    astride({
      lean: 12 + 3 * Math.sin(p * TAU),
      nod: -4,
      breath: p,
      feet: onRide(f, feet),
      knees: 0.6,
      hands: [onRide(f, [-0.1, 0.26, 0.31]), onRide(f, [0.1, 0.26, 0.31])],
      pronate: 80,
    });
}

/**
 * The merry-go-round (playground, `model.merryGoRound`, a child's or a teen's): sat on its platform (0.26 m up) at
 * a slot's place, 0.7 m out from its middle facing in (content `places`), between two of its four bent rails,
 * which run on the diagonals from the column's top (0.9 m up) out to 0.78 m and down to the platform. Leant in,
 * a hand on each rail where it comes down beside them (`roundGrip`: m out along the rail, for the size's reach),
 * the legs out ahead towards the middle, the knees up.
 */
export const RIDE_ROUND = rideSizes('rideRound', 1, 2);
const ROUND = { place: 0.7, seat: 0.26, top: 0.9, out: 0.78, column: 0.05, foot: 0.25 };
/** How far out along a rail (m from the middle) a rider of size `s` holds it: further down it for a child's shorter arms. */
const roundGrip = (s: number) => (s > 0.7 ? 0.66 : 0.6);
/** The rail's height (m) where it is `r` m out from the middle (the model's curve: level at the column, down at its end). */
function roundRailY(r: number): number {
  const u = Math.sqrt(Math.max(0, (ROUND.out - r) / (ROUND.out - ROUND.column)));
  return ROUND.top - (ROUND.top - ROUND.foot) * (1 - u) * (1 - u);
}
function roundPose(f: RideFit): (p: number) => PoseSpec {
  const r = roundGrip(f.s);
  // The rails on the rider's left and right, out along the diagonals (m from the seat's point, the rider's frame).
  const side = r / Math.SQRT2;
  const grip = (k: number): Vec => onRide(f, [k * side, roundRailY(r) - ROUND.seat, ROUND.place - side]);
  const feet: Vec = f.s > 0.7 ? [-0.11, 0.08, 0.42] : [-0.1, 0.06, 0.36];
  return (p) =>
    astride({
      lean: 16 + 2 * Math.sin(p * TAU),
      nod: -6,
      breath: p,
      feet: onRide(f, feet),
      knees: 0.15,
      up: 1,
      toes: -10,
      hands: [grip(-1), grip(1)],
      pronate: 20,
    });
}

/**
 * The swing (outdoor, `model.swingSet`, anyone's): its seat 0.5 m up, the chains 0.17 m either side of the
 * rider's middle, straight up from the seat. Both hands on the chains at the chest; pumping in time with the
 * swing (characters.ts syncs the clip to its loop, 1.6 s): leant back with the legs out straight on the way
 * forward, sat up with them tucked under the seat on the way back. The seat swings the rider with it.
 */
export const RIDE_SWING = rideSizes('rideSwing', 0, 2);
export const SWING_S = 1.6;
function swingPose(f: RideFit): (p: number) => PoseSpec {
  const grip = (x: number): Vec => onRide(f, [x, 0.6 * f.s, 0]);
  return (p) => {
    // The swing is back at the loop's quarter, forward at three quarters: the legs out on the way forward.
    const e = 0.5 - 0.5 * Math.cos(TAU * (p - 0.2));
    const tucked: Vec = [-0.1, 0.2, -0.2];
    const out: Vec = [-0.1, 0.44, 0.52];
    return astride({
      lean: 8 - 30 * e,
      nod: 6 - 4 * e,
      breath: 0,
      feet: lerpV(tucked, out, smooth(e)),
      knees: 0,
      up: 0.6,
      toes: 10 + 20 * e,
      hands: [grip(-0.17), grip(0.17)],
      pronate: 0,
    });
  };
}

export function rideClips(): ClipDef[] {
  return [
    ...SLIDE.map(({ clip, fit }) => cycle(clip, { seconds: SLIDE_S, loop: true }, slidePose(fit))),
    ...BUNK_UP.map(({ clip, fit }) => cycle(clip, { seconds: BUNK_UP_S, loop: false }, bunkUpPose(fit))),
    ...BUNK_DOWN.map(({ clip, fit }) => cycle(clip, { seconds: BUNK_DOWN_S, loop: false }, bunkDownPose(fit))),
    ...RIDE_HORSE.map(({ clip, fit }) => cycle(clip, { seconds: 2.4, loop: true }, horsePose(fit))),
    ...RIDE_SPRING.map(({ clip, fit }) => cycle(clip, { seconds: 2, loop: true }, springPose(fit))),
    ...RIDE_SEESAW.map(({ clip, fit }) => cycle(clip, { seconds: 2.8, loop: true }, seesawPose(fit))),
    ...RIDE_ROUND.map(({ clip, fit }) => cycle(clip, { seconds: 2.4, loop: true }, roundPose(fit))),
    ...RIDE_SWING.map(({ clip, fit }) => cycle(clip, { seconds: SWING_S, loop: true }, swingPose(fit))),
  ];
}
