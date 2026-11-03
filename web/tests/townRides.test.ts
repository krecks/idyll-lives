// The bikes and e-scooters a new town gives its households (`game/townRides.ts`, laid out by
// `layoutTown`): most households have some, about a bicycle a teen or grown-up (up to three, more
// often without a car), a kids' bike a child, an e-scooter in about one household in six; each on its
// own household's lot, outdoors, off the paths and the driveway (where the car parks), clear of the
// doorways, its front tile reached from the street and nothing else on the lot cut off by it.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('town rides', async () => {
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
  const { AssetRegistry } = await import('../src/assets/registry');
  const { DRIVEWAY_MIN_WIDTH, generateWorld, layoutTown, populate, useTownContent } = await import('../src/game/town');
  const { RIDES } = await import('../src/game/townRides');
  const { frontTile } = await import('../src/game/surfaces');
  type Templates = import('../src/game/town').Templates;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const rides = new Set<string>(Object.values(RIDES));
  const towns = (['medium', 'dense', 'farm'] as const).flatMap((kind) => (['cosy', 'normal'] as const).flatMap((size) => [4242, 77].map((seed) => ({ label: `${kind} ${size} ${seed}`, town: populate(content, assets, t, generateWorld(content, t, kind, seed, undefined, size)) }))));
  const laid = towns.map(({ label, town }) => ({ label, town, layout: layoutTown(content, t, town, null) }));
  const size = (o: { def: string; rot: number }): [number, number] => {
    const [fw, fd] = content.object(o.def)?.footprint ?? [1, 1];
    return o.rot % 2 ? [fd, fw] : [fw, fd];
  };
  const inside = (r: { x: number; z: number; w: number; d: number }, x: number, z: number) => x >= r.x - 1e-6 && z >= r.z - 1e-6 && x <= r.x + r.w + 1e-6 && z <= r.z + r.d + 1e-6;

  test('most households have some, about a bicycle a teen or grown-up, a kids’ bike a child', () => {
    let households = 0;
    let withRides = 0;
    let bikes = 0;
    let riders = 0;
    let scooters = 0;
    let kids = 0;
    let kidsBikes = 0;
    for (const { label, town, layout } of laid) {
      for (const { household, slot } of town.households) {
        const plot = layout.plots[slot];
        const mine = layout.objects.filter((o) => rides.has(o.def) && o.z < town.depth && inside(plot, o.x, o.z));
        households++;
        if (mine.length) withRides++;
        const stages = household.members.map((m) => content.stageOf(m.age)?.id);
        const n = stages.filter((s) => s && s !== 'baby' && s !== 'child').length;
        const own = mine.filter((o) => o.def === RIDES.bike).length;
        expect(own, `${label} plot ${slot}: at most three bicycles, one a rider`).toBeLessThanOrEqual(Math.min(3, n));
        bikes += own;
        riders += Math.min(3, n);
        scooters += mine.filter((o) => o.def === RIDES.scooter).length;
        kids += household.members.filter((m) => content.stageOf(m.age)?.id === 'child' && m.age >= 4).length;
        kidsBikes += mine.filter((o) => o.def === RIDES.kids).length;
      }
    }
    expect(households).toBeGreaterThan(40);
    expect(withRides / households).toBeGreaterThan(0.6);
    expect(bikes / riders).toBeGreaterThan(0.5);
    expect(kidsBikes / Math.max(1, kids)).toBeGreaterThan(0.7);
    expect(scooters / households).toBeGreaterThan(0.06);
    expect(scooters / households).toBeLessThan(0.3);
  });

  test('on their own lot, outdoors, off the paths and the driveway, clear of the doorways, reached from the street', () => {
    for (const { label, town, layout } of laid) {
      const D = town.depth;
      const ground = layout.objects.filter((o) => o.z < D);
      for (const [slot, plot] of layout.plots.entries()) {
        const mine = ground.filter((o) => rides.has(o.def) && inside(plot, o.x, o.z));
        if (!mine.length) continue;
        expect(town.households.some((h) => h.slot === slot), `${label} plot ${slot}: someone lives there`).toBe(true);
        // The plot's tiles: its house (the walls' box) and what stands on the ground block the way.
        const W = Math.round(plot.w);
        const H = Math.round(plot.d);
        const at = (x: number, z: number) => (z - plot.z) * W + (x - plot.x);
        const onPlot = (x: number, z: number) => x >= plot.x && z >= plot.z && x < plot.x + W && z < plot.z + H;
        const walls = layout.walls.filter(([x0, z0, x1, z1]) => Math.max(z0, z1) <= D && inside(plot, x0, z0) && inside(plot, x1, z1));
        const box = walls.length ? [Math.min(...walls.flatMap((w) => [w[0], w[2]])), Math.min(...walls.flatMap((w) => [w[1], w[3]])), Math.max(...walls.flatMap((w) => [w[0], w[2]])), Math.max(...walls.flatMap((w) => [w[1], w[3]]))] : null;
        const blocked = new Uint8Array(W * H);
        for (let z = plot.z; z < plot.z + H; z++) for (let x = plot.x; x < plot.x + W; x++) if (box && x >= box[0] && x < box[2] && z >= box[1] && z < box[3]) blocked[at(x, z)] = 1;
        for (const g of layout.garages.filter((g) => g.plot === slot)) for (let z = Math.floor(g.z0); z < Math.ceil(g.z1); z++) for (let x = Math.floor(g.x0); x < Math.ceil(g.x1); x++) if (onPlot(x, z)) blocked[at(x, z)] = 1;
        const things = ground.filter((o) => inside(plot, o.x, o.z) && (!content.object(o.def)?.layer || content.object(o.def)?.layer === 'floor'));
        for (const o of things) {
          const [w, d] = size(o);
          for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) if (onPlot(o.x + dx, o.z + dz)) blocked[at(o.x + dx, o.z + dz)] = 1;
        }
        const paths = layout.paths.filter((r) => r.x < plot.x + plot.w && r.x + r.w > plot.x && r.z < plot.z + plot.d && r.z + r.d > plot.z);
        const onPath = (x: number, z: number, wide = false) => paths.some((r) => (!wide || Math.min(r.w, r.d) > DRIVEWAY_MIN_WIDTH) && x < r.x + r.w && x + 1 > r.x && z < r.z + r.d && z + 1 > r.z);
        // From where the paths and the driveway come in off the street.
        const seen = new Uint8Array(W * H);
        const queue: [number, number][] = [];
        for (let z = plot.z; z < plot.z + H; z++) {
          for (let x = plot.x; x < plot.x + W; x++) {
            const edge = x === plot.x || z === plot.z || x === plot.x + W - 1 || z === plot.z + H - 1;
            if (edge && onPath(x, z) && !blocked[at(x, z)]) (seen[at(x, z)] = 1), queue.push([x, z]);
          }
        }
        for (let i = 0; i < queue.length; i++) {
          const [x, z] = queue[i];
          for (const [nx, nz] of [[x + 1, z], [x - 1, z], [x, z + 1], [x, z - 1]]) {
            if (!onPlot(nx, nz) || blocked[at(nx, nz)] || seen[at(nx, nz)]) continue;
            seen[at(nx, nz)] = 1;
            queue.push([nx, nz]);
          }
        }
        const doorsteps = layout.doors
          .filter((d) => d.z < D && inside(plot, d.x, d.z))
          .flatMap((d) => (d.axis === 'x' ? [[d.x, d.z - 1], [d.x, d.z]] : [[d.x - 1, d.z], [d.x, d.z]]));
        for (const o of mine) {
          const [w, d] = size(o);
          const where = `${label} plot ${slot}: ${o.def} at ${o.x},${o.z} rot ${o.rot}`;
          expect(onPlot(o.x, o.z) && onPlot(o.x + w - 1, o.z + d - 1), `${where} on its lot`).toBe(true);
          for (let dz = 0; dz < d; dz++) {
            for (let dx = 0; dx < w; dx++) {
              const [x, z] = [o.x + dx, o.z + dz];
              expect(box && x >= box[0] && x < box[2] && z >= box[1] && z < box[3], `${where} outdoors`).toBeFalsy();
              expect(onPath(x, z), `${where} off the paths and the driveway`).toBe(false);
              expect(doorsteps.some(([sx, sz]) => sx === x && sz === z), `${where} clear of the doorways`).toBe(false);
            }
          }
          const [fx, fz] = frontTile(o.x, o.z, w, d, o.rot);
          expect(onPlot(fx, fz) && seen[at(fx, fz)] === 1, `${where} reached from the street`).toBe(true);
          expect(onPath(fx, fz, true), `${where} used from off the driveway`).toBe(false);
        }
        // Every door out of the house still reached.
        for (const [x, z] of doorsteps) {
          if (!onPlot(x, z) || blocked[at(x, z)]) continue;
          expect(seen[at(x, z)], `${label} plot ${slot}: the doorstep at ${x},${z} reached`).toBe(1);
        }
      }
    }
  });
});
