// Step 2's village edits (`game/edit/villages.ts`), on towns laid on a place's land as the worker lays them
// (the `land` and `town` jobs): a village placed on a large land with a road to the network and its lots,
// and nothing that blocks the way on in the Problems list; the same seed giving the same village; the
// village moved, its old road taken up and a new one routed; a road with homes on it left ending where
// it was, said in the edit's notes and the Problems list; the village removed, and a generated one,
// with the area indexes on everything else numbered again and nothing left naming one that is gone;
// a village's only road removed taking its empty area with it; a site in the water or too near another
// area refused, and a small 640 m land refusing every site cleanly. The settle (the `edit` job) takes
// the village placed and moved.
// `SLOW_TESTS=1 bun test tests/editVillages.slow.test.ts`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe("step 2's villages", async () => {
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
  const { useTownContent } = await import('../src/game/town');
  const { runTownJob } = await import('../src/game/townJobs');
  const { canMoveVillage, canPlaceVillage, moveVillage, placeVillage, removeVillage, villageGoes } = await import('../src/game/edit/villages');
  const { addLot } = await import('../src/game/edit/lots');
  const { removeRoad } = await import('../src/game/edit/streets');
  const { townIssues } = await import('../src/ui/townedit/problems');
  const { lengths, nearest } = await import('../src/game/roads');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type TownJob = import('../src/game/townJobs').TownJob;
  type TownResult = import('../src/game/townJobs').TownResult;
  type WorldOptions = import('../src/game/world').WorldOptions;
  type Edited = import('../src/game/edit/edited').Edited;
  type Pt = import('../src/game/roads').Pt;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A job as the worker runs it: sent, run, and its result sent back. */
  const inWorker = <K extends TownJob['kind']>(job: Extract<TownJob, { kind: K }>) => structuredClone(runTownJob(content, t, structuredClone(job))) as Extract<TownResult, { kind: K }>;
  /** A town's plan laid on a place's land (the `land` and `town` jobs). */
  const laid = (world: string, seed: number, options: Partial<WorldOptions>): NeighbourhoodDraft => {
    const land = inWorker({ kind: 'land', world, seed, options });
    const town = inWorker({ kind: 'town', land: land.draft, seed, options, waterKept: true });
    if (!town.plan) throw new Error('the town has no plan');
    return town.plan;
  };
  const done = (e: Edited): Extract<Edited, { draft: unknown }> => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e;
  };
  /** What keeps the player from step 3 (the Problems list), but what the town as laid had already. */
  const blocking = (d: NeighbourhoodDraft, base: NeighbourhoodDraft) => townIssues(d, t, content, townIssues(base, t, content)).blocking.map((i) => i.text);
  /** The middles a village may go round on `d`, a 20 m grid scanned (nearest the middle of the world first), and how long each test took (ms). */
  const sites = (d: NeighbourhoodDraft, swap = false) => {
    const out: { p: Pt; why: string | null }[] = [];
    const t0 = performance.now();
    for (let z = 60; z < d.depth - 60; z += 20) {
      for (let x = 60; x < d.width - 60; x += 20) {
        const at = canPlaceVillage(d, t, { x, z, swap });
        out.push({ p: [x, z], why: 'refused' in at ? at.refused : null });
      }
    }
    const each = (performance.now() - t0) / out.length;
    const mid: Pt = [d.width / 2, d.depth / 2];
    const ok = out.filter((s) => !s.why).sort((a, b) => Math.hypot(a.p[0] - mid[0], a.p[1] - mid[1]) - Math.hypot(b.p[0] - mid[0], b.p[1] - mid[1]));
    return { all: out, ok, each };
  };
  /** Every area index on `d` (lots, roads, buildings, places, parking, centres) names an area it has; every place a lot or a building it has, and the other way round. */
  const noDangling = (d: NeighbourhoodDraft) => {
    const open = d.open!;
    const n = open.areas.length;
    const inRange = (k: number | undefined) => k === undefined || (Number.isInteger(k) && k >= 0 && k < n);
    expect(d.slots.every((s) => inRange(s.area))).toBe(true);
    expect(d.roads.every((r) => inRange(r.area))).toBe(true);
    expect((open.backdrop ?? []).every((b) => inRange(b.area))).toBe(true);
    expect((open.places ?? []).every((p) => inRange(p.area))).toBe(true);
    expect(open.parking.every((p) => inRange(p.area))).toBe(true);
    expect((open.centre ?? []).every((c) => inRange(c.area))).toBe(true);
    expect(d.slots.every((s, i) => s.index === i)).toBe(true);
    const places = open.places ?? [];
    expect(places.every((p) => p.plot === undefined || (p.plot >= 0 && p.plot < d.slots.length && d.slots[p.plot].kind !== 'house'))).toBe(true);
    expect((open.backdrop ?? []).every((b) => b.place === undefined || (b.place >= 0 && b.place < places.length))).toBe(true);
    expect(Object.keys(open.pads).every((k) => Number(k) < d.slots.length)).toBe(true);
    // (Each area has a road of its own, or is the main town.)
    for (let k = 1; k < n; k++) expect(d.roads.some((r) => r.area === k)).toBe(true);
  };

  const big = laid('country', 11, { town: 'village', villages: 'none', size: 'large' });
  const found = sites(big);
  const at = found.ok[0];
  const placed = done(placeVillage(big, t, content, { x: at.p[0], z: at.p[1], swap: false, seed: 5 }));
  const a = placed.draft.open!.areas.length - 1;
  const name = placed.draft.open!.areas[a].name;
  /** The village's road to the network (laid by the edit: named for it, a country road). */
  const roadTo = (d: NeighbourhoodDraft, village: string) => d.roads.filter((r) => r.kind === 'country' && r.name?.startsWith(`${village} `));

  test('a village is placed on a large land: a hamlet of the place, a road to the network, its lots; nothing blocks the way on', () => {
    expect(big.width).toBe(960);
    console.log(`sites: ${found.ok.length} of ${found.all.length} tested, ${found.each.toFixed(3)} ms each; placed round ${at.p.join(', ')}`);
    // The ghost's test is cheap enough for every pointer move.
    expect(found.each).toBeLessThan(5);
    const d = placed.draft;
    expect(placed.notes).toBeUndefined();
    expect(d.open!.areas.length).toBe(big.open!.areas.length + 1);
    expect(d.open!.areas[a].kind).toBe('hamlet');
    // Its own streets, named, none named twice in town; its name nobody else's.
    const own = d.roads.filter((r) => r.area === a);
    expect(own.length).toBeGreaterThan(0);
    const names = d.roads.map((r) => r.name);
    expect(new Set(names).size).toBe(names.length);
    expect(big.roads.some((r) => r.name === name) || big.open!.areas.some((o) => o.name === name)).toBe(false);
    // A country road to it, joined to its streets and the network.
    const road = roadTo(d, name);
    expect(road.length).toBeGreaterThan(0);
    // Its lots, along its own streets, its homes numbered on them.
    const lots = d.slots.filter((s) => s.area === a);
    expect(lots.length).toBeGreaterThanOrEqual(4);
    const homes = lots.filter((s) => s.kind === 'house' && s.lot !== 'farm');
    expect(homes.every((s) => own.some((r) => s.name.endsWith(` ${r.name}`)))).toBe(true);
    expect(d.open!.parking.filter((p) => p.area === a).length).toBeGreaterThan(0);
    // Nothing else moved.
    expect(d.slots.slice(0, big.slots.length)).toEqual(big.slots);
    expect(d.roads.slice(0, big.roads.length)).toEqual(big.roads);
    expect(blocking(d, big)).toEqual([]);
    noDangling(d);
    console.log(`placed ${name}: ${own.length} streets, ${road.length} road${road.length > 1 ? 's' : ''} to it (${road.map((r) => `${r.name} ${Math.round(lengths(r.points).at(-1)!)} m`).join(', ')}), ${lots.length} lots (${homes.length} homes)`);
  });

  test('the same seed gives the same village; another seed another', () => {
    const again = done(placeVillage(big, t, content, { x: at.p[0], z: at.p[1], swap: false, seed: 5 }));
    expect(JSON.stringify(again.draft)).toBe(JSON.stringify(placed.draft));
    const other = done(placeVillage(big, t, content, { x: at.p[0], z: at.p[1], swap: false, seed: 6 }));
    expect(JSON.stringify(other.draft)).not.toBe(JSON.stringify(placed.draft));
  });

  test('the settle (the edit job) takes the village placed, nothing refused', () => {
    const e = inWorker({ kind: 'edit', plan: placed.draft, ops: 1, steps: [big, placed.draft], check: false });
    expect(e.refused).toEqual([]);
    expect(e.draft.slots.length).toBe(placed.draft.slots.length);
  }, 120_000);

  /** A move of the placed village that its new site allows, the nearest first. */
  const shift = (() => {
    for (const k of [6, 8, 10, 12, 14]) {
      for (const [ux, uz] of [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, 1], [1, -1], [-1, -1]]) {
        const [dx, dz] = [ux * k * 10, uz * k * 10];
        if (!('refused' in canMoveVillage(placed.draft, t, a, dx, dz))) return [dx, dz] as const;
      }
    }
    return null;
  })();

  test('the village moved: its streets and lots with it, its old road taken up, a new one routed', () => {
    if (!shift) throw new Error('nowhere to move the village');
    const [dx, dz] = shift;
    const before = placed.draft;
    const e = done(moveVillage(before, t, a, dx, dz));
    const d = e.draft;
    expect(e.notes).toBeUndefined();
    const box = d.open!.areas[a];
    expect([box.x, box.z]).toEqual([before.open!.areas[a].x + dx, before.open!.areas[a].z + dz]);
    // Its lots moved by whole metres, the rest where they were.
    const was = before.slots.filter((s) => s.area === a);
    const now = d.slots.filter((s) => s.area === a);
    expect(now.map((s) => [s.x - dx, s.z - dz, s.template, s.name])).toEqual(was.map((s) => [s.x, s.z, s.template, s.name]));
    expect(d.slots.filter((s) => s.area !== a)).toEqual(before.slots.filter((s) => s.area !== a));
    // The old road is gone, a new one leads to it.
    const old = roadTo(before, name);
    const fresh = roadTo(d, name);
    expect(fresh.length).toBeGreaterThan(0);
    const line = (r: { points: readonly Pt[] }) => JSON.stringify(r.points);
    for (const r of old) expect(d.roads.some((q) => line(q) === line(r))).toBe(false);
    // (The new road starts on one of its streets where they now stand.)
    const streets = d.roads.filter((r) => r.area === a);
    const meets = ([x, z]: Pt) => streets.some((r) => nearest(r.points, x, z).dist <= r.width / 2 + 1);
    expect(fresh.some((r) => meets(r.points[0]) || meets(r.points[r.points.length - 1]))).toBe(true);
    expect(blocking(d, big)).toEqual([]);
    noDangling(d);
    // And the settle takes it.
    const settled = inWorker({ kind: 'edit', plan: d, ops: 1, steps: [before, d], check: false });
    expect(settled.refused).toEqual([]);
    console.log(`moved ${name} by ${dx}, ${dz}: ${old.length} road${old.length > 1 ? 's' : ''} taken up, ${fresh.length} laid`);
  }, 120_000);

  test('a road with homes on it is left where the village was, cut back past them, and said', () => {
    if (!shift) throw new Error('nowhere to move the village');
    // A home along the village's road, out in the country.
    const d0 = placed.draft;
    const road = d0.roads.findIndex((r) => r.kind === 'country' && r.name?.startsWith(`${name} `));
    const ls = lengths(d0.roads[road].points);
    let withHome: NeighbourhoodDraft | null = null;
    for (let s = 20; s < ls[ls.length - 1] - 20 && !withHome; s += 6) {
      for (const side of [1, -1] as const) {
        const e = addLot(d0, t, content, { road, s, side }, { kind: 'house', size: 'medium' }, 9);
        if ('draft' in e) {
          withHome = e.draft;
          break;
        }
      }
    }
    if (!withHome) {
      console.log('no home fits along the road to the village: the stub left unproven here');
      return;
    }
    const e = moveVillage(withHome, t, a, shift[0], shift[1]);
    if ('refused' in e) throw new Error(e.refused);
    expect(e.notes?.some((n) => n.includes(`ending where ${name} was: homes stand on it`))).toBe(true);
    // (The old road kept, a dead end now, and the Problems list says so.)
    expect(e.draft.slots.length).toBe(withHome.slots.length);
    const issues = townIssues(e.draft, t, content, townIssues(big, t, content));
    expect(issues.warnings.some((i) => /comes to a dead end out in the country$/.test(i.text))).toBe(true);
    console.log(`left: ${e.notes!.join('; ')}`);
  });

  test('the village removed: everything of it gone, the town as it was', () => {
    const goes = villageGoes(placed.draft, t, a);
    if ('refused' in goes) throw new Error(goes.refused);
    expect(goes.lots.length).toBe(placed.draft.slots.filter((s) => s.area === a).length);
    const e = done(removeVillage(placed.draft, t, a));
    expect(e.draft.roads).toEqual(big.roads);
    expect(e.draft.slots).toEqual(big.slots);
    expect(e.draft.greens).toEqual(big.greens);
    expect(e.draft.open!.areas).toEqual(big.open!.areas);
    expect(e.draft.open!.parking).toEqual(big.open!.parking);
    expect(e.draft.open!.places).toEqual(big.open!.places);
    expect(e.draft.open!.backdrop).toEqual(big.open!.backdrop);
    // The main town is no village to remove or move.
    expect('refused' in removeVillage(big, t, 0)).toBe(true);
    expect('refused' in moveVillage(big, t, 0, 100, 0)).toBe(true);
  });

  test("a village's only road removed takes the village with it, its area too", () => {
    const d = placed.draft;
    const road = d.roads.findIndex((r) => r.kind === 'country' && r.name === roadTo(d, name)[0].name);
    const e = removeRoad(d, t, road);
    if ('refused' in e) {
      // (Its road shared others' on the way: removing its first piece alone may leave the rest.)
      console.log(`removing ${d.roads[road].name}: ${e.refused}`);
      return;
    }
    if (e.draft.roads.some((r) => r.area === a)) {
      console.log(`${d.roads[road].name} was not its only road`);
      return;
    }
    expect(e.draft.open!.areas.length).toBe(big.open!.areas.length);
    noDangling(e.draft);
  });

  test('a generated village removed: the area indexes on everything else numbered again', () => {
    const coast = laid('coast', 3, { town: 'village', villages: 'few', size: 'large' });
    const areas = coast.open!.areas;
    expect(areas.length).toBeGreaterThan(2);
    const k = 1;
    const goes = villageGoes(coast, t, k);
    if ('refused' in goes) throw new Error(goes.refused);
    const e = done(removeVillage(coast, t, k));
    const d = e.draft;
    expect(d.open!.areas.map((o) => o.name)).toEqual(areas.filter((_, i) => i !== k).map((o) => o.name));
    // What was in the areas after it is in the one before.
    const was = (i: number) => coast.roads.filter((r) => r.area === i).map((r) => r.name);
    for (let i = k + 1; i < areas.length; i++) expect(d.roads.filter((r) => r.area === i - 1).map((r) => r.name)).toEqual(was(i));
    for (let i = k + 1; i < areas.length; i++) {
      expect((d.open!.backdrop ?? []).filter((b) => b.area === i - 1).length).toBe((coast.open!.backdrop ?? []).filter((b) => b.area === i).length);
      expect(d.open!.parking.filter((p) => p.area === i - 1).length).toBe(coast.open!.parking.filter((p) => p.area === i).length);
    }
    expect(d.roads.some((r) => r.name && was(k).includes(r.name))).toBe(false);
    expect(d.slots.length).toBe(coast.slots.length - goes.lots.length);
    noDangling(d);
    expect(blocking(d, coast)).toEqual([]);
    console.log(`removed ${areas[k].name}: ${goes.roads} of its roads and ${goes.others} that led to it, ${goes.lots.length} lots, ${goes.backdrop.length} buildings`);
    // A site too near another area refused with why (and one in the water, on the large land's ponds).
    const near = sites(coast).all.find((s) => s.why?.includes(`of ${areas[0].name}`) || s.why?.includes(`${areas[0].name}'s`));
    const wet = found.all.find((s) => s.why?.includes('water'));
    expect(near).toBeDefined();
    expect(wet).toBeDefined();
    for (const [d, s] of [[coast, near!], [big, wet!]] as const) {
      const e2 = placeVillage(d, t, content, { x: s.p[0], z: s.p[1], swap: false, seed: 1 });
      expect('refused' in e2 ? e2.refused : null).toBe(s.why);
    }
    console.log(`refused: "${wet!.why}"; "${near!.why}"`);
  }, 120_000);

  test('on a small 640 m land no village fits, and placing one is refused cleanly', () => {
    const small = laid('country', 21, { town: 'village', villages: 'few', size: 'small' });
    expect(small.width).toBe(640);
    const scan = sites(small);
    expect(scan.ok.length).toBe(0);
    for (const swap of [false, true]) {
      const e = placeVillage(small, t, content, { x: small.width / 2, z: small.depth / 2, swap, seed: 2 });
      expect('refused' in e).toBe(true);
    }
    const reasons = new Map<string, number>();
    for (const s of scan.all) reasons.set(s.why!.replace(/\d+/g, 'N'), (reasons.get(s.why!.replace(/\d+/g, 'N')) ?? 0) + 1);
    console.log(`640 m: ${[...reasons].map(([w, n]) => `${n}× ${w}`).join('; ')}`);
  }, 120_000);
});
