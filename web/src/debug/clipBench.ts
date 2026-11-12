/**
 * How the dev pages play a model's clips the way the game does (`clips.ts`, `inspect.ts`): which kind
 * each clip is by the model entry's `motions`, how long its turn takes, and where it is at a time into
 * the bench. Not part of the build.
 *
 * - a use's clip (`uses`, `open`, `night`, and clips nothing plays): at rest (and its `delay`), opening,
 *   held open, closing again (played backwards);
 * - a use's clip that repeats (`every`): at rest (and its `delay`), then from rest to rest twice, `every`
 *   seconds apart;
 * - a loop (`loops`, `running`): at rest, then round after round, stopping at rest;
 * - the ambient clip (`idle`): round after round, always, alongside the others;
 * - a clock's hands (`clock`): always, an hour of the game's time a second;
 * - the growth clip (`growth`): each stage in turn (planted, growing, ripe, picked), moving on to the
 *   next at the clip's pace, as the game does (`GROWTH_TIMES`).
 */

import type { ModelEntry } from '../assets/types';
import type { ModelClip } from '../render/shared/modelClips';
import { GROWTH_TIMES } from '../render/shared/objectMotion';

/** How a clip plays (see the file header). */
export type Kind = 'use' | 'repeat' | 'loop' | 'idle' | 'growth' | 'clock';

/** Seconds at rest before a clip plays, held open (a use), and running (a loop's rounds, at least two). */
export const REST = 0.9;
export const HOLD = 1.6;
export const RUNNING = 2.4;
/** How long each growth stage shows (planted, growing, ripe, picked), moving on from the one before included. */
export const STAGE = 1.6;

/** The entry's `motions` as written (read loosely: a bench shows what's there). */
interface RawMotions {
  uses?: Record<string, RawUse | RawUse[]>;
  open?: string;
  loops?: Record<string, string>;
  idle?: string;
  night?: string;
  growth?: string;
  running?: string;
  clock?: { hour?: string; minute?: string; at?: number };
}
type RawUse = string | { clip: string; delay?: number; every?: number };

/** How a clip the entry names plays (and a use's `delay` and `every`, a clock hand's `period` and `at`). */
export interface Playing {
  kind: Kind;
  delay?: number;
  every?: number;
  period?: number;
  at?: number;
}

/** How each clip the entry names plays; clips it doesn't name play as a use's. */
export function kinds(entry: ModelEntry | undefined): Map<string, Playing> {
  const m = (entry as { motions?: RawMotions } | undefined)?.motions ?? {};
  const out = new Map<string, Playing>();
  for (const value of Object.values(m.uses ?? {})) {
    for (const u of Array.isArray(value) ? value : [value]) {
      if (typeof u !== 'string' && (u.delay || u.every)) out.set(u.clip, { kind: u.every ? 'repeat' : 'use', delay: u.delay, every: u.every });
    }
  }
  for (const clip of Object.values(m.loops ?? {})) out.set(clip, { kind: 'loop' });
  if (m.running) out.set(m.running, { kind: 'loop' });
  if (m.idle) out.set(m.idle, { kind: 'idle' });
  if (m.growth) out.set(m.growth, { kind: 'growth' });
  if (m.clock?.hour) out.set(m.clock.hour, { kind: 'clock', period: 720, at: m.clock.at });
  if (m.clock?.minute) out.set(m.clock.minute, { kind: 'clock', period: 60, at: m.clock.at });
  return out;
}

/** A clip as a bench shows it. */
export interface Shown extends Playing {
  name: string;
  clip: ModelClip;
}

/** A model's clips (`template.motion.clips`) as the entry says they play, in the file's order. */
export function shownClips(clips: Map<string, ModelClip> | undefined, entry: ModelEntry | undefined): Shown[] {
  const of = kinds(entry);
  return [...(clips ?? [])].map(([name, clip]) => ({ name, kind: 'use' as Kind, ...of.get(name), clip }));
}

/** Seconds one clip's turn takes. */
export function turnLength({ kind, clip, delay = 0, every = 0 }: Shown): number {
  const d = clip.duration;
  if (kind === 'repeat') return REST + delay + every + Math.max(d, every);
  if (kind === 'loop') return REST + d * Math.max(2, Math.ceil(RUNNING / Math.max(d, 1e-3)));
  if (kind === 'growth') return GROWTH_TIMES.length * STAGE;
  return REST + delay + d + HOLD + d;
}

/** How far into its clip a turn is at `t` seconds into it. */
export function clipTime({ kind, clip, delay = 0, every = 0 }: Shown, t: number): number {
  const d = clip.duration;
  if (kind === 'growth') {
    const n = GROWTH_TIMES.length;
    const stage = Math.min(Math.floor(t / STAGE), n - 1);
    const from = GROWTH_TIMES[(stage + n - 1) % n] * d;
    const to = GROWTH_TIMES[stage] * d;
    return from + Math.sign(to - from) * Math.min(Math.abs(to - from), t - stage * STAGE);
  }
  if (t < REST + delay) return 0;
  t -= REST + delay;
  if (kind === 'repeat') return t % Math.max(every, d) < d ? t % Math.max(every, d) : 0;
  if (kind === 'loop') return d > 0 ? t % d : 0;
  if (t < d) return t;
  if (t < d + HOLD) return d;
  return Math.max(0, 2 * d + HOLD - t);
}

/** Seconds every clip that takes a turn (not the ambient one or a clock's hands) takes, one after another. */
export function cycleLength(clips: readonly Shown[]): number {
  return clips.filter((c) => c.kind !== 'idle' && c.kind !== 'clock').reduce((sum, c) => sum + turnLength(c), 0);
}

/**
 * Where each of `clips` is at `seconds` into the bench (by clip name), and the one taking its turn:
 * only `solo` if it isn't null; every clip held at `hold` seconds (at most its length) if that isn't null.
 */
export function times(clips: readonly Shown[], seconds: number, solo: string | null, hold: number | null): { at: Map<string, number>; turn: string | null } {
  const at = new Map<string, number>();
  const playing = clips.filter((c) => solo === null || c.name === solo);
  if (hold !== null) {
    for (const c of playing) at.set(c.name, Math.min(hold, c.clip.duration));
    return { at, turn: playing.length === 1 ? playing[0].name : null };
  }
  for (const c of playing) if (c.kind === 'idle') at.set(c.name, c.clip.duration > 0 ? seconds % c.clip.duration : 0);
  // An hour of the game's time a second.
  for (const c of playing) if (c.kind === 'clock') at.set(c.name, (((((seconds * 60 - (c.at ?? 0)) / c.period!) % 1) + 1) % 1) * c.clip.duration);
  const turns = playing.filter((c) => c.kind !== 'idle' && c.kind !== 'clock');
  const cycle = cycleLength(turns);
  let t = cycle > 0 ? seconds % cycle : 0;
  for (const c of turns) {
    const length = turnLength(c);
    if (t < length) {
      at.set(c.name, clipTime(c, t));
      return { at, turn: c.name };
    }
    t -= length;
  }
  return { at, turn: null };
}
