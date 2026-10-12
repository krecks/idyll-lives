/**
 * Chat messages to the text the model reads, in ChatML (`<|im_start|>role\n…<|im_end|>\n`, as
 * LFM2.5 writes it): a small function instead of the model's Jinja template. The text comes in
 * `parts`; the model's state after each part but the last is kept (`generate.ts`), so calls that
 * start the same only read what differs:
 *
 * 1. the start, the system message and the turns before the last user message (worked examples);
 * 2. if given, the `lead` of the last user message (who is speaking: the same from line to line
 *    of one speaker, while level 1 is the same for everyone);
 * 3. the rest of it, up to where the assistant's reply starts, and the reply's first words if
 *    given (`Mia: "` makes a small model write just what Mia says).
 *
 * Plain TypeScript without runtime imports (the Node checks run it by stripping the types).
 */

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

/** The model's chat format: ChatML, after `bos` if the model starts with one. */
export interface ChatFormat {
  bos: string;
}

export interface ChatPrompt {
  /** The text in parts, the model's state kept after each but the last (empty parts left out). */
  parts: string[];
}

const open = (role: string) => `<|im_start|>${role}\n`;
const close = '<|im_end|>\n';
const answer = '<|im_start|>assistant\n';

/** `lead`: the start of the last user message, its own part (see above). */
export function formatChat(format: ChatFormat, messages: readonly ChatMessage[], reply = '', lead = ''): ChatPrompt {
  const last = messages.findLastIndex((m) => m.role === 'user');
  const at = last < 0 ? messages.length : last;
  const turn = (m: ChatMessage) => open(m.role) + m.content + close;
  const shared = format.bos + messages.slice(0, at).map(turn).join('');
  const rest = messages.slice(at);
  if (!lead || !rest.length) return { parts: [shared, rest.map(turn).join('') + answer + reply].filter((p, i, all) => p || i === all.length - 1) };
  const [first, ...more] = rest;
  return {
    parts: [shared, open(first.role) + lead, first.content + close + more.map(turn).join('') + answer + reply].filter((p, i, all) => p || i === all.length - 1),
  };
}
