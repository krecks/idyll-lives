// Step 2's proposed layout and step 3's Build (`game/proposal.ts`, `game/build.ts`, the town worker's `layout`
// and `build` jobs): the layout's roads are the town job's and it has no lots, the town built from an untouched
// layout is the town job's town (and its households); a layout painted again (a block erased, a block made shops,
// a street drawn again with homes along it, the villages and zones cleared and a small town painted) builds the same town
// twice and no hard problem its layout hadn't, the rest of the town standing as it was; an untouched layout built
// again from another seed is about as dense as the generator's town, kind by kind (a table printed); a cluster
// painted far from every town along a street of its own is a village; the brush never paints water or a road; and
// step 2's summary is within a quarter of what Build makes (exactly it for an untouched layout; an edit that leaves the zones be moves it by what it changes). `SLOW_TESTS=1 bun test tests/buildFromUses.slow.test.ts`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('build from uses', async () => {
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
  const { AssetRegistry } = await import('../src/assets/registry');
  const { objectInfos } = await import('../src/game/housegen');
  const { plotRect, populate, useTownContent } = await import('../src/game/town');
  const { runTownJob } = await import('../src/game/townJobs');
  const { editedProblems } = await import('../src/game/edit/checked');
  const { entriesOf } = await import('../src/game/edit/ground');
  const { lotKey } = await import('../src/game/edit/lots');
  const { drawStreet, removeRoad } = await import('../src/game/edit/streets');
  const { removeArea } = await import('../src/game/edit/villages');
  const { noUseCells, paintUses } = await import('../src/game/edit/uses');
  const { layOutUses, PROPOSAL_SEED } = await import('../src/game/build');
  const { townSummary, usesSummary } = await import('../src/game/proposal');
  const { placeOfTemplate } = await import('../src/game/placeLots');
  const { unrle } = await import('../src/game/landMasks');
  const { mulberry32 } = await import('../src/game/noise');
  const { leftOf, lengths, pointAt, RoadIndex } = await import('../src/game/roads');
  const { unpackUses, USE_CELL, usesEmpty, layoutHash } = await import('../src/game/uses');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type TownJob = import('../src/game/townJobs').TownJob;
  type TownResult = import('../src/game/townJobs').TownResult;
  type Proposal = import('../src/game/townJobs').Proposal;
  type UseKind = import('../src/game/uses').UseKind;
  type UseStamp = import('../src/game/edit/uses').UseStamp;
  type Pt = import('../src/game/roads').Pt;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A job as the worker runs it: sent, run, and its result sent back. */
  const inWorker = <K extends TownJob['kind']>(job: Extract<TownJob, { kind: K }>) => structuredClone(runTownJob(content, t, structuredClone(job))) as Extract<TownResult, { kind: K }>;
  /** A draft as bytes: its typed arrays (the land) written out. */
  const bytes = (v: unknown) => JSON.stringify(v, (_, x) => (ArrayBuffer.isView(x) ? Array.from(x as Int16Array).join(',') : x));
  const done = (e: { draft: NeighbourhoodDraft } | { refused: string }): NeighbourhoodDraft => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e.draft;
  };
  const build = (proposal: Proposal, plan: NeighbourhoodDraft, seed: number) => inWorker({ kind: 'build', plan, proposal, seed });
  const hardOf = (d: NeighbourhoodDraft) => editedProblems(d, t, content).hard;
  /** Stamps of a 32 m brush along road `r` of `d` on `side`, its edge at the kerb. */
  const along = (d: NeighbourhoodDraft, r: number, side: 1 | -1): UseStamp[] => {
    const road = d.roads[r];
    const ls = lengths(road.points);
    const out: UseStamp[] = [];
    for (let s = 0; s <= ls[ls.length - 1]; s += 6) {
      const { p, t: tt } = pointAt(road.points, s, ls);
      const n = leftOf(tt).map((v) => v * side);
      out.push({ x: p[0] + n[0] * (road.width / 2 + 16), z: p[1] + n[1] * (road.width / 2 + 16), size: 32 });
    }
    return out;
  };
  /** What a town has, as step 2's summary counts it: homes, shops and workplaces, parks and outdoor spots, civic places. */
  const counts = (d: NeighbourhoodDraft) => townSummary(d);

  const cases = [
    { world: 'country', seed: 7, options: { town: 'town', villages: 'few' } },
    { world: 'farm', seed: 11, options: { town: 'town', villages: 'few' } },
    { world: 'seaside', seed: 3, options: { town: 'village', villages: 'few', hills: 'hilly' } },
  ] as const;
  /** Each case's land, its `town` and `layout` jobs (made once). */
  const made = new Map<string, { town: Extract<TownResult, { kind: 'town' }>; layout: Extract<TownResult, { kind: 'layout' }> }>();
  const caseOf = (c: { world: string; seed: number; options: Partial<import('../src/core/protocol').WorldOptions> }) => {
    const key = `${c.world} ${c.seed}`;
    let m = made.get(key);
    if (!m) {
      const land = inWorker({ kind: 'land', world: c.world, seed: c.seed, options: c.options });
      const job = { land: land.draft, seed: c.seed, options: c.options, waterKept: true };
      m = { town: inWorker({ kind: 'town', ...job }), layout: inWorker({ kind: 'layout', ...job }) };
      made.set(key, m);
    }
    return m;
  };
  /** The main town's streets and lanes, longest first. */
  const streetsOf = (d: NeighbourhoodDraft) =>
    d.roads
      .map((r, i) => ({ r, i, L: lengths(r.points)[r.points.length - 1] }))
      .filter(({ r }) => r.area === 0 && (r.kind === 'street' || r.kind === 'lane') && !r.bridge)
      .sort((a, b) => b.L - a.L);

  for (const c of cases) {
    const name = `${c.world} ${c.seed}`;

    test(`${name}: the layout is the town job's roads with no lots; built untouched, it is the town job's town`, () => {
      const { town, layout } = caseOf(c);
      const { proposal } = layout;
      expect(proposal.plan.roads).toEqual(town.draft.roads);
      expect(proposal.plan.open!.areas).toEqual(town.draft.open!.areas);
      expect(proposal.plan.open!.centre).toEqual(town.draft.open!.centre);
      expect(proposal.plan.greens).toEqual(town.draft.greens);
      expect(proposal.plan.slots).toEqual([]);
      expect(proposal.plan.open!.places).toEqual([]);
      expect(proposal.plan.open!.parking).toEqual([]);
      expect(proposal.hash).toBe(layoutHash(proposal.plan));
      // (What step 2 shows: the layout's streets settled, no lots; its plan the layout.)
      expect(layout.draft.slots).toEqual([]);
      expect(layout.draft.roads).toEqual(town.draft.roads);
      expect(layout.plan).toEqual(proposal.plan);
      expect(layout.houses).toEqual([]);
      // The uses: painted where the town built something, none under its roads or water.
      const cells = unpackUses(proposal.plan.open!.uses!);
      const none = noUseCells(proposal.plan);
      expect(cells.some((k) => k > 0)).toBe(true);
      for (let k = 0; k < cells.length; k++) if (none[k]) expect(cells[k]).toBe(0);
      // Built untouched (the proposal's seed): the town job's town, its plan and marks, and its households.
      const built = build(proposal, proposal.plan, PROPOSAL_SEED);
      expect(built.draft).toEqual(town.draft);
      expect(built.plan).toEqual(town.plan);
      expect(built.marks).toEqual(town.marks);
      expect(built.houses.map(([id]) => id).sort()).toEqual(town.houses.map(([id]) => id).sort());
      expect(populate(content, assets, t, built.draft, undefined, mulberry32(5)).households).toEqual(populate(content, assets, t, town.draft, undefined, mulberry32(5)).households);
    }, 60_000);

    test(`${name}: painted again, it builds the same town twice, the rest standing, and no hard problem its layout hadn't`, () => {
      const { town, layout } = caseOf(c);
      const { proposal } = layout;
      const plan = proposal.plan;
      const a0 = plan.open!.areas[0];
      const mid: Pt = [a0.x + a0.w / 2, a0.z + a0.d / 2];
      const before = new Set(town.draft.slots.map(lotKey));
      const scripts: { what: string; plan: NeighbourhoodDraft; seed: number; check?: (d: NeighbourhoodDraft) => void }[] = [];
      // A block erased: nothing built where it was, most of the town standing as it was.
      const erased = done(paintUses(plan, [{ x: mid[0], z: mid[1], size: 64 }], null));
      scripts.push({
        what: 'a block erased',
        plan: erased,
        seed: PROPOSAL_SEED,
        check: (d) => {
          const near = d.slots.filter((s) => {
            const r = plotRect(t, s);
            return Math.hypot(r.x + r.w / 2 - mid[0], r.z + r.d / 2 - mid[1]) < 24;
          });
          expect(near.filter((s) => !before.has(lotKey(s)))).toEqual([]);
          expect(d.slots.filter((s) => before.has(lotKey(s))).length).toBeGreaterThan(town.draft.slots.length * 0.8);
        },
      });
      // A block made shops and work.
      scripts.push({ what: 'a block made shops', plan: done(paintUses(plan, [{ x: mid[0], z: mid[1], size: 64 }], 'shops')), seed: PROPOSAL_SEED });
      // A street taken up and drawn again, homes painted along it.
      let drawn: { plan: NeighbourhoodDraft; road: number } | null = null;
      for (const { r, i } of streetsOf(plan)) {
        const gone = removeRoad(plan, t, i);
        if (!('draft' in gone)) continue;
        const points = r.ends[0] === 'dead' ? [...r.points].reverse() : r.points;
        const e = drawStreet(gone.draft, t, content, { points, kind: 'street', clear: true, fill: false, seed: 6 });
        if (!('draft' in e)) continue;
        const road = e.draft.roads.findIndex((q) => !gone.draft.roads.includes(q));
        let d = e.draft;
        for (const side of [1, -1] as const) d = done(paintUses(d, along(d, road, side), 'homes'));
        drawn = { plan: d, road };
        break;
      }
      expect(drawn).not.toBeNull();
      scripts.push({
        what: 'a street drawn again with homes',
        plan: drawn!.plan,
        seed: PROPOSAL_SEED,
        check: (d) => {
          const road = d.roads.findIndex((r) => r.name === drawn!.plan.roads[drawn!.road].name);
          expect(entriesOf(d, t).filter((e, k) => e.road === road && d.slots[k].kind === 'house').length).toBeGreaterThan(0);
        },
      });
      // Cleared (the villages gone, the zones cleared), a small town painted along the longest streets, built from another seed.
      let small: NeighbourhoodDraft = { ...plan, open: { ...plan.open!, uses: usesEmpty(plan.open!.uses!.cols, plan.open!.uses!.rows) } };
      // (As Clear all keeps the main town's area alone: each village's streets and area gone.)
      for (let a = plan.open!.areas.length - 1; a >= 1; a--) small = removeArea({ ...small, roads: small.roads.filter((r) => r.area !== a) }, a);
      expect(small.open!.areas.length).toBe(1);
      const kinds: UseKind[] = ['homes', 'shops', 'homes', 'green', 'public', 'rows'];
      streetsOf(plan)
        .slice(0, 3)
        .forEach(({ i }, k) => {
          for (const side of [1, -1] as const) {
            const e = paintUses(small, along(small, i, side), kinds[k * 2 + (side > 0 ? 0 : 1)]);
            if ('draft' in e) small = e.draft;
          }
        });
      scripts.push({ what: 'a small town', plan: small, seed: 5, check: (d) => expect(counts(d).homes).toBeGreaterThan(5) });

      for (const s of scripts) {
        const had = new Set(hardOf(s.plan));
        const a = build(proposal, s.plan, s.seed);
        const b = build(proposal, s.plan, s.seed);
        expect(bytes(b.draft)).toBe(bytes(a.draft));
        // (Built from its layout: no lots left over a slope limit to give up but those of the town.)
        expect({ what: s.what, hard: hardOf(a.draft).filter((h) => !had.has(h)) }).toEqual({ what: s.what, hard: [] });
        expect(a.draft.open!.uses).toBeUndefined();
        expect(a.draft.roads.filter((r) => r.kind !== 'trail').map((r) => r.name)).toEqual(s.plan.roads.filter((r) => r.kind !== 'trail').map((r) => r.name));
        s.check?.(a.draft);
      }
    }, 180_000);
  }

  test("step 2's summary is within a quarter of what Build makes (a few either way)", () => {
    const close = (what: string, est: number, got: number, share = 0.25) => expect({ what, est, ok: Math.abs(est - got) <= Math.max(3, share * got) }).toEqual({ what, est, ok: true });
    const { layout } = caseOf(cases[0]);
    const { proposal } = layout;
    let small: NeighbourhoodDraft = { ...proposal.plan, open: { ...proposal.plan.open!, uses: usesEmpty(proposal.plan.open!.uses!.cols, proposal.plan.open!.uses!.rows) } };
    const kinds: UseKind[] = ['homes', 'shops', 'rows', 'green', 'public', 'homes', 'shops', 'homes'];
    streetsOf(proposal.plan)
      .slice(0, 4)
      .forEach(({ i }, k) => {
        for (const side of [1, -1] as const) {
          const e = paintUses(small, along(small, i, side), kinds[(k * 2 + (side > 0 ? 0 : 1)) % kinds.length]);
          if ('draft' in e) small = e.draft;
        }
      });
    const plans = [
      { what: `${cases[0].world} proposal`, proposal, plan: proposal.plan },
      // (Painted by hand, its shops read high: 47 for 38 built with 7 m lanes, 47 for 37 with 6 m, so within 30 %;
      // over 24 proposals of eight places the estimate holds, 835 shops for 817 built, 2026-11-23.)
      { what: `${cases[0].world} painted`, proposal, plan: small, share: 0.3 },
      { what: `${cases[1].world} proposal`, proposal: caseOf(cases[1]).layout.proposal, plan: caseOf(cases[1]).layout.proposal.plan },
    ];
    // (Untouched, given its proposal: what Build gives it, the proposal's town, exactly.)
    expect(usesSummary(proposal.plan, proposal)).toEqual(townSummary(build(proposal, proposal.plan, PROPOSAL_SEED).draft));
    // (Its zones cleared, given its proposal: the estimate as it is, nothing of the proposal's town kept.)
    const bare: NeighbourhoodDraft = { ...proposal.plan, open: { ...proposal.plan.open!, uses: usesEmpty(proposal.plan.open!.uses!.cols, proposal.plan.open!.uses!.rows) } };
    expect(usesSummary(bare, proposal)).toEqual(usesSummary(bare));
    for (const p of plans) {
      // (Everything zoned built again: another seed.)
      const got = counts(layOutUses(p.proposal, p.plan, t, content, 9));
      const est = usesSummary(p.plan);
      for (const k of ['homes', 'shops', 'parks', 'public'] as const) close(`${p.what} ${k}`, est[k], got[k], 'share' in p ? p.share : undefined);
    }
  }, 120_000);

  test("a street drawn out of the town into the open, the zones left be: step 2's summary moves by what it changes, about what Build then makes", () => {
    const { layout } = caseOf(cases[0]);
    const { proposal } = layout;
    const plan = proposal.plan;
    const exact = townSummary(proposal.built);
    const [W, D] = [plan.width, plan.depth];
    // A dead end 120 m long off a street of the town, square to it, into land no road is within 60 m of.
    const index = new RoadIndex(plan.roads);
    let edited: NeighbourhoodDraft | null = null;
    const why = new Map<string, number>();
    for (const { r, i } of streetsOf(plan)) {
      const ls = lengths(r.points);
      for (let s = 20; s < ls[ls.length - 1] - 20 && !edited; s += 10) {
        const { p, t: tt } = pointAt(r.points, s, ls);
        for (const side of [1, -1] as const) {
          const n = leftOf(tt).map((v) => v * side) as Pt;
          const end: Pt = [p[0] + n[0] * 126, p[1] + n[1] * 126];
          if (end[0] < 20 || end[1] < 20 || end[0] > W - 20 || end[1] > D - 20) continue;
          if ([40, 80, 120].some((d) => index.nearest(p[0] + n[0] * d, p[1] + n[1] * d, 30))) continue;
          const e = drawStreet(plan, t, content, { points: [[p[0] + n[0] * 6, p[1] + n[1] * 6], end], kind: 'street', clear: true, fill: false, seed: 3 });
          const k = 'refused' in e ? e.refused : `${e.draft.roads.length - plan.roads.length} roads`;
          why.set(k, (why.get(k) ?? 0) + 1);
          if ('draft' in e && e.draft.roads.length === plan.roads.length + 1 && e.draft.roads[i] === plan.roads[i]) {
            edited = e.draft;
            break;
          }
        }
      }
      if (edited) break;
    }
    if (!edited) throw new Error(`no dead end could be drawn out of the town: ${JSON.stringify([...why])}`);
    const raw = usesSummary(edited);
    const est = usesSummary(edited, proposal);
    const got = counts(build(proposal, edited, PROPOSAL_SEED).draft);
    console.log(`untouched ${JSON.stringify(exact)}; a dead end drawn: estimated ${JSON.stringify(raw)} on its own, ${JSON.stringify(est)} kept to the proposal's town; built ${JSON.stringify(got)}`);
    for (const k of ['homes', 'shops', 'parks', 'public'] as const) {
      // (Its zones as they were: the proposal's town but for what the street adds, and what Build makes of it.)
      expect({ k, est: est[k], ok: Math.abs(est[k] - exact[k]) <= Math.max(2, 0.05 * exact[k]) }).toEqual({ k, est: est[k], ok: true });
      expect({ k, est: est[k], got: got[k], ok: Math.abs(est[k] - got[k]) <= Math.max(3, 0.1 * got[k]) }).toEqual({ k, est: est[k], got: got[k], ok: true });
    }
  }, 120_000);

  test("built again from another seed, an untouched layout is about as dense as the generator's town, kind by kind", () => {
    // (The worlds above and a few more: a river town, a mountain town, a city.)
    const worlds = [...cases, { world: 'river', seed: 5, options: { town: 'town', villages: 'few' } }, { world: 'mountain', seed: 8, options: { town: 'town', villages: 'few', hills: 'mountains' } }, { world: 'country', seed: 21, options: { town: 'city', villages: 'few' } }] as const;
    const kinds = ['homes', 'terraces', 'shops', 'buildings', 'parks', 'public'] as const;
    /** What a town has of each kind: house lots (not terraced, not farms), terraced lots, shops and workplaces (their lots), the buildings no home's lot is (the named places' lots and the buildings of flats), parks and outdoor spots, civic places. */
    const density = (d: NeighbourhoodDraft): Record<(typeof kinds)[number], number> => ({
      ...counts(d),
      homes: d.slots.filter((s) => s.kind === 'house' && s.lot !== 'farm' && s.lot !== 'terrace').length,
      terraces: d.slots.filter((s) => s.lot === 'terrace').length,
      buildings: d.slots.filter((s) => s.kind === 'flats' || placeOfTemplate(s.template)).length,
    });
    const seeds = [9, 23];
    const total = Object.fromEntries(kinds.map((k) => [k, [0, 0]])) as Record<(typeof kinds)[number], [number, number]>;
    const lines = [['world', ...kinds].map((h, i) => (i ? h.padStart(18) : h.padEnd(24))).join('')];
    for (const c of worlds) {
      const { town, layout } = caseOf(c);
      const gen = density(town.draft);
      const built = seeds.map((seed) => density(layOutUses(layout.proposal, layout.proposal.plan, t, content, seed)));
      const cells = kinds.map((k) => {
        const got = built.reduce((n, b) => n + b[k], 0) / built.length;
        total[k][0] += gen[k];
        total[k][1] += got;
        return `${gen[k]} → ${got.toFixed(0)}${gen[k] ? ` (${Math.round((100 * got) / gen[k])}%)` : ''}`.padStart(18);
      });
      lines.push([`${c.world} ${c.seed} ${c.options.town}`.padEnd(24), ...cells].join(''));
    }
    lines.push(['all'.padEnd(24), ...kinds.map((k) => `${total[k][0]} → ${total[k][1].toFixed(0)} (${Math.round((100 * total[k][1]) / total[k][0])}%)`.padStart(18))].join(''));
    console.log(`Generator's town → built again (mean of seeds ${seeds.join(', ')}):\n${lines.join('\n')}`);
    // Over all the worlds, each kind within a sixth of the generator's (shops and parks, fewer and the most scattered, within a quarter).
    const within = { homes: 0.15, terraces: 0.15, shops: 0.25, buildings: 0.15, parks: 0.25, public: 0.15 };
    for (const k of kinds) expect({ k, ok: Math.abs(total[k][1] / total[k][0] - 1) <= within[k] }).toEqual({ k, ok: true });
  }, 300_000);

  test('a cluster painted along a street of no area, far from every town, is a village of its own', () => {
    const { layout } = caseOf(cases[0]);
    const { proposal } = layout;
    const plan = proposal.plan;
    const areas = plan.open!.areas;
    const middle = (a: (typeof areas)[number]): Pt => [a.x + a.w / 2, a.z + a.d / 2];
    let painted: NeighbourhoodDraft | null = null;
    // A street out from a country road, well away from the towns (drawn as a street of no area, as step 2 draws one).
    for (const r of plan.roads.map((_, i) => i).filter((i) => plan.roads[i].kind === 'country' && !plan.roads[i].bridge)) {
      const road = plan.roads[r];
      const ls = lengths(road.points);
      for (let s = 60; s < ls[ls.length - 1] - 60 && !painted; s += 40) {
        const { p, t: tt } = pointAt(road.points, s, ls);
        for (const side of [1, -1] as const) {
          const n = leftOf(tt).map((v) => v * side);
          const c: Pt = [p[0] + n[0] * 70, p[1] + n[1] * 70];
          if (areas.some((a) => Math.hypot(middle(a)[0] - c[0], middle(a)[1] - c[1]) < 200)) continue;
          const e = drawStreet(plan, t, content, { points: [p, [p[0] + n[0] * 140, p[1] + n[1] * 140]], kind: 'street', clear: false, fill: false, seed: 4 });
          if (!('draft' in e)) continue;
          const at = e.draft.roads.length - 1;
          let d: NeighbourhoodDraft = { ...e.draft, roads: e.draft.roads.map((q, i) => (i === at ? { ...q, kind: 'street' as const, width: 6, area: undefined } : q)) };
          for (const sd of [1, -1] as const) d = done(paintUses(d, along(d, at, sd), 'homes'));
          painted = d;
          break;
        }
      }
      if (painted) break;
    }
    expect(painted).not.toBeNull();
    const built = build(proposal, painted!, PROPOSAL_SEED).draft;
    const fresh = built.open!.areas.length - 1;
    expect(built.open!.areas.length).toBe(areas.length + 1);
    expect(areas.map((a) => a.name)).not.toContain(built.open!.areas[fresh].name);
    expect(built.slots.filter((s) => s.area === fresh).length).toBeGreaterThanOrEqual(4);
    expect(built.roads.find((r) => r.name === painted!.roads[painted!.roads.length - 1].name)?.area).toBe(fresh);
    expect(hardOf(built).filter((h) => !new Set(hardOf(painted!)).has(h))).toEqual([]);
  }, 60_000);

  test('the brush never paints water or a road on a place', () => {
    const { layout } = caseOf(cases[2]);
    const plan = layout.proposal.plan;
    const stamps: UseStamp[] = [];
    for (let z = 0; z <= plan.depth; z += 40) for (let x = 0; x <= plan.width; x += 40) stamps.push({ x, z, size: 64 });
    const layer = done(paintUses(plan, stamps, 'green')).open!.uses!;
    const cells = unpackUses(layer);
    const water = unrle(plan.open!.water, plan.width * plan.depth);
    const roads = new RoadIndex(plan.roads.filter((r) => r.kind !== 'trail'));
    let wet = 0;
    for (let k = 0; k < cells.length; k++) {
      const [x, z] = [((k % layer.cols) + 0.5) * USE_CELL, (Math.floor(k / layer.cols) + 0.5) * USE_CELL];
      const on = water[Math.floor(z) * plan.width + Math.floor(x)] === 1;
      if (on) wet++;
      if (!cells[k]) continue;
      expect(on).toBe(false);
      expect(roads.onRoad(x, z)).toBe(false);
    }
    expect(wet).toBeGreaterThan(0);
  });
});
