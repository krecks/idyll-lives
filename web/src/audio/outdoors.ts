/**
 * The street and the weather, synthesised (no sound files): a car's engine and its tyres on the road, the
 * town heard from above, and the wind in the leaves and the grass. Each a loop made once (off the main
 * thread, `water.worker.ts`), its end running on into its start; `outside.ts` plays them by what the
 * camera sees: a car by its speed and how near it passes, the town by how far out the camera is, the
 * leaves by the gusts the grass shows.
 *
 * - An engine heard from the kerb is a low hum: the firing's tone (55 Hz at the loop's own speed; played
 *   faster as the car goes faster) and its overtones, each wavering a little, with the rasp of the exhaust
 *   under it. Nothing much over 1 kHz gets out from under a bonnet.
 * - Tyres on a road are a broad hiss round 1 kHz that never quite holds still.
 * - A town from above is traffic too far off to tell apart: a rumble, and the swell of a car now and then.
 * - Leaves are thousands of tiny taps: bright noise that flutters, each side its own.
 */

import { nodeFilter } from './mono';
import { Biquad, fold, Pink, random, setLevel, track, wander, type Stereo } from './water';

/** The outdoors' loops, by name: one channel for what has a place (a car), two for what is all round. */
export const OUTDOORS = {
  carEngine: carEngineSound,
  carTyres: carTyresSound,
  town: townSound,
  leaves: leavesSound,
} satisfies Record<string, (rate: number) => Stereo | Float32Array>;

export type OutdoorKind = keyof typeof OUTDOORS;

/** The average level each is made to (as the water's and the things': a plain gain on it is its level in the game). */
const RMS = 0.2;

/** One channel of a loop made on both: the left, levelled. */
function mono(s: Stereo): Float32Array {
  return setLevel({ left: s.left, right: s.left.slice() }, RMS).left;
}

/** A car's engine at a steady pull (4 s: 220 turns of its firing, so its hum meets itself at the seam). */
function carEngineSound(rate: number): Float32Array {
  const r = random(0xca51);
  const seconds = 4;
  const firing = 55;
  // The exhaust's rasp: low noise (its end run on into its start), chopped at the firing's pace below.
  const noise = track(rate, seconds, 0.5);
  const rasp = new Biquad(rate).set('bandpass', 260, 0.7);
  const pink = new Pink(r);
  for (let i = 0; i < noise.left.length; i++) noise.left[i] = rasp.run(pink.next());
  const d = fold(noise).left;
  // The overtones, each a little weaker than the last, each wavering at its own slow pace (whole turns a loop).
  const parts = Array.from({ length: 12 }, (_, k) => ({ k: k + 1, amp: (k % 2 ? 0.7 : 1) / (k + 1) ** 1.15, wobble: 1 + Math.floor(r() * 5), depth: 0.1 + 0.2 * r(), phase: r() * 2 * Math.PI }));
  for (let i = 0; i < d.length; i++) {
    const t = i / rate;
    let v = 0;
    for (const p of parts) v += p.amp * (1 + p.depth * Math.sin((2 * Math.PI * p.wobble * t) / seconds + p.phase)) * Math.sin(2 * Math.PI * firing * p.k * t + p.phase * p.k);
    const beat = 0.5 + 0.5 * Math.sin(2 * Math.PI * firing * t);
    d[i] = v + 1.6 * d[i] * (0.4 + 0.6 * beat * beat);
  }
  // From under the bonnet: the highs stay in (the filter run round the loop, so the seam stays whole).
  const hum = nodeFilter(d, rate, 'lowpass', 900, -3.01, true);
  return setLevel({ left: hum, right: hum.slice() }, RMS).left;
}

/** A car's tyres on the road at a steady speed. */
function carTyresSound(rate: number): Float32Array {
  const r = random(0x7e5);
  const out = track(rate, 4, 0.5);
  const n = out.left.length;
  const drift = wander(r, n, rate, 0.7);
  const pink = new Pink(r);
  const body = new Biquad(rate).set('bandpass', 1000, 0.5);
  const hiss = new Biquad(rate).set('highpass', 2800, 0.6);
  const top = new Biquad(rate).set('lowpass', 6000, 0.7);
  for (let i = 0; i < n; i++) {
    if ((i & 63) === 0) body.set('bandpass', 1000 * 2 ** (0.12 * drift[i]), 0.5);
    const w = pink.next();
    out.left[i] = (body.run(w) + 0.25 * top.run(hiss.run(w))) * (1 + 0.1 * drift[i]);
  }
  return mono(fold(out));
}

/** The town from above: the rumble of its streets, and a car's swell now and then, each side its own. */
function townSound(rate: number): Stereo {
  const r = random(0x70c4);
  const out = track(rate, 16, 1.5);
  const n = out.left.length;
  for (const ch of [out.left, out.right]) {
    const swell = wander(r, n, rate, 2.5);
    const far = wander(r, n, rate, 6);
    const pink = new Pink(r);
    const pass = new Pink(r);
    const rumble = new Biquad(rate).set('lowpass', 220, 0.6);
    const road = new Biquad(rate).set('bandpass', 520, 0.45);
    const top = new Biquad(rate).set('lowpass', 1400, 0.7);
    for (let i = 0; i < n; i++) {
      const car = Math.max(0, swell[i]) ** 2;
      ch[i] = rumble.run(pink.next()) * (1.6 + 0.3 * far[i]) + top.run(road.run(pass.next())) * (0.35 + 0.9 * car);
    }
  }
  return setLevel(fold(out), RMS);
}

/** The wind in the leaves and the grass, at a full gust: bright noise that flutters, each side its own. */
function leavesSound(rate: number): Stereo {
  const r = random(0x1eaf);
  const out = track(rate, 10, 1);
  const n = out.left.length;
  for (const ch of [out.left, out.right]) {
    // The leaves' flutter: fast, and the boughs' slower heave under it.
    const flutter = wander(r, n, rate, 0.05);
    const heave = wander(r, n, rate, 0.6);
    const pink = new Pink(r);
    const leaves = new Biquad(rate).set('bandpass', 5200, 0.5);
    const grass = new Biquad(rate).set('bandpass', 1500, 0.6);
    const air = new Biquad(rate).set('lowpass', 380, 0.6);
    const low = new Pink(r);
    for (let i = 0; i < n; i++) {
      const w = pink.next();
      const f = 0.55 + 0.45 * Math.tanh(flutter[i]);
      const h = 1 + 0.25 * Math.tanh(heave[i]);
      ch[i] = h * (leaves.run(w) * 2.2 * f + grass.run(w) * 0.5 * (0.6 + 0.4 * f)) + air.run(low.next()) * 0.5;
    }
  }
  return setLevel(fold(out), RMS);
}
