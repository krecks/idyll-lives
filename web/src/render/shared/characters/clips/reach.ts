/**
 * Reaching up to things at fixed heights, standing (characters.ts `STEP_IN` steps the user up to each and
 * picks the size for their height, as the street's clips, `street.ts`): clean things put away in a wall
 * cupboard (its pulls reached and the doors drawn open, a stack lifted from the worktop onto the shelf, a door
 * pushed shut), the longcase clock wound at its dial, the songbird feeder's house filled from a scoop, and
 * lines written on the whiteboard; reaching down into the chest freezer under its lid, and the doorbell pressed. Each size is fitted to a body of its share `s` of the reference skeleton's
 * size standing `to` m from the object's middle: the object's points (m: `x` to the user's right, `y` up, `z`
 * from its middle towards the user) divided by `s` give the clip's. They go up on tiptoe, lift the shoulder
 * and lean in as far as the hands need (`upFor`), and reach as far as they can if it is out of reach: a child
 * can't reach a wall cupboard or the clock's dial, and puts the things on the worktop under it, or pulls the
 * clock's chains in its trunk instead. The dishes, the key, the scoop and the marker are mimed.
 */

import { cycle, mix, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { track, tiptoe, type Key } from './games';
import { bump, hangArms, stand, TAU } from './idle';
import { armIK, footIK, len, lerpV, pointOn, sub, wrist, type ArmOptions } from './ik';
import { smooth } from '../../../../game/noise';
import { SHARES_177, sizes, type Fit, type Size } from './street';
import { stoopBody } from './uses';

/** The smallest share that reaches a wall cupboard's pulls or the clock's dial (a teen's; a child's doesn't). */
const REACHES_UP = 0.75;
/** From the shoulder to the palm's centre, a little short of straight (m, reference skeleton). */
const ARM = 0.57;
/** The most a reach goes up on tiptoe (m, the clip's), lifts the shoulder (deg) and leans in (deg). */
const UP = { tip: 0.08, shrug: 18, lean: 34 };

/** A point of the object (m: to the user's right, up, from its middle towards the user) in the clip's frame. */
function at(fit: Fit, x: number, y: number, z: number): Vec {
  return [x / fit.s, y / fit.s, (fit.to - z) / fit.s];
}

/** A hand's place and how it is held (`armIK`'s options), `w` of the way there from hanging (default all). */
type Hand = ArmOptions & { at: Vec; w?: number };

/**
 * Standing, reaching up: on tiptoe by `tip` (m, the clip's), the shoulders lifted `shrug` degrees (the side
 * reaching most, `side`, all of it), the back leant in `lean` degrees (the hips back to keep over the feet),
 * the head looking `head` (nod, turn, tilt); the weight to the right by `shift`.
 */
function upBody(tip: number, shrug: number, lean: number, head: Vec, p: number, shift: number, side: number): PoseSpec {
  const b = stand({ breath: p, depth: 0.6, lean, shift, head, fwd: -0.0018 * Math.max(0, lean) });
  const cl = b.clavicle_l ?? [0, 0, 0];
  const cr = b.clavicle_r ?? [0, 0, 0];
  const body: PoseSpec = {
    ...b,
    clavicle_l: [cl[0], cl[1], cl[2] - shrug * (side < 0 ? 1 : 0.4)],
    clavicle_r: [cr[0], cr[1], cr[2] + shrug * (side > 0 ? 1 : 0.4)],
  };
  if (tip <= 0.0005) return body;
  const lifted: PoseSpec = { ...body, move: [body.move![0], body.move![1] + tip, body.move![2]] };
  const [ankle, deg] = tiptoe(tip);
  return { ...lifted, ...footIK('l', ankle, lifted, { foot: [deg, -4, 0] }), ...footIK('r', ankle, lifted, { foot: [deg, 4, 0] }), ball_l: [-deg, 0, 0], ball_r: [-deg, 0, 0] };
}

/** Looking ahead, a little down (the clip's frame). */
const AHEAD: Vec = [0, 1.45, 1.5];
/** Where a hanging hand's palm is, about (the clip's frame): for finding the reach of a hand on its way from hanging. */
const REST_L: Vec = [-0.2, 0.9, 0.06];
const REST_R: Vec = [0.2, 0.9, 0.06];

/**
 * The body for reaching the hands' places (a null hand hangs): how far up on tiptoe, the shoulder lifted and
 * leant in, the least that brings each place within reach, found by halving along one path (a place above the
 * shoulders: up on tiptoe and the shoulder lifted first, then leant in as far as it is out in front; a lower
 * one: leant in), all of it if out of reach; `lean` more leant in anyway. The head looks at `look`. A hand on its
 * way from hanging (`w` under 1) goes from where the hanging hand's palm is, so it leaves and comes back to it.
 */
function upFor(l: Hand | null, r: Hand | null, look: Vec | null, p: number, o: { lean?: number; shift?: number; tip?: boolean } = {}): PoseSpec {
  const goals: [side: 'l' | 'r', at: Vec][] = [];
  if (l) goals.push(['l', lerpV(REST_L, l.at, l.w ?? 1)]);
  if (r) goals.push(['r', lerpV(REST_R, r.at, r.w ?? 1)]);
  const top = Math.max(0, ...goals.map(([, g]) => g[1]));
  // (How much of the path is up on tiptoe: places above about the shoulders' height; how far it leans in: as
  // far as a place is out in front of the shoulders.)
  const high = Math.min(1, Math.max(0, (top - 1.3) / 0.25));
  const out = Math.max(0, ...goals.map(([, g]) => g[2]));
  const most = UP.lean * (1 - high + high * Math.min(1, Math.max(0, (out - 0.3) / 0.3)));
  const side = r && l ? ((r.w ?? 1) * r.at[1] >= (l.w ?? 1) * l.at[1] ? 1 : -1) : r ? 1 : -1;
  const shift = o.shift ?? 0.2;
  const leanAt = (u: number) => (o.lean ?? 0) + most * (high * Math.max(0, 2 * u - 1) + (1 - high) * u);
  const make = (u: number, head: Vec) => upBody((o.tip === false ? 0 : UP.tip) * high * Math.min(1, 2 * u), UP.shrug * high * Math.min(1, 2 * u), leanAt(u), head, p, shift, side);
  const short = (u: number) => {
    const b = make(u, [0, 0, 0]);
    return Math.max(-1, ...goals.map(([s, g]) => len(sub(g, pointOn(b, `upperarm_${s}`, [0, 0, 0]))) - ARM));
  };
  let u = 0;
  if (goals.length && short(0) > 0) {
    let lo = 0;
    let hi = 1;
    if (short(1) > 0) lo = hi;
    else for (let k = 0; k < 12; k++) [lo, hi] = short((lo + hi) / 2) > 0 ? [(lo + hi) / 2, hi] : [lo, (lo + hi) / 2];
    u = hi;
  }
  // The head turned and nodded to the place looked at, from the eyes (the lean already pitches it a third).
  let head: Vec = [6, 0, 0];
  if (look) {
    const eye = pointOn(make(u, [0, 0, 0]), 'Head', [0, 0.08, 0.08]);
    const d = sub(look, eye);
    const elev = (Math.atan2(d[1], Math.hypot(d[0], d[2])) * 180) / Math.PI;
    head = [Math.max(-45, Math.min(40, -elev - 0.3 * leanAt(u))), ((Math.atan2(d[0], d[2]) * 180) / Math.PI) * 0.8, 0];
  }
  return handsOn(make(u, head), l, r);
}

/** The hands at their places on `body` (a null hand hangs; one on its way from hanging, `w` under 1, blended from the hanging arm). */
function handsOn(body: PoseSpec, l: Hand | null, r: Hand | null): PoseSpec {
  const hang: PoseSpec = { ...body, ...hangArms(5, 8, 16) };
  const ref = reference();
  const arm = (s: 'l' | 'r', h: Hand | null): PoseSpec => {
    if (!h) return {};
    const sx = s === 'l' ? -1 : 1;
    const w = h.w ?? 1;
    const opts: ArmOptions = { palm: true, pole: [0.7 * sx, -0.6, -0.4], ...h };
    if (w >= 1) return armIK(s, h.at, body, opts);
    // (On the way from hanging: placed along the way from the hanging palm, and blended from the hanging arm.)
    const from = pointOn(hang, `hand_${s}`, [ref.grip[0] * -sx, ref.grip[1], ref.grip[2]]);
    const bones = [`upperarm_${s}`, `lowerarm_${s}`, `hand_${s}`] as const;
    return mix(Object.fromEntries(bones.map((b) => [b, hang[b]])) as PoseSpec, armIK(s, lerpV(from, h.at, w), body, opts), w);
  };
  return { ...hang, ...arm('l', l), ...arm('r', r) };
}

/** A hand's track (in seconds; a null place: hanging): its place at `t` (held from the nearest place while hanging) and how far from hanging it is (0 hanging, 1 at it). */
type Track = readonly (readonly [t: number, v: Vec | null])[];
function placed(keys: Track, t: number): [Vec, number] {
  // (A hand going back to hanging holds its last place, one coming from it the next.)
  const k: Key<Vec>[] = keys.map(([kt, v], i) => [kt, v ?? (i > 0 ? keys[i - 1][1] : null) ?? keys.slice(i).find(([, n]) => n)?.[1] ?? REST_R]);
  const w: Key<number>[] = keys.map(([kt, v]) => [kt, v ? 1 : 0]);
  return [track(k, t), track(w, t)];
}

/** A track's hand at `t` (null while it hangs), held as `opts` say. */
function handOn(keys: Track, t: number, opts: ArmOptions): Hand | null {
  const [place, w] = placed(keys, t);
  return w > 0.001 ? { ...opts, at: place, w } : null;
}

// --- wall cupboards -----------------------------------------------------------------------------

/**
 * A cupboard's door that opens: its hinge and pull (m to the user's right), the pull's height and depth, the
 * hand that opens it. Only the doors the model's `doors_open` or `door_open` swings
 * (crates/art/wet/src/kitchen_counters, dining.py): both of the wall cabinet's, the glazed one's right door (the user's left), the corner one's single
 * door, the china cabinet's left glazed door (the user's right). They open 100 degrees in 0.8 s as the use
 * starts and close after 1.5 s more (the clip's `CUPBOARD_S` round fits it).
 */
interface Door {
  hand: 'l' | 'r';
  hinge: number;
  pull: number;
  y: number;
  z: number;
}
/** A cupboard: its doors, where the stack is taken from (the worktop under it, the china cabinet's base) and put (the shelf), and how far from its middle the user stands, by their share of the size. */
interface Cupboard {
  doors: readonly Door[];
  from: Vec;
  shelf: Vec;
  to: (s: number) => number;
}
const WALL_PULL = { y: 1.56, z: -0.04 };
const CUPBOARDS: Record<string, Cupboard> = {
  // Over a 0.6 m deep worktop (its front about 0.17 m out): the toes up to it.
  putAway: {
    doors: [
      { hand: 'r', hinge: 0.487, pull: 0.06, ...WALL_PULL },
      { hand: 'l', hinge: -0.487, pull: -0.06, ...WALL_PULL },
    ],
    from: [0, 0.95, 0.08],
    shelf: [0, 1.51, -0.24],
    to: (s) => 0.36 + 0.08 * s,
  },
  putAwayGlass: { doors: [{ hand: 'l', hinge: -0.48, pull: -0.04, y: 1.6, z: -0.02 }], from: [0, 0.95, 0.08], shelf: [-0.18, 1.51, -0.25], to: (s) => 0.36 + 0.08 * s },
  putAwayCorner: { doors: [{ hand: 'r', hinge: 0.47, pull: -0.06, y: 1.6, z: -0.02 }], from: [0.1, 0.95, 0.08], shelf: [0.16, 1.51, -0.24], to: (s) => 0.36 + 0.08 * s },
  // The china cabinet stands on the floor, its base's top 0.9 m up and 0.09 m out.
  putAwayHutch: { doors: [{ hand: 'r', hinge: 0.43, pull: 0.04, y: 1.45, z: -0.03 }], from: [0, 0.93, -0.04], shelf: [0.2, 1.43, -0.2], to: (s) => 0.33 + 0.08 * s },
};
/** The cupboards by object def (characters.ts `STEP_IN`). */
export const CUPBOARD_OF: Record<string, string> = { wallCabinet: 'putAway', wallCabinetGlass: 'putAwayGlass', wallCabinetCorner: 'putAwayCorner', chinaCabinet: 'putAwayHutch' };
type CupboardFit = Fit & { cupboard: Cupboard };
export const PUT_AWAY: Record<string, Size<CupboardFit>[]> = Object.fromEntries(
  Object.entries(CUPBOARDS).map(([name, c]) => [name, sizes(name, SHARES_177, (s) => ({ s, to: c.to(s), cupboard: c }))]),
);
const CUPBOARD_S = 3.2;

/** A door's pull opened `deg` degrees (towards the user, about its hinge), m in the object's frame, and a hand's place on it a little in front. */
function pullAt(d: Door, deg: number): [x: number, y: number, z: number] {
  const a = (deg * Math.PI) / 180;
  return [d.hinge + (d.pull - d.hinge) * Math.cos(a), d.y, d.z + Math.abs(d.pull - d.hinge) * Math.sin(a) + 0.03];
}

/**
 * Putting clean things away: up to the pulls (on tiptoe as far as needed) and the doors drawn open along their
 * swing, the hands down to the stack on the worktop, lifted up and slid onto the shelf, out again, and a door
 * pushed shut as it closes. A child, who can't reach, puts the stack on the worktop's back under it, looking up.
 */
function cupboardPose(fit: CupboardFit): (p: number) => PoseSpec {
  const { s, cupboard: c } = fit;
  const reaches = s >= REACHES_UP;
  const o = (v: readonly [number, number, number]) => at(fit, v[0], v[1], v[2]);
  const from = (dx: number) => o([c.from[0] + dx, c.from[1], c.from[2]]);
  const put = reaches ? c.shelf : ([c.shelf[0], c.from[1] + 0.02, c.shelf[2] + (c.from[2] - c.shelf[2]) * 0.45] as const);
  const shelf = (dx: number, dy = 0, dz = 0) => o([put[0] + dx, put[1] + dy, put[2] + dz]);
  const tracks: Record<'l' | 'r', Track> = { l: [], r: [] };
  // The stack in both hands, under it, the palms up.
  const stack = (side: 'l' | 'r') => (side === 'l' ? -0.11 : 0.11);
  for (const side of ['l', 'r'] as const) {
    const door = c.doors.find((d) => d.hand === side);
    const keys: [number, Vec | null][] = [[0, null]];
    if (door && reaches) {
      keys.push([0.45, o(pullAt(door, 0))], [0.6, o(pullAt(door, 0))]);
      for (const [t, deg] of [[0.75, 15], [0.9, 32], [1.05, 48]] as const) keys.push([t, o(pullAt(door, deg))]);
    } else keys.push([0.9, null]);
    keys.push([1.4, from(stack(side))], [1.55, from(stack(side))], [1.95, shelf(stack(side), 0.06, 0.14)], [2.15, shelf(stack(side))], [2.3, shelf(stack(side), 0.02, 0.16)]);
    if (door && reaches) {
      // Its door pushed shut along its swing (the model's closes from 2.3 s to 3.1 s).
      for (const [t, deg] of [[2.55, 70], [2.75, 45], [2.95, 15]] as const) keys.push([t, o(pullAt(door, deg))]);
    }
    keys.push([CUPBOARD_S, null]);
    tracks[side] = keys;
  }
  // Up at the doors (a child at the shelf out of reach), down to the stack, up with it to the shelf, at the door shutting.
  const up = reaches ? o([0, c.doors[0].y, c.doors[0].z]) : o([0, c.shelf[1] + 0.1, c.shelf[2]]);
  const LOOK: Key<Vec>[] = [[0, up], [1.05, up], [1.4, from(0)], [1.6, from(0)], [1.95, shelf(0, 0.05, 0)], [2.35, shelf(0, 0.05, 0)], [2.7, up], [CUPBOARD_S, up]];
  return (p) => {
    const t = p * CUPBOARD_S;
    // The palms up under the stack while it is carried; on a pull, the fist thumb up.
    const carry = bump(t, 1.25, 2.4, 0.12);
    const hand = (side: 'l' | 'r') => {
      const sx = side === 'l' ? -1 : 1;
      return handOn(tracks[side], t, { pronate: 10 - 80 * carry, pole: [0.75 * sx, -0.7, -0.3 + 0.2 * carry], hand: wrist(side, -10 * carry, 0) });
    };
    return upFor(hand('l'), hand('r'), track(LOOK, t), p, { lean: 4 * carry, shift: 0.15 });
  };
}

// --- the longcase clock ---------------------------------------------------------------------------

/**
 * The grandfather clock (crates/art/rooms/src/decor): the dial 1.77 m up, its face 0.044 m behind the middle; the winding
 * holes either side under its middle (`HOLES`), the hood's left side (`HOOD`, where the other hand steadies
 * it), and the trunk door's window from 0.6 to 1.4 m, 0.08 m behind the middle (a child's hands in at the
 * weights' chains, `CHAINS`).
 */
const HOLES = [0.065, 1.7, -0.044 + 0.07] as const;
const HOOD = [-0.25, 1.74, -0.03] as const;
const CHAINS = { x: 0.06, y: [0.78, 1.08], z: -0.12 } as const;
type ClockFit = Fit;
export const CLOCK: Size<ClockFit>[] = sizes('windClock', SHARES_177, (s) => ({ s, to: s > 0.9 ? 0.35 : s > 0.7 ? 0.3 : 0.25 }));
const CLOCK_S = 4.5;

/**
 * Winding the clock: up to the dial, the key into the right-hand hole and four turns of the wrist, across to
 * the left-hand one and four more, the left hand on the hood's side; down, and a look at the time. A child pulls
 * the weights' chains up hand over hand through the trunk door's window.
 */
function clockPose(fit: ClockFit): (p: number) => PoseSpec {
  const { s } = fit;
  const o = (x: number, y: number, z: number) => at(fit, x, y, z);
  if (s < REACHES_UP) {
    return (p) => {
      const t = p * CLOCK_S;
      // Four pulls, a hand each in turn: up the chain, closed on it, down.
      const n = Math.max(0, Math.min(3.999, (t - 0.4) / 0.9));
      const k = n % 1;
      const right = Math.floor(n) % 2 === 0;
      const on = bump(t, 0.2, 4.1, 0.35);
      const low = CHAINS.y[0];
      const high = CHAINS.y[1];
      const pull = (mine: boolean) => (mine ? high - (high - low) * Math.sin((Math.PI / 2) * k) : low + (high - low) * k);
      const hand = (side: 'l' | 'r', mine: boolean): Hand | null => {
        if (on < 0.001) return null;
        const sx = side === 'l' ? -1 : 1;
        return { at: o(sx * CHAINS.x, pull(mine), CHAINS.z), w: on, pronate: 0, pole: [0.8 * sx, -0.8, -0.2], hand: wrist(side, 10, 0) };
      };
      return upFor(hand('l', !right), hand('r', right), o(0, 0.95, CHAINS.z), p, { lean: 6 });
    };
  }
  const right = o(HOLES[0], HOLES[1], HOLES[2]);
  const left = o(-HOLES[0], HOLES[1], HOLES[2]);
  const hood = o(HOOD[0], HOOD[1], HOOD[2]);
  const R: Track = [
    [0, null],
    [0.55, right],
    [2.05, right],
    [2.4, left],
    [3.9, left],
    [4.35, null],
    [CLOCK_S, null],
  ];
  const L: Track = [
    [0, null],
    [0.5, hood],
    [3.9, hood],
    [4.35, null],
    [CLOCK_S, null],
  ];
  return (p) => {
    const t = p * CLOCK_S;
    // Four turns of the key in each hole: the wrist rolls and comes back for the next.
    const turning = bump(t, 0.6, 2.0, 0.08) + bump(t, 2.45, 3.85, 0.08);
    const roll = 45 * Math.sin(((t - 0.6) / 0.35) * TAU) * turning;
    const r = handOn(R, t, { pronate: 20 + roll, pole: [0.8, -0.6, -0.2], hand: wrist('r', -15, 0) });
    const l = handOn(L, t, { pronate: 70, pole: [-0.8, -0.6, -0.2], hand: wrist('l', -10, 0) });
    const look = lerpV(o(0, 1.77, -0.04), placed(R, Math.min(3.9, Math.max(0.55, t)))[0], bump(t, 0.25, 4.1, 0.3));
    return upFor(l, r, look, p);
  };
}

// --- the bird feeder ------------------------------------------------------------------------------

/**
 * The songbird feeder (`crates/art/packs/src/pack_kind`), used from its post (`STEP_IN`'s lane): its house 1.38 to 1.58 m up,
 * its front 0.13 m behind the middle and its middle 0.22 m, the roof lifted for a refill (`lid_open`); the
 * seed tray round the post at 1.38 m, its front 0.06 m behind the middle.
 */
const HOUSE = { top: 1.6, front: -0.13, mid: -0.22 };
const TRAY = [0.0, 1.41, -0.08] as const;
type FeederFit = Fit;
export const FEEDER: Size<FeederFit>[] = sizes('fillFeeder', SHARES_177, (s) => ({ s, to: s > 0.9 ? 0.25 : s > 0.7 ? 0.22 : 0.18 }));
const FEEDER_S = 4;

/**
 * Filling the feeder: the left hand steadying the house, the scoop (mimed, in the right) up over its open top
 * and tipped twice to pour, down to the hip again. A child, who can't reach the house, tips it onto the tray.
 */
function feederPose(fit: FeederFit): (p: number) => PoseSpec {
  const { s } = fit;
  const o = (x: number, y: number, z: number) => at(fit, x, y, z);
  const reaches = s >= REACHES_UP;
  const over = reaches ? o(0.03, HOUSE.top + 0.1, HOUSE.mid + 0.03) : o(TRAY[0] + 0.05, TRAY[1], TRAY[2]);
  const side = reaches ? o(-0.13, 1.47, HOUSE.mid) : o(-0.12, TRAY[1] - 0.02, TRAY[2] - 0.04);
  const low: Vec = [0.17, 1.0, 0.2];
  const R: Key<Vec>[] = [
    [0, low],
    [0.7, [over[0] + 0.04, over[1] - 0.06, over[2] - 0.06]],
    [1.0, over],
    [2.3, over],
    [2.7, [over[0] + 0.03, over[1] - 0.04, over[2] - 0.05]],
    [3.4, low],
    [FEEDER_S, low],
  ];
  const L: Track = [
    [0, null],
    [0.6, side],
    [2.6, side],
    [3.2, null],
    [FEEDER_S, null],
  ];
  return (p) => {
    const t = p * FEEDER_S;
    // Tipped to pour twice, and back level.
    const tip = bump(t, 1.05, 1.6, 0.15) + bump(t, 1.7, 2.25, 0.15);
    const r: Hand = { at: track(R, t), pronate: -20 + 90 * tip, pole: [0.8, -0.6, -0.3], hand: wrist('r', -10 + 10 * tip, 15 * tip) };
    const l = handOn(L, t, { pronate: 80, pole: [-0.8, -0.6, -0.3], hand: wrist('l', -10, 0) });
    return upFor(l, r, lerpV(AHEAD, over, bump(t, 0.2, 3.1, 0.4)), p);
  };
}

// --- the whiteboard -------------------------------------------------------------------------------

/**
 * The office whiteboard (`office.py`): the board from 1.11 to 1.69 m up, its face 0.4155 m behind the middle;
 * the marker's tip on it with the fist 0.065 m off (`PEN`). Three lines are written, the highest a little over
 * the shoulders and lower the smaller the writer (`LINES`, m up).
 */
const FACE = -0.4155;
const PEN = 0.065;
type BoardFit = Fit & { lines: readonly number[] };
export const BOARD: Size<BoardFit>[] = sizes('writeBoard', SHARES_177, (s) =>
  s > 0.9 ? { s, to: 0.04, lines: [1.52, 1.43, 1.34] } : s > 0.7 ? { s, to: -0.02, lines: [1.43, 1.35, 1.27] } : { s, to: -0.08, lines: [1.3, 1.23, 1.16] },
);
const BOARD_S = 6;
/** A line written from `x0` to `x1` (m, to the user's right) in this long (s), then the hand back to the next line's start. */
const LINE = { x0: -0.14, x1: 0.24, write: 1.55, back: 0.45 };

/**
 * Writing on the whiteboard: three lines left to right at the board, the letters' strokes up and down and the
 * marker lifted between words, back to the start of the next one lower; the weight and the head going with the
 * hand, the other hand holding the cap at the waist.
 */
function boardPose(fit: BoardFit): (p: number) => PoseSpec {
  const o = (x: number, y: number, z: number) => at(fit, x, y, z);
  const each = LINE.write + LINE.back;
  return (p) => {
    const t = p * BOARD_S;
    const i = Math.min(2, Math.floor(t / each));
    const u = t - i * each;
    let x: number;
    let y = fit.lines[i];
    let off = 0;
    if (u < LINE.write) {
      const k = u / LINE.write;
      x = LINE.x0 + (LINE.x1 - LINE.x0) * k;
      // Letters: up and down strokes; a gap (the marker off the board) between words.
      y += 0.018 * Math.sin(k * TAU * 11);
      off = 0.025 * bump(k, 0.3, 0.36, 0.02) + 0.025 * bump(k, 0.66, 0.72, 0.02);
    } else {
      // Off the board and back to the left, down to the next line (or round to the first).
      const k = smooth(Math.min(1, (u - LINE.write) / LINE.back));
      const next = fit.lines[(i + 1) % 3];
      x = LINE.x1 + (LINE.x0 - LINE.x1) * k;
      y += (next - y) * k;
      off = 0.06 * Math.sin(Math.PI * k);
    }
    const pen = o(x, y, FACE + PEN + off);
    const r: Hand = { at: pen, pronate: 40, pole: [0.8, -0.6, -0.2], hand: wrist('r', 15, 0) };
    const l: Hand = { at: [-0.1, 1.02, 0.22], pronate: 60, pole: [-0.5, -1, -0.2], hand: wrist('l', 20, 0) };
    return upFor(l, r, pen, p, { shift: 0.3 * (x / 0.2), tip: false });
  };
}

// --- the chest freezer ----------------------------------------------------------------------------

/**
 * The chest freezer (`crates/art/wet/src/kitchen/freezers.rs`, 2 × 1, 0.68 m deep against the wall): the box's rim
 * 0.82 m up, its front's face 0.25 m out from the middle; the lid hinged along its back (0.845 m up, 0.41 m behind
 * the middle), its handle at the front, 0.68 m from the hinge. The lid lifts 85° in 1.2 s as the use starts (`lid_open`), stays up 3.5 s and
 * comes down in 1.2 s. Used from `CHEST_TO` m out (characters.ts `STAND_BACK`), the toes at the box.
 */
const LID = { y: 0.845, z: -0.41, dy: 0.015, dz: 0.68, deg: 85, open: 1.2, held: 3.5 };
export const CHEST_TO = 0.45;
export const CHEST: Size[] = sizes('chestFetch', SHARES_177, (s) => ({ s, to: CHEST_TO }));
const CHEST_S = 6.4;
/** The lid's lift (deg) `t` s into the use. */
function lidAt(t: number): number {
  const shut = LID.open + LID.held;
  return LID.deg * (t < LID.open ? smooth(t / LID.open) : t < shut ? 1 : 1 - smooth((t - shut) / LID.open));
}
/** The lid's handle `x` m to the user's right, `t` s into the use (m: the object's frame). */
function lidHandle(t: number, x: number): [number, number, number] {
  const a = (lidAt(t) * Math.PI) / 180;
  return [x, LID.y + LID.dz * Math.sin(a) + LID.dy * Math.cos(a), LID.z + LID.dz * Math.cos(a) - LID.dy * Math.sin(a)];
}
/** Where the hands go into the well (m, the object's frame: the right's; the left's mirrored), by the size: a grown-up's near its floor, a child's just under the rim. */
const WELL = (s: number): [number, number, number] => (s > 0.9 ? [0.13, 0.4, -0.08] : s > 0.7 ? [0.13, 0.48, -0.02] : [0.12, 0.62, 0.1]);

/**
 * Getting something out of the chest freezer: the right hand to the lid's handle, lifting it with it as it swings
 * up and letting it go on; bent over the rim (from the hips, the knees giving), both hands down into the well,
 * rummaging and taking hold; straightening up with it at the chest; the right hand up to meet the lid as it comes
 * down and lowering it shut, back to the side. The smaller let it go sooner going up and meet it lower coming down.
 */
function chestPose(fit: Fit): (p: number) => PoseSpec {
  const { s } = fit;
  const o = (v: readonly [number, number, number]) => at(fit, v[0], v[1], v[2]);
  const w = WELL(s);
  const well = (sx: number, dx = 0, dz = 0) => o([sx * w[0] + dx, w[1], w[2] + dz]);
  const rim = (sx: number) => o([sx * 0.15, 0.95, 0.22]);
  const held = (sx: number): Vec => [sx * 0.08, 1.12, 0.3];
  const stoop = (bend: number, p: number, gaze: number) => stoopBody(bend, -0.0016 * bend, 0, p, gaze);
  // How far bent over the rim for the hands to be in the well (found while baking): the shoulders within an arm of it.
  let deep = -1;
  const solve = () => {
    deep = 10;
    while (deep < 85 && len(sub(well(1), pointOn(stoop(deep, 0, 60), 'upperarm_r', [0, 0, 0]))) > ARM) deep++;
  };
  // The lid lifted with the hand as far as the size reaches up (the times it is let go at and caught at, coming down).
  const [lets, caught] = s > 0.9 ? [0.7, 5.25] : s > 0.7 ? [0.5, 5.4] : [0.4, 5.55];
  const R: Track = [
    [0, null],
    [0.3, o(lidHandle(Math.min(0.3, lets - 0.15), 0.12))],
    [lets, o(lidHandle(lets, 0.12))],
    [1.0, null],
    [1.25, rim(1)],
    [1.5, well(1)],
    [1.9, well(1, -0.04, 0.04)],
    [2.4, well(1, 0.03, -0.02)],
    [2.8, well(1)],
    [3.1, rim(1)],
    [3.5, held(1)],
    [caught - 0.35, held(1)],
    [caught, o(lidHandle(caught, 0.12))],
    [5.9, o(lidHandle(5.9, 0.12))],
    [6.3, null],
    [CHEST_S, null],
  ];
  const L: Track = [
    [0, null],
    [1.0, null],
    [1.25, rim(-1)],
    [1.5, well(-1)],
    [2.0, well(-1, 0.03, 0)],
    [2.5, well(-1, 0, 0.03)],
    [2.8, well(-1)],
    [3.1, rim(-1)],
    [3.5, held(-1)],
    [5.7, held(-1)],
    [6.2, null],
    [CHEST_S, null],
  ];
  return (p) => {
    if (deep < 0) solve();
    const t = p * CHEST_S;
    const bend = track<number>([[0, 6], [0.5, 14], [1.0, 8], [1.5, deep], [2.8, deep], [3.5, 8], [caught, 4], [5.9, 16], [CHEST_S, 6]], t);
    const gaze = track<number>([[0, 20], [0.5, 30], [1.0, 10], [1.5, 65], [2.8, 65], [3.5, 25], [caught, -15], [5.9, 35], [CHEST_S, 20]], t);
    // The palms down into the well and on the handle; up under what is held at the chest (the right's till it goes to the lid).
    const hand = (side: 'l' | 'r', keys: Track) => {
      const sx = side === 'l' ? -1 : 1;
      const hold = bump(t, 3.2, side === 'l' ? 6.1 : caught - 0.05, 0.3);
      return handOn(keys, t, { pronate: 70 - 80 * hold, pole: [0.75 * sx, -0.7, -0.3], hand: wrist(side, -10, 0) });
    };
    return handsOn(stoop(bend, p, gaze), hand('l', L), hand('r', R));
  };
}

// --- the doorbell ---------------------------------------------------------------------------------

/**
 * The doorbell (`crates/art/rooms/src/decor/fittings.rs`, on the wall, its face 0.43 m behind the middle): the
 * button 1.34 m up, its face 0.394 m behind the middle. Stood up to it (characters.ts `STEP_IN`), the feet nearer
 * the smaller they are, as at the whiteboard; the fingertips 0.085 m ahead of the palm's centre.
 */
const BELL = { y: 1.34, z: -0.394, tips: 0.085 };
// (A child, 1.12 m, stretches up for it on tiptoe and doesn't quite reach.)
export const PRESS: Size[] = sizes('pressBell', SHARES_177, (s) => ({ s, to: s > 0.9 ? 0.06 : s > 0.7 ? 0 : -0.06 }));
const PRESS_S = 4;

/** Ringing: the right hand up to the button, a short press with the fingertips and off; down, and waiting at the door. */
function pressPose(fit: Fit): (p: number) => PoseSpec {
  const o = (x: number, y: number, z: number) => at(fit, x, y, z);
  const off = o(0, BELL.y, BELL.z + BELL.tips + 0.05);
  const on = o(0, BELL.y, BELL.z + BELL.tips);
  const R: Track = [
    [0, null],
    [0.55, off],
    [0.68, on],
    [0.9, on],
    [1.05, off],
    [1.6, null],
    [PRESS_S, null],
  ];
  return (p) => {
    const t = p * PRESS_S;
    const r = handOn(R, t, { pronate: 15, pole: [0.8, -0.7, -0.2], hand: wrist('r', -20, 0) });
    return upFor(null, r, lerpV(AHEAD, on, bump(t, 0.2, 1.4, 0.3)), p);
  };
}

export function reachClips(): ClipDef[] {
  return [
    ...Object.values(PUT_AWAY).flatMap((sz) => sz.map(({ clip, fit }) => cycle(clip, { seconds: CUPBOARD_S, loop: true }, cupboardPose(fit)))),
    ...CLOCK.map(({ clip, fit }) => cycle(clip, { seconds: CLOCK_S, loop: true }, clockPose(fit))),
    ...FEEDER.map(({ clip, fit }) => cycle(clip, { seconds: FEEDER_S, loop: true }, feederPose(fit))),
    ...BOARD.map(({ clip, fit }) => cycle(clip, { seconds: BOARD_S, loop: true }, boardPose(fit))),
    ...CHEST.map(({ clip, fit }) => cycle(clip, { seconds: CHEST_S, loop: true }, chestPose(fit))),
    ...PRESS.map(({ clip, fit }) => cycle(clip, { seconds: PRESS_S, loop: true, hands: 'open' }, pressPose(fit))),
  ];
}
