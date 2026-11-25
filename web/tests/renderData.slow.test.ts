// The landscape's data made in the render data worker (`render/shared/renderData.ts`,
// `renderData.worker.ts`) is the page's, to the byte: the world goes over as the worker gets it
// (`landWorld`, through a structured clone), the land is laid out there again, and the grass field
// (its town mask made on the page, `withTownGrass`) and the terrain's heights and colours come back
// as `grassField` and `shadeTerrain` make them on the page, after the page's layout has grown its
// woods and lawns (as `pc/nature.ts` does before the grass).
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('render data worker', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  usePublicFetch();
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { previewWorld } = await import('../src/game/townPreview');
  const { CountryRoads } = await import('../src/render/shared/countryRoads');
  const { grassField } = await import('../src/render/shared/grass');
  const { LandscapeLayout } = await import('../src/render/shared/nature');
  const { Land, landWorld, withTownGrass } = await import('../src/render/shared/renderData');
  type WorldStructure = import('../src/core/protocol').WorldStructure;
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  /** The page's layout as `pc/nature.ts` has it when it asks for the grass: its woods and lawns grown. */
  async function pageLand(world: WorldStructure) {
    const land = new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name));
    const grown = land as unknown as { woods(): Promise<unknown>; lawns(world: WorldStructure): Promise<unknown>; shadeTerrain(p: Float32Array, c: Float32Array): Promise<void>; ring: number };
    await grown.woods();
    await grown.lawns(world);
    return grown;
  }

  /** A terrain tile's grid (as `pc/nature.ts` lays one out) at (`x0`, `z0`). */
  function grid(x0: number, z0: number, size = 128, n = 33): Float32Array {
    const positions = new Float32Array(n * n * 3);
    for (let v = 0; v < n * n; v++) {
      positions[v * 3] = x0 + ((v % n) * size) / (n - 1);
      positions[v * 3 + 2] = z0 + (Math.floor(v / n) * size) / (n - 1);
    }
    return positions;
  }

  const worlds = [
    { kind: 'farm', seed: 5 },
    { kind: 'mountain', seed: 77 },
    { kind: 'seaside', seed: 2024 },
    { kind: 'lakeside', seed: 31 },
  ] as const;

  for (const w of worlds) {
    test(
      `${w.kind} ${w.seed}: the worker's grass field and terrain are the page's`,
      async () => {
        const world = previewWorld(content, t, generateWorld(content, t, w.kind, w.seed), null);
        const page = await pageLand(world);
        // In the worker: the world as it is sent, the answers as they come back.
        const sent = structuredClone(landWorld(world));
        const worker = new Land(sent);
        const field = withTownGrass(structuredClone(await worker.grass(sent, page.ring)), world);
        const direct = await grassField(world, page as unknown as InstanceType<typeof LandscapeLayout>, undefined, page.ring);
        expect(field.colour.some((b) => b > 0)).toBe(true);
        expect({ ...field, colour: null, ground: null, town: null, patches: null }).toEqual({ ...direct, colour: null, ground: null, town: null, patches: null });
        expect(Buffer.from(field.colour).equals(Buffer.from(direct.colour))).toBe(true);
        expect(Buffer.from(field.ground.buffer).equals(Buffer.from(direct.ground.buffer))).toBe(true);
        expect(Buffer.from(field.town).equals(Buffer.from(direct.town))).toBe(true);
        expect(Buffer.from(field.patches).equals(Buffer.from(direct.patches))).toBe(true);

        for (const [x0, z0] of [
          [-150, -150],
          [world.width / 2 - 64, world.depth / 2 - 64],
          [world.width + 20, -40],
        ]) {
          const positions = grid(x0, z0);
          const colors = new Float32Array((positions.length / 3) * 4);
          await page.shadeTerrain(positions, colors);
          const shaded = structuredClone(grid(x0, z0));
          const shadedColors = structuredClone(await worker.shade(shaded));
          expect(Buffer.from(shaded.buffer).equals(Buffer.from(positions.buffer))).toBe(true);
          expect(Buffer.from(shadedColors.buffer).equals(Buffer.from(colors.buffer))).toBe(true);
        }
      },
      60_000,
    );
  }

  test("a world with a relief grid: the worker's terrain is the page's", async () => {
    // (A hand-made one, among mountains: the generator's grid goes over in the world's meta.)
    const { reliefWorld } = await import('./support/landFixtures');
    const world = reliefWorld(140, { slope: 0.75, mountains: 140 }, 'mountain');
    const page = await pageLand(world);
    const sent = structuredClone(landWorld(world));
    const worker = new Land(sent);
    for (const [x0, z0] of [
      [-200, -200],
      [200, 380],
      [world.width + 20, world.depth - 60],
    ]) {
      const positions = grid(x0, z0);
      const colors = new Float32Array((positions.length / 3) * 4);
      await page.shadeTerrain(positions, colors);
      const shaded = structuredClone(grid(x0, z0));
      const shadedColors = structuredClone(await worker.shade(shaded));
      expect(Buffer.from(shaded.buffer).equals(Buffer.from(positions.buffer))).toBe(true);
      expect(Buffer.from(shadedColors.buffer).equals(Buffer.from(colors.buffer))).toBe(true);
      // (And the grid's land is there: well above the classic valley's.)
      if (z0 === 380) expect(Math.max(...positions.filter((_, i) => i % 3 === 1))).toBeGreaterThan(40);
    }
  });
});
