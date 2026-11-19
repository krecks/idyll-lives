// Flats (`game/flats.ts`) from the shipped content: buildings of flats round a stair hall, the flats
// over a shop and the real storeys above an office, each laid out the same whoever lives there, every
// flat behind its own door off the hall, a shell nobody has moved into, and flats the cheapest homes.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('flats', async () => {
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
  const { flatName, flatsFor, flatsId, flatsTemplate, floorsFor, floorsId, overId, placeFloors, placeOfId, storeysOfId } = await import('../src/game/flats');
  const { FLAT_FLOOR, flatPriceParts, priceOf } = await import('../src/game/price');
  const { mulberry32 } = await import('../src/game/noise');
  type Templates = import('../src/game/town').Templates;
  type Residents = import('../src/game/housegen').Residents;
  type Rect = import('../src/game/town').Rect;
  type Placement = import('../src/game/town').Placement;

  const content = await Content.load('http://localhost/content/base.json');
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  // As `loadTemplates` puts them together (it fetches by the page's base address, which bun hasn't).
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const cfg = t.gen!;
  const len = content.object(cfg.stairs)!.footprint![1];
  const inside = (r: Rect, x: number, z: number) => x >= r.x && z >= r.z && x < r.x + r.w && z < r.z + r.d;
  /** The tiles a thing stands on. */
  const tiles = (o: Placement): [number, number][] => {
    const [fw, fd] = content.object(o.def)?.footprint ?? [1, 1];
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    return Array.from({ length: w * d }, (_, i) => [o.x + (i % w), o.z + Math.floor(i / w)]);
  };
  const isBed = (def: string) => content.object(def)?.group === 'beds';
  const couple: Residents = { couples: 1, adults: 0, children: 0, babies: 0 };
  const family: Residents = { couples: 1, adults: 0, children: 2, babies: 0, stages: ['child', 'child'] };
  /** A flat's layout alone (walls, doors, windows, rooms), to compare flats furnished for different households. */
  const layoutOf = (l: { walls: unknown; doors: unknown; windows: unknown; rooms: unknown }) => JSON.stringify([l.walls, l.doors, l.windows, l.rooms]);

  test('ids say what they are', () => {
    expect(flatsId(42, 3, true, true)).toBe('flats:42:3:1:p');
    expect(flatsId(42, 2, false, false)).toBe('flats:42:2:0');
    expect(storeysOfId('flats:42:3:1')).toBe(3);
    expect(storeysOfId(overId('shopSmall', 7, 2))).toBe(2);
    expect(placeOfId(overId('cafeSmall', 7, 2))).toBe('cafeSmall');
    expect(placeOfId(floorsId('bank', 7, 3, 'office'))).toBe('bank');
    expect(placeOfId('gen:small:1')).toBeNull();
    expect(storeysOfId('gen:small:1')).toBe(1);
    expect(flatName('Mill Court', 2)).toBe('Flat 3, Mill Court');
  });

  test('a lot for a building of flats, as a slot takes it', () => {
    const rand = mulberry32(5);
    for (const zone of ['centre', 'town'] as const) {
      for (let i = 0; i < 10; i++) {
        const s = flatsFor(t, zone, rand)!;
        expect(s.template).toMatch(/^flats:\d+:[23]:[01](:p)?$/);
        const b = flatsTemplate(t, s.template)!;
        expect([s.w, s.d]).toEqual([b.lot.width, b.lot.depth]);
        expect(s.dress).toEqual(b.look);
        expect(s.dress!.use).toBe('flats');
        expect(s.dress!.storeys).toBe(storeysOfId(s.template));
        // A small house's lot or so: room for its flats, a front strip and a yard.
        expect(s.w).toBeGreaterThanOrEqual(11);
        expect(s.w).toBeLessThanOrEqual(23);
        expect(s.d).toBeGreaterThanOrEqual(16);
        expect(s.d).toBeLessThanOrEqual(23);
      }
    }
  });

  test('the same id gives the same building', () => {
    const a = JSON.stringify(flatsTemplate(t, 'flats:1234:3:1:p'));
    // (Another furnishing, then the first again: generated, not only kept.)
    flatsTemplate(t, 'flats:1234:3:1:p', [couple]);
    expect(JSON.stringify(flatsTemplate(t, 'flats:1234:3:1:p'))).toBe(a);
  });

  for (const id of ['flats:104729:3:1', 'flats:209458:2:1:p', 'flats:314187:3:0', 'flats:418916:2:0:p']) {
    test(`${id}: its flats round the stair hall`, () => {
      const b = flatsTemplate(t, id)!;
      const [, , storeys, wide, bays] = id.split(':');
      expect(b.storeys).toBe(Number(storeys));
      expect(b.shell.length).toBe(b.storeys);
      expect(b.units.length).toBe(b.storeys * (wide === '1' ? 2 : 1));
      // The lot: the building, a metre each side, a front strip (deep enough for bays with them) and a yard.
      expect(b.lot.width).toBe(b.box.w + 2 * cfg.flats!.side);
      expect(b.box.z).toBe(bays ? cfg.flats!.frontBays : cfg.flats!.front);
      expect(b.lot.depth - b.box.z - b.box.d).toBeGreaterThanOrEqual(cfg.flats!.yard[0]);
      // The hall: its street door first on the ground, stairs on every storey but the top.
      expect(b.front).toEqual(b.shell[0].doors[0]);
      expect(b.front.z).toBe(b.box.z);
      expect(inside(b.hall, b.front.x, b.front.z)).toBe(true);
      b.shell.forEach((s, k) => expect(s.objects.filter((o) => o.def === cfg.stairs).length).toBe(k + 1 < b.storeys ? 1 : 0));
      for (const [i, u] of b.units.entries()) {
        expect(u.storey).toBe(Math.floor(i / (wide === '1' ? 2 : 1)));
        // Beside the hall, the whole depth of the building.
        expect(u.rect.z).toBe(b.hall.z);
        expect(u.rect.d).toBe(b.hall.d);
        expect(u.rect.x + u.rect.w === b.hall.x || u.rect.x === b.hall.x + b.hall.w).toBe(true);
        expect(u.rect.w).toBeGreaterThanOrEqual(cfg.classes.flat.width[0]);
        // Its door off the hall's floor past the flights; one at home behind it.
        expect(inside(b.hall, ...u.door)).toBe(true);
        expect(u.door[1]).toBeGreaterThanOrEqual(b.hall.z + len + 1);
        expect(inside(u.rect, Math.floor(u.inside[0]), Math.floor(u.inside[1]))).toBe(true);
        expect(Math.abs(Math.floor(u.inside[0]) - u.door[0]) + Math.abs(Math.floor(u.inside[1]) - u.door[1])).toBe(1);
        // Everything in it inside it; a toilet in a bathroom, a sink and a stove in the kitchen.
        for (const o of u.layout.objects) for (const [x, z] of tiles(o)) expect(inside(u.rect, x, z)).toBe(true);
        const roomOf = (def: string) => u.layout.rooms.find((r) => u.layout.objects.some((o) => o.def === def && inside(r, o.x, o.z)))?.kind;
        expect(roomOf(u.layout.objects.find((o) => o.def.startsWith('toilet'))!.def)).toBe('bathroom');
        expect(u.layout.rooms.some((r) => r.kind === 'kitchen')).toBe(true);
        // No window to the hall: the wall the flat shares with it.
        const hallSide = u.rect.x === b.hall.x + b.hall.w ? u.rect.x : u.rect.x + u.rect.w;
        expect(u.layout.windows.some((w) => w.axis === 'z' && w.x === hallSide)).toBe(false);
      }
      // The same plan on every storey (furnished for nobody, the same things too).
      const sides = wide === '1' ? 2 : 1;
      for (let i = sides; i < b.units.length; i++) expect(JSON.stringify(b.units[i].layout)).toBe(JSON.stringify(b.units[i - sides].layout));
      // A letterbox each, kept by its flat, by the street door; the yard's things behind.
      const boxes = b.outside.filter((o) => o.unit !== undefined);
      expect(boxes.map((o) => o.unit).sort()).toEqual(b.units.map((_, i) => i));
      for (const o of boxes) {
        expect(o.def).toBe(cfg.flats!.letterbox);
        expect(o.z).toBe(b.box.z - 1);
        expect(o.x).not.toBe(b.front.x);
      }
      for (const o of b.outside.filter((o) => o.unit === undefined)) expect(o.z).toBeGreaterThanOrEqual(b.box.z + b.box.d);
      // Bays for at most half its flats, in the front strip, clear of the path and the letterboxes.
      expect(b.bays.length).toBeLessThanOrEqual(Math.floor(b.units.length / 2));
      if (!bays) expect(b.bays.length).toBe(0);
      for (const { rect, unit } of b.bays) {
        expect(unit).toBeLessThan(b.units.length);
        expect(rect.z + rect.d).toBeLessThanOrEqual(b.box.z);
        expect(rect.x).toBeGreaterThanOrEqual(0);
        expect(rect.x + rect.w).toBeLessThanOrEqual(b.lot.width);
        expect(inside(rect, b.front.x, b.box.z - 1)).toBe(false);
        for (const o of boxes) expect(inside(rect, o.x, o.z)).toBe(false);
      }
      expect(b.paths).toEqual([{ x: b.front.x, z: 0, w: 1, d: b.box.z }]);
    });
  }

  test('a flat is laid out the same whoever lives there, and furnished for them', () => {
    const id = 'flats:523645:2:1';
    const bare = flatsTemplate(t, id)!;
    const lived = flatsTemplate(t, id, [family, couple, 'shell', undefined])!;
    lived.units.forEach((u, i) => expect(layoutOf(u.layout)).toBe(layoutOf(bare.units[i].layout)));
    // A family's: beds for all four (two to a room if short of bedrooms), a bed for two among them.
    const beds = lived.units[0].layout.objects.filter((o) => isBed(o.def));
    const places = beds.reduce((n, o) => n + (content.object(o.def)?.slots ?? 1), 0);
    expect(places).toBeGreaterThanOrEqual(Math.min(4, 2 * lived.units[0].bedrooms));
    expect(beds.some((o) => (content.object(o.def)?.slots ?? 1) >= 2)).toBe(true);
    // A shell: its fittings and lights, nothing loose.
    const shell = lived.units[2].layout.objects;
    expect(shell.some((o) => isBed(o.def))).toBe(false);
    expect(shell.some((o) => o.def.startsWith('toilet'))).toBe(true);
    expect(shell.some((o) => (content.object(o.def)?.stations ?? []).includes('stove'))).toBe(true);
    const loose = shell.filter((o) => ['sofa', 'tv', 'tableDining4', 'fridge'].includes(o.def) || content.object(o.def)?.layer === 'surface');
    expect(loose).toEqual([]);
    expect(shell.length).toBeLessThan(bare.units[2].layout.objects.length);
  });

  test('a house nobody has moved into may be a shell', () => {
    let seen = 0;
    for (let seed = 1; seed <= 12; seed++) {
      const lot = t.lotSizes!.medium!;
      const empty = generateHouse(cfg, t.info!, lot, t.plot, 'medium', seed * 7919, t.lampTiles, t.windowTiles, undefined, 'roomy');
      const shell = generateHouse(cfg, t.info!, lot, t.plot, 'medium', seed * 7919, t.lampTiles, t.windowTiles, 'shell', 'roomy');
      if (!empty || !shell) continue;
      seen++;
      const plan = (h: typeof empty) => JSON.stringify([h.walls, h.doors, h.windows, h.rooms, h.upper?.walls, h.upper?.doors, h.basement?.walls, h.lot]);
      expect(plan(shell)).toBe(plan(empty));
      const all = [...shell.objects, ...(shell.upper?.objects ?? []), ...(shell.basement?.objects ?? [])];
      expect(all.some((o) => isBed(o.def))).toBe(false);
      expect(all.some((o) => o.def.startsWith('toilet'))).toBe(true);
      // Nothing in the garden: everything indoors (within the walls' box).
      const xs = shell.walls.flatMap(([x0, , x1]) => [x0, x1]);
      const zs = shell.walls.flatMap(([, z0, , z1]) => [z0, z1]);
      for (const o of shell.objects) {
        expect(o.x).toBeGreaterThanOrEqual(Math.min(...xs));
        expect(o.z).toBeGreaterThanOrEqual(Math.min(...zs));
        expect(o.x).toBeLessThan(Math.max(...xs));
        expect(o.z).toBeLessThan(Math.max(...zs));
      }
    }
    expect(seen).toBeGreaterThan(8);
  });

  test('flats over a shop: a stair hall of their own cut from its ground floor', () => {
    // (Not the café with its tables out front: its letterboxes would have nowhere clear of the porch over its door.)
    for (const place of ['shopSmall', 'cafeTiny', 'boutique']) {
      for (const storeys of [2, 3]) {
        const b = flatsTemplate(t, overId(place, 77 + storeys, storeys))!;
        expect(b.place).toBe(place);
        const p = t.parks.find((q) => q.id === place)!;
        const ground = b.ground!;
        // The place keeps its front door; nothing of it stands in the hall.
        expect(ground.doors![0]).toEqual(p.doors![0]);
        expect(inside(b.hall, b.front.x, b.front.z)).toBe(true);
        expect(b.front.x).not.toBe(p.doors![0].x);
        for (const o of ground.objects) for (const [x, z] of tiles(o)) expect(inside(b.hall, x, z)).toBe(false);
        // A wall between the shop and the hall, the whole depth.
        const side = b.hall.x === b.box.x ? b.hall.x + b.hall.w : b.hall.x;
        expect(ground.walls!.some(([x0, z0, x1, z1]) => x0 === side && x1 === side && Math.min(z0, z1) <= b.box.z && Math.max(z0, z1) >= b.box.z + b.box.d)).toBe(true);
        // A flat on every storey above, none on the ground.
        expect(b.units.map((u) => u.storey)).toEqual(Array.from({ length: storeys - 1 }, (_, k) => k + 1));
        for (const u of b.units) {
          expect(inside(b.hall, ...u.door)).toBe(true);
          expect(inside(b.box, u.rect.x, u.rect.z)).toBe(true);
          expect(inside(b.box, u.rect.x + u.rect.w - 1, u.rect.z + u.rect.d - 1)).toBe(true);
        }
        expect(b.outside.filter((o) => o.unit !== undefined).length).toBe(b.units.length);
      }
    }
  });

  test('real storeys above an office, up stairs along a side wall', () => {
    const id = floorsFor(t, 'officeSuite', 'office', 3, mulberry32(3))!;
    expect(id).toMatch(/^floors:officeSuite:\d+:3:office$/);
    const f = placeFloors(t, id)!;
    expect(f.storeys).toBe(3);
    expect(f.floors.length).toBe(2);
    // The ground's flight, the second on storey 1, none on the top storey.
    expect(f.ground.objects.filter((o) => o.def === cfg.stairs).length).toBe(1);
    expect(f.floors[0].objects.filter((o) => o.def === cfg.stairs).length).toBe(1);
    expect(f.floors[1].objects.filter((o) => o.def === cfg.stairs).length).toBe(0);
    for (const floor of f.floors) {
      const kinds = floor.rooms.map((r) => r.kind);
      expect(kinds).toContain('hall');
      expect(kinds).toContain('office');
      // Desks and chairs at them; a toilet somewhere up there in all.
      expect(floor.objects.some((o) => o.def === 'deskSimple')).toBe(true);
    }
    expect(f.floors.some((floor) => floor.objects.some((o) => o.def.startsWith('toilet')))).toBe(true);
    // A place with a storey of its own above (the hotel) gets none.
    expect(placeFloors(t, floorsId('hotel', 1, 2, 'office'))).toBeUndefined();
  });

  test('flats are the cheapest homes: their share of the land, their floor at a flat\'s rate', () => {
    const rates = content.economy.homePrice!;
    const price = (def: string) => content.object(def)?.price ?? 0;
    const flats: number[] = [];
    for (let seed = 1; seed <= 6; seed++) {
      const id = flatsId(seed * 1009, 2 + (seed % 2), seed % 3 !== 0, false);
      const b = flatsTemplate(t, id)!;
      for (const i of b.units.keys()) {
        const parts = flatPriceParts(content, t, { template: id }, i)!;
        expect(parts.land).toBe(Math.round((rates.perTile * b.lot.width * b.lot.depth) / b.units.length));
        const floor = b.units[i].layout.rooms.reduce((n, r) => n + r.w * r.d, 0);
        expect(parts.house).toBe(Math.round(FLAT_FLOOR * rates.perFloorTile * floor * (floor / (rates.sizeRef ?? 100)) ** (rates.sizePremium ?? 0)));
        flats.push(parts.total);
      }
    }
    expect(flatPriceParts(content, t, { template: 'flats:1009:3:1' }, 99)).toBeNull();
    // Cottages on snug lots, the cheapest houses.
    const cottages: number[] = [];
    for (let seed = 1; cottages.length < 12 && seed < 200; seed++) {
      const h = generateHouse(cfg, t.info!, t.lotSizes!.small!, t.plot, 'small', seed * 31, t.lampTiles, t.windowTiles, undefined, 'snug');
      if (h?.class === 'cottage') cottages.push(priceOf(price, rates, h, h.lot!.width, h.lot!.depth).total);
    }
    const mean = (v: number[]) => v.reduce((a, b) => a + b, 0) / v.length;
    expect(mean(flats)).toBeLessThan(mean(cottages));
  });
});
