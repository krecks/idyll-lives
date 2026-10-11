/**
 * Where a resident's line comes from: the voice director (`index.ts`) says what happened (a
 * `LineRequest`) and a writer turns it into words. `ScriptWriter` picks a hand-written line
 * (`lines.ts`) at once; `talk/writer.ts` asks the language model, which takes a moment.
 */

import type { FrameState } from '../core/bridge';
import type { Line, Lines } from './lines';

/** What a resident's line is about, by content id. */
export interface LineRequest {
  /** Who speaks, their snapshot row now, and who they talk to (-1: nobody). */
  speaker: number;
  row: number;
  partner: number;
  kind: 'social' | 'use' | 'emotion' | 'thought';
  /** Conversations: the social interaction, and the opener or the answer once it went well or badly. */
  social?: string;
  phase?: 'start' | 'good' | 'bad';
  /** Using something: the object and interaction ids, and the animation tag. */
  object?: string;
  interaction?: string;
  tag?: string;
  emotion?: string;
  /** Planner thought kind (snapshot `thought`, 1..) and its subject. */
  thought?: number;
  thoughtSubject?: number;
  /** Variation: picks among lines, seeds sampling. */
  seed: number;
}

export interface LineWriter {
  /** Takes a while (the line may come too late to say). */
  readonly slow: boolean;
  /** The line for a request, or null for nothing to say (or `signal` aborted it). */
  write(req: LineRequest, frame: FrameState, signal: AbortSignal): Promise<Line | null>;
}

/** Hand-written lines, picked by hash. */
export class ScriptWriter implements LineWriter {
  readonly slow = false;

  constructor(private readonly lines: Lines) {}

  write(req: LineRequest): Promise<Line | null> {
    return Promise.resolve(scripted(this.lines, req));
  }
}

/** The hand-written line for a request (`Lines` remembers what each resident said lately). */
export function scripted(lines: Lines, req: LineRequest): Line | null {
  const { speaker: who, seed } = req;
  switch (req.kind) {
    case 'social':
      return req.social && req.phase ? lines.social(who, req.social, req.phase, seed) : null;
    case 'use':
      return lines.use(who, req.tag, { object: req.object, interaction: req.interaction }, seed);
    case 'emotion':
      return req.emotion ? lines.emotion(who, req.emotion, seed) : null;
    case 'thought':
      return req.thought ? lines.thought(who, req.thought, seed) : null;
  }
}
