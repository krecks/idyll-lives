/**
 * Generates sample houses with the shipped config and content (web/public/content/housegen.json)
 * and writes them as small town files, one house each, for the simulation to check
 * (`cargo test -p sim-core --test generated_houses -- --ignored`, reading HOUSES).
 *
 * Run: node --experimental-strip-types tools/housegen/sample.ts out.json [houses per lot size]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { register } from 'node:module';
import type { ObjectDef } from '../../web/src/content/content.ts';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';

// The game's modules import each other without extensions (Vite finds them); so does Node, with this.
register(
  'data:text/javascript,' +
    encodeURIComponent(`export async function resolve(spec, ctx, next) {
      try { return await next(spec, ctx); } catch (e) { if (/^\\.\\.?\\//.test(spec) && !/\\.[a-z]+$/.test(spec)) return next(spec + '.ts', ctx); throw e; }
    }`),
);
const { generateHouse, objectInfos } = await import('../../web/src/game/housegen.ts');

const root = new URL('../../web/public/content/', import.meta.url);
const cfg = JSON.parse(readFileSync(new URL('housegen.json', root), 'utf8')) as HouseGenConfig;
type ContentFile = { objects?: ObjectDef[]; include?: string[]; roomRules?: { lampTiles?: number } };
const baseFile = JSON.parse(readFileSync(new URL('base.json', root), 'utf8')) as ContentFile;
// Objects of base.json and the files it includes (the catalogue in furniture.json, the garden, packs).
const base = {
  objects: [baseFile, ...(baseFile.include ?? []).map((f) => JSON.parse(readFileSync(new URL(f, root), 'utf8')) as ContentFile)].flatMap((f) => f.objects ?? []),
};
const houses = JSON.parse(readFileSync(new URL('houses.json', root), 'utf8')) as { plot: { width: number; depth: number }; lotSizes: Record<string, { width: number; depth: number }> };
// The first definition of each object, with `Content.radius`'s default and limits.
const byId = new Map<string, ObjectDef>();
for (const o of base.objects) if (!byId.has(o.id)) byId.set(o.id, o);
const info = objectInfos({ object: (id) => byId.get(id), radius: (d) => Math.max(0.02, Math.min(0.45, d?.radius ?? 0.12)) });

const out = process.argv[2] ?? 'houses.json';
const perSize = Number(process.argv[3] ?? 60);
const towns: unknown[] = [];
const stats = { tried: 0, made: 0, two: 0, value: 0, seats: 0, dining: 0, tops: 0, rooms: 0 };
// Per room kind: how many, their floor in all, the biggest (tiles).
const kinds = new Map<string, { n: number; area: number; most: number; long: number }>();
// A farm lot's house is a large lot's (town.ts `houseLot`).
for (const [size, lot] of Object.entries(houses.lotSizes).filter(([size]) => size !== 'farm')) {
  for (let seed = 1; seed <= perSize; seed++) {
    stats.tried++;
    const h = generateHouse(cfg, info, lot, houses.plot, size, seed * 7919, baseFile.roomRules?.lampTiles);
    if (!h) continue;
    stats.made++;
    if (h.upper) stats.two++;
    const all = [...h.objects, ...(h.upper?.objects ?? [])];
    stats.value += all.reduce((n, o) => n + (byId.get(o.def)?.price ?? 0), 0);
    const seats = all.filter((o) => byId.get(o.def)?.seatAt).length;
    stats.seats += seats;
    if (seats) stats.dining++;
    stats.tops += all.filter((o) => byId.get(o.def)?.layer === 'surface').length;
    for (const r of [...(h.rooms ?? []), ...(h.upper?.rooms ?? [])]) {
      const k = kinds.get(r.kind) ?? { n: 0, area: 0, most: 0, long: 0 };
      kinds.set(r.kind, { n: k.n + 1, area: k.area + r.w * r.d, most: Math.max(k.most, r.w * r.d), long: Math.max(k.long, r.w, r.d) });
      stats.rooms++;
    }
    // Back on the lot (the layouts' frame is a medium lot's).
    const dx = Math.floor((lot.width - houses.plot.width) / 2);
    const depth = lot.depth;
    const moveDoor = (lift: number) => (d: { x: number; z: number; axis: string }) => ({ x: d.x + dx, z: d.z + lift, axis: d.axis });
    const walls = [
      ...h.walls.map(([a, b, c, d]) => [a + dx, b, c + dx, d]),
      ...(h.upper?.walls ?? []).map(([a, b, c, d]) => [a + dx, b + depth, c + dx, d + depth]),
    ];
    towns.push({
      size,
      seed: seed * 7919,
      name: h.name,
      description: h.description,
      // The rooms as planned, on the lot (the storey above `depth` rows down, like the town).
      rooms: [...(h.rooms ?? []).map((r) => ({ ...r, x: r.x + dx })), ...(h.upper?.rooms ?? []).map((r) => ({ ...r, x: r.x + dx, z: r.z + depth }))],
      town: {
        width: lot.width,
        depth,
        storeys: h.upper ? 2 : 1,
        walls,
        doors: [...h.doors.map(moveDoor(0)), ...(h.upper?.doors ?? []).map(moveDoor(depth))],
        windows: [...(h.windows ?? []).map(moveDoor(0)), ...(h.upper?.windows ?? []).map(moveDoor(depth))],
        objects: [...h.objects.map((o) => ({ ...o, x: o.x + dx })), ...(h.upper?.objects ?? []).map((o) => ({ ...o, x: o.x + dx, z: o.z + depth }))],
        plots: [{ name: 'Home', x: 0, z: 0, w: lot.width, d: depth, entry: h.spawns[0] && [h.spawns[0][0] + dx, h.spawns[0][1]] }],
        households: [{ name: 'Test', plot: 0, player: true }],
        sims: [],
      },
    });
  }
}
writeFileSync(out, JSON.stringify(towns));
console.log(`${stats.made} of ${stats.tried} houses made (${stats.two} with two storeys) → ${out}`);
const per = (n: number) => (n / Math.max(1, stats.made)).toFixed(1);
console.log(`  per house: $${per(stats.value)} of things, ${per(stats.seats)} chairs at tables (${stats.dining} houses with some), ${per(stats.tops)} things on tops`);
console.log(`  ${per(stats.rooms)} rooms per house; by kind (count, mean and largest floor, longest side):`);
for (const [k, v] of [...kinds].sort((a, b) => b[1].n - a[1].n)) console.log(`    ${k}: ${v.n}, ${(v.area / v.n).toFixed(1)} m², up to ${v.most} m², ${v.long} m long`);
for (const t of towns.slice(0, 8) as { size: string; name: string; description: string }[]) console.log(`  ${t.size}: ${t.name} — ${t.description}`);
