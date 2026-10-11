/**
 * Residents talking in their own words: the game's side of the language model (`llm/`).
 * `writer.ts` writes a resident's line for the voice director from what the game knows about
 * the moment (`context.ts`, `prompt.ts`, `clean.ts`); `devices.svelte.ts` tests whether the
 * chosen model and the voices run well enough here, and where; `talk.svelte.ts` follows the
 * settings. The words used to ask are content (`content/talk/<lang>.json`, `text.ts`).
 */

import { generateText } from '../llm';
import { settings } from '../settings/settings.svelte';
import { cleanLine } from './clean';
import { loadDeviceReports, runDeviceTest, testPrompts, type SetupId } from './devices.svelte';
import { currentModel, ensureTalk } from './talk.svelte';
import { loadTalk } from './text';

export { LlmWriter, WRITE_MS } from './writer';
export { applyReport, deviceTest, reportFor, reportKey, runDeviceTest, SETUPS, stopDeviceTest, type DeviceReport, type SetupResult } from './devices.svelte';
export type { TalkText } from './prompt';
export { currentModel, ensureTalk, llmChosen, llmWrites, talkDevices, testDevices } from './talk.svelte';
export { loadTalk } from './text';

void loadDeviceReports();
// `?debug`: the device test from the console or `tools/llm/devices.mjs` (`await __llmBench()`).
if (new URLSearchParams(location.search).has('debug')) {
  Object.assign(window, { __llmBench: async (only?: SetupId[]) => runDeviceTest((await loadTalk('en'))!, currentModel(), only) });
}

let samples = 0;

/** A line the model writes for a sample moment (settings: "Write a sample line"), or null if none passed. */
export async function sampleLine(): Promise<string | null> {
  const text = await loadTalk(settings.voiceLanguage);
  if (!text) return null;
  await ensureTalk();
  const asks = testPrompts(text);
  const ask = asks[samples++ % asks.length];
  if (!ask) return null;
  const g = await generateText(ask.messages, { seed: Date.now() >>> 0, stopAt: ['"'] }, { reply: ask.reply, lead: ask.lead });
  return g ? cleanLine(g.text, { me: 'Mia', names: ['Tom'], blocklist: text.blocklist }) : null;
}
