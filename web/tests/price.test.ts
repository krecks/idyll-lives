// A home's price (`game/price.ts`), from the shipped content and house layouts: the land grows with the lot,
// the house with its floor (every storey and the basement), the furniture with what's in it; an empty lot costs
// the land alone, and a public place isn't for sale.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('home prices', async () => {
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
  const { objectInfos } = await import('../src/game/housegen');
  const { EMPTY_LOT, generatedId } = await import('../src/game/town');
  const { homePrice, priceOf, priceParts } = await import('../src/game/price');
  type Templates = import('../src/game/town').Templates;
  type PlotSlot = import('../src/game/town').PlotSlot;
  type HouseTemplate = import('../src/game/town').HouseTemplate;

  const content = await Content.load('http://localhost/content/base.json');
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  // As `loadTemplates` puts them together (it fetches by the page's base address, which bun hasn't).
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const rates = content.economy.homePrice!;
  const slot = (template: string, w?: number, d?: number, kind: PlotSlot['kind'] = 'house'): PlotSlot => ({ index: 0, kind, template, name: 'Plot', x: 0, z: 0, facing: 0, lot: 'medium', w, d });
  const thingPrice = (def: string) => content.object(def)?.price ?? 0;
  /** A one-room house, `w` × `d` tiles, with `things` in it. */
  const box = (w: number, d: number, things: string[] = []): HouseTemplate => ({
    id: 'box',
    name: 'Box',
    description: '',
    bedrooms: 1,
    walls: [
      [2, 2, 2 + w, 2],
      [2 + w, 2, 2 + w, 2 + d],
      [2 + w, 2 + d, 2, 2 + d],
      [2, 2 + d, 2, 2],
    ],
    doors: [],
    objects: things.map((def, i) => ({ def, x: 3 + i, z: 3, rot: 0 })),
    spawns: [],
  });
  const priced = content.objectList.find((o) => (o.price ?? 0) > 0)!;
  /** What `floor` tiles of house cost (a bigger house more a tile: `sizePremium`). */
  const built = (floor: number) => Math.round(rates.perFloorTile * floor * (floor / (rates.sizeRef ?? 100)) ** (rates.sizePremium ?? 0));

  test('the rates are in the content', () => {
    expect(rates.perTile).toBeGreaterThan(0);
    expect(rates.perFloorTile).toBeGreaterThan(0);
    expect(rates.round).toBeGreaterThan(0);
  });

  test('a bigger lot costs more, for the same house', () => {
    const [small, big] = [priceParts(content, t, slot('maple', 20, 20))!, priceParts(content, t, slot('maple', 30, 30))!];
    expect(big.land).toBeGreaterThan(small.land);
    expect(big.land).toBe(rates.perTile * 30 * 30);
    expect([big.house, big.furniture]).toEqual([small.house, small.furniture]);
    expect(big.total).toBeGreaterThan(small.total);
    expect(big.total % rates.round).toBe(0);
  });

  test('a bigger house costs more, on the same lot', () => {
    const [small, big] = [priceOf(thingPrice, rates, box(6, 6), 30, 30), priceOf(thingPrice, rates, box(10, 8), 30, 30)];
    expect(small.house).toBe(built(36));
    expect(big.house).toBe(built(80));
    expect(big.total).toBeGreaterThan(small.total);
    // More a tile, too.
    expect(big.house / 80).toBeGreaterThan(small.house / 36);
    // Every storey counts, the basement too; planned rooms count as their floor.
    const tall: HouseTemplate = { ...box(6, 6), upper: { walls: box(6, 6).walls, doors: [], objects: [] }, basement: { walls: [], doors: [], objects: [], rooms: [{ kind: 'storage', x: 2, z: 2, w: 3, d: 4 }] } };
    expect(priceOf(thingPrice, rates, tall, 30, 30).house).toBe(built(36 + 36 + 12));
    // The fixed layouts: a bigger one costs more.
    const [cottage, house] = [priceParts(content, t, slot('maple', 30, 26))!, priceParts(content, t, slot('oakridge', 30, 26))!];
    expect(house.house).toBeGreaterThan(cottage.house);
  });

  test('more furniture costs more', () => {
    const [bare, furnished] = [priceOf(thingPrice, rates, box(6, 6), 30, 30), priceOf(thingPrice, rates, box(6, 6, [priced.id, priced.id]), 30, 30)];
    expect(bare.furniture).toBe(0);
    expect(furnished.furniture).toBe(Math.round(rates.furniture * 2 * priced.price!));
    expect(furnished.total).toBeGreaterThanOrEqual(bare.total);
    // Unknown things are worth nothing.
    expect(priceOf(thingPrice, rates, box(6, 6, ['no.such.thing']), 30, 30).furniture).toBe(0);
  });

  test('a generated house is priced with its storeys and what it is furnished with', () => {
    const id = generatedId('large', 4242, 'roomy');
    const parts = priceParts(content, t, slot(id, 32, 28))!;
    expect(parts.house).toBeGreaterThan(0);
    expect(parts.furniture).toBeGreaterThan(0);
    expect(parts.total).toBe(Math.round((parts.land + parts.house + parts.furniture) / rates.round) * rates.round);
  });

  test('an empty lot costs the land alone', () => {
    const land = Math.round((rates.perTile * 24 * 22) / rates.round) * rates.round;
    expect(priceParts(content, t, slot('maple', 24, 22), true)).toEqual({ land: rates.perTile * 24 * 22, house: 0, furniture: 0, total: land });
    expect(homePrice(content, t, slot(EMPTY_LOT, 24, 22))).toBe(land);
    expect(homePrice(content, t, slot('maple', 24, 22))).toBeGreaterThan(land);
  });

  test('a public place has no price', () => {
    expect(priceParts(content, t, slot('park', 30, 26, 'park'))).toBeNull();
    expect(priceParts(content, t, slot('shop', 30, 26, 'shop'))).toBeNull();
    expect(homePrice(content, t, slot('park', 30, 26, 'park'))).toBe(0);
  });
});
