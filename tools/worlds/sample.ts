/**
 * Makes sample worlds of every kind (web/src/game/world.ts) with the shipped content, the way a
 * new game does (a world, its neighbours, a player household moving into an empty house), and
 * writes their town files for the simulation to check
 * (`cargo test -p sim-core --test generated_worlds -- --ignored`, reading WORLDS). Checks the
 * layout first: lots inside the town, apart from each other and off the roads, each by its road (a
 * park may stand on a green),
 * the ways out of town on its edge. And the neighbours: every household's savings cover its
 * home's price (web/src/game/wealth.ts, price.ts); per kind of house, how old its owners are and
 * how high up they work. Problems are listed (and the exit code is 1).
 *
 * Run: bun tools/worlds/sample.ts worlds.json [worlds per kind]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';
import type { Templates } from '../../web/src/game/town.ts';

// The loaders fetch from the page's address: here, files under web/public.
const root = new URL('../../web/public/', import.meta.url);
Object.assign(globalThis, { location: { href: root.href, search: '' } });
const fetchFile = async (url: string | URL) => {
  const text = readFileSync(fileURLToPath(new URL(url, root)), 'utf8');
  return { ok: true, status: 200, text: async () => text, json: async () => JSON.parse(text) };
};
Object.assign(globalThis, { fetch: fetchFile });

const { Content } = await import('../../web/src/content/content.ts');
const { AssetRegistry } = await import('../../web/src/assets/registry.ts');
const { objectInfos } = await import('../../web/src/game/housegen.ts');
const { randomHousehold } = await import('../../web/src/game/household.ts');
const { assembleTown, generateWorld, houseTemplate, populate, vacantSlots } = await import('../../web/src/game/town.ts');
const { WORLD_KINDS } = await import('../../web/src/game/world.ts');
const { homePrice } = await import('../../web/src/game/price.ts');
const { savingsOf } = await import('../../web/src/game/wealth.ts');
const { nearest, roadMask } = await import('../../web/src/game/roads.ts');

type Rect = { x: number; z: number; w: number; d: number };
/** What's wrong with a town's layout. */
function layoutProblems(town: { width: number; depth: number; plots: (Rect & { public: boolean })[]; exits: [number, number][]; meta: { roads?: { points: [number, number][]; width: number }[]; facing?: number[] } }): string[] {
  const out: string[] = [];
  const roads = town.meta.roads ?? [];
  const mask = roadMask(roads, town.width, town.depth, 0);
  town.plots.forEach((p, i) => {
    if (p.x < 0 || p.z < 0 || p.x + p.w > town.width || p.z + p.d > town.depth) out.push(`plot ${i} off the town`);
    for (const [j, q] of town.plots.entries()) if (j > i && p.x < q.x + q.w && q.x < p.x + p.w && p.z < q.z + q.d && q.z < p.z + p.d) out.push(`plots ${i} and ${j} overlap`);
    let onRoad = false;
    for (let z = p.z; z < p.z + p.d; z++) for (let x = p.x; x < p.x + p.w; x++) onRoad ||= mask[z * town.width + x] === 1 && roads.some((r) => nearest(r.points, x + 0.5, z + 0.5).dist < r.width / 2);
    if (onRoad) out.push(`plot ${i} on a road`);
    // The middle of its front, by a road.
    const f = town.meta.facing?.[i] ?? 0;
    const [fx, fz] = f === 0 ? [p.x + p.w / 2, p.z] : f === 1 ? [p.x, p.z + p.d / 2] : f === 2 ? [p.x + p.w / 2, p.z + p.d] : [p.x + p.w, p.z + p.d / 2];
    const gap = Math.min(...roads.map((r) => nearest(r.points, fx, fz).dist - r.width / 2));
    // (A park may stand in the middle of a green.)
    if (roads.length && gap > 13 && !p.public) out.push(`plot ${i} ${gap.toFixed(1)} m from its road`);
  });
  for (const [x, z] of town.exits) if (Math.min(x, z, town.width - x, town.depth - z) > 1) out.push(`a way out of town at ${x}, ${z}, not on its edge`);
  if (!town.exits.length) out.push('no way out of town');
  return out;
}
const problems: string[] = [];

const content = await Content.load(new URL('content/base.json', root).href);
const assets = await AssetRegistry.load(new URL('assets/manifest.json', root).href);
const read = (name: string) => JSON.parse(readFileSync(fileURLToPath(new URL(`content/${name}`, root)), 'utf8'));
const templates: Templates = { ...(read('houses.json') as Templates), gen: read('housegen.json') as HouseGenConfig, info: objectInfos(content), lampTiles: content.lampTiles };

const out = process.argv[2] ?? 'worlds.json';
const perKind = Number(process.argv[3] ?? 3);
const worlds: unknown[] = [];
/** House classes (housegen.json `classes`), as a generated house's description names them. */
const HOUSE_CLASSES = ['cottage', 'bungalow', 'town house', 'family house', 'villa'];
/** Per house class: homes lived in, and their owners' summed eldest grown-up's age, top job grade (-1 none), how many are settled (`SETTLED_AGE` or `SENIOR`), price, savings. */
const owned: Record<string, { homes: number; age: number; level: number; settled: number; price: number; savings: number }> = {};
const SETTLED_AGE = 35;
/** Grade F, Senior. */
const SENIOR = 5;
const forSale: Record<string, number> = {};
for (const kind of WORLD_KINDS) {
  for (let i = 1; i <= perKind; i++) {
    const seed = i * 104729 + kind.length;
    const town = populate(content, assets, templates, generateWorld(content, templates, kind, seed));
    const home = vacantSlots(town)[0];
    if (!home) throw new Error(`${kind} ${seed}: no empty house`);
    const player = randomHousehold(content, assets, 2);
    const file = JSON.parse(assembleTown(content, templates, town, player, home.index));
    for (const p of layoutProblems(file)) problems.push(`${kind} ${seed}: ${p}`);
    for (const { household, slot } of town.households) {
      const s = town.slots[slot];
      const price = homePrice(content, templates, s);
      const savings = savingsOf(content, household);
      if (savings < price) problems.push(`${kind} ${seed}: the ${household.name}s saved $${savings} for ${s.name}, which costs $${price}`);
      const house = houseTemplate(templates, s.template);
      const cls = HOUSE_CLASSES.find((c) => house?.description.toLowerCase().includes(c)) ?? house?.name ?? s.template;
      const adults = household.members.filter((m) => content.isAdult(m.age));
      const owners = (owned[cls] ??= { homes: 0, age: 0, level: 0, settled: 0, price: 0, savings: 0 });
      const eldest = Math.max(0, ...adults.map((m) => m.age));
      const level = Math.max(-1, ...adults.map((m) => m.job?.level ?? -1));
      owners.homes++;
      owners.age += eldest;
      owners.level += level;
      if (eldest >= SETTLED_AGE || level >= SENIOR) owners.settled++;
      owners.price += price;
      owners.savings += savings;
    }
    for (const s of vacantSlots(town)) {
      const house = houseTemplate(templates, s.template);
      const cls = HOUSE_CLASSES.find((c) => house?.description.toLowerCase().includes(c)) ?? house?.name ?? s.template;
      forSale[cls] = (forSale[cls] ?? 0) + 1;
    }
    worlds.push({ kind, seed, name: town.name, town: file });
    const farms = town.slots.filter((s) => s.lot === 'farm').length;
    const basements = town.slots.filter((s) => s.kind === 'house' && houseTemplate(templates, s.template)?.basement).length;
    console.log(
      `${kind} ${seed}: ${town.name} (${town.world?.tagline}) ${town.width}×${town.depth}, ${town.roads?.length ?? town.streets.length} road(s), ` +
        `${town.slots.filter((s) => s.kind === 'house').length} houses${farms ? ` (${farms} farms)` : ''}, ${basements} with a basement, ${town.households.length} households, ` +
        `${file.sims.length} residents, ${file.objects.length} things`,
    );
  }
}
console.log(`Owners per kind of house (the eldest grown-up, the highest job grade, A = 0; settled: ${SETTLED_AGE}+ or grade F up):`);
for (const cls of [...HOUSE_CLASSES, ...Object.keys(owned).filter((c) => !HOUSE_CLASSES.includes(c))]) {
  const o = owned[cls];
  if (!o && !forSale[cls]) continue;
  const n = o?.homes ?? 0;
  const mean = (v: number) => (n ? (v / n).toFixed(1) : '-');
  const dollars = (v: number) => (n ? `$${Math.round(v / n / 1000)}k` : '-');
  console.log(
    `  ${cls.padEnd(13)} ${String(n).padStart(3)} lived in, ${String(forSale[cls] ?? 0).padStart(3)} for sale: age ${mean(o?.age ?? 0)}, grade ${mean(o?.level ?? 0)}, ${n ? Math.round((100 * o!.settled) / n) : '-'}% settled, price ${dollars(o?.price ?? 0)}, saved ${dollars(o?.savings ?? 0)}`,
  );
}
writeFileSync(out, JSON.stringify(worlds));
console.log(`${worlds.length} worlds → ${out}`);
if (problems.length) {
  console.log(`${problems.length} problems:`);
  for (const p of problems.slice(0, 60)) console.log(`  ${p}`);
  process.exitCode = 1;
}
