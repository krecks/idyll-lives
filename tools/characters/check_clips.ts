/**
 * Checks the residents' clips (web/src/render/babylon/characters/clips) baked on the built rig:
 *
 *     node --experimental-strip-types tools/characters/check_clips.ts [rig.json]
 *
 * - every clip the runtime plays exists, with the right loop flag (and the phase-2 clips too);
 * - no NaNs; loops wrap seamlessly (the step from the last frame to the first is in line with the
 *   steps either side of it: at most 2 degrees more than the larger);
 * - the planted foot of `walk` slides less than 1 cm per cycle against the ground speed, and a
 *   foot flat on the floor in any other loop (or the short one-shots) stays put (< 1 cm);
 * - every clip's length is within 15% of its target (the runtime sequences on them);
 * - seated on a 0.45 m seat (`sit` fitted with the body's `seatDrop`), the feet reach the floor,
 *   and the other seated clips (and the seated ends of sitDown / standUp) keep `sit`'s pelvis;
 * - `lie` lies straight and down: knees nearly straight and not above the hips, the ankles below the
 *   pelvis (the legs on the mattress, not floating), the hands in front of the body;
 * - the baby's lying clips keep the baby on the mattress and inside the crib: no joint more than
 *   3 cm (reference skeleton) behind the back, none further out to the side than `BABY_REACH`, the
 *   knees bent.
 * Exits non-zero on any failure.
 */

import { readFileSync } from 'node:fs';
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

/** Every clip the runtime plays (characters.ts ACTIONS / choose, reactions.ts) and whether it loops. */
const RUNTIME: Record<string, boolean> = {
  idle: true, talk: true, walk: true, jog: true, sit: true, sitTalk: true, sitDown: false, standUp: false, sitHands: true, sitHandsNear: true,
  interact: false, pickUp: false, dance: true, punchJab: false, punchCross: false, hit: false, kneel: true, crouch: true,
  eat: false, water: true, harvest: true, pick: true, foldArms: true, no: true, yes: false, phone: true, rail: true, spell: true,
  push: true, plant: false, lantern: true, babyLieAsleep: true, babyLieAwake: true,
  // The drinking fountain and the climbing frame, in sizes (characters.ts STEP_IN, clips/street.ts).
  fountainDrink181: true, fountainDrink163: true, fountainDrink147: true, fountainDrink129: true, fountainDrink112: true,
  climb172: true, climb147: true, climb112: true,
};
/** Phase-2 clips (the runtime picks them up later) and whether they loop. */
const PHASE2: Record<string, boolean> = { lie: true, chop: true, stir: true, flip: true, drink: false, carry: true, carryOne: true, wave: false };
/** Lengths (s) from the plan's clip table; within 15%. */
const LENGTH: Record<string, number> = {
  idle: 2.5, talk: 2.9, walk: 1.1, jog: 0.7, sit: 1.7, sitTalk: 2.9, sitDown: 1.33, standUp: 1.07, sitHands: 1.7, sitHandsNear: 1.7, interact: 2.0, pickUp: 0.87,
  dance: 1.0, punchJab: 0.87, punchCross: 1.0, hit: 0.33, kneel: 5.2, crouch: 2.9, eat: 1.33, water: 3.8, harvest: 2.5, pick: 3, plant: 2.8, foldArms: 2.5,
  yes: 2.5, no: 2.5, phone: 2.9, lantern: 2.5, rail: 2.5, spell: 2.1, push: 2.7, babyLieAsleep: 9, babyLieAwake: 6,
  fountainDrink181: 3, fountainDrink163: 3, fountainDrink147: 3, fountainDrink129: 3, fountainDrink112: 3, climb172: 8, climb147: 8, climb112: 8,
};
/** The baby lying in the crib: how far out to the side a joint may reach (m, reference skeleton: about 0.23 m on a baby, whose crib's bars are about 0.4 m from its middle). */
const BABY_REACH = 0.6;
/** One-shots whose feet stay planted (sitting down and standing up move them on purpose). */
const PLANTED = new Set(['interact', 'pickUp', 'eat', 'yes', 'punchJab', 'punchCross', 'hit', 'plant', 'drink', 'wave']);

const path = process.argv[2] ?? new URL('../../web/public/assets/characters/rig.json', import.meta.url).pathname;
const rig = JSON.parse(readFileSync(path, 'utf8'));
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
const errors: string[] = [];
const CB = set.clipBones;
const ref = bodies.get(set.reference?.body ?? 'male')!;

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
/** World joint positions of frame f on the reference skeleton (rest rotations are the identity). */
function fk(c: any, f: number): number[][] {
  const n = set.bones.length;
  const wq: number[][] = [];
  const wt: number[][] = [];
  for (let b = 0; b < n; b++) {
    const lq = b < CB ? quat(c, f, b) : [0, 0, 0, 1];
    let t = [ref.restT[b * 3], ref.restT[b * 3 + 1], ref.restT[b * 3 + 2]];
    if (b === bone.pelvis) t = [t[0] + c.pelvis[f * 3] - set.sourcePelvis[0], t[1] + c.pelvis[f * 3 + 1] - set.sourcePelvis[1], t[2] + c.pelvis[f * 3 + 2] - set.sourcePelvis[2]];
    const p = set.parents[b];
    if (p < 0) {
      wq.push(lq);
      wt.push(t);
    } else {
      const r = rotate(wq[p], t);
      wt.push([wt[p][0] + r[0], wt[p][1] + r[1], wt[p][2] + r[2]]);
      wq.push(qmul(wq[p], lq));
    }
  }
  return wt;
}

for (const [name, loop] of [...Object.entries(RUNTIME), ...Object.entries(PHASE2)]) {
  const c = clips.get(name);
  if (!c) {
    errors.push(`missing clip ${name}`);
    continue;
  }
  if (c.loop !== loop) errors.push(`${name}: loop is ${c.loop}, the runtime expects ${loop}`);
}
for (const [name, c] of clips) {
  if ([...c.q, ...c.pelvis].some((v) => !Number.isFinite(v))) errors.push(`${name}: NaN`);
  if (c.loop && c.frames > 2) {
    // The step from the last frame to the first is in line with the steps either side of it.
    const n = c.frames;
    for (let b = 0; b < CB; b++) {
      const near = Math.max(angle(quat(c, n - 2, b), quat(c, n - 1, b)), angle(quat(c, 0, b), quat(c, 1, b)));
      const wrap = angle(quat(c, n - 1, b), quat(c, 0, b));
      if (wrap > near + 2) errors.push(`${name}: ${set.bones[b]} jumps ${wrap.toFixed(1)} deg where the loop wraps`);
    }
  }
}
for (const [name, target] of Object.entries(LENGTH)) {
  const c = clips.get(name);
  if (c && Math.abs(c.duration / target - 1) > 0.15) errors.push(`${name}: ${c.duration.toFixed(2)} s, the runtime expects ~${target} s`);
}
// Walk: the planted (lowest) foot moves back at the clip's speed.
const walk = clips.get('walk');
if (walk) {
  const dt = 1 / set.fps;
  for (const side of ['l', 'r']) {
    const ys: number[] = [];
    const zs: number[] = [];
    for (let f = 0; f < walk.frames; f++) {
      const p = fk(walk, f)[bone[`foot_${side}`]];
      ys.push(p[1]);
      zs.push(p[2]);
    }
    const low = Math.min(...ys);
    let slide = 0;
    for (let f = 0; f < walk.frames; f++) {
      const g = (f + 1) % walk.frames;
      if (ys[f] < low + 0.003 && ys[g] < low + 0.003) slide += Math.abs(zs[g] - zs[f] + walk.speed * dt);
    }
    if (slide > 0.01) errors.push(`walk: the ${side} foot slides ${(slide * 100).toFixed(1)} cm per cycle`);
  }
}
// Feet flat on the floor stay put (every loop but walk; the short one-shots).
const restAnkle = fk({ q: Int16Array.from({ length: CB * 4 }, (_, k) => (k % 4 === 3 ? 32767 : 0)), pelvis: Float32Array.from(set.sourcePelvis) }, 0)[bone.foot_l][1];
for (const [name, c] of clips) {
  if (name === 'walk' || !(c.loop || PLANTED.has(name))) continue;
  for (const side of ['l', 'r']) {
    let slide = 0;
    let prev: number[] | null = null;
    const last = c.loop ? c.frames : c.frames - 1;
    for (let f = 0; f <= last; f++) {
      const p = fk(c, f % c.frames)[bone[`foot_${side}`]];
      const down = Math.abs(p[1] - restAnkle) < 0.004;
      if (down && prev) slide += Math.hypot(p[0] - prev[0], p[2] - prev[2]);
      prev = down ? p : null;
    }
    if (slide > 0.01) errors.push(`${name}: the ${side} foot slides ${(slide * 100).toFixed(1)} cm on the floor`);
  }
}
// Seated clips share sit's pelvis (the runtime fits seats to it): within 1 cm.
const sitClip = clips.get('sit');
if (sitClip) {
  const at = (c: any, f: number) => fk(c, f)[bone.pelvis];
  const ref0 = at(sitClip, 0);
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
    if (d > 0.01) errors.push(`${name}: the seated pelvis is ${(d * 100).toFixed(1)} cm from sit's`);
  }
}
// Lying: legs nearly straight, hands in front of the body.
const lie = clips.get('lie');
if (lie) {
  const w = fk(lie, 0);
  for (const side of ['l', 'r']) {
    const a = w[bone[`thigh_${side}`]];
    const k = w[bone[`calf_${side}`]];
    const f = w[bone[`foot_${side}`]];
    const u = [k[0] - a[0], k[1] - a[1], k[2] - a[2]];
    const v = [f[0] - k[0], f[1] - k[1], f[2] - k[2]];
    const bend = (Math.acos((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / Math.hypot(...u) / Math.hypot(...v)) * 180) / Math.PI;
    if (bend > 20) errors.push(`lie: the ${side} knee is bent ${bend.toFixed(0)} deg`);
    if (w[bone[`hand_${side}`]][2] < w[bone.pelvis][2] + 0.05) errors.push(`lie: the ${side} hand is not in front of the body`);
    // Down on the mattress (z is up once lying): the buttocks reach about 15 cm behind the pelvis joint, the
    // calves 7 cm behind the knee and the heels 7 cm behind the ankle, so for the legs to lie on what the
    // back lies on the knees can't rise above the hips and the ankles must be well below the pelvis.
    if (k[2] > a[2]) errors.push(`lie: the ${side} knee is ${((k[2] - a[2]) * 100).toFixed(1)} cm above the hip (the legs float)`);
    if (f[2] > w[bone.pelvis][2] - 0.05) errors.push(`lie: the ${side} ankle is ${((f[2] - w[bone.pelvis][2]) * 100).toFixed(1)} cm from the pelvis, not under it (the heels float)`);
  }
}
// The baby lying in the crib: on its back on the mattress (nothing behind the back of the pelvis and
// the chest), within reach of the middle (not through the bars), the knees bent (drawn up).
for (const name of ['babyLieAsleep', 'babyLieAwake']) {
  const c = clips.get(name);
  if (!c) continue;
  let behind = 0;
  let wide = 0;
  let straight = 180;
  for (let f = 0; f < c.frames; f++) {
    const w = fk(c, f);
    const back = Math.min(w[bone.pelvis][2], w[bone.spine_03][2]);
    for (let b = 1; b < CB; b++) {
      behind = Math.max(behind, back - w[b][2]);
      wide = Math.max(wide, Math.abs(w[b][0] - w[bone.pelvis][0]));
    }
    for (const side of ['l', 'r']) {
      const a = w[bone[`thigh_${side}`]];
      const k = w[bone[`calf_${side}`]];
      const t = w[bone[`foot_${side}`]];
      const u = [k[0] - a[0], k[1] - a[1], k[2] - a[2]];
      const v = [t[0] - k[0], t[1] - k[1], t[2] - k[2]];
      straight = Math.min(straight, (Math.acos((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / Math.hypot(...u) / Math.hypot(...v)) * 180) / Math.PI);
    }
  }
  if (process.env.VERBOSE) console.log(`${name}: behind ${(behind * 100).toFixed(1)} cm, out ${(wide * 100).toFixed(1)} cm, knees bent at least ${straight.toFixed(0)} deg`);
  if (behind > 0.03) errors.push(`${name}: a joint is ${(behind * 100).toFixed(1)} cm behind the back (under the mattress)`);
  if (wide > BABY_REACH) errors.push(`${name}: a joint reaches ${(wide * 100).toFixed(0)} cm out to the side (through the crib's bars)`);
  if (straight < 25) errors.push(`${name}: a knee straightens to ${straight.toFixed(0)} deg`);
}
// Seated on a 0.45 m seat: the feet reach the floor (within 4 cm).
const sit = clips.get('sit');
if (sit && ref.seatDrop !== undefined) {
  const w = fk(sit, 0);
  const root = 0.45 - (w[bone.pelvis][1] - ref.seatDrop);
  const foot = fk({ ...sit, q: Int16Array.from({ length: sit.q.length }, (_, k) => (k % 4 === 3 ? 32767 : 0)), pelvis: Float32Array.from(set.sourcePelvis) }, 0)[bone.foot_l][1];
  const gap = w[bone.foot_l][1] + root - foot;
  if (Math.abs(gap) > 0.04) errors.push(`sit: on a 0.45 m seat the feet are ${(gap * 100).toFixed(0)} cm off the floor`);
}

if (errors.length) {
  for (const e of errors) console.log('FAIL', e);
  process.exit(1);
}
console.log(`check_clips: ${clips.size} clips ok`);
