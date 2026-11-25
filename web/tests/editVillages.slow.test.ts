// Step 2's village edits (`game/edit/villages.ts`), on towns laid on a place's land as the worker lays them
// (the `land` and `town` jobs): a village placed on a large land with a road to the network and its lots,
// and nothing that blocks the way on in the Problems list; the same seed giving the same village; the
// village moved, its old road taken up and a new one routed; a road with homes on it left ending where
// it was, said in the edit's notes and the Problems list; the village removed, and a generated one,
// with the area indexes on everything else numbered again and nothing left naming one that is gone;
// a village's only road removed taking its empty area with it; a site in the water or too near another
// area refused, and a small 640 m land refusing every site cleanly; a village across a river painted over the
// land bridged to the town, the water under its bridge water again once it is removed. The settle (the `edit`
// job) takes the village placed and moved.
// `SLOW_TESTS=1 bun test tests/editVillages.slow.test.ts`.
import { describe, expect, test } from 'bun:test';
import { inWorkerOf, loadContent, templatesOf, usePublicFetch } from './support/content';

describe("step 2's villages", async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  usePublicFetch();
  const { plotRect, useTownContent } = await import('../src/game/town');
  const { canMoveVillage, canPlaceVillage, moveVillage, placeVillage, removeVillage, villageGoes } = await import('../src/game/edit/villages');
  const { addLot, removeLots } = await import('../src/game/edit/lots');
  const { removeRoad } = await import('../src/game/edit/streets');
  const { townIssues } = await import('../src/ui/townedit/problems');
  const { editedProblems } = await import('../src/game/edit/checked');
  const { COURT, evenly, leftOf, lengths, nearest, pointAt, RoadIndex } = await import('../src/game/roads');
  const { entriesOf, entryOf } = await import('../src/game/edit/ground');
  const { deckTiles } = await import('../src/game/macro');
  const { unrle } = await import('../src/game/landMasks');
  const { wetTile } = await import('../src/game/landEdit');
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type WorldOptions = import('../src/game/world').WorldOptions;
  type Edited = import('../src/game/edit/edited').Edited;
  type Pt = import('../src/game/roads').Pt;
  type RoadPlan = import('../src/core/protocol').RoadPlan;
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  const inWorker = await inWorkerOf(content, t);
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
  /** The town checks' hard problems on an edited town (`townProblems`, as the edit job checks it), but those the town as laid had. */
  const hardNew = (d: NeighbourhoodDraft, base: NeighbourhoodDraft) => {
    const had = new Set(editedProblems(base, t, content).hard);
    return editedProblems(d, t, content).hard.filter((h) => !had.has(h));
  };
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
  /** Every area index on `d` (lots, roads, places, parking and car parks, centres) names an area it has; every place a lot it has; every bay a car park it has, which it is in. */
  const noDangling = (d: NeighbourhoodDraft) => {
    const open = d.open!;
    const n = open.areas.length;
    const inRange = (k: number | undefined) => k === undefined || (Number.isInteger(k) && k >= 0 && k < n);
    expect(d.slots.every((s) => inRange(s.area))).toBe(true);
    expect(d.roads.every((r) => inRange(r.area))).toBe(true);
    expect((open.carParks ?? []).every((p) => inRange(p.area))).toBe(true);
    expect(open.parking.every((b) => b.park === undefined || (b.park >= 0 && b.park < (open.carParks ?? []).length && open.carParks![b.park].area === b.area))).toBe(true);
    expect((open.places ?? []).every((p) => inRange(p.area))).toBe(true);
    expect(open.parking.every((p) => inRange(p.area))).toBe(true);
    expect((open.centre ?? []).every((c) => inRange(c.area))).toBe(true);
    expect(d.slots.every((s, i) => s.index === i)).toBe(true);
    const places = open.places ?? [];
    expect(places.every((p) => p.plot === undefined || (p.plot >= 0 && p.plot < d.slots.length && d.slots[p.plot].kind !== 'house'))).toBe(true);
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
    expect(hardNew(d, big)).toEqual([]);
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
    expect(hardNew(d, big)).toEqual([]);
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
    // (Not where the village moves to: a home there stands in its way.)
    let withHome: NeighbourhoodDraft | null = null;
    let moved: Edited | null = null;
    for (let s = 20; s < ls[ls.length - 1] - 20 && !withHome; s += 6) {
      for (const side of [1, -1] as const) {
        const e = addLot(d0, t, content, { road, s, side }, { kind: 'house', size: 'medium' }, 9);
        if (!('draft' in e)) continue;
        moved = moveVillage(e.draft, t, a, shift[0], shift[1]);
        if ('refused' in moved && moved.refused === 'a lot stands there') continue;
        withHome = e.draft;
        break;
      }
    }
    if (!withHome || !moved) {
      console.log('no home fits along the road to the village: the stub left unproven here');
      return;
    }
    const e = moved;
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
    expect(e.draft.open!.carParks).toEqual(big.open!.carParks);
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

  /**
   * A generated world's first village that may go (`villageGoes`), one with areas after it first (their indexes then
   * numbered again): one whose road another area's street goes on into, with homes along it past a dead end's length,
   * is refused (coast 3's Lindenwick and Wrenby, since the motorway went down before the villages: School Lane would end
   * 79 m past the last road it meets).
   */
  const going = (d: NeighbourhoodDraft): number => {
    const n = d.open!.areas.length;
    const why: string[] = [];
    for (const i of [...Array.from({ length: n - 2 }, (_, i) => i + 1), n - 1]) {
      const goes = villageGoes(d, t, i);
      if (!('refused' in goes)) return i;
      why.push(`${d.open!.areas[i].name}: ${goes.refused}`);
    }
    throw new Error(`no village may go: ${why.join('; ')}`);
  };

  test('a generated village removed: the area indexes on everything else numbered again', () => {
    const coast = laid('coast', 5, { town: 'village', villages: 'few', size: 'large' });
    const areas = coast.open!.areas;
    expect(areas.length).toBeGreaterThan(2);
    const k = going(coast);
    expect(k).toBeLessThan(areas.length - 1);
    const goes = villageGoes(coast, t, k);
    if ('refused' in goes) throw new Error(goes.refused);
    const e = done(removeVillage(coast, t, k));
    const d = e.draft;
    expect(d.open!.areas.map((o) => o.name)).toEqual(areas.filter((_, i) => i !== k).map((o) => o.name));
    // What was in the areas after it is in the one before.
    const was = (i: number) => coast.roads.filter((r) => r.area === i).map((r) => r.name);
    for (let i = k + 1; i < areas.length; i++) expect(d.roads.filter((r) => r.area === i - 1).map((r) => r.name)).toEqual(was(i));
    for (let i = k + 1; i < areas.length; i++) {
      expect((d.open!.carParks ?? []).filter((p) => p.area === i - 1).length).toBe((coast.open!.carParks ?? []).filter((p) => p.area === i).length);
      expect(d.open!.parking.filter((p) => p.area === i - 1).length).toBe(coast.open!.parking.filter((p) => p.area === i).length);
    }
    expect(d.roads.some((r) => r.name && was(k).includes(r.name))).toBe(false);
    expect(d.slots.length).toBe(coast.slots.length - goes.lots.length);
    noDangling(d);
    expect(blocking(d, coast)).toEqual([]);
    expect(hardNew(d, coast)).toEqual([]);
    console.log(`removed ${areas[k].name}: ${goes.roads} of its roads and ${goes.others} that led to it, ${goes.lots.length} lots, ${goes.parking} parking bays`);
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

  /**
   * A generated coast world with a long lane made by hand (so it doesn't hang on how the generator
   * happens to lay the villages): a lane of another area 70 m out into open land (`a` to `m`), and a
   * country road from its end to village `k`'s nearest street. Taking that country road up (the
   * village removed or moved) leaves the lane a dead end 70 m long. And a home on the lane past a
   * court's length (`home`), to add.
   */
  const longLane = (() => {
    type Made = { coast: NeighbourhoodDraft; k: number; d0: NeighbourhoodDraft; home: NeighbourhoodDraft['slots'][number] };
    let coastOnce: NeighbourhoodDraft | null = null;
    const made = new Map<number, Made>();
    return (k = 1): Made => {
      if (made.has(k)) return made.get(k)!;
      const coast = (coastOnce ??= laid('coast', 3, { town: 'village', villages: 'few', size: 'large' }));
      const rects = coast.slots.map((s) => plotRect(t, s));
      const index = new RoadIndex(coast.roads);
      const bounds = (poly: readonly Pt[]) => [Math.min(...poly.map((p) => p[0])), Math.min(...poly.map((p) => p[1])), Math.max(...poly.map((p) => p[0])), Math.max(...poly.map((p) => p[1]))];
      const buildings = (coast.open!.carParks ?? []).map((p) => bounds(p.poly));
      // (Clear of the lots, the car parks and the other roads, the first 20 m but near its street.)
      const free = (a: Pt, b: Pt) =>
        evenly([a, b], 2).every(([x, z]) => {
          const d = Math.hypot(x - a[0], z - a[1]);
          const m = d < 20 ? 2 : 6;
          if (rects.some((r) => x > r.x - m && x < r.x + r.w + m && z > r.z - m && z < r.z + r.d + m)) return false;
          if (buildings.some(([x0, z0, x1, z1]) => x > x0 - m && x < x1 + m && z > z0 - m && z < z1 + m)) return false;
          if (d > 10 && index.nearest(x, z, 10)) return false;
          return x > 30 && z > 30 && x < coast.width - 30 && z < coast.depth - 30;
        });
      // (The lots whose way in would be the lane or the road taken off: a lot's way in elsewhere stays as it was.)
      const before = entriesOf(coast, t);
      let found: { site: { area: number; a: Pt; m: Pt }; lane: RoadPlan; d0: NeighbourhoodDraft } | null = null;
      for (const r of coast.roads) {
        if (found || r.area === undefined || r.area === k || r.kind === 'trail' || r.kind === 'country') continue;
        const ls = lengths(r.points);
        for (let s = 30; s < ls[ls.length - 1] - 30 && !found; s += 6) {
          const { p, t: tg } = pointAt(r.points, s, ls);
          const n = leftOf(tg);
          for (const side of [1, -1]) {
            const m: Pt = [p[0] + n[0] * side * 70, p[1] + n[1] * side * 70];
            if (found || !free(p, m)) continue;
            const site = { area: r.area, a: p, m };
            let [to, best]: [Pt, number] = [[0, 0], Infinity];
            for (const o of coast.roads.filter((q) => q.area === k && q.kind !== 'trail')) {
              const at = nearest(o.points, m[0], m[1]);
              if (at.dist < best) [to, best] = [pointAt(o.points, at.s).p, at.dist];
            }
            const lane: RoadPlan = { points: evenly([p, m], 4), width: 7, ends: ['join', 'join'], kind: 'lane', name: 'Test Lane', area: site.area };
            const road: RoadPlan = { points: evenly([m, to], 4), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Test Road' };
            const laid: NeighbourhoodDraft = { ...coast, roads: [...coast.roads, lane, road] };
            const moved = entriesOf(laid, t).flatMap((e, i) => (e.road !== before[i].road ? [i] : []));
            found = { site, lane, d0: removeLots(laid, new Set(moved)) };
          }
        }
      }
      if (!found) throw new Error('no open land beside a street of another area');
      const { site, d0 } = found;
      const { a, m } = site;
      // (A home on the lane 62 m along it, its way in on the lane.)
      const u: Pt = [(m[0] - a[0]) / 70, (m[1] - a[1]) / 70];
      const n = leftOf(u);
      const house = coast.slots.find((s) => s.kind === 'house')!;
      const lanes = d0.roads.length - 2;
      let home: NeighbourhoodDraft['slots'][number] | null = null;
      for (const side of [1, -1]) {
        for (const facing of [0, 1, 2, 3] as const) {
          if (home) break;
          const r0 = plotRect(t, { ...house, facing });
          const off = 8 + Math.max(r0.w, r0.d) / 2;
          const c: Pt = [a[0] + u[0] * 62 + n[0] * side * off, a[1] + u[1] * 62 + n[1] * side * off];
          const s1 = { ...house, facing, x: Math.round(house.x + c[0] - (r0.x + r0.w / 2)), z: Math.round(house.z + c[1] - (r0.z + r0.d / 2)) };
          if (entryOf(d0, t, s1).road === lanes) home = { ...s1, index: d0.slots.length, name: 'Test House', area: site.area };
        }
      }
      if (!home) throw new Error('no home fits on the lane');
      made.set(k, { coast, k, d0, home });
      return made.get(k)!;
    };
  })();
  /** How far road `r` reaches past the last other road of `roads` meeting it, at its dead end (0: none). */
  const stubOf = (roads: readonly RoadPlan[], r: RoadPlan) => {
    const e = r.ends.indexOf('dead');
    if (e < 0) return 0;
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    let last = r.ends[1 - e] === 'join' ? 0 : -Infinity;
    for (const o of roads) {
      if (o === r) continue;
      for (const [x, z] of [o.points[0], o.points[o.points.length - 1]]) {
        const at = nearest(r.points, x, z, ls);
        if (at.dist <= Math.max(o.width, r.width) / 2 + 1) last = Math.max(last, e === 1 ? at.s : L - at.s);
      }
    }
    return L - Math.max(0, last);
  };

  test("a village removed: another village's street left a long dead end where its road met it is cut back to a court, or refused with homes past one", () => {
    const { coast, k, d0, home } = longLane(going(longLane().coast));
    const e = done(removeVillage(d0, t, k));
    const left = e.draft.roads.find((r) => r.name === 'Test Lane')!;
    expect(e.draft.roads.some((r) => r.name === 'Test Road')).toBe(false);
    expect(left.ends).toEqual(['join', 'dead']);
    expect(lengths(left.points).at(-1)!).toBeLessThanOrEqual(COURT + 0.5);
    expect(e.notes ?? []).toContain(`Test Lane is cut back to a dead end where ${coast.open!.areas[k].name} was`);
    expect(hardNew(e.draft, d0)).toEqual([]);
    // A home on the lane past a court's length: the removal is refused, with why.
    const refused = removeVillage({ ...d0, slots: [...d0.slots, home] }, t, k);
    expect('refused' in refused ? refused.refused : null).toBe(`without ${coast.open!.areas[k].name}, Test Lane would end 70 m past the last road it meets, with homes on it past a dead end's ${COURT} m`);
  }, 120_000);

  test("a village moved: another village's street left a long dead end where its old road met it is cut back to a court, or refused with homes past one", () => {
    // (The first village that moves, by a move its new site allows, the nearest first: one with homes
    // along a road only it leads to the town by may not move.)
    let found: { lane: ReturnType<typeof longLane>; shift: readonly [number, number]; e: Extract<Edited, { draft: unknown }> } | null = null;
    for (let k = 1; k < longLane().coast.open!.areas.length && !found; k++) {
      // (Without the homes built up along the country roads into it, its outskirts: they reach the town only through it,
      // so it may not move with them standing.)
      const raw = longLane(k);
      const spots = entriesOf(raw.d0, t);
      const d0 = removeLots(raw.d0, new Set(raw.d0.slots.filter((s, i) => (s.area ?? 0) === k && raw.d0.roads[spots[i].road]?.kind === 'country').map((s) => s.index)));
      const lane = { ...raw, d0, home: { ...raw.home, index: d0.slots.length } };
      for (const step of [6, 8, 10, 12, 14]) {
        for (const [ux, uz] of [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, 1], [1, -1], [-1, -1]]) {
          if (found) continue;
          const shift = [ux * step * 10, uz * step * 10] as const;
          if ('refused' in canMoveVillage(lane.d0, t, k, shift[0], shift[1])) continue;
          const e = moveVillage(lane.d0, t, k, shift[0], shift[1]);
          if (!('refused' in e)) found = { lane, shift, e };
        }
      }
    }
    if (!found) throw new Error('no village moves');
    const { lane: made, shift, e } = found;
    const { coast, k, d0, home } = made;
    const name = coast.open!.areas[k].name;
    // (Its old road taken up; the lane, unless its new road meets it again, cut back to a court.)
    expect(e.draft.roads.some((r) => r.name === 'Test Road')).toBe(false);
    const lane = e.draft.roads.find((r) => r.name === 'Test Lane')!;
    console.log(`moved ${name} by ${shift.join(', ')}: Test Lane ${lengths(lane.points).at(-1)!.toFixed(0)} m, ${lane.ends.join('/')}`);
    expect(stubOf(e.draft.roads, lane)).toBeLessThanOrEqual(COURT + 0.5);
    expect(lane.ends[1] === 'join' || (e.notes ?? []).includes(`Test Lane is cut back to a dead end where ${name} was`)).toBe(true);
    expect(hardNew(e.draft, d0).filter((h) => h.includes('Test Lane'))).toEqual([]);
    // A home on the lane past a court's length: the move is refused, with why (unless the new road meets the lane again).
    if (lane.ends[1] === 'dead') {
      const refused = moveVillage({ ...d0, slots: [...d0.slots, home] }, t, k, shift[0], shift[1]);
      expect('refused' in refused ? refused.refused : null).toBe(`with ${name} moved, Test Lane would end 70 m past the last road it meets, with homes on it past a dead end's ${COURT} m`);
    }
  }, 120_000);

  /** A village placed across a river `across` m wide carved straight into the land 180 m off the town's site: its bridge (`most` m of deck at most), and the water under it water again once it is removed. */
  const acrossRiver = (across: number, most: number) => {
    const options: Partial<WorldOptions> = { town: 'village', villages: 'none', size: 'large' };
    const land = inWorker({ kind: 'land', world: 'country', seed: 11, options }).draft;
    // A river across the land, its banks wavering as a painted one's do, 180 m off the town's site (carved straight into the land).
    const site = land.open!.plan!.site;
    const z0 = site[1] > land.depth / 2 ? site[1] - 180 : site[1] + 180;
    const was = land.open!.land!;
    const shaped = { ...was, h: was.h.slice(), wl: was.wl.slice(), cover: was.cover.slice(), edited: true };
    for (let j = 0; j < shaped.rows; j++) {
      for (let i = 0; i < shaped.cols; i++) {
        if (Math.abs(j * 2 - z0 + 3 * Math.sin(i / 4)) >= across / 2) continue;
        shaped.h[j * shaped.cols + i] = Math.min(shaped.h[j * shaped.cols + i], -150);
        shaped.wl[j * shaped.cols + i] = -40;
      }
    }
    const town = inWorker({ kind: 'town', land: { ...land, open: { ...land.open!, land: shaped } }, seed: 11, options, waterKept: false });
    const base = town.plan!;
    expect(base.open!.areas.length).toBe(1);
    expect(base.roads.some((r) => r.bridge)).toBe(false);
    // A site across the river from the town (the nearest the river first).
    const far = (z: number) => (z < z0) !== (site[1] < z0);
    let at: Pt | null = null;
    for (let d = 140; d < base.depth && !at; d += 20) {
      const z = site[1] < z0 ? z0 + d : z0 - d;
      if (z < 60 || z > base.depth - 60) break;
      for (let x = 100; x < base.width - 100 && !at; x += 20) if (far(z) && !('refused' in canPlaceVillage(base, t, { x, z, swap: false }))) at = [x, z];
    }
    expect(at).not.toBeNull();
    const e = done(placeVillage(base, t, content, { x: at![0], z: at![1], swap: false, seed: 3 }));
    const d = e.draft;
    expect(e.notes).toBeUndefined();
    const v = d.open!.areas.length - 1;
    const village = d.open!.areas[v].name;
    // A bridge to it, its deck walked dry.
    const bridge = d.roads.find((r) => r.bridge && r.bridge[1] > r.bridge[0]);
    expect(bridge?.name).toBe(`${village} Bridge`);
    expect(bridge!.bridge![1] - bridge!.bridge![0]).toBeGreaterThan(across);
    expect(bridge!.bridge![1] - bridge!.bridge![0]).toBeLessThanOrEqual(most);
    const W = d.width;
    const deck: number[] = [];
    deckTiles(bridge!, bridge!.bridge!, W, d.depth, (tile) => deck.push(tile));
    const wetLand = (tile: number) => wetTile(d.open!.land!, tile % W, Math.floor(tile / W));
    const under = deck.filter(wetLand);
    expect(under.length).toBeGreaterThan(50);
    const dry = unrle(d.open!.water, W * d.depth);
    expect(under.every((tile) => dry[tile] === 0)).toBe(true);
    // One network, the village on it; nothing new about bridges, decks or networks among the town's checks.
    expect(hardNew(d, base).filter((line) => /bridge|deck|network|no road/i.test(line))).toEqual([]);
    expect(blocking(d, base)).toEqual([]);
    // Removed: its bridge with it, the water under it water again.
    const gone = done(removeVillage(d, t, v)).draft;
    expect(gone.roads.some((r) => r.bridge)).toBe(false);
    const walk = unrle(gone.open!.water, W * gone.depth);
    expect(under.every((tile) => walk[tile] === 1)).toBe(true);
  };

  test('a village across a painted river gets a bridge; removed, the water under the bridge is water again', () => acrossRiver(30, 45), 180_000);

  test("a village across a 50 m painted river gets a bridge as a drawn street's (70 m of deck at most, not the generator's 45); removed, the water under it is water again", () => acrossRiver(50, 70), 180_000);

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
