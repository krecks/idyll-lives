/**
 * Residents' lines written by the language model (`llm/`): the moment in words
 * (`context.ts`), a prompt (`prompt.ts`), a reply in time, and the reply made into a line
 * (`clean.ts`). A line that comes too late, or doesn't pass, isn't said: the hand-written lines
 * only show the model the tone (their first words are barred, so it doesn't copy them).
 */

import type { FrameState } from '../core/bridge';
import { generateText, type GenResult } from '../llm';
import { fill, type Line, type Lines } from '../voice/lines';
import { hashUnit } from '../voice/voices';
import type { LineRequest, LineWriter } from '../voice/writer';
import { cleanLine } from './clean';
import { gatherMoment, type TalkSources } from './context';
import { buildMessages, type TalkText } from './prompt';

/** Time the model has to write a line, ms (speech is made after). */
export const WRITE_MS = 3500;
/** A persona read ahead may wait this long for the model. */
const PRIME_MS = 30_000;
/** Hand-written lines shown as examples. */
const EXAMPLES = 2;
const MAX_LOG = 30;

/** With `?debug`: what was asked and written, newest last (`window.__voices.writer.log`). */
export interface WriterLog {
  key: string;
  prompt: string;
  raw: string | null;
  line: string | null;
  ms: number;
  stats?: Pick<GenResult, 'promptTokens' | 'cachedTokens' | 'prefillMs' | 'decodeMs' | 'stop'> & { tokens: number };
}

export class LlmWriter implements LineWriter {
  readonly slow = true;
  readonly log: WriterLog[] = [];
  private readonly debug = new URLSearchParams(location.search).has('debug');

  constructor(
    private readonly text: TalkText,
    private readonly sources: TalkSources,
    /** Hand-written lines, for examples and tones (null while loading). */
    private readonly lines: () => Lines | null,
  ) {}

  /**
   * Reads a resident's persona ahead (when they're picked), so their first line only reads the
   * moment: on one thread the persona and worked examples take seconds, more than a line has.
   */
  prime(req: LineRequest, frame: FrameState): void {
    const moment = gatherMoment(this.text, this.sources, req, frame);
    if (!moment) return;
    const built = buildMessages(this.text, moment.persona, moment.situation, { kind: 'emotion', emotion: 'calm' }, []);
    if (built) void generateText(built.messages, { maxTokens: 1 }, { reply: built.reply, lead: built.lead, deadline: Date.now() + PRIME_MS }).catch(() => null);
  }

  async write(req: LineRequest, frame: FrameState, signal: AbortSignal): Promise<Line | null> {
    const moment = gatherMoment(this.text, this.sources, req, frame);
    if (!moment) return null;
    const scripted = this.lines()?.candidates(req) ?? null;
    const other = moment.names[0] ?? '';
    const examples = pick(scripted?.lines ?? [], req.seed, EXAMPLES).map((l) => fill(l, moment.me, other));
    const built = buildMessages(this.text, moment.persona, moment.situation, moment.event, examples);
    if (!built) return null;
    const quote = built.reply.trim().slice(-1);
    const start = performance.now();
    const result = await generateText(
      built.messages,
      { seed: (req.seed * 2654435761) >>> 0, stopAt: [quote, '"', '“', '”'], avoidStart: examples },
      { reply: built.reply, lead: built.lead, deadline: Date.now() + WRITE_MS, signal },
    ).catch(() => null);
    const line = result && !result.nonFinite ? cleanLine(result.text, { me: moment.me, names: moment.names, blocklist: this.text.blocklist }) : null;
    if (this.debug) {
      const prompt = built.lead + built.messages[built.messages.length - 1].content;
      const stats = result ? { promptTokens: result.promptTokens, cachedTokens: result.cachedTokens, prefillMs: result.prefillMs, decodeMs: result.decodeMs, stop: result.stop, tokens: result.ids.length } : undefined;
      if (this.log.push({ key: scripted?.key ?? req.kind, prompt, raw: result?.text ?? null, line, ms: performance.now() - start, stats }) > MAX_LOG) this.log.shift();
    }
    return line ? { text: line, tone: scripted?.tone ?? toneOf(req), key: `llm:${scripted?.key ?? req.kind}` } : null;
  }
}

/** Up to `n` different lines by hash. */
function pick(lines: readonly string[], seed: number, n: number): string[] {
  if (!lines.length) return [];
  const first = Math.floor(hashUnit(seed, 11) * lines.length);
  return Array.from({ length: Math.min(n, lines.length) }, (_, i) => lines[(first + i) % lines.length]);
}

/** A tone when no hand-written lines give one. */
function toneOf(req: LineRequest): string | undefined {
  if (req.kind === 'emotion' && ['happy', 'sad', 'angry', 'flirty'].includes(req.emotion ?? '')) return req.emotion;
  if (req.kind === 'social' && req.phase === 'good') return 'happy';
  return undefined;
}
