// The named places' lots (`game/placeLots.ts`): every civic place and every other kind of named place
// has a layout of its own in houses.json (kind `place`, on a lot of its own size), and every such
// layout is one of theirs; a lot made for one is its layout's size, a shop, dressed as its kind and
// named as its place; any public lot's place is the one the generator gives it, a civic one's with
// its kind's hours; a layout's storey above goes on the town's next rows with stairs up to it; and the
// town file carries the lots' looks (`meta.lotLooks`) and the places' hours (`plots[].hours`). (In
// generated towns: `placeLots.slow.test.ts`.)
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('named places as lots', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  usePublicFetch();
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { mulberry32 } = await import('../src/game/noise');
  const { CIVIC, HOURS, hoursFor, LOT_HOURS, namerFor, ZONE } = await import('../src/game/places');
  const { assembleTown, generateWorld, layoutTown, useTownContent, vacantSlots } = await import('../src/game/town');
  const { lotName, lotPlaceKind, placeOfTemplate, placeTemplate, placeUnit, slotPlace } = await import('../src/game/placeLots');
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type PlotSlot = import('../src/game/town').PlotSlot;
  type ZoneKind = import('../src/game/places').ZoneKind;
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const t = await templatesOf(content);

  /** A small generated world (a hamlet, no villages). */
  const hamlet = generateWorld(content, t, 'country', 4245, undefined, undefined, { town: 'hamlet', villages: 'none' });
  const layouts = t.parks.filter((p) => p.kind === 'place');
  const zones = Object.keys(ZONE) as ZoneKind[];
  const SIZES = ['hamlet', 'village', 'town', 'city'] as const;

  /** `draft` with its first public lot in town (not an outdoor spot) made a lot of civic place `civic` (its index into `CIVIC`), as `placeUnit` makes one. */
  const withCivic = (draft: NeighbourhoodDraft, civic: number): { draft: NeighbourhoodDraft; slot: PlotSlot } => {
    const at = draft.slots.find((s) => s.kind === 'shop' || s.kind === 'park')!;
    const slot: PlotSlot = { ...at, ...placeUnit(t, CIVIC[civic], 'town', mulberry32(civic + 1)), index: at.index, area: at.area };
    return { draft: { ...draft, slots: draft.slots.map((s) => (s.index === at.index ? slot : s)) }, slot };
  };

  test('every civic place and every other kind has its layouts, each a named place\'s on a lot of its own', () => {
    expect(layouts.length).toBeGreaterThanOrEqual(28);
    const used = new Set<string>();
    for (const c of CIVIC) {
      for (const size of SIZES) {
        const id = placeTemplate(c, size, Math.random);
        expect(layouts.map((p) => p.id), `${c.label} in a ${size}`).toContain(id);
        expect(placeOfTemplate(id)).toEqual({ civic: CIVIC.indexOf(c) });
        used.add(id);
      }
    }
    for (const k of zones) {
      for (const id of [...ZONE[k].templates, ...ZONE[k].compact]) {
        expect(layouts.map((p) => p.id), k).toContain(id);
        expect(placeOfTemplate(id)).toEqual({ zone: k });
        used.add(id);
      }
    }
    expect(layouts.filter((p) => !used.has(p.id)).map((p) => p.id)).toEqual([]);
    for (const p of layouts) expect(p.lot, p.id).toBeDefined();
    // A hamlet's hall is the parish hall, a town's the town hall; a hamlet's shop the village shop.
    expect(placeTemplate(CIVIC[0], 'hamlet', Math.random)).toBe('parishHall');
    expect(placeTemplate(CIVIC[0], 'city', Math.random)).toBe('townHall');
    expect(placeTemplate(CIVIC.find((c) => c.label === 'Boutique')!, 'hamlet', Math.random)).toBe('villageShop');
    // The park, the sim's own shops and the countryside's places are no named place's.
    for (const id of ['park', 'market', 'cafe', 'grocer', 'picnicMeadow']) expect(placeOfTemplate(id)).toBeNull();
  });

  test("a lot made for a place is its layout's size, a shop, dressed as its kind and as tall as its real storeys, the same from the same draws", () => {
    for (const what of [...CIVIC, ...zones]) {
      const u = placeUnit(t, what, 'town', mulberry32(7));
      const p = t.parks.find((q) => q.id === u.template)!;
      const kind = typeof what === 'string' ? ZONE[what].kind : what.kind;
      expect(u.kind).toBe('shop');
      expect([u.w, u.d]).toEqual([p.lot!.width, p.lot!.depth]);
      expect(u.dress!.use).toBe(kind.use);
      // (Its ground floor, and the storey above where its layout has one: nothing stands over it yet.)
      expect(u.dress!.storeys).toBe(p.upper ? 2 : 1);
      expect(kind.roofs).toContain(u.dress!.roof);
      expect(placeUnit(t, what, 'town', mulberry32(7))).toEqual(u);
    }
  });

  test('a lot is named as its place, none twice', () => {
    const taken = new Set<string>();
    const namer = namerFor(mulberry32(3), 'Ashby', 'town', ['Smith', 'Jones'], ['Mill Lane, Ashby'], taken);
    const school = CIVIC.findIndex((c) => c.label === 'School');
    expect(lotName({ template: placeTemplate(CIVIC[school], 'town', Math.random) }, namer)).toBe('Ashby School');
    expect(lotName({ template: 'school' }, namer)).toBe('Ashby School 2');
    expect(lotName({ template: 'shopSmall' }, namer)).toBeString();
    expect(lotName({ template: 'grocer' }, namer)).toBeNull();
    expect(taken.has('Ashby School')).toBe(true);
  });

  test("a public lot's place is the one the generator gives it, called as its lot is", () => {
    // (A flats building is homes, no place.)
    const publicLots = hamlet.slots.filter((s) => s.kind !== 'house' && s.kind !== 'flats');
    expect(publicLots.length).toBeGreaterThan(0);
    // (But the main town's first office, which takes the careers no place there has, its hours widened to theirs.)
    const office = hamlet.open!.places!.find((p) => p.area === 0 && p.kind === 'office')!;
    for (const s of publicLots) {
      const had = hamlet.open!.places!.find((p) => p.plot === s.index)!;
      const made = slotPlace(hamlet, t, content, s);
      if (had === office) {
        expect(had.work!.slice(0, made.work?.length ?? 0)).toEqual(made.work ?? []);
        expect(had.hours!.open).toBeLessThanOrEqual(made.hours!.open);
      } else expect(made).toEqual({ ...had, name: s.name });
      if (s.dress) expect(had.name).toBe(s.name);
      expect(lotPlaceKind(s)).toBe(had.kind);
    }
  });

  test("a civic place's lot keeps its kind's hours, widened to its careers' shifts", () => {
    const school = CIVIC.findIndex((c) => c.label === 'School');
    const { draft, slot } = withCivic(hamlet, school);
    const p = slotPlace(draft, t, content, slot);
    expect(p.kind).toBe('school');
    expect(p.work).toEqual(['education']);
    expect(p.hours).toEqual(hoursFor(HOURS.school, ['education'], content.careerCategories));
    expect(p.hours).not.toEqual(hoursFor(LOT_HOURS, ['education'], content.careerCategories));
    // With a namer, a name another place has is made the area's.
    const taken = new Set([slot.name]);
    const named = slotPlace(draft, t, content, slot, namerFor(mulberry32(1), 'Ashby', 'hamlet', ['Smith'], [], taken));
    expect(named.name).not.toBe(slot.name);
    expect(named.name).toEndWith(slot.name.replace(/^The /, ''));
  });

  test("a layout's storey above goes on the town's next rows, with stairs up to it, and the town file has the looks and the hours", () => {
    const hotel = CIVIC.findIndex((c) => c.label === 'Hotel');
    const { draft, slot } = withCivic(hamlet, hotel);
    expect(t.parks.find((p) => p.id === slot.template)!.upper).toBeDefined();
    const layout = layoutTown(content, t, draft, null);
    expect(layout.plots[slot.index].storeys).toBe(2);
    expect(layout.storeys).toBeGreaterThanOrEqual(2);
    const r = layout.plots[slot.index];
    const inPlot = (x: number, z: number) => x >= r.x && x <= r.x + r.w && z >= r.z && z <= r.z + r.d;
    const up = layout.walls.filter(([x0, z0, x1, z1]) => z0 >= draft.depth && z1 >= draft.depth && z0 < 2 * draft.depth && inPlot(x0, z0 - draft.depth) && inPlot(x1, z1 - draft.depth));
    expect(up.length).toBeGreaterThanOrEqual(4);
    expect(layout.objects.some((o) => o.def === 'stairs' && inPlot(o.x, o.z))).toBe(true);
    // The town file: the lot's look in the meta, every public lot's hours as its place has them.
    const places = [...draft.open!.places!.filter((p) => p.plot !== slot.index), slotPlace(draft, t, content, slot)];
    const withPlaces = { ...draft, open: { ...draft.open!, places } };
    const home = vacantSlots(withPlaces)[0].index;
    const file = JSON.parse(assembleTown(content, t, withPlaces, randomHousehold(content, assets, 2, [], {}, mulberry32(1)), home, undefined, mulberry32(2)));
    expect(file.meta.lotLooks[slot.index]).toEqual(slot.dress);
    expect(Object.keys(file.meta.lotLooks).map(Number)).toEqual(withPlaces.slots.flatMap((s, i) => (s.dress ? [i] : [])));
    for (const [i, s] of withPlaces.slots.entries()) {
      const p = places.find((q) => q.plot === i);
      expect(file.plots[i].hours, s.name).toEqual(p?.hours);
    }
    expect(file.plots[slot.index].storeys).toBe(2);
    // A town without named places' lots has no looks.
    const bare = { ...hamlet, slots: hamlet.slots.map(({ dress: _, ...s }) => s) };
    expect(JSON.parse(assembleTown(content, t, bare, randomHousehold(content, assets, 2, [], {}, mulberry32(1)), vacantSlots(bare)[0].index, undefined, mulberry32(2))).meta.lotLooks).toBeUndefined();
  });
});
