/**
 * Following the settings: whether residents' lines are the model's (only once the device test
 * found it usable on this computer; the test runs on its own when needed), and the model loaded
 * from the start while they are (unloaded when they aren't).
 */

import { untrack } from 'svelte';
import { ensureLlm, llmStatus, unloadLlm } from '../llm';
import { settings } from '../settings/settings.svelte';
import { applyReport, deviceTest, report, runDeviceTest, usableHere, type DeviceReport } from './devices.svelte';
import { loadTalk } from './text';

/** The player chose the model's lines (whether or not they're usable here yet). */
export function llmChosen(): boolean {
  return settings.voices && settings.voiceWriter === 'llm';
}

/** Residents' lines are written by the model: chosen, and the device test found it usable here. */
export function llmWrites(): boolean {
  return llmChosen() && usableHere();
}

/** Runs the device test and applies what it found. */
export async function testDevices(): Promise<DeviceReport | null> {
  const text = await loadTalk(settings.voiceLanguage);
  if (!text) return null;
  const r = await runDeviceTest(text);
  if (r) applyReport(r);
  return r;
}

/**
 * Follows the settings: tests the model if it wasn't tested here (once the GPU's name is known:
 * results are kept per GPU; not again if the player stopped it), loads it as soon as its lines
 * are used, unloads it when they aren't. Nothing changes while the device test runs (it loads
 * what it tests).
 */
$effect.root(() => {
  $effect(() => {
    if (!deviceTest.gpu) return;
    const chosen = llmChosen();
    const tested = report() !== null;
    const stopped = deviceTest.stopped;
    const writes = llmWrites();
    const running = deviceTest.running;
    untrack(() => {
      if (running) return;
      if (chosen && !tested && !stopped) void testDevices().catch(() => {});
      else if (writes && llmStatus.state === 'off') void ensureLlm().catch(() => {});
      else if (!writes && llmStatus.state !== 'off') unloadLlm();
    });
  });
});
