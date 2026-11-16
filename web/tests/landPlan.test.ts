// A place's land before its town (step 1: `macro.ts` `planLand`, `landDraft`, `landFollow.ts` `seedBare`): the
// same place, options and seed make the same land, byte for byte; it has no town on it; the town and the villages
// chosen change none of it (its size, hills and woods do); its town's site stands on dry land, by the shore
// where the place has water on a side.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe("a place's land", async () => {
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { objectInfos } = await import('../src/game/housegen');
  const { useTownContent } = await import('../src/game/town');
  const { landDraft, planLand } = await import('../src/game/macro');
  const { LAND_CELL, landHash, wetAt } = await import('../src/game/land');
  const { LAND_SIZE, specOf, WORLD_KINDS } = await import('../src/game/world');
  type Templates = import('../src/game/town').Templates;
  type WorldOptions = import('../src/game/world').WorldOptions;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  test('the same place, options and seed make the same land', () => {
    const options: Partial<WorldOptions> = { size: 'small', hills: 'hilly' };
    const [a, b] = [landDraft(content, t, 'river', options, 4242), landDraft(content, t, 'river', options, 4242)];
    expect(landHash(a.open!.land)).toBe(landHash(b.open!.land));
    expect(JSON.stringify(a.open!.plan)).toBe(JSON.stringify(b.open!.plan));
    expect(JSON.stringify(a.open!.zones)).toBe(JSON.stringify(b.open!.zones));
    expect(a.name).toBe(b.name);
    // (Another seed, another land.)
    expect(landHash(landDraft(content, t, 'river', options, 4243).open!.land)).not.toBe(landHash(a.open!.land));
    // Its plan is the place's own (`planLand`).
    expect(JSON.stringify(planLand(specOf('river', options), 4242))).toBe(JSON.stringify(a.open!.plan));
  });

  test('no town on it', () => {
    const land = landDraft(content, t, 'farm', { size: 'medium' }, 77);
    expect(land.slots).toEqual([]);
    expect(land.roads).toEqual([]);
    expect(land.greens).toEqual([]);
    expect(land.households).toEqual([]);
    const open = land.open!;
    expect(open.areas).toEqual([]);
    expect(open.parking).toEqual([]);
    expect(open.backdrop ?? []).toEqual([]);
    expect(open.places ?? []).toEqual([]);
    // (No farmland or orchards: they come with a town's farms.)
    expect(open.zones.filter((z) => z.kind === 'fields' || z.kind === 'orchard')).toEqual([]);
    // Its land, as big as its size, the town's to follow (`seeded` 'follow'), not shaped by anyone.
    expect(land.width).toBe(LAND_SIZE.medium);
    expect(open.land!.cols * LAND_CELL).toBe(LAND_SIZE.medium);
    expect(open.land!.seeded).toBe('follow');
    expect(open.land!.edited).toBe(false);
    expect(open.plan!.size).toBe('medium');
  });

  test('the town and the villages change none of it', () => {
    for (const kind of ['country', 'lakeside', 'river'] as const) {
      const base = landDraft(content, t, kind, { size: 'large', town: 'hamlet', villages: 'none' }, 909);
      for (const other of [{ town: 'city', villages: 'many' }, { town: 'village', villages: 'few' }] as const) {
        const land = landDraft(content, t, kind, { size: 'large', ...other }, 909);
        expect(landHash(land.open!.land), `${kind} ${other.town}`).toBe(landHash(base.open!.land));
        expect(JSON.stringify(land.open!.plan)).toBe(JSON.stringify(base.open!.plan));
      }
      // (Its size, its hills and its woods do.)
      expect(landHash(landDraft(content, t, kind, { size: 'medium', town: 'hamlet', villages: 'none' }, 909).open!.land)).not.toBe(landHash(base.open!.land));
      expect(landHash(landDraft(content, t, kind, { size: 'large', town: 'hamlet', villages: 'none', hills: 'flat' }, 909).open!.land)).not.toBe(landHash(base.open!.land));
    }
  });

  test("its town's site is on dry land, by the shore where its water is on a side", () => {
    for (const kind of WORLD_KINDS) {
      for (const seed of [11, 4242]) {
        const land = landDraft(content, t, kind, {}, seed);
        const { plan, land: field } = land.open as Required<NonNullable<typeof land.open>>;
        const [sx, sz] = plan.site;
        const tag = `${kind} ${seed}`;
        // Dry for 40 m round it.
        for (let z = sz - 40; z <= sz + 40; z += LAND_CELL) {
          for (let x = sx - 40; x <= sx + 40; x += LAND_CELL) expect(wetAt(field, Math.round(x / LAND_CELL), Math.round(z / LAND_CELL)), `${tag}: wet at ${x}, ${z}`).toBe(false);
        }
        // By the water on its side (the sea, the lake, the river, the island's coast): within 200 m of the site.
        if (['seaside', 'lakeside', 'river', 'island'].includes(kind)) {
          const dir = plan.side === 'zmax' ? 1 : -1;
          let found = -1;
          for (let d = 0; d <= 200 && found < 0; d += LAND_CELL) if (wetAt(field, Math.round(sx / LAND_CELL), Math.round((sz + dir * d) / LAND_CELL))) found = d;
          expect(found, `${tag}: no water within 200 m on its ${plan.side} side`).toBeGreaterThan(40);
        }
        // Within the land, the level it is built at its 0.
        expect(sx).toBeGreaterThan(0);
        expect(sz).toBeLessThan(land.depth);
        expect(Math.abs(field.h[Math.round(sz / LAND_CELL) * field.cols + Math.round(sx / LAND_CELL)] / 100)).toBeLessThan(25);
      }
    }
  });
});
