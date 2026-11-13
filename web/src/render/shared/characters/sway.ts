/**
 * Hair and hems move after the body (secondary motion), with no bones of their own: per resident two lag vectors,
 * one for the hair and one for the hems (a skirt's, a jacket's, an apron's), each a point on a damped spring towed
 * by the head or by the hips. The residents' code steps them every frame a resident is posed in full
 * (`residents.ts` `writeSway`) and writes them into the resident's row of the pose texture (`layout.ts`
 * `SWAY_TEXEL`); the character shader moves every loose vertex by the vector times its looseness
 * (`shaders/character.ts` `gameSimSway`: the hair's is baked, tone.x, by hair.rs; a hem's is its distance from the
 * hem, the shells' fields.x), never towards the body's axis.
 *
 * The model, per point: the offset `e` (m, world axes) of a loose end from where the body carries it obeys
 * `e'' + 2 z w e' + w^2 (e - e0) = -a''`, with `a` the towing point (a place fixed to the head or the hips, so a
 * nod or a turn of the head swings the hair too) and `e0 = -drag a'` (the air: a runner's hair streams back). The
 * towing point is known once a frame, so it moves in straight lines between frames: its acceleration is an impulse
 * at each frame (`e'` jumps by the change of its velocity) and between two the spring swings free, which has a
 * closed form. Nothing is integrated in steps, so the motion is the same at any frame rate (the towing point's
 * path sampled finer or coarser is the only difference), and a long frame cannot blow it up.
 *
 * Still when the resident is: at rest `e` decays to nothing, and what is drawn has a dead band (`SWAY.dead`), so
 * breathing and the idle's small shifts move nothing and a still resident's row stays as it was (no upload).
 */

/** A point's spring. */
export interface SwaySpring {
  /** Its own swing, per second. */
  readonly hz: number;
  /** Its damping, of critical (under 1: it swings past and back). */
  readonly damping: number;
  /** How far it trails per m/s of the towing point's speed (s). */
  readonly drag: number;
  /** The furthest it is drawn from rest at looseness 1 (m, on a grown body). */
  readonly max: number;
  /** How much of the vertical lag is kept (hair bounces less than it swings), and the furthest down (m). */
  readonly lift: number;
  readonly down: number;
}

export const SWAY = {
  /** Long hair's tips: a slow swing that takes a second and a half to settle. */
  hair: { hz: 1.9, damping: 0.3, drag: 0.02, max: 0.13, lift: 0.6, down: 0.012 } as SwaySpring,
  /** A skirt's hem: quicker and sooner at rest (cloth on the legs). */
  hem: { hz: 2.4, damping: 0.42, drag: 0.012, max: 0.07, lift: 0.3, down: 0.004 } as SwaySpring,
  /** A towing point faster than this (m/s) was not moved but put somewhere else: the point starts again at rest. */
  snap: 15,
  /** Offsets under `dead[0]` (m) draw nothing, from `dead[1]` in full. */
  dead: [0.0015, 0.005] as readonly [number, number],
  /** With the towing point still, an offset and a speed under these (m, m/s) are none: the spring is at rest. */
  rest: [0.0002, 0.002] as readonly [number, number],
  /** Where the hair is towed from, in the head's frame from its joint (m on a grown body: behind the neck). */
  hairAt: [0, -0.1, -0.1] as readonly [number, number, number],
  /** Where the hems are towed from, in the pelvis' frame from its joint (between the thighs). */
  hemAt: [0, -0.3, 0] as readonly [number, number, number],
  /** How upright the head or the hips must be for any (their up axis' y: none under the first, all from the second). */
  upright: [0.5, 0.85] as readonly [number, number],
  /** Where on a shell hems are loose: below the pelvis joint plus this (m, at rest on a grown body). */
  hemBelow: 0.08,
} as const;

/**
 * Floats of a point's state: the offset, its velocity, the towing point, the towing point's velocity, and what of
 * them is known (0 nothing, 1 the towing point, 2 its velocity too).
 */
export const SWAY_FLOATS = 13;

/** The point at `o` forgets where it was: at rest wherever it is towed from next. */
export function swayReset(state: Float32Array, o: number): void {
  state.fill(0, o, o + SWAY_FLOATS);
}

/**
 * Steps the point at `state[o..]` by `dt` seconds, towed from (ax, ay, az) now, and writes what is drawn of it,
 * times `k`, to `out[at..at + 2]` (m, world axes). `dt` 0 (paused) leaves everything as it is. Returns whether
 * anything is drawn.
 */
export function swayStep(state: Float32Array, o: number, ax: number, ay: number, az: number, dt: number, s: SwaySpring, k: number, out: Float32Array, at: number): boolean {
  if (dt <= 0 && state[o + 12]) return out[at] !== 0 || out[at + 1] !== 0 || out[at + 2] !== 0;
  if (!state[o + 12] || dt <= 0) {
    swayReset(state, o);
    state[o + 12] = 1;
  } else {
    const vx = (ax - state[o + 6]) / dt;
    const vy = (ay - state[o + 7]) / dt;
    const vz = (az - state[o + 8]) / dt;
    if (vx * vx + vy * vy + vz * vz > SWAY.snap * SWAY.snap) {
      swayReset(state, o);
      state[o + 12] = 1;
    } else if (state[o + 12] < 2) {
      // (Its first velocity: come into view walking, the hair is not flung back.)
      state[o + 9] = vx;
      state[o + 10] = vy;
      state[o + 11] = vz;
      state[o + 12] = 2;
    } else {
      const w = 2 * Math.PI * s.hz;
      const sigma = s.damping * w;
      const wd = w * Math.sqrt(1 - s.damping * s.damping);
      const decay = Math.exp(-sigma * dt);
      const cos = Math.cos(wd * dt);
      const sin = Math.sin(wd * dt);
      for (let c = 0; c < 3; c++) {
        const v = c === 0 ? vx : c === 1 ? vy : vz;
        // The towing point's change of velocity, as an impulse; then the free swing about where the air holds it.
        const e0 = -s.drag * v;
        const u0 = state[o + c] - e0;
        const du0 = state[o + 3 + c] - (v - state[o + 9 + c]);
        const b = (du0 + sigma * u0) / wd;
        state[o + c] = decay * (u0 * cos + b * sin) + e0;
        state[o + 3 + c] = decay * (du0 * cos - (sigma * b + wd * u0) * sin);
        state[o + 9 + c] = v;
      }
      if (vx === 0 && vy === 0 && vz === 0) {
        const e2 = state[o] * state[o] + state[o + 1] * state[o + 1] + state[o + 2] * state[o + 2];
        const v2 = state[o + 3] * state[o + 3] + state[o + 4] * state[o + 4] + state[o + 5] * state[o + 5];
        if (e2 < SWAY.rest[0] * SWAY.rest[0] && v2 < SWAY.rest[1] * SWAY.rest[1]) state.fill(0, o, o + 6);
      }
    }
  }
  state[o + 6] = ax;
  state[o + 7] = ay;
  state[o + 8] = az;
  // What is drawn: the vertical lag in part and hardly any down, no further than `max`, nothing inside the dead band.
  const x = state[o];
  const y = Math.max(state[o + 1] * s.lift, -s.down);
  const z = state[o + 2];
  const len = Math.hypot(x, y, z);
  const t = Math.min(1, Math.max(0, (len - SWAY.dead[0]) / (SWAY.dead[1] - SWAY.dead[0])));
  const f = len > 0 ? (k * t * t * (3 - 2 * t) * Math.min(len, s.max)) / len : 0;
  out[at] = f > 0 ? x * f : 0;
  out[at + 1] = f > 0 ? y * f : 0;
  out[at + 2] = f > 0 ? z * f : 0;
  return f > 0;
}
