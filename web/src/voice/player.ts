/**
 * Plays voice clips through Web Audio: the voice bus (`audio/mixer.ts`, "Voice volume"), each clip at
 * its place (`audio/space.ts`): a stereo panner so a voice comes from where the resident is on screen,
 * the reverb of the room they speak in, and a wall's muffling when the camera is in on another room.
 * A voice is not made quieter by the camera's distance. While a clip plays the world's sounds give way
 * to it (`voiceSpeaking`).
 */

import { audioContext, bus, onHide, space, updateVolumes, voiceSpeaking } from '../audio/mixer';
import { Placed, type Place } from '../audio/space';
import { SAMPLE_RATE } from './protocol';

export interface Playing {
  /** Where the speaker is now (left to right, their room, behind a wall or not). */
  setPlace(place: Place): void;
  stop(): void;
  readonly ended: Promise<void>;
  /** When its first sample is heard: seconds on the `performance.now()` clock (output latency included). */
  readonly startTime: number;
}

/** Clips playing now. A hidden tab stops them: its audio stops, so they would never reach their end (lines wait for it). */
const playing = new Set<() => void>();
onHide(() => {
  for (const stop of [...playing]) stop();
});

/** Plays a clip at `place`; without one (the menus), in the middle with no room. */
export function playClip(samples: Float32Array, place: Place = { pan: 0 }): Playing | null {
  const ac = audioContext();
  const voices = bus('voices');
  const around = space('voices');
  if (!ac || !voices || !around || samples.length === 0) return null;
  const buffer = ac.createBuffer(1, samples.length, SAMPLE_RATE);
  buffer.copyToChannel(samples as Float32Array<ArrayBuffer>, 0);
  const src = ac.createBufferSource();
  src.buffer = buffer;
  const placed = new Placed(ac, around, place);
  src.connect(placed.input);
  const stop = () => {
    try {
      src.stop();
    } catch {
      // Already stopped.
    }
    done();
  };
  // Stopping ends it at once, even while the audio is stopped and its `ended` event can't come.
  let done = () => {};
  const ended = new Promise<void>((resolve) => {
    done = () => {
      // (Once: a clip stopped by hand also gets its `ended` event.)
      if (playing.delete(stop)) {
        voiceSpeaking(false);
        // (Its room goes on a moment after it: the longest response is under a second.)
        setTimeout(() => placed.disconnect(), 1000);
      }
      resolve();
    };
  });
  src.onended = () => done();
  playing.add(stop);
  voiceSpeaking(true);
  src.start();
  const startTime = performance.now() / 1000 + (ac.outputLatency || ac.baseLatency || 0);
  return {
    setPlace(p) {
      placed.place(p, ac.currentTime);
    },
    stop,
    ended,
    startTime,
  };
}

/** Applies a changed "Voice volume" right away. */
export function updateVoiceVolume(): void {
  updateVolumes();
}
