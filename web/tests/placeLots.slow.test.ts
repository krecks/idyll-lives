// The named places' lots in generated towns (`game/placeLots.ts`): the generator draws no named place's layout for a
// town's own public places (`wantedLots`), and puts down a lot for its named places (`macro.ts` `namedLots`), each
// dressed and called as its place is. (The lots themselves: `placeLots.test.ts`.)
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('named places as lots, in generated towns', async () => {
  usePublicFetch();
  const { CIVIC } = await import('../src/game/places');
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { lotPlaceKind, placeOfTemplate } = await import('../src/game/placeLots');
  type PlotSlot = import('../src/game/town').PlotSlot;
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  test("a town's own public places are never a named place's layout, and its named places' lots are dressed and named as their places", () => {
    for (const seed of [1, 2, 3]) {
      const world = generateWorld(content, t, 'country', seed, undefined, undefined, { town: 'town', villages: 'few' });
      // Every lot built from a named place's layout is dressed as one (`placeUnit`), and no other is.
      // (A flats building is dressed as flats, no place.)
      expect(world.slots.filter((s) => s.kind !== 'flats' && !!placeOfTemplate(s.template) !== !!s.dress).map((s) => s.template)).toEqual([]);
      const named = world.slots.filter((s) => s.dress && s.kind === 'shop');
      // Every civic place of a town in the main town; a hamlet's in every other area.
      const civicOfSlot = (s: PlotSlot) => {
        const what = placeOfTemplate(s.template);
        return what && 'civic' in what ? what.civic : -1;
      };
      for (const c of CIVIC) expect(named.some((s) => !s.area && civicOfSlot(s) === CIVIC.indexOf(c)), c.label).toBe(true);
      for (let a = 1; a < world.open!.areas.length; a++) expect(named.filter((s) => s.area === a && 'civic' in (placeOfTemplate(s.template) ?? {})).length).toBeGreaterThan(0);
      for (const s of named) {
        const place = world.open!.places!.find((p) => p.plot === s.index)!;
        expect(place.name).toBe(s.name);
        expect(place.kind).toBe(lotPlaceKind(s));
      }
      expect(new Set(named.map((s) => s.name)).size).toBe(named.length);
    }
  }, 120_000);
});
