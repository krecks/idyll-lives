/**
 * What residents say, per language (`content/voice/<lang>.json`), keyed by content ids: social
 * interactions, objects and their interactions, interaction ids, animation tags, emotions and
 * planner thought kinds. Content packs add or replace lines under the same keys (`voice` in a
 * pack). The line for an event is picked with a hash, never the simulation's random numbers,
 * so voices can't change the game.
 */

import { mergeDeep } from '../content/content';
import { hashUnit } from './voices';
import type { LineRequest } from './writer';

/** Lines, alone or with a tone for all of them. */
type Group = string[] | { lines: string[]; tone?: string };

/** Lines for one object: for any of its interactions (`lines`), and per interaction id. */
interface ObjectLines {
  lines?: string[];
  tone?: string;
  interactions?: Record<string, Group>;
}

interface LinesFile {
  social: Record<string, { start: string[]; good: string[]; bad: string[]; tone?: string }>;
  object?: Record<string, ObjectLines>;
  interaction?: Record<string, Group>;
  action: Record<string, Group>;
  emotion: Record<string, Group>;
  thought: Record<string, Group>;
}

export interface Line {
  text: string;
  tone?: string;
  /** Where it came from, e.g. `object:fridge:gourmet` (for debugging). */
  key: string;
}

/** What a resident is using, by content id. */
export interface Use {
  object?: string;
  interaction?: string;
}

/** Planner thought kinds in snapshot order (`thought` 1..10). */
const THOUGHTS = ['skipped', 'noPlace', 'kept', 'goal', 'roomLoved', 'roomDisliked', 'broken', 'accident', 'crying', 'dinnerReady'] as const;

/** A planner thought kind's id (snapshot `thought`, 1..). */
export function thoughtId(kind: number): string | undefined {
  return THOUGHTS[kind - 1];
}
/** Lines a resident said lately per key, not picked again while others are left. */
const RECENT = 3;
/** Forget everyone's recent lines past this many keys (a long session with many residents). */
const MAX_RECENT_KEYS = 4096;

const files = new Map<string, Promise<LinesFile | null>>();

export function loadLines(language: string): Promise<LinesFile | null> {
  let file = files.get(language);
  if (!file) {
    file = fetch(`${import.meta.env.BASE_URL}content/voice/${language}.json`)
      .then((r) => (r.ok ? (r.json() as Promise<LinesFile>) : null))
      .catch(() => null);
    files.set(language, file);
  }
  return file;
}

export class Lines {
  private readonly data: LinesFile;
  /** Recently said line indices per resident and key, newest last. */
  private readonly recent = new Map<string, number[]>();

  /** `packs`: the same layout from content packs (`Content.voice[language]`), laid over the file. */
  constructor(file: LinesFile, packs?: Record<string, unknown>) {
    this.data = packs ? (mergeDeep(file as unknown as Record<string, unknown>, packs) as unknown as LinesFile) : file;
  }

  social(who: number, id: string, part: 'start' | 'good' | 'bad', seed: number): Line | null {
    const s = this.data.social[id];
    return s ? this.pick(who, `social:${id}:${part}`, s[part], part === 'bad' && s.tone === 'happy' ? undefined : s.tone, seed) : null;
  }

  /**
   * A line for using something, the most specific first: this object's lines for this
   * interaction, the object's lines, lines for the interaction id on any object, lines for the
   * animation tag.
   */
  use(who: number, tag: string | undefined, use: Use, seed: number): Line | null {
    const { object, interaction } = use;
    const o = object ? this.data.object?.[object] : undefined;
    if (o && interaction) {
      const line = this.group(who, `object:${object}:${interaction}`, o.interactions?.[interaction], seed);
      if (line) return line;
    }
    if (o) {
      const line = this.pick(who, `object:${object}`, o.lines, o.tone, seed);
      if (line) return line;
    }
    if (interaction) {
      const line = this.group(who, `interaction:${interaction}`, this.data.interaction?.[interaction], seed);
      if (line) return line;
    }
    return tag ? this.group(who, `action:${tag}`, this.data.action[tag], seed) : null;
  }

  emotion(who: number, id: string, seed: number): Line | null {
    return this.group(who, `emotion:${id}`, this.data.emotion[id], seed);
  }

  thought(who: number, kind: number, seed: number): Line | null {
    const id = THOUGHTS[kind - 1];
    return id ? this.group(who, `thought:${id}`, this.data.thought[id], seed) : null;
  }

  /**
   * The lines a request would pick from and their tone, without picking (examples for the
   * language model): the same, most specific first, as `social`, `use`, `emotion`, `thought`.
   */
  candidates(req: LineRequest): { lines: string[]; tone?: string; key: string } | null {
    const of = (key: string, g: Group | undefined) => (!g ? null : Array.isArray(g) ? { lines: g, key } : { lines: g.lines, tone: g.tone, key });
    const found = (c: { lines: string[]; tone?: string; key: string } | null) => (c?.lines?.length ? c : null);
    switch (req.kind) {
      case 'social': {
        const s = req.social ? this.data.social[req.social] : undefined;
        if (!s || !req.phase) return null;
        return found({ lines: s[req.phase], tone: req.phase === 'bad' && s.tone === 'happy' ? undefined : s.tone, key: `social:${req.social}:${req.phase}` });
      }
      case 'use': {
        const { object, interaction, tag } = req;
        const o = object ? this.data.object?.[object] : undefined;
        return (
          (o && interaction ? found(of(`object:${object}:${interaction}`, o.interactions?.[interaction])) : null) ??
          (o ? found({ lines: o.lines ?? [], tone: o.tone, key: `object:${object}` }) : null) ??
          (interaction ? found(of(`interaction:${interaction}`, this.data.interaction?.[interaction])) : null) ??
          (tag ? found(of(`action:${tag}`, this.data.action[tag])) : null)
        );
      }
      case 'emotion':
        return req.emotion ? found(of(`emotion:${req.emotion}`, this.data.emotion[req.emotion])) : null;
      case 'thought': {
        const id = req.thought ? THOUGHTS[req.thought - 1] : undefined;
        return id ? found(of(`thought:${id}`, this.data.thought[id])) : null;
      }
    }
  }

  private group(who: number, key: string, g: Group | undefined, seed: number): Line | null {
    if (!g) return null;
    return Array.isArray(g) ? this.pick(who, key, g, undefined, seed) : this.pick(who, key, g.lines, g.tone, seed);
  }

  /** A line by hash, skipping the ones this resident said last under this key. */
  private pick(who: number, key: string, lines: string[] | undefined, tone: string | undefined, seed: number): Line | null {
    if (!lines?.length) return null;
    const n = lines.length;
    const memo = `${who}:${key}`;
    const recent = this.recent.get(memo) ?? [];
    let at = Math.floor(hashUnit(seed, 7) * n);
    for (let i = 0; i < n && recent.includes(at); i++) at = (at + 1) % n;
    recent.push(at);
    // Remember fewer than there are lines, so there is always another one to pick.
    while (recent.length > Math.min(RECENT, n - 1)) recent.shift();
    if (!this.recent.has(memo) && this.recent.size >= MAX_RECENT_KEYS) this.recent.clear();
    this.recent.set(memo, recent);
    return { text: lines[at], tone, key };
  }
}

/** Fills `{name}` (the other resident) and `{me}`. */
export function fill(text: string, me: string, other: string): string {
  return text.replaceAll('{me}', me || 'me').replaceAll('{name}', other || 'friend');
}
