/**
 * Walking and jogging, made procedurally from a gait: each foot strikes with the heel, rolls flat,
 * stays planted (moving back at the ground speed in the clip's frame), peels off over the ball and
 * swings forward on a smooth arc; the pelvis bobs (high at mid-stance), sways over the standing
 * leg, rolls and turns with the stride, the chest counter-turns, the arms swing against the legs
 * (a little behind the shoulders) and the head stays steady.
 */

import { add, cycle, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { TAU } from './idle';
import { apply, elbow, euler, footIK, hang, smooth, sub, wrist } from './ik';

/** Reference walk: 1.1 m/s over a 1.1 s cycle (two steps). */
const WALK_SPEED = 1.1;
const WALK_SECONDS = 1.1;

export interface Gait {
  /** Ground travelled per cycle (m); 0 runs on the spot. */
  stride: number;
  /** How far behind the hip the stance is centred (m). */
  back: number;
  /** Share of the cycle a foot is on the ground. */
  duty: number;
  /** Heel strike (toes up, deg), heel off (phase of the cycle) and toe off (toes down, deg). */
  heel: number;
  heelOff: number;
  toeOff: number;
  /** Mid-swing ankle lift (m above its planted height), the feet nearer the midline (m), toes out (deg). */
  lift: number;
  narrow: number;
  toeOut: number;
  /** Pelvis: drop at the steps (m), constant sink (m), side sway (m), roll / turn / forward tilt (deg). */
  bob: number;
  sink: number;
  sway: number;
  roll: number;
  turn: number;
  tilt: number;
  /** Lean of the back (deg), the arms' swing (deg forward and back), the elbows' bend (deg). */
  lean: number;
  arms: number;
  elbows: number;
  /** Running: the pelvis is low mid-stance and high in flight; the swing foot kicks back first (m). */
  run?: boolean;
  kick?: number;
}

export const WALK: Gait = {
  stride: WALK_SPEED * WALK_SECONDS,
  back: 0.07,
  duty: 0.6,
  heel: 14,
  heelOff: 0.3,
  toeOff: 40,
  lift: 0.06,
  narrow: 0.025,
  toeOut: 6,
  bob: 0.03,
  sink: 0.006,
  sway: 0.018,
  roll: 3.5,
  turn: 5,
  tilt: 3,
  lean: 3,
  arms: 17,
  elbows: 16,
};

interface FootState {
  /** Ankle offset from its rest place (before the gait's narrowing). */
  ankle: Vec;
  /** Foot pitch (deg, + toes down) and the toes bent back (deg). */
  pitch: number;
  toes: number;
}

/** A foot at phase q of its own cycle (0 = heel strike): planted between heel and ball pivots, a Hermite arc through the swing. */
function footAt(q: number, g: Gait): FootState {
  const ref = reference();
  const ball = sub(ref.joint('ball_l'), ref.joint('foot_l'));
  // The heel's contact point (ankle frame), on the same sole line as the ball joint.
  const heel: Vec = [0, ball[1], -0.05];
  const S = g.stride;
  const flat = 0.08;
  // Heel point (ankle-offset coordinates) at strike: the stance centred `back` behind the hip.
  const zHeel = S * (g.duty / 2) + heel[2] - g.back;
  const stance = (t: number): FootState => {
    const ground = zHeel - S * t;
    if (t < flat) {
      const th = -g.heel * (1 - smooth(t / flat));
      const a = sub([0, heel[1], ground], apply(euler([th, 0, 0]), heel));
      return { ankle: a, pitch: th, toes: 0 };
    }
    if (t < g.heelOff) return { ankle: [0, 0, ground - heel[2]], pitch: 0, toes: 0 };
    const u = (t - g.heelOff) / (g.duty - g.heelOff);
    const th = g.toeOff * u * u * (1.6 - 0.6 * u);
    const pivot: Vec = [0, ball[1], ground - heel[2] + ball[2]];
    return { ankle: sub(pivot, apply(euler([th, 0, 0]), ball)), pitch: th, toes: -th };
  };
  if (q < g.duty) return stance(q);
  // Swing: from toe-off to the next heel strike, matching the stance's velocities at both ends.
  const s = (q - g.duty) / (1 - g.duty);
  const dq = 1e-3;
  const span = 1 - g.duty;
  const a0 = stance(g.duty - dq).ankle;
  const a1 = stance(g.duty).ankle;
  const b0 = stance(0).ankle;
  const b1 = stance(dq).ankle;
  const herm = (p0: number, v0: number, p1: number, v1: number) => {
    const s2 = s * s;
    const s3 = s2 * s;
    return (2 * s3 - 3 * s2 + 1) * p0 + (s3 - 2 * s2 + s) * v0 + (-2 * s3 + 3 * s2) * p1 + (s3 - s2) * v1;
  };
  const vel = (x0: number, x1: number) => ((x1 - x0) / dq) * span;
  const z = herm(a1[2], vel(a0[2], a1[2]), b0[2], vel(b0[2], b1[2]));
  const yBase = herm(a1[1], vel(a0[1], a1[1]) * 0.5, b0[1], 0);
  // Lift: the knee pulls the foot up early in the swing, then it reaches forward and down.
  const arc = Math.sin(Math.PI * Math.pow(s, 0.75));
  const y = yBase + Math.max(0, g.lift - yBase) * (g.run ? Math.pow(arc, 1.6) : arc);
  const kick = -(g.kick ?? 0) * Math.sin(Math.PI * s) * (1 - 2 * s);
  // Toes down at toe-off, level by mid-swing, up for the heel strike.
  const pitch = s < 0.45 ? g.toeOff * (1 - smooth(s / 0.45)) : -g.heel * smooth((s - 0.45) / 0.55);
  return { ankle: [0, y, z + kick], pitch, toes: -g.toeOff * (1 - smooth(s / 0.3)) };
}

/** A gait pose at phase p (0 = left heel strike). */
export function gaitPose(p: number, g: Gait): PoseSpec {
  const wrap = (x: number) => ((x % 1) + 1) % 1;
  const L = footAt(wrap(p), g);
  const R = footAt(wrap(p + 0.5), g);
  // Pelvis: low just after each heel strike, high over the standing leg (the left at ~0.3).
  const bob = g.run ? -g.sink - (g.bob / 2) * (1 + Math.cos(4 * Math.PI * (p - g.duty / 2))) : -g.sink - (g.bob / 2) * (1 + Math.cos(4 * Math.PI * (p + 0.03)));
  const side = Math.sin(TAU * (p - 0.05));
  const turn = Math.cos(TAU * p);
  const nod = Math.cos(4 * Math.PI * (p - 0.12));
  const body: PoseSpec = {
    move: [-g.sway * side, bob, 0],
    pelvis: [g.tilt, g.turn * turn, -g.roll * side],
    spine_01: [g.lean * 0.5 - g.tilt * 0.6, -g.turn * 0.4 * turn, g.roll * 0.5 * side],
    spine_02: [g.lean * 0.3, -g.turn * 0.6 * turn, g.roll * 0.35 * side],
    spine_03: [g.lean * 0.2 + 0.6 * nod, -g.turn * 0.6 * turn, g.roll * 0.15 * side],
    neck_01: [-g.lean * 0.4, g.turn * 0.3 * turn, 0],
    // The head steadies itself against the bob, a beat late.
    Head: [-g.tilt * 0.4 - g.lean * 0.4 + 1.2 * nod, g.turn * 0.5 * turn, -g.roll * 0.2 * side],
  };
  const leg = (side: 'l' | 'r', s: FootState) => {
    const sx = side === 'l' ? -1 : 1;
    return footIK(side, [s.ankle[0] - sx * g.narrow, s.ankle[1], s.ankle[2]], body, { foot: [s.pitch, sx * g.toeOut, 0] });
  };
  // Arms: against the legs, lagging the shoulders; the elbow bends more going forward.
  const swingL = -Math.cos(TAU * (p - 0.05));
  const arm = (side: 'l' | 'r', s: number): PoseSpec => ({
    [`upperarm_${side}`]: hang(side, 3 + g.arms * s, 7 + 1.5 * Math.max(0, s)),
    [`lowerarm_${side}`]: elbow(side, g.elbows * (1 + 0.9 * Math.max(0, s)), 10),
    [`hand_${side}`]: wrist(side, 8 + 6 * Math.max(0, s)),
  });
  return add(body, leg('l', L), leg('r', R), { ball_l: [L.toes, 0, 0], ball_r: [R.toes, 0, 0] }, arm('l', swingL), arm('r', -swingL), {
    clavicle_l: [0, 1.5 * swingL, 0],
    clavicle_r: [0, 1.5 * swingL, 0],
  });
}

/** Jogging on the spot: short ground contact with a flight phase, knees up, the arms bent and pumping. */
export const JOG: Gait = {
  stride: 0,
  back: 0.0,
  duty: 0.4,
  heel: 0,
  heelOff: 0.12,
  toeOff: 22,
  lift: 0.24,
  narrow: 0.03,
  toeOut: 4,
  bob: 0.03,
  sink: 0.0,
  sway: 0.012,
  roll: 3,
  turn: 7,
  tilt: 4,
  lean: 4,
  arms: 30,
  elbows: 80,
  run: true,
  kick: 0.22,
};

/** Jogging along (`jogAlong`): the jog travelling at 2.5 m/s, a stride per cycle, leaning into it a little more. */
const JOG_SECONDS = 0.7;
const JOG_SPEED = 2.5;
const JOG_ALONG: Gait = { ...JOG, stride: JOG_SPEED * JOG_SECONDS, back: 0.02, lean: 7 };

function jogPose(p: number, g: Gait = JOG): PoseSpec {
  const swingL = -Math.cos(TAU * (p - 0.04));
  return {
    ...gaitPose(p, g),
    upperarm_l: hang('l', -2 + 24 * swingL, 9),
    upperarm_r: hang('r', -2 - 24 * swingL, 9),
    lowerarm_l: elbow('l', 84 + 12 * swingL, 15),
    lowerarm_r: elbow('r', 84 - 12 * swingL, 15),
  };
}

export function locomotionClips(): ClipDef[] {
  return [
    cycle('walk', { seconds: WALK_SECONDS, loop: true, speed: WALK_SPEED }, (p) => gaitPose(p, WALK)),
    cycle('jog', { seconds: JOG_SECONDS, loop: true }, (p) => jogPose(p)),
    // Residents on their way faster than the walk can step (the game sped up; children's short legs).
    cycle('jogAlong', { seconds: JOG_SECONDS, loop: true, speed: JOG_SPEED }, (p) => jogPose(p, JOG_ALONG)),
  ];
}
