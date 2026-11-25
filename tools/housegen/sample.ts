/**
 * Generates sample houses with the shipped config and content (web/public/content/housegen.json)
 * and writes them as small town files, one house each, for the simulation to check
 * (`cargo test -p sim-core --test sim generated_houses:: -- --ignored`, reading HOUSES).
 *
 * Each house is furnished for one of a few households in turn (empty, a couple, two friends, a
 * family, a couple and a teenager) and checked here against them all: the same layout for each,
 * beds enough, no toys without children nor any no child there may use, a child's bed every
 * child there may sleep in, no bathroom off the room the front door opens into, the front door into a
 * shared room, no way through a bathroom, a laundry or a bedroom, and a lot cut to fit the house
 * (the garden sizes in turn) that it covers no more and no less of than `COVER` says. Each house
 * is priced as the game prices a home (`price.ts`, furnished for nobody): the prices by house
 * class and garden size are listed, and the bigger the class, the dearer. Problems are listed
 * (and the exit code is 1). The houses are written turned to face a street on each
 * side in turn, as towns turn their lots.
 *
 * Then buildings of flats (`flats.ts`), the flats over places and places with real storeys above
 * them, each a town of its own the same way (`kind` `flats` or `place`), each flat a plot of its own
 * with a household in it (or a shell): checked here as homes are, each flat's layout the same
 * whoever lives there, and priced (flats the cheapest homes); a building of flats made in 15 ms
 * or less (the median). The simulation checks each flat is reached from the street door, has a
 * toilet in a bathroom and beds for its household. A problem too when fewer than 90 % of the houses
 * tried are made.
 *
 * `--no-timing`: the flats' time not checked against its 15 ms (a run beside others, on a slow machine).
 *
 * Run: bun tools/housegen/sample.ts out.json [houses per lot size] [buildings of flats] [--no-timing]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import type { GardenSize, HouseGenConfig, Residents } from '../../web/src/game/housegen.ts';
import type { Door, Facing, HouseTemplate, Placement, PlannedRoom } from '../../web/src/game/town.ts';

const { GARDEN_SIZES, generateHouse, objectInfos, residentsKey, ROW_GAP } = await import('../../web/src/game/housegen.ts');
const { turner } = await import('../../web/src/game/town.ts');
const { priceOf } = await import('../../web/src/game/price.ts');

/** How much of its lot a house covers (its ground floor), by garden size: least and most. */
const COVER: Record<GardenSize, [number, number]> = { snug: [0.2, 0.6], roomy: [0.16, 0.45], big: [0.12, 0.32] };
/** A house in a row (a terrace's, `row` in its class) covers more of its narrow lot, whatever its garden. */
const ROW_COVER: [number, number] = [0.3, 0.6];

const root = new URL('../../web/public/content/', import.meta.url);
const cfg = JSON.parse(readFileSync(new URL('housegen.json', root), 'utf8')) as HouseGenConfig;
// The content as the game loads it (base.json and the files it includes, merged by `mergeContent`).
Object.assign(globalThis, { location: { href: root.href, search: '' } });
const { Content } = await import('../../web/src/content/content.ts');
const content = await Content.load(new URL('base.json', root).href);
const houses = JSON.parse(readFileSync(new URL('houses.json', root), 'utf8')) as { plot: { width: number; depth: number }; lotSizes: Record<string, { width: number; depth: number }> };
const info = objectInfos(content);

/** `--no-timing`: the flats' time not checked against its budget (`tools/ci-local.sh generated`). */
const timing = !process.argv.includes('--no-timing');
if (!timing) process.argv.splice(process.argv.indexOf('--no-timing'), 1);
const out = process.argv[2] ?? 'houses.json';
const perSize = Number(process.argv[3] ?? 60);
const towns: unknown[] = [];
const households: [string, Residents | undefined][] = [
  ['empty', undefined],
  ['couple', { couples: 1, adults: 0, children: 0, babies: 0 }],
  ['friends', { couples: 0, adults: 2, children: 0, babies: 0 }],
  ['family', { couples: 1, adults: 0, children: 2, babies: 0, stages: ['child', 'child'] }],
  ['teenager', { couples: 1, adults: 0, children: 1, babies: 0, stages: ['teen'] }],
];
const problems: string[] = [];
const kidsOnly = cfg.residents?.kidsOnly;
const forKids = (def: string) => {
  const o = content.object(def);
  return !!o && !!(kidsOnly?.categories?.includes(o.category ?? '') || kidsOnly?.groups?.includes(o.group ?? ''));
};
const bedIds = new Set(Object.values(cfg.residents?.beds ?? {}).flat());
const isBed = (def: string) => content.object(def)?.group === 'beds' || bedIds.has(def);
/** The layout alone (walls, doors, windows, rooms' places), to compare houses furnished for different households. */
const layout = (h: HouseTemplate) =>
  JSON.stringify([h.walls, h.doors, h.windows, h.upper?.walls, h.upper?.doors, h.basement?.walls, h.basement?.doors, ...[h.rooms, h.upper?.rooms, h.basement?.rooms].map((rs) => rs?.map((r) => [r.x, r.z, r.w, r.d]))]);
/** What's wrong with house `h` (furnished for `who`). */
function check(h: HouseTemplate, who: Residents | undefined): string[] {
  const all = [...h.objects, ...(h.upper?.objects ?? []), ...(h.basement?.objects ?? [])];
  return [...furnishingProblems(all, h.bedrooms, who), ...planProblems(h.rooms ?? [], h.doors, (into) => !!h.doors[0] && into.z === h.doors[0].z)];
}
/** What's wrong with a home's things (`all`, its `bedrooms`) for `who` living there. */
function furnishingProblems(all: Placement[], bedrooms: number, who: Residents | undefined): string[] {
  const out: string[] = [];
  const kids = !!who && who.children + who.babies > 0;
  if (!kids && all.some((o) => forKids(o.def))) out.push(`things for children: ${[...new Set(all.filter((o) => forKids(o.def)).map((o) => o.def))].join(', ')}`);
  // Nothing for children no child here may use (the race car bed and the toy box for a teenager).
  const stages = who?.stages ?? [];
  const fits = (def: string, stage: string) => info(def)?.usedBy?.includes(stage) ?? true;
  const unfit = (def: string) => forKids(def) && !who?.babies && stages.length > 0 && !stages.some((s) => fits(def, s));
  if (all.some((o) => unfit(o.def))) out.push(`things for children nobody here may use: ${[...new Set(all.filter((o) => unfit(o.def)).map((o) => o.def))].join(', ')}`);
  // A child's bed is for every child here.
  const childBeds = all.filter((o) => isBed(o.def) && (cfg.residents?.beds.child ?? []).includes(o.def));
  const tooSmall = childBeds.filter((o) => stages.some((s) => !fits(o.def, s)));
  if (tooSmall.length) out.push(`a child's bed a child here may not sleep in: ${[...new Set(tooSmall.map((o) => o.def))].join(', ')}`);
  if (who) {
    const beds = all.filter((o) => isBed(o.def));
    const doubles = beds.filter((o) => (content.object(o.def)?.slots ?? 1) >= 2).length;
    // Places to sleep (a bunk bed has two); short of bedrooms two share one, so two a room at most.
    const places = beds.reduce((n, o) => n + (content.object(o.def)?.slots ?? 1), 0);
    if (places < sleepers(who, bedrooms)) out.push(`${places} places to sleep for ${2 * who.couples + who.adults + who.children}`);
    if (who.couples && doubles < who.couples) out.push('no bed for two for the couple');
  }
  return out;
}
/** The places to sleep a home of `bedrooms` must have for `who` (short of bedrooms, two share one). */
function sleepers(who: Residents, bedrooms: number): number {
  return Math.min(2 * who.couples + who.adults + who.children, 2 * bedrooms);
}
/**
 * What's wrong with a home's plan: its `rooms` and `doors`, the first its front door, which opens
 * into the room it is on the side of that `front` says (a house's: the room on the street side).
 */
function planProblems(rooms: PlannedRoom[], doors: Door[], front: (into: PlannedRoom) => boolean): string[] {
  const out: string[] = [];
  const touches = (a: { x: number; z: number; w: number; d: number }, d: { x: number; z: number; axis: string }) =>
    d.axis === 'x' ? (d.z === a.z || d.z === a.z + a.d) && d.x >= a.x && d.x < a.x + a.w : (d.x === a.x || d.x === a.x + a.w) && d.z >= a.z && d.z < a.z + a.d;
  // The front door: the first door, into a shared room.
  const into = doors[0] && rooms.find((r) => touches(r, doors[0]) && front(r));
  if (!into || !['entry', 'living', 'kitchen', 'dining'].includes(into.kind)) out.push(`front door into ${into?.kind ?? 'nothing'}`);
  if (rooms.some((r) => r.kind === 'entry') && into?.kind !== 'entry') out.push('an entrance hall the front door doesn\'t open into');
  for (const bath of rooms.filter((r) => r.kind === 'bathroom')) {
    if (into && doors.slice(1).some((d) => touches(bath, d) && touches(into, d))) out.push(`bathroom off the ${into.kind} the front door opens into`);
  }
  // Nobody walks through a bathroom or a laundry, nor through a bedroom (but into its own
  // bathroom): one door to anything else each, besides the bedroom's to its bathrooms.
  for (const r of rooms.filter((r) => ['bathroom', 'laundry', 'bedroom'].includes(r.kind))) {
    const ways = doors.filter((d) => touches(r, d)).filter((d) => !rooms.some((o) => o !== r && o.kind === 'bathroom' && r.kind === 'bedroom' && touches(o, d)));
    if (ways.length > 1) out.push(`a way through the ${r.kind} at ${r.x},${r.z}`);
  }
  return out;
}
const stats = { tried: 0, made: 0, two: 0, basements: 0, value: 0, seats: 0, dining: 0, tops: 0, rooms: 0, backs: 0, gardenDoors: 0 };
/** Per house size and garden size: how much of the lot houses cover (each house's share). */
const covers = new Map<string, number[]>();
// Per room kind: how many, their floor in all, the biggest (tiles).
const kinds = new Map<string, { n: number; area: number; most: number; long: number }>();
/** The house classes, smallest first (`housegen.json` `classes`, by the label houses carry as `class`; a flat is no house's). */
const CLASSES = Object.values(cfg.classes)
  .map((c) => c.label)
  .filter((c) => c !== 'flat');
/** Per house class, and per class and garden size: the houses' prices (as `price.ts` prices a home on its lot). */
const prices = new Map<string, number[]>();
const objectPrice = (def: string) => content.object(def)?.price ?? 0;
// A farm lot's house is a large lot's (town.ts `houseLot`).
for (const [size, lot] of Object.entries(houses.lotSizes).filter(([size]) => size !== 'farm')) {
  for (let seed = 1; seed <= perSize; seed++) {
    stats.tried++;
    const garden = GARDEN_SIZES[seed % GARDEN_SIZES.length];
    const name = `${size} ${seed * 7919} ${garden}`;
    const variants = households.map(([name, who]) => [name, who, generateHouse(cfg, info, lot, houses.plot, size, seed * 7919, content.lampTiles, content.windowTiles, who, garden)] as const);
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
    if (!ground.length) problems.push(`${name}: no rooms planned`);
    const [x0, x1, z0, z1] = [Math.min(...ground.map((r) => r.x)), Math.max(...ground.map((r) => r.x + r.w)), Math.min(...ground.map((r) => r.z)), Math.max(...ground.map((r) => r.z + r.d))];
    // The lot cut to fit it: inside it, and covered no more and no less than its garden size says.
    if (!h.lot) problems.push(`${name}: no lot cut to fit the house`);
    else {
      // (A house in a row stands on its lot's edge on one side, `ROW_GAP` off it on the other.)
      const row = Object.values(cfg.classes).some((c) => c.row && c.label === h.class);
      const cover = ((x1 - x0) * (z1 - z0)) / (h.lot.width * h.lot.depth);
      const [least, most] = row ? ROW_COVER : COVER[garden];
      if ((row ? x0 !== 0 || x1 !== h.lot.width - ROW_GAP : x0 < 1 || x1 > h.lot.width - 1) || z1 > h.lot.depth - 4) problems.push(`${name}: the house (${x0}..${x1} × ..${z1}) off its ${h.lot.width}×${h.lot.depth} lot`);
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
      const [bare, rates] = [variants[0][2]!, content.economy.homePrice];
      const { width, depth } = bare.lot ?? lot;
      const { total } = priceOf(objectPrice, rates, bare, width, depth);
      for (const k of [cls, `${cls} ${garden}`]) prices.set(k, [...(prices.get(k) ?? []), total]);
      // The same lot with the house cleared away.
      prices.set(`empty ${garden}`, [...(prices.get(`empty ${garden}`) ?? []), priceOf(objectPrice, rates, undefined, width, depth).total]);
    }
    if (h.upper) stats.two++;
    if (h.basement) stats.basements++;
    const all = [...h.objects, ...(h.upper?.objects ?? []), ...(h.basement?.objects ?? [])];
    stats.value += all.reduce((n, o) => n + (content.object(o.def)?.price ?? 0), 0);
    const seats = all.filter((o) => content.object(o.def)?.seatAt).length;
    stats.seats += seats;
    if (seats) stats.dining++;
    stats.tops += all.filter((o) => content.object(o.def)?.layer === 'surface').length;
    for (const r of [...(h.rooms ?? []), ...(h.upper?.rooms ?? []), ...(h.basement?.rooms ?? [])]) {
      const k = kinds.get(r.kind) ?? { n: 0, area: 0, most: 0, long: 0 };
      kinds.set(r.kind, { n: k.n + 1, area: k.area + r.w * r.d, most: Math.max(k.most, r.w * r.d), long: Math.max(k.long, r.w, r.d) });
      stats.rooms++;
    }
    // On its own lot, turned to face a street on any side (as `town.ts` turns lots onto a town).
    const facing = (seed % 4) as Facing;
    const { width, depth } = h.lot ?? lot;
    // (A house in a row, its wall on its lot's edge, on a town a metre wider all round, as a
    // town has land beyond its lots: a room against the town's edge is outdoors to the sim.)
    const pad = Object.values(cfg.classes).some((c) => c.row && c.label === h.class) ? 1 : 0;
    const turn = turner(pad, pad, width, depth, facing);
    const [W, D] = facing % 2 ? [depth + 2 * pad, width + 2 * pad] : [width + 2 * pad, depth + 2 * pad];
    const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];
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
        plots: [{ name: 'Home', x: pad, z: pad, w: W - 2 * pad, d: D - 2 * pad, entry: h.spawns[0] && turn.point(h.spawns[0]) }],
        households: [{ name: 'Test', plot: 0, player: true }],
        sims: [],
      },
    });
  }
}

// ---- buildings of flats, flats over places, floors above places --------------------------------
// Sampled as towns of their own like the houses: each building on its lot, turned to face a street
// on each side in turn, every storey on its rows; each flat a plot of its own (`of` the lot), a
// household in most (one a shell now and then), each flat checked here as a house is (beds enough,
// nothing for children without them, its front door from the hall into a shared room, no way
// through a bathroom, a laundry or a bedroom), its layout the same whoever lives there, and priced
// as the game prices a flat (`price.ts` `flatPriceOf`): the cheapest homes.
const { flatsTemplate, flatName, flatsStats, floorsId, overId, placeFloors } = await import('../../web/src/game/flats.ts');
const { flatPriceOf } = await import('../../web/src/game/price.ts');
type Furnished = Residents | 'shell' | undefined;
const templates = { ...houses, houses: [], parks: (houses as unknown as { parks: unknown[] }).parks, gen: cfg, info, lampTiles: content.lampTiles, windowTiles: content.windowTiles } as unknown as import('../../web/src/game/town.ts').Templates;
type Storey = { walls: [number, number, number, number][]; doors: Door[]; windows?: Door[]; objects: (Placement & { unit?: number })[]; rooms?: PlannedRoom[] };
/**
 * A building as a town of its own: `lot` turned `facing`, its `storeys` floors (each a list of
 * layouts on that storey's rows), what stands outside, its lot (`ground`) and its flats as plots
 * (each `of` the lot; things kept by a flat, `unit`, are its), a household in each flat `who` has.
 */
function buildingTown(lot: { width: number; depth: number }, facing: Facing, floors: Storey[][], outside: (Placement & { unit?: number })[], ground: { name: string; entry: [number, number]; public: boolean; kind: string }, flats: { rect: { x: number; z: number; w: number; d: number }; storey: number; door: [number, number]; inside: [number, number]; entry: [number, number] }[], who: Furnished[]) {
  const turn = turner(0, 0, lot.width, lot.depth, facing);
  const [W, D] = facing % 2 ? [lot.depth, lot.width] : [lot.width, lot.depth];
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];
  const all = floors.flatMap((list, k) => list.map((f) => [f, k * D] as const));
  const tileOn = ([x, z]: [number, number], lift: number): [number, number] => {
    const [cx, cz] = turn.point([x + 0.5, z + 0.5]);
    return [Math.floor(cx), Math.floor(cz) + lift];
  };
  const keep = (o: Placement & { unit?: number }, lift: number) => ({ ...turn.place(o, footprint, lift), ...(o.unit !== undefined ? { plot: 1 + o.unit } : {}) });
  return {
    rooms: all.flatMap(([f, lift]) => (f.rooms ?? []).map((r) => ({ ...r, ...turn.rect(r), z: turn.rect(r).z + lift }))),
    town: {
      width: W,
      depth: D,
      storeys: floors.length,
      walls: all.flatMap(([f, lift]) => f.walls.map((w) => turn.wall(w, lift))),
      doors: all.flatMap(([f, lift]) => f.doors.map((d) => turn.edge(d, lift))),
      windows: all.flatMap(([f, lift]) => (f.windows ?? []).map((d) => turn.edge(d, lift))),
      objects: [...all.flatMap(([f, lift]) => f.objects.map((o) => keep(o, lift))), ...outside.map((o) => keep(o, 0))],
      plots: [
        { name: ground.name, x: 0, z: 0, w: W, d: D, public: ground.public, kind: ground.kind, entry: turn.point(ground.entry) },
        ...flats.map((f, i) => {
          const [ix, iz] = turn.point(f.inside);
          return { name: flatName(ground.name, i), ...turn.rect(f.rect), public: false, kind: 'flat', of: 0, storey: f.storey, entry: turn.point(f.entry), door: tileOn(f.door, f.storey * D), inside: [ix, iz + f.storey * D] };
        }),
      ],
      households: who.flatMap((w, i) => (w && w !== 'shell' ? [{ name: `Flat ${i + 1}`, plot: 1 + i, ...(i === 0 ? { player: true } : {}) }] : [])),
      sims: [],
    },
  };
}
/** The layout of a flat (to compare one furnished for different households). */
const flatLayout = (f: { walls: unknown; doors: unknown; windows: unknown; rooms: PlannedRoom[] }) => JSON.stringify([f.walls, f.doors, f.windows, f.rooms]);
const residentsCycle = households.filter(([, who]) => who).map(([, who]) => who!);
const flatPrices: number[] = [];
const buildMs: number[] = [];
const flatStats = { buildings: 0, flats: 0, over: 0, overFlats: 0, floors: 0, floorRooms: 0, shells: 0, bays: 0 };
const perFlats = Number(process.argv[4] ?? 24);
/** Checks flat `u` of building `label` for `who`; its layout against the same flat furnished for nobody (`bare`). */
function checkFlat(label: string, u: { rect: { x: number; z: number; w: number; d: number }; bedrooms: number; door: [number, number]; inside: [number, number]; layout: { walls: unknown; doors: Door[]; windows: unknown; objects: Placement[]; rooms: PlannedRoom[] } }, bare: typeof u, who: Furnished) {
  if (flatLayout(u.layout) !== flatLayout(bare.layout)) problems.push(`${label}: another layout`);
  if (who === 'shell') {
    const loose = u.layout.objects.filter((o) => isBed(o.def) || content.object(o.def)?.group === 'seating' || (content.object(o.def)?.category === 'living' && o.def !== cfg.stairs));
    if (loose.length) problems.push(`${label}: a shell with ${[...new Set(loose.map((o) => o.def))].join(', ')}`);
    if (!u.layout.objects.some((o) => o.def === 'toilet' || o.def === 'toiletCompact')) problems.push(`${label}: a shell without a toilet`);
    return;
  }
  for (const p of furnishingProblems(u.layout.objects, u.bedrooms, who)) problems.push(`${label}: ${p}`);
  // Its front door: the hall's tile before it, the flat's behind it.
  const [ix, iz] = [Math.floor(u.inside[0]), Math.floor(u.inside[1])];
  const inFlat = (x: number, z: number, r: { x: number; z: number; w: number; d: number }) => x >= r.x && z >= r.z && x < r.x + r.w && z < r.z + r.d;
  if (inFlat(...u.door, u.rect) || !inFlat(ix, iz, u.rect)) problems.push(`${label}: its door at ${u.door} isn't from the hall`);
  for (const p of planProblems(u.layout.rooms, u.layout.doors, (r) => inFlat(ix, iz, r))) problems.push(`${label}: ${p}`);
  for (const o of u.layout.objects) {
    const [fw, fd] = content.object(o.def)?.footprint ?? [1, 1];
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    if (!inFlat(o.x, o.z, u.rect) || !inFlat(o.x + w - 1, o.z + d - 1, u.rect)) problems.push(`${label}: the ${o.def} at ${o.x},${o.z} is out of the flat`);
  }
}
// Buildings of flats: two or three storeys, wide and narrow, with bays and without.
for (let seed = 1; seed <= perFlats; seed++) {
  const id = `flats:${seed * 104729}:${2 + (seed % 2)}:${(seed >> 1) % 2}${(seed >> 2) % 2 ? ':p' : ''}`;
  const t0 = performance.now();
  const bare = flatsTemplate(templates, id);
  buildMs.push(performance.now() - t0);
  if (!bare) {
    problems.push(`${id}: no building`);
    continue;
  }
  // A household in each flat, one a shell now and then.
  const who: Furnished[] = bare.units.map((_, i) => (seed % 3 === 0 && i === bare.units.length - 1 ? 'shell' : residentsCycle[(seed + i) % residentsCycle.length]));
  const b = flatsTemplate(templates, id, who)!;
  flatStats.buildings++;
  flatStats.flats += b.units.length;
  flatStats.shells += who.filter((w) => w === 'shell').length;
  flatStats.bays += b.bays.length;
  b.units.forEach((u, i) => checkFlat(`${id} flat ${i + 1}`, u, bare.units[i], who[i]));
  for (const i of bare.units.keys()) flatPrices.push(flatPriceOf(objectPrice, content.economy.homePrice, bare, i).total);
  if (b.outside.filter((o) => o.unit !== undefined).length !== b.units.length) problems.push(`${id}: ${b.outside.filter((o) => o.unit !== undefined).length} letterboxes for ${b.units.length} flats`);
  const floors: Storey[][] = b.shell.map((s, k) => [s, ...b.units.filter((u) => u.storey === k).map((u) => u.layout)]);
  const facing = (seed % 4) as Facing;
  const made = buildingTown(b.lot, facing, floors, b.outside, { name: b.name, entry: b.entry, public: false, kind: 'flats' }, b.units.map((u) => ({ ...u, entry: b.entry })), who);
  towns.push({ kind: 'flats', size: 'flats', facing, seed: seed * 104729, household: who.map((w) => (w === 'shell' ? 'shell' : residentsKey(w))).join(' '), name: b.name, description: b.description, flats: b.units.map((u, i) => ({ plot: 1 + i, need: who[i] && who[i] !== 'shell' ? sleepers(who[i] as Residents, u.bedrooms) : 0 })), ...made });
}
// Flats over the places that may have them, two and three storeys.
for (const place of ['shopSmall', 'shopGoods', 'cafeSmall', 'restaurant', 'boutique', 'villageShop', 'farmShop']) {
  for (const storeys of [2, 3]) {
    for (let seed = 1; seed <= 2; seed++) {
      const id = overId(place, seed * 7919 + storeys, storeys);
      const bare = flatsTemplate(templates, id);
      if (!bare) {
        if (place !== 'restaurant') problems.push(`${id}: no flats over it`);
        continue;
      }
      const who: Furnished[] = bare.units.map((_, i) => residentsCycle[(seed + i + storeys) % residentsCycle.length]);
      const b = flatsTemplate(templates, id, who)!;
      flatStats.over++;
      flatStats.overFlats += b.units.length;
      b.units.forEach((u, i) => checkFlat(`${id} flat ${i + 1}`, u, bare.units[i], who[i]));
      const ground = b.ground!;
      const floors: Storey[][] = b.shell.map((s, k) => [...(k === 0 ? [{ walls: ground.walls ?? [], doors: ground.doors ?? [], windows: ground.windows, objects: ground.objects }] : []), s, ...b.units.filter((u) => u.storey === k).map((u) => u.layout)]);
      const facing = ((seed + storeys) % 4) as Facing;
      const made = buildingTown(b.lot, facing, floors, b.outside, { name: ground.name, entry: ground.entry ?? [b.lot.width / 2, 1.5], public: true, kind: 'shop' }, b.units.map((u) => ({ ...u, entry: b.entry })), who);
      towns.push({ kind: 'flats', ground: 'place', size: place, facing, seed: seed * 7919 + storeys, household: who.map((w) => residentsKey(w as Residents)).join(' '), name: ground.name, description: b.description, flats: b.units.map((u, i) => ({ plot: 1 + i, need: sleepers(who[i] as Residents, u.bedrooms) })), ...made });
    }
  }
}
// Real storeys above the places that may have them.
const FLOORS: [string, string][] = [['officeSuite', 'office'], ['bank', 'office'], ['newspaper', 'office'], ['solicitors', 'office'], ['police', 'office'], ['medical', 'office'], ['works', 'office'], ['armyReserve', 'office'], ['fireStation', 'workshop'], ['garage', 'workshop'], ['workshopUnit', 'workshop'], ['studios', 'workshop'], ['builders', 'workshop'], ['parishHall', 'hall']];
for (const [place, use] of FLOORS) {
  for (const storeys of [2, 3]) {
    const id = floorsId(place, 7919 + storeys, storeys, use);
    const b = placeFloors(templates, id);
    if (!b) {
      // (A low workshop or hall may have no room for a second flight.)
      if (storeys === 2 || use === 'office') problems.push(`${id}: no floors above it`);
      continue;
    }
    flatStats.floors++;
    flatStats.floorRooms += b.floors.reduce((n, f) => n + f.rooms.length, 0);
    const floors: Storey[][] = [[{ walls: b.ground.walls ?? [], doors: b.ground.doors ?? [], windows: b.ground.windows, objects: b.ground.objects }], ...b.floors.map((f) => [f])];
    const lot = b.ground.lot ?? templates.plot;
    const facing = (storeys % 4) as Facing;
    const made = buildingTown(lot, facing, floors, [], { name: b.ground.name, entry: b.ground.entry ?? [lot.width / 2, 1.5], public: true, kind: 'shop' }, [], []);
    towns.push({ kind: 'place', ground: 'place', size: place, facing, seed: 7919 + storeys, household: use, name: b.ground.name, description: `${b.ground.name} with ${storeys - 1} real storey${storeys > 2 ? 's' : ''} above it`, ...made });
  }
}
buildMs.sort((a, b) => a - b);
const atShare = (q: number) => buildMs[Math.min(buildMs.length - 1, Math.floor(q * buildMs.length))];
console.log(`${flatStats.buildings} buildings of flats (${flatStats.flats} flats, ${flatStats.shells} shells, ${flatStats.bays} bays), ${flatStats.over} places with ${flatStats.overFlats} flats over them, ${flatStats.floors} places with ${flatStats.floorRooms} rooms on storeys above`);
console.log(`  a building of flats in ${atShare(0.5).toFixed(1)} ms (median; 90%: ${atShare(0.9).toFixed(1)}, most ${atShare(1).toFixed(1)}), ${flatsStats.made} made in ${flatsStats.ms.toFixed(0)} ms in all`);
if (timing && atShare(0.5) > 15) problems.push(`a building of flats takes ${atShare(0.5).toFixed(1)} ms (median), more than 15`);
if (flatPrices.length) prices.set('flat', flatPrices);

if (stats.made < stats.tried * 0.9) problems.push(`only ${stats.made} of ${stats.tried} houses made`);
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
// Flats are the cheapest homes: dearer on average than none of the house classes.
if (prices.has('flat')) {
  const flat = mean(prices.get('flat')!);
  console.log(`    flat: ${priced(prices.get('flat')!)}`);
  for (const [c, p] of means) if (p <= flat) problems.push(`a ${c} costs ${Math.round(p).toLocaleString()} on average, a flat ${Math.round(flat).toLocaleString()}: flats are the cheapest homes`);
}
for (const t of towns.slice(0, 8) as { size: string; name: string; description: string }[]) console.log(`  ${t.size}: ${t.name} — ${t.description}`);
if (problems.length) {
  console.log(`${problems.length} problems:`);
  for (const p of problems.slice(0, 60)) console.log(`  ${p}`);
  process.exitCode = 1;
}
