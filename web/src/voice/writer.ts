/**
 * Where a resident's line comes from: the voice director (`index.ts`) says what happened (a
 * `LineRequest`) and it's turned into words: `scripted` picks a hand-written line (`lines.ts`)
 * at once; `LlmWriter` (`talk/writer.ts`) asks the language model, which takes a moment.
 */

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
