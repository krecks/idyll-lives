/**
 * Following the settings: whether residents' lines are the model's (only once the device test
 * found it usable on this computer where it runs, `llmDevice`; the test runs on its own when
 * needed), and the model loaded there from the start while they are (on a machine low on memory,
 * when the first line wants it; unloaded when they aren't).
 */

import { untrack } from 'svelte';
import { ensureLlm, llmStatus, unloadLlm } from '../llm';
import { settings } from '../settings/settings.svelte';
import { LOW_MEMORY } from '../settings/lowMemory';
import { applyReport, deviceTest, llmDevice, report, reportFor, runDeviceTest, usableHere, type DeviceReport } from './devices.svelte';
import { loadTalk } from './text';

/** The player chose the model's lines (whether or not they're usable here yet). */
export function llmChosen(): boolean {
  return settings.voices && settings.voiceWriter === 'llm';
}

/** Residents' lines are written by the model: chosen, and the device test found it usable here. */
export function llmWrites(): boolean {
  return llmChosen() && usableHere();
}

/**
 * Runs the device test where the model runs and applies what it found. On Auto it tests the
 * graphics card, and the processor if the card is missing or wanting; `again`: those already
 * tested too (else only what has no result yet).
 */
export async function testDevices(again = true): Promise<DeviceReport | null> {
  const text = await loadTalk(settings.voiceLanguage);
  if (!text) return null;
  let r: DeviceReport | null = null;
  if (settings.llmDevice === 'auto' && deviceTest.gpu !== 'none' && (again || !reportFor('gpu'))) {
    r = await runDeviceTest(text, 'gpu');
    if (!r) return null;
  }
  const device = llmDevice();
  if (r?.device !== device && (again || !reportFor(device))) r = await runDeviceTest(text, device);
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
    const device = llmDevice();
    const running = deviceTest.running;
    untrack(() => {
      if (running) return;
      if (chosen && !tested && !stopped) void testDevices(false).catch(() => {});
      // On another device (the setting changed): there instead. (Low on memory, it loads when the
      // first line wants it: `wantLlm`.)
      else if (writes && llmStatus.state !== 'off' && llmStatus.device !== device) void ensureLlm(device).catch(() => {});
      else if (writes && llmStatus.state === 'off' && !LOW_MEMORY) void ensureLlm(device).catch(() => {});
      else if (!writes && llmStatus.state !== 'off') unloadLlm();
    });
  });
});
