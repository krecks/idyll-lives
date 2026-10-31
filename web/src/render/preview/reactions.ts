/**
 * Household-creator reactions: short performances on the stage, built from the game's own clips and
 * facial expressions, with a head tilt or a slump on top for some.
 */

import { EXPRESSIONS } from '../shared/characters/expressions';
import type { Face } from '../shared/characters/face';
import { addRotation, type Additive } from '../shared/characters/pose';
import type { StageReaction } from '../types';

export interface Performance {
  /** Clip played instead of the idle (one-shots play once); none = idle. */
  clip?: string;
  /** Seconds a looping clip (or a clip-less reaction) lasts; one-shots last their clip. */
  seconds?: number;
  face?: Partial<Face>;
  /** A wave at the viewer (the `wave` clip): the Sim turns to the front for it. */
  wave?: boolean;
  /** Eyes on the camera (default true). */
  eyeContact?: boolean;
  /** Finger pose while performing (1 = open hand). */
  hands?: number;
  /** A small head tilt (flirting). */
  tilt?: number;
  /** Shoulders and head dropping (sadness, shyness). */
  slump?: number;
}

const SMILE: Partial<Face> = { smile: 0.75, wide: 0.35, lidL: 0.4, browUp: 0.2 };
const GRIN: Partial<Face> = { ...EXPRESSIONS.happy, jaw: 0.07 };

/** Per trait id (the base content's traits); other traits get `FRIENDLY`. */
const TRAITS: Record<string, Performance> = {
  foodie: { clip: 'eat', face: SMILE, hands: 0.35 },
  bookworm: { clip: 'lantern', seconds: 2.2, face: { ...EXPRESSIONS.focused, smile: 0.3 }, eyeContact: false, hands: 0.35 },
  couchPotato: { clip: 'foldArms', seconds: 2.4, face: EXPRESSIONS.relaxed },
  neat: { clip: 'yes', face: { ...EXPRESSIONS.confident, smile: 0.5 } },
  slob: { clip: 'pickUp', face: { ...EXPRESSIONS.playful, lidU: 0.25 }, hands: 0.3 },
  energetic: { clip: 'jog', seconds: 1.8, face: EXPRESSIONS.energized, hands: 0.1 },
  lazy: { clip: 'foldArms', seconds: 2.2, face: { jaw: 0.55, lidU: 0.55, browUp: 0.3, smile: 0 }, eyeContact: false },
  cheerful: { clip: 'wave', wave: true, face: GRIN },
  gloomy: { seconds: 2.6, face: EXPRESSIONS.sad, eyeContact: false, slump: 1 },
  natureLover: { clip: 'water', seconds: 2.6, face: EXPRESSIONS.relaxed, eyeContact: false, hands: 0.35 },
  outgoing: { clip: 'wave', wave: true, face: EXPRESSIONS.playful },
  loner: { seconds: 2.4, face: EXPRESSIONS.embarrassed, eyeContact: false, slump: 0.6, tilt: -0.08 },
  romantic: { seconds: 2.4, face: EXPRESSIONS.flirty, tilt: 0.16 },
  hotHeaded: { clip: 'punchJab', face: EXPRESSIONS.angry, hands: 0.1 },
  kind: { clip: 'yes', face: { ...SMILE, browIn: 0.25 } },
};
const FRIENDLY: Performance = { clip: 'yes', face: SMILE };

export function reactionFor(reaction: StageReaction): Performance {
  switch (reaction) {
    case 'hello':
      return { clip: 'wave', wave: true, face: SMILE };
    case 'cheer':
      return { clip: 'wave', wave: true, face: GRIN };
    case 'admire':
      return { seconds: 1.4, face: { ...SMILE, browUp: 0.35 } };
    default:
      return TRAITS[reaction.slice('trait:'.length)] ?? FRIENDLY;
  }
}

/** Head tilt (radians, eased in and out over `seconds`). */
export function tilt(add: Additive, bone: Record<string, number>, t: number, seconds: number, amount: number): void {
  const k = smoothstep(0, 0.4, t) * (1 - smoothstep(seconds - 0.4, seconds, t));
  if (k <= 0) return;
  addRotation(add, bone.Head, 0, 0, 1, amount * k + Math.sin(t * 1.7) * 0.03 * k);
  addRotation(add, bone.pelvis, 0, 0, 1, Math.sin(t * 1.3) * 0.04 * k);
}

/** Shoulders round and the head drops, eased in and out over `seconds`. */
export function slump(add: Additive, bone: Record<string, number>, t: number, seconds: number, amount: number): void {
  const k = smoothstep(0, 0.5, t) * (1 - smoothstep(seconds - 0.5, seconds, t)) * amount;
  if (k <= 0) return;
  addRotation(add, bone.spine_02, 1, 0, 0, 0.12 * k);
  addRotation(add, bone.spine_03, 1, 0, 0, 0.1 * k);
  addRotation(add, bone.neck_01, 1, 0, 0, 0.16 * k);
  addRotation(add, bone.Head, 1, 0, 0, 0.2 * k);
  addRotation(add, bone.clavicle_l, 1, 0, 0, 0.12 * k);
  addRotation(add, bone.clavicle_r, 1, 0, 0, 0.12 * k);
}

function smoothstep(a: number, b: number, x: number): number {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
}
