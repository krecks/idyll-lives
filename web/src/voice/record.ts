/**
 * A player's own voice for a resident, from the microphone or a sound file, made into a
 * `Recording`: compressed by the browser (`MediaRecorder`, Opus where it can, 32 kbit/s), at
 * most 12 seconds, so it fits in the save (about 50 KB as base64). Pocket TTS then speaks like
 * it. Nothing leaves this computer; the save keeps it.
 *
 * Kyutai's terms for Pocket TTS forbid cloning a voice without its owner's explicit consent, so
 * the creator asks for the player's own voice or someone's who agreed.
 */

import { sha256Hex } from '../ml/assets';
import type { Recording } from './voices';

export const MAX_SECONDS = 12;
export const MIN_SECONDS = 3;
const BITRATE = 32_000;
/** A sound file longer than this isn't read (a song, a podcast): the first 12 s are used anyway. */
const MAX_FILE_BYTES = 50 * 2 ** 20;

/** Whether this browser can record (microphone and `MediaRecorder`). */
export function canRecord(): boolean {
  return typeof MediaRecorder !== 'undefined' && !!navigator.mediaDevices?.getUserMedia;
}

function mimeType(): string {
  for (const t of ['audio/webm;codecs=opus', 'audio/ogg;codecs=opus', 'audio/mp4']) if (MediaRecorder.isTypeSupported(t)) return t;
  return '';
}

async function toRecording(blob: Blob): Promise<Recording> {
  const bytes = await blob.arrayBuffer();
  let bin = '';
  const view = new Uint8Array(bytes);
  for (let i = 0; i < view.length; i += 0x8000) bin += String.fromCharCode(...view.subarray(i, i + 0x8000));
  const audio = btoa(bin);
  return { audio, type: blob.type || 'audio/webm', sha: (await sha256Hex(new TextEncoder().encode(audio).buffer as ArrayBuffer)).slice(0, 16) };
}

/** Records `stream` until `stop` (or 12 s); `level` reads how loud it is now, 0..1. */
function record(stream: MediaStream, ctx: AudioContext, onEnd: () => void) {
  const recorder = new MediaRecorder(stream, { mimeType: mimeType() || undefined, audioBitsPerSecond: BITRATE });
  const chunks: Blob[] = [];
  recorder.ondataavailable = (e) => e.data.size && chunks.push(e.data);
  const done = new Promise<Blob>((resolve, reject) => {
    recorder.onstop = () => resolve(new Blob(chunks, { type: recorder.mimeType }));
    recorder.onerror = () => reject(new Error('recording failed'));
  });
  const analyser = ctx.createAnalyser();
  analyser.fftSize = 1024;
  ctx.createMediaStreamSource(stream).connect(analyser);
  const buf = new Float32Array(analyser.fftSize);
  const started = performance.now();
  const timer = setTimeout(() => stop(), MAX_SECONDS * 1000);
  recorder.start();
  function stop() {
    clearTimeout(timer);
    if (recorder.state !== 'inactive') recorder.stop();
    onEnd();
  }
  return {
    stop,
    seconds: () => (performance.now() - started) / 1000,
    level(): number {
      analyser.getFloatTimeDomainData(buf);
      let peak = 0;
      for (const v of buf) peak = Math.max(peak, Math.abs(v));
      return Math.min(1, peak * 1.4);
    },
    done: done.then(async (blob) => ({ recording: await toRecording(blob), seconds: (performance.now() - started) / 1000 })),
  };
}

export type Recorder = ReturnType<typeof record>;

/** Starts recording from the microphone (asks for permission). */
export async function recordMicrophone(): Promise<Recorder> {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: true, noiseSuppression: true, channelCount: 1 } });
  const ctx = new AudioContext();
  return record(stream, ctx, () => {
    stream.getTracks().forEach((t) => t.stop());
    void ctx.close();
  });
}

/**
 * A sound file made into a recording: decoded, its first 12 seconds played silently into the
 * recorder (so it's compressed like a recording; takes as long as the sound, at most 12 s).
 */
export async function recordFile(file: File, onProgress: (seconds: number) => void): Promise<Recording> {
  if (file.size > MAX_FILE_BYTES) throw new Error('that file is too big');
  const ctx = new AudioContext();
  try {
    const audio = await ctx.decodeAudioData(await file.arrayBuffer());
    if (audio.duration < MIN_SECONDS) throw new Error(`that sound is shorter than ${MIN_SECONDS} seconds`);
    const seconds = Math.min(MAX_SECONDS, audio.duration);
    const source = ctx.createBufferSource();
    source.buffer = audio;
    const sink = ctx.createMediaStreamDestination();
    source.connect(sink);
    const rec = record(sink.stream, ctx, () => source.stop());
    source.start(0, 0, seconds);
    const tick = setInterval(() => onProgress(rec.seconds()), 100);
    source.onended = () => rec.stop();
    try {
      return (await rec.done).recording;
    } finally {
      clearInterval(tick);
    }
  } finally {
    void ctx.close();
  }
}
