/**
 * The residents' faces for their emotions and for how a social was taken, the resting face, the standing stance,
 * and the stage directions the character creator and the portraits give a resident row (`residents.ts`).
 */

import type { Face } from './face';
import type { Additive } from './pose';

/** Expression per emotion id (content `emotions`, packs included); unknown ids stay neutral. */
export const EXPRESSIONS: Record<string, Partial<Face>> = {
  happy: { mood: 'happy' },
  flirty: { mood: 'flirty' },
  confident: { mood: 'confident' },
  focused: { mood: 'focused', gazeDown: 0.15 },
  inspired: { mood: 'inspired' },
  energized: { mood: 'energized' },
  playful: { mood: 'playful' },
  relaxed: { mood: 'relaxed' },
  embarrassed: { mood: 'embarrassed', gazeDown: 0.6 },
  sad: { mood: 'sad', gazeDown: 0.45 },
  tense: { mood: 'tense' },
  uncomfortable: { mood: 'uncomfortable' },
  bored: { mood: 'bored', gazeDown: 0.2 },
  angry: { mood: 'angry' },
  dazed: { mood: 'dazed' },
  surprised: { mood: 'surprised' },
  tired: { mood: 'tired', gazeDown: 0.3 },
  disgusted: { mood: 'disgusted' },
};
/**
 * The resting face every expression starts from: a soft closed-lip smile, relaxed brows lifted a
 * touch with soft inner ends, the lower lids a little raised as in a real smile. Scaled by mood
 * (gone when a Sim is very unhappy) and left out under emotions that frown or knit the brows, which
 * define the face themselves; the face returns to it when they pass.
 */
export const FRIENDLY: Partial<Face> = { smile: 0.35, lidL: 0.15, browUp: 0.15, browIn: 0.1 };
/** Emotions that set the whole face themselves (no resting smile under them). */
export const COLD = new Set(['focused', 'embarrassed', 'sad', 'tense', 'uncomfortable', 'bored', 'angry', 'dazed', 'tired', 'disgusted']);
/** Mood (0..1) below which the resting smile fades out, and where it is full. */
export const MOOD_COLD = 0.2;
export const MOOD_WARM = 0.55;
/**
 * Relaxed standing: the clips stand that way themselves (feet under the hips, the weight drifting
 * from leg to leg, arms hanging loose). Over a skirt the hanging arms move out a little (radians,
 * model space; faded in on the clips below, see `stance`).
 */
export const STANCE = { skirt: -0.22 };
/** Clips whose arms hang at the sides. */
export const RELAX_ARMS = new Set(['idle', 'yes', 'no']);
export const ASLEEP: Partial<Face> = { mood: 'asleep', smile: 0 };
export const LAUGH: Partial<Face> = { mood: 'laugh', smile: 0, frown: 0, browDown: 0, press: 0 };
export const KISS: Partial<Face> = { mood: 'kiss', smile: 0, frown: 0, browDown: 0, press: 0 };
export const HUG: Partial<Face> = { mood: 'hug', smile: 0, frown: 0, browDown: 0, press: 0 };
export const NONE: Partial<Face> = {};
/** Faces for how a social was taken, by the clip it's shown with (`GESTURES`, `REFUSED`); a laugh is `LAUGH`. */
export const SOCIAL_FACES: Record<string, Partial<Face>> = {
  refuse: { mood: 'uncomfortable', smile: 0 },
  rebuffed: { mood: 'sad', smile: 0, gazeDown: 0.5 },
  shrug: { mood: 'embarrassed', smile: 0 },
  foldArms: { mood: 'bored', smile: 0 },
};
/**
 * Stage directions for one Sim row (character creator and portraits; never set in game): a clip to
 * play instead of the activity's, an expression on top of the emotion, a point to look at, an
 * extra procedural layer, and `still` (portraits: the clip held at `time`, no blinks or glances,
 * no smoothing). A directed Sim turns to the snapshot's yaw at once (the director smooths it).
 */
export interface StageDirection {
  clip?: string | null;
  /** With `still`: the clip time (s) to hold. */
  time?: number;
  /** Finger pose (0 = the clip's grip, 1 = open). */
  hands?: number;
  face?: Partial<Face> | null;
  /** World point the eyes (and partly the head) turn to. */
  lookAt?: { x: number; y: number; z: number } | null;
  /** Extra model-space rotations after the built-in layers (gestures). */
  pose?: ((add: Additive, bone: Record<string, number>, time: number) => void) | null;
  still?: boolean;
}
