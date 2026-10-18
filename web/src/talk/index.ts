/**
 * Residents talking in their own words: the game's side of the language model (`llm/`).
 * `writer.ts` writes a resident's line for the voice director from what the game knows about
 * the moment (`context.ts`, `prompt.ts`, `clean.ts`); `devices.svelte.ts` tests whether the
 * model and the voices run well enough here; `talk.svelte.ts` follows the settings. The words used to ask are content (`content/talk/<lang>.json`, `text.ts`).
 */

import { ensureLlm, generateText } from '../llm';
import { settings } from '../settings/settings.svelte';
import { cleanLine } from './clean';
import { loadDeviceReports, runDeviceTest, testPrompts } from './devices.svelte';
import { loadTalk } from './text';
import { cutShort, LINE_STOPS, LINE_TOKENS } from './writer';

export { LlmWriter, WRITE_MS } from './writer';
export { browserHint, deviceTest, report, runDeviceTest, stopDeviceTest, type DeviceReport, type TestResult } from './devices.svelte';
export type { TalkText } from './prompt';
export { llmChosen, llmWrites, testDevices } from './talk.svelte';
export { loadTalk } from './text';

void loadDeviceReports();
// `?debug`: the device test from the console or `tools/llm/devices.mjs` (`await __llmBench()`).
if (new URLSearchParams(location.search).has('debug')) {
  Object.assign(window, { __llmBench: async () => runDeviceTest((await loadTalk('en'))!) });
}

let samples = 0;

/** A line the model writes for a sample moment (settings: "Write a sample line"), or null if none passed. */
export async function sampleLine(): Promise<string | null> {
  const text = await loadTalk(settings.voiceLanguage);
  if (!text) return null;
  await ensureLlm();
  const asks = testPrompts(text);
  const at = samples++ % asks.length;
  const ask = asks[at];
  if (!ask) return null;
  // `testPrompts` asks the moment at `at` for `at + 1` sentences.
  const g = await generateText(ask.messages, { seed: Date.now() >>> 0, maxTokens: LINE_TOKENS, stopAt: LINE_STOPS, sentences: at + 1 }, { reply: ask.reply, lead: ask.lead });
  return g ? cleanLine(g.text, { me: 'Mia', names: ['Tom'], blocklist: text.blocklist, cut: cutShort(g) }) : null;
}
