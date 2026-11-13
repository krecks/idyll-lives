// Where a sound is heard from (`audio/acoustics.ts`, `audio/rooms.ts`, and what they do to the samples:
// `audio/offline.ts`, the same arithmetic as the mix's graph in `audio/space.ts`): the camera's distance leaves a
// sound alone at the game's own zoom and never silences it; a room's response is a quiet, short, wide tail of
// energy 1; a sound in a room, behind a wall or far off stays within the mix.
import { describe, expect, test } from 'bun:test';
import { Acoustics, DISTANCE, distanceCutoff, distanceGain, focus, muffled, OCCLUDE } from '../src/audio/acoustics';
import { everySound, GUARD } from '../src/audio/catalog';
import { loudness } from '../src/audio/loudness';
import { convolve, spaced } from '../src/audio/offline';
import { ROOM_SIZES, ROOMS, roomResponse, roomSize } from '../src/audio/rooms';
import type { Stereo } from '../src/audio/water';
import { DEFAULT_ORBIT, RADIUS_LIMITS, TOWN_FULL, TOWN_MAX } from '../src/render/shared/lens';

const RATE = 48000;
const db = (x: number) => 20 * Math.log10(x);
const listed = everySound();
const hear = (name: string): Stereo => listed.find((l) => l.name === name)!.heard(RATE);
const lufs = (s: Stereo) => loudness(s.left, s.right, RATE);

describe('the camera\'s distance', () => {
  test('at the game\'s own zoom and closer a sound is as it was made', () => {
    for (const r of [RADIUS_LIMITS[0], 12, DEFAULT_ORBIT.radius]) {
      expect(distanceGain(r)).toBe(1);
      expect(distanceCutoff(r)).toBe(DISTANCE.open);
    }
    // (The filter passes everything from half the sample rate up.)
    expect(DISTANCE.open).toBeGreaterThanOrEqual(RATE / 2);
    const tv = hear('tv');
    const near = spaced(tv, RATE, { radius: DEFAULT_ORBIT.radius }, true);
    expect(near.left).toEqual(tv.left);
    expect(near.right).toEqual(tv.right);
  });

  test('further out it falls and dulls, and never to nothing', () => {
    let gain = 1;
    let cutoff = DISTANCE.open;
    for (let r = DEFAULT_ORBIT.radius + 1; r <= TOWN_MAX; r += 1) {
      const g = distanceGain(r);
      const c = distanceCutoff(r);
      expect(g).toBeLessThanOrEqual(gain);
      expect(c).toBeLessThanOrEqual(cutoff);
      gain = g;
      cutoff = c;
    }
    // Twice as far, half the level; the play camera's farthest is the floor, 12 dB down; the town view 20 dB.
    expect(db(distanceGain(48))).toBeCloseTo(-6.02, 1);
    expect(db(distanceGain(RADIUS_LIMITS[1]))).toBeCloseTo(-11.95, 1);
    expect(db(distanceGain(TOWN_FULL))).toBeCloseTo(-20, 1);
    expect(distanceGain(TOWN_MAX)).toBeGreaterThanOrEqual(DISTANCE.town);
    expect(distanceCutoff(TOWN_MAX)).toBe(DISTANCE.least);
    // A camera that says nothing sensible is the game's own.
    expect(distanceGain(0)).toBe(1);
    expect(distanceGain(NaN)).toBe(1);
  });

  test('a far sound is that much quieter to the ear, and duller', () => {
    const shower = hear('shower');
    const far = spaced(shower, RATE, { radius: RADIUS_LIMITS[1] }, true);
    // 12 dB of level, and what the low-pass takes of a bright sound.
    expect(lufs(far) - lufs(shower)).toBeLessThan(-12);
    expect(lufs(far) - lufs(shower)).toBeGreaterThan(-16);
  });
});

describe('walls', () => {
  test('closed in on a room the others are muffled; from the game\'s own zoom out nothing is', () => {
    expect(focus(RADIUS_LIMITS[0])).toBe(1);
    expect(focus(OCCLUDE.near)).toBe(1);
    expect(focus(17)).toBeCloseTo(0.5, 1);
    expect(focus(DEFAULT_ORBIT.radius)).toBe(0);
    expect(focus(TOWN_MAX)).toBe(0);
    expect(muffled(0)).toEqual({ cutoff: DISTANCE.open, gain: 1 });
    expect(muffled(1).cutoff).toBeCloseTo(OCCLUDE.cutoff, 6);
    expect(db(muffled(1).gain)).toBeCloseTo(-4, 1);
  });

  test('which room a sound is in, how big it is, and whether it is the one in view', () => {
    // A lot 8 wide, two storeys of 4 rows: a 2 × 2 bathroom (1) and a 4 × 3 kitchen (2) on the ground, a 6 × 4 loft (3) above.
    const rooms = new Uint16Array(8 * 8);
    for (let z = 0; z < 2; z++) for (let x = 0; x < 2; x++) rooms[z * 8 + x] = 1;
    for (let z = 0; z < 3; z++) for (let x = 3; x < 7; x++) rooms[z * 8 + x] = 2;
    for (let z = 4; z < 8; z++) for (let x = 0; x < 6; x++) rooms[z * 8 + x] = 3;
    const a = new Acoustics();
    expect(a.sizeAt(1, 1)).toBe(null);
    a.setWorld({ width: 8, depth: 8, rooms });
    expect(a.roomAt(0.5, 1.9)).toBe(1);
    expect(a.roomAt(7.5, 0.5)).toBe(0);
    expect(a.roomAt(-1, 0)).toBe(0);
    expect(a.roomAt(3, 99)).toBe(0);
    expect(a.sizeAt(0.5, 0.5)).toBe('small');
    expect(a.sizeAt(4, 1)).toBe('medium');
    expect(a.sizeAt(7.5, 3.5)).toBe(null);
    expect(a.sizeAt(2, 6)).toBe('medium');
    // Close in on the kitchen: the bathroom, the loft and the garden are behind walls.
    a.look(4, 1, 8);
    expect(a.muffleAt(5, 2)).toBe(0);
    expect(a.muffleAt(0.5, 0.5)).toBe(1);
    expect(a.muffleAt(2, 6)).toBe(1);
    expect(a.muffleAt(7.5, 3.5)).toBe(1);
    // From the game's own zoom the whole house is in view.
    a.look(4, 1, DEFAULT_ORBIT.radius);
    expect(a.muffleAt(0.5, 0.5)).toBe(0);
    // Looking at the garden from close: indoors is behind a wall, the garden is not.
    a.look(7.5, 3.5, 8);
    expect(a.muffleAt(7.2, 0.5)).toBe(0);
    expect(a.muffleAt(4, 1)).toBe(1);
    expect(roomSize(9)).toBe('small');
    expect(roomSize(10)).toBe('medium');
    expect(roomSize(500)).toBe('large');
  });

  test('a muffled sound is quieter and has lost its highs', () => {
    const shower = hear('shower');
    const through = spaced(shower, RATE, { muffle: 1 }, true);
    expect(lufs(through) - lufs(shower)).toBeLessThan(-4);
    const highs = (s: Stereo) => {
      let sum = 0;
      for (let i = 1; i < s.left.length; i++) sum += (s.left[i] - s.left[i - 1]) ** 2;
      return sum;
    };
    expect(highs(through)).toBeLessThan(highs(shower) / 50);
  });
});

describe('rooms', () => {
  for (const size of ROOM_SIZES) {
    test(`${size}: a short, wide tail of energy 1, the same every time`, () => {
      const { left, right } = roomResponse(size, RATE);
      expect(roomResponse(size, RATE).left).toEqual(left);
      let energy = 0;
      let cross = 0;
      let late = 0;
      for (let i = 0; i < left.length; i++) {
        energy += left[i] ** 2 + right[i] ** 2;
        cross += left[i] * right[i];
        if (i > left.length * 0.75) late += left[i] ** 2 + right[i] ** 2;
      }
      expect(Number.isFinite(energy)).toBe(true);
      expect(energy / 2).toBeCloseTo(1, 5);
      // No longer than its decay says, nearly gone by its last quarter, nothing at its very end.
      expect(left.length / RATE).toBeLessThan(0.8);
      expect(late / energy).toBeLessThan(0.002);
      expect(Math.abs(left[left.length - 1])).toBeLessThan(1e-3);
      // The two sides differ (a wide room), and it holds no copy of the sound itself (nothing at time 0).
      expect(Math.abs(cross) / (energy / 2)).toBeLessThan(0.1);
      expect(left[0]).toBe(0);
      expect(right[0]).toBe(0);
      // Quiet against the sound: 13 to 15 dB under.
      expect(db(ROOMS[size].send)).toBeLessThan(-12);
      expect(db(ROOMS[size].send)).toBeGreaterThan(-16);
    });
  }

  test('a click through a room comes back as the room\'s response', () => {
    const { left } = roomResponse('small', RATE);
    const click = new Float32Array(100);
    click[10] = 0.5;
    const out = convolve(click, left, false);
    expect(out.length).toBe(100 + left.length - 1);
    for (const i of [0, 200, 1000, 5000]) expect(out[10 + i]).toBeCloseTo(0.5 * left[i], 6);
    // Round a loop, nothing is lost: the tail comes back in at the start.
    const sum = (d: Float32Array) => d.reduce((s, v) => s + v, 0);
    expect(sum(convolve(click, left, true))).toBeCloseTo(sum(out), 4);
  });

  test('in a room a sound stays within the mix, hardly louder than dry', () => {
    // The loudest loop, the quietest, a sound of one channel, a game's knocks and a sound played once.
    for (const name of ['shower', 'potteryWheel', 'tv', 'pool', 'fridgeDoor']) {
      const l = listed.find((s) => s.name === name)!;
      const dry = hear(name);
      for (const size of ROOM_SIZES) {
        const wet = spaced(dry, RATE, { room: size }, l.plays === 'loop');
        const loud = lufs(wet);
        expect(loud).toBeGreaterThan(GUARD.things[0]);
        expect(loud).toBeLessThan(GUARD.things[1]);
        if (l.plays === 'loop') {
          expect(loud - lufs(dry)).toBeGreaterThan(-0.1);
          expect(loud - lufs(dry)).toBeLessThan(0.6);
        }
        let peak = 0;
        for (const v of wet.left) peak = Math.max(peak, Math.abs(v));
        expect(peak).toBeLessThan(1);
      }
    }
  }, 60_000);
});
