#!/usr/bin/env node
/**
 * Checks the language model that writes residents' lines as the browser's worker runs it
 * (`engine.mjs`: its 4-bit build, the worker's own `generate.ts`; on the CPU here, four threads),
 * with the game's own prompts (`web/src/talk/prompt.ts`, `content/talk/en.json`) and cleaning
 * (`clean.ts`):
 *
 * - `cleanLine` keeps what's said and drops labels, quotes, narration, stage directions and
 *   blocked words;
 * - prompts stay within their token budget, in three parts: rules and worked examples the same
 *   for everyone, the persona the same from line to line, then the moment; replies reuse the
 *   model's state after the parts they share (a new resident's first line only reads their
 *   persona and the moment);
 * - greedy replies are the same in one session (the persona's state cached) and in a fresh
 *   one, and match golden text (a change to the model, the prompt or the loop shows up here);
 * - sampled replies differ by seed;
 * - and prints the time per line.
 *
 * Needs `pnpm llm` first. Usage (from web/): `pnpm check:llm`.
 */

import { readFileSync } from 'node:fs';
import { countTokens, engine, formatChat, generate, SPEC, tokenizer } from './engine.mjs';

const web = new URL('../../web/', import.meta.url);
const P = await import(new URL('src/talk/prompt.ts', web).href);
const { cleanLine } = await import(new URL('src/talk/clean.ts', web).href);
const text = JSON.parse(readFileSync(new URL('public/content/talk/en.json', web), 'utf8'));

const fail = (message) => {
  console.error(`llm check FAILED: ${message}`);
  process.exit(1);
};

// ---- cleaning -----------------------------------------------------------------------------------

const clean = (raw, names = ['Tom']) => cleanLine(raw, { me: 'Mia', names, blocklist: text.blocklist });
const CLEAN = [
  ['Time to cook something nice.', 'Time to cook something nice.'],
  ['Mia says: "How was your day, Tom?"', 'How was your day, Tom?'],
  ["Mia's line: \"Pasta, fresh pasta!\"", 'Pasta, fresh pasta!'],
  ['"Hey Tom!" Mia waves.', 'Hey Tom!'],
  ['Mia smiles as she drifts off to sleep.', null],
  ['Tom laughs and says "good one"', null],
  ['*stretches* Time for bed', 'Time for bed.'],
  ['Mia, stop it!', 'Stop it!'],
  ['I love this 😀 so much!!', 'I love this so much!!'],
  ['What the hell is this crap?', null],
  ['Mia', null],
  ['One. Two. Three. Four.', 'One. Two. Three. Four.'],
  ['One. Two. Three. Four. Five. Six.', 'One. Two. Three. Four. Five.'],
  ['Look at [read](/ɹˈɛd/) this', 'Look at this.'],
  ['<|im_end|>', null],
  ['Sure, as an AI I can help.', null],
];
for (const [raw, want] of CLEAN) {
  const got = clean(raw);
  if (got !== want) fail(`cleanLine(${JSON.stringify(raw)}) = ${JSON.stringify(got)}, expected ${JSON.stringify(want)}`);
}
// Cut short (out of tokens or time): the unfinished sentence goes (with nothing finished, the line).
const CUT = [
  ['What a day. I think I will', 'What a day.'],
  ['What a day. I think I will!', 'What a day. I think I will!'],
  ['What a day and', null],
];
for (const [raw, want] of CUT) {
  const got = cleanLine(raw, { me: 'Mia', names: ['Tom'], blocklist: text.blocklist, cut: true });
  if (got !== want) fail(`cleanLine(${JSON.stringify(raw)}, cut) = ${JSON.stringify(got)}, expected ${JSON.stringify(want)}`);
}

// ---- prompts ------------------------------------------------------------------------------------

const MIA = { first: 'Mia', person: text.people.youngAdult.female, job: 'Line cook (Culinary)', personality: [text.traits.cheerful, text.traits.foodie, text.perks.charming] };
const OTTO = { first: 'Otto', person: text.people.elder.male, personality: [text.traits.hotHeaded, text.traits.gloomy] };
const LILY = { first: 'Lily', person: text.people.child.female, school: true, personality: [text.traits.energetic, text.traits.natureLover] };
/** Moments as the game describes them, everything filled in (the longest prompts). */
const MOMENTS = [
  [MIA, { when: 'Tuesday evening', place: 'at home, in the kitchen', feeling: ['happy', 'good conversation', 'hungry'], mind: ['reach Cooking level 5', 'a place to work out'], nearby: ['fridge', 'dining table', 'counter', 'sink'] }, { kind: 'use', action: 'cook a meal (stove)' }],
  [MIA, { when: 'Saturday morning', place: 'in the garden', feeling: ['tired'], mind: [], nearby: ['bench', 'grill'], partner: { name: 'Tom', relation: 'good friend' } }, { kind: 'social', social: 'chat', phase: 'start', other: 'Tom' }],
  [OTTO, { when: 'Monday afternoon', place: 'at home, in the living room', feeling: ['angry'], mind: [], nearby: ['TV', 'sofa'], partner: { name: 'Mia', relation: 'neighbour you just met' } }, { kind: 'social', social: 'joke', phase: 'bad', other: 'Mia' }],
  [OTTO, { when: 'Thursday morning', place: 'at home, in the bathroom', feeling: [], mind: [], nearby: ['shower', 'sink'] }, { kind: 'thought', thought: 'broken', subject: 'toilet' }],
  [LILY, { when: 'Sunday midday', place: 'in the garden', feeling: ['energized'], mind: [], nearby: ['rose bush', 'swing set'] }, { kind: 'use', action: 'water (rose bush)' }],
  [LILY, { when: 'Wednesday afternoon', place: 'at home, in the kitchen', feeling: ['happy'], mind: [], nearby: ['fridge'] }, { kind: 'emotion', emotion: 'inspired' }],
];
/** Tokens at most per part: rules and worked examples (everyone's), the persona (cached per resident), the moment. */
const BUDGET = [380, 80, 160];
const asks = MOMENTS.map(([persona, situation, event]) => {
  const built = P.buildMessages(text, persona, situation, event, []);
  if (!built) fail(`no prompt for ${JSON.stringify(event)}`);
  const prompt = formatChat(SPEC.chat, built.messages, built.reply, built.lead);
  if (prompt.parts.length !== 3) fail(`prompt for ${persona.first} ${event.kind}: ${prompt.parts.length} parts, expected 3`);
  const tokens = prompt.parts.map((p) => countTokens(tokenizer, p));
  if (tokens.some((n, i) => n > BUDGET[i])) fail(`prompt for ${persona.first} ${event.kind}: ${tokens.join(' + ')} tokens, over ${BUDGET.join(' + ')}`);
  if (prompt.parts[0].includes(persona.first) || !prompt.parts[1].includes(persona.first)) fail('the persona must be in the second part, not the shared one');
  return { persona, prompt, tokens };
});
if (asks.some((a) => a.prompt.parts[0] !== asks[0].prompt.parts[0])) fail('every resident must share the first part');
if (asks[0].prompt.parts[1] !== asks[1].prompt.parts[1]) fail('two lines of the same resident must share the persona part');

// ---- replies ------------------------------------------------------------------------------------

const GREEDY = { maxTokens: 96, temperature: 0, topK: 1, topP: 1, repetitionPenalty: 1.15, seed: 1, stopAtNewline: true, stopAt: ['"'] };
/**
 * Greedy replies to `MOMENTS`, cleaned. WASM arithmetic is the same on every machine (with the
 * same threads), so these hold everywhere; update them only on purpose (a new model, prompt
 * wording or loop change).
 */
const THREADS = 4;
const GOLDEN = [
  "Cookin' level 5, a kitchen full of food. The fridge is just out here.",
  "Hey Tom, you’re not the first to mention your lunch spot. The garden bench is perfect for a quick bite.",
  "You were really funny last night. I didn't even notice.",
  "Looks like the shower broke. I had to scramble for a change of clothes and maybe a quick bath.",
  "Roses are red, and I love them. The swing set is perfect for a sunny afternoon. Water the garden before it gets too hot.",
  "I can't believe I did it again! The plants are so happy, and the fridge is full of fresh herbs. It's amazing how much they make me feel.",
];

const e1 = await engine(THREADS);
const times = [];
const first = [];
for (const [i, a] of asks.entries()) {
  const r = await generate(e1, a.prompt, GREEDY);
  if (r.nonFinite) fail(`logits that aren't numbers for ${a.persona.first}`);
  // A resident's first line reads their persona; the rules and worked examples come from the cache.
  const firstOfTheirs = asks.findIndex((x) => x.persona === a.persona) === i;
  const expected = firstOfTheirs ? (i === 0 ? 0 : a.tokens[0]) : a.tokens[0] + a.tokens[1];
  if (r.cachedTokens !== expected) fail(`${a.persona.first} ${MOMENTS[i][2].kind}: ${r.cachedTokens} tokens from the cache, expected ${expected}`);
  first.push(r);
  times.push(r);
}
// The same in the same session (state from the cache), read in one go, and in a fresh session.
// (4-bit on the CPU is slow: the first two moments are enough.)
for (const [i, a] of asks.slice(0, 2).entries()) {
  const again = await generate(e1, a.prompt, GREEDY);
  if (again.cachedTokens === 0) fail('the second reply to the same persona should reuse the cached state');
  if (again.text !== first[i].text) fail(`same session, different reply: ${JSON.stringify(again.text)} vs ${JSON.stringify(first[i].text)}`);
  const whole = await generate(e1, a.prompt, GREEDY, () => false, false);
  if (whole.nonFinite || !whole.ids.length) fail(`read in one go, no reply for ${a.persona.first}`);
}
const e2 = await engine(THREADS);
for (const [i, a] of asks.slice(0, 2).entries()) {
  const fresh = await generate(e2, a.prompt, GREEDY);
  if (fresh.text !== first[i].text) fail(`fresh session, different reply: ${JSON.stringify(fresh.text)} vs ${JSON.stringify(first[i].text)}`);
}
const lines = first.map((r, i) => clean(r.text, MOMENTS[i][2].other ? [MOMENTS[i][2].other] : []));
if (GOLDEN[0] === 'PENDING') {
  console.log(`llm check: golden lines not pinned yet:\n${JSON.stringify(lines, null, 2)}`);
} else if (JSON.stringify(lines) !== JSON.stringify(GOLDEN)) {
  fail(`replies changed:\n${JSON.stringify(lines, null, 2)}\nexpected\n${JSON.stringify(GOLDEN, null, 2)}`);
}
if (lines.filter(Boolean).length < Math.ceil(lines.length / 2)) fail(`most greedy replies should pass cleanLine: ${JSON.stringify(lines)}`);

// Sampling: different seeds, different lines (most of the time).
const sampled = new Set();
for (let seed = 1; seed <= 3; seed++) sampled.add((await generate(e1, asks[1].prompt, { ...GREEDY, temperature: 0.7, topK: 40, topP: 0.9, seed })).text);
if (sampled.size < 2) fail(`three seeds wrote the same line: ${[...sampled][0]}`);

const avg = (f) => times.reduce((n, r) => n + f(r), 0) / times.length;
console.log(
  `llm check passed: ${CLEAN.length} cleaning cases, ${asks.length} prompts (≤ ${[0, 1, 2].map((i) => Math.max(...asks.map((a) => a.tokens[i]))).join(' + ')} tokens), ` +
    `${THREADS} threads on the CPU: prompt ${avg((r) => r.prefillMs).toFixed(0)} ms (nothing cached: ${times[0].prefillMs.toFixed(0)} ms; a new resident: ${times[2].prefillMs.toFixed(0)} ms), ` +
    `${(avg((r) => r.ids.length) / (avg((r) => r.decodeMs) / 1000)).toFixed(1)} tokens/s, ${avg((r) => r.prefillMs + r.decodeMs).toFixed(0)} ms per line`,
);
for (const [i, l] of lines.entries()) console.log(`  ${MOMENTS[i][0].first} ${MOMENTS[i][2].kind}: ${JSON.stringify(l ?? `✗ ${first[i].text}`)}`);
