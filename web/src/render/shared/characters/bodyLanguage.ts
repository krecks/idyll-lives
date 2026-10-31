/**
 * Body language (`residents.ts` `procedural`): how a resident holds themselves for the emotion the face shows
 * (`EXPRESSIONS`), a layer over whatever clip plays: the back bent or held up, the head bowed or raised, the
 * shoulders dropped or drawn up, the arms held in or hanging loose, the elbows bent, and the walk's arm swing smaller
 * or larger. How fast they walk is sim-core's (an emotion's `walkSpeed`, content `emotions`); the walk's stride
 * follows it.
 *
 * It changes only what the body is free to change: standing free or walking, all of it; carrying something, all but
 * the arms; using a thing standing, or in a hug, a fight or a cuddle, only the head (the hands stay on what they hold);
 * seated, the head and a little of the upper back; lying, nothing. A change of emotion, or of what they do, eases in
 * over about a second.
 */

import type { Additive } from './pose';
import { addRotation } from './pose';

/** How a resident holds themselves (degrees; 0 is the clips' own bearing). */
export interface Bearing {
  /** The back bent forward (+) or held up straight (-), spread over the spine. */
  stoop: number;
  /** The head bowed (+) or raised (-). */
  head: number;
  /** The shoulders dropped and rounded forward (+) or drawn up (-). */
  shoulders: number;
  /** The arms held in to the sides (+) or hanging loose, out from them (-). */
  arms: number;
  /** The elbows bent further. */
  elbows: number;
  /** The walk's arm swing: its share more (+) or less (-) than the clip's. */
  swing: number;
}
const KEYS = ['stoop', 'head', 'shoulders', 'arms', 'elbows', 'swing'] as const;
/** Floats a rig keeps for its bearing: the six of `Bearing`, then the weights of the posture, the arms and the head. */
export const BEARING_FLOATS = KEYS.length + 3;

/**
 * The bearing per emotion id (content `emotions`, as `EXPRESSIONS`); unknown ids and none keep the clips' own. Sad
 * and tired slump, the head down and the swing small; bored and dazed less so; happy, energised, playful and
 * confident stand up straight, the head up, the arms loose and swinging; angry is upright and stiff, the arms tight,
 * the elbows bent; tense and uncomfortable draw the shoulders up and hold the arms in; embarrassed hunches, the head
 * down.
 */
export const BEARINGS: Record<string, Partial<Bearing>> = {
  sad: { stoop: 8, head: 12, shoulders: 7, arms: 3, swing: -0.55 },
  tired: { stoop: 9, head: 9, shoulders: 6, swing: -0.45 },
  bored: { stoop: 5, head: 5, shoulders: 3, swing: -0.25 },
  dazed: { stoop: 4, head: 6, shoulders: 2, swing: -0.35 },
  embarrassed: { stoop: 6, head: 12, shoulders: -4, arms: 5, elbows: 6, swing: -0.35 },
  tense: { head: -2, shoulders: -7, arms: 6, elbows: 10, swing: -0.35 },
  uncomfortable: { stoop: 3, head: 4, shoulders: -5, arms: 5, elbows: 6, swing: -0.25 },
  angry: { stoop: -2, head: 4, shoulders: -4, arms: 7, elbows: 16, swing: -0.2 },
  disgusted: { stoop: -3, head: -3, shoulders: -3, arms: 3 },
  focused: { stoop: 3, head: 5, swing: -0.1 },
  relaxed: { stoop: 2, shoulders: 4, arms: -2, swing: -0.1 },
  happy: { stoop: -3, head: -2, arms: -3, swing: 0.3 },
  energized: { stoop: -4, head: -3, arms: -3, elbows: 6, swing: 0.4 },
  playful: { stoop: -2, head: -2, arms: -4, swing: 0.35 },
  confident: { stoop: -4, head: -3, shoulders: 2, arms: -2, swing: 0.2 },
  inspired: { stoop: -3, head: -3, swing: 0.15 },
  flirty: { stoop: -3, head: -2, swing: 0.15 },
  surprised: { stoop: -3, head: -4, shoulders: -4 },
};

/** What the body is free to change this frame (0..1 each): the back and shoulders, the arms, the head. */
export interface Freedom {
  posture: number;
  arms: number;
  head: number;
}

/** How quickly a bearing eases in (1/s): about a second to settle. */
const EASE = 3;
const D2R = Math.PI / 180;

/**
 * Eases `state` (a rig's `BEARING_FLOATS`) towards emotion `emotion`'s bearing and `free`, and adds the result to
 * `add`. `walkPhase`, `walkW`: the walk clip's phase (0..1, the left heel's strike at 0) and weight, for the swing
 * (0 weight when not walking). Bones `B` by name.
 */
export function bearing(state: Float32Array, emotion: string, free: Freedom, walkPhase: number, walkW: number, dt: number, add: Additive, B: Record<string, number>): void {
  const want = BEARINGS[emotion];
  const f = 1 - Math.exp(-dt * EASE);
  const n = KEYS.length;
  for (let k = 0; k < n; k++) state[k] += ((want?.[KEYS[k]] ?? 0) - state[k]) * f;
  state[n] += (free.posture - state[n]) * f;
  state[n + 1] += (free.arms - state[n + 1]) * f;
  state[n + 2] += (free.head - state[n + 2]) * f;
  const [stoop, head, shoulders, arms, elbows, swing] = [state[0], state[1], state[2], state[3], state[4], state[5]];
  const [wp, wa, wh] = [state[n], state[n + 1], state[n + 2]];
  // (Model-space turns, as `addRotation` takes them: + about x bends forward, the left side's + about z brings it down.)
  if (wp > 1e-3) {
    const s = stoop * D2R * wp;
    addRotation(add, B.spine_01, 1, 0, 0, 0.3 * s);
    addRotation(add, B.spine_02, 1, 0, 0, 0.35 * s);
    addRotation(add, B.spine_03, 1, 0, 0, 0.35 * s);
    const c = shoulders * D2R * wp;
    addRotation(add, B.clavicle_l, 0, 0, 1, c);
    addRotation(add, B.clavicle_r, 0, 0, 1, -c);
    addRotation(add, B.clavicle_l, 0, 1, 0, 0.6 * Math.max(0, c));
    addRotation(add, B.clavicle_r, 0, 1, 0, -0.6 * Math.max(0, c));
  }
  if (wa > 1e-3) {
    const a = arms * D2R * wa;
    addRotation(add, B.upperarm_l, 0, 0, 1, a);
    addRotation(add, B.upperarm_r, 0, 0, 1, -a);
    const e = elbows * D2R * wa;
    addRotation(add, B.lowerarm_l, 1, 0, 0, -e);
    addRotation(add, B.lowerarm_r, 1, 0, 0, -e);
    if (walkW > 1e-3) {
      // The walk's arms swing 17° forward and back against the legs (`WALK`), the left forward at the right heel's
      // strike: more or less of that.
      const sw = -Math.cos(2 * Math.PI * (walkPhase - 0.05)) * 17 * D2R * swing * wa * walkW;
      addRotation(add, B.upperarm_l, 1, 0, 0, -sw);
      addRotation(add, B.upperarm_r, 1, 0, 0, sw);
    }
  }
  if (wh > 1e-3) {
    const h = head * D2R * wh;
    addRotation(add, B.neck_01, 1, 0, 0, 0.4 * h);
    addRotation(add, B.Head, 1, 0, 0, 0.6 * h);
  }
}
