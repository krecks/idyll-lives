/**
 * Checks the residents' clips (web/src/render/babylon/characters/clips) baked on the built rig:
 *
 *     node --experimental-strip-types tools/characters/check_clips.ts [rig.json] [--body male|female|all]
 *         [--stage <stage>[,<stage>…]|all] [--update-baseline]
 *
 * - `--body`: the bodies to check on (default `male`, the reference).
 * - `--stage`: the life stages (`baby`, `child`, `teen`, `youngAdult`, `adult`, `elder`, or `all`;
 *   default `youngAdult,baby`: the reference stage, and the babies' for their own clips).
 * - `--update-baseline`: rewrite `clip-baseline.json` (next to this file) with this run's failures,
 *   keeping the entries of bodies and stages it didn't check, and exit 0.
 *
 * What it checks:
 * - every clip the runtime plays exists, with the right loop flag (and the phase-2 clips too);
 * - no NaNs; loops wrap seamlessly (the step from the last frame to the first is in line with the
 *   steps either side of it: at most 2 degrees more than the larger);
 * - the planted foot of the locomotion clips (`walk`, `jogAlong`, `runAlong`: those with a `speed`) slides less
 *   than 1 cm per cycle against the ground speed, and a foot flat on the floor in any other loop (or
 *   the short one-shots) stays put (< 1 cm);
 * - every clip's length is within 15% of its target (the runtime sequences on them);
 * - seated (`sit` fitted with the body's `seatDrop`) on a 0.45 m seat, a child's on the kids' chair
 *   (0.31 m), the feet reach the floor, and the other seated clips (and the seated ends of sitDown /
 *   standUp) keep `sit`'s pelvis;
 * - `lie` lies straight and down: knees nearly straight and not above the hips, the ankles 5 cm
 *   below the pelvis (the legs on the mattress, not floating), the hands 5 cm in front of the body;
 * - the baby's lying clips keep the baby on the mattress and inside the crib: no joint more than
 *   3 cm behind the back, none further out to the side than `BABY_REACH`, the knees bent.
 * The lengths of the last two (body measures, not tolerances) are the reference skeleton's, scaled
 * by the stage's height.
 *
 * Bodies and stages: the clips are baked once on the reference skeleton (the male young adult), as
 * the runtime does, and played on each body and stage the way `skin` (pose.ts) plays them: the
 * stage's own bone offsets (`restT`, rig.json `bodies.<body>.stages`), the clip's pelvis motion
 * scaled by its `pelvisScale`, and the locomotion's ground speed scaled by it too (characters.ts
 * `legs`). `walk` gets the runtime's hip roll per body (characters.ts `GAIT_SWAY`). Of the clips
 * made in sizes (the fountain's, the climbing frame's, the telescope's, the fruit trees' and the shower's) each stage gets the size its height picks
 * (characters.ts `stepFit`), and babies only their own clips (only those are checked at `baby`, and
 * only the others at every other stage). What it doesn't cover: the resident's height slider (a
 * uniform scale, though it can change the size picked), the build and head size (skinning only),
 * the elder's stoop (upper back and head, off when lying: no joint checked here), the other
 * procedural layers (look-at, breathing, turning on the spot) and the fits the runtime makes to
 * seats, beds and objects (`tuckIn`, `SEATS`, `BEDS`, `legsOut`, pedals). Children on an adult's
 * 0.45 m seat dangle their feet, as real ones do; that isn't checked.
 *
 * Loop flags, NaNs, seams and lengths are the same on every skeleton: they are checked once (body
 * and stage `*`). A failure listed in clip-baseline.json is printed as known and doesn't fail; a
 * listed one that no longer fails is printed as fixed (remove it). Exits non-zero on a new failure.
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { registerHooks } from 'node:module';

// The web sources import without extensions (Vite resolves them); add `.ts` here.
registerHooks({
  resolve(specifier, context, next) {
    try {
      return next(specifier, context);
    } catch (err) {
      if (specifier.startsWith('.')) return next(`${specifier}.ts`, context);
      throw err;
    }
  },
});

const { clipDefs } = await import('../../web/src/render/babylon/characters/clips/index.ts');
const { bake } = await import('../../web/src/render/babylon/characters/clips/dsl.ts');
const { CLIMB, CLIMB_LANES, climbClip, FOUNTAIN } = await import('../../web/src/render/babylon/characters/clips/street.ts');
const { EYEPIECE, PICK_APPLE, PICK_LEMON, SHOWER_CHILD } = await import('../../web/src/render/babylon/characters/clips/uses.ts');

/** Every clip the runtime plays (characters.ts ACTIONS / choose, reactions.ts) and whether it loops. */
const RUNTIME: Record<string, boolean> = {
  idle: true, talk: true, walk: true, jog: true, jogAlong: true, runAlong: true, sit: true, sitTalk: true, sitDown: false, standUp: false, sitHands: true, sitHandsNear: true, sitHandsFar: true,
  interact: false, pickUp: false, dance: true, punchJab: false, punchCross: false, hit: false, kneel: true, crouch: true,
  eat: false, water: true, harvest: true, pick: true, foldArms: true, no: true, yes: false, phone: true, rail: true, spell: true,
  push: true, plant: false, lantern: true, babyLieAsleep: true, babyLieAwake: true,
  // The drinking fountain and the climbing frame, in sizes (and the frame's in lanes: characters.ts STEP_IN, clips/street.ts).
  fountainDrink181: true, fountainDrink163: true, fountainDrink147: true, fountainDrink129: true, fountainDrink112: true,
  climb172a: true, climb147a: true, climb112a: true, climb172b: true, climb147b: true, climb112b: true,
  // The telescope's eyepiece and the fruit trees, in sizes (characters.ts STEP_IN, clips/uses.ts); the shower, a child's without the turn.
  eyepiece181: true, eyepiece163: true, eyepiece147: true, eyepiece129: true, eyepiece112: true,
  pickApple177: false, pickApple147: false, pickApple112: false, pickLemon177: false, pickLemon147: false, pickLemon112: false,
  shower: true, showerChild: true,
};
/** Phase-2 clips (the runtime picks them up later) and whether they loop. */
const PHASE2: Record<string, boolean> = { lie: true, chop: true, stir: true, flip: true, drink: false, carry: true, carryOne: true, wave: false };
/** Lengths (s) from the plan's clip table; within 15%. */
const LENGTH: Record<string, number> = {
  idle: 2.5, talk: 2.9, walk: 1.1, jog: 0.7, jogAlong: 0.7, runAlong: 0.6, sit: 1.7, sitTalk: 2.9, sitDown: 1.33, standUp: 1.07, sitHands: 1.7, sitHandsNear: 1.7, sitHandsFar: 1.7, interact: 2.0, pickUp: 0.87,
  dance: 1.0, punchJab: 0.87, punchCross: 1.0, hit: 0.33, kneel: 5.2, crouch: 2.9, eat: 1.33, water: 3.8, harvest: 2.5, pick: 3, plant: 2.8, foldArms: 2.5,
  yes: 2.5, no: 2.5, phone: 2.9, lantern: 2.5, rail: 2.5, spell: 2.1, push: 2.7, babyLieAsleep: 9, babyLieAwake: 6,
  fountainDrink181: 3, fountainDrink163: 3, fountainDrink147: 3, fountainDrink129: 3, fountainDrink112: 3, climb172a: 13, climb147a: 13, climb112a: 13, climb172b: 13, climb147b: 13, climb112b: 13,
  eyepiece181: 6, eyepiece163: 6, eyepiece147: 6, eyepiece129: 6, eyepiece112: 6, pickApple177: 3.3, pickApple147: 3.3, pickApple112: 3.3, pickLemon177: 3.3, pickLemon147: 3.3, pickLemon112: 3.3,
  shower: 14, showerChild: 14,
};
/** The baby lying in the crib: how far out to the side a joint may reach (m, reference skeleton: about 0.23 m on a baby, whose crib's bars are about 0.4 m from its middle). */
const BABY_REACH = 0.6;
/** The baby's own clips (characters.ts `BABY_LIE`): the only ones a baby plays. */
const BABY = new Set(['babyLieAsleep', 'babyLieAwake']);
/** One-shots whose feet stay planted (sitting down and standing up move them on purpose). */
const PLANTED = new Set(['interact', 'pickUp', 'eat', 'yes', 'punchJab', 'punchCross', 'hit', 'plant', 'drink', 'wave', ...[...PICK_APPLE, ...PICK_LEMON].map(({ clip }: any) => clip)]);
/** The seat `sit` is checked on (m): a chair's, a child's the kids' chair (content `kidsChair`). */
const SEAT: Record<string, number> = { child: 0.31 };
const SEAT_HEIGHT = 0.45;
const STAGES = ['baby', 'child', 'teen', 'youngAdult', 'adult', 'elder'];
/** The clips made in sizes (the tallest first, `min` the height each starts at), as characters.ts `STEP_IN` (and `ACTIONS.shower`) uses them. */
const SIZED: { min: number; clips: string[] }[][] = [
  FOUNTAIN.map(({ clip, min }: any) => ({ min, clips: [clip] })),
  CLIMB.map(({ clip, min }: any) => ({ min, clips: CLIMB_LANES.map((_: number, lane: number) => climbClip(clip, lane)) })),
  ...[EYEPIECE, PICK_APPLE, PICK_LEMON].map((sizes) => sizes.map(({ clip, min }: any) => ({ min, clips: [clip] }))),
  [{ min: SHOWER_CHILD, clips: ['shower'] }, { min: 0, clips: ['showerChild'] }],
];
/** The runtime's hip roll while walking, per body (rad), read from characters.ts so the two can't drift apart. */
const GAIT_SWAY: Record<string, number> = (() => {
  const src = readFileSync(new URL('../../web/src/render/babylon/characters.ts', import.meta.url), 'utf8');
  const m = /const GAIT_SWAY[^=]*=\s*\{([^}]*)\}/.exec(src);
  if (!m) throw new Error('check_clips: GAIT_SWAY not found in characters.ts');
  return Object.fromEntries(m[1].split(',').map((kv) => kv.split(':').map((s) => s.trim())).filter((kv) => kv.length === 2).map(([k, v]) => [k, Number(v)]));
})();
const BASELINE = new URL('./clip-baseline.json', import.meta.url).pathname;

// --- arguments --------------------------------------------------------------------------------

const argv = process.argv.slice(2);
let path = new URL('../../web/public/assets/characters/rig.json', import.meta.url).pathname;
let bodyArg = 'male';
let stageArg = 'youngAdult,baby';
let updateBaseline = false;
for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  if (a === '--body') bodyArg = argv[++i];
  else if (a === '--stage') stageArg = argv[++i];
  else if (a === '--update-baseline') updateBaseline = true;
  else if (a.startsWith('--')) usage(`unknown flag ${a}`);
  else path = a;
}
function usage(why: string): never {
  console.error(`check_clips: ${why}\nusage: check_clips.ts [rig.json] [--body male|female|all] [--stage ${STAGES.join('|')}[,…]|all] [--update-baseline]`);
  process.exit(2);
}

const rig = JSON.parse(readFileSync(path, 'utf8'));
const bodyNames = bodyArg === 'all' ? Object.keys(rig.bodies) : bodyArg?.split(',') ?? [];
for (const b of bodyNames) if (!rig.bodies[b]) usage(`no body ${b} (${Object.keys(rig.bodies).join(', ')})`);
const stageNames = stageArg === 'all' ? STAGES : stageArg?.split(',') ?? [];
for (const s of stageNames) if (!STAGES.includes(s)) usage(`no stage ${s} (${STAGES.join(', ')})`);
if (!bodyNames.length || !stageNames.length) usage('no body or stage');

const bodies = new Map(Object.entries(rig.bodies).map(([name, b]: [string, any]) => [name, { ...b, restT: Float32Array.from(b.restT) }]));
const bone: Record<string, number> = {};
rig.bones.forEach((n: string, i: number) => (bone[n] = i));
const set = {
  bones: rig.bones,
  parents: Int32Array.from(rig.parents),
  bone,
  clipBones: rig.clipBones,
  fps: rig.fps,
  sourcePelvis: rig.sourcePelvis,
  reference: rig.reference ?? null,
  bodies,
} as any;
const clips = bake(clipDefs(), set);
const CB = set.clipBones;
const refBody = bodies.get(set.reference?.body ?? 'male')!;

/** A body at a life stage, as the runtime poses it (scale 1). */
interface Skeleton {
  body: string;
  stage: string;
  restT: Float32Array;
  pelvisScale: number;
  seatDrop?: number;
  height: number;
}
function skeleton(body: string, stage: string): Skeleton {
  const b = rig.bodies[body];
  const s = stage === b.stage ? b : b.stages?.[stage];
  if (!s) throw new Error(`check_clips: ${body} has no ${stage} stage`);
  return { body, stage, restT: Float32Array.from(s.restT), pelvisScale: s.pelvisScale, seatDrop: s.seatDrop, height: s.height };
}

// --- failures and the baseline ----------------------------------------------------------------

interface Failure {
  clip: string;
  check: string;
  what: string;
  body: string;
  stage: string;
}
const failures: (Failure & { msg: string })[] = [];
/** Records a failure of `check` on `what` (a bone, a side) of `clip`; `sk` none: the same on every skeleton. */
function fail(sk: Skeleton | null, clip: string, check: string, what: string, msg: string): void {
  failures.push({ clip, check, what, body: sk?.body ?? '*', stage: sk?.stage ?? '*', msg });
}
const key = (f: Failure) => `${f.clip}|${f.check}|${f.what}|${f.body}|${f.stage}`;

// --- maths ------------------------------------------------------------------------------------

function quat(c: any, f: number, b: number): [number, number, number, number] {
  const o = (f * CB + b) * 4;
  return [c.q[o] / 32767, c.q[o + 1] / 32767, c.q[o + 2] / 32767, c.q[o + 3] / 32767];
}
function angle(a: number[], b: number[]): number {
  const d = Math.abs(a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]);
  return (2 * Math.acos(Math.min(1, d)) * 180) / Math.PI;
}
function qmul(a: number[], b: number[]): number[] {
  return [
    a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
    a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
    a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
    a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
  ];
}
function rotate(q: number[], v: number[]): number[] {
  const t = [2 * (q[1] * v[2] - q[2] * v[1]), 2 * (q[2] * v[0] - q[0] * v[2]), 2 * (q[0] * v[1] - q[1] * v[0])];
  return [v[0] + q[3] * t[0] + (q[1] * t[2] - q[2] * t[1]), v[1] + q[3] * t[1] + (q[2] * t[0] - q[0] * t[2]), v[2] + q[3] * t[2] + (q[0] * t[1] - q[1] * t[0])];
}
/**
 * World joint positions of frame f on `sk` (rest rotations are the identity), as pose.ts `skin` at
 * scale 1; `add`: model-space rotations per bone, as the runtime's `addRotation`.
 */
function fk(c: any, f: number, sk: Skeleton, add?: Map<number, number[]>): number[][] {
  const n = set.bones.length;
  const wq: number[][] = [];
  const wt: number[][] = [];
  const sp = set.sourcePelvis;
  for (let b = 0; b < n; b++) {
    const lq = b < CB ? quat(c, f, b) : [0, 0, 0, 1];
    let t = [sk.restT[b * 3], sk.restT[b * 3 + 1], sk.restT[b * 3 + 2]];
    if (b === bone.pelvis) t = [t[0] + (c.pelvis[f * 3] - sp[0]) * sk.pelvisScale, t[1] + (c.pelvis[f * 3 + 1] - sp[1]) * sk.pelvisScale, t[2] + (c.pelvis[f * 3 + 2] - sp[2]) * sk.pelvisScale];
    const p = set.parents[b];
    let q: number[];
    if (p < 0) {
      q = lq;
      wt.push(t);
    } else {
      const r = rotate(wq[p], t);
      wt.push([wt[p][0] + r[0], wt[p][1] + r[1], wt[p][2] + r[2]]);
      q = qmul(wq[p], lq);
    }
    const a = add?.get(b);
    wq.push(a ? qmul(a, q) : q);
  }
  return wt;
}
/** The rest pose (every clip bone at the identity, the pelvis at rest). */
const REST = { q: Int16Array.from({ length: CB * 4 }, (_, k) => (k % 4 === 3 ? 32767 : 0)), pelvis: Float32Array.from(set.sourcePelvis) };
function kneeBend(w: number[][], side: string): number {
  const a = w[bone[`thigh_${side}`]];
  const k = w[bone[`calf_${side}`]];
  const f = w[bone[`foot_${side}`]];
  const u = [k[0] - a[0], k[1] - a[1], k[2] - a[2]];
  const v = [f[0] - k[0], f[1] - k[1], f[2] - k[2]];
  return (Math.acos((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / Math.hypot(...u) / Math.hypot(...v)) * 180) / Math.PI;
}

// --- the same on every skeleton ---------------------------------------------------------------

for (const [name, loop] of [...Object.entries(RUNTIME), ...Object.entries(PHASE2)]) {
  const c = clips.get(name);
  if (!c) {
    fail(null, name, 'missing', '', `missing clip ${name}`);
    continue;
  }
  if (c.loop !== loop) fail(null, name, 'loop', '', `${name}: loop is ${c.loop}, the runtime expects ${loop}`);
}
for (const [name, c] of clips) {
  if ([...c.q, ...c.pelvis].some((v) => !Number.isFinite(v))) fail(null, name, 'nan', '', `${name}: NaN`);
  if (c.loop && c.frames > 2) {
    // The step from the last frame to the first is in line with the steps either side of it.
    const n = c.frames;
    for (let b = 0; b < CB; b++) {
      const near = Math.max(angle(quat(c, n - 2, b), quat(c, n - 1, b)), angle(quat(c, 0, b), quat(c, 1, b)));
      const wrap = angle(quat(c, n - 1, b), quat(c, 0, b));
      if (wrap > near + 2) fail(null, name, 'seam', set.bones[b], `${name}: ${set.bones[b]} jumps ${wrap.toFixed(1)} deg where the loop wraps`);
    }
  }
}
for (const [name, target] of Object.entries(LENGTH)) {
  const c = clips.get(name);
  if (c && Math.abs(c.duration / target - 1) > 0.15) fail(null, name, 'length', '', `${name}: ${c.duration.toFixed(2)} s, the runtime expects ~${target} s`);
}

// --- per body and stage -----------------------------------------------------------------------

/** The clips a resident of `sk`'s height plays (of the sized ones, the size characters.ts `stepFit` picks; a baby only its own). */
function clipsAt(sk: Skeleton): [string, any][] {
  const other = new Set<string>();
  for (const sizes of SIZED) {
    const pick = sizes.find(({ min }) => sk.height >= min) ?? sizes[sizes.length - 1];
    for (const s of sizes) if (s !== pick) for (const c of s.clips) other.add(c);
  }
  return [...clips].filter(([name]) => !BABY.has(name) && !other.has(name));
}

function checkSkeleton(sk: Skeleton): void {
  if (sk.stage === 'baby') return checkBaby(sk);
  const restAnkle = fk(REST, 0, sk)[bone.foot_l][1];
  const dt = 1 / set.fps;
  const sway = GAIT_SWAY[sk.body] ?? 0;
  for (const [name, c] of clipsAt(sk)) {
    if (c.speed > 0) {
      // Locomotion: the planted (lowest) foot moves back at the clip's ground speed (scaled by the
      // legs, as the runtime plays it); walking, the hips roll over the standing leg.
      const speed = c.speed * sk.pelvisScale;
      for (const side of ['l', 'r']) {
        const ys: number[] = [];
        const zs: number[] = [];
        for (let f = 0; f < c.frames; f++) {
          const roll = name === 'walk' ? Math.sin((f / c.frames) * Math.PI * 2) * sway : 0;
          const add = roll ? new Map([[bone.pelvis, [0, 0, Math.sin(roll / 2), Math.cos(roll / 2)]]]) : undefined;
          const p = fk(c, f, sk, add)[bone[`foot_${side}`]];
          ys.push(p[1]);
          zs.push(p[2]);
        }
        const low = Math.min(...ys);
        let slide = 0;
        for (let f = 0; f < c.frames; f++) {
          const g = (f + 1) % c.frames;
          if (ys[f] < low + 0.003 && ys[g] < low + 0.003) slide += Math.abs(zs[g] - zs[f] + speed * dt);
        }
        if (slide > 0.01) fail(sk, name, 'slide', `foot_${side}`, `${name}: the ${side} foot slides ${(slide * 100).toFixed(1)} cm per cycle`);
      }
    } else if (c.loop || PLANTED.has(name)) {
      // Feet flat on the floor stay put (every other loop; the short one-shots).
      for (const side of ['l', 'r']) {
        let slide = 0;
        let prev: number[] | null = null;
        const last = c.loop ? c.frames : c.frames - 1;
        for (let f = 0; f <= last; f++) {
          const p = fk(c, f % c.frames, sk)[bone[`foot_${side}`]];
          const down = Math.abs(p[1] - restAnkle) < 0.004;
          if (down && prev) slide += Math.hypot(p[0] - prev[0], p[2] - prev[2]);
          prev = down ? p : null;
        }
        if (slide > 0.01) fail(sk, name, 'slide', `foot_${side}`, `${name}: the ${side} foot slides ${(slide * 100).toFixed(1)} cm on the floor`);
      }
    }
  }
  checkSeatedAndLying(sk, restAnkle);
}

function checkSeatedAndLying(sk: Skeleton, restAnkle: number): void {
  // Seated clips share sit's pelvis (the runtime fits seats to it): within 1 cm.
  const sit = clips.get('sit');
  if (sit) {
    const at = (c: any, f: number) => fk(c, f, sk)[bone.pelvis];
    const ref0 = at(sit, 0);
    const seatedAt: [string, number][] = [
      ['sitTalk', 0],
      ['sitHands', 0],
      ['sitDown', -1],
      ['standUp', 0],
    ];
    for (const [name, f] of seatedAt) {
      const c = clips.get(name);
      if (!c) continue;
      const p = at(c, f < 0 ? c.frames - 1 : f);
      const d = Math.hypot(p[0] - ref0[0], p[1] - ref0[1], p[2] - ref0[2]);
      if (d > 0.01) fail(sk, name, 'seated', 'pelvis', `${name}: the seated pelvis is ${(d * 100).toFixed(1)} cm from sit's`);
    }
    // On the seat: the feet reach the floor (within 4 cm).
    if (sk.seatDrop !== undefined) {
      const seat = SEAT[sk.stage] ?? SEAT_HEIGHT;
      const w = fk(sit, 0, sk);
      const root = seat - (w[bone.pelvis][1] - sk.seatDrop);
      const gap = w[bone.foot_l][1] + root - restAnkle;
      if (Math.abs(gap) > 0.04) fail(sk, 'sit', 'sit', 'feet', `sit: on a ${seat} m seat the feet are ${(gap * 100).toFixed(0)} cm off the floor`);
    }
  }
  // Lying: legs nearly straight, hands in front of the body (the lengths on the reference skeleton,
  // scaled to the stage's height).
  const lie = clips.get('lie');
  if (lie) {
    const w = fk(lie, 0, sk);
    const cm5 = 0.05 * (sk.height / refBody.height);
    for (const side of ['l', 'r']) {
      const a = w[bone[`thigh_${side}`]];
      const k = w[bone[`calf_${side}`]];
      const f = w[bone[`foot_${side}`]];
      const bend = kneeBend(w, side);
      if (bend > 20) fail(sk, 'lie', 'lie-knee', `calf_${side}`, `lie: the ${side} knee is bent ${bend.toFixed(0)} deg`);
      if (w[bone[`hand_${side}`]][2] < w[bone.pelvis][2] + cm5) fail(sk, 'lie', 'lie-hand', `hand_${side}`, `lie: the ${side} hand is not in front of the body`);
      // Down on the mattress (z is up once lying): the buttocks reach about 15 cm behind the pelvis joint, the
      // calves 7 cm behind the knee and the heels 7 cm behind the ankle, so for the legs to lie on what the
      // back lies on the knees can't rise above the hips and the ankles must be well below the pelvis.
      if (k[2] > a[2]) fail(sk, 'lie', 'lie-knee-up', `calf_${side}`, `lie: the ${side} knee is ${((k[2] - a[2]) * 100).toFixed(1)} cm above the hip (the legs float)`);
      if (f[2] > w[bone.pelvis][2] - cm5) fail(sk, 'lie', 'lie-heel', `foot_${side}`, `lie: the ${side} ankle is ${((f[2] - w[bone.pelvis][2]) * 100).toFixed(1)} cm from the pelvis, not under it (the heels float)`);
    }
  }
}

function checkBaby(sk: Skeleton): void {
  // The baby lying in the crib: on its back on the mattress (nothing behind the back of the pelvis and
  // the chest), within reach of the middle (not through the bars), the knees bent (drawn up). The
  // limits are on the reference skeleton, scaled to the stage's height.
  const k = sk.height / refBody.height;
  for (const name of BABY) {
    const c = clips.get(name);
    if (!c) continue;
    let behind = 0;
    let wide = 0;
    let straight = 180;
    for (let f = 0; f < c.frames; f++) {
      const w = fk(c, f, sk);
      const back = Math.min(w[bone.pelvis][2], w[bone.spine_03][2]);
      for (let b = 1; b < CB; b++) {
        behind = Math.max(behind, back - w[b][2]);
        wide = Math.max(wide, Math.abs(w[b][0] - w[bone.pelvis][0]));
      }
      for (const side of ['l', 'r']) straight = Math.min(straight, kneeBend(w, side));
    }
    if (process.env.VERBOSE) console.log(`${sk.body}/${sk.stage} ${name}: behind ${(behind * 100).toFixed(1)} cm, out ${(wide * 100).toFixed(1)} cm, knees bent at least ${straight.toFixed(0)} deg`);
    if (behind > 0.03 * k) fail(sk, name, 'baby-behind', '', `${name}: a joint is ${(behind * 100).toFixed(1)} cm behind the back (under the mattress)`);
    if (wide > BABY_REACH * k) fail(sk, name, 'baby-wide', '', `${name}: a joint reaches ${(wide * 100).toFixed(0)} cm out to the side (through the crib's bars)`);
    if (straight < 25) fail(sk, name, 'baby-knee', '', `${name}: a knee straightens to ${straight.toFixed(0)} deg`);
  }
}

for (const b of bodyNames) for (const s of stageNames) checkSkeleton(skeleton(b, s));

// --- report -----------------------------------------------------------------------------------

/** Whether the run checked the body and stage of `f` (`*`: every run does). */
const inRun = (f: Failure) => (f.body === '*' || bodyNames.includes(f.body)) && (f.stage === '*' || stageNames.includes(f.stage));
let known: Failure[] = [];
try {
  known = JSON.parse(readFileSync(BASELINE, 'utf8')).known ?? [];
} catch (err: any) {
  if (err.code !== 'ENOENT') throw err;
}
const knownKeys = new Set(known.map(key));
const failing = new Set(failures.map(key));
const where = (f: Failure) => (f.body === '*' ? '' : `${f.body}/${f.stage} `);
let fresh = 0;
for (const f of failures) {
  const old = knownKeys.has(key(f));
  if (!old) fresh++;
  console.log(`${old ? 'known' : 'FAIL '} ${where(f)}${f.msg}`);
}
const fixed = known.filter((f) => inRun(f) && !failing.has(key(f)));
for (const f of fixed) console.log(`fixed — remove from the baseline: ${where(f)}${f.clip} ${f.check} ${f.what}`.trimEnd());
console.log(`check_clips: ${clips.size} clips on ${bodyNames.join(', ')} at ${stageNames.join(', ')}: ${fresh} new, ${failures.length - fresh} known, ${fixed.length} fixed`);

if (updateBaseline) {
  const seen = new Set<string>();
  const entries = [...known.filter((f) => !inRun(f)), ...failures]
    .map(({ clip, check, what, body, stage }) => ({ clip, check, what, body, stage }))
    .filter((f) => !seen.has(key(f)) && seen.add(key(f)))
    .sort((a, b) => (key(a) < key(b) ? -1 : key(a) > key(b) ? 1 : 0));
  const about = 'Known failures of tools/characters/check_clips.ts: printed as known, not failing. Body and stage "*": the same on every skeleton. Rewrite with --update-baseline.';
  writeFileSync(BASELINE, `{\n  "about": ${JSON.stringify(about)},\n  "known": [\n${entries.map((e) => `    ${JSON.stringify(e)}`).join(',\n')}\n  ]\n}\n`);
  console.log(`check_clips: wrote ${entries.length} known failures to ${BASELINE}`);
} else if (fresh) process.exit(1);
