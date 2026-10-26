/**
 * Things in use for the sounds of the world (`world.ts`), synthesised like the water (`water.ts`,
 * whose building blocks they share; no sound files):
 * - machines: a treadmill, the bikes, a sewing machine, a pottery wheel, coffee machines, a blender
 *   and a stand mixer, a microwave, a toaster, an oven's fan, a cooker hood;
 * - games: pool, foosball, ping-pong, darts, an arcade cabinet, basketball;
 * - a workbench's tools and a bike's, a fire, a chemistry set bubbling, a punch bag, crockery
 *   smashed in a bin, a cello played with a bow, someone singing to a karaoke track;
 * - played once: a fridge door, a pedal bin, the vending machines, a phone box's call and its
 *   hanging up, the microwave's ping, toast popping up, coffee poured into a cup.
 *
 * Most are made the same few ways:
 * - a motor or a fan: a tone at its speed with its harmonics (a transformer's hum at twice the
 *   mains, 100 Hz; a fan's blades passing), over the rush of air or the rumble of what it drives;
 * - something struck (`strike`): a few modes ringing out over a short burst of noise, at the
 *   frequencies of the thing's size and stuff (a basketball's air rings at about 950 Hz);
 * - events in a loop (darts, a rally) wrap round its end into its start, so nothing is cut off at
 *   the seam; steady sounds under them are cross-faded as the water's are.
 * Each is a stereo loop several seconds long (or a sound played once: `ONCE_THINGS`), made off the
 * main thread (`water.worker.ts`) the first time it's wanted.
 */

import { Biquad, bubble, fold, gains, Pink, ramp, random, room, setLevel, track, wander, WATER, type Stereo, type Track } from './water';

/** The sounds of things, by name. */
export const THINGS = {
  treadmill: treadmillSound,
  bike: bikeSound,
  tools: toolsSound,
  bikeTools: bikeToolsSound,
  pool: poolSound,
  foosball: foosballSound,
  pingPong: pingPongSound,
  darts: dartsSound,
  arcade: arcadeSound,
  sewing: sewingSound,
  potteryWheel: potteryWheelSound,
  bubbling: bubblingSound,
  espresso: espressoSound,
  blender: blenderSound,
  mixer: mixerSound,
  microwave: microwaveSound,
  toaster: toasterSound,
  ovenFan: ovenFanSound,
  hood: hoodSound,
  fire: fireSound,
  basketball: basketballSound,
  singing: singingSound,
  cello: celloSound,
  punchBag: punchBagSound,
  smash: smashSound,
  fridgeDoor: fridgeDoorSound,
  binLid: binLidSound,
  vendSnack: vendSnackSound,
  vendCan: vendCanSound,
  phoneBox: phoneBoxSound,
  hangUp: hangUpSound,
  ping: pingSound,
  toastPop: toastPopSound,
  pour: pourSound,
} satisfies Record<string, (rate: number) => Stereo>;

export type ThingKind = keyof typeof THINGS;

/** The ones played once as something happens (a use starting or ending), not looped while it lasts. */
export const ONCE_THINGS = ['fridgeDoor', 'binLid', 'vendSnack', 'vendCan', 'phoneBox', 'hangUp', 'ping', 'toastPop', 'pour'] as const satisfies readonly ThingKind[];

/** Every sound made off the main thread: the water's and the things'. */
export const MADE = { ...WATER, ...THINGS };

export type MadeKind = keyof typeof MADE;

// The machines.

/**
 * A treadmill with someone running on it: the motor's hum (about 58 Hz, turning the belt at a
 * run), the belt's hiss over the deck, the rollers' rumble, the feet landing at 165 steps a
 * minute (each a thump in the deck and a slap on the belt, one foot a little heavier) and the
 * belt's seam ticking past once a turn.
 */
function treadmillSound(rate: number): Stereo {
  const r = random(31);
  const seconds = 8;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const wobble = wander(r, n, rate, 0.5);
  const hum = motor(n, rate, (i) => 58 * (1 + 0.006 * wobble[i]), [0.9, 1, 0.5, 0.45, 0.2, 0.25, 0.1, 0.12]);
  const whine = motor(n, rate, (i) => 1240 * (1 + 0.004 * wobble[i]), [1, 0.2]);
  const hiss = noise(n, rate, r, 'bandpass', 900, 0.6);
  const rumble = noise(n, rate, r, 'lowpass', 140, 0.7);
  for (let i = 0; i < n; i++) {
    const v = hum[i] * 0.06 + whine[i] * 0.004 + hiss[i] * 0.5 + rumble[i] * 0.5;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  // 22 steps in the 8 seconds, so the rhythm runs on round the loop.
  const step = seconds / 22;
  for (let k = 0; k < 22; k++) {
    const at = k * step + (r() - 0.5) * 0.012;
    const heavy = k % 2 ? 0.85 : 1;
    strike(ev, at, 0.9 * heavy, (k % 2 ? 0.08 : -0.08), [[72, 0.045, 1], [150, 0.025, 0.4]], { hz: 900, q: 0.7, tau: 0.012, amp: 1.6 });
    // The shoe leaving the belt.
    strike(ev, at + 0.12, 0.15, 0, [], { hz: 2600, q: 0.8, tau: 0.02, amp: 1 });
  }
  for (let k = 0; k < 9; k++) strike(ev, k * (seconds / 9) + 0.05, 0.12, 0, [[2100, 0.004, 1]], { hz: 3000, q: 1, tau: 0.002, amp: 1 });
  return done(rate, [bed, ev], 0.14, 0, [5.3, 7.9, 10.7, 13.1], 0.15);
}

/**
 * An exercise bike pedalled at 80 a minute: the flywheel's whirr against its brake swelling with
 * each push of a pedal (two a turn), the drive's hum under it and a pedal creaking once a turn.
 */
function bikeSound(rate: number): Stereo {
  const r = random(32);
  const seconds = 6;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const cadence = 80 / 60;
  const whirr = noise(n, rate, r, 'bandpass', 520, 0.8);
  const air = noise(n, rate, r, 'bandpass', 1500, 0.5);
  const drive = motor(n, rate, (i) => 210 * (1 + 0.02 * Math.sin((2 * Math.PI * 2 * cadence * i) / rate)), [1, 0.4, 0.2]);
  for (let i = 0; i < n; i++) {
    const push = Math.sin((Math.PI * 2 * cadence * i) / rate) ** 2;
    const v = (whirr[i] * 0.8 + air[i] * 0.25) * (0.75 + 0.25 * push) + drive[i] * 0.012 * (0.8 + 0.2 * push);
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  // A pedal's bearing creaking at the bottom of each turn.
  for (let k = 0; k < seconds * cadence; k++) strike(ev, k / cadence + 0.3 + (r() - 0.5) * 0.02, 0.06, 0.1, [[1750, 0.02, 1], [2830, 0.012, 0.5]], { hz: 2200, q: 3, tau: 0.03, amp: 0.4 });
  return done(rate, [bed, ev], 0.14, 0, [5.3, 7.9, 10.7, 13.1], 0.15);
}

/**
 * A sewing machine: runs of stitching (the motor speeding up to about 13 stitches a second, its
 * whirr, and the needle bar and the hook clacking each stitch), pauses to turn the cloth, the
 * odd snip of scissors.
 */
function sewingSound(rate: number): Stereo {
  const r = random(33);
  const seconds = 10;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const runs: [number, number, number][] = [
    [0.3, 2.6, 13],
    [3.3, 4.4, 9],
    [5.1, 8.3, 14],
  ];
  // How fast it's going (stitches a second), with speeding up and slowing down.
  const speed = new Float32Array(n);
  for (const [a, b, top] of runs) for (let i = Math.round(a * rate); i < Math.min(n, b * rate + 0.2 * rate); i++) speed[i] = top * ramp(a, a + 0.25, i / rate) * (1 - ramp(b, b + 0.15, i / rate));
  const whirr = motor(n, rate, (i) => 5.5 * speed[i], [1, 0.8, 0.6, 0.5, 0.4, 0.35, 0.3, 0.25, 0.2, 0.18, 0.15, 0.12]);
  const brush = noise(n, rate, r, 'bandpass', 3200, 0.7);
  const lp = new Biquad(rate).set('lowpass', 2500, 0.7);
  for (let i = 0; i < n; i++) {
    const s = speed[i] / 14;
    const v = lp.run(whirr[i]) * 0.05 * s + brush[i] * 0.12 * s;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  // Each stitch: the needle bar and hook, then the take-up lever half a turn on.
  let phase = 0;
  for (let i = 0; i < n; i++) {
    const before = phase;
    phase += speed[i] / rate;
    if (Math.floor(phase) > Math.floor(before)) {
      const s = speed[i] / 14;
      strike(ev, i / rate, 0.35 * s, 0, [[2300, 0.006, 1], [3700, 0.004, 0.6], [180, 0.015, 0.4]], { hz: 2600, q: 0.8, tau: 0.002, amp: 1.2 });
    }
    if (Math.floor(phase + 0.5) > Math.floor(before + 0.5)) strike(ev, i / rate, 0.15 * (speed[i] / 14), 0, [[1600, 0.005, 1]], { hz: 1900, q: 1, tau: 0.002, amp: 0.8 });
  }
  // The cloth turned and smoothed between runs, and a snip.
  for (const at of [2.9, 4.7]) rustle(ev, r, at, 0.3, 0.05, 0.1);
  strike(ev, 8.9, 0.25, -0.1, [[5200, 0.012, 1], [7400, 0.008, 0.6]], { hz: 6000, q: 0.8, tau: 0.003, amp: 1.5 });
  strike(ev, 8.96, 0.15, -0.1, [[4800, 0.01, 1]], { hz: 5500, q: 0.8, tau: 0.002, amp: 1 });
  return done(rate, [bed, ev], 0, 0.75, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * A potter's wheel: the motor's quiet whine and the hum of its controller, the head turning
 * about 150 a minute, wet hands on the spinning clay (a swish that comes round with each turn
 * and squelches when they press), water squeezed from a sponge.
 */
function potteryWheelSound(rate: number): Stereo {
  const r = random(34);
  const seconds = 9;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const wobble = wander(r, n, rate, 0.8);
  const press = wander(r, n, rate, 0.6);
  const whine = motor(n, rate, (i) => 640 * (1 + 0.003 * wobble[i]), [1, 0.3, 0.15]);
  const mains = motor(n, rate, () => 100, [1, 0.4, 0.2]);
  const swish = noise(n, rate, r, 'bandpass', 1100, 0.6);
  const squelch = noise(n, rate, r, 'lowpass', 450, 0.9);
  const rumble = noise(n, rate, r, 'lowpass', 110, 0.7);
  const turn = 2.5;
  for (let i = 0; i < n; i++) {
    const t = i / rate;
    const p = Math.max(0, press[i]);
    const round = 0.75 + 0.25 * Math.sin(2 * Math.PI * turn * t + 0.4 * wobble[i]);
    const v = whine[i] * 0.004 + mains[i] * 0.01 + swish[i] * 0.55 * round * (0.6 + 0.4 * p) + squelch[i] * 0.9 * p * p * round + rumble[i] * 0.4;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  // The sponge squeezed over the clay: a trickle of drops.
  for (const at of [2.2, 6.4]) for (let t = at; t < at + 0.7; t += -Math.log(1 - r()) / 40) for (const ch of [bed.left, bed.right]) bubble(ch, rate, t, 700 + 1600 * r(), 0.04 * (0.3 + r()), 0.2 + r() * 0.3);
  return done(rate, [bed], 0.13, 0, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * A chemistry set: a flask boiling over a burner (small bubbles crowding up the glass, now and
 * then a big one), the burner's soft roar, a glass rod stirred against the beaker.
 */
function bubblingSound(rate: number): Stereo {
  const r = random(35);
  const seconds = 8;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const roar = noise(n, rate, r, 'lowpass', 380, 0.7);
  const hiss = noise(n, rate, r, 'bandpass', 3200, 0.5);
  for (let i = 0; i < n; i++) {
    const v = roar[i] * 0.35 + hiss[i] * 0.06;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const surge = wander(r, n, rate, 0.5);
  for (let t = r() * 0.01; t < bed.seconds; t += -Math.log(1 - r()) / 90) {
    const s = Math.max(0.2, 1 + 0.6 * surge[Math.min(n - 1, Math.round(t * rate))]);
    const [l, rr] = gains((r() - 0.5) * 0.3);
    const f = 1500 * 2.6 ** r();
    const a = 0.08 * s * (0.2 + r() ** 2);
    bubble(bed.left, rate, t, f, a * l, 0.3 + r() * 0.4);
    bubble(bed.right, rate, t, f, a * rr, 0.3 + r() * 0.4);
  }
  for (let t = 0.6 + r(); t < bed.seconds; t += 0.8 + r() * 1.6) for (const ch of [bed.left, bed.right]) bubble(ch, rate, t, 520 + 400 * r(), 0.12, 0.4);
  const ev = loop(rate, seconds);
  for (const at of [3.1, 3.5, 3.85]) strike(ev, at + r() * 0.05, 0.12, 0.2, [[3150, 0.12, 1], [7300, 0.06, 0.5], [11900, 0.03, 0.3]], { hz: 5000, q: 1, tau: 0.001, amp: 0.6 });
  return done(rate, [bed, ev], 0.12, 0, [3.1, 5.3, 7.1, 9.7], 0.12);
}

/**
 * An espresso machine making a coffee: beans ground (the grinder's whine and the crunch of beans
 * thinning out), the grounds tamped and the handle locked in, the pump's buzz (a vibration pump:
 * 50 Hz and its harmonics) with the coffee trickling into the cup, the steam wand hissing into
 * milk (a tearing hiss as it froths, then a deeper roll as it heats), and the old grounds knocked
 * out.
 */
function espressoSound(rate: number): Stereo {
  const r = random(36);
  const seconds = 16;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const grinding = (t: number) => ramp(0.2, 0.4, t) * (1 - ramp(2.6, 2.9, t));
  const pumping = (t: number) => ramp(4.6, 4.7, t) * (1 - ramp(9.5, 9.6, t));
  const steaming = (t: number) => ramp(10.5, 10.55, t) * (1 - ramp(13.4, 13.5, t));
  const grinder = motor(n, rate, (i) => 160 * (0.7 + 0.3 * grinding(i / rate)), [1, 0.7, 0.6, 0.5, 0.45, 0.4, 0.3, 0.25, 0.2, 0.2, 0.15, 0.12]);
  const pump = motor(n, rate, () => 50, [1, 0.9, 0.8, 0.7, 0.5, 0.45, 0.4, 0.3, 0.25, 0.2, 0.15, 0.1, 0.08, 0.06]);
  const pumpLp = new Biquad(rate).set('lowpass', 1800, 0.7);
  const grit = noise(n, rate, r, 'bandpass', 3000, 0.5);
  const steam = noise(n, rate, r, 'highpass', 2500, 0.6);
  const roll = noise(n, rate, r, 'bandpass', 900, 0.6);
  const trickle = noise(n, rate, r, 'bandpass', 2300, 1.2);
  const flutter = wander(r, n, rate, 0.015);
  for (let i = 0; i < n; i++) {
    const t = i / rate;
    const g = grinding(t);
    const p = pumping(t);
    const s = steaming(t);
    // The steam: a purge, then frothing (fluttering, tearing) for a second and a half, then rolling.
    const froth = ramp(10.9, 11.0, t) * (1 - ramp(12.2, 12.5, t));
    const tear = 1 + 0.6 * froth * flutter[i];
    // The pump labours as the pressure builds against the puck.
    const load = 0.8 + 0.2 * ramp(4.7, 5.6, t);
    const v = grinder[i] * 0.03 * g + grit[i] * 0.35 * g * (1 - 0.5 * ramp(0.4, 2.7, t)) + pumpLp.run(pump[i]) * 0.05 * p * load + steam[i] * 0.7 * s * tear * (1 - 0.4 * ramp(12.2, 12.5, t)) + roll[i] * 0.5 * s * ramp(12, 12.6, t) + trickle[i] * 0.08 * p * ramp(5.6, 6.2, t);
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  // Beans crunching between the burrs, fewer as they're ground.
  for (let t = 0.3; t < 2.7; t += -Math.log(1 - r()) / (400 * (1 - ramp(0.4, 2.7, t)) + 40)) strike(ev, t, 0.05 * (0.3 + r()), (r() - 0.5) * 0.2, [], { hz: 2000 + 3000 * r(), q: 1, tau: 0.0015, amp: 1 });
  // Tamped, the handle locked in; knocked out at the end.
  strike(ev, 3.4, 0.5, 0, [[210, 0.03, 1], [520, 0.015, 0.4]], { hz: 800, q: 0.7, tau: 0.006, amp: 0.8 });
  strike(ev, 4.05, 0.35, 0, [[1700, 0.025, 1], [3100, 0.015, 0.6], [5200, 0.008, 0.3]], { hz: 3000, q: 0.7, tau: 0.003, amp: 1 });
  for (const at of [14.2, 14.55]) strike(ev, at, 0.6, 0.1, [[160, 0.05, 1], [620, 0.02, 0.5], [1850, 0.012, 0.3]], { hz: 1000, q: 0.6, tau: 0.008, amp: 1 });
  return done(rate, [bed, ev], 0.13, 0, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * A blender: a universal motor at about 18,000 turns a minute (300 Hz and its many harmonics,
 * with the commutator's whine high above), its fan's rush, the fruit and ice churning in the jug,
 * the speed dipping as lumps hit the blades and knocking the glass.
 */
function blenderSound(rate: number): Stereo {
  const r = random(37);
  const seconds = 5;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const load = wander(r, n, rate, 0.12);
  const churn = wander(r, n, rate, 0.03);
  const speed = (i: number) => 300 * (1 - 0.04 * Math.max(0, load[i]) - 0.015 * churn[i]);
  const hum = motor(n, rate, speed, [1, 0.8, 0.7, 0.6, 0.5, 0.45, 0.4, 0.35, 0.3, 0.25, 0.22, 0.2, 0.18, 0.16, 0.14, 0.12]);
  const comm = motor(n, rate, (i) => 12 * speed(i), [1, 0.3]);
  const fan = noise(n, rate, r, 'bandpass', 2600, 0.45);
  // The brushes sparking on the commutator: a hiss above.
  const hiss = noise(n, rate, r, 'highpass', 5000, 0.7);
  const slosh = noise(n, rate, r, 'lowpass', 520, 0.8);
  for (let i = 0; i < n; i++) {
    const v = hum[i] * 0.05 + comm[i] * 0.02 + fan[i] * 1.1 + hiss[i] * 0.25 + slosh[i] * 0.45 * (1 + 0.6 * churn[i]);
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  for (let t = r() * 0.2; t < seconds; t += 0.08 + r() * 0.5) strike(ev, t, 0.12 * (0.3 + r()), (r() - 0.5) * 0.2, [[2500 + 600 * r(), 0.02, 1], [4300 + 900 * r(), 0.01, 0.6]], { hz: 3500, q: 0.8, tau: 0.002, amp: 1 });
  return done(rate, [bed, ev], 0.16, 0, [3.1, 5.3, 7.1, 9.7], 0.12);
}

/** A stand mixer: a slower motor (about 160 Hz) with its gears' whine, and the beater slapping the batter round the bowl. */
function mixerSound(rate: number): Stereo {
  const r = random(38);
  const seconds = 6;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const wobble = wander(r, n, rate, 0.3);
  const hum = motor(n, rate, (i) => 160 * (1 + 0.01 * wobble[i]), [1, 0.7, 0.5, 0.4, 0.3, 0.25, 0.2, 0.15]);
  const gears = motor(n, rate, (i) => 1440 * (1 + 0.01 * wobble[i]), [1, 0.3]);
  const air = noise(n, rate, r, 'bandpass', 1800, 0.5);
  const batter = noise(n, rate, r, 'lowpass', 700, 0.8);
  // The beater goes round the bowl about four times a second.
  for (let i = 0; i < n; i++) {
    const t = i / rate;
    const slap = Math.max(0, Math.sin(2 * Math.PI * 4 * t)) ** 6;
    const v = hum[i] * 0.04 + gears[i] * 0.008 + air[i] * 0.25 + batter[i] * (0.15 + 0.8 * slap);
    bed.left[i] = v;
    bed.right[i] = v;
  }
  return done(rate, [bed], 0.12, 0, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * A microwave cooking: the transformer's hum (twice the 50 Hz mains, 100 Hz, with its harmonics),
 * the cooling fan's rush and its blades, the turntable's motor, the odd spit from the food; all
 * inside a metal box.
 */
function microwaveSound(rate: number): Stereo {
  const r = random(39);
  const seconds = 8;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const wobble = wander(r, n, rate, 1.2);
  const hum = motor(n, rate, () => 100, [1, 0.55, 0.4, 0.25, 0.15, 0.1]);
  const blades = motor(n, rate, (i) => 245 * (1 + 0.005 * wobble[i]), [1, 0.3]);
  const fan = noise(n, rate, r, 'bandpass', 750, 0.5);
  for (let i = 0; i < n; i++) {
    const v = hum[i] * 0.07 + blades[i] * 0.012 + fan[i] * 0.45;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  for (const at of [1.3, 4.2, 4.35, 6.8]) strike(ev, at + r() * 0.1, 0.08, 0, [], { hz: 3000 + 2000 * r(), q: 0.9, tau: 0.003, amp: 1 });
  return done(rate, [bed, ev], 0.12, 0, [3.7, 6.1, 8.3, 10.9], 0.15, 3000);
}

/** A toaster browning: the elements ticking as they heat, a faint buzz and the hiss of the heat. */
function toasterSound(rate: number): Stereo {
  const r = random(40);
  const seconds = 5;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const buzz = motor(n, rate, () => 100, [1, 0.5, 0.3]);
  const hiss = noise(n, rate, r, 'highpass', 4000, 0.6);
  for (let i = 0; i < n; i++) {
    const v = buzz[i] * 0.02 + hiss[i] * 0.08;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  for (let t = r() * 0.3; t < seconds; t += 0.25 + r() * 0.7) strike(ev, t, 0.2 * (0.3 + r()), (r() - 0.5) * 0.2, [[2400 + 2600 * r(), 0.006, 1], [6100 + 1000 * r(), 0.003, 0.5]], { hz: 4000, q: 1, tau: 0.001, amp: 0.8 });
  return done(rate, [bed, ev], 0.06, 0, [3.1, 5.3, 7.1, 9.7], 0.1);
}

/**
 * A fan oven: its motor's hum (50 Hz mains, 100 Hz), the fan at about 2,700 turns a minute with
 * eight blades (a tone near 360 Hz), the hot air's rush inside the oven, a tick of the metal
 * now and then.
 */
function ovenFanSound(rate: number): Stereo {
  const r = random(41);
  const seconds = 8;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const wobble = wander(r, n, rate, 1.5);
  const hum = motor(n, rate, () => 50, [0.5, 1, 0.3, 0.3, 0.1]);
  const blades = motor(n, rate, (i) => 360 * (1 + 0.004 * wobble[i]), [1, 0.35, 0.1]);
  const air = noise(n, rate, r, 'bandpass', 600, 0.5);
  for (let i = 0; i < n; i++) {
    const v = hum[i] * 0.05 + blades[i] * 0.03 + air[i] * 0.5;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  for (const at of [2.1, 5.7]) strike(ev, at + r() * 0.2, 0.15, 0, [[2900, 0.01, 1], [5300, 0.006, 0.5]], { hz: 3500, q: 1, tau: 0.002, amp: 1 });
  return done(rate, [bed, ev], 0.1, 0, [3.7, 6.1, 8.3, 10.9], 0.15, 2600);
}

/** A cooker hood drawing the steam off a hob: a centrifugal fan's broad rush, its blades' whine and the motor's hum. */
function hoodSound(rate: number): Stereo {
  const r = random(42);
  const seconds = 8;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const wobble = wander(r, n, rate, 0.8);
  const hum = motor(n, rate, () => 100, [1, 0.5, 0.3]);
  const blades = motor(n, rate, (i) => 690 * (1 + 0.004 * wobble[i]), [1, 0.25]);
  const rush = noise(n, rate, r, 'bandpass', 520, 0.45);
  const high = noise(n, rate, r, 'bandpass', 1900, 0.6);
  for (let i = 0; i < n; i++) {
    const v = hum[i] * 0.03 + blades[i] * 0.012 + rush[i] * (0.7 + 0.04 * wobble[i]) + high[i] * 0.15;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  return done(rate, [bed], 0.14, 0, [3.1, 5.3, 7.1, 9.7], 0.12);
}

/**
 * A wood fire: the flames' low roar flickering, crackles all the time (most tiny, some sharp
 * pops as sap bursts) and now and then the hiss of steam from damp wood.
 */
function fireSound(rate: number): Stereo {
  const r = random(43);
  const seconds = 10;
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const flicker = wander(r, n, rate, 0.25);
  const slow = wander(r, n, rate, 1.5);
  const roar = noise(n, rate, r, 'lowpass', 320, 0.7);
  const lick = noise(n, rate, r, 'bandpass', 850, 0.6);
  const steam = noise(n, rate, r, 'bandpass', 4200, 0.7);
  for (let i = 0; i < n; i++) {
    const t = i / rate;
    const s = ramp(5.8, 6.4, t) * (1 - ramp(7.2, 8.2, t));
    const v = roar[i] * (0.9 + 0.35 * flicker[i] + 0.2 * slow[i]) + lick[i] * 0.12 * Math.max(0, 1 + flicker[i]) + steam[i] * 0.06 * s;
    bed.left[i] = v;
    bed.right[i] = v;
  }
  const ev = loop(rate, seconds);
  for (let t = r() * 0.01; t < seconds; t += -Math.log(1 - r()) / 28) {
    const a = 0.25 * r() ** 3;
    strike(ev, t, a, (r() - 0.5) * 0.6, [], { hz: 1500 + 5000 * r(), q: 1, tau: 0.0006 + 0.002 * r(), amp: 1.5 });
  }
  for (let t = 0.5 + r(); t < seconds - 0.2; t += 1 + r() * 2.5) {
    const pan = (r() - 0.5) * 0.5;
    strike(ev, t, 0.5 + 0.4 * r(), pan, [[300 + 200 * r(), 0.01, 0.6]], { hz: 1200 + 1500 * r(), q: 0.7, tau: 0.006, amp: 1.5 });
    // A sharp pop often throws off a few small sparks.
    for (let k = 0; k < 3; k++) strike(ev, t + 0.02 + r() * 0.15, 0.1 * r(), pan, [], { hz: 4000 + 3000 * r(), q: 1, tau: 0.0008, amp: 1.5 });
  }
  return done(rate, [bed, ev], 0.14, 0, [7.1, 9.3, 11.7, 13.9], 0.1);
}

// The games.

/**
 * Pool: a shot every few seconds. The cue's leather tip on the cue ball (a dull tock), the balls
 * clacking (phenolic resin: a click of a few milliseconds, most of it 3 to 6 kHz), a cushion's
 * thump, the balls rolling on the cloth, now and then one dropping into a pocket; a cue chalked.
 */
function poolSound(rate: number): Stereo {
  const r = random(44);
  const seconds = 20;
  const ev = loop(rate, seconds);
  const bed = track(rate, seconds);
  const n = bed.left.length;
  const rumble = noise(n, rate, r, 'bandpass', 220, 0.9);
  const rolling = new Float32Array(n);
  const shots = [0.6, 3.8, 7.1, 10.2, 13.6, 16.8];
  for (const [k, at] of shots.entries()) {
    const pan = (r() - 0.5) * 0.6;
    const power = 0.5 + 0.5 * r();
    // Chalking the cue before some shots.
    if (k % 2 === 0) for (let c = 0; c < 3; c++) strike(ev, at - 1.6 + c * 0.18, 0.04, pan, [], { hz: 3200, q: 1.5, tau: 0.04, amp: 1 });
    strike(ev, at, 0.4 * power, pan, [[1100, 0.006, 1]], { hz: 1500, q: 0.6, tau: 0.003, amp: 1.2 });
    const hit = at + 0.18 + 0.3 * r() * (1 - power);
    ball(ev, hit, 0.9 * power, pan);
    ball(ev, hit + 0.2 + 0.3 * r(), 0.5 * power, -pan);
    if (r() < 0.5) ball(ev, hit + 0.55 + 0.4 * r(), 0.3 * power, pan);
    const cushion = hit + 0.4 + 0.5 * r();
    strike(ev, cushion, 0.3 * power, pan * 0.5, [[260, 0.03, 1], [520, 0.015, 0.3]], { hz: 600, q: 0.7, tau: 0.008, amp: 1 });
    if (k % 2) {
      // Into the pocket: a knock on the pocket's edge, the drop and a clack against the balls already in.
      const drop = cushion + 0.5 + 0.3 * r();
      strike(ev, drop, 0.4, pan, [[320, 0.04, 1], [810, 0.02, 0.4]], { hz: 900, q: 0.6, tau: 0.006, amp: 1 });
      ball(ev, drop + 0.18, 0.35, pan);
      ball(ev, drop + 0.27, 0.15, pan);
    }
    for (let i = Math.round(at * rate); i < Math.min(n, (at + 3.5) * rate); i++) {
      const t = i / rate - at;
      rolling[i] += power * Math.exp(-t / 1.1) * ramp(0.1, 0.3, t);
    }
  }
  for (let i = 0; i < n; i++) {
    bed.left[i] = rumble[i] * 0.3 * rolling[i];
    bed.right[i] = rumble[i] * 0.3 * rolling[i];
  }
  return done(rate, [bed, ev], SPARSE, 0, [5.3, 7.9, 10.7, 13.1], 0.15);
}

/** Two pool balls clacking: a very short, hard click. */
function ball(out: Track, at: number, amp: number, pan: number): void {
  strike(out, at, amp, pan, [[3800, 0.003, 0.6], [6200, 0.002, 0.4]], { hz: 4500, q: 0.5, tau: 0.0015, amp: 1.6 });
}

/**
 * Foosball: the hard ball whacked by the plastic men and knocking round the wooden box, the rods
 * slid and spun, quick flurries between quieter build-ups, and a goal (the ball's clunk into the
 * goal and its rumble down the return).
 */
function foosballSound(rate: number): Stereo {
  const r = random(45);
  const seconds = 12;
  const ev = loop(rate, seconds);
  let t = 0.2;
  while (t < seconds - 1) {
    // A flurry: shots and passes, the ball off the walls, a rod slid.
    const hits = 3 + Math.floor(r() * 6);
    for (let k = 0; k < hits && t < seconds - 1; k++) {
      const pan = (r() - 0.5) * 0.7;
      const shot = r() < 0.3;
      strike(ev, t, shot ? 0.7 : 0.35 + 0.2 * r(), pan, [[1800 + 300 * r(), 0.01, 1], [3100, 0.006, 0.5], [190, 0.04, 0.5]], { hz: 2500, q: 0.7, tau: 0.003, amp: 1.4 });
      if (r() < 0.5) strike(ev, t + 0.05 + 0.1 * r(), 0.3, -pan, [[420, 0.02, 1], [1100, 0.01, 0.4]], { hz: 1200, q: 0.7, tau: 0.004, amp: 1 });
      if (r() < 0.3) rustle(ev, r, t + 0.02, 0.15, 0.06, pan);
      t += shot ? 0.25 + 0.2 * r() : 0.12 + 0.3 * r();
    }
    t += 0.6 + r() * 1.2;
  }
  // A goal: in, and down the return to the end.
  const goal = 7.4;
  strike(ev, goal, 0.8, 0.4, [[220, 0.05, 1], [560, 0.02, 0.5]], { hz: 900, q: 0.6, tau: 0.008, amp: 1.2 });
  for (let k = 0; k < 12; k++) strike(ev, goal + 0.15 + k * 0.05, 0.08, 0.5, [[300 + 100 * r(), 0.02, 1]], { hz: 600, q: 1, tau: 0.004, amp: 0.6 });
  strike(ev, goal + 0.85, 0.4, 0.6, [[260, 0.04, 1], [700, 0.02, 0.4]], { hz: 800, q: 0.7, tau: 0.005, amp: 1 });
  return done(rate, [ev], SPARSE, 0, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * Ping-pong: rallies of the ball off the bats and the table, a hit then a bounce about a quarter
 * of a second later, back and forth; a rally ends with the ball bouncing away on the floor, a
 * pause, and a serve.
 */
function pingPongSound(rate: number): Stereo {
  const r = random(46);
  // Two rallies, planned first: the loop is as long as they take.
  const plan: ((ev: Track) => void)[] = [];
  let t = 0.3;
  for (let rally = 0; rally < 2; rally++) {
    // The serve bounces on the server's side first.
    let side = r() < 0.5 ? -1 : 1;
    const serve = t;
    const server = side;
    plan.push((ev) => {
      bat(ev, serve, 0.5, server * 0.5);
      bounce(ev, serve + 0.18, 0.5, server * 0.25);
    });
    t += 0.45;
    const shots = 4 + Math.floor(r() * 8);
    for (let k = 0; k < shots; k++) {
      const at = t;
      const gap = 0.2 + 0.12 * r();
      const loud = 0.5 + 0.3 * r();
      const from = side;
      plan.push((ev) => {
        bounce(ev, at, 0.6, -from * 0.25);
        bat(ev, at + gap, loud, -from * 0.5);
      });
      side = -side;
      t += gap + 0.24 + 0.12 * r();
    }
    // Missed: off the end and away across the floor, each bounce lower and sooner.
    let gap = 0.4;
    let a = 0.4;
    for (let k = 0; k < 7; k++, gap *= 0.78, a *= 0.75) {
      const at = t;
      const level = a;
      const pan = side * 0.8;
      plan.push((ev) => strike(ev, at, level, pan, [[2200, 0.006, 1], [5700, 0.003, 0.4]], { hz: 2000, q: 0.8, tau: 0.002, amp: 0.8 }));
      t += gap;
    }
    t += 1.5 + r();
  }
  const ev = loop(rate, t);
  for (const p of plan) p(ev);
  return done(rate, [ev], SPARSE, 0, [5.3, 7.9, 10.7, 13.1], 0.18);
}

/** A ping-pong ball off a bat: its shell's pock over the bat's wooden knock. */
function bat(out: Track, at: number, amp: number, pan: number): void {
  strike(out, at, amp, pan, [[1500, 0.012, 1], [5700, 0.004, 0.3], [900, 0.008, 0.5]], { hz: 1800, q: 0.8, tau: 0.004, amp: 1 });
}

/** A ping-pong ball bouncing on the table: the pock and the table top's thump under it. */
function bounce(out: Track, at: number, amp: number, pan: number): void {
  strike(out, at, amp, pan, [[1500, 0.012, 1], [5700, 0.004, 0.3], [240, 0.035, 0.6], [520, 0.02, 0.3]], { hz: 1600, q: 0.8, tau: 0.003, amp: 0.8 });
}

/**
 * Darts: three thrown about a second apart, each a dull thud into the bristle board (with the
 * board's backing knocking on the wall), then a walk to the board and the three pulled out; now
 * and then one hits the wire and falls to the floor.
 */
function dartsSound(rate: number): Stereo {
  const r = random(47);
  const seconds = 12;
  const ev = loop(rate, seconds);
  for (const [round, start] of [0.4, 6.4].entries()) {
    for (let k = 0; k < 3; k++) {
      const at = start + k * (1 + 0.25 * r());
      if (round === 1 && k === 1) {
        // Off the wire, and clattering on the floor.
        strike(ev, at, 0.25, 0.1, [[3800, 0.012, 1], [6100, 0.008, 0.5]], { hz: 4000, q: 0.8, tau: 0.002, amp: 1 });
        strike(ev, at + 0.42, 0.2, 0.2, [[2600, 0.01, 1], [4100, 0.006, 0.6]], { hz: 3000, q: 0.8, tau: 0.002, amp: 1 });
        strike(ev, at + 0.55, 0.08, 0.25, [[2900, 0.008, 1]], { hz: 3000, q: 0.8, tau: 0.002, amp: 1 });
        continue;
      }
      strike(ev, at, 0.6 + 0.2 * r(), 0.05 * (k - 1), [[140, 0.05, 0.6], [320, 0.02, 0.3]], { hz: 950, q: 0.6, tau: 0.008, amp: 1.5 });
    }
    // Pulled out of the board: a short scratch each, and the darts knocking together in the hand.
    for (let k = 0; k < 3; k++) strike(ev, start + 4.1 + k * 0.4 + r() * 0.1, 0.18, 0, [], { hz: 2600, q: 2, tau: 0.03, amp: 1.5 });
    strike(ev, start + 5.4, 0.1, 0, [[4400, 0.03, 1], [7100, 0.02, 0.5]], { hz: 5000, q: 1, tau: 0.001, amp: 0.5 });
  }
  return done(rate, [ev], SPARSE, 0, [5.3, 7.9, 10.7, 13.1], 0.15);
}

/**
 * An arcade cabinet: a chiptune (square-wave lead and a triangle bass in A minor, a noise hi-hat),
 * the game's own zaps, booms and a bonus jingle, and the player's buttons and joystick clicking;
 * through the cabinet's small speaker.
 */
function arcadeSound(rate: number): Stereo {
  const r = random(48);
  const beat = 60 / 140;
  const seconds = 16 * beat;
  const out = loop(rate, seconds);
  const music = new Float32Array(out.left.length);
  const note = (midi: number) => 440 * 2 ** ((midi - 69) / 12);
  // The lead: eighth notes walking the minor pentatonic, a rest now and then.
  const scale = [57, 60, 62, 64, 67, 69, 72, 74, 76];
  let at = 3;
  for (let e = 0; e < 32; e++) {
    at = Math.max(0, Math.min(scale.length - 1, at + Math.round((r() - 0.5) * 3)));
    if (r() < 0.12) continue;
    chip(music, rate, e * (beat / 2), beat / 2 - 0.02, note(scale[at] + 12), 0.25, 0.22);
  }
  // The bass: root and fifth, a quarter note each, round A, F, C, G.
  for (let b = 0; b < 16; b++) {
    const root = [45, 41, 48, 43][Math.floor(b / 4)];
    tri(music, rate, b * beat, beat * 0.9, note(root + (b % 2 ? 7 : 0) - 12), 0.35);
  }
  for (let e = 0; e < 32; e++) if (e % 2) strike(out, e * (beat / 2), 0.08, 0, [], { hz: 7000, q: 0.7, tau: 0.01, amp: 1.5 });
  for (let i = 0; i < music.length; i++) {
    out.left[i] += music[i];
    out.right[i] += music[i];
  }
  // The game: zaps falling in pitch, a boom, the bonus jingle.
  for (let t = 0.3; t < seconds - 0.3; t += 0.35 + r() * 0.9) zap(out, t, 1800 + 600 * r(), 0.12);
  for (const t of [2.4, 5.1]) strike(out, t, 0.6, 0, [[70, 0.12, 1]], { hz: 300, q: 0.5, tau: 0.12, amp: 2 });
  chip(out.left, rate, 4.2, 0.07, 988, 0.5, 0.2);
  chip(out.left, rate, 4.27, 0.25, 1319, 0.5, 0.2);
  chip(out.right, rate, 4.2, 0.07, 988, 0.5, 0.2);
  chip(out.right, rate, 4.27, 0.25, 1319, 0.5, 0.2);
  // Buttons (microswitches) in bursts, and the joystick's knocks.
  for (let t = 0.1; t < seconds - 0.1; t += 0.12 + r() * 0.5) {
    strike(out, t, 0.1, 0.15, [[4200, 0.003, 1], [6800, 0.002, 0.6]], { hz: 3000, q: 0.8, tau: 0.001, amp: 1 });
    if (r() < 0.4) strike(out, t + 0.05, 0.06, -0.1, [[900, 0.006, 1], [2300, 0.004, 0.5]], { hz: 1500, q: 0.8, tau: 0.002, amp: 1 });
  }
  return done(rate, [out], 0.12, 0, [3.1, 5.3, 7.1, 9.7], 0.1, 5000, 180);
}

/** A square-ish note (a pulse of `duty`, its harmonics added up to 8 kHz), with a short attack and release. */
function chip(d: Float32Array, rate: number, at: number, seconds: number, hz: number, duty: number, amp: number): void {
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  const top = Math.floor(8000 / hz);
  const k = Array.from({ length: top }, (_, i) => (2 / ((i + 1) * Math.PI)) * Math.sin((i + 1) * Math.PI * duty));
  for (let j = 0; j < len; j++) {
    const i = (start + j) % d.length;
    const t = j / rate;
    let v = 0;
    for (let h = 0; h < top; h++) v += k[h] * Math.sin(2 * Math.PI * hz * (h + 1) * t);
    d[i] += v * amp * Math.min(1, j / (0.003 * rate), (len - j) / (0.01 * rate));
  }
}

/** A triangle-wave note (the chip's bass). */
function tri(d: Float32Array, rate: number, at: number, seconds: number, hz: number, amp: number): void {
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  for (let j = 0; j < len; j++) {
    const x = (hz * j) / rate;
    const v = 4 * Math.abs(x - Math.floor(x + 0.5)) - 1;
    d[(start + j) % d.length] += v * amp * Math.min(1, j / (0.003 * rate), (len - j) / (0.01 * rate));
  }
}

/** A game's zap: a square wave falling fast in pitch. */
function zap(out: Track, at: number, from: number, amp: number): void {
  const rate = out.rate;
  const start = Math.round(at * rate);
  const len = Math.round(0.14 * rate);
  let phase = 0;
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    phase += (from * Math.exp(-t / 0.05)) / rate;
    const v = (Math.sin(2 * Math.PI * phase) + Math.sin(6 * Math.PI * phase) / 3 + Math.sin(10 * Math.PI * phase) / 5) * amp * (1 - j / len);
    const i = (start + j) % out.left.length;
    out.left[i] += v;
    out.right[i] += v;
  }
}

/**
 * Basketball on an outdoor court: dribbling (the ball's slap on the court and the ring of the air
 * inside it, about 950 and 1500 Hz for a ball 24 cm across), a shot that goes in with a swish of
 * the net or clangs off the rim and the board, and the ball bouncing away under the hoop, each
 * bounce lower and sooner (it keeps about 80 % of its speed).
 */
function basketballSound(rate: number): Stereo {
  const r = random(49);
  // Two shots, the second after more dribbling, and the ball picked up again as the loop comes round.
  const seconds = 21.5;
  const ev = loop(rate, seconds);
  let t = 0.2;
  for (const [k, made] of [true, false].entries()) {
    const dribbles = 6 + k * 2;
    for (let d = 0; d < dribbles; d++, t += 0.5 + 0.04 * r()) dribble(ev, t, 0.7 + 0.15 * r(), (r() - 0.5) * 0.2);
    // The shot: up for about a second.
    t += 1.1;
    if (made) for (let j = 0; j < 2; j++) strike(ev, t + j * 0.04, 0.25, 0, [], { hz: 4200, q: 0.6, tau: 0.06, amp: 1 });
    else {
      strike(ev, t, 0.5, 0.05, [[420, 0.22, 1], [1100, 0.16, 0.8], [1960, 0.1, 0.6], [3100, 0.07, 0.4]], { hz: 1500, q: 0.6, tau: 0.004, amp: 1 });
      strike(ev, t + 0.14, 0.4, -0.05, [[180, 0.05, 1], [430, 0.02, 0.4]], { hz: 700, q: 0.6, tau: 0.006, amp: 1 });
    }
    // Down from the hoop (3 m: about 0.8 s), then bouncing away.
    t += 0.78;
    let gap = 2 * 0.78 * 0.8;
    let a = 1;
    for (let b = 0; b < 6; b++, a *= 0.8, gap *= 0.8) {
      dribble(ev, t, a, 0.3);
      t += gap;
    }
    // Picked up again.
    t += 0.4;
  }
  return done(rate, [ev], SPARSE, 0, [], 0);
}

/** A basketball hitting the court: the slap of its skin and the ring of the air inside. */
function dribble(out: Track, at: number, amp: number, pan: number): void {
  strike(out, at, amp, pan, [[110, 0.03, 0.8], [950, 0.05, 0.5], [1520, 0.03, 0.3]], { hz: 1500, q: 0.6, tau: 0.003, amp: 0.8 });
}

// The rest.

/**
 * A workbench: a nail hammered in (the steel's ring rising as it goes in, then the dull last
 * blows on the wood), a board sawn in strokes (the teeth catching, the push louder than the
 * pull), the cut-off dropping, sanding, a screwdriver's ratchet, a tool put down.
 */
function toolsSound(rate: number): Stereo {
  const r = random(50);
  const seconds = 20;
  const ev = loop(rate, seconds);
  for (let k = 0; k < 6; k++) {
    const at = 0.4 + k * 0.42;
    const rise = 1 + 0.03 * k;
    if (k < 5) strike(ev, at, 0.6, 0.1, [[2600 * rise, 0.05, 0.6], [4100 * rise, 0.03, 0.4], [6900 * rise, 0.02, 0.2], [180, 0.04, 0.8], [420, 0.03, 0.4]], { hz: 2000, q: 0.7, tau: 0.004, amp: 1.2 });
    else strike(ev, at, 0.8, 0.1, [[180, 0.05, 1], [420, 0.03, 0.5], [900, 0.015, 0.3]], { hz: 1200, q: 0.7, tau: 0.006, amp: 1.2 });
  }
  strike(ev, 3.6, 0.35, 0.2, [[250, 0.03, 1], [1700, 0.03, 0.4]], { hz: 900, q: 0.7, tau: 0.005, amp: 1 });
  // Sawing: eight strokes.
  let t = 4.6;
  for (let k = 0; k < 8; k++) {
    const push = k % 2 === 0;
    const len = push ? 0.55 : 0.45;
    saw(ev, r, t, len, push ? 0.5 : 0.3, -0.1);
    t += len + 0.04;
  }
  strike(ev, t + 0.1, 0.6, -0.2, [[150, 0.06, 1], [380, 0.03, 0.4], [820, 0.015, 0.2]], { hz: 700, q: 0.6, tau: 0.008, amp: 1 });
  // Sanding.
  t += 1.5;
  for (let k = 0; k < 10; k++, t += 0.4) rustle(ev, r, t, 0.35, 0.3, 0, 1900, 0.5);
  // A screwdriver's ratchet: three turns.
  t += 0.8;
  for (let k = 0; k < 3; k++, t += 0.9) for (let c = 0; c < 8; c++) strike(ev, t + c * 0.07, 0.12, 0.15, [[3500, 0.006, 1], [6100, 0.004, 0.5]], { hz: 4000, q: 1, tau: 0.001, amp: 0.8 });
  strike(ev, t + 0.3, 0.3, 0.1, [[2100, 0.06, 0.5], [3700, 0.04, 0.4], [300, 0.02, 0.6]], { hz: 2000, q: 0.7, tau: 0.003, amp: 1 });
  return done(rate, [ev], SPARSE, 0, [7.1, 9.3, 11.7, 13.9], 0.18);
}

/** A hand saw's stroke: the teeth catching about 70 times a second, in a high band, swelling and fading with the stroke. */
function saw(out: Track, r: () => number, at: number, seconds: number, amp: number, pan: number): void {
  const rate = out.rate;
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  const [l, rr] = gains(pan);
  const teeth = new Biquad(rate).set('bandpass', 3600, 0.9);
  const board = new Biquad(rate).set('bandpass', 420, 1.5);
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    const x = r() * 2 - 1;
    const env = Math.sin((Math.PI * j) / len) ** 0.7;
    const catchy = 0.5 + 0.5 * Math.abs(Math.sin(Math.PI * 70 * t));
    const v = (teeth.run(x) * catchy * 1.5 + board.run(x) * 0.6) * env * amp;
    put(out, start + j, v, l, rr);
  }
}

/**
 * A bike looked after on its stand: a wheel spun and its freewheel ticking slower and slower, a
 * spanner's ratchet, the gears changed, the tyres pumped up.
 */
function bikeToolsSound(rate: number): Stereo {
  const r = random(51);
  const seconds = 14;
  const ev = loop(rate, seconds);
  // The freewheel: pawls ticking from about 28 a second down to a few as the wheel slows.
  for (const start of [0.3, 7.5]) {
    let t = start;
    for (let k = 0; t < start + 3.5; k++) {
      const rateNow = 28 * Math.exp(-(t - start) / 1.6);
      strike(ev, t, 0.1, -0.2, [[5200, 0.004, 1], [7800, 0.003, 0.5]], { hz: 5000, q: 1, tau: 0.001, amp: 0.6 });
      t += 1 / Math.max(3, rateNow);
    }
  }
  // A spanner's ratchet: three swings.
  for (let k = 0; k < 3; k++) for (let c = 0; c < 7; c++) strike(ev, 4.2 + k * 0.8 + c * 0.07, 0.15, 0.2, [[3400, 0.006, 1], [5900, 0.004, 0.5]], { hz: 3800, q: 1, tau: 0.001, amp: 0.8 });
  // The gears changed: the derailleur's clack.
  strike(ev, 6.8, 0.3, -0.1, [[1200, 0.03, 1], [2900, 0.02, 0.6]], { hz: 2000, q: 0.7, tau: 0.003, amp: 1 });
  // The tyre pumped: four strokes of rising hiss.
  for (let k = 0; k < 4; k++) {
    const at = 11.4 + k * 0.6;
    const start = Math.round(at * rate);
    const len = Math.round(0.4 * rate);
    const bp = new Biquad(rate);
    for (let j = 0; j < len; j++) {
      if ((j & 31) === 0) bp.set('bandpass', 1200 + 800 * (j / len), 1.2);
      put(ev, start + j, bp.run(r() * 2 - 1) * 0.4 * Math.sin((Math.PI * j) / len), 0.7, 0.7);
    }
  }
  return done(rate, [ev], SPARSE, 0, [5.3, 7.9, 10.7, 13.1], 0.12);
}

/**
 * A punch bag worked over: combinations of punches (each the glove's slap on the leather and the
 * thump of the heavy bag, about 85 Hz), the chain jingling above as it swings, the mount creaking,
 * a sharp breath out with each punch.
 */
function punchBagSound(rate: number): Stereo {
  const r = random(52);
  const seconds = 12;
  const ev = loop(rate, seconds);
  const combos = [
    [0.4, 0.65, 0.95],
    [2.7, 3.05],
    [4.6, 4.8, 5.05, 5.4],
    [7.3],
    [8.6, 8.9, 9.2],
    [10.9, 11.1],
  ];
  for (const combo of combos) {
    for (const [k, at] of combo.entries()) {
      const power = k === combo.length - 1 ? 1 : 0.7 + 0.15 * r();
      const pan = (r() - 0.5) * 0.2;
      strike(ev, at, power, pan, [[85, 0.07, 1], [140, 0.04, 0.5]], { hz: 1300, q: 0.7, tau: 0.006, amp: 1.6 });
      strike(ev, at - 0.02, 0.12 * power, pan, [], { hz: 2600, q: 0.8, tau: 0.04, amp: 1 });
      for (let c = 0; c < 4; c++) strike(ev, at + 0.04 + 0.03 * c + 0.02 * r(), 0.05 * power, 0.1, [[3000 + 3000 * r(), 0.05, 1], [7000 + 2000 * r(), 0.03, 0.4]], { hz: 5000, q: 1, tau: 0.001, amp: 0.4 });
    }
    // Swinging back: the mount creaks.
    const last = combo[combo.length - 1];
    strike(ev, last + 0.55, 0.08, 0.1, [[600, 0.05, 1]], { hz: 900, q: 4, tau: 0.05, amp: 1 });
  }
  return done(rate, [ev], 0, 0.75, [5.3, 7.9, 10.7, 13.1], 0.15);
}

/**
 * Plates and glasses smashed in a metal bin: each the crash of the impact (the bin's body ringing
 * under it), then the shards: dozens of small pieces clinking down, the bigger ones landing last,
 * then a tinkle as they settle. Glass rings higher and longer than china.
 */
function smashSound(rate: number): Stereo {
  const r = random(53);
  const seconds = 12;
  const ev = loop(rate, seconds);
  for (const [k, at] of [0.3, 3.4, 6.5, 9.2].entries()) {
    const glass = k % 2 === 1;
    const pan = (r() - 0.5) * 0.3;
    strike(ev, at, 0.8, pan, [[310, 0.3, 0.4], [740, 0.25, 0.35], [1350, 0.18, 0.3], [2200, 0.12, 0.2], [90, 0.05, 0.8]], { hz: 3000, q: 0.4, tau: 0.012, amp: 2.4 });
    const pieces = 40 + Math.floor(r() * 30);
    for (let p = 0; p < pieces; p++) {
      const dt = -Math.log(1 - r()) * 0.09;
      const f = glass ? 3000 + 6000 * r() : 2000 + 4000 * r();
      const ring = glass ? 0.02 + 0.06 * r() : 0.01 + 0.03 * r();
      strike(ev, at + 0.01 + dt, 0.25 * Math.exp(-dt / 0.25) * (0.3 + r()), pan + (r() - 0.5) * 0.4, [[f, ring, 1], [f * 2.3, ring * 0.6, 0.5]], { hz: f, q: 1, tau: 0.001, amp: 0.6 });
    }
    for (let p = 0; p < 4; p++) {
      const dt = 0.25 + 0.4 * r();
      strike(ev, at + dt, 0.25, pan, [[1500 + 1500 * r(), 0.04, 1], [4200, 0.02, 0.4]], { hz: 2000, q: 0.8, tau: 0.002, amp: 1 });
    }
  }
  return done(rate, [ev], 0, 0.75, [5.3, 7.9, 10.7, 13.1], 0.15);
}

/**
 * A cello played with a bow, a slow lament in D minor: each note a bowed string (a sawtooth, with
 * the bow's scratch as it bites and its hiss along the string), vibrato coming in as it's held,
 * through the body's resonances (the air near 100 Hz, the wood near 190 and 290 Hz, the bridge
 * near 2.2 kHz); in a living room.
 */
function celloSound(rate: number): Stereo {
  const r = random(54);
  const notes: [number, number][] = [
    [50, 2],
    [53, 1],
    [52, 1],
    [50, 1.5],
    [48, 0.5],
    [45, 2],
    [46, 1.5],
    [45, 0.5],
    [43, 1],
    [45, 1],
    [38, 3],
  ];
  const seconds = notes.reduce((s, [, d]) => s + d, 0);
  const ev = loop(rate, seconds);
  const n = ev.left.length;
  const dry = new Float32Array(n);
  let at = 0;
  for (const [k, [midi, dur]] of notes.entries()) {
    const hz = 440 * 2 ** ((midi - 69) / 12);
    const start = Math.round(at * rate);
    const len = Math.round((dur + 0.15) * rate);
    const top = Math.min(40, Math.floor(6000 / hz));
    const scratch = new Biquad(rate).set('highpass', 1500, 0.7);
    const swell = k % 2 ? 0.85 : 1;
    let phase = r();
    for (let j = 0; j < len; j++) {
      const t = j / rate;
      // Vibrato: about 5.5 a second, a sixth of a semitone, coming in after a third of a second.
      const vib = 0.01 * ramp(0.3, 0.7, t) * Math.sin(2 * Math.PI * 5.5 * t);
      phase += (hz * (1 + vib + 0.0008 * (r() - 0.5))) / rate;
      let v = 0;
      for (let h = 1; h <= top; h++) v += Math.sin(2 * Math.PI * h * phase) / h;
      // The bow: biting in, a swell along the stroke, lifting off.
      const env = ramp(0, 0.1, t) * (1 - ramp(dur, dur + 0.15, t)) * (0.75 + 0.25 * Math.sin((Math.PI * Math.min(t, dur)) / dur)) * swell;
      const bite = Math.exp(-t / 0.05);
      dry[(start + j) % n] += v * env * 0.3 + scratch.run(r() * 2 - 1) * (0.02 + 0.12 * bite) * env;
    }
    at += dur;
  }
  // The body.
  const body: [number, number, number][] = [
    [100, 4, 0.8],
    [190, 5, 1],
    [290, 4, 0.8],
    [450, 3, 0.5],
    [600, 3, 0.4],
    [1100, 2, 0.5],
    [2200, 1.5, 0.7],
  ];
  const filters = body.map(([hz, q]) => new Biquad(rate).set('bandpass', hz, q));
  // Twice round, so the resonances have settled at the loop's start.
  for (let pass = 0; pass < 2; pass++) {
    for (let i = 0; i < n; i++) {
      let v = dry[i] * 0.5;
      for (let b = 0; b < body.length; b++) v += filters[b].run(dry[i]) * body[b][2];
      if (pass) {
        ev.left[i] = v;
        ev.right[i] = v;
      }
    }
  }
  return done(rate, [ev], 0.12, 0, [17.3, 23.9, 29.1, 34.7], 0.3);
}

/**
 * Someone singing to a karaoke track: a pop backing (pad chords round C, G, A minor and F, a bass,
 * a soft kick, snare and hi-hat at 120 bpm) and over it a voice (a glottal buzz through the
 * formants of the vowel being sung, vibrato on the held notes, the consonants' hiss), a little
 * echo on the microphone.
 */
function singingSound(rate: number): Stereo {
  const r = random(55);
  const beat = 0.5;
  const seconds = 16 * beat;
  const out = loop(rate, seconds);
  const n = out.left.length;
  const backing = new Float32Array(n);
  const voice = new Float32Array(n);
  const hz = (midi: number) => 440 * 2 ** ((midi - 69) / 12);
  const chords = [
    [48, 52, 55],
    [43, 47, 50],
    [45, 48, 52],
    [41, 45, 48],
  ];
  for (const [bar, chord] of chords.entries()) {
    for (const m of chord) pad(backing, rate, bar * 4 * beat, 4 * beat, hz(m + 12), 0.06);
    for (let b = 0; b < 4; b++) pad(backing, rate, (bar * 4 + b) * beat, beat * 0.9, hz(chord[0] - 12), 0.12, 3);
  }
  for (let b = 0; b < 16; b++) {
    const t = b * beat;
    if (b % 2 === 0) strike(out, t, 0.5, 0, [[60, 0.08, 1]], { hz: 200, q: 0.7, tau: 0.01, amp: 0.6 });
    else strike(out, t, 0.25, 0, [[190, 0.04, 0.6]], { hz: 2500, q: 0.6, tau: 0.06, amp: 1.2 });
    strike(out, t + beat / 2, 0.05, 0.1, [], { hz: 8000, q: 0.7, tau: 0.02, amp: 1.5 });
  }
  // The tune: [midi, beats, vowel], a rest as 0.
  const tune: [number, number, Vowel][] = [
    [64, 1, 'a'],
    [64, 0.5, 'e'],
    [67, 0.5, 'i'],
    [69, 1.5, 'o'],
    [0, 0.5, 'a'],
    [67, 1, 'a'],
    [66, 0.5, 'e'],
    [67, 0.5, 'u'],
    [62, 2, 'o'],
    [64, 1, 'e'],
    [64, 0.5, 'a'],
    [67, 0.5, 'i'],
    [72, 1.5, 'a'],
    [71, 0.5, 'e'],
    [69, 2, 'o'],
    [0, 2, 'a'],
  ];
  let at = 0;
  for (const [midi, beats, vowel] of tune) {
    if (midi > 0) sing(voice, rate, r, at, beats * beat, hz(midi), vowel);
    at += beats * beat;
  }
  for (let i = 0; i < n; i++) {
    const v = backing[i] + voice[i];
    out.left[i] += v;
    out.right[i] += v;
  }
  return done(rate, [out], 0.13, 0, [23.1, 29.3, 37.9, 41.3], 0.25, 7000, 90);
}

type Vowel = 'a' | 'e' | 'i' | 'o' | 'u';

/** The first three formants of the vowels (Hz), a sung voice's. */
const FORMANTS: Record<Vowel, [number, number, number]> = {
  a: [800, 1150, 2900],
  e: [480, 1720, 2520],
  i: [300, 2200, 3000],
  o: [450, 800, 2830],
  u: [325, 700, 2530],
};

/** A sung note: a glottal buzz (harmonics falling 9 dB an octave) through the vowel's formants, with a consonant's hiss at the start and vibrato as it's held. */
function sing(d: Float32Array, rate: number, r: () => number, at: number, seconds: number, hz: number, vowel: Vowel): void {
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  const [f1, f2, f3] = FORMANTS[vowel];
  const filters = [new Biquad(rate).set('bandpass', f1, f1 / 90), new Biquad(rate).set('bandpass', f2, f2 / 110), new Biquad(rate).set('bandpass', f3, f3 / 160)];
  const weights = [1, 0.9, 0.6];
  const hiss = new Biquad(rate).set('highpass', 4500, 0.7);
  const top = Math.floor(5000 / hz);
  const consonant = r() < 0.6;
  let phase = 0;
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    const vib = 0.017 * ramp(0.25, 0.5, t) * Math.sin(2 * Math.PI * 5.5 * t);
    phase += (hz * (1 + vib)) / rate;
    let src = 0;
    for (let h = 1; h <= top; h++) src += Math.sin(2 * Math.PI * h * phase) / h ** 1.5;
    src += (r() * 2 - 1) * 0.05;
    let v = 0;
    for (let k = 0; k < 3; k++) v += filters[k].run(src) * weights[k];
    const env = ramp(0.02, 0.06, t) * (1 - ramp(seconds - 0.06, seconds, t));
    const s = consonant ? Math.exp(-t / 0.03) * (1 - ramp(0.05, 0.06, t)) : 0;
    d[(start + j) % d.length] += v * env * 2.2 + hiss.run(r() * 2 - 1) * s * 0.25;
  }
}

/** A soft synth note: a few harmonics, slowly attacked and released (the backing's pad and bass). */
function pad(d: Float32Array, rate: number, at: number, seconds: number, hz: number, amp: number, harmonics = 6): void {
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    let v = 0;
    for (let h = 1; h <= harmonics; h++) v += Math.sin(2 * Math.PI * h * hz * t) / (h * 1.5);
    d[(start + j) % d.length] += v * amp * ramp(0, 0.05, t) * (1 - ramp(seconds - 0.08, seconds, t));
  }
}

// Played once.

/**
 * A fridge door opened and shut: the handle, the seal letting go with a soft suck, the bottles in
 * the door clinking as it swings, a jar taken out, and about a second and a half later the door
 * pushed shut (a rush of air and the seal's thump), the bottles rattling again.
 */
function fridgeDoorSound(rate: number): Stereo {
  const r = random(60);
  const out = once(rate, 2.6);
  strike(out, 0.02, 0.15, 0, [[1700, 0.006, 1]], { hz: 2500, q: 1, tau: 0.002, amp: 0.6 });
  strike(out, 0.08, 0.6, 0, [[90, 0.02, 0.8]], { hz: 380, q: 0.8, tau: 0.02, amp: 1 });
  for (const at of [0.22, 0.3, 0.46]) glass(out, at + 0.03 * r(), 0.15 + 0.1 * r(), 0.15, 1800 + 800 * r());
  glass(out, 0.95, 0.12, 0.1, 1400);
  swish(out, r, 1.42, 0.16, 0.25, 500);
  strike(out, 1.58, 1, 0, [[70, 0.06, 1], [160, 0.03, 0.5]], { hz: 300, q: 0.7, tau: 0.015, amp: 1.2 });
  for (const at of [1.6, 1.65]) glass(out, at + 0.02 * r(), 0.12, 0.15, 2000 + 600 * r());
  return done(rate, [out], 0, 0.8, [3.1, 5.3, 7.1, 9.7], 0.2);
}

/**
 * A pedal bin: the pedal pressed and the steel lid flying up against its stop, the plate scraped
 * with a fork (a squeaky scratch, three strokes) and the leftovers dropping in, the lid let fall
 * shut.
 */
function binLidSound(rate: number): Stereo {
  const r = random(61);
  const out = once(rate, 2.8);
  strike(out, 0, 0.3, 0, [[900, 0.02, 1], [2300, 0.015, 0.6]], { hz: 1500, q: 0.8, tau: 0.003, amp: 1 });
  lid(out, 0.09, 0.6);
  for (let k = 0; k < 3; k++) {
    const at = 0.45 + k * 0.3;
    const start = Math.round(at * rate);
    const len = Math.round(0.22 * rate);
    const bp = new Biquad(rate).set('bandpass', 3400, 1.5);
    for (let j = 0; j < len; j++) {
      const t = j / rate;
      const stick = 0.5 + 0.5 * Math.abs(Math.sin(2 * Math.PI * (2700 + 300 * Math.sin(t * 40)) * t));
      put(out, start + j, bp.run(r() * 2 - 1) * stick * 0.5 * Math.sin((Math.PI * j) / len), 0.7, 0.7);
    }
  }
  strike(out, 0.8, 0.5, 0, [[120, 0.04, 1]], { hz: 400, q: 0.6, tau: 0.04, amp: 1 });
  lid(out, 1.75, 0.9);
  return done(rate, [out], 0, 0.8, [3.1, 5.3, 7.1, 9.7], 0.2);
}

/** A thin steel lid hitting its stop or the bin's rim: a quick metal ring. */
function lid(out: Track, at: number, amp: number): void {
  strike(out, at, amp, 0, [[620, 0.12, 0.6], [1530, 0.09, 0.5], [2780, 0.06, 0.4], [4100, 0.04, 0.25]], { hz: 2500, q: 0.6, tau: 0.003, amp: 1.2 });
}

/**
 * A snack from a vending machine: the coins in and rolling down the chute, a button, the spiral's
 * motor turning for a second and a half, the packet dropping into the tray and the flap pushed.
 */
function vendSnackSound(rate: number): Stereo {
  const r = random(62);
  const out = once(rate, 4.2);
  coins(out, r, 0);
  coins(out, r, 0.45);
  strike(out, 1.2, 0.3, 0, [[3000, 0.004, 1]], { hz: 2500, q: 1, tau: 0.002, amp: 0.8 });
  beep(out, 1.25, 0.12, 1000, 0.04);
  const start = Math.round(1.5 * rate);
  const len = Math.round(1.5 * rate);
  const whine = motor(len, rate, () => 140, [1, 0.7, 0.5, 0.3, 0.2]);
  const gears = motor(len, rate, () => 1120, [1, 0.3]);
  for (let j = 0; j < len; j++) {
    const env = ramp(0, 0.08, j / rate) * (1 - ramp(1.4, 1.5, j / rate));
    put(out, start + j, (whine[j] * 0.06 + gears[j] * 0.01) * env, 0.7, 0.7);
  }
  for (let t = 1.55; t < 2.95; t += 0.065) strike(out, t, 0.04, 0, [[1800 + 400 * r(), 0.004, 1]], { hz: 2000, q: 1, tau: 0.001, amp: 0.5 });
  strike(out, 3.05, 0.5, 0, [[110, 0.03, 1], [330, 0.02, 0.4]], { hz: 600, q: 0.6, tau: 0.008, amp: 1 });
  for (let k = 0; k < 14; k++) strike(out, 3.05 + r() * 0.15, 0.04, 0, [], { hz: 3500 + 3000 * r(), q: 1, tau: 0.001, amp: 1 });
  flap(out, 3.7);
  return done(rate, [out], 0, 0.8, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * A can from a vending machine: the coins, a button, the latch letting go and the can tumbling
 * down inside (a full can's dull clank) into the metal tray, the flap; then the can opened, a
 * crack and a hiss.
 */
function vendCanSound(rate: number): Stereo {
  const r = random(63);
  const out = once(rate, 4.8);
  coins(out, r, 0);
  coins(out, r, 0.45);
  strike(out, 1.2, 0.3, 0, [[3000, 0.004, 1]], { hz: 2500, q: 1, tau: 0.002, amp: 0.8 });
  beep(out, 1.25, 0.12, 1000, 0.04);
  strike(out, 1.55, 0.4, 0, [[400, 0.03, 1], [1900, 0.01, 0.4]], { hz: 1200, q: 0.8, tau: 0.003, amp: 1 });
  for (const [at, a] of [
    [1.7, 0.5],
    [1.86, 0.6],
    [1.98, 0.35],
  ]) strike(out, at, a, 0, [[1100, 0.04, 0.5], [2600, 0.03, 0.4], [4200, 0.02, 0.3], [180, 0.02, 0.6]], { hz: 1500, q: 0.7, tau: 0.003, amp: 1 });
  strike(out, 2.1, 0.9, 0, [[500, 0.08, 0.6], [1250, 0.05, 0.4], [180, 0.03, 0.8]], { hz: 1000, q: 0.6, tau: 0.004, amp: 1 });
  flap(out, 2.7);
  strike(out, 3.8, 0.5, 0, [[3900, 0.004, 1]], { hz: 4000, q: 0.8, tau: 0.002, amp: 2 });
  const start = Math.round(3.81 * rate);
  const len = Math.round(0.7 * rate);
  const hp = new Biquad(rate).set('bandpass', 6000, 0.8);
  for (let j = 0; j < len; j++) put(out, start + j, hp.run(r() * 2 - 1) * 0.5 * Math.exp(-j / rate / 0.12), 0.7, 0.7);
  for (let k = 0; k < 40; k++) strike(out, 3.85 + r() * 0.6, 0.015, 0, [], { hz: 5000 + 4000 * r(), q: 1, tau: 0.0008, amp: 1 });
  return done(rate, [out], 0, 0.8, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/** A coin into a machine: its clink at the slot and the rattle down the chute into the box. */
function coins(out: Track, r: () => number, at: number): void {
  strike(out, at, 0.2, 0, [[3800, 0.05, 1], [9100, 0.03, 0.5], [8500, 0.02, 0.2]], { hz: 4000, q: 1, tau: 0.001, amp: 0.6 });
  for (let k = 0; k < 5; k++) strike(out, at + 0.08 + k * 0.025 + 0.01 * r(), 0.06, 0, [[3200 + 600 * r(), 0.01, 1]], { hz: 3500, q: 1, tau: 0.001, amp: 0.5 });
  strike(out, at + 0.25, 0.15, 0, [[1800, 0.03, 1], [4300, 0.02, 0.4]], { hz: 2500, q: 0.8, tau: 0.002, amp: 0.6 });
}

/** A machine's flap pushed in and swinging back. */
function flap(out: Track, at: number): void {
  strike(out, at, 0.3, 0, [[1200, 0.02, 1], [2700, 0.012, 0.4]], { hz: 1500, q: 0.7, tau: 0.003, amp: 1 });
  strike(out, at + 0.25, 0.18, 0, [[1250, 0.02, 1], [2800, 0.012, 0.4]], { hz: 1500, q: 0.7, tau: 0.003, amp: 1 });
}

/**
 * A call from a phone box: the receiver lifted, the dial tone (350 and 440 Hz) faint from the
 * earpiece, a coin dropped in, the number pressed (each button's click and its pair of tones),
 * and the ringing at the other end (400 and 450 Hz, twice in a row, then a pause).
 */
function phoneBoxSound(rate: number): Stereo {
  const r = random(64);
  const out = once(rate, 7.5);
  strike(out, 0, 0.5, 0, [[900, 0.02, 1], [2100, 0.012, 0.5]], { hz: 1500, q: 0.7, tau: 0.003, amp: 1 });
  strike(out, 0.06, 0.2, 0, [[3200, 0.004, 1]], { hz: 3000, q: 1, tau: 0.001, amp: 0.6 });
  const earpiece = new Float32Array(out.left.length);
  tones(earpiece, rate, 0.2, 1.1, [350, 440]);
  coins(out, r, 0.65);
  const keys: [number, number][] = [
    [697, 1209],
    [770, 1336],
    [852, 1477],
    [697, 1336],
    [941, 1336],
    [770, 1209],
    [852, 1336],
  ];
  for (const [k, pair] of keys.entries()) {
    const at = 1.5 + k * 0.22 + 0.04 * r();
    strike(out, at, 0.25, 0, [[1500, 0.01, 1], [3300, 0.006, 0.4]], { hz: 2000, q: 0.8, tau: 0.002, amp: 1 });
    tones(earpiece, rate, at + 0.01, 0.12, pair);
  }
  for (const at of [3.7, 4.3, 6.7, 7.3]) tones(earpiece, rate, at, 0.4, [400, 450]);
  // The earpiece is small and held to an ear: faint, and only the telephone's band.
  const hp = new Biquad(rate).set('highpass', 300, 0.7);
  const lp = new Biquad(rate).set('lowpass', 3400, 0.7);
  for (let i = 0; i < earpiece.length; i++) put(out, i, lp.run(hp.run(earpiece[i])) * 0.05, 0.7, 0.7);
  return done(rate, [out], 0, 0.8, [2.3, 3.7, 5.1, 6.7], 0.2);
}

/** The receiver put back on its hook: a plastic clack, a bounce, and the hook's switch. */
function hangUpSound(rate: number): Stereo {
  const out = once(rate, 0.8);
  strike(out, 0.01, 0.8, 0, [[850, 0.025, 1], [2100, 0.015, 0.5], [180, 0.03, 0.6]], { hz: 1500, q: 0.7, tau: 0.004, amp: 1.2 });
  strike(out, 0.05, 0.3, 0, [[900, 0.02, 1], [2200, 0.012, 0.4]], { hz: 1500, q: 0.7, tau: 0.003, amp: 1 });
  strike(out, 0.07, 0.2, 0, [[3200, 0.004, 1]], { hz: 3000, q: 1, tau: 0.001, amp: 0.6 });
  return done(rate, [out], 0, 0.8, [2.3, 3.7, 5.1, 6.7], 0.2);
}

/** A microwave done: the timer's click and its bell's ping, then the door's latch let go and the door swung open. */
function pingSound(rate: number): Stereo {
  const out = once(rate, 2.4);
  strike(out, 0, 0.2, 0, [[2400, 0.005, 1]], { hz: 2500, q: 1, tau: 0.002, amp: 0.6 });
  strike(out, 0.03, 0.7, 0, [[2350, 0.6, 1], [5450, 0.3, 0.35], [9990, 0.15, 0.15]], { hz: 4000, q: 1, tau: 0.001, amp: 0.4 });
  strike(out, 1.4, 0.4, 0, [[1100, 0.02, 1], [2600, 0.012, 0.5]], { hz: 1800, q: 0.7, tau: 0.003, amp: 1 });
  strike(out, 1.55, 0.2, 0, [[420, 0.04, 1]], { hz: 600, q: 4, tau: 0.08, amp: 0.4 });
  return done(rate, [out], 0, 0.75, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/** Toast popping up: the spring letting go, the carriage's clank at the top (and a bounce), the slices knocking, then lifted out. */
function toastPopSound(rate: number): Stereo {
  const r = random(65);
  const out = once(rate, 1.6);
  strike(out, 0, 0.9, 0, [[1300, 0.03, 0.6], [2900, 0.02, 0.5], [4700, 0.012, 0.3], [190, 0.08, 0.25]], { hz: 2000, q: 0.7, tau: 0.003, amp: 1.2 });
  strike(out, 0.05, 0.35, 0, [[1350, 0.02, 0.6], [3000, 0.015, 0.4]], { hz: 2000, q: 0.7, tau: 0.002, amp: 1 });
  for (const at of [0.03, 0.07]) strike(out, at + 0.01 * r(), 0.2, 0, [[700, 0.01, 1]], { hz: 1000, q: 0.8, tau: 0.003, amp: 0.8 });
  rustle(out, r, 0.75, 0.2, 0.06, 0, 2500, 0.8);
  return done(rate, [out], 0, 0.75, [3.1, 5.3, 7.1, 9.7], 0.15);
}

/**
 * Coffee poured into a cup: the stream's splash, and the ring of the air left in the cup rising
 * as it fills (a quarter wave in the space above the coffee: about 950 Hz with 9 cm left, near
 * 2.4 kHz with 3.5 cm).
 */
function pourSound(rate: number): Stereo {
  const r = random(66);
  const out = once(rate, 2.6);
  const n = out.left.length;
  const splash = new Biquad(rate).set('bandpass', 2600, 0.5);
  const air = new Biquad(rate);
  const pink = new Pink(r);
  const burble = wander(r, n, rate, 0.03);
  for (let i = 0; i < n; i++) {
    const t = i / rate;
    const filled = ramp(0.1, 2.1, t);
    if ((i & 31) === 0) air.set('bandpass', 343 / (4 * (0.09 - 0.055 * filled)), 6);
    const on = ramp(0.05, 0.12, t) * (1 - ramp(2.05, 2.2, t));
    const w = pink.next();
    put(out, i, (air.run(w) * 2.2 + splash.run(w) * 0.35) * on * (1 + 0.3 * burble[i]), 0.7, 0.7);
  }
  for (let t = 0.1; t < 2.1; t += -Math.log(1 - r()) / 60) for (const ch of [out.left, out.right]) bubble(ch, rate, t, 900 + 1800 * r(), 0.03, 0.3);
  return done(rate, [out], 0, 0.7, [2.3, 3.7, 5.1, 6.7], 0.15);
}

/** A glass bottle or jar knocking another: a clink. */
function glass(out: Track, at: number, amp: number, pan: number, hz: number): void {
  strike(out, at, amp, pan, [[hz, 0.06, 1], [hz * 2.32, 0.04, 0.5], [hz * 4.25, 0.025, 0.3]], { hz: hz * 2, q: 1, tau: 0.001, amp: 0.5 });
}

/** A short electronic beep. */
function beep(out: Track, at: number, seconds: number, hz: number, amp: number): void {
  const rate = out.rate;
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    const v = (Math.sin(2 * Math.PI * hz * t) + 0.3 * Math.sin(6 * Math.PI * hz * t)) * amp * Math.min(1, j / (0.003 * rate), (len - j) / (0.003 * rate));
    put(out, start + j, v, 0.7, 0.7);
  }
}

/** A pair of tones (a telephone's), on for `seconds`. */
function tones(d: Float32Array, rate: number, at: number, seconds: number, hz: number[]): void {
  const start = Math.round(at * rate);
  const len = Math.min(d.length - start, Math.round(seconds * rate));
  for (let j = 0; j < len; j++) {
    const t = j / rate;
    let v = 0;
    for (const f of hz) v += Math.sin(2 * Math.PI * f * t);
    d[start + j] += v * Math.min(1, j / (0.005 * rate), (len - j) / (0.005 * rate));
  }
}

// Building blocks.

/**
 * The average level of the loops of separate knocks (darts, a rally): set by their average rather
 * than their peak, as their knocks are short; the sharpest are rounded off a little (`setLevel`).
 */
const SPARSE = 0.046;

/** A mode: its frequency (Hz), the time it takes to die to a third (s) and its level. */
type Mode = [hz: number, tau: number, amp: number];

/** A short burst of noise in a band (its centre in Hz, its Q), dying away over `tau` seconds, at `amp`. */
interface Burst {
  hz: number;
  q: number;
  tau: number;
  amp: number;
}

/** Tracks whose events wrap round their end into their start (`loop`). */
const wraps = new WeakSet<Track>();

/** A track whose events wrap round its end into its start (a loop of events, no overlap). */
function loop(rate: number, seconds: number): Track {
  const t = track(rate, seconds, 0);
  wraps.add(t);
  return t;
}

/** A track for a sound played once: nothing wraps. */
function once(rate: number, seconds: number): Track {
  return track(rate, seconds, 0);
}

/** Adds `v` at sample `i`, left and right at `l` and `r`; past the end it wraps on a loop of events, else it's dropped. */
function put(t: Track, i: number, v: number, l: number, r: number): void {
  const n = t.left.length;
  if (i >= n) {
    if (!wraps.has(t)) return;
    i %= n;
  }
  t.left[i] += v * l;
  t.right[i] += v * r;
}

/** Something struck: its modes ringing out over a burst of noise, from `at` (s), at `amp`, placed at `pan`. */
function strike(t: Track, at: number, amp: number, pan: number, modes: Mode[], burst?: Burst): void {
  const rate = t.rate;
  const start = Math.round(at * rate);
  const longest = Math.max(burst ? burst.tau : 0, ...modes.map((m) => m[1]));
  const len = Math.round(longest * 7 * rate);
  const [l, r] = gains(pan);
  const band = burst ? new Biquad(rate).set('bandpass', burst.hz, burst.q) : null;
  let s = (start * 2654435761 + 977) >>> 0;
  const rnd = () => (s = (s * 1664525 + 1013904223) >>> 0) / 4294967296;
  const phase = modes.map(() => rnd() * 2 * Math.PI);
  const step = modes.map(([hz]) => (2 * Math.PI * Math.min(hz, rate * 0.45)) / rate);
  const fall = modes.map(([, tau]) => Math.exp(-1 / (tau * rate)));
  const level = modes.map(([, , a]) => a);
  let noiseLevel = burst ? burst.amp : 0;
  const noiseFall = burst ? Math.exp(-1 / (burst.tau * rate)) : 0;
  const attack = 0.0004 * rate;
  for (let j = 0; j < len; j++) {
    let v = 0;
    for (let m = 0; m < modes.length; m++) {
      v += level[m] * Math.sin(phase[m] + step[m] * j);
      level[m] *= fall[m];
    }
    if (band) {
      v += band.run(rnd() * 2 - 1) * noiseLevel;
      noiseLevel *= noiseFall;
    }
    put(t, start + j, v * amp * Math.min(1, j / attack), l, r);
  }
}

/** A rustle or a rub: noise in a band (`hz`, `q`), swelling and fading over `seconds`. */
function rustle(t: Track, r: () => number, at: number, seconds: number, amp: number, pan: number, hz = 4000, q = 0.5): void {
  const rate = t.rate;
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  const [l, rr] = gains(pan);
  const bp = new Biquad(rate).set('bandpass', hz, q);
  for (let j = 0; j < len; j++) put(t, start + j, bp.run(r() * 2 - 1) * amp * (0.6 + 0.4 * r()) * Math.sin((Math.PI * j) / len), l, rr);
}

/** A rush of air (a door swung): low noise swelling and fading. */
function swish(t: Track, r: () => number, at: number, seconds: number, amp: number, hz: number): void {
  const rate = t.rate;
  const start = Math.round(at * rate);
  const len = Math.round(seconds * rate);
  const lp = new Biquad(rate).set('lowpass', hz, 0.7);
  for (let j = 0; j < len; j++) put(t, start + j, lp.run(r() * 2 - 1) * amp * Math.sin((Math.PI * j) / len) ** 2, 0.7, 0.7);
}

/** A motor's (or a fan's) tone: harmonics at `partials` levels (the fundamental first) of a speed `hz(i)` (Hz at sample `i`). */
function motor(n: number, rate: number, hz: (i: number) => number, partials: number[]): Float32Array {
  const d = new Float32Array(n);
  const top = partials.length;
  let phase = 0;
  for (let i = 0; i < n; i++) {
    const f = hz(i);
    phase += (2 * Math.PI * f) / rate;
    if (phase > 2 * Math.PI) phase -= 2 * Math.PI;
    let v = 0;
    for (let k = 0; k < top; k++) if ((k + 1) * f < rate * 0.45) v += partials[k] * Math.sin((k + 1) * phase);
    d[i] = v;
  }
  return d;
}

/** Pink noise through a filter: a rush, a hiss or a rumble. */
function noise(n: number, rate: number, r: () => number, type: 'lowpass' | 'highpass' | 'bandpass', hz: number, q: number): Float32Array {
  const d = new Float32Array(n);
  const p = new Pink(r);
  const f = new Biquad(rate).set(type, hz, q);
  for (let i = 0; i < n; i++) d[i] = f.run(p.next());
  return d;
}

/**
 * Puts the parts together: the steady ones (with an overlap) folded into their start, the events
 * added; then a room (`delays` in ms, none for out of doors, mixed in at `wet`), the highs and lows
 * taken off (`low`, `high`: a box or a small speaker), and the level set to an average of `rms`,
 * or with 0 to a peak of `peak`.
 */
function done(rate: number, parts: Track[], rms: number, peak: number, delays: number[], wet: number, low = 0, high = 0): Stereo {
  const folded = parts.map((p) => (p.overlap > 0 ? fold(p) : p));
  const n = Math.min(...folded.map((p) => p.left.length));
  const result: Stereo = { left: new Float32Array(n), right: new Float32Array(n) };
  for (const p of folded) {
    for (let i = 0; i < n; i++) {
      result.left[i] += p.left[i];
      result.right[i] += p.right[i];
    }
  }
  for (const [c, ch] of [result.left, result.right].entries()) {
    if (low > 0) {
      const lp = new Biquad(rate).set('lowpass', low, 0.7);
      for (let i = 0; i < n; i++) ch[i] = lp.run(ch[i]);
    }
    if (high > 0) {
      const hp = new Biquad(rate).set('highpass', high, 0.7);
      for (let i = 0; i < n; i++) ch[i] = hp.run(ch[i]);
    }
    if (delays.length) room(ch, rate, c ? delays.map((d) => d * 1.09) : delays, 0.55, 0.4, wet);
  }
  return setLevel(result, rms, peak || 1);
}
