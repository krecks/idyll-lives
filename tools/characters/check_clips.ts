/**
 * Checks the residents' clips (web/src/render/babylon/characters/clips) baked on the built rig:
 *
 *     node --experimental-strip-types tools/characters/check_clips.ts [rig.json]
 *
 * - every clip the runtime plays exists, with the right loop flag;
 * - no NaNs; loops wrap seamlessly (the step from the last frame to the first is in line with the
 *   steps either side of it: at most 1.5x, plus 2 degrees);
 * - the planted foot of `walk` slides less than 1 cm per cycle against the ground speed;
 * - the lengths the runtime sequences on (sitDown, standUp, eat, interact, yes, pickUp) are within
 *   15% of their targets;
 * - seated on a 0.45 m seat (`sit` fitted with the body's `seatDrop`), the feet reach the floor.
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
  idle: true, talk: true, walk: true, jog: true, sit: true, sitTalk: true, sitDown: false, standUp: false, sitHands: true,
  interact: false, pickUp: false, dance: true, punchJab: false, punchCross: false, hit: false, kneel: true, crouch: true,
  eat: false, water: true, harvest: true, foldArms: true, no: true, yes: false, phone: true, rail: true, spell: true,
  push: true, plant: false, lantern: true,
};
const LENGTH: Record<string, number> = { sitDown: 1.33, standUp: 1.07, eat: 1.33, interact: 2.0, yes: 2.5, pickUp: 0.87 };

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

for (const [name, loop] of Object.entries(RUNTIME)) {
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
      if (wrap > near * 1.5 + 2) errors.push(`${name}: ${set.bones[b]} jumps ${wrap.toFixed(1)} deg where the loop wraps`);
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
