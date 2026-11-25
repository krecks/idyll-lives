// "Lay the town again" (`macro.ts` `generateFromSpec`'s `shaped`, step 1): a lake painted over a world's main
// town, the town laid again on that land, gives one road network as the sim joins its roads, no lot or road
// in the water, a bridge no longer than the generator's over a river that is now the land's to the villages across it,
// and the world keeps its name and its main town's shape; the draft carries the land (graded flat under the
// town) and its water and zones come from it. Unshaped worlds are as they were (the classic and world-option
// goldens). `SLOW_TESTS=1 bun test tests/relaidTown.slow.test.ts`.
import { describe, expect, test } from 'bun:test';
import { networks } from './support/roads';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('the town laid again on a shaped land', async () => {
  usePublicFetch();
  const { generateWorld, plotRect, useTownContent } = await import('../src/game/town');
  const { LandEditor, surveyDraft, wetTile } = await import('../src/game/landEdit');
  const { unrle } = await import('../src/game/landMasks');
  const { LAND_CELL } = await import('../src/game/land');
  const { deckTiles } = await import('../src/game/macro');
  type WorldOptions = import('../src/game/world').WorldOptions;
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  const WORLDS: [string, number, Partial<WorldOptions>][] = [
    ['country', 4242, { town: 'village', villages: 'few' }],
    ['river', 7, { town: 'village', villages: 'few' }],
  ];

  for (const [kind, seed, options] of WORLDS) {
    test(
      `${kind} ${seed}: a lake over the town, the town laid again`,
      () => {
        const draft = generateWorld(content, t, kind, seed, undefined, undefined, options);
        const land = surveyDraft(draft);
        const editor = new LandEditor(land, () => 1);
        // A lake 120 m across over the main town's middle.
        const a = draft.open!.areas[0];
        const [cx, cz] = [a.x + a.w / 2, a.z + a.d / 2];
        editor.begin({ tool: 'lake', radius: 60, strength: 0.6 }, cx, cz);
        editor.end();
        expect(wetTile(land, Math.floor(cx), Math.floor(cz))).toBe(true);
        expect(land.edited).toBe(true);

        const relaid = generateWorld(content, t, kind, seed, undefined, undefined, options, land);
        const tag = `${kind} ${seed}`;
        // Its name, its size, its main town's shape (as many streets), a town.
        expect(relaid.name).toBe(draft.name);
        expect(relaid.width).toBe(land.cols * LAND_CELL);
        expect(relaid.roads.filter((r) => r.area === 0 && r.kind !== 'trail' && r.kind !== 'country').length, tag).toBe(draft.roads.filter((r) => r.area === 0 && r.kind !== 'trail' && r.kind !== 'country').length);
        // (Most of its homes: those of villages over the river, now off the town's piece of land, are not.)
        const homes = (d: typeof draft) => d.slots.filter((s) => s.kind === 'house').length;
        expect(homes(relaid)).toBeGreaterThan(homes(draft) * 0.6);
        // One road network.
        expect(networks(relaid.roads), `${tag}: networks`).toBe(1);
        // The river is the land's now: a bridge over it to the villages across it, no longer than the generator's.
        const decks = relaid.roads.filter((r) => r.bridge && r.bridge[1] > r.bridge[0]);
        if (kind === 'river') expect(decks.length, `${tag}: bridges`).toBeGreaterThan(0);
        for (const r of decks) expect(r.bridge![1] - r.bridge![0], `${tag}: ${r.name}`).toBeLessThanOrEqual(45);
        // No lot, and no road, in the water; the sim's water is the land's.
        const shaped = relaid.open!.land!;
        expect(shaped.edited).toBe(true);
        const W = relaid.width;
        const water = unrle(relaid.open!.water, W * relaid.depth);
        for (const s of relaid.slots) {
          const r = plotRect(t, s);
          for (let z = r.z; z < r.z + r.d; z++) for (let x = r.x; x < r.x + r.w; x++) expect(water[z * W + x], `${tag}: ${s.name} in the water`).toBe(0);
        }
        for (const r of relaid.roads) for (const [x, z] of r.points) if (x >= 0 && z >= 0 && x < W && z < relaid.depth) expect(water[Math.floor(z) * W + Math.floor(x)], `${tag}: ${r.name} in the water`).toBe(0);
        // (But under the decks, walked dry.)
        const deck = new Set<number>();
        for (const r of decks) deckTiles(r, r.bridge!, W, relaid.depth, (tile) => deck.add(tile));
        let wet = 0;
        for (let z = 0; z < relaid.depth; z += 4) for (let x = 0; x < W; x += 4) if (!deck.has(z * W + x) && wetTile(shaped, x, z) !== !!water[z * W + x]) wet++;
        expect(wet).toBe(0);
        // The lake is a water zone; the land is flat under the lots.
        expect(relaid.open!.zones.some((z) => z.kind === 'water')).toBe(true);
        for (const s of relaid.slots.slice(0, 20)) {
          const r = plotRect(t, s);
          const k = Math.round((r.z + r.d / 2) / LAND_CELL) * shaped.cols + Math.round((r.x + r.w / 2) / LAND_CELL);
          expect(shaped.h[k]).toBe(0);
        }
        // The player's land is not changed by it.
        expect(land.h.some((h, k) => h !== shaped.h[k])).toBe(true);
      },
      180_000,
    );
  }
});
