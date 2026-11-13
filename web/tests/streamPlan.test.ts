/**
 * What a game downloads first (`game/streamPlan.ts`): every game's files (the garden birds' models
 * among them) and the warm-up's lots' things with the house, and the town's model files read ahead of time in play nearest first,
 * not the focus lot's (built anyway) nor a file twice.
 */
import { beforeAll, describe, expect, test } from 'bun:test';
import type { AssetRegistry } from '../src/assets/registry';
import type { Content } from '../src/content/content';
import type { ObjectPlacement } from '../src/core/protocol';
import { gameAssetPlan, parseAheadFiles, streamRanks } from '../src/game/streamPlan';
import { BIRD_MODELS } from '../src/render/shared/birds';

beforeAll(() => {
  // (The plan makes URLs absolute against the page's, as the browser has it.)
  (globalThis as { location?: unknown }).location ??= { href: 'https://game.test/' };
});

const MODELS = ['model.chair', 'model.table', 'model.bed', 'model.lamp', 'model.sofa', ...BIRD_MODELS];

const assets = {
  has: (key: string, type: string) => type === 'model' && MODELS.includes(key),
  get: (key: string, type: string) => (type === 'model' && MODELS.includes(key) ? { type: 'model', url: `https://game.test/assets/${key}.glb` } : undefined),
  keys: () => [],
} as unknown as AssetRegistry;

const content = {
  object: (def: string) => ({ model: `model.${def}` }),
  shop: [],
  items: [],
} as unknown as Content;

const at = (def: string, x: number, z: number) => ({ def, x, z, w: 1, d: 1 }) as unknown as ObjectPlacement;

describe('stream plan', () => {
  test("the garden birds' models come with the house", () => {
    const { home } = gameAssetPlan(content, assets, { objects: [] }, null);
    for (const key of BIRD_MODELS) expect(home).toContain(`https://game.test/assets/${key}.glb`);
  });

  test("the warm-up's lots' things come with the house, the rest of the town's after", () => {
    const objects = [at('chair', 1, 1), at('table', 50, 50), at('bed', 200, 5)];
    const view = { x: 0, z: 0, w: 10, d: 10 };
    const file = (def: string) => `https://game.test/assets/model.${def}.glb`;
    const plain = gameAssetPlan(content, assets, { objects }, view);
    expect(plain.home).toContain(file('chair'));
    expect(plain.rest).toEqual(expect.arrayContaining([file('table'), file('bed')]));
    const warmed = gameAssetPlan(content, assets, { objects }, view, [{ x: 45, z: 45, w: 10, d: 10 }]);
    expect(warmed.home).toEqual(expect.arrayContaining([file('chair'), file('table')]));
    expect(warmed.rest).toEqual([file('bed')]);
  });

  test('files read ahead: round the camera first, then the nearest; not the focus lot, none twice', () => {
    const objects = [at('chair', 1, 1), at('table', 50, 50), at('bed', 200, 5), at('lamp', 52, 50), at('sofa', 90, 5), at('table', 51, 50)];
    const view = { x: 0, z: 0, w: 10, d: 10 };
    const watch = { x: 40, z: 40, w: 20, d: 20 };
    const ranks = streamRanks(content, assets, { objects }, view, watch);
    const files = parseAheadFiles(content, assets, { objects }, ranks);
    expect(files.map((f) => f.url.replace('https://game.test/assets/model.', ''))).toEqual(['table.glb', 'lamp.glb', 'sofa.glb', 'bed.glb']);
    // (Keyed for the stream as it keys its downloads: without the build's query.)
    expect(files[0].file).toBe('https://game.test/assets/model.table.glb');
    expect(parseAheadFiles(content, assets, { objects }, ranks, 2)).toHaveLength(2);
  });
});
