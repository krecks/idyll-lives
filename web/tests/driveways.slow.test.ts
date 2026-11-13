// Every car a new town parks on a driveway can drive out of it (`game/town.ts`: `LotPlacer`
// puts a house only where its driveway's line meets a road squarely, `layoutTown` lays it on the
// side that does, across the verge to that road): the driveway's line meets the road the town file
// says it does (`drives[].road`, `s`) squarely enough for the simulation to turn out onto it
// (sim-core `cars::OBLIQUE`: not coming alongside it, on a bend the road takes away from the lot or
// by a junction), and its paving goes on from where the car parks to that road's, with no grass
// between (across a trail on the way, which a car's way on doesn't count). From the sample worlds
// the generated-world checks failed on (`tools/worlds/sample.ts`, made here the same way).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;
/** sim-core `cars::OBLIQUE`: the least cosine of a driveway's line to its road's side. */
const OBLIQUE = 0.5;

type Pt = [number, number];
type Rect = { x: number; z: number; w: number; d: number };
type Road = { points: Pt[]; width: number; kind?: string };
type TownFile = {
  roads: Road[];
  drives: { plot: number; x: number; z: number; front: number; frontX?: number; road?: number; s?: number }[];
  cars: { plot: number; x: number; z: number }[];
  meta: { paths: Rect[] };
};

/** The way a polyline's segment at arc length `s` runs (as sim-core `Road::tangent`). */
function wayAt(pts: readonly Pt[], s: number): Pt {
  let [i, at] = [1, 0];
  for (; i < pts.length - 1; i++) {
    const l = Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]);
    if (at + l > s) break;
    at += l;
  }
  const [a, b] = [pts[i - 1], pts[i]];
  const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1;
  return [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
}

/** How far (x, z) is from a polyline. */
function off(pts: readonly Pt[], [x, z]: Pt): number {
  let best = Infinity;
  for (let i = 1; i < pts.length; i++) {
    const [[ax, az], [bx, bz]] = [pts[i - 1], pts[i]];
    const [dx, dz] = [bx - ax, bz - az];
    const f = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz || 1)));
    best = Math.min(best, Math.hypot(x - ax - f * dx, z - az - f * dz));
  }
  return best;
}

/** What's wrong with the way out of each car's driveway in a town file (none: an empty list). */
function wayOutProblems(file: TownFile): string[] {
  const problems: string[] = [];
  const paved = (q: Pt) =>
    file.meta.paths.some((r) => q[0] >= r.x - 0.05 && q[0] <= r.x + r.w + 0.05 && q[1] >= r.z - 0.05 && q[1] <= r.z + r.d + 0.05) ||
    file.roads.some((r) => r.kind !== 'trail' && off(r.points, q) <= r.width / 2);
  for (const car of file.cars) {
    const drive = file.drives.find((d) => d.plot === car.plot);
    if (drive?.road === undefined || drive.s === undefined) {
      problems.push(`plot ${car.plot}: its driveway meets no road`);
      continue;
    }
    const road = file.roads[drive.road];
    const way: Pt = drive.frontX ? [drive.frontX, 0] : [0, drive.front];
    const t = wayAt(road.points, drive.s);
    const square = Math.abs(t[0] * way[1] - t[1] * way[0]);
    if (square < OBLIQUE) problems.push(`plot ${car.plot}: its driveway comes alongside road ${drive.road} (${square.toFixed(2)} of square)`);
    // From where it parks along the driveway's line onto the road's paving, all of it paved.
    for (let k = 0; ; k += 0.25) {
      const q: Pt = [car.x + way[0] * k, car.z + way[1] * k];
      if (off(road.points, q) <= road.width / 2) break;
      if (k > 40) {
        problems.push(`plot ${car.plot}: its driveway's line doesn't get to road ${drive.road}`);
        break;
      }
      if (!paved(q)) {
        problems.push(`plot ${car.plot}: grass at ${q[0].toFixed(1)},${q[1].toFixed(1)} on its way to road ${drive.road}`);
        break;
      }
    }
  }
  return problems;
}

describe('driveways', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  const realRandom = Math.random;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
    Math.random = realRandom;
  });
  const { Content } = await import('../src/content/content');
  const { objectInfos } = await import('../src/game/housegen');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { assembleTown, generateWorld, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { mulberry32 } = await import('../src/game/noise');
  type Templates = import('../src/game/town').Templates;
  type WorldKind = import('../src/game/world').WorldKind;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A world's town file as `tools/worlds/sample.ts` makes it (its households seeded by its seed, the player in the first empty house). */
  const townFile = (kind: WorldKind, seed: number): TownFile => {
    Math.random = mulberry32(seed);
    const town = populate(content, assets, t, generateWorld(content, t, kind, seed));
    const home = vacantSlots(town)[0];
    return JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), home.index)) as TownFile;
  };

  // Seed base 14's worlds whose cars had no way out: their driveways came alongside the road
  // (Brackenshaw's by a loop bending away from the lot's corner, Kingfisher Lake's by a junction).
  const noWayOut: [WorldKind, number][] = [
    ['forest', 425059],
    ['lakeside', 215603],
    ['forest', 320330],
    ['dense', 320329],
    ['seaside', 215602],
  ];
  for (const [kind, seed] of noWayOut) {
    test(
      `${kind} ${seed}: every car can turn out of its driveway onto its road`,
      () => {
        const file = townFile(kind, seed);
        expect(file.cars.length).toBeGreaterThan(0);
        expect(wayOutProblems(file)).toEqual([]);
      },
      30_000,
    );
  }

  // Seed base 13's Snowdale: the driveway of plot 112 stopped where its line crossed a trail, 16 m
  // short of the lane its car drives onto, and the car drove over the grass between. Snowdale's
  // hamlets have other houses since the living town's terraces (2026-11-12), and whichever plot
  // has such a driveway moves as the layout does: the first of these worlds with a car whose
  // driveway's line crosses a trail on the way to its road has it paved across.
  const acrossTrail: [WorldKind, number][] = [
    ['mountain', 241142],
    ['mountain', 207684],
    ['mountain', 128494],
  ];
  test('a driveway across a trail is paved all the way to its road', () => {
    const crossing = (file: TownFile) =>
      file.cars.find((car) => {
        const drive = file.drives.find((d) => d.plot === car.plot);
        if (drive?.road === undefined) return false;
        const road = file.roads[drive.road];
        const way: Pt = drive.frontX ? [drive.frontX, 0] : [0, drive.front];
        for (let k = 0; k < 40; k += 0.25) {
          const q: Pt = [car.x + way[0] * k, car.z + way[1] * k];
          if (off(road.points, q) <= road.width / 2) return false;
          if (file.roads.some((r) => r.kind === 'trail' && off(r.points, q) <= r.width / 2)) return true;
        }
        return false;
      });
    for (const [kind, seed] of acrossTrail) {
      const file = townFile(kind, seed);
      if (!crossing(file)) continue;
      expect(wayOutProblems(file)).toEqual([]);
      return;
    }
    throw new Error(`no driveway across a trail in ${acrossTrail.map(([k, s]) => `${k} ${s}`).join(', ')}: find another world that has one`);
  }, 120_000);
});
