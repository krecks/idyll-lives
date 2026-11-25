// Choose Home's homes (`ui/homes.ts`): a home is a house's lot or a flat of a building of flats or of the flats over a
// place; the homes for sale are both, a lived-in flat leaving its building's others for sale; each flat is named as
// the game names it and knows its storey; the list keeps a building's flats together, and a click on a building
// chooses the flat chosen already, else the cheapest the household can afford.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe("Choose Home's homes", async () => {
  usePublicFetch();
  const { AssetRegistry } = await import('../src/assets/registry');
  const { flatsIdOf, generateWorld, useTownContent } = await import('../src/game/town');
  const { flatName, flatsTemplate } = await import('../src/game/flats');
  const { flatPrice } = await import('../src/game/price');
  const { peopleStreams, populate } = await import('../src/game/people');
  const { homeKey, homeOffers, homeOnLot, homeRows, homesForSale, livedIn, sameHome, storeyName } = await import('../src/ui/homes');
  type PlotSlot = import('../src/game/town').PlotSlot;
  type HomeOffer = import('../src/ui/homes').HomeOffer;
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const t = await templatesOf(content);
  // (A hamlet whose centre has flats over its shops and a building of flats.)
  const hamlet = generateWorld(content, t, 'country', 4242, undefined, undefined, { town: 'hamlet', villages: 'none' });
  const town = populate(content, assets, t, hamlet, undefined, peopleStreams(2026));

  test('the homes are the houses and every flat, each flat named as the game names it, on its storey, at its price', () => {
    const all = homeOffers(content, t, hamlet);
    const flats = all.filter((h) => h.flat !== undefined);
    expect(flats.length).toBeGreaterThan(0);
    expect(all.filter((h) => h.flat === undefined).every((h) => h.lot.kind === 'house' && h.name === h.lot.name)).toBe(true);
    expect(new Set(all.map((h) => h.key)).size).toBe(all.length);
    for (const f of flats) {
      const b = flatsTemplate(t, flatsIdOf(f.lot))!;
      expect(f.lot.kind === 'flats' || f.lot.kind === 'shop').toBe(true);
      expect(f.name).toBe(flatName(f.lot.name, f.flat!));
      expect(f.name).toBe(`Flat ${f.flat! + 1}, ${f.lot.name}`);
      expect(f.storey).toBe(b.units[f.flat!].storey);
      expect(f.storey).toBeGreaterThanOrEqual(0);
      expect(f.bedrooms).toBe(b.units[f.flat!].bedrooms);
      expect(f.price).toBe(flatPrice(content, t, { template: flatsIdOf(f.lot) }, f.flat!));
      expect(f.key).toBe(homeKey(f));
    }
    // (The flats over a place are upstairs, and every flat has a price.)
    expect(flats.filter((f) => f.lot.kind === 'shop').every((f) => (f.storey ?? 0) >= 1)).toBe(true);
    expect(flats.every((f) => f.price > 0)).toBe(true);
    expect(storeyName(0)).toBe('Ground floor');
    expect(storeyName(2)).toBe('Floor 2');
  });

  test("a flat lived in is not for sale, its building's other flats are; a house lived in is not", () => {
    const sale = homesForSale(content, t, town);
    const keys = new Set(sale.map((h) => h.key));
    for (const h of town.households) expect(keys.has(homeKey(h))).toBe(false);
    expect(sale.every((h) => !livedIn(town, h))).toBe(true);
    expect(sale.length + town.households.length).toBe(homeOffers(content, t, town).length);
    // A household in one flat of a building: that flat is not for sale, the building's others are.
    const flat = homeOffers(content, t, hamlet).find((h) => h.flat !== undefined && flatsTemplate(t, flatsIdOf(h.lot))!.units.length > 1)!;
    const one = { ...hamlet, households: [{ household: town.households[0].household, slot: flat.slot, flat: flat.flat }] };
    const left = homesForSale(content, t, one);
    expect(left.some((h) => h.key === flat.key)).toBe(false);
    expect(left.filter((h) => h.slot === flat.slot).length).toBe(flatsTemplate(t, flatsIdOf(flat.lot))!.units.length - 1);
    expect(sameHome(flat, { slot: flat.slot })).toBe(false);
  });

  // ---- The list and a click, on homes of their own ----

  const lot = (index: number, area = 0): PlotSlot => ({ index, kind: index === 3 ? 'flats' : 'house', template: 'maple', lot: 'small', name: `Lot ${index}`, x: index * 20, z: 0, facing: 0, area });
  const offer = (slot: number, price: number, flat?: number, storey?: number, area = 0): HomeOffer => ({
    slot,
    ...(flat === undefined ? {} : { flat, storey }),
    key: homeKey({ slot, flat }),
    lot: lot(slot, area),
    name: flat === undefined ? `Lot ${slot}` : `Flat ${flat + 1}, Lot ${slot}`,
    bedrooms: 1,
    price,
  });
  const homes = [offer(1, 90_000), offer(2, 40_000), offer(3, 30_000, 2, 1), offer(3, 25_000, 0, 0), offer(3, 50_000, 1, 1), offer(4, 10_000, undefined, undefined, 1), offer(5, 70_000)];

  test("the list: their own home first, the main town's before a village's, cheapest first, a building's flats together by storey", () => {
    const rows = homeRows(homes, null).map((h) => h.key);
    expect(rows).toEqual(['3:0', '3:1', '3:2', '2', '5', '1', '4']);
    // Their own flat first; the building's other flats where its cheapest of them comes.
    expect(homeRows(homes, { slot: 3, flat: 1 }).map((h) => h.key)).toEqual(['3:1', '3:0', '3:2', '2', '5', '1', '4']);
    expect(homeRows(homes, { slot: 3, flat: 0 }).map((h) => h.key)).toEqual(['3:0', '3:1', '3:2', '2', '5', '1', '4']);
    expect(homeRows(homes, { slot: 1 }).map((h) => h.key)[0]).toBe('1');
  });

  test('a click on a building chooses its flat chosen already, else theirs, else the cheapest they can afford, else the cheapest', () => {
    const dearOver = (most: number) => (h: HomeOffer) => h.price > most;
    expect(homeOnLot(homes, 3, null, null, dearOver(1e9))?.key).toBe('3:0');
    expect(homeOnLot(homes, 3, { slot: 3, flat: 1 }, null, dearOver(1e9))?.key).toBe('3:1');
    expect(homeOnLot(homes, 3, { slot: 2 }, { slot: 3, flat: 2 }, dearOver(1e9))?.key).toBe('3:2');
    // (Only the dearest within reach.)
    expect(homeOnLot(homes.filter((h) => h.key !== '3:0'), 3, null, null, dearOver(40_000))?.key).toBe('3:2');
    expect(homeOnLot(homes, 3, null, null, dearOver(1))?.key).toBe('3:0');
    expect(homeOnLot(homes, 2, null, null, dearOver(1e9))?.key).toBe('2');
    expect(homeOnLot(homes, 9, null, null, dearOver(1e9))).toBeNull();
  });
});
