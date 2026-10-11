/**
 * Dancing and fighting: a bouncing dance, the jab and the cross (heavy bag, fights) and the hit
 * recoil. Fighting clips start and end in a guard: fists up by the chin, the left foot forward,
 * knees soft, so the runtime can chain them.
 */

import { clip, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { stand, TAU } from './idle';
import { armIK, wrist, type ArmOptions } from './ik';

type Hand = ArmOptions & { at: Vec };

function hands(body: PoseSpec, l: Hand, r: Hand): PoseSpec {
  return { ...body, ...armIK('l', l.at, body, { pole: [-0.4, -1, -0.2], ...l }), ...armIK('r', r.at, body, { pole: [0.4, -1, -0.2], ...r }) };
}

interface Fight {
  /** Turn of the hips and chest (deg, + to the left: the right shoulder forward), lean (deg), pelvis offset. */
  turn: number;
  lean: number;
  fwd: number;
  sink: number;
  head: Vec;
  l: Vec;
  r: Vec;
  /** Fist roll (deg, + palm down) per hand. */
  rollL?: number;
  rollR?: number;
}

const GUARD: Fight = { turn: -10, lean: 6, fwd: 0, sink: 0.03, head: [6, 8, 0], l: [-0.09, 1.38, 0.3], r: [0.11, 1.35, 0.22] };

/** A fighting stance pose: left foot forward, right back, both planted. */
function fight(f: Fight): PoseSpec {
  const body = stand({ sink: f.sink, fwd: f.fwd, lean: f.lean, twist: f.turn, head: f.head, shift: 0.15, feet: [0.02, 0, 0.13], footR: [0.03, 0, -0.13] });
  return hands(body, { at: f.l, pronate: f.rollL ?? 30, hand: wrist('l', 15, 0) }, { at: f.r, pronate: f.rollR ?? 30, hand: wrist('r', 15, 0) });
}

const mixF = (a: Fight, b: Partial<Fight>): Fight => ({ ...a, ...b });

/** Dancing: two bounces a beat, hips swaying, arms pumping in turn, the head bopping a touch late. */
function dancePose(p: number): PoseSpec {
  const beat = Math.abs(Math.sin(p * TAU));
  const s = Math.sin(p * TAU);
  const c = Math.cos(p * TAU);
  const body = stand({ sink: 0.015 + 0.06 * (1 - beat), shift: 0.9 * s, twist: 12 * s, lean: 4, head: [4 + 5 * Math.abs(Math.sin(p * TAU - 0.4)), 8 * s, -5 * s] });
  const up = (k: number) => ((1 + k) / 2) ** 2;
  return hands(
    body,
    { at: [-0.22 + 0.04 * c, 1.08 + 0.48 * up(s), 0.2 + 0.1 * up(s)], pole: [-0.8, -1, -0.2], pronate: 20, hand: wrist('l', 10, 0) },
    { at: [0.22 + 0.04 * c, 1.08 + 0.48 * up(-s), 0.2 + 0.1 * up(-s)], pole: [0.8, -1, -0.2], pronate: 20, hand: wrist('r', 10, 0) },
  );
}

export function actionClips(): ClipDef[] {
  return [
    cycle('dance', { seconds: 1, loop: true }, dancePose),
    // The jab: a small dip, the lead fist snaps straight out with the shoulder behind it, and back.
    clip('punchJab', { seconds: 26 / 30, loop: false }, [
      [0, () => fight(GUARD)],
      [0.14, () => fight(mixF(GUARD, { sink: 0.045, l: [-0.06, 1.38, 0.2] })), 'smooth'],
      [0.32, () => fight(mixF(GUARD, { turn: -24, lean: 9, fwd: 0.04, l: [0.01, 1.42, 0.62], rollL: 85, head: [6, 16, 0] })), 'out'],
      [0.44, () => fight(mixF(GUARD, { turn: -22, lean: 8, fwd: 0.035, l: [0.0, 1.42, 0.58], rollL: 85, head: [6, 15, 0] })), 'smooth'],
      [0.72, () => fight(mixF(GUARD, { turn: -8 })), 'smooth'],
      [1, () => fight(GUARD), 'smooth'],
    ]),
    // The cross: the hips and chest turn the rear fist through, the back heel lifts with it.
    clip('punchCross', { seconds: 1, loop: false }, [
      [0, () => fight(GUARD)],
      [0.16, () => fight(mixF(GUARD, { turn: -20, sink: 0.045, r: [0.12, 1.36, 0.1] })), 'smooth'],
      [0.36, () => fight(mixF(GUARD, { turn: 22, lean: 12, fwd: 0.05, r: [0.0, 1.42, 0.64], rollR: 85, l: [-0.14, 1.4, 0.16], head: [8, -8, 0] })), 'out'],
      [0.48, () => fight(mixF(GUARD, { turn: 20, lean: 11, fwd: 0.045, r: [0.0, 1.41, 0.6], rollR: 85, l: [-0.14, 1.4, 0.16], head: [8, -7, 0] })), 'smooth'],
      [0.78, () => fight(mixF(GUARD, { turn: -4 })), 'smooth'],
      [1, () => fight(GUARD), 'smooth'],
    ]),
    // Hit: the head snaps back and aside, the chest folds back, the guard comes up late.
    clip('hit', { seconds: 10 / 30, loop: false }, [
      [0, () => fight(GUARD)],
      [0.3, () => fight(mixF(GUARD, { lean: -14, fwd: -0.05, sink: 0.05, turn: 4, head: [-16, -14, 8], l: [-0.1, 1.5, 0.2], r: [0.1, 1.46, 0.14] })), 'out'],
      [1, () => fight(mixF(GUARD, { lean: 0, fwd: -0.02, head: [4, 6, 2] })), 'smooth'],
    ]),
  ];
}
