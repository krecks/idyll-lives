// Whether making speech makes the game worse (`voice/busyShare.ts`, as `voice/index.ts` counts it):
// a share of bad moments while speech is made, judged every so many of them and only against the
// share while it isn't. The frames and the simulation's seconds are counted apart, so the frames'
// short window (two seconds of frames) doesn't start the simulation's twenty seconds again: with
// smooth frames, a simulation held back by speech still pauses the voices.
import { describe, expect, test } from 'bun:test';
import { BusyShare } from '../src/voice/busyShare';

describe('busy share', () => {
  test('judged every window of busy moments, and only when speech makes it worse', () => {
    const share = new BusyShare(10, 0.3);
    // Four in ten slow while speech is made, none while it isn't: worse.
    for (let i = 0; i < 9; i++) expect(share.add(true, i < 4)).toBe(false);
    expect(share.add(true, false)).toBe(true);
    // Slow anyway (as slow without speech): not speech's fault.
    for (let i = 0; i < 10; i++) share.add(false, i < 5);
    let verdict = false;
    for (let i = 0; i < 10; i++) verdict = share.add(true, i < 4);
    expect(verdict).toBe(false);
  });

  test('the simulation falling behind pauses the voices though the frames are smooth', () => {
    // As the voice director counts: 60 frames a second, a simulation report each second, speech
    // being made all along; the frames smooth, the simulation behind every second.
    const frames = new BusyShare(120, 0.1);
    const sim = new BusyShare(20, 0.3);
    let simPaused = -1;
    for (let second = 1; second <= 30 && simPaused < 0; second++) {
      for (let f = 0; f < 60; f++) expect(frames.add(true, false)).toBe(false);
      if (sim.add(true, true)) simPaused = second;
    }
    expect(simPaused).toBe(20);
  });
});
