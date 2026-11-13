/**
 * Little interface sounds, synthesised (no sound files; `audio/blips.ts` makes them): picking things
 * up, putting them down, money changing hands, walls going up. Quiet by design, and off with the
 * "Sound effects" setting. Played through the mixer (`audio/mixer.ts`).
 */

import { blipSamples, type Blip } from '../audio/blips';
import { audioContext, bus } from '../audio/mixer';
import { settings } from '../settings/settings.svelte';

export type Sound = Blip;

/** The same sound again within this many ms is skipped (a burst of hover moves, a held key). */
const DEBOUNCE_MS = 45;

/** The sounds as audio buffers, each made the first time it plays (again at another sample rate). */
const made = new Map<Sound, AudioBuffer>();
const last = new Map<Sound, number>();

export function play(sound: Sound): void {
  if (!settings.sound) return;
  const now = performance.now();
  if (now - (last.get(sound) ?? -Infinity) < DEBOUNCE_MS) return;
  last.set(sound, now);
  const ac = audioContext();
  const out = bus('effects');
  if (!ac || !out) return;
  let buffer = made.get(sound);
  if (!buffer || buffer.sampleRate !== ac.sampleRate) {
    const samples = blipSamples(sound, ac.sampleRate);
    buffer = new AudioBuffer({ numberOfChannels: 1, length: samples.length, sampleRate: ac.sampleRate });
    buffer.copyToChannel(samples as Float32Array<ArrayBuffer>, 0);
    made.set(sound, buffer);
  }
  const source = ac.createBufferSource();
  source.buffer = buffer;
  source.connect(out);
  source.onended = () => source.disconnect();
  source.start(ac.currentTime + 0.005);
}
