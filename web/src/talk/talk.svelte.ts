/**
 * Following the settings: the chosen model, whether its lines are used (only once the device
 * test found it usable on this computer; the test runs on its own when needed), where the model
 * and the voices run, and the model loaded only while it writes the lines.
 */

import { untrack } from 'svelte';
import { ensureLlm, llmStatus, modelById, unloadLlm, type ModelSpec } from '../llm';
import type { Device } from '../ml/ort';
import { settings } from '../settings/settings.svelte';
import { applyReport, devicePick, deviceTest, reportFor, reportKey, runDeviceTest, type DeviceReport } from './devices.svelte';
import { loadTalk } from './text';

/** The model chosen in settings. */
export function currentModel(): ModelSpec {
  return modelById(settings.llmModel);
}

/** The player chose the model's lines (whether or not they're usable here yet). */
export function llmChosen(): boolean {
  return settings.voices && settings.voiceWriter === 'llm';
}

/** Residents' lines are written by the model: chosen, and the device test found it usable here. */
export function llmWrites(): boolean {
  return llmChosen() && devicePick(currentModel()) !== null;
}

/** Where the model runs (the device test's pick; the GPU without one). The voices always run on the CPU. */
export function talkDevices(): { llm: Device } {
  return devicePick(currentModel()) ?? { llm: 'gpu' };
}

/** Loads the chosen model where it runs best (switching if that changed). */
export function ensureTalk(): Promise<void> {
  const devices = talkDevices();
  return ensureLlm({ device: devices.llm, model: currentModel().id });
}

/** Runs the device test for the chosen model and applies what it found. */
export async function testDevices(): Promise<DeviceReport | null> {
  const text = await loadTalk(settings.voiceLanguage);
  if (!text) return null;
  const model = currentModel();
  const report = await runDeviceTest(text, model);
  if (report) applyReport(report, model);
  return report;
}

/**
 * Follows the settings (not the model's state): tests the chosen model if it wasn't tested
 * here (once the GPU's name is known: results are kept per GPU; not again if the player
 * stopped it); unloads it when its lines are off; moves it when the pick changes. Nothing
 * changes while the device test runs (it loads what it tests).
 */
$effect.root(() => {
  $effect(() => {
    if (!deviceTest.gpu) return;
    const chosen = llmChosen();
    const model = currentModel();
    const tested = reportFor(model) !== null;
    const stopped = deviceTest.stopped.includes(reportKey(model));
    const devices = talkDevices();
    untrack(() => {
      if (deviceTest.running) return;
      if (chosen && !tested && !stopped) {
        void testDevices().catch(() => {});
        return;
      }
      if (!llmWrites()) {
        if (llmStatus.state !== 'off') unloadLlm();
        return;
      }
      if (llmStatus.state !== 'off' && (llmStatus.device !== devices.llm || llmStatus.model !== model.id)) void ensureTalk().catch(() => {});
    });
  });
});
