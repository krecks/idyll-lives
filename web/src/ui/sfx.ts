/**
 * Little interface sounds, synthesised (no sound files; `crates/sound/src/blips.rs` makes them, through `audio/synth.ts`): picking things
 * up, putting them down, money changing hands, walls going up; and in Live a pie menu opening and an item
 * chosen, the clock's speed, money in and out and the household's milestones. Quiet by design, and off with
 * the "Sound effects" setting (at the "Effects volume" when on). Played through the mixer (`audio/mixer.ts`).
 */

import type { Blip } from '../audio/kinds';
import { audioContext, bus } from '../audio/mixer';
import { blipSamples, synthReady } from '../audio/synth';
import { settings } from '../settings/settings.svelte';

export type Sound = Blip;

/** The same sound again within this many ms is skipped (a burst of hover moves, a held key). */
const DEBOUNCE_MS = 45;
/** Sounds that would nag if they came often: skipped (never queued) within this many ms of the last. */
const GAP_MS: Partial<Record<Sound, number>> = { moneyIn: 1500, moneyOut: 1500, notify: 4000, notifyLow: 4000 };
/** Sounds that share another's clock for the gap: money in and out, the two chimes. */
const CLOCK: Partial<Record<Sound, Sound>> = { moneyOut: 'moneyIn', notifyLow: 'notify' };

/** The sounds as audio buffers, each made the first time it plays (again at another sample rate). */
const made = new Map<Sound, AudioBuffer>();
const last = new Map<Sound, number>();

export function play(sound: Sound): void {
  if (!settings.sound) return;
  const now = performance.now();
  const clock = CLOCK[sound] ?? sound;
  if (now - (last.get(clock) ?? -Infinity) < (GAP_MS[sound] ?? DEBOUNCE_MS)) return;
  last.set(clock, now);
  const ac = audioContext();
  const out = bus('effects');
  if (!ac || !out) return;
  let buffer = made.get(sound);
  if (!buffer || buffer.sampleRate !== ac.sampleRate) {
    // (Not before the sounds can be made: the page loads them as the game boots, `synth.ts`.)
    if (!synthReady()) return;
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
