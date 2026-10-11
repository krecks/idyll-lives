/**
 * The chat messages that ask the language model for a resident's line, in three parts whose
 * model state is kept as far as it's the same (`llm/chat.ts`, `llm/generate.ts`):
 *
 * 1. the rules and a few worked examples (residents of their own, moments unlike the real
 *    ones, so there's nothing to copy): the same for everyone;
 * 2. the resident: who they are, their personality, their job (`lead`): the same from line to
 *    line of theirs;
 * 3. the moment: when and where, how they feel, what's on their mind and around them, who
 *    they're with, what just happened, and hand-written lines for it if any
 *    (`content/voice/<lang>.json`) to show the tone.
 *
 * Words come from `content/talk/<lang>.json` (`TalkText`). Plain TypeScript with type-only
 * imports (the Node check runs it by stripping the types).
 */

import type { ChatMessage } from '../llm/chat';

/** `content/talk/<lang>.json`. */
export interface TalkText {
  rules: string;
  ask: string;
  /** The ask in a conversation: says to {name}. */
  askTo: string;
  examples: string;
  /** The reply's first words, written for the model (`Mia: "`): it writes only what's said, up to the closing quote. */
  reply: string;
  /** Worked examples before the real moment: a resident of their own (`me`, `person`, `personality`), a moment with `{me}`, and the line. */
  demos: { me: string; person: string; personality: string[]; moment: string; line: string }[];
  traits: Record<string, string>;
  perks: Record<string, string>;
  /** Per life stage, per gender id ('' for any other). */
  people: Record<string, Record<string, string>>;
  intro: string;
  job: string;
  school: string;
  personality: string;
  needs: Record<string, string>;
  times: Record<'night' | 'early' | 'morning' | 'midday' | 'afternoon' | 'evening', string>;
  weekdays: string[];
  situation: Record<'when' | 'home' | 'homeRoom' | 'visiting' | 'out' | 'weather' | 'feeling' | 'mind' | 'nearby' | 'holding' | 'with' | 'now', string>;
  relations: Record<string, string>;
  social: Record<string, { start: string; reply: string }>;
  outcome: { good: string; bad: string };
  use: string;
  emotion: string;
  thought: Record<string, string>;
  blocklist: string[];
}

/** Who is speaking, in words (the system message: stays the same while they do). */
export interface Persona {
  first: string;
  /** "a young woman" (`TalkText.people`). */
  person: string;
  /** "Line cook (Culinary)"; none for those without a job. */
  job?: string;
  school?: boolean;
  /** Personality in a few words each: traits, then perks. */
  personality: string[];
}

/** The moment, in words. */
export interface Situation {
  /** "Tuesday evening". */
  when: string;
  /** "at home, in the kitchen". */
  place: string;
  /** None until the game has weather. */
  weather?: string;
  /** Emotion, strongest feelings, needs running low. */
  feeling: string[];
  /** Thoughts, wishes and goals. */
  mind: string[];
  nearby: string[];
  holding?: string;
  /** Who they're talking to, and what they are to them ("good friend"). */
  partner?: { name: string; relation: string };
}

/** What just happened, that the line is about. */
export type LineEvent =
  /** A conversation: the opener (`start`), or the answer once it went well or badly. */
  | { kind: 'social'; social: string; phase: 'start' | 'good' | 'bad'; other: string }
  /** Starting to use something: "Cook a meal (Stove)". */
  | { kind: 'use'; action: string }
  | { kind: 'emotion'; emotion: string }
  /** A planner thought kind (`TalkText.thought`) and what it's about. */
  | { kind: 'thought'; thought: string; subject: string };

/** Fills `{key}` placeholders. */
export function fillIn(template: string, values: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (all, key: string) => values[key] ?? all);
}

/** Who is speaking, in words (the start of the last message: `lead`). */
export function personaText(text: TalkText, p: Persona): string {
  const me = p.first;
  const parts = [fillIn(text.intro, { me, person: p.person })];
  if (p.job) parts.push(fillIn(text.job, { me, subject: p.job }));
  else if (p.school) parts.push(fillIn(text.school, { me }));
  if (p.personality.length) parts.push(fillIn(text.personality, { me, subject: p.personality.join(', ') }));
  return parts.join(' ');
}

/** The event in words, about `me`. */
export function eventText(text: TalkText, me: string, e: LineEvent): string | null {
  switch (e.kind) {
    case 'social': {
      const s = text.social[e.social];
      if (!s) return null;
      const values = { me, name: e.other };
      if (e.phase === 'start') return fillIn(s.start, values);
      return fillIn(s.reply, values) + fillIn(e.phase === 'good' ? text.outcome.good : text.outcome.bad, values);
    }
    case 'use':
      return fillIn(text.use, { me, action: e.action });
    case 'emotion':
      return fillIn(text.emotion, { me, emotion: e.emotion });
    case 'thought': {
      const t = text.thought[e.thought];
      return t ? fillIn(t, { me, subject: e.subject }) : null;
    }
  }
}

export function situationMessage(text: TalkText, me: string, s: Situation, event: string, examples: readonly string[], to?: string): string {
  const w = text.situation;
  const lines = [`${fillIn(w.when, { subject: s.when })}, ${s.place}.`];
  if (s.weather) lines.push(fillIn(w.weather, { me, subject: s.weather }));
  if (s.feeling.length) lines.push(fillIn(w.feeling, { me, subject: s.feeling.join(', ') }));
  if (s.mind.length) lines.push(fillIn(w.mind, { me, subject: s.mind.join('; ') }));
  if (s.nearby.length) lines.push(fillIn(w.nearby, { me, subject: s.nearby.join(', ') }));
  if (s.holding) lines.push(fillIn(w.holding, { me, subject: s.holding }));
  if (s.partner) lines.push(fillIn(w.with, { me, name: s.partner.name, subject: s.partner.relation }));
  lines.push(fillIn(w.now, { me, subject: event }));
  if (examples.length) lines.push(`${fillIn(text.examples, { me })} ${examples.map((x) => `"${x}"`).join(' / ')}`);
  lines.push(to ? fillIn(text.askTo, { me, name: to }) : fillIn(text.ask, { me }));
  return lines.join('\n');
}

/**
 * The messages for one line, the persona that opens the last of them (`lead`) and the reply's
 * first words (`TalkText.reply`), or null if the event can't be put in words. Everything before
 * the last message (rules, worked examples) is the same for everyone.
 */
export function buildMessages(
  text: TalkText,
  persona: Persona,
  situation: Situation,
  event: LineEvent,
  examples: readonly string[],
): { messages: ChatMessage[]; lead: string; reply: string } | null {
  const me = persona.first;
  const now = eventText(text, me, event);
  if (!now) return null;
  const to = event.kind === 'social' ? event.other : undefined;
  const reply = fillIn(text.reply, { me });
  const quote = reply.trim().slice(-1);
  const demos = text.demos.flatMap((d): ChatMessage[] => [
    { role: 'user', content: `${personaText(text, { first: d.me, person: d.person, personality: d.personality })}\n${fillIn(d.moment, { me: d.me })}` },
    { role: 'assistant', content: `${fillIn(text.reply, { me: d.me })}${d.line}${quote}` },
  ]);
  return {
    messages: [{ role: 'system', content: text.rules }, ...demos, { role: 'user', content: situationMessage(text, me, situation, now, examples, to) }],
    lead: `${personaText(text, persona)}\n`,
    reply,
  };
}

/** "Tuesday evening" from the weekday (0 = Monday) and minute of the day. */
export function whenText(text: TalkText, weekday: number, minute: number): string {
  const h = minute / 60;
  const part = h < 5 ? 'night' : h < 8 ? 'early' : h < 11.5 ? 'morning' : h < 13.5 ? 'midday' : h < 17.5 ? 'afternoon' : h < 22 ? 'evening' : 'night';
  return `${text.weekdays[((weekday % 7) + 7) % 7] ?? ''} ${text.times[part]}`.trim();
}
