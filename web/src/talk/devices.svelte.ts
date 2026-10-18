/**
 * The device test: whether the language model and the voices run fast enough and right on this
 * computer. Both run only on the GPU (WebGPU): the model as its 4-bit build (on the CPU it took
 * 5–7 s a line), the voices with Supertonic 3. They work in their own workers; the GPU also draws
 * the game, so the test watches the frames too. Without WebGPU nothing runs.
 *
 * It writes five lines (greedy, within the writer's limits and time, 1 to 5 sentences) and
 * speaks them a piece at a time as a game does, measuring load time, prompt and writing speed,
 * the time from asking to having the first piece's audio, how fast a line's speech is made
 * against its length (it must keep up with playback), and frame times. Right means: no logits
 * that aren't numbers, lines that pass `cleanLine`, and speech whose samples are numbers. The
 * model's lines are used if it's right, smooth and in time; if not, or without WebGPU,
 * residents say hand-written lines; if even speech alone is too slow, voices are switched off
 * too (`applyReport`).
 *
 * Results are kept per model version, voice model and GPU (an M3 Pro, 2026-10-12: 0.5 s from
 * asking for a line to hearing it, speech made in a tenth of its length).
 */

import { ensureLlm, generateText, MODEL, unloadLlm } from '../llm';
import { hasWebGpu } from '../ml/ort';
import { settings } from '../settings/settings.svelte';
import { toast } from '../ui/state.svelte';
import { speechPieces } from '../voice/lines';
import { SAMPLE_RATE } from '../voice/protocol';
import { MODEL_ID as VOICE_MODEL_ID, synthesize } from '../voice/service.svelte';
import type { VoiceParams } from '../voice/voices';
import { cleanLine } from './clean';
import { buildMessages, type LineEvent, type Persona, type Situation, type TalkText } from './prompt';
import { cutShort, LINE_STOPS, LINE_TOKENS, WRITE_MS } from './writer';

export interface TestResult {
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
  /** Writing a line, making its first piece's speech (`speechPieces`), and from asking for a line to having that, ms (median). */
  writeMs: number;
  speechMs: number;
  lineMs: number;
  /** Time to make a whole line's speech over its length (median; above 1 the next piece comes late). */
  speechRtf: number;
  /** Sentences a line had (mean), and lines cut short by `WRITE_MS`. */
  sentences: number;
  cut: number;
  /** Frame times while working, ms: median and 95th percentile. */
  frameP50: number;
  frameP95: number;
  /** Logits that weren't numbers (a broken model or runtime). */
  nonFinite: boolean;
  /** Lines that passed `cleanLine`, of those written. */
  clean: number;
  written: number;
  /** No samples that aren't numbers. */
  voiceOk: boolean;
  /** Right: ran, numbers fine, most lines clean, voices fine. */
  ok: boolean;
  lines: string[];
}

export interface DeviceReport {
  model: string;
  gpu: string;
  date: string;
  /** Frame times with nothing running, ms. */
  idleP50: number;
  idleP95: number;
  result: TestResult;
  /** Right, frames smooth and lines in time: the model writes residents' lines. */
  usable: boolean;
  /** Speech alone is fast enough (else voices are off). */
  voicesOk: boolean;
}

/** Version 4: results are kept per language model, voice model and GPU; older results are dropped. */
const KEY = 'littlelives.deviceTests.4';
const OLD_KEYS = ['littlelives.deviceTests', 'littlelives.deviceTests.2', 'littlelives.deviceTests.3'];
/** A line must be written and spoken within this (the director gives a written line `MAX_WAIT_LLM_MS` in all). */
const LINE_MS = WRITE_MS;
/** Speech alone must be made within this (the voice speed test's "ok"). */
const SPEECH_MS = 3000;
/** A line's speech must be made faster than it plays, so its next piece is ready in time. */
const MAX_RTF = 1;
const VOICE: VoiceParams = { style: 'M1', pitch: 1, depth: 1, speed: 1 };

export const deviceTest = $state({
  running: false,
  /** What it's doing now, and how far it is, 0..1. */
  step: '',
  progress: 0,
  /** Asked to stop (`stopDeviceTest`). */
  stopping: false,
  /** The GPU's name ('none' without one; '' until known: results are kept per GPU). */
  gpu: '',
  /** Results by model version, voice model and GPU (`reportKey`). */
  reports: readReports(),
  /** Stopped this session: not started again on its own. */
  stopped: false,
});

/** Where the result for this computer is kept: per model version, voice model and GPU. */
export function reportKey(): string {
  return `${MODEL.version}|${VOICE_MODEL_ID}|${deviceTest.gpu}`;
}

/** The result on this computer, if it was tested. */
export function report(): DeviceReport | null {
  return deviceTest.reports[reportKey()] ?? null;
}

/** Whether the test found the model usable here. */
export function usableHere(): boolean {
  return report()?.usable ?? false;
}

/**
 * Firefox 158's WebGPU is much slower for these models on macOS (2026-10-18: 9.7 tokens a second,
 * 3.5–4.5 s a line, against 160–200 and 0.25–0.4 s in Chromium, 114 and 0.7 s in Safari), so a
 * failed test there is the browser, not the computer. Say so; the test still runs (a later
 * Firefox may be faster).
 */
export function isFirefox(): boolean {
  return /\bFirefox\//.test(navigator.userAgent);
}

function readReports(): Record<string, DeviceReport> {
  try {
    for (const key of OLD_KEYS) localStorage.removeItem(key);
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

/** Five moments to write lines for, asking for 1 to 5 sentences (one persona, so its state is cached after the first; also the settings' sample). */
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
  return moments.map(([s, e], i) => buildMessages(text, persona, { ...base, ...s }, e, [], i + 1));
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

/** Runs the test; null if it was stopped (`stopDeviceTest`). */
export async function runDeviceTest(text: TalkText): Promise<DeviceReport | null> {
  if (deviceTest.running) throw new Error('the device test is already running');
  deviceTest.running = true;
  deviceTest.stopping = false;
  deviceTest.progress = 0;
  const check = () => {
    if (deviceTest.stopping) throw new Stopped('stopped');
  };
  try {
    deviceTest.gpu = await adapterName();
    deviceTest.step = 'Measuring frames';
    const idle = (await watchFrames(() => new Promise((r) => setTimeout(r, 1000)))).frames;
    // Smooth enough: the slow frames no worse than two and a half typical idle ones (or 40 ms).
    // (The idle 95th percentile catches page loads and the like: the median is steadier.)
    const smoothLimit = Math.max(median(idle) * 2.5, 40);
    const asks = testPrompts(text).filter((p): p is NonNullable<typeof p> => !!p);
    // Progress: loading counts as two lines' worth.
    const steps = asks.length + 2;
    let stepsDone = 0;
    const advance = (n = 1) => {
      stepsDone += n;
      deviceTest.progress = Math.min(1, stepsDone / steps);
    };
    const r: TestResult = { ran: false, loadMs: 0, prefillMs: 0, prefillCachedMs: 0, tokPerS: 0, writeMs: 0, speechMs: 0, lineMs: 0, speechRtf: 0, sentences: 0, cut: 0, frameP50: 0, frameP95: 0, nonFinite: false, clean: 0, written: 0, voiceOk: false, ok: false, lines: [] };
    if (!(await hasWebGpu())) {
      r.error = 'no WebGPU here';
    } else {
      try {
        check();
        deviceTest.step = 'Starting the model';
        unloadLlm();
        const t0 = performance.now();
        await ensureLlm();
        r.loadMs = performance.now() - t0;
        // Warm-up: the first run of a session sets up its kernels.
        await generateText([{ role: 'user', content: 'Hi' }], { maxTokens: 2, temperature: 0 });
        await synthesize('Ready.', VOICE);
        check();
        advance(2);
        deviceTest.step = 'Testing on the GPU';
        const prefill: number[] = [];
        const speed: number[] = [];
        const write: number[] = [];
        const speech: number[] = [];
        const rtf: number[] = [];
        const lineMs: number[] = [];
        let sentences = 0;
        let nanAudio = false;
        const { frames } = await watchFrames(async () => {
          for (const [i, ask] of asks.entries()) {
            const start = performance.now();
            // As in a game: the writer's limits, and its time (cut short, the finished sentences are said).
            const g = await generateText(ask.messages, { temperature: 0, maxTokens: LINE_TOKENS, stopAt: LINE_STOPS, sentences: i + 1 }, { reply: ask.reply, lead: ask.lead, deadline: Date.now() + WRITE_MS });
            check();
            advance();
            if (!g) continue;
            write.push(performance.now() - start);
            r.written++;
            if (g.stop === 'deadline') r.cut++;
            r.nonFinite ||= g.nonFinite;
            prefill.push(g.prefillMs);
            if (g.decodeMs > 0) speed.push(g.ids.length / (g.decodeMs / 1000));
            const line = cleanLine(g.text, { me: 'Mia', names: ['Tom'], blocklist: text.blocklist, cut: cutShort(g) });
            if (line) r.clean++;
            r.lines.push(line ?? `✗ ${g.text}`);
            sentences += line?.match(/[.!?…]+/g)?.length ?? 0;
            // Spoken a piece at a time, as the voice director does: the first piece is heard once it's made.
            const s0 = performance.now();
            let samples = 0;
            for (const [k, piece] of speechPieces(line ?? 'Hello there.').entries()) {
              const clip = await synthesize(piece, VOICE);
              if (k === 0) {
                speech.push(performance.now() - s0);
                lineMs.push(performance.now() - start);
              }
              samples += clip.samples.length;
              nanAudio ||= clip.samples.some((v) => !Number.isFinite(v));
            }
            rtf.push((performance.now() - s0) / 1000 / Math.max(0.1, samples / SAMPLE_RATE));
            // Hopelessly slow: two lines tell; don't wait for five.
            if (i === 1 && median(lineMs) > LINE_MS * 3) break;
          }
        });
        r.ran = true;
        r.prefillMs = prefill[0] ?? 0;
        r.prefillCachedMs = median(prefill.slice(1));
        r.tokPerS = median(speed);
        r.writeMs = median(write);
        r.speechMs = median(speech);
        r.lineMs = median(lineMs);
        r.speechRtf = median(rtf);
        r.sentences = sentences / Math.max(1, r.written);
        r.frameP50 = median(frames);
        r.frameP95 = percentile(frames, 0.95);
        r.voiceOk = !nanAudio;
        r.ok = !r.nonFinite && r.written > 0 && r.clean >= Math.ceil(r.written / 2) && r.voiceOk;
        advance(steps - stepsDone);
        return save({
          model: MODEL.version,
          gpu: deviceTest.gpu,
          date: new Date().toISOString(),
          idleP50: median(idle),
          idleP95: percentile(idle, 0.95),
          result: r,
          usable: r.ok && r.frameP95 <= smoothLimit && r.lineMs <= LINE_MS && r.speechRtf <= MAX_RTF,
          voicesOk: r.voiceOk && r.speechMs <= SPEECH_MS && r.speechRtf <= MAX_RTF,
        });
      } catch (err) {
        if (deviceTest.stopping) throw new Stopped('stopped');
        r.error = String(err instanceof Error ? err.message : err);
      }
    }
    return save({ model: MODEL.version, gpu: deviceTest.gpu, date: new Date().toISOString(), idleP50: median(idle), idleP95: percentile(idle, 0.95), result: r, usable: false, voicesOk: false });
  } catch (err) {
    if (!(err instanceof Stopped)) throw err;
    deviceTest.stopped = true;
    return null;
  } finally {
    deviceTest.running = false;
    deviceTest.stopping = false;
    deviceTest.step = '';
    deviceTest.progress = 0;
    // The model loads again if its lines are used (`talk.svelte.ts`).
    unloadLlm();
  }
}

/** Keeps a report for this computer. */
function save(r: DeviceReport): DeviceReport {
  deviceTest.reports = { ...deviceTest.reports, [reportKey()]: r };
  try {
    localStorage.setItem(KEY, JSON.stringify(deviceTest.reports));
  } catch {
    // Storage unavailable; the result still holds for this session.
  }
  return r;
}

/**
 * What a report means for the settings: not usable switches the model's lines off (back to
 * hand-written ones); speech too slow switches voices off. Says so once.
 */
export function applyReport(r: DeviceReport): void {
  if (!r.voicesOk && r.result.ran) {
    settings.voices = false;
    settings.voiceWriter = 'lines';
    toast('Resident voices are off: this computer makes speech too slowly for them.', 7000);
    return;
  }
  if (!r.usable && settings.voiceWriter === 'llm') {
    settings.voiceWriter = 'lines';
    const hint = r.result.ran && isFirefox() ? " Firefox's WebGPU is the slow part: Chrome, Edge or Safari run it in under a second a line." : '';
    toast(
      r.result.ran
        ? `${MODEL.label} doesn't run well enough here: residents say hand-written lines.${hint}`
        : `${MODEL.label} needs a graphics card that supports WebGPU: residents say hand-written lines.${hint}`,
      hint ? 9000 : 7000,
    );
  }
}
