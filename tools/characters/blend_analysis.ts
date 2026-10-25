/**
 * Exploratory: measures what the runtime's crossfade pipeline (residents.ts `start` / `sampleClip` /
 * `blendInto`, pose.ts) does to the baked clips. Not a check; prints numbers.
 *
 *     bun tools/characters/blend_analysis.ts
 *
 * 1. The transitions the runtime actually starts (from `ACTIONS`' sequences, and idle/walk in and
 *    out of them): how wide they are, and which are widest.
 * 2. Clips that bake a big pelvis yaw (the turns at the end of the slide, the ladder, the climb).
 * 3. The pop when a fade is interrupted just under the runtime's 0.5 rule (`start`, `rig.w < 0.5`).
 * 4. The pop when the upper-body clip is swapped at full weight (residents.ts:2089).
 * 5. nlerp vs slerp at the midpoint, for the widest of those.
 */

import { readFileSync } from "node:fs";

const { clipDefs } =
  await import("../../web/src/render/shared/characters/clips/index.ts");
const { bake } =
  await import("../../web/src/render/shared/characters/clips/dsl.ts");
const { sampleClip } =
  await import("../../web/src/render/shared/characters/pose.ts");
const { ACTIONS, STEP_IN } =
  await import("../../web/src/render/shared/characters/actions.ts");

const path = new URL(
  "../../web/public/assets/characters/rig.json",
  import.meta.url,
).pathname;
const rig = JSON.parse(readFileSync(path, "utf8"));
const bodies = new Map(
  Object.entries(rig.bodies).map(([name, b]: [string, any]) => [
    name,
    { ...b, restT: Float32Array.from(b.restT) },
  ]),
);
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
const NB = rig.bones.length;
const q1 = new Float32Array(NB * 4);
const q2 = new Float32Array(NB * 4);
const qN = new Float32Array(NB * 4);
const qS = new Float32Array(NB * 4);
const p = new Float32Array(3);

function pose(name: string, t: number, q: Float32Array): void {
  const c = clips.get(name);
  if (!c) throw new Error(`no clip ${name}`);
  sampleClip(set, c, t, q, p);
}
/** Angle (deg) between two local rotations. */
function ang(a: Float32Array, b: Float32Array, k: number): number {
  const o = k * 4;
  const d = Math.abs(
    a[o] * b[o] +
      a[o + 1] * b[o + 1] +
      a[o + 2] * b[o + 2] +
      a[o + 3] * b[o + 3],
  );
  return (2 * Math.acos(Math.min(1, d)) * 180) / Math.PI;
}
function diff(
  a: Float32Array,
  b: Float32Array,
  mask: Float32Array | null = null,
): { max: number; bone: string; rms: number } {
  let max = 0;
  let at = 0;
  let sum = 0;
  for (let k = 0; k < NB; k++) {
    if (mask && mask[k] <= 0) continue;
    const d = ang(a, b, k);
    sum += d * d;
    if (d > max) [max, at] = [d, k];
  }
  return { max, bone: rig.bones[at], rms: Math.sqrt(sum / NB) };
}
function nlerp(
  a: Float32Array,
  b: Float32Array,
  t: number,
  out: Float32Array,
): void {
  for (let k = 0; k < NB; k++) {
    const o = k * 4;
    let x1 = b[o];
    let y1 = b[o + 1];
    let z1 = b[o + 2];
    let w1 = b[o + 3];
    if (a[o] * x1 + a[o + 1] * y1 + a[o + 2] * z1 + a[o + 3] * w1 < 0)
      [x1, y1, z1, w1] = [-x1, -y1, -z1, -w1];
    const x = a[o] + (x1 - a[o]) * t;
    const y = a[o + 1] + (y1 - a[o + 1]) * t;
    const z = a[o + 2] + (z1 - a[o + 2]) * t;
    const w = a[o + 3] + (w1 - a[o + 3]) * t;
    const l = 1 / Math.sqrt(x * x + y * y + z * z + w * w);
    out[o] = x * l;
    out[o + 1] = y * l;
    out[o + 2] = z * l;
    out[o + 3] = w * l;
  }
}
function slerp(
  a: Float32Array,
  b: Float32Array,
  t: number,
  out: Float32Array,
): void {
  for (let k = 0; k < NB; k++) {
    const o = k * 4;
    let x1 = b[o];
    let y1 = b[o + 1];
    let z1 = b[o + 2];
    let w1 = b[o + 3];
    let dot = a[o] * x1 + a[o + 1] * y1 + a[o + 2] * z1 + a[o + 3] * w1;
    if (dot < 0) {
      [x1, y1, z1, w1] = [-x1, -y1, -z1, -w1];
      dot = -dot;
    }
    dot = Math.min(1, dot);
    const th = Math.acos(dot);
    const s = Math.sin(th);
    const ka = s < 1e-6 ? 1 - t : Math.sin((1 - t) * th) / s;
    const kb = s < 1e-6 ? t : Math.sin(t * th) / s;
    out[o] = a[o] * ka + x1 * kb;
    out[o + 1] = a[o + 1] * ka + y1 * kb;
    out[o + 2] = a[o + 2] * ka + z1 * kb;
    out[o + 3] = a[o + 3] * ka + w1 * kb;
  }
}
const smooth = (t: number) => t * t * (3 - 2 * t);
const dur = (n: string) => clips.get(n)!.duration;
const samples = (n: string) => [
  0,
  dur(n) * 0.25,
  dur(n) * 0.5,
  dur(n) * 0.75,
  Math.max(0, dur(n) - 1 / rig.fps),
];
/** Worst per-bone difference over 5x5 phase samples. */
function widest(a: string, b: string, mask: Float32Array | null = null) {
  let worst = { max: 0, bone: "", rms: 0 };
  for (const ta of samples(a)) {
    pose(a, ta, q1);
    for (const tb of samples(b)) {
      pose(b, tb, q2);
      const d = diff(q1, q2, mask);
      if (d.max > worst.max) worst = d;
      if (d.rms > worst.rms) worst.rms = d.rms;
    }
  }
  return worst;
}
const upperMask = new Float32Array(NB);
for (let b = 0; b < NB; b++) {
  let p2 = b;
  let w = 0;
  while (p2 >= 0) {
    const n = rig.bones[p2];
    if (n === "spine_02") {
      w = 1;
      break;
    }
    if (n === "spine_01" && p2 === b) w = 0.5;
    p2 = set.parents[p2];
  }
  upperMask[b] = w;
}

// --- 1. the transitions the runtime starts -----------------------------------------------------

/** [from, to, fade] the runtime can start: a tag's sequence steps, and idle/walk in and out of them. */
const edges = new Map<
  string,
  { a: string; b: string; fade: number; why: string }
>();
const addEdge = (a: string, b: string, fade: number, why: string) => {
  if (a === b || !clips.has(a) || !clips.has(b)) return;
  const k = `${a}|${b}`;
  if (!edges.has(k)) edges.set(k, { a, b, fade, why });
};
const SEQ_FADE = 0.3;
for (const [tag, act] of Object.entries(ACTIONS) as [string, any][]) {
  const seq: readonly string[] | undefined = Array.isArray(act.stand)
    ? act.stand
    : undefined;
  if (seq?.length) {
    addEdge("idle", seq[0], 0.25, `${tag} start`);
    addEdge("walk", seq[0], 0.25, `${tag} start (walked in)`);
    for (let i = 0; i + 1 < seq.length; i++)
      addEdge(seq[i], seq[i + 1], SEQ_FADE, `${tag} step`);
    addEdge(seq[seq.length - 1], seq[0], SEQ_FADE, `${tag} repeat`);
    addEdge(seq[seq.length - 1], "idle", 0.25, `${tag} end`);
    addEdge(seq[seq.length - 1], "walk", 0.25, `${tag} end (walk off)`);
  }
  const sit = typeof act.sit === "string" ? act.sit : undefined;
  if (sit) {
    addEdge("sitDown", sit, SEQ_FADE, `${tag} seated`);
    addEdge(sit, "standUp", SEQ_FADE, `${tag} up`);
  }
}
for (const [def, step] of Object.entries(STEP_IN) as [string, any][]) {
  for (const [h, seq] of step.by as [number, readonly string[]][]) {
    if (!seq?.length) continue;
    addEdge("idle", seq[0], 0.25, `${def} step in`);
    addEdge("walk", seq[0], 0.25, `${def} step in (walked)`);
    for (let i = 0; i + 1 < seq.length; i++)
      addEdge(seq[i], seq[i + 1], SEQ_FADE, `${def} step`);
    addEdge(seq[seq.length - 1], "idle", 0.25, `${def} step out`);
  }
}
addEdge("idle", "walk", 0.25, "locomotion");
addEdge("walk", "idle", 0.25, "locomotion");
addEdge("walk", "jogAlong", 0.25, "locomotion");
addEdge("jogAlong", "walk", 0.25, "locomotion");
addEdge("sitDown", "sit", SEQ_FADE, "sitting");
addEdge("sit", "standUp", SEQ_FADE, "standing");
addEdge("standUp", "walk", 0.25, "rising");

/** The handoff the sequence actually makes: `a` at its end (or a loop's start), `b` at its start. */
function natural(a: string, b: string) {
  pose(a, Math.max(0, dur(a) - 1 / rig.fps), q1);
  pose(b, 0, q2);
  return diff(q1, q2);
}
const rows = [...edges.values()].map((e) => ({
  ...e,
  ...natural(e.a, e.b),
  worst: widest(e.a, e.b).max,
}));
rows.sort((x, y) => y.max - x.max);
const over = (d: number) => rows.filter((r) => r.max > d).length;
const h = rows.map((r) => r.max).sort((a, b) => a - b);
console.log(
  `  handoff width: median ${h[Math.floor(h.length / 2)].toFixed(0)}°, p90 ${h[Math.floor(h.length * 0.9)].toFixed(0)}°, max ${h[h.length - 1].toFixed(0)}°`,
);
console.log(
  `clips: ${clips.size}; transitions the runtime can start (ACTIONS + STEP_IN): ${rows.length}`,
);
console.log(
  `  over 45°: ${over(45)}, over 90°: ${over(90)}, over 120°: ${over(120)}`,
);
console.log(
  "\n=== 1. widest reachable transitions at the handoff (deg; `worst` = any phase pair) ===",
);
for (const r of rows.slice(0, 15))
  console.log(
    `  ${r.max.toFixed(0).padStart(4)}° (worst ${r.worst.toFixed(0).padStart(3)}°) rms ${r.rms.toFixed(0).padStart(2)}° ${r.bone.padEnd(11)} ${r.a} -> ${r.b}   (${r.why}, fade ${r.fade}s)`,
  );

// --- 2. baked pelvis yaw ----------------------------------------------------------------------

const yawOf = (k: number) => {
  const o = k * 4;
  return (
    (2 *
      Math.atan2(Math.hypot(q1[o], q1[o + 1], q1[o + 2]), Math.abs(q1[o + 3])) *
      180) /
    Math.PI
  );
};
const spinners: { clip: string; yaw: number }[] = [];
for (const n of clips.keys()) {
  let max = 0;
  for (const t of [
    0,
    ...Array.from({ length: 20 }, (_, i) => (dur(n) * (i + 1)) / 20),
  ]) {
    pose(n, t, q1);
    max = Math.max(max, yawOf(bone.pelvis));
  }
  if (max > 45) spinners.push({ clip: n, yaw: max });
}
spinners.sort((a, b) => b.yaw - a.yaw);
console.log(
  "\n=== 2. clips that turn the pelvis by more than 45° (baked yaw, deg) ===",
);
console.log(
  `  ${spinners.length}/${clips.size}: ${spinners
    .slice(0, 18)
    .map((s) => `${s.clip} ${s.yaw.toFixed(0)}°`)
    .join(", ")}${spinners.length > 18 ? ", …" : ""}`,
);
const spinEdges = rows.filter(
  (r) => spinners.some((s) => s.clip === r.a || s.clip === r.b) && r.max > 90,
);
console.log(
  `  reachable transitions over 90° that involve one: ${spinEdges.length} (${spinEdges
    .slice(0, 6)
    .map((r) => `${r.a}->${r.b}`)
    .join(", ")})`,
);

// --- 3. interrupting a fade -------------------------------------------------------------------

console.log(
  "\n=== 3. pop when a fade is interrupted at the worst weight under the 0.5 rule ===",
);
const wI = 0.499;
// Both players advance through the fade, so the interrupt lands at a correlated phase pair: `f` of the
// fade in, `a` at t0 + f*fade and `b` at f*fade.
const pops = rows.map((r) => {
  let worst = { max: 0, bone: "", rms: 0 };
  // A one-shot only ever fades out at its end; a loop can be caught at any phase.
  const starts = clips.get(r.a)!.loop
    ? samples(r.a)
    : [Math.max(0, dur(r.a) - 1 / rig.fps)];
  for (const t0 of starts) {
    for (const f of [0.5, 0.75, 0.999]) {
      const dt = f * r.fade;
      const w = f * 0.5;
      pose(r.a, Math.min(dur(r.a), t0 + dt), q1);
      pose(r.b, Math.min(dur(r.b) - 1 / rig.fps, dt), q2);
      nlerp(q1, q2, smooth(w), qN); // the pose the fade is at
      const d = diff(qN, q2); // against pure b: what the next fade starts from
      if (d.max > worst.max) worst = d;
    }
  }
  return { ...r, pop: worst };
});
pops.sort((x, y) => y.pop.max - x.pop.max);
const popVals = pops.map((r) => r.pop.max).sort((a, b) => a - b);
const q = (v: number, p2: number) => v[Math.floor((v.length - 1) * p2)];
console.log(
  `  median ${q(popVals, 0.5).toFixed(0)}°, p90 ${q(popVals, 0.9).toFixed(0)}°, max ${popVals[popVals.length - 1].toFixed(0)}°; over 30°: ${popVals.filter((v) => v > 30).length}/${popVals.length}`,
);
for (const r of pops.slice(0, 10)) {
  const frames = Math.round(wI * r.fade * 60);
  console.log(
    `  ${r.pop.max.toFixed(0).padStart(3)}° at ${r.pop.bone.padEnd(11)} rms ${r.pop.rms.toFixed(0).padStart(2)}°  ${r.a} -> ${r.b} (fade ${r.fade}s, ${frames} frames at 60 fps at risk)`,
  );
}

// --- 3b. what raising the 0.5 rule to T would leave ---------------------------------------------

console.log(
  "\n=== 3b. the pop left, by the weight `start` keeps `prev` up to (one-line change) ===",
);
const deltas = rows.map((r) => r.max);
for (const T of [0.5, 0.7, 0.85, 0.9, 0.95]) {
  const left = deltas.map((d) => (1 - smooth(T)) * d).sort((a, b) => a - b);
  console.log(
    `  T=${T}: pop median ${q(left, 0.5).toFixed(1).padStart(4)}°, p90 ${q(left, 0.9).toFixed(1).padStart(5)}°, max ${left[left.length - 1].toFixed(1).padStart(5)}°`,
  );
}

// --- 4. upper-body clip swapped at full weight -------------------------------------------------

console.log(
  "\n=== 4. upper-body clip swapped at full weight (residents.ts:2089) ===",
);
const upperPairs: [string, string][] = [
  ["carry", "eat"],
  ["carry", "carryOne"],
  ["eat", "drink"],
  ["brushHair", "carry"],
  ["phoneText", "eat"],
  ["carry", "phoneText"],
];
for (const [a, b] of upperPairs) {
  if (!clips.has(a) || !clips.has(b)) {
    console.log(`  ${a} -> ${b}: missing`);
    continue;
  }
  const d = widest(a, b, upperMask);
  console.log(
    `  ${a} -> ${b}: ${d.max.toFixed(0)}° at ${d.bone} in one frame (weight stays 1)`,
  );
}
console.log(
  `  the upper layer takes over fully: ${[...upperMask].filter((w) => w === 1).length}/${NB} bones, half: ${[...upperMask].filter((w) => w === 0.5).length}`,
);

// --- 5. nlerp vs slerp ------------------------------------------------------------------------

console.log("\n=== 5. nlerp vs slerp, for the widest transitions ===");
let worstDev = { dev: 0, bone: "", pair: 0, why: "" };
for (const r of rows.slice(0, 30)) {
  for (const ta of samples(r.a)) {
    pose(r.a, ta, q1);
    for (const tb of samples(r.b)) {
      pose(r.b, tb, q2);
      nlerp(q1, q2, 0.5, qN);
      slerp(q1, q2, 0.5, qS);
      const d = diff(qN, qS);
      if (d.max > worstDev.dev)
        worstDev = {
          dev: d.max,
          bone: d.bone,
          pair: diff(q1, q2).max,
          why: `${r.a} -> ${r.b}`,
        };
    }
  }
}
console.log(
  `  midpoint: worst ${worstDev.dev.toFixed(1)}° off (${worstDev.bone}) on ${worstDev.why}, a ${worstDev.pair.toFixed(0)}° pair`,
);
// nlerp is the same great circle, but not at a constant rate: a wide pair swings fastest mid-fade.
const rate = (a: number, t: number, n: boolean) => {
  const rad = (a * Math.PI) / 180;
  if (n)
    return (
      (2 *
        Math.atan2(
          (1 - t) * Math.sin(rad / 2),
          (1 - t) * Math.cos(rad / 2) + t,
        ) *
        180) /
      Math.PI
    );
  return a * t;
};
let worstRate = { err: 0, pair: 0, why: "" };
for (const r of rows.slice(0, 30)) {
  for (const t of [0.15, 0.25, 0.35, 0.65, 0.75, 0.85]) {
    const err = Math.abs(rate(r.max, t, true) - rate(r.max, t, false));
    if (err > worstRate.err)
      worstRate = { err, pair: r.max, why: `${r.a} -> ${r.b}` };
  }
}
console.log(
  `  rate: worst ${worstRate.err.toFixed(1)}° of arc off the constant rate on ${worstRate.why}, a ${worstRate.pair.toFixed(0)}° pair (max swing speed x${(1 / Math.cos((worstRate.pair * Math.PI) / 720)).toFixed(2)})`,
);
let near = 0;
for (const r of rows)
  if (Math.abs(Math.cos((r.max * Math.PI) / 360)) < 0.1) near++;
console.log(
  `  near-antipodal handoffs (|dot| < 0.1, the sign of the arc ambiguous): ${near}/${rows.length}`,
);
