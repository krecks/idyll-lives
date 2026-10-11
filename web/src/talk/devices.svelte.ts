/**
 * The device test: whether the chosen language model and the voices run fast enough and right
 * on this computer, and where. The model is used as its 4-bit build as it is: fast on the GPU
 * (WebGPU), slow on the CPU (WASM). Setups:
 *
 * | | model | voices |
 * |---|---|---|
 * | A | GPU | CPU |
 * | B | GPU | GPU |
 * | C | CPU | CPU: only without a GPU, or if neither GPU setup worked |
 *
 * Splitting between the two workers is the split worth having: the model and the voices work
 * side by side without taking each other's threads. The GPU also draws the game, so the test
 * watches the frames too.
 *
 * Each setup writes five lines (greedy, so setups can be compared) and speaks them, measuring
 * load time, prompt and writing speed, speech time, the time from asking to having audio, and
 * frame times. Right means: no logits that aren't numbers, lines that pass `cleanLine`, and
 * voices as loud as on the CPU. The fastest setup that's right, smooth and in time is used.
 * If none is, the model's lines are switched off (hand-written ones are said); if even speech
 * alone is too slow, voices are switched off too (`applyReport`).
 *
 * Results are kept per model and GPU (measured on an Apple M-series Mac, 2026-10-11, LFM2.5:
 * B 0.18 s a line, A 0.9 s).
 */

import { ensureLlm, generateText, llmStatus, unloadLlm, type ModelSpec } from '../llm';
import { hasWebGpu, type Device } from '../ml/ort';
import { settings } from '../settings/settings.svelte';
import { toast } from '../ui/state.svelte';
import { synthesize, useVoiceDevice, voiceDevice } from '../voice/service.svelte';
import { cleanLine } from './clean';
import { buildMessages, type LineEvent, type Persona, type Situation, type TalkText } from './prompt';
import { WRITE_MS } from './writer';

export type SetupId = 'A' | 'B' | 'C';

export interface Setup {
  id: SetupId;
  llm: Device;
  voice: Device;
}

export const SETUPS: readonly Setup[] = [
  { id: 'A', llm: 'gpu', voice: 'cpu' },
  { id: 'B', llm: 'gpu', voice: 'gpu' },
  { id: 'C', llm: 'cpu', voice: 'cpu' },
];

export interface SetupResult extends Setup {
  /** It loaded and ran (`error` otherwise). */
  ran: boolean;
  error?: string;
  /** Model load, ms. */
  loadMs: number;
  /** Reading the first prompt (nothing cached) and the others (persona cached), ms (median). */
  prefillMs: number;
  prefillCachedMs: number;
  /** Writing speed, tokens per second (median). */
  tokPerS: number;
  /** Making a line's speech, and from asking for a line to having its audio, ms (median). */
  speechMs: number;
  lineMs: number;
  /** Frame times while working, ms: median and 95th percentile. */
  frameP50: number;
  frameP95: number;
  /** Logits that weren't numbers (a broken model or runtime). */
  nonFinite: boolean;
  /** Lines that passed `cleanLine`, of those written. */
  clean: number;
  written: number;
  /** Voices as loud as on the CPU (within 30 %) and no samples that aren't numbers. */
  voiceOk: boolean;
  /** Right: ran, numbers fine, most lines clean, voices fine. */
  ok: boolean;
  /** Right, frames smooth and lines in time. */
  usable: boolean;
  lines: string[];
}

export interface DeviceReport {
  model: string;
  gpu: string;
  date: string;
  /** Frame times with nothing running, ms. */
  idleP50: number;
  idleP95: number;
  results: SetupResult[];
  /** Where the model and voices run; null: nowhere fast enough (the model's lines are off). */
  pick: { llm: Device; voice: Device; setup: SetupId } | null;
  /** Speech alone is fast enough somewhere (else voices are off). */
  voicesOk: boolean;
}

const KEY = 'littlelives.deviceTests';
/** A line must be written and spoken within this (the director gives a written line `MAX_WAIT_LLM_MS` in all). */
const LINE_MS = WRITE_MS;
/** Speech alone must be made within this (the voice speed test's "ok"). */
const SPEECH_MS = 3000;
const VOICE = { mix: [5, 5, 1] as [number, number, number], speed: 1, pitch: 1, depth: 1 };

export const deviceTest = $state({
  running: false,
  /** What it's doing now, and how far it is, 0..1. */
  step: '',
  progress: 0,
  /** Asked to stop (`stopDeviceTest`). */
  stopping: false,
  /** The GPU's name ('none' without one; '' until known: results are kept per GPU). */
  gpu: '',
  /** Results by model version and GPU (`reportKey`). */
  reports: readReports(),
  /** Tests stopped this session (`reportKey`): not started again on their own. */
  stopped: [] as string[],
});

/** Where a model's result is kept: per model version and GPU. */
export function reportKey(model: ModelSpec): string {
  return `${model.version}|${deviceTest.gpu}`;
}

/** The result for a model on this computer, if it was tested. */
export function reportFor(model: ModelSpec): DeviceReport | null {
  return deviceTest.reports[reportKey(model)] ?? null;
}

/** Where a model and the voices run, if the test found it usable here. */
export function devicePick(model: ModelSpec): { llm: Device; voice: Device } | null {
  return reportFor(model)?.pick ?? null;
}

function readReports(): Record<string, DeviceReport> {
  try {
    const r = JSON.parse(localStorage.getItem(KEY) ?? '{}') as Record<string, DeviceReport>;
    return r && typeof r === 'object' ? r : {};
  } catch {
    return {};
  }
}

/** Learns the GPU's name (results are kept per GPU). */
export async function loadDeviceReports(): Promise<void> {
  deviceTest.gpu = await adapterName();
}

/** Stops a running test: the model unloads, what it was doing fails, nothing is kept. */
export function stopDeviceTest(): void {
  if (!deviceTest.running) return;
  deviceTest.stopping = true;
  deviceTest.step = 'Stopping…';
  unloadLlm();
}

class Stopped extends Error {}

async function adapterName(): Promise<string> {
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(): Promise<{ info?: { vendor?: string; architecture?: string; device?: string } } | null> } }).gpu;
  try {
    const info = (await gpu?.requestAdapter())?.info;
    return info ? [info.vendor, info.architecture, info.device].filter(Boolean).join(' ') || 'gpu' : 'none';
  } catch {
    return 'none';
  }
}

/** Five moments to write lines for (one persona, so its state is cached after the first; also the settings' sample). */
export function testPrompts(text: TalkText): ReturnType<typeof buildMessages>[] {
  const persona: Persona = { first: 'Mia', person: text.people.youngAdult?.female ?? 'a young woman', job: 'Line cook (Culinary)', personality: [text.traits.cheerful, text.traits.foodie].filter(Boolean) };
  const base: Situation = { when: 'Tuesday evening', place: 'at home, in the kitchen', feeling: ['happy'], mind: [], nearby: ['fridge', 'dining table'] };
  const moments: [Partial<Situation>, LineEvent][] = [
    [{}, { kind: 'use', action: 'cook a meal (stove)' }],
    [{ place: 'in the garden', partner: { name: 'Tom', relation: 'good friend' } }, { kind: 'social', social: 'chat', phase: 'start', other: 'Tom' }],
    [{ partner: { name: 'Tom', relation: 'friend' } }, { kind: 'social', social: 'joke', phase: 'good', other: 'Tom' }],
    [{ feeling: ['tired'] }, { kind: 'thought', thought: 'skipped', subject: 'work out' }],
    [{ place: 'at home, in the living room' }, { kind: 'emotion', emotion: 'inspired' }],
  ];
  return moments.map(([s, e]) => buildMessages(text, persona, { ...base, ...s }, e, []));
}

const median = (xs: number[]) => (xs.length ? [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)] : 0);
const percentile = (xs: number[], p: number) => (xs.length ? [...xs].sort((a, b) => a - b)[Math.min(xs.length - 1, Math.floor(xs.length * p))] : 0);

/** Frame times (ms) while `work` runs. */
async function watchFrames<T>(work: () => Promise<T>): Promise<{ value: T; frames: number[] }> {
  const frames: number[] = [];
  let last = performance.now();
  let on = true;
  const tick = (t: number) => {
    frames.push(t - last);
    last = t;
    if (on) requestAnimationFrame(tick);
  };
  requestAnimationFrame((t) => {
    last = t;
    requestAnimationFrame(tick);
  });
  try {
    return { value: await work(), frames };
  } finally {
    on = false;
  }
}

function rms(x: Float32Array): number {
  let s = 0;
  for (const v of x) s += v * v;
  return Math.sqrt(s / Math.max(1, x.length));
}

/**
 * Runs the test for a model (`only`: some setups). The GPU setups run where there's WebGPU;
 * the CPU one only if neither of them is usable. Null if it was stopped (`stopDeviceTest`).
 */
export async function runDeviceTest(text: TalkText, model: ModelSpec, only?: SetupId[]): Promise<DeviceReport | null> {
  if (deviceTest.running) throw new Error('the device test is already running');
  deviceTest.running = true;
  deviceTest.stopping = false;
  deviceTest.progress = 0;
  const before = { llm: llmStatus.state === 'off' ? null : llmStatus.device, voice: voiceDevice() };
  const check = () => {
    if (deviceTest.stopping) throw new Stopped('stopped');
  };
  try {
    deviceTest.gpu = await adapterName();
    const gpu = await hasWebGpu();
    deviceTest.step = 'Measuring frames';
    const idle = (await watchFrames(() => new Promise((r) => setTimeout(r, 1000)))).frames;
    // Smooth enough: the slow frames no worse than two and a half typical idle ones (or 40 ms).
    // (The idle 95th percentile catches page loads and the like: the median is steadier.)
    const smoothLimit = Math.max(median(idle) * 2.5, 40);
    const asks = testPrompts(text).filter((p): p is NonNullable<typeof p> => !!p);
    // Progress: per setup, loading counts as two lines' worth.
    const planned = SETUPS.filter((x) => (only ? only.includes(x.id) : x.llm === 'gpu' ? gpu : true));
    const perSetup = asks.length + 2;
    let stepsDone = 0;
    const advance = (n = 1) => {
      stepsDone += n;
      deviceTest.progress = Math.min(1, stepsDone / Math.max(1, planned.length * perSetup));
    };
    const results: SetupResult[] = [];
    let cpuRms = 0;
    for (const setup of SETUPS) {
      if (only && !only.includes(setup.id)) continue;
      // The CPU only when no GPU setup will do (unless asked for).
      if (setup.llm === 'cpu' && !only && results.some((x) => x.usable)) continue;
      const r: SetupResult = { ...setup, ran: false, loadMs: 0, prefillMs: 0, prefillCachedMs: 0, tokPerS: 0, speechMs: 0, lineMs: 0, frameP50: 0, frameP95: 0, nonFinite: false, clean: 0, written: 0, voiceOk: false, ok: false, usable: false, lines: [] };
      results.push(r);
      if ((setup.llm === 'gpu' || setup.voice === 'gpu') && !gpu) {
        r.error = 'no WebGPU here';
        continue;
      }
      const doneBefore = stepsDone;
      try {
        check();
        deviceTest.step = `Starting on the ${setup.llm.toUpperCase()}`;
        unloadLlm();
        useVoiceDevice(setup.voice);
        const t0 = performance.now();
        await ensureLlm({ device: setup.llm, model: model.id });
        r.loadMs = performance.now() - t0;
        // Warm-up: the first run of a session sets up its kernels.
        await generateText([{ role: 'user', content: 'Hi' }], { maxTokens: 2, temperature: 0 });
        await synthesize('Ready.', VOICE);
        check();
        advance(2);
        deviceTest.step = setup.voice === setup.llm ? `Testing on the ${setup.llm.toUpperCase()}` : `Testing: ${setup.llm.toUpperCase()}, voices on the ${setup.voice.toUpperCase()}`;
        const prefill: number[] = [];
        const speed: number[] = [];
        const speech: number[] = [];
        const lineMs: number[] = [];
        let loud = 0;
        let nanAudio = false;
        const { frames } = await watchFrames(async () => {
          for (const [i, ask] of asks.entries()) {
            const start = performance.now();
            const g = await generateText(ask.messages, { temperature: 0, maxTokens: 24, stopAt: ['"'] }, { reply: ask.reply, lead: ask.lead });
            check();
            advance();
            if (!g) continue;
            r.written++;
            r.nonFinite ||= g.nonFinite;
            prefill.push(g.prefillMs);
            if (g.decodeMs > 0) speed.push(g.ids.length / (g.decodeMs / 1000));
            const line = cleanLine(g.text, { me: 'Mia', names: ['Tom'], blocklist: text.blocklist });
            if (line) r.clean++;
            r.lines.push(line ?? `✗ ${g.text}`);
            const s0 = performance.now();
            const clip = await synthesize(line ?? 'Hello there.', VOICE);
            speech.push(performance.now() - s0);
            lineMs.push(performance.now() - start);
            loud += rms(clip.samples);
            nanAudio ||= clip.samples.some((v) => !Number.isFinite(v));
            // Hopelessly slow (the CPU, often): two lines tell; don't wait for five.
            if (i === 1 && median(lineMs) > LINE_MS * 3) break;
          }
        });
        r.ran = true;
        r.prefillMs = prefill[0] ?? 0;
        r.prefillCachedMs = median(prefill.slice(1));
        r.tokPerS = median(speed);
        r.speechMs = median(speech);
        r.lineMs = median(lineMs);
        r.frameP50 = median(frames);
        r.frameP95 = percentile(frames, 0.95);
        const meanRms = loud / Math.max(1, r.written);
        if (setup.voice === 'cpu' && !cpuRms) cpuRms = meanRms;
        r.voiceOk = !nanAudio && (setup.voice === 'cpu' || !cpuRms || Math.abs(meanRms - cpuRms) / cpuRms < 0.3);
        r.ok = !r.nonFinite && r.written > 0 && r.clean >= Math.ceil(r.written / 2) && r.voiceOk;
        r.usable = r.ok && r.frameP95 <= smoothLimit && r.lineMs <= LINE_MS;
      } catch (err) {
        if (deviceTest.stopping) throw new Stopped('stopped');
        r.error = String(err instanceof Error ? err.message : err);
      }
      // A setup cut short still counts as done.
      advance(doneBefore + perSetup - stepsDone);
    }
    const best = results.filter((x) => x.usable).sort((a, b) => a.lineMs - b.lineMs)[0];
    const report: DeviceReport = {
      model: model.version,
      gpu: deviceTest.gpu,
      date: new Date().toISOString(),
      idleP50: median(idle),
      idleP95: percentile(idle, 0.95),
      results,
      pick: best ? { llm: best.llm, voice: best.voice, setup: best.id } : null,
      voicesOk: results.some((x) => x.ran && x.voiceOk && x.speechMs <= SPEECH_MS),
    };
    deviceTest.reports = { ...deviceTest.reports, [reportKey(model)]: report };
    try {
      localStorage.setItem(KEY, JSON.stringify(deviceTest.reports));
    } catch {
      // Storage unavailable; the result still holds for this session.
    }
    return report;
  } catch (err) {
    if (!(err instanceof Stopped)) throw err;
    deviceTest.stopped = [...deviceTest.stopped, reportKey(model)];
    return null;
  } finally {
    deviceTest.running = false;
    deviceTest.stopping = false;
    deviceTest.step = '';
    deviceTest.progress = 0;
    // Back to how it was (the pick applies from the next line on).
    unloadLlm();
    useVoiceDevice(before.voice);
    if (before.llm) void ensureLlm({ device: before.llm, model: model.id }).catch(() => {});
  }
}

/**
 * What a report means for the settings: no usable setup switches the model's lines off (back to
 * hand-written ones); speech too slow everywhere switches voices off. Says so once.
 */
export function applyReport(report: DeviceReport, model: ModelSpec): void {
  if (!report.voicesOk && report.results.some((x) => x.ran)) {
    settings.voices = false;
    settings.voiceWriter = 'lines';
    toast('Resident voices are off: this computer makes speech too slowly for them.', 7000);
    return;
  }
  if (!report.pick && settings.voiceWriter === 'llm') {
    settings.voiceWriter = 'lines';
    const gpu = report.results.some((x) => x.llm === 'gpu' && x.ran);
    toast(
      gpu
        ? `${model.label} doesn't run well enough here: residents say hand-written lines.`
        : `${model.label} is too slow without a graphics card that supports WebGPU: residents say hand-written lines.`,
      7000,
    );
  }
}
