// A place's new world carries its land with its town graded into it (`game/landFollow.ts` `withLand`,
// `Land.seeded` 'follow'): from the town job, in its preview and in its town file, as the surface worked out
// from that file has it (its roads and pads on the land stored), unshaped (`edited` false) and kept by
// `applyLand` while unshaped; a town laid again on it is graded into it again, not flattened. Classic worlds
// and drafts from older code get none (they stay flat). `bun test tests/landFollow.slow.test.ts` by hand; the slow set is `bun run test:slow` in web/ (`SLOW_TESTS=1 tools/ci-local.sh web`; CI on v* tags and by hand).
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('a land the town follows', async () => {
  usePublicFetch();
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { assembleTown, generateWorld, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { previewWorld } = await import('../src/game/townPreview');
  const { runTownJob } = await import('../src/game/townJobs');
  const { withLand } = await import('../src/game/landFollow');
  const { applyLand, LandEditor } = await import('../src/game/landEdit');
  const { decodeLand, encodeLand, LAND_CELL } = await import('../src/game/land');
  const { generateFromSpec } = await import('../src/game/macro');
  const { CLASSIC } = await import('../src/game/world');
  const { builtSurfaceOf, ENTRY_MAX } = await import('../src/render/shared/builtSurface');
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const t = await templatesOf(content);

  test('a new place carries its land, graded under its town, to its town file', () => {
    const made = runTownJob(content, t, { kind: 'world', world: 'heath', seed: 4242, options: { town: 'village', villages: 'few', hills: 'hilly' } });
    if (made.kind !== 'world') throw new Error('not a world');
    const land = made.draft.open!.land!;
    expect(land.seeded).toBe('follow');
    expect(land.edited).toBe(false);
    expect(made.preview.meta!.land).toEqual(encodeLand(land));
    // (Not flat: the town is on its hills.)
    const pads = builtSurfaceOf(made.preview, land);
    const heights = made.preview.plots.map((_, i) => pads.padY(i));
    expect(Math.max(...heights) - Math.min(...heights)).toBeGreaterThan(5);
    // The town file carries it; the surface worked out from it stands on the land stored.
    const town = populate(content, assets, t, made.draft);
    const file = JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index));
    expect(file.meta.land).toEqual(encodeLand(land));
    const stored = decodeLand(file.meta.land);
    const surface = builtSurfaceOf(file, stored);
    let most = 0;
    file.plots.forEach((p: { x: number; z: number; w: number; d: number }, i: number) => {
      const [x, z] = [Math.ceil(p.x / LAND_CELL) * LAND_CELL, Math.ceil(p.z / LAND_CELL) * LAND_CELL];
      if (x > p.x + p.w || z > p.z + p.d) return;
      most = Math.max(most, Math.abs(surface.at(x, z) - stored.h[(z / LAND_CELL) * stored.cols + x / LAND_CELL] / 100));
      expect(surface.at(x, z)).toBeGreaterThanOrEqual(surface.padY(i) - 1e-6);
    });
    // (Worked out again at load, a pad may settle elsewhere in its window by its road, but not past it.)
    expect(most).toBeLessThanOrEqual(ENTRY_MAX);
    // Unshaped, step 1 keeps it with the world.
    const editor = new LandEditor(decodeLand(file.meta.land), () => 1);
    expect(applyLand(made.draft, editor).open!.land).toBe(land);
  }, 60_000);

  test('a town laid again on it is graded into it, not flattened', () => {
    const laid = generateWorld(content, t, 'heath', 4242, undefined, undefined, { town: 'village', villages: 'few', hills: 'hilly' });
    const first = withLand(laid, previewWorld(content, t, laid, null)).open!.land!;
    const shaped = { ...first, edited: true };
    const again = generateWorld(content, t, 'heath', 4242, undefined, undefined, { town: 'hamlet', villages: 'few', hills: 'hilly' }, shaped);
    const preview = previewWorld(content, t, again, null);
    const graded = withLand(again, preview).open!.land!;
    expect(graded.seeded).toBe('follow');
    expect(graded.edited).toBe(true);
    const surface = builtSurfaceOf(preview, graded);
    const heights = preview.plots.map((_, i) => surface.padY(i));
    expect(Math.max(...heights) - Math.min(...heights)).toBeGreaterThan(2);
  }, 60_000);

  test('classic worlds and drafts from older code get no land', () => {
    const classic = generateFromSpec(content, t, CLASSIC.small, 11);
    expect(classic.open?.heights).toBeUndefined();
    const preview = previewWorld(content, t, classic, null);
    expect(withLand(classic, preview)).toBe(classic);
    expect(preview.meta?.land).toBeUndefined();
    const town = populate(content, assets, t, classic);
    expect(JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index)).meta.land).toBeUndefined();
    // A place's draft from before heights (and before its land came first: none with it).
    const placed = generateWorld(content, t, 'farm', 7);
    const { heights: _heights, land: _land, plan: _plan, ...older } = placed.open!;
    const old = { ...placed, open: older };
    expect(withLand(old, previewWorld(content, t, old, null))).toBe(old);
  }, 60_000);
});
