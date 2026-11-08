// The garden fences, hedges and field fences a town puts up (`game/gardenFences.ts`), from the shipped
// content and house layouts: on tile edges of the ground, never on a wall, across a path or driveway or
// in a doorway; the same whichever lot the player takes; tidy (one fence on a shared edge, its corners
// met); laid out the same on lots turned a quarter; and written into the town file as runs.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('garden fences', async () => {
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
  const { generateWorld, layoutTown, useTownContent, vacantSlots } = await import('../src/game/town');
  const { fenceRuns, townFences } = await import('../src/game/gardenFences');
  type Templates = import('../src/game/town').Templates;
  type TownFence = import('../src/game/gardenFences').TownFence;

  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const key = (f: { axis: string; x: number; z: number }) => `${f.axis}:${f.x}:${f.z}`;
  const towns = (['medium', 'dense', 'farm'] as const).flatMap((kind) => (['cosy', 'normal'] as const).map((size) => ({ kind, size, town: generateWorld(content, t, kind, 4242, undefined, size) })));

  test('on tile edges of the ground, never on a wall, across a path or driveway, or in a doorway', () => {
    for (const { kind, size, town } of towns) {
      const layout = layoutTown(content, t, town, null);
      const label = `${kind} ${size}`;
      expect(layout.fences.length, label).toBeGreaterThan(20);
      const walls = new Set<string>();
      for (const [x0, z0, x1, z1] of layout.walls) {
        if (Math.max(z0, z1) > town.depth) continue;
        if (z0 === z1) for (let x = Math.min(x0, x1); x < Math.max(x0, x1); x++) walls.add(`h:${x}:${z0}`);
        else for (let z = Math.min(z0, z1); z < Math.max(z0, z1); z++) walls.add(`v:${x0}:${z}`);
      }
      const doors = new Set(layout.doors.filter((d) => d.z < town.depth).map((d) => `${d.axis === 'x' ? 'h' : 'v'}:${d.x}:${d.z}`));
      const seen = new Set<string>();
      for (const f of layout.fences) {
        expect([f.x, f.z].every(Number.isInteger), `${label}: ${key(f)} on the grid`).toBe(true);
        expect(f.x >= 0 && f.z >= 0 && f.x <= town.width && f.z <= town.depth, `${label}: ${key(f)} in town`).toBe(true);
        expect(walls.has(key(f)), `${label}: ${key(f)} on a wall`).toBe(false);
        expect(doors.has(key(f)), `${label}: ${key(f)} in a doorway`).toBe(false);
        expect(seen.has(key(f)), `${label}: ${key(f)} twice`).toBe(false);
        seen.add(key(f));
        // A path or driveway crossing it: on both sides of it.
        const crossed = layout.paths.find((r) =>
          f.axis === 'h' ? r.x < f.x + 1 && r.x + r.w > f.x && r.z < f.z - 0.01 && r.z + r.d > f.z + 0.01 : r.z < f.z + 1 && r.z + r.d > f.z && r.x < f.x - 0.01 && r.x + r.w > f.x + 0.01,
        );
        expect(crossed, `${label}: ${key(f)} across a path`).toBeUndefined();
      }
      // Hedges and pickets round gardens, rail fences round farms and their fields (gates in them).
      const styles = new Set(layout.fences.map((f) => f.style));
      expect([...styles].every((s) => content.fenceStyles.some((c) => c.id === s)), label).toBe(true);
      expect(layout.fences.some((f) => f.kind === 'gate'), label).toBe(true);
      if (kind === 'farm') expect(styles.has('rails'), label).toBe(true);
      else expect(styles.has('picket') || styles.has('hedge'), label).toBe(true);
    }
  });

  test('the same whichever lot the player takes', () => {
    for (const { kind, size, town } of towns) {
      const base = layoutTown(content, t, town, null).fences.map(key).sort();
      for (const slot of vacantSlots(town).slice(0, 2)) expect(layoutTown(content, t, town, slot.index).fences.map(key).sort(), `${kind} ${size} with the player at ${slot.index}`).toEqual(base);
    }
  });

  test('tidy: a garden closed behind its house, every corner met, on lots facing any way', () => {
    // Two gardens side by side on a street along x (street at z = 0: facing 0), a picket fence and a
    // hedge (by their places), houses 8 × 9 with a driveway on the right of the left one; and the same
    // turned a quarter (the town's (x, z) at (-z, x), as the street lays such a plot out).
    const plotsOf = (turn: boolean) => {
      const r = (x: number, z: number, w: number, d: number) => (turn ? { x: -(z + d), z: x, w: d, d: w } : { x, z, w, d });
      const box = (x0: number, z0: number, x1: number, z1: number): [number, number, number, number] => {
        const q = r(x0, z0, x1 - x0, z1 - z0);
        return [q.x, q.z, q.x + q.w, q.z + q.d];
      };
      // Turned, the street (z = 0) is east of the lots (x = 0): facing 3.
      return [
        { rect: r(2, 0, 18, 24), facing: (turn ? 3 : 0) as 0 | 3, house: box(5, 5, 13, 14), home: true, drive: r(13.6, 0, 3, 10.4), fenced: true },
        { rect: r(20, 0, 16, 24), facing: (turn ? 3 : 0) as 0 | 3, house: box(24, 6, 32, 15), home: true, drive: null, fenced: true },
      ];
    };
    const plan = (turn: boolean) => townFences({ plots: plotsOf(turn), fields: [], walls: new Set(), blockers: [], paths: [] });
    const flat = plan(false);
    expect(flat.length).toBeGreaterThan(40);
    // Turned back: an `h` edge at (x, z) of the turned town is the `v` edge at (z, -x - 1) of the flat one.
    const back = (f: TownFence) => (f.axis === 'h' ? `v:${f.z}:${-f.x - 1}` : `h:${f.z}:${-f.x}`);
    expect(new Set(flat.map((f) => f.style))).toEqual(new Set(['picket', 'hedge']));
    expect(plan(true).map(back).sort()).toEqual(flat.map(key).sort());
    // Every corner meets another fence or the house; at least one gate into each garden.
    const houseLines = (x: number, z: number) => plotsOf(false).some(({ house: [x0, z0, x1, z1] }) => ((x === x0 || x === x1) && z >= z0 && z <= z1) || ((z === z0 || z === z1) && x >= x0 && x <= x1));
    const ends = new Map<string, number>();
    for (const f of flat) for (const c of f.axis === 'h' ? [`${f.x}:${f.z}`, `${f.x + 1}:${f.z}`] : [`${f.x}:${f.z}`, `${f.x}:${f.z + 1}`]) ends.set(c, (ends.get(c) ?? 0) + 1);
    const loose = [...ends].filter(([c, n]) => n < 2 && !houseLines(...(c.split(':').map(Number) as [number, number])));
    expect(loose).toEqual([]);
    // Behind the left house's driveway (10.4 m back): its cross fence at 12 m, the shared side from there.
    expect(flat.some((f) => f.axis === 'v' && f.x === 20 && f.z === 11)).toBe(false);
    expect(flat.some((f) => f.axis === 'v' && f.x === 20 && f.z === 12)).toBe(true);
    expect(flat.some((f) => f.axis === 'h' && f.z === 12 && f.x >= 13 && f.x < 20)).toBe(true);
    // One fence on the shared side and the back, a gate in each cross fence (two each).
    expect(flat.filter((f) => f.kind === 'gate').length).toBe(4);
    // A thing just behind a gate (a rain barrel, a statue: nothing residents use, which the fence may
    // stand beside): the gate goes beside it, in the same fence, to be walked through.
    const gates = flat.filter((f) => f.kind === 'gate');
    // (The tile at a gate's (x, z) is the one behind it, the garden's, on a street along x.)
    const behind = gates.map((g) => ({ x: g.x, z: g.z, w: 1, d: 1 }));
    expect(gates.every((g) => g.axis === 'h')).toBe(true);
    const moved = townFences({ plots: plotsOf(false), fields: [], walls: new Set(), blockers: behind, paths: [] }).filter((f) => f.kind === 'gate');
    expect(moved.length).toBe(gates.length);
    const taken = (x: number, z: number) => behind.some((b) => b.x === x && b.z === z);
    for (const g of moved) {
      expect(gates.some((o) => key(o) === key(g)), `${key(g)} where it was`).toBe(false);
      expect(gates.some((o) => o.axis === g.axis && o.z === g.z && Math.abs(o.x - g.x) === 1), `${key(g)} beside where it was`).toBe(true);
      expect(taken(g.x, g.z) || taken(g.x, g.z - 1), `${key(g)} by a thing`).toBe(false);
    }
  });

  test('no land shut in between gardens, in a big world whose lots leave gaps between them', () => {
    // (A big small town: its lots kept off junctions and bends left 90 tiles between four gardens.)
    const town = generateWorld(content, t, 'small', 314192, undefined, 'big');
    const layout = layoutTown(content, t, town, null);
    const W = town.width;
    const D = town.depth;
    const fence = new Set(layout.fences.filter((f) => f.kind === 'fence').map(key));
    const onPlot = new Uint8Array(W * D);
    for (const p of layout.plots) {
      for (let z = Math.max(0, Math.floor(p.z)); z < Math.min(D, Math.ceil(p.z + p.d)); z++) for (let x = Math.max(0, Math.floor(p.x)); x < Math.min(W, Math.ceil(p.x + p.w)); x++) onPlot[z * W + x] = 1;
    }
    // Land on no plot reached from the world's corner with the fences standing (as the sim walks).
    const seen = new Uint8Array(W * D);
    const stack = [0];
    seen[0] = 1;
    while (stack.length) {
      const i = stack.pop()!;
      const [x, z] = [i % W, Math.floor(i / W)];
      for (const [nx, nz, edge] of [[x + 1, z, `v:${x + 1}:${z}`], [x - 1, z, `v:${x}:${z}`], [x, z + 1, `h:${x}:${z + 1}`], [x, z - 1, `h:${x}:${z}`]] as const) {
        const j = nz * W + nx;
        if (nx < 0 || nz < 0 || nx >= W || nz >= D || seen[j] || fence.has(edge)) continue;
        seen[j] = 1;
        stack.push(j);
      }
    }
    const shut: string[] = [];
    for (let i = 0; i < W * D; i++) if (!seen[i] && !onPlot[i]) shut.push(`${i % W},${Math.floor(i / W)}`);
    expect(shut.slice(0, 5), `${shut.length} tiles on no plot shut in`).toEqual([]);
  }, 120_000);

  test('written into the town file as runs', () => {
    const layout = layoutTown(content, t, towns[0].town, null);
    const runs = fenceRuns(layout.fences);
    expect(runs.length).toBeLessThan(layout.fences.length);
    const edges = runs.flatMap(({ line: [x0, z0, x1, z1], style, gate }) => {
      const out: string[] = [];
      if (z0 === z1) for (let x = x0; x < x1; x++) out.push(`h:${x}:${z0}:${style}:${gate ? 'gate' : 'fence'}`);
      else for (let z = z0; z < z1; z++) out.push(`v:${x0}:${z}:${style}:${gate ? 'gate' : 'fence'}`);
      return out;
    });
    expect(edges.sort()).toEqual(layout.fences.map((f) => `${key(f)}:${f.style}:${f.kind}`).sort());
  });
});
