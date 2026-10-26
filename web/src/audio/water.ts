/**
 * Water for the sounds of the world (`world.ts`), synthesised (no sound files): a shower, a tap
 * running into a basin, washing up, a bath filling and someone soaking in it, a washing machine,
 * a fountain and a toilet's flush; and, made from the same parts, a tumble dryer and a dishwasher
 * being loaded.
 *
 * Each is a stereo loop several seconds long, made once and played on repeat, so it costs nothing
 * while playing. Running water is mostly two things, and both are built here:
 * - the jet's rush: pink noise in a broad band (most of it between about 500 Hz and 8 kHz), its
 *   centre and level wandering slowly as the flow and whatever it lands on change;
 * - drops and bubbles: thousands of tiny ticks where the spray lands, and the short rising
 *   "blip" of each air bubble the stream pushes under the surface (the gurgle of a tap), modelled
 *   as a decaying sine whose pitch rises a little as it rings (van den Doel's bubble model).
 * Then the room: a short, damped reverb for a tiled stall, a basin, a bathroom.
 * The end of each loop is cross-faded into its start, so the seam can't be heard (the flush is
 * played once: `ONCE`). They take a moment to make, so they're made off the main thread
 * (`water.worker.ts`).
 */

/** Two channels of samples. */
export interface Stereo {
  left: Float32Array;
  right: Float32Array;
}

/** The water sounds, by name. */
export const WATER = {
  shower: showerSound,
  tap: tapSound,
  washUp: washUpSound,
  bathFill: bathFillSound,
  bathSoak: bathSoakSound,
  washer: washerSound,
  dryer: dryerSound,
  crockery: crockerySound,
  fountain: fountainSound,
  flush: flushSound,
} satisfies Record<string, (rate: number) => Stereo>;

export type WaterKind = keyof typeof WATER;

/** The sounds played once as something happens (a flush), not looped while it lasts. */
export const ONCE = ['flush'] as const satisfies readonly WaterKind[];

/** A shower: spray on the tiles and on someone standing in it, in a small tiled stall. */
export function showerSound(rate: number): Stereo {
  const r = random(11);
  const out = track(rate, 9);
  const n = out.left.length;
  // Someone moving under the spray: more of it lands on them (duller, a little fuller) or on the tiles (brighter).
  const body = wander(r, n, rate, 0.9);
  // The flow itself, never quite steady.
  const flow = wander(r, n, rate, 0.12);
  const shared = new Pink(r);
  for (const [c, ch] of [out.left, out.right].entries()) {
    const own = new Pink(r);
    const spray = new Biquad(rate);
    const spray2 = new Biquad(rate);
    const fizz = new Biquad(rate).set('highpass', 4500, 0.6);
    const fizz2 = new Biquad(rate).set('lowpass', 10000, 0.6);
    const tray = new Biquad(rate).set('lowpass', 320, 0.7);
    const patter = new Biquad(rate).set('bandpass', 3800, 0.6);
    const floor = new Biquad(rate).set('highpass', 170, 0.6);
    const sh = new Pink(r);
    let drop = 0;
    for (let i = 0; i < n; i++) {
      const b = 0.5 + 0.5 * Math.tanh(body[i]);
      if ((i & 31) === 0) {
        const centre = 3400 * 2 ** (-0.7 * b + 0.08 * flow[i]);
        spray.set('bandpass', centre, 0.55);
        spray2.set('bandpass', centre * 0.45, 0.9);
      }
      const w = 0.55 * shared.next() + 0.8 * own.next();
      const level = 1 + 0.1 * flow[i];
      // Drops landing: a dense run of tiny ticks of all sizes.
      drop = r() < 1400 / rate ? (r() - 0.5) * 6 * r() ** 3 : 0;
      ch[i] = floor.run(level * (spray.run(w) * 1.2 + spray2.run(w) * (0.25 + 0.35 * b) + (1 - 0.4 * b) * 0.6 * fizz2.run(fizz.run(w))) + tray.run(sh.next()) * (0.12 + 0.2 * b) + patter.run(drop) * 0.8);
    }
    // Now and then a few bigger drops ringing in the water on the floor.
    for (let t = r() * 0.05; t < out.seconds; t += -Math.log(1 - r()) / 45) bubble(ch, rate, t, 1400 * 4 ** r(), 0.15 * (0.2 + r() ** 2), 0.1 + r() * 0.3);
    room(ch, rate, c ? [5.3, 7.9, 10.7, 13.1] : [4.7, 8.3, 11.3, 12.7], 0.62, 0.35, 0.3);
  }
  return finish(out, 0.2);
}

/** A tap running into a basin: a narrower, gurgling stream. */
export function tapSound(rate: number): Stereo {
  const r = random(12);
  const out = track(rate, 7);
  stream(out, r, { centre: 1500, q: 0.9, bubbles: 240, gurgle: 0.15, low: 450, high: 2600, rumble: 0.15 });
  for (const [c, ch] of [out.left, out.right].entries()) room(ch, rate, c ? [2.3, 3.7, 5.1, 6.7] : [2.1, 3.3, 4.9, 6.1], 0.5, 0.4, 0.18);
  return finish(out, 0.2);
}

/** A bath filling: a fuller stream plunging into deep water, in a bathroom. */
export function bathFillSound(rate: number): Stereo {
  const r = random(13);
  const out = track(rate, 8);
  stream(out, r, { centre: 950, q: 0.7, bubbles: 480, gurgle: 0.12, low: 180, high: 1400, rumble: 0.5 });
  for (const [c, ch] of [out.left, out.right].entries()) room(ch, rate, c ? [11.3, 17.9, 23.1, 29.3] : [12.7, 16.1, 24.7, 27.9], 0.7, 0.45, 0.28);
  return finish(out, 0.2);
}

/** Someone soaking in the bath: the water lapping as they move, a drip from the tap now and then. */
export function bathSoakSound(rate: number): Stereo {
  const r = random(14);
  const out = track(rate, 13);
  for (let t = 0.4 + r(); t < out.seconds; t += 1.6 + r() * 3) slosh(out, rate, r, t, 0.5 + r() * 0.9, 0.5 + r() * 0.5, (r() - 0.5) * 0.8, 500 + r() * 600);
  for (let t = r() * 2; t < out.seconds; t += 0.9 + r() * 2.6) {
    const pan = 0.3 + (r() - 0.5) * 0.3;
    const [l, rr] = gains(pan);
    const amp = 0.25 + 0.35 * r();
    // A tiny click as it lands, then the plink of the bubble it traps.
    tick(out.left, rate, t, 0.08 * amp * l);
    tick(out.right, rate, t, 0.08 * amp * rr);
    const f = 900 + 1300 * r();
    bubble(out.left, rate, t + 0.002, f, amp * l, 0.5 + r() * 0.6);
    bubble(out.right, rate, t + 0.002, f, amp * rr, 0.5 + r() * 0.6);
  }
  for (const [c, ch] of [out.left, out.right].entries()) room(ch, rate, c ? [11.3, 17.9, 23.1, 29.3] : [12.7, 16.1, 24.7, 27.9], 0.72, 0.45, 0.35);
  return finish(out, 0, 0.7);
}

/** Washing up: the tap turned on and off, water swished in the bowl, plates and cutlery clinking. */
export function washUpSound(rate: number): Stereo {
  const r = random(15);
  const out = track(rate, 14);
  stream(out, r, { centre: 1700, q: 0.8, bubbles: 200, gurgle: 0.12, low: 500, high: 2800, rumble: 0.12 });
  // The tap on for a few seconds, then off while things are scrubbed.
  const on = new Float32Array(out.left.length);
  let t = 0;
  while (t < out.seconds) {
    const len = 2.2 + r() * 3;
    const gap = 1 + r() * 2.2;
    const a = Math.round(t * rate);
    const b = Math.round((t + len) * rate);
    for (let i = a; i < Math.min(on.length, b + Math.round(0.25 * rate)); i++) {
      const up = (i - a) / (0.08 * rate);
      const down = 1 - (i - b) / (0.18 * rate);
      on[i] = Math.max(0, Math.min(1, up, i < b ? 1 : down));
    }
    t += len + gap;
  }
  for (const ch of [out.left, out.right]) for (let i = 0; i < ch.length; i++) ch[i] *= on[i] * 0.8;
  // Swishing a plate through the water.
  for (let t = 0.3 + r(); t < out.seconds; t += 0.6 + r() * 1.4) slosh(out, rate, r, t, 0.18 + r() * 0.35, 0.4 + r() * 0.3, (r() - 0.5) * 0.5, 700 + r() * 900);
  // Crockery: clinks in little runs, some of them under the water, and the odd rattle of cutlery.
  for (let t = 0.2 + r() * 0.5; t < out.seconds; t += 0.35 + r() * 1.3) {
    const pan = (r() - 0.5) * 0.6;
    if (r() < 0.18) {
      const f = 3200 + 1800 * r();
      for (let k = 0, at = t; k < 2 + Math.floor(r() * 3); k++, at += 0.03 + r() * 0.06) clink(out, rate, at, f * (0.95 + r() * 0.1), 0.18 + 0.12 * r(), pan, [1, 1.53, 2.37, 3.11], 0.18, false);
      continue;
    }
    const f = 1300 + 1500 * r();
    const under = r() < 0.3;
    clink(out, rate, t, f, under ? 0.35 : 0.25 + 0.5 * r() ** 2, pan, [1, 2.76, 5.4, 8.93], 0.08, under);
    if (r() < 0.4) clink(out, rate, t + 0.07 + r() * 0.15, f * (0.9 + r() * 0.25), 0.15 + 0.3 * r(), pan, [1, 2.76, 5.4, 8.93], 0.08, under);
  }
  for (const [c, ch] of [out.left, out.right].entries()) room(ch, rate, c ? [3.1, 5.3, 7.1, 9.7] : [2.9, 4.7, 7.9, 9.1], 0.55, 0.4, 0.2);
  return finish(out, 0.17);
}

/** A washing machine: the drum turning one way then the other, the washing lifted and dropped back into the water, the motor's hum. */
export function washerSound(rate: number): Stereo {
  const r = random(16);
  const out = track(rate, 16);
  const n = out.left.length;
  // Turning between these times (seconds); paused for a moment in between to change direction.
  const turns: [number, number][] = [
    [0.8, 7.3],
    [8.8, 15.3],
    [16.8, 23.3],
  ];
  const turning = new Float32Array(n);
  for (const [a, b] of turns) for (let i = Math.round(a * rate); i < Math.min(n, b * rate); i++) turning[i] = Math.min(1, (i / rate - a) / 0.5, (b - i / rate) / 0.5);
  const hum = new Biquad(rate).set('lowpass', 500, 0.7);
  const swish = new Biquad(rate).set('lowpass', 700, 0.6);
  const swishHp = new Biquad(rate).set('highpass', 200, 0.6);
  const wobble = wander(r, n, rate, 0.3);
  let phase = 0;
  const p = new Pink(r);
  for (let i = 0; i < n; i++) {
    const s = turning[i];
    // A soft, buzzy hum that rises with the motor's speed.
    phase += (2 * Math.PI * 92 * (0.6 + 0.4 * s) * (1 + 0.01 * wobble[i])) / rate;
    const saw = Math.sin(phase) + 0.5 * Math.sin(2 * phase) + 0.3 * Math.sin(3 * phase) + 0.15 * Math.sin(5 * phase);
    const v = hum.run(saw) * 0.025 * s + swishHp.run(swish.run(p.next())) * 0.8 * (0.3 + 0.7 * s);
    out.left[i] = v;
    out.right[i] = v;
  }
  // Each time round, the washing falls back into the water: a slosh and a soft thud.
  for (const [a, b] of turns) {
    for (let t = a + 0.4 + r() * 0.3; t < b - 0.3; t += 1.15 + r() * 0.2) {
      slosh(out, rate, r, t, 0.55 + r() * 0.2, 0.8 + 0.4 * r(), (r() - 0.5) * 0.2, 800 + r() * 700);
      thud(out, rate, t + 0.06, 0.1 + 0.06 * r());
    }
  }
  // Inside a metal box: the highs muffled, a little ring.
  for (const [c, ch] of [out.left, out.right].entries()) {
    const lp = new Biquad(rate).set('lowpass', 3000, 0.7);
    const hp = new Biquad(rate).set('highpass', 110, 0.7);
    for (let i = 0; i < ch.length; i++) ch[i] = hp.run(lp.run(ch[i]));
    room(ch, rate, c ? [3.7, 6.1, 8.3, 10.9] : [4.1, 5.9, 8.9, 10.3], 0.55, 0.3, 0.2);
  }
  return finish(out, 0.16);
}

/** A tumble dryer: the drum turning steadily one way, the motor's hum and the fan's rush, the dry washing lifted and dropped (softer than a wash), now and then a button or a zip ticking on the drum. */
export function dryerSound(rate: number): Stereo {
  const r = random(18);
  const out = track(rate, 12);
  const n = out.left.length;
  const hum = new Biquad(rate).set('lowpass', 450, 0.7);
  const fan = new Biquad(rate).set('bandpass', 650, 0.5);
  const fanHp = new Biquad(rate).set('highpass', 180, 0.6);
  const wobble = wander(r, n, rate, 0.4);
  let phase = 0;
  const p = new Pink(r);
  for (let i = 0; i < n; i++) {
    phase += (2 * Math.PI * 100 * (1 + 0.008 * wobble[i])) / rate;
    const saw = Math.sin(phase) + 0.5 * Math.sin(2 * phase) + 0.3 * Math.sin(3 * phase) + 0.15 * Math.sin(5 * phase);
    const v = hum.run(saw) * 0.03 + fanHp.run(fan.run(p.next())) * 0.55 * (1 + 0.1 * wobble[i]);
    out.left[i] = v;
    out.right[i] = v;
  }
  // The washing tumbling over, about twice a turn: soft thuds.
  for (let t = 0.3 + r() * 0.2; t < out.seconds - 0.3; t += 0.5 + r() * 0.45) thud(out, rate, t, 0.05 + 0.05 * r());
  // A button or a zip knocking on the drum.
  for (let t = 0.8 + r(); t < out.seconds - 0.3; t += 1.2 + r() * 2.5) clink(out, rate, t, 2400 + 1600 * r(), 0.05 + 0.06 * r(), (r() - 0.5) * 0.3, [1, 2.32, 4.25, 6.63], 0.05, false);
  // Inside a metal box, as the washing machine.
  for (const [c, ch] of [out.left, out.right].entries()) {
    const lp = new Biquad(rate).set('lowpass', 3000, 0.7);
    const hp = new Biquad(rate).set('highpass', 110, 0.7);
    for (let i = 0; i < ch.length; i++) ch[i] = hp.run(lp.run(ch[i]));
    room(ch, rate, c ? [3.7, 6.1, 8.3, 10.9] : [4.1, 5.9, 8.9, 10.3], 0.55, 0.3, 0.2);
  }
  return finish(out, 0.14);
}

/** Loading a dishwasher: plates and mugs set into the rack, a handful of cutlery dropped in the basket, a rack rolled out or in; no water. */
export function crockerySound(rate: number): Stereo {
  const r = random(20);
  const out = track(rate, 12);
  for (let t = 0.3 + r() * 0.4; t < out.seconds; ) {
    const pan = (r() - 0.5) * 0.5;
    const what = r();
    if (what < 0.15) {
      // Cutlery into the basket: a quick rattle.
      const f = 3200 + 1800 * r();
      for (let k = 0, at = t; k < 3 + Math.floor(r() * 4); k++, at += 0.02 + r() * 0.05) clink(out, rate, at, f * (0.9 + r() * 0.2), 0.12 + 0.12 * r(), pan, [1, 1.53, 2.37, 3.11], 0.15, false);
      t += 0.8 + r() * 1.2;
    } else if (what < 0.25) {
      roll(out, rate, r, t, 0.45 + r() * 0.3, pan);
      t += 1 + r();
    } else {
      // A plate or a mug knocking the rack as it goes in, sometimes again as it settles.
      const f = 1300 + 1500 * r();
      clink(out, rate, t, f, 0.2 + 0.35 * r() ** 2, pan, [1, 2.76, 5.4, 8.93], 0.07, false);
      if (r() < 0.4) clink(out, rate, t + 0.06 + r() * 0.1, f * (0.9 + r() * 0.2), 0.1 + 0.2 * r(), pan, [1, 2.76, 5.4, 8.93], 0.07, false);
      t += 0.7 + r() * 1.3;
    }
  }
  for (const [c, ch] of [out.left, out.right].entries()) room(ch, rate, c ? [3.1, 5.3, 7.1, 9.7] : [2.9, 4.7, 7.9, 9.1], 0.55, 0.4, 0.2);
  return finish(out, 0, 0.7);
}

/** A fountain heard from beside it: the jet falling back into the basin as a broad, splashing rush, drops pattering all over the water; out of doors, so no echo. */
export function fountainSound(rate: number): Stereo {
  const r = random(17);
  const out = track(rate, 10);
  stream(out, r, { centre: 1100, q: 0.45, bubbles: 600, gurgle: 0.1, low: 500, high: 3500, rumble: 0.3 });
  // Drops landing all over the basin, each somewhere of its own left to right.
  for (let t = r() * 0.01; t < out.seconds; t += -Math.log(1 - r()) / 500) {
    const [l, rr] = gains((r() - 0.5) * 1.4);
    const amp = 0.06 * (0.2 + r() ** 3);
    const f = 1000 * 4 ** r();
    tick(out.left, rate, t, amp * 0.5 * l);
    tick(out.right, rate, t, amp * 0.5 * rr);
    bubble(out.left, rate, t + 0.001, f, amp * l, 0.2 + r() * 0.4);
    bubble(out.right, rate, t + 0.001, f, amp * rr, 0.2 + r() * 0.4);
  }
  return finish(out, 0.18);
}

/**
 * A toilet flushing, played once: the lever's clunk, the cistern emptying into the bowl in a rush
 * that swirls away, the gulp as the trap empties, then the cistern filling again with a thin hiss
 * and a trickle into the bowl until the valve shuts.
 */
export function flushSound(rate: number): Stereo {
  const r = random(19);
  const out = track(rate, 9, 0);
  const n = out.left.length;
  const swirl = wander(r, n, rate, 0.25);
  const burble = wander(r, n, rate, 0.04);
  const shared = new Pink(r);
  // How the rush goes: on at once, falling away as the cistern empties; and the refill after it.
  const rush = (t: number) => (t < 0.15 ? 0 : Math.min(1, (t - 0.15) / 0.12) * Math.exp(-Math.max(0, t - 0.9) / 1.1));
  const refill = (t: number) => ramp(2.2, 3.2, t) * (1 - ramp(7.4, 8.6, t));
  for (const [c, ch] of [out.left, out.right].entries()) {
    const own = new Pink(r);
    const jet = new Biquad(rate);
    const low = new Biquad(rate).set('lowpass', 300, 0.8);
    const hiss = new Biquad(rate).set('bandpass', 3400, 3);
    const hp = new Biquad(rate).set('highpass', 110, 0.7);
    const deep = new Pink(r);
    for (let i = 0; i < n; i++) {
      const t = i / rate;
      if ((i & 31) === 0) jet.set('bandpass', 900 * 2 ** (0.5 * swirl[i] + 0.25 * burble[i]), 0.7);
      const w = 0.6 * shared.next() + 0.75 * own.next();
      const a = rush(t);
      ch[i] = hp.run(jet.run(w) * 1.6 * a * (1 + 0.3 * burble[i]) + low.run(deep.next()) * 0.9 * a + hiss.run(w) * 0.5 * refill(t));
    }
    // Bubbles in the swirl...
    for (let t = 0.3 + r() * 0.01; t < 3.2; t += -Math.log(1 - r()) / 260) bubble(ch, rate, t, 300 * (2600 / 300) ** r() ** 0.8, 0.18 * rush(t) * (0.2 + r() ** 2), 0.1 + r() * 0.3);
    // ...the gulp as the trap empties: a run of big, low bubbles...
    for (let k = 0, t = 2.3 + r() * 0.1; k < 7; k++, t += 0.07 + r() * 0.08) bubble(ch, rate, t, 160 + 260 * r(), 0.5 * (1 - k / 9), 0.4 + r() * 0.4);
    // ...and the trickle back into the bowl as the cistern fills.
    for (let t = 3; t < 8.4; t += -Math.log(1 - r()) / 30) bubble(ch, rate, t, 900 + 1500 * r(), 0.05 * refill(t), 0.3);
    room(ch, rate, c ? [11.3, 17.9, 23.1, 29.3] : [12.7, 16.1, 24.7, 27.9], 0.62, 0.45, 0.25);
  }
  // The lever: a plastic clunk.
  thud(out, rate, 0.02, 0.3);
  tick(out.left, rate, 0.02, 0.25);
  tick(out.right, rate, 0.021, 0.25);
  return finish(out, 0, 0.8);
}

// Building blocks.

export interface Track extends Stereo {
  rate: number;
  /** Seconds in the loop, and seconds after it made to fade into its start. */
  seconds: number;
  overlap: number;
}

/** Room for a loop of `seconds`, plus a little more to cross-fade into its start (none for a sound played once). */
export function track(rate: number, seconds: number, overlap = 0.6): Track {
  const n = Math.round((seconds + overlap) * rate);
  return { left: new Float32Array(n), right: new Float32Array(n), rate, seconds: seconds + overlap, overlap };
}

/**
 * Water running into water: the jet's rush (pink noise in a band, its centre and level burbling)
 * with a faint fizz above, bubbles in a range of sizes, and a low rumble in the water below.
 */
export function stream(out: Track, r: () => number, o: { centre: number; q: number; bubbles: number; gurgle: number; low: number; high: number; rumble: number }): void {
  const { rate } = out;
  const n = out.left.length;
  // The burble: fast, uneven wobbles in the stream.
  const burble = wander(r, n, rate, 0.035);
  const slow = wander(r, n, rate, 0.7);
  const shared = new Pink(r);
  for (const [c, ch] of [out.left, out.right].entries()) {
    const own = new Pink(r);
    const jet = new Biquad(rate);
    const fizz = new Biquad(rate).set('bandpass', 4200, 0.8);
    const rumble = new Biquad(rate).set('lowpass', 260, 0.8);
    const deep = new Pink(r);
    for (let i = 0; i < n; i++) {
      const g = burble[i] * 0.7 + slow[i] * 0.3;
      if ((i & 31) === 0) jet.set('bandpass', o.centre * 2 ** (0.35 * g), o.q);
      const w = 0.6 * shared.next() + 0.75 * own.next();
      ch[i] = jet.run(w) * (1 + 0.3 * g) * 1.3 + fizz.run(w) * 0.12 + rumble.run(deep.next()) * o.rumble;
    }
    // Bubbles: more of the small, high ones, bunched where the stream burbles.
    const pan = c ? 1 : -1;
    for (let t = r() * 0.01; t < out.seconds; t += -Math.log(1 - r()) / o.bubbles) {
      const i = Math.min(n - 1, Math.round(t * rate));
      const f = o.low * (o.high / o.low) ** r() ** 0.7;
      const amp = o.gurgle * (0.15 + r() ** 2) * Math.max(0.2, 1 + 0.6 * burble[i]) * (1 + 0.2 * pan * (r() - 0.5));
      bubble(ch, rate, t, f, amp, 0.05 + r() * 0.25);
    }
  }
}

/** One air bubble ringing: a sine at `f0` (Hz) dying away quickly, its pitch rising by `rise` as it goes. */
export function bubble(d: Float32Array, rate: number, at: number, f0: number, amp: number, rise: number): void {
  const damp = 0.043 * f0 + 0.0014 * f0 ** 1.5;
  const start = Math.round(at * rate);
  const len = Math.min(d.length - start, Math.round((4.5 / damp) * rate));
  // A turning pointer (its height is the sine), re-tuned every few samples: much cheaper than sin and exp per sample.
  const k = Math.exp(-damp / rate);
  let env = amp;
  let re = 1;
  let im = 0;
  for (let j0 = 0; j0 < len; j0 += 32) {
    const w = (2 * Math.PI * f0 * (1 + (rise * damp * (j0 + 16)) / rate)) / rate;
    const c = Math.cos(w);
    const s = Math.sin(w);
    for (let j = j0; j < Math.min(len, j0 + 32); j++) {
      const x = re * c - im * s;
      im = re * s + im * c;
      re = x;
      d[start + j] += env * im;
      env *= k;
    }
  }
}

/** A very short click. */
export function tick(d: Float32Array, rate: number, at: number, amp: number): void {
  const start = Math.round(at * rate);
  const len = Math.min(d.length - start, Math.round(0.0015 * rate));
  let s = (start * 2654435761) >>> 0;
  for (let j = 0; j < len; j++) {
    s = (s * 1664525 + 1013904223) >>> 0;
    d[start + j] += amp * (s / 2147483648 - 1) * (1 - j / len);
  }
}

/** Water lapping or swished about: a swell of low, filtered noise with a few big bubbles in it. */
export function slosh(out: Track, rate: number, r: () => number, at: number, seconds: number, amp: number, pan: number, cutoff: number): void {
  const start = Math.round(at * rate);
  const len = Math.min(out.left.length - start, Math.round(seconds * 1.6 * rate));
  if (len <= 0) return;
  const [l, rr] = gains(pan);
  const lp = new Biquad(rate).set('lowpass', cutoff, 0.9);
  const hp = new Biquad(rate).set('highpass', 140, 0.7);
  const shape = wander(r, len, rate, 0.04);
  for (let j = 0; j < len; j++) {
    const t = j / rate / seconds;
    const env = t < 0.3 ? Math.sin(((t / 0.3) * Math.PI) / 2) ** 2 : Math.exp(-(t - 0.3) * 3.2);
    const v = hp.run(lp.run(r() * 2 - 1)) * amp * 1.4 * env * (1 + 0.5 * shape[j]);
    out.left[start + j] += v * l;
    out.right[start + j] += v * rr;
  }
  const count = Math.round(seconds * (12 + r() * 20));
  for (let k = 0; k < count; k++) {
    const t = at + seconds * 0.6 * r() ** 1.5;
    const f = 220 + 700 * r() ** 1.5;
    const a = 0.05 * amp * (0.3 + r());
    bubble(out.left, rate, t, f, a * l, 0.1 + r() * 0.3);
    bubble(out.right, rate, t, f, a * rr, 0.1 + r() * 0.3);
  }
}

/** A dish or a fork knocking another: a few inharmonic partials ringing out after a click; under water, duller and shorter. */
export function clink(out: Track, rate: number, at: number, f0: number, amp: number, pan: number, ratios: number[], ring: number, under: boolean): void {
  const start = Math.round(at * rate);
  const [l, rr] = gains(pan);
  let s = (start * 40503 + 7) >>> 0;
  const rnd = () => (s = (s * 1664525 + 1013904223) >>> 0) / 4294967296;
  const partials = ratios
    .map((k, i) => ({ f: f0 * k * (0.97 + 0.06 * rnd()), a: (under ? 0.6 : 1) / (1 + i * (under ? 3 : 0.9)), tau: (ring * (under ? 0.35 : 1)) / (1 + i * 0.8), phase: rnd() * 6.28 }))
    .filter((p) => p.f < (under ? 2500 : 16000));
  const len = Math.min(out.left.length - start, Math.round(ring * 6 * rate));
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    let v = 0;
    for (const p of partials) v += p.a * Math.exp(-t / p.tau) * Math.sin(2 * Math.PI * p.f * t + p.phase);
    // The knock itself, before the ring.
    if (j < 0.002 * rate && !under) v += (rnd() * 2 - 1) * 0.6 * (1 - j / (0.002 * rate));
    v *= amp * Math.min(1, j / (0.0004 * rate));
    out.left[start + j] += v * l;
    out.right[start + j] += v * rr;
  }
}

/** A soft, low thud (wet washing landing in the drum). */
export function thud(out: Track, rate: number, at: number, amp: number): void {
  const start = Math.round(at * rate);
  const len = Math.min(out.left.length - start, Math.round(0.25 * rate));
  let phase = 0;
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    phase += (2 * Math.PI * (70 - 25 * Math.min(1, t / 0.1))) / rate;
    const v = amp * Math.sin(phase) * Math.exp(-t / 0.06) * Math.min(1, t / 0.006);
    out.left[start + j] += v;
    out.right[start + j] += v;
  }
}

/** A dishwasher's rack rolled on its rails: a low rumble with the wheels ticking over the joins, then the knock as it stops. */
function roll(out: Track, rate: number, r: () => number, at: number, seconds: number, pan: number): void {
  const start = Math.round(at * rate);
  const len = Math.min(out.left.length - start, Math.round(seconds * rate));
  if (len <= 0) return;
  const [l, rr] = gains(pan);
  const band = new Biquad(rate).set('bandpass', 420, 1.2);
  for (let j = 0; j < len; j++) {
    const v = band.run(r() * 2 - 1) * 0.5 * Math.sin((Math.PI * j) / len) ** 0.5;
    out.left[start + j] += v * l;
    out.right[start + j] += v * rr;
  }
  for (let t = at + 0.02; t < at + seconds; t += 0.03 + r() * 0.03) {
    tick(out.left, rate, t, 0.08 * l);
    tick(out.right, rate, t, 0.08 * rr);
  }
  clink(out, rate, at + seconds, 900 + 300 * r(), 0.25, pan, [1, 2.76, 5.4], 0.05, false);
}

/** 0 before `a`, 1 after `b`, a smooth step between. */
export function ramp(a: number, b: number, x: number): number {
  const u = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return u * u * (3 - 2 * u);
}

/** Left and right levels for `pan` from -1 (left) to 1 (right), keeping the loudness. */
export function gains(pan: number): [number, number] {
  const a = ((Math.max(-1, Math.min(1, pan)) + 1) * Math.PI) / 4;
  return [Math.cos(a), Math.sin(a)];
}

/**
 * A small room: four damped echoes going round (delays in ms) and a diffuser, mixed in at `wet`.
 * Different delays per channel make it wide.
 */
export function room(d: Float32Array, rate: number, delays: number[], feedback: number, damping: number, wet: number): void {
  const combs = delays.map((ms) => ({ buf: new Float32Array(Math.round((ms / 1000) * rate)), i: 0, lp: 0 }));
  const passes = [5.1, 1.7].map((ms) => ({ buf: new Float32Array(Math.round((ms / 1000) * rate)), i: 0 }));
  for (let j = 0; j < d.length; j++) {
    const x = d[j];
    let y = 0;
    for (const c of combs) {
      const v = c.buf[c.i];
      c.lp = v * (1 - damping) + c.lp * damping;
      c.buf[c.i] = x + c.lp * feedback;
      c.i = (c.i + 1) % c.buf.length;
      y += v;
    }
    y /= combs.length;
    for (const p of passes) {
      const v = p.buf[p.i];
      const u = y + v * 0.5;
      p.buf[p.i] = u;
      p.i = (p.i + 1) % p.buf.length;
      y = v - u * 0.5;
    }
    d[j] = x * (1 - wet * 0.5) + y * wet;
  }
}

/**
 * Folds the overlap at the end into the start (an equal-power cross-fade, so the loop runs on without
 * a seam or a dip), cuts the overlap off, then sets the level: to an average of `rms`, or with 0 to a
 * peak of `peak`; any peaks over 0.7 are rounded off.
 */
export function finish(t: Track, rms: number, peak = 1): Stereo {
  return setLevel(fold(t), rms, peak);
}

/** The loop's overlap folded into its start and cut off (`finish` without the level). */
export function fold(t: Track): Stereo {
  const f = Math.round(t.overlap * t.rate);
  const n = t.left.length - f;
  const result: Stereo = { left: new Float32Array(n), right: new Float32Array(n) };
  for (const [src, dst] of [
    [t.left, result.left],
    [t.right, result.right],
  ]) {
    dst.set(src.subarray(0, n));
    for (let i = 0; i < f; i++) {
      const x = i / f;
      dst[i] = src[i] * Math.sin((x * Math.PI) / 2) + src[n + i] * Math.cos((x * Math.PI) / 2);
    }
  }
  return result;
}

/** Sets the level, as `finish`. */
export function setLevel(result: Stereo, rms: number, peak = 1): Stereo {
  const n = result.left.length;
  let sum = 0;
  let top = 0;
  for (const d of [result.left, result.right]) {
    for (let i = 0; i < n; i++) {
      sum += d[i] * d[i];
      top = Math.max(top, Math.abs(d[i]));
    }
  }
  const scale = rms > 0 ? rms / Math.sqrt(sum / (2 * n) || 1) : peak / (top || 1);
  for (const d of [result.left, result.right]) {
    for (let i = 0; i < n; i++) {
      const x = d[i] * scale;
      const a = Math.abs(x);
      d[i] = a <= 0.7 ? x : Math.sign(x) * (0.7 + 0.3 * Math.tanh((a - 0.7) / 0.3));
    }
  }
  return result;
}

/** Slow random wandering around 0 (about ±1), changing over roughly `tau` seconds. */
export function wander(r: () => number, n: number, rate: number, tau: number): Float32Array {
  const out = new Float32Array(n);
  const k = 1 / (tau * rate);
  const step = Math.sqrt(2 * k) * Math.sqrt(3);
  const a = Math.min(1, 4 * k);
  let x = 0;
  let y = 0;
  for (let i = 0; i < n; i++) {
    x += -x * k + step * (r() * 2 - 1);
    y += (x - y) * a;
    out[i] = y;
  }
  return out;
}

/** Pink noise (equal energy per octave, like most of nature's noises), about ±1. */
export class Pink {
  private b0 = 0;
  private b1 = 0;
  private b2 = 0;

  constructor(private readonly r: () => number) {}

  next(): number {
    const w = this.r() * 2 - 1;
    this.b0 = 0.99765 * this.b0 + w * 0.099046;
    this.b1 = 0.963 * this.b1 + w * 0.2965164;
    this.b2 = 0.57 * this.b2 + w * 1.0526913;
    return (this.b0 + this.b1 + this.b2 + w * 0.1848) * 0.25;
  }
}

/** A two-pole filter (the Web Audio `BiquadFilterNode`'s formulas), whose settings can change as it runs. */
export class Biquad {
  private b0 = 1;
  private b1 = 0;
  private b2 = 0;
  private a1 = 0;
  private a2 = 0;
  private x1 = 0;
  private x2 = 0;
  private y1 = 0;
  private y2 = 0;

  constructor(private readonly rate: number) {}

  set(type: 'lowpass' | 'highpass' | 'bandpass', frequency: number, q: number): this {
    const w = (2 * Math.PI * Math.min(frequency, this.rate * 0.45)) / this.rate;
    const cos = Math.cos(w);
    const alpha = Math.sin(w) / (2 * q);
    const a0 = 1 + alpha;
    if (type === 'bandpass') {
      this.b0 = alpha / a0;
      this.b1 = 0;
      this.b2 = -alpha / a0;
    } else {
      const k = type === 'lowpass' ? 1 - cos : 1 + cos;
      this.b0 = this.b2 = k / 2 / a0;
      this.b1 = (type === 'lowpass' ? k : -k) / a0;
    }
    this.a1 = (-2 * cos) / a0;
    this.a2 = (1 - alpha) / a0;
    return this;
  }

  run(x: number): number {
    const y = this.b0 * x + this.b1 * this.x1 + this.b2 * this.x2 - this.a1 * this.y1 - this.a2 * this.y2;
    this.x2 = this.x1;
    this.x1 = x;
    this.y2 = this.y1;
    this.y1 = y;
    return y;
  }
}

/** A small seeded generator, so the sounds are the same every time. */
export function random(seed: number): () => number {
  let s = seed >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    return (s >>> 0) / 4294967296;
  };
}
