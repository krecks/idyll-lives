/**
 * A face per resident: the looks' faces (`household.ts`) and how a resident's row takes one (`identity.ts`).
 */

import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import type { AssetRegistry } from '../src/assets/registry';
import { FACE_CHANNELS, FACE_GROUPS, faceFromId, lookFromSeed, randomFace, randomLook } from '../src/game/household';
import { mulberry32 } from '../src/game/noise';
import type { BodyData, MorphSet } from '../src/render/shared/characters/data';
import { faceOf, writeIdentity } from '../src/render/shared/characters/identity';
import { IDENTITY_PAIRS } from '../src/render/shared/characters/layout';

/** No palettes: every colour is the fallback, the faces are what is looked at. */
const assets = { get: () => undefined } as unknown as AssetRegistry;

describe('faces', () => {
  test('a random face stays within -1..1, mostly near the shared face', () => {
    const rand = mulberry32(7);
    let near = 0;
    let sum = 0;
    const n = 500;
    for (let k = 0; k < n; k++) {
      const face = randomFace(rand);
      expect(Object.keys(face)).toEqual([...FACE_CHANNELS]);
      for (const v of Object.values(face)) {
        expect(Math.abs(v)).toBeLessThanOrEqual(1);
        if (Math.abs(v) <= 0.6) near++;
        sum += v;
      }
    }
    expect(near / (n * FACE_CHANNELS.length)).toBeGreaterThan(0.85);
    expect(Math.abs(sum / (n * FACE_CHANNELS.length))).toBeLessThan(0.02);
  });

  test('the creator shows every channel once', () => {
    expect(FACE_GROUPS.flatMap((g) => g.channels.map((c) => c.id)).toSorted()).toEqual([...FACE_CHANNELS].toSorted());
  });

  test('a look without a face gets one from the id: always the same, and nobody else has it', () => {
    expect(faceFromId(12)).toEqual(faceFromId(12));
    const faces = new Set(Array.from({ length: 208 }, (_, id) => JSON.stringify(faceFromId(id))));
    expect(faces.size).toBe(208);
    const own = { jawWidth: 0.4 };
    expect(faceOf({ face: own } as never, 3)).toBe(own);
    expect(faceOf({} as never, 3)).toBe(faceFromId(3));
    expect(faceOf(undefined, 3)).toBe(faceFromId(3));
  });

  test("a random look's face takes nothing from the look's dice", () => {
    const counted = (seed: number) => {
      const rand = mulberry32(seed);
      const dice = { n: 0, roll: () => (dice.n++, rand()) };
      return dice;
    };
    const [a, b] = [counted(3), counted(3)];
    const own = randomLook(assets, 'male', a.roll);
    const given = randomLook(assets, 'male', b.roll, b.roll, mulberry32(99));
    // (The same dice either way: a town made from a seed is the town it was before faces existed.)
    expect(a.n).toBe(b.n);
    expect({ ...own, face: undefined }).toEqual({ ...given, face: undefined });
    expect(Object.keys(own.face ?? {})).toEqual([...FACE_CHANNELS]);
    expect(randomLook(assets, 'male', counted(3).roll).face).toEqual(own.face);
    // Another look, another face.
    expect(randomLook(assets, 'male', counted(4).roll).face).not.toEqual(own.face);
  });

  test('a look from a seed has a face, the same every time, and a child one between its parents', () => {
    const a = lookFromSeed(assets, 'female', 101);
    expect(lookFromSeed(assets, 'female', 101)).toEqual(a);
    expect(Object.keys(a.face ?? {})).toEqual([...FACE_CHANNELS]);
    const b = lookFromSeed(assets, 'male', 202);
    expect(b.face).not.toEqual(a.face);
    for (let seed = 1; seed <= 50; seed++) {
      const child = lookFromSeed(assets, 'female', seed, [a, b]);
      for (const c of FACE_CHANNELS) {
        const [lo, hi] = [Math.min(a.face![c], b.face![c]), Math.max(a.face![c], b.face![c])];
        // (Its own share is half a random face's: at most 0.5 either way.)
        expect(child.face![c]).toBeGreaterThanOrEqual(Math.max(-1, lo - 0.51));
        expect(child.face![c]).toBeLessThanOrEqual(Math.min(1, hi + 0.51));
      }
    }
    // A parent from an older save has no face: the shared one stands in.
    expect(Object.values(lookFromSeed(assets, 'male', 5, [{ ...a, face: undefined }]).face ?? {}).every((v) => Math.abs(v) <= 0.5)).toBe(true);
  });

  test("the row's identity texels: a pair a channel, scaled to the body's stage, and the eyes' share", () => {
    const channels = ['sil', 'jawOpen', ...FACE_CHANNELS.map((c) => `id.${c}`)];
    const eyes = new Float32Array(FACE_CHANNELS.length * 4);
    eyes.set([0.002, 0, 0, 0], FACE_CHANNELS.indexOf('eyeSpacing') * 4);
    eyes.set([0, 0, 0, 0.07], FACE_CHANNELS.indexOf('eyeSize') * 4);
    const morphs = { channels, channel: Object.fromEntries(channels.map((c, i) => [c, i])), expressions: 2, eyes: { female: eyes } } as unknown as MorphSet;
    const body = { family: 'female', faceScale: 0.5 } as BodyData;
    const out = new Float32Array(4 + IDENTITY_PAIRS * 2 + 4).fill(9);
    const moved = new Float32Array(4).fill(9);
    writeIdentity(morphs, body, { jawWidth: 1, eyeSpacing: -1, eyeSize: 3, nothing: 1, chinLength: Number.NaN }, out, 4, moved);
    // (Only its own texels.)
    expect([...out.subarray(0, 4), ...out.subarray(4 + IDENTITY_PAIRS * 2)]).toEqual([9, 9, 9, 9, 9, 9, 9, 9]);
    const pairs = Object.fromEntries(FACE_CHANNELS.map((c, k) => [c, [out[4 + k * 2], out[5 + k * 2]]]));
    expect(pairs.jawWidth).toEqual([2 + FACE_CHANNELS.indexOf('jawWidth'), 0.5]);
    expect(pairs.eyeSpacing[1]).toBe(-0.5);
    // (Past 1 is 1; what isn't a number is the shared face's.)
    expect(pairs.eyeSize[1]).toBe(0.5);
    expect(pairs.chinLength[1]).toBe(0);
    expect(pairs.noseWidth[1]).toBe(0);
    expect(moved[0]).toBeCloseTo(-0.001, 6);
    expect(moved[3]).toBeCloseTo(0.035, 6);
    expect([moved[1], moved[2]]).toEqual([0, 0]);
  });

  test("the character set's identity channels are the creator's, and fit the row", () => {
    const rig = JSON.parse(readFileSync(new URL('../public/assets/characters/rig.json', import.meta.url), 'utf8')) as { morphs: { channels: string[]; identity: { first: number; count: number; eyes: Record<string, Record<string, number[]>> } } };
    const { first, count, eyes } = rig.morphs.identity;
    expect(rig.morphs.channels.slice(first)).toEqual(FACE_CHANNELS.map((c) => `id.${c}`));
    expect(count).toBe(FACE_CHANNELS.length);
    expect(count).toBeLessThanOrEqual(IDENTITY_PAIRS);
    for (const moved of Object.values(eyes)) for (const c of Object.keys(moved)) expect(rig.morphs.channels.indexOf(c)).toBeGreaterThanOrEqual(first);
  });
});
