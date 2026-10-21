/**
 * Rides and climbs at things: a child's round on the slide (`STEP_IN` in characters.ts: round to its ladder,
 * up it, in under the bar onto the deck, sat at the top holding the hoop, down the chute on the back, off at
 * the run-out and back round to the front), the upper bunk's ladder (characters.ts `BUNK_LADDER`: up it and
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

import { cycle, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, hangArms } from './idle';
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
 * deck's top 1.43 m up, 0.72 m to 0.28 m behind the middle, a bar across its back at 1.86 m and the roof's eaves
 * 1.93 m up; a hoop over the chute's mouth (0.27 m round its middle, 1.43 m up, 0.27 m behind the middle); the
 * chute from the deck's front (1.43 m up, 0.29 m behind) down a smoothstep to its run-out, 0.28 m up and
 * level at 0.95 m in front, its floor 2.5 cm thick.
 */
const SLIDE_LADDER = { rail: 0.25, foot: -0.94, top: [1.42, -0.7] as const, rung: 0.2367, rungs: 5 };
const SLIDE_DECK = 1.43;
const SLIDE_HOOP = { y: 1.43, z: -0.27, r: 0.27 };
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
 * foot in turn; ducking under the bar across the deck's back, hands onto the deck, a knee up and in on hands
 * and knees; sat at the top, the legs into the chute, holding the hoop, a look down; a push, lain back under the
 * hoop, down the chute on the back (the body on its floor, the legs ahead along it), sat up on the run-out, the
 * feet down, up off it, and a turn round back to where the round began.
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
    // 3. Over the top: the chest down under the bar, hands onto the deck, a knee up onto it and the other, in on
    // hands and knees to sit at the front.
    const o0 = c1;
    const o1 = 0.55;
    const ok = (k: number) => o0 + (o1 - o0) * k;
    const knee = (side: Side, z: number): Vec => C((side === 'l' ? -1 : 1) * 0.1 * s, SLIDE_DECK + 0.06 * s, z - 0.34 * s);
    const deckPalm = (side: Side, z: number): Vec => C((side === 'l' ? -1 : 1) * 0.13 * s, SLIDE_DECK + 0.02, z);
    const crawlHips = (z: number): Vec => C(0, SLIDE_DECK + 0.5 * s, z);
    r.grab('l', ok(0.2), deckPalm('l', -0.56), { from: ok(0.04), elbow: 1 });
    r.grab('r', ok(0.3), deckPalm('r', -0.53), { from: ok(0.14), elbow: 1 });
    r.body(ok(0.3), { at: climbHips(rung(5)[0] - 0.05), hip: 55, bend: 10, nod: -35 });
    r.step('l', ok(0.55), knee('l', -0.62), { from: ok(0.32), pitch: 140, knee: -1 });
    r.body(ok(0.58), { at: crawlHips(-0.75), hip: 82, bend: 5, nod: -45 });
    r.step('r', ok(0.72), knee('r', -0.6), { from: ok(0.55), pitch: 140, knee: -1 });
    r.grab('l', ok(0.82), deckPalm('l', -0.4), { from: ok(0.7) });
    r.body(ok(0.85), { at: crawlHips(-0.66), hip: 82, bend: 5, nod: -45 });
    r.grab('r', ok(0.92), deckPalm('r', -0.38), { from: ok(0.8) });
    r.step('l', ok(0.95), knee('l', -0.52), { from: ok(0.84) });
    r.step('r', ok(1), knee('r', -0.5), { from: ok(0.9) });
    r.body(o1, { at: crawlHips(-0.58), hip: 80, bend: 5, nod: -45 });
    // 4. Sat at the top: down onto the bottom, the legs out into the chute (under the hoop), the hands onto the
    // hoop's sides; leant forward under the roof, a look down the slide.
    const t0 = o1;
    const t1 = 0.61;
    const tk = (k: number) => t0 + (t1 - t0) * k;
    const zSit = -0.5;
    const sitAt = C(0, SLIDE_DECK + sd * s, zSit);
    const legOut = (side: Side, z: number): Vec => C((side === 'l' ? -1 : 1) * 0.09 * s, chuteY(z) + 0.07 * s, z);
    r.body(tk(0.45), { at: [sitAt[0], sitAt[1] + 0.12, sitAt[2] + 0.1], hip: 40, bend: 10, nod: -15 });
    r.step('l', tk(0.6), legOut('l', zSit + 0.72 * s), { from: tk(0.25), pitch: -20, knee: 1 });
    r.step('r', tk(0.7), legOut('r', zSit + 0.72 * s), { from: tk(0.35), pitch: -20, knee: 1 });
    r.body(tk(0.75), { at: sitAt, hip: 28, bend: 12, nod: 8 });
    const hoop = (side: Side, a: number): Vec => C((side === 'l' ? -1 : 1) * SLIDE_HOOP.r * Math.cos((a * Math.PI) / 180), SLIDE_HOOP.y + SLIDE_HOOP.r * Math.sin((a * Math.PI) / 180), SLIDE_HOOP.z - 0.03);
    r.grab('l', tk(0.8), hoop('l', 35), { from: tk(0.45), elbow: 0 });
    r.grab('r', tk(0.85), hoop('r', 35), { from: tk(0.5), elbow: 0 });
    r.body(t1, { nod: 20 });
    // 5. A pull on the hoop and lain back under it, the hands let go; down the chute on the back, the body on its
    // floor and the legs ahead along it; sat up on the run-out.
    const s0 = t1 + 0.02;
    const s1 = 0.71;
    r.body(s0, { at: C(0, SLIDE_DECK + sd * s, -0.42), hip: 20, bend: 8, nod: 10 });
    const n = 12;
    const zEnd = CHUTE.end[1] - 0.12;
    const zFrom = -0.38;
    for (let k = 1; k <= n; k++) {
      const u = k / n;
      // Quick down the steep middle, slowing on the level run-out.
      const e = 1 - (1 - u) * (1 - u);
      const t = s0 + (s1 - s0) * u;
      const z = zFrom + (zEnd - zFrom) * e;
      const slope = chuteSlope(z);
      // Lain back (the back 15 deg off the chute) under the hoop and down the steep part, sitting up on the level.
      const up = smooth((u - 0.6) / 0.4);
      const lean = 15 + 75 * up;
      const y = chuteY(z) + (0.11 + 0.06 * up) * s;
      r.body(t, { at: C(0, y, z), hip: slope - (90 - lean), bend: -4 + 10 * up, nod: 25 - 20 * up }, 'lin');
      // The legs out ahead along the chute (the heels on it, the knees a little bent), off its end at the last.
      let zf = z;
      while (zf < CHUTE.end[1] + 0.05 && Math.hypot(zf - z, chuteY(zf) - y) < 0.72 * s) zf += 0.01;
      for (const side of ['l', 'r'] as const) r.step(side, t, legOut(side, zf), { pitch: -15, knee: 0.6, how: 'lin' });
      if (k === 2) r.grabBoth(t, null, null, { elbow: 0, how: 'set' });
    }
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

export function rideClips(): ClipDef[] {
  return [
    ...SLIDE.map(({ clip, fit }) => cycle(clip, { seconds: SLIDE_S, loop: true }, slidePose(fit))),
  ];
}
