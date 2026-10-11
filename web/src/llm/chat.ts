/**
 * Chat messages to the text a model reads, per chat format: small functions instead of the
 * models' Jinja templates. The text comes in `parts`; the model's state after each part but
 * the last is kept (`generate.ts`), so calls that start the same only read what differs:
 *
 * 1. the start, the system message and the turns before the last user message (worked examples);
 * 2. if given, the `lead` of the last user message (who is speaking: the same from line to line
 *    of one speaker, while level 1 is the same for everyone);
 * 3. the rest of it, up to where the assistant's reply starts, and the reply's first words if
 *    given (`Mia: "` makes a small model write just what Mia says).
 *
 * `detectChatFormat` tells the format from a model's Jinja template (models added from Hugging
 * Face). Plain TypeScript without runtime imports (the Node checks run it by stripping the types).
 */

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export type ChatFormat =
  /** `<|im_start|>role\n…<|im_end|>\n` (LFM2, SmolLM, Qwen), after `bos` if the model starts with one. */
  | { kind: 'chatml'; bos: string }
  /** `<start_of_turn>user\n…<end_of_turn>\n`, the assistant as `model`, the system message joined to the first user turn (Gemma). */
  | { kind: 'gemma'; bos: string };

export interface ChatPrompt {
  /** The text in parts, the model's state kept after each but the last (empty parts left out). */
  parts: string[];
}

/** `lead`: the start of the last user message, its own part (see above). */
export function formatChat(format: ChatFormat, messages: readonly ChatMessage[], reply = '', lead = ''): ChatPrompt {
  const last = messages.findLastIndex((m) => m.role === 'user');
  const split = last < 0 ? messages.length : last;
  let turns: { role: string; content: string }[] = messages.map((m) => ({ role: m.role, content: m.content }));
  let at = split;
  let open: (role: string) => string;
  let close: string;
  let answer: string;
  if (format.kind === 'gemma') {
    // No system role: the system message opens the first user turn.
    const system = messages[0]?.role === 'system' ? messages[0].content : '';
    const rest = system ? messages.slice(1) : messages;
    turns = rest.map((m, i) => ({ role: m.role === 'assistant' ? 'model' : 'user', content: i === 0 && system ? `${system}\n\n${m.content}` : m.content }));
    at = system ? split - 1 : split;
    open = (role) => `<start_of_turn>${role}\n`;
    close = '<end_of_turn>\n';
    answer = '<start_of_turn>model\n';
  } else {
    open = (role) => `<|im_start|>${role}\n`;
    close = '<|im_end|>\n';
    answer = '<|im_start|>assistant\n';
  }
  const turn = (m: { role: string; content: string }) => open(m.role) + m.content + close;
  const shared = format.bos + turns.slice(0, at).map(turn).join('');
  const rest = turns.slice(at);
  if (!lead || !rest.length) return { parts: [shared, rest.map(turn).join('') + answer + reply].filter((p, i, all) => p || i === all.length - 1) };
  const [first, ...more] = rest;
  return {
    parts: [shared, open(first.role) + lead, first.content + close + more.map(turn).join('') + answer + reply].filter((p, i, all) => p || i === all.length - 1),
  };
}

/** The tokens that end a turn in a format. */
export function turnEnds(format: ChatFormat): string[] {
  return format.kind === 'gemma' ? ['<end_of_turn>'] : ['<|im_end|>'];
}

/**
 * A model's chat format from its Jinja chat template and start-of-text token, or null if it's
 * not one these functions write.
 */
export function detectChatFormat(template: string, bosToken: string | null): ChatFormat | null {
  // Only when the template writes the start-of-text token itself.
  const bos = bosToken && /bos_token/.test(template) ? bosToken : '';
  if (template.includes('<|im_start|>')) return { kind: 'chatml', bos };
  if (template.includes('<start_of_turn>')) return { kind: 'gemma', bos };
  return null;
}
