// Step 2's build-up (`game/edit/buildup.ts`) on a town laid as step 2 lays one (the `land` and `town`
// jobs: its plan, the land as laid): a street built up the same way twice for the same seed; nothing
// built off the street's frontage and nothing standing moved; a street through the centre gets no
// homes there, a village's street a village's kinds; the homes the same whichever mode; undo gives
// the draft's bytes back; a civic place offered until the town has it; a street built up until it is
// full, then refused with why; its buildings cleared; on hilly land the settle takes every building
// built (those that would break a slope limit left out first); a street drawn with the town's buildings
// along it in one edit; and a town built up so has no hard problem it hadn't (`edit/checked.ts` `editedProblems`).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('build-up', async () => {
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
  const { plotRect, useTownContent } = await import('../src/game/town');
  const { runTownJob } = await import('../src/game/townJobs');
  const { inside, lengths, RoadIndex } = await import('../src/game/roads');
  const { civicChoices, civicOf } = await import('../src/game/backdrop');
  const { removeLots } = await import('../src/game/edit/lots');
  const { entriesOf, frontOf } = await import('../src/game/edit/ground');
  const { TownHistory } = await import('../src/game/edit/history');
  const { editedProblems } = await import('../src/game/edit/checked');
  const { addCivic, areaOfRoad, buildingsOn, builtAlong, buildUp, clearBuildings, civicOffered } = await import('../src/game/edit/buildup');
  const { drawStreet, removeRoad } = await import('../src/game/edit/streets');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type TownJob = import('../src/game/townJobs').TownJob;
  type TownResult = import('../src/game/townJobs').TownResult;
  type WorldOptions = import('../src/game/world').WorldOptions;
  type Edited = import('../src/game/edit/edited').Edited;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  const inWorker = <K extends TownJob['kind']>(job: Extract<TownJob, { kind: K }>) => structuredClone(runTownJob(content, t, structuredClone(job))) as Extract<TownResult, { kind: K }>;
  /** A town's plan as step 2 edits it (the land as laid under it). */
  const planOf = (world: string, seed: number, options: Partial<WorldOptions>): NeighbourhoodDraft => {
    const land = inWorker({ kind: 'land', world, seed, options });
    const town = inWorker({ kind: 'town', land: land.draft, seed, options, waterKept: true });
    if (!town.plan) throw new Error('the town has no plan');
    return town.plan;
  };
  const bytes = (v: unknown) => JSON.stringify(v, (_, x) => (ArrayBuffer.isView(x) ? Array.from(x as Int16Array).join(',') : x));
  const made = (e: Edited): NeighbourhoodDraft => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e.draft;
  };
  const centroid = (poly: readonly [number, number][]): [number, number] => [poly.reduce((s, p) => s + p[0], 0) / poly.length, poly.reduce((s, p) => s + p[1], 0) / poly.length];
  /** `draft` with road `road` stripped bare: its lots and the buildings fronting it gone. */
  const bare = (draft: NeighbourhoodDraft, road: number): NeighbourhoodDraft => {
    const e = entriesOf(draft, t);
    const lots = new Set(draft.slots.filter((_, i) => e[i].road === road).map((s) => s.index));
    return removeLots(draft, lots, new Set(buildingsOn(draft, road)));
  };
  /** The roads of area `area` (streets and lanes) by length, longest first. */
  const streetsOf = (draft: NeighbourhoodDraft, area: number) =>
    draft.roads
      .map((r, i) => ({ r, i, L: lengths(r.points).at(-1)! }))
      .filter(({ r }) => r.area === area && (r.kind === 'street' || r.kind === 'lane') && !r.bridge)
      .sort((a, b) => b.L - a.L)
      .map(({ i }) => i);

  const town = planOf('country', 11, { town: 'town', villages: 'few', size: 'medium' });
  const centre = town.open!.centre![0].poly;
  /** A main-town street through none of the centre, stripped bare. */
  const plainRoad = streetsOf(town, 0).find((i) => !town.roads[i].points.some(([x, z]) => inside(centre, x, z)))!;
  const plain = bare(town, plainRoad);
  /** The street the centre is on, stripped bare. */
  const highRoad = streetsOf(town, 0).sort((a, b) => town.roads[b].points.filter(([x, z]) => inside(centre, x, z)).length - town.roads[a].points.filter(([x, z]) => inside(centre, x, z)).length)[0];
  const high = bare(town, highRoad);

  test('the same build-up twice gives the same buildings; the draft it was made on is untouched', () => {
    const before = bytes(plain);
    const a = buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 7 });
    const b = buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 7 });
    expect(bytes(made(a))).toBe(bytes(made(b)));
    expect(bytes(plain)).toBe(before);
    // Another seed, another street's worth.
    const c = made(buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 8 }));
    expect(bytes(c.open!.backdrop)).not.toBe(bytes(made(a).open!.backdrop));
  });

  test('nothing is built off the street, and nothing standing moves', () => {
    const e = buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 7 });
    const d = made(e);
    const had = plain.open!.backdrop!.length;
    const fresh = d.open!.backdrop!.slice(had);
    expect(fresh.length).toBeGreaterThan(3);
    expect(d.open!.backdrop!.slice(0, had)).toEqual(plain.open!.backdrop!);
    expect(d.slots.slice(0, plain.slots.length).map((s) => [s.x, s.z, s.facing, s.template])).toEqual(plain.slots.map((s) => [s.x, s.z, s.facing, s.template]));
    expect(d.open!.places!.slice(0, plain.open!.places!.length)).toEqual(plain.open!.places!);
    // Every new building fronts the street (the road nearest it), and every new home meets it.
    const on = new Set(buildingsOn(d, plainRoad));
    for (let k = had; k < d.open!.backdrop!.length; k++) expect(on.has(k)).toBe(true);
    const entries = entriesOf(d, t);
    for (let i = plain.slots.length; i < d.slots.length; i++) expect(entries[i].road).toBe(plainRoad);
    // The new places named, none twice, and each building's place is its own.
    const names = d.open!.places!.map((p) => p.name);
    expect(new Set(names).size).toBe(names.length);
    d.open!.backdrop!.forEach((b) => b.place !== undefined && expect(d.open!.places![b.place]).toBeDefined());
    console.log(`plain street: ${d.slots.length - plain.slots.length} homes, ${fresh.length} buildings, ${d.open!.places!.length - plain.open!.places!.length} places, left out ${'leftOut' in e ? e.leftOut : 0}`);
  });

  test("a street through the centre gets no homes there, and the centre's buildings", () => {
    const d = made(buildUp(high, t, content, { road: highRoad, mode: 'both', seed: 3 }));
    for (const s of d.slots.slice(high.slots.length)) {
      const [x, z] = frontOf(t, s);
      expect(inside(centre, x, z)).toBe(false);
    }
    const inCentre = d.open!.backdrop!.slice(high.open!.backdrop!.length).filter((b) => inside(centre, ...centroid(b.poly)));
    expect(inCentre.length).toBeGreaterThan(3);
    const CENTRE_USES = new Set(['shop', 'flats', 'cafe', 'restaurant', 'office', 'terrace', 'hall']);
    for (const b of inCentre) expect(CENTRE_USES.has(b.use)).toBe(true);
    console.log(`high street: ${inCentre.length} buildings in the centre (${[...new Set(inCentre.map((b) => b.use))].join(', ')}), ${d.slots.length - high.slots.length} homes outside it`);
  });

  test("a village's street gets a village's kinds and homes", () => {
    const area = town.open!.areas.findIndex((_, a) => a > 0 && streetsOf(town, a).length > 0);
    expect(area).toBeGreaterThan(0);
    const road = streetsOf(town, area)[0];
    expect(areaOfRoad(town, road)).toBe(area);
    const v = bare(town, road);
    const d = made(buildUp(v, t, content, { road, mode: 'both', seed: 5 }));
    const fresh = d.open!.backdrop!.slice(v.open!.backdrop!.length);
    expect(d.slots.length).toBeGreaterThan(v.slots.length);
    const HAMLET_USES = new Set(['house', 'terrace', 'workshop', 'barn', 'shop', 'cafe', 'hall']);
    for (const b of fresh) {
      expect(HAMLET_USES.has(b.use)).toBe(true);
      expect(b.area).toBe(area);
    }
    console.log(`village street: ${d.slots.length - v.slots.length} homes, ${fresh.length} buildings (${[...new Set(fresh.map((b) => b.use))].join(', ')})`);
  });

  test('a country road out of every town and village is not built up, and says why', () => {
    const road = town.roads.findIndex((_, i) => town.roads[i].kind === 'country' && !town.roads[i].bridge && areaOfRoad(town, i) < 0);
    expect(road).toBeGreaterThanOrEqual(0);
    const e = buildUp(town, t, content, { road, mode: 'both', seed: 1 });
    expect('refused' in e && e.refused).toMatch(/country road out of the town and its villages/);
  });

  test('the homes stay the same whichever mode', () => {
    const sig = (d: NeighbourhoodDraft) => d.slots.slice(plain.slots.length).map((s) => JSON.stringify(plotRect(t, s)) + s.template);
    const homes = sig(made(buildUp(plain, t, content, { road: plainRoad, mode: 'homes', seed: 9 })));
    const both = sig(made(buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 9 })));
    const buildings = made(buildUp(plain, t, content, { road: plainRoad, mode: 'buildings', seed: 9 }));
    expect(both.length).toBeGreaterThan(0);
    // (Both: the main town's homes on a share of the frontage, the first the lot fill puts down.)
    expect(both.length).toBeLessThan(homes.length);
    expect(homes.slice(0, both.length)).toEqual(both);
    expect(buildings.slots.length).toBe(plain.slots.length);
    expect(buildings.open!.backdrop!.length).toBeGreaterThan(plain.open!.backdrop!.length);
  });

  test("undo gives the draft's bytes back", () => {
    const history = new TownHistory(plain);
    const before = bytes(history.draft);
    history.push(made(buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 2 })));
    expect(bytes(history.draft)).not.toBe(before);
    expect(bytes(history.undo())).toBe(before);
  });

  test('a street is built up until it is full, then refused with why; its buildings cleared', () => {
    let d = plain;
    let refused = '';
    for (let k = 0; k < 12 && !refused; k++) {
      const e = buildUp(d, t, content, { road: plainRoad, mode: 'both', seed: 100 + k });
      if ('refused' in e) refused = e.refused;
      else d = e.draft;
    }
    expect(refused).toBe(`nothing more fits along ${plain.roads[plainRoad].name}`);
    const on = buildingsOn(d, plainRoad);
    expect(on.length).toBeGreaterThan(0);
    const cleared = clearBuildings(d, plainRoad);
    const c = made(cleared);
    expect('cleared' in cleared && cleared.cleared.backdrop).toEqual(on);
    expect(buildingsOn(c, plainRoad)).toEqual([]);
    expect(c.open!.backdrop!.length).toBe(d.open!.backdrop!.length - on.length);
    // Their places with them; every building's place still its own.
    c.open!.backdrop!.forEach((b) => b.place !== undefined && expect(c.open!.places![b.place]).toBeDefined());
    expect(c.slots).toEqual(d.slots);
    expect('refused' in clearBuildings(c, plainRoad) && (clearBuildings(c, plainRoad) as { refused: string }).refused).toMatch(/nothing is built along/);
  });

  test('a civic place is offered until the town has it, then refused', () => {
    // The town's church taken away, offered, put back by hand.
    const church = town.open!.backdrop!.findIndex((b) => b.area === 0 && b.place !== undefined && civicOf(town.open!.places![b.place]) === 1);
    expect(church).toBeGreaterThanOrEqual(0);
    const without = removeLots(town, new Set(), new Set([church]));
    expect(civicChoices(town, 0).some((c) => c.index === 1)).toBe(false);
    expect(civicOffered(without, 0).filter((c) => c.civic === 1).length).toBe(1);
    // Where it stood (the town is built up close: elsewhere it may not fit), the spot found to the metre.
    const door = town.open!.places![town.open!.backdrop![church].place!];
    const near = new RoadIndex(without.roads).nearest(door.x, door.z, 60)!;
    let put: NeighbourhoodDraft | null = null;
    for (let s = Math.floor(near.s) - 3; s <= near.s + 3 && !put; s++) {
      for (const side of [1, -1] as const) {
        const e = addCivic(without, t, content, { civic: 1, road: near.road, s, side, seed: 4 });
        if ('draft' in e) {
          put = e.draft;
          // The same op asked again: the same edit (the Lot tool asks on every pointer move).
          expect(addCivic(without, t, content, { civic: 1, road: near.road, s, side, seed: 4 })).toBe(e);
          break;
        }
      }
    }
    expect(put).not.toBeNull();
    const d = put!;
    const b = d.open!.backdrop!.at(-1)!;
    expect(b.use).toBe('church');
    expect(civicOf(d.open!.places![b.place!])).toBe(1);
    expect(d.open!.places![b.place!].name).toMatch(/^St .+'s Church$/);
    expect(civicOffered(d, 0).some((c) => c.civic === 1)).toBe(false);
    const again = addCivic(d, t, content, { civic: 1, road: streetsOf(d, 0)[0], s: 40, side: 1, seed: 1 });
    expect('refused' in again && again.refused).toMatch(/has a church already/);
  });

  test('on hilly land the settle takes every building a street is built up with: those that would break a slope limit are left out first', () => {
    const settle = (steps: readonly NeighbourhoodDraft[]) => inWorker({ kind: 'edit', plan: steps[steps.length - 1], ops: steps.length - 1, steps, check: false });
    const hilly = planOf('heath', 5, { town: 'town', villages: 'few', size: 'medium', hills: 'hilly' });
    let [built, left] = [0, 0];
    for (const road of streetsOf(hilly, 0).slice(0, 3)) {
      const b = bare(hilly, road);
      const e = buildUp(b, t, content, { road, mode: 'both', seed: 1 });
      if ('refused' in e) continue;
      built += e.draft.open!.backdrop!.length - b.open!.backdrop!.length;
      left += e.leftOut ?? 0;
      expect(settle([b, e.draft]).refused).toEqual([]);
    }
    expect(built).toBeGreaterThan(10);
    console.log(`hilly: ${built} buildings, ${left} left out`);
  }, 120_000);

  test('a street drawn with the town buildings along it is one edit: the street, its homes and its buildings', () => {
    // A dead-end street of the main town taken up, and drawn again as the Street tool draws one.
    let found: { gone: NeighbourhoodDraft; points: [number, number][]; drawn: Edited } | null = null;
    for (const road of streetsOf(town, 0)) {
      const r = town.roads[road];
      if (r.ends[0] !== 'join' || r.ends[1] !== 'dead') continue;
      const e = removeRoad(town, t, road);
      if (!('draft' in e)) continue;
      // (Its line again, or its ends and middle: drawn over, its curve is made again.)
      const pts = r.points.map(([x, z]): [number, number] => [x, z]);
      for (const points of [pts, [pts[0], pts[Math.floor(pts.length / 2)], pts[pts.length - 1]]]) {
        const drawn = drawStreet(e.draft, t, content, { points, kind: 'street', clear: true, fill: false, seed: 6 });
        if ('draft' in drawn) found = { gone: e.draft, points, drawn };
        if (found) break;
      }
      if (found) break;
    }
    expect(found).not.toBeNull();
    const { gone, points, drawn } = found!;
    const d = made(builtAlong(gone, drawn, t, content, 'both', 6));
    const road = d.roads.findIndex((r) => !gone.roads.includes(r));
    expect(road).toBeGreaterThanOrEqual(0);
    expect(areaOfRoad(d, road)).toBe(0);
    expect(d.slots.length).toBeGreaterThan(gone.slots.length);
    expect(buildingsOn(d, road).length).toBeGreaterThan(0);
    expect(made(builtAlong(gone, drawn, t, content, 'both', 6))).toEqual(d);
    // Homes alone: `drawStreet`'s own fill, no buildings.
    const homes = made(drawStreet(gone, t, content, { points, kind: 'street', clear: true, fill: true, seed: 6 }));
    expect(homes.open!.backdrop!.length).toBe(gone.open!.backdrop!.length);
  });

  test('a town built up has no hard problem it had not', () => {
    const was = new Set(editedProblems(town, t, content).hard);
    let d = plain;
    d = made(buildUp(d, t, content, { road: plainRoad, mode: 'both', seed: 7 }));
    const highUp = buildUp(bare(d, highRoad), t, content, { road: highRoad, mode: 'both', seed: 7 });
    d = made(highUp);
    const fresh = editedProblems(d, t, content).hard.filter((p) => !was.has(p));
    expect(fresh).toEqual([]);
  });
});
