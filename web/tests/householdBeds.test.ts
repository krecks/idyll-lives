// A generated house is furnished for who lives there (`housegen.ts` `fitFor`, `household.ts` `residentsOf`):
// a couple gets a bed for two, and a child a bed their life stage may sleep in, so a couple's
// seventeen-year-old gets no race car bed (a child's) and no toys only a child may use; a little one may.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('beds for who lives there', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
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
  const { generateHouse, objectInfos } = await import('../src/game/housegen');
  const { residentsOf } = await import('../src/game/household');
  type HouseGenConfig = import('../src/game/housegen').HouseGenConfig;
  type HouseholdDraft = import('../src/game/household').HouseholdDraft;
  type Residents = import('../src/game/housegen').Residents;

  const content = await Content.load('http://localhost/content/base.json');
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const cfg = json<HouseGenConfig>('housegen.json');
  const houses = json<{ plot: { width: number; depth: number }; lotSizes: Record<string, { width: number; depth: number }> }>('houses.json');
  const info = objectInfos(content);

  /** A couple and their child of `age`. */
  const family = (age: number): HouseholdDraft => {
    const member = (uid: string, years: number) => ({ uid, name: uid, gender: 'female', attractedTo: ['male'], appearance: {} as never, traits: [], perks: [], age: years });
    return {
      name: 'Nakamura',
      members: [member('jonah', 58), member('elif', 58), member('ravi', age)],
      bonds: [
        { a: 'jonah', b: 'elif', preset: 'partners' },
        { a: 'ravi', b: 'jonah', preset: 'parent' },
        { a: 'ravi', b: 'elif', preset: 'parent' },
      ],
    };
  };
  /** Every object of the houses made for `who` on medium lots (seeds 1..n) with a bedroom each for the couple and the child (as a town houses them). */
  const furnished = (who: Residents, n = 30) =>
    Array.from({ length: n }, (_, k) => generateHouse(cfg, info, houses.lotSizes.medium, houses.plot, 'medium', (k + 1) * 7919, content.lampTiles, content.windowTiles, who, 'roomy'))
      .filter((h): h is NonNullable<typeof h> => !!h && h.bedrooms >= 2)
      .map((h) => [...h.objects, ...(h.upper?.objects ?? []), ...(h.basement?.objects ?? [])].map((o) => o.def));
  const childBeds = new Set(cfg.residents?.beds.child ?? []);
  const mayUse = (def: string, stage: string) => {
    const d = content.object(def);
    // (Nothing to do with it, a table: anyone's.)
    return !!d && (!d.interactions.length || d.interactions.some((it) => content.mayUse(d, it, stage)));
  };

  test('a child is counted at their life stage', () => {
    expect(residentsOf(content, family(17))).toEqual({ couples: 1, adults: 0, children: 1, babies: 0, stages: ['teen'] });
    expect(residentsOf(content, family(6)).stages).toEqual(['child']);
    // Eighteen: a grown-up with a bed of their own.
    expect(residentsOf(content, family(18))).toMatchObject({ couples: 1, adults: 1, children: 0 });
  });

  test("a teenager's bed and things are a teenager's, and the couple has a bed for two", () => {
    const homes = furnished(residentsOf(content, family(17)));
    expect(homes.length).toBeGreaterThan(10);
    for (const defs of homes) {
      // A bed for two and one more (a child's bed, else the bedroom's own), each one a teenager may sleep in.
      const beds = defs.filter((d) => content.object(d)?.group === 'beds' || childBeds.has(d));
      expect(beds.length).toBeGreaterThanOrEqual(2);
      for (const d of beds) expect([d, mayUse(d, 'teen')]).toEqual([d, true]);
      expect(defs).not.toContain('kidsBedCar');
      for (const d of defs.filter((d) => content.object(d)?.category === 'kids')) expect([d, mayUse(d, 'teen')]).toEqual([d, true]);
      expect(defs.some((d) => (content.object(d)?.slots ?? 1) >= 2 && content.object(d)?.group === 'beds')).toBe(true);
    }
  });

  test("a little one's room may have the race car bed and toys", () => {
    const homes = furnished(residentsOf(content, family(6)), 60);
    expect(homes.some((defs) => defs.includes('kidsBedCar'))).toBe(true);
    expect(homes.some((defs) => defs.includes('toyBox') || defs.includes('rockingHorse'))).toBe(true);
  });
});
