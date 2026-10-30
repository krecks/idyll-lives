/**
 * Generates sample houses with the shipped config and content (web/public/content/housegen.json)
 * and writes them as small town files, one house each, for the simulation to check
 * (`cargo test -p sim-core --test generated_houses -- --ignored`, reading HOUSES).
 *
 * Each house is furnished for one of a few households in turn (empty, a couple, two friends, a
 * family) and checked here against them all: the same layout for each, beds enough, no toys
 * without children, no bathroom off the room the front door opens into, the front door into a
 * shared room, no way through a bathroom, a laundry or a bedroom, and a lot cut to fit the house
 * (the garden sizes in turn) that it covers no more and no less of than `COVER` says. Each house
 * is priced as the game prices a home (`price.ts`, furnished for nobody): the prices by house
 * class and garden size are listed, and the bigger the class, the dearer. Problems are listed
 * (and the exit code is 1). The houses are written turned to face a street on each
 * side in turn, as towns turn their lots.
 *
 * Run: bun tools/housegen/sample.ts out.json [houses per lot size]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import type { Economy, ObjectDef } from '../../web/src/content/content.ts';
import type { GardenSize, HouseGenConfig, Residents } from '../../web/src/game/housegen.ts';
import type { Facing, HouseTemplate } from '../../web/src/game/town.ts';

const { GARDEN_SIZES, generateHouse, objectInfos } = await import('../../web/src/game/housegen.ts');
const { turner } = await import('../../web/src/game/town.ts');
const { priceOf } = await import('../../web/src/game/price.ts');

/** How much of its lot a house covers (its ground floor), by garden size: least and most. */
const COVER: Record<GardenSize, [number, number]> = { snug: [0.2, 0.6], roomy: [0.16, 0.45], big: [0.12, 0.32] };

const root = new URL('../../web/public/content/', import.meta.url);
const cfg = JSON.parse(readFileSync(new URL('housegen.json', root), 'utf8')) as HouseGenConfig;
type ContentFile = { objects?: ObjectDef[]; include?: string[]; roomRules?: { lampTiles?: number; windowTiles?: number }; economy?: Partial<Economy> };
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
const households: [string, Residents | undefined][] = [
  ['empty', undefined],
  ['couple', { couples: 1, adults: 0, children: 0, babies: 0 }],
  ['friends', { couples: 0, adults: 2, children: 0, babies: 0 }],
  ['family', { couples: 1, adults: 0, children: 2, babies: 0 }],
];
const problems: string[] = [];
const kidsOnly = cfg.residents?.kidsOnly;
const forKids = (def: string) => {
  const o = byId.get(def);
  return !!o && !!(kidsOnly?.categories?.includes(o.category ?? '') || kidsOnly?.groups?.includes(o.group ?? ''));
};
const bedIds = new Set(Object.values(cfg.residents?.beds ?? {}).flat());
const isBed = (def: string) => byId.get(def)?.group === 'beds' || bedIds.has(def);
/** The layout alone (walls, doors, windows, rooms' places), to compare houses furnished for different households. */
const layout = (h: HouseTemplate) =>
  JSON.stringify([h.walls, h.doors, h.windows, h.upper?.walls, h.upper?.doors, h.basement?.walls, h.basement?.doors, ...[h.rooms, h.upper?.rooms, h.basement?.rooms].map((rs) => rs?.map((r) => [r.x, r.z, r.w, r.d]))]);
/** What's wrong with house `h` (furnished for `who`). */
function check(h: HouseTemplate, who: Residents | undefined): string[] {
  const out: string[] = [];
  const all = [...h.objects, ...(h.upper?.objects ?? []), ...(h.basement?.objects ?? [])];
  const kids = !!who && who.children + who.babies > 0;
  if (!kids && all.some((o) => forKids(o.def))) out.push(`things for children: ${[...new Set(all.filter((o) => forKids(o.def)).map((o) => o.def))].join(', ')}`);
  if (who) {
    const beds = all.filter((o) => isBed(o.def));
    const doubles = beds.filter((o) => (byId.get(o.def)?.slots ?? 1) >= 2).length;
    // Places to sleep (a bunk bed has two); short of bedrooms two share one, so two a room at most.
    const places = beds.reduce((n, o) => n + (byId.get(o.def)?.slots ?? 1), 0);
    const need = 2 * who.couples + who.adults + who.children;
    if (places < Math.min(need, 2 * h.bedrooms)) out.push(`${places} places to sleep for ${need}`);
    if (who.couples && doubles < who.couples) out.push('no bed for two for the couple');
  }
  // The front door: on the street wall (z = the house's front), the first door.
  const front = h.doors[0];
  const rooms = h.rooms ?? [];
  const into = rooms.find((r) => front && r.x <= front.x && front.x < r.x + r.w && r.z === front.z);
  if (!into || !['entry', 'living', 'kitchen', 'dining'].includes(into.kind)) out.push(`front door into ${into?.kind ?? 'nothing'}`);
  if (rooms.some((r) => r.kind === 'entry') && into?.kind !== 'entry') out.push('an entrance hall the front door doesn\'t open into');
  const touches = (a: { x: number; z: number; w: number; d: number }, d: { x: number; z: number; axis: string }) =>
    d.axis === 'x' ? (d.z === a.z || d.z === a.z + a.d) && d.x >= a.x && d.x < a.x + a.w : (d.x === a.x || d.x === a.x + a.w) && d.z >= a.z && d.z < a.z + a.d;
  for (const bath of rooms.filter((r) => r.kind === 'bathroom')) {
    if (into && h.doors.slice(1).some((d) => touches(bath, d) && touches(into, d))) out.push(`bathroom off the ${into.kind} the front door opens into`);
  }
  // Nobody walks through a bathroom or a laundry, nor through a bedroom (but into its own
  // bathroom): one door to anything else each, besides the bedroom's to its bathrooms.
  for (const r of rooms.filter((r) => ['bathroom', 'laundry', 'bedroom'].includes(r.kind))) {
    const ways = h.doors.filter((d) => touches(r, d)).filter((d) => !rooms.some((o) => o !== r && o.kind === 'bathroom' && r.kind === 'bedroom' && touches(o, d)));
    if (ways.length > 1) out.push(`a way through the ${r.kind} at ${r.x},${r.z}`);
  }
  return out;
}
const stats = { tried: 0, made: 0, two: 0, basements: 0, value: 0, seats: 0, dining: 0, tops: 0, rooms: 0, backs: 0, gardenDoors: 0 };
/** Per house size and garden size: how much of the lot houses cover (each house's share). */
const covers = new Map<string, number[]>();
// Per room kind: how many, their floor in all, the biggest (tiles).
const kinds = new Map<string, { n: number; area: number; most: number; long: number }>();
/** The house classes, smallest first (`housegen.json` `classes`, by the label houses carry as `class`). */
const CLASSES = Object.values(cfg.classes).map((c) => c.label);
/** Per house class, and per class and garden size: the houses' prices (as `price.ts` prices a home on its lot). */
const prices = new Map<string, number[]>();
const objectPrice = (def: string) => byId.get(def)?.price ?? 0;
// A farm lot's house is a large lot's (town.ts `houseLot`).
for (const [size, lot] of Object.entries(houses.lotSizes).filter(([size]) => size !== 'farm')) {
  for (let seed = 1; seed <= perSize; seed++) {
    stats.tried++;
    const garden = GARDEN_SIZES[seed % GARDEN_SIZES.length];
    const name = `${size} ${seed * 7919} ${garden}`;
    const variants = households.map(([name, who]) => [name, who, generateHouse(cfg, info, lot, houses.plot, size, seed * 7919, baseFile.roomRules?.lampTiles, baseFile.roomRules?.windowTiles, who, garden)] as const);
    if (variants.some(([, , v]) => !v !== !variants[0][2])) problems.push(`${name}: made for some households, not others`);
    for (const [household, who, v] of variants) {
      if (!v) continue;
      if (layout(v) !== layout(variants[0][2]!)) problems.push(`${name} (${household}): another layout`);
      if (JSON.stringify(v.lot) !== JSON.stringify(variants[0][2]!.lot)) problems.push(`${name} (${household}): another lot`);
      for (const p of check(v, who)) problems.push(`${name} (${household}): ${p}`);
    }
    const [household, , h] = variants[seed % variants.length];
    if (!h) continue;
    stats.made++;
    // A garden door: in the back wall, else a side wall (none where the stairs and a room's only window take it).
    const ground = h.rooms ?? [];
    const [x0, x1, z0, z1] = [Math.min(...ground.map((r) => r.x)), Math.max(...ground.map((r) => r.x + r.w)), Math.min(...ground.map((r) => r.z)), Math.max(...ground.map((r) => r.z + r.d))];
    // The lot cut to fit it: inside it, and covered no more and no less than its garden size says.
    if (!h.lot) problems.push(`${name}: no lot cut to fit the house`);
    else {
      const cover = ((x1 - x0) * (z1 - z0)) / (h.lot.width * h.lot.depth);
      const [least, most] = COVER[garden];
      if (x0 < 1 || x1 > h.lot.width - 1 || z1 > h.lot.depth - 4) problems.push(`${name}: the house (${x0}..${x1} × ..${z1}) off its ${h.lot.width}×${h.lot.depth} lot`);
      if (cover < least || cover > most) problems.push(`${name}: covers ${(cover * 100).toFixed(0)}% of its ${h.lot.width}×${h.lot.depth} lot`);
      const k = `${size} ${garden}`;
      covers.set(k, [...(covers.get(k) ?? []), cover]);
    }
    const outer = h.doors.slice(1).filter((d) => (d.axis === 'x' ? d.z === z1 : d.x === x0 || d.x === x1) && d.z >= z0);
    if (outer.length) stats.gardenDoors++;
    if (outer.some((d) => d.axis === 'x')) stats.backs++;
    // Priced as it stands for sale: furnished for nobody in particular.
    const cls = h.class;
    if (!cls) problems.push(`${name}: no house class`);
    else {
      const [bare, rates] = [variants[0][2]!, baseFile.economy?.homePrice];
      const { width, depth } = bare.lot ?? lot;
      const { total } = priceOf(objectPrice, rates, bare, width, depth);
      for (const k of [cls, `${cls} ${garden}`]) prices.set(k, [...(prices.get(k) ?? []), total]);
      // The same lot with the house cleared away.
      prices.set(`empty ${garden}`, [...(prices.get(`empty ${garden}`) ?? []), priceOf(objectPrice, rates, undefined, width, depth).total]);
    }
    if (h.upper) stats.two++;
    if (h.basement) stats.basements++;
    const all = [...h.objects, ...(h.upper?.objects ?? []), ...(h.basement?.objects ?? [])];
    stats.value += all.reduce((n, o) => n + (byId.get(o.def)?.price ?? 0), 0);
    const seats = all.filter((o) => byId.get(o.def)?.seatAt).length;
    stats.seats += seats;
    if (seats) stats.dining++;
    stats.tops += all.filter((o) => byId.get(o.def)?.layer === 'surface').length;
    for (const r of [...(h.rooms ?? []), ...(h.upper?.rooms ?? []), ...(h.basement?.rooms ?? [])]) {
      const k = kinds.get(r.kind) ?? { n: 0, area: 0, most: 0, long: 0 };
      kinds.set(r.kind, { n: k.n + 1, area: k.area + r.w * r.d, most: Math.max(k.most, r.w * r.d), long: Math.max(k.long, r.w, r.d) });
      stats.rooms++;
    }
    // On its own lot, turned to face a street on any side (as `town.ts` turns lots onto a town).
    const facing = (seed % 4) as Facing;
    const { width, depth } = h.lot ?? lot;
    const turn = turner(0, 0, width, depth, facing);
    const [W, D] = facing % 2 ? [depth, width] : [width, depth];
    const footprint = (def: string): [number, number] => byId.get(def)?.footprint ?? [1, 1];
    // The storey above `D` rows down, the basement on the rows after every storey's (as `town.ts` puts them).
    const storeys = h.upper ? 2 : 1;
    const floors = [[h, 0], [h.upper, D], [h.basement, storeys * D]] as const;
    const onTown = <T>(f: (floor: NonNullable<(typeof floors)[number][0]>, lift: number) => T[]) => floors.flatMap(([floor, lift]) => (floor ? f(floor, lift) : []));
    towns.push({
      size,
      garden,
      facing,
      seed: seed * 7919,
      household,
      name: h.name,
      description: h.description,
      // The rooms as planned, on the town (each storey on its rows, like the town).
      rooms: onTown((f, lift) =>
        (f.rooms ?? []).map((r) => {
          const at = turn.rect(r);
          return { ...r, ...at, z: at.z + lift };
        }),
      ),
      town: {
        width: W,
        depth: D,
        storeys,
        ...(h.basement ? { basement: true } : {}),
        walls: onTown((f, lift) => f.walls.map((w) => turn.wall(w, lift))),
        doors: onTown((f, lift) => f.doors.map((d) => turn.edge(d, lift))),
        windows: onTown((f, lift) => (f.windows ?? []).map((d) => turn.edge(d, lift))),
        objects: onTown((f, lift) => f.objects.map((o) => turn.place(o, footprint, lift))),
        plots: [{ name: 'Home', x: 0, z: 0, w: W, d: D, entry: h.spawns[0] && turn.point(h.spawns[0]) }],
        households: [{ name: 'Test', plot: 0, player: true }],
        sims: [],
      },
    });
  }
}
writeFileSync(out, JSON.stringify(towns));
console.log(`${stats.made} of ${stats.tried} houses made (${stats.two} with two storeys, ${stats.basements} with a basement) → ${out}`);
const per = (n: number) => (n / Math.max(1, stats.made)).toFixed(1);
console.log(`  per house: $${per(stats.value)} of things, ${per(stats.seats)} chairs at tables (${stats.dining} houses with some), ${per(stats.tops)} things on tops`);
console.log(`  ${stats.gardenDoors} houses with a garden door (${stats.backs} in the back wall, the others in a side wall)`);
console.log(`  ${per(stats.rooms)} rooms per house; by kind (count, mean and largest floor, longest side):`);
for (const [k, v] of [...kinds].sort((a, b) => b[1].n - a[1].n)) console.log(`    ${k}: ${v.n}, ${(v.area / v.n).toFixed(1)} m², up to ${v.most} m², ${v.long} m long`);
console.log('  share of the lot a house covers (least, mean, most), by house and garden size:');
for (const [k, v] of [...covers].sort()) console.log(`    ${k}: ${[Math.min(...v), v.reduce((a, b) => a + b, 0) / v.length, Math.max(...v)].map((c) => `${(c * 100).toFixed(0)}%`).join(', ')}`);
const mean = (v: number[]) => v.reduce((a, b) => a + b, 0) / v.length;
const priced = (v: number[]) => `$${Math.min(...v).toLocaleString()} / $${Math.round(mean(v)).toLocaleString()} / $${Math.max(...v).toLocaleString()} (${v.length})`;
console.log('  prices (least / mean / most, and how many), by house class and garden size:');
for (const c of CLASSES) {
  if (!prices.has(c)) continue;
  console.log(`    ${c}: ${priced(prices.get(c)!)}`);
  for (const g of GARDEN_SIZES) if (prices.has(`${c} ${g}`)) console.log(`      ${g}: ${priced(prices.get(`${c} ${g}`)!)}`);
}
console.log(`  an empty lot (a house's lot, the house cleared away): ${GARDEN_SIZES.filter((g) => prices.has(`empty ${g}`)).map((g) => `${g} ${priced(prices.get(`empty ${g}`)!)}`).join(', ')}`);
// The bigger the class, the dearer: a bungalow and a town house are much the same (wider and lower, or narrower and taller), so they may swap a little.
const means = CLASSES.flatMap((c) => (prices.has(c) ? [[c, mean(prices.get(c)!)] as const] : []));
for (const [i, [a, pa]] of means.entries()) {
  for (const [b, pb] of means.slice(i + 1)) {
    const near = a === 'bungalow' && b === 'town house';
    if (near ? pb < pa * 0.85 : pb <= pa) problems.push(`a ${b} costs $${Math.round(pb).toLocaleString()} on average, a ${a} $${Math.round(pa).toLocaleString()}: the bigger, the dearer`);
  }
}
for (const t of towns.slice(0, 8) as { size: string; name: string; description: string }[]) console.log(`  ${t.size}: ${t.name} — ${t.description}`);
if (problems.length) {
  console.log(`${problems.length} problems:`);
  for (const p of problems.slice(0, 60)) console.log(`  ${p}`);
  process.exitCode = 1;
}
