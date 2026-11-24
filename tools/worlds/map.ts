/**
 * Maps of generated worlds as SVG, and how often the motorway fits: every place (web/src/game/world.ts `WORLD_KINDS`) at
 * each land size (`--sizes`, default all three), `n` worlds of each (seeds as `sample.ts` draws them), laid the way a new
 * game lays them (step 1's land, `landDraft`; step 2's town on it, `layOn`; `--sizes own`: the place's own size), and the classic worlds (`--classic`: their
 * kinds, seeds 11 and 22). Each map draws the water, the woods and fields, the areas' boxes, the lots and the roads by
 * kind: the motorway's carriageways, its overpasses (`raised`) and bridges, its ramps, the roads under it. North (game z)
 * is up, as the game's 3D view has it from above: the ground is drawn in game coordinates under a flip in y.
 *
 * A line per world: whether it has a motorway (else why not: `diagnostics.motorway`), its crossings, its ramps'
 * lengths, how many villages the world has, and what is wrong with it (`motorway.ts` `motorwayProblems`, and any crossing not under a deck); then the
 * share of worlds with one (and their villages), per place and per size.
 *
 * `--seed n`: that seed alone (as the bench's big world, `?bench=bigworld`, makes its town from seed 7), `--options
 * town=city,villages=many`: those options over each size's. `--spots <file>`: per world with a motorway, where its
 * junction's ramps meet the road under it and where that road passes under it, as offsets (m) from the main town's
 * middle as the bench's town view finds it (`game/townPreview.ts` `townCentre`): `tools/bench/roadShots.mjs --spots`.
 *
 * Run: bun tools/worlds/map.ts <out dir> [worlds per place and size] [--kinds farm,river] [--sizes small,large] [--classic] [--seed-base n] [--seed n] [--options town=city] [--spots file] [--no-maps]
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { HouseGenConfig } from '../../web/src/game/housegen.ts';
import type { RoadPlan } from '../../web/src/core/protocol.ts';
import type { NeighbourhoodDraft, Templates } from '../../web/src/game/town.ts';

const root = new URL('../../web/public/', import.meta.url);
Object.assign(globalThis, { location: { href: root.href, search: '' } });
const fetchFile = async (url: string | URL) => {
  const text = readFileSync(fileURLToPath(new URL(url, root)), 'utf8');
  return { ok: true, status: 200, text: async () => text, json: async () => JSON.parse(text) };
};
Object.assign(globalThis, { fetch: fetchFile });

const { Content } = await import('../../web/src/content/content.ts');
const { objectInfos } = await import('../../web/src/game/housegen.ts');
const { plotRect, useTownContent } = await import('../../web/src/game/town.ts');
const { generateFromSpec, landDraft, layOn } = await import('../../web/src/game/macro.ts');
const { CLASSIC, WORLD_KINDS } = await import('../../web/src/game/world.ts');
const { motorwayProblems } = await import('../../web/src/game/motorway.ts');
const { deckSpans, lengths, pointAt } = await import('../../web/src/game/roads.ts');
const { previewWorld, townCentre } = await import('../../web/src/game/townPreview.ts');
type LandSize = 'small' | 'medium' | 'large';

const content = await Content.load(new URL('content/base.json', root).href);
useTownContent(content);
const read = (name: string) => JSON.parse(readFileSync(fileURLToPath(new URL(`content/${name}`, root)), 'utf8'));
const templates: Templates = { ...(read('houses.json') as Templates), gen: read('housegen.json') as HouseGenConfig, info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

const flag = (name: string) => {
  const i = process.argv.indexOf(name);
  return i < 0 ? undefined : process.argv.splice(i, 2)[1];
};
const has = (name: string) => {
  const i = process.argv.indexOf(name);
  if (i >= 0) process.argv.splice(i, 1);
  return i >= 0;
};
const seedBase = Number(flag('--seed-base') ?? 0);
const oneSeed = flag('--seed');
const forced = Object.fromEntries((flag('--options') ?? '').split(',').filter(Boolean).map((kv) => kv.split('=')));
const spotsFile = flag('--spots');
const spots: Record<string, unknown>[] = [];
const kinds = (flag('--kinds')?.split(',') ?? [...WORLD_KINDS]) as (typeof WORLD_KINDS)[number][];
const sizes = (flag('--sizes')?.split(',') ?? ['small', 'medium', 'large']) as LandSize[];
const classic = has('--classic');
const maps = !has('--no-maps');
const out = process.argv[2] ?? 'bench-results/maps';
const perKind = Number(process.argv[3] ?? 2);
mkdirSync(out, { recursive: true });

const KIND_STYLE: Record<string, [string, number]> = {
  motorway: ['#1f4e9c', 1],
  ramp: ['#3c7be0', 1],
  avenue: ['#7a3d9a', 1],
  country: ['#8c6b3f', 1],
  street: ['#555', 1],
  lane: ['#777', 1],
  trail: ['#b5a27a', 0.6],
};

/** One world's map, north up (the ground under `matrix(1 0 0 -1 0 D)`: z = 0 at the bottom, z = D at the top; the title upright above it). */
function svg(d: NeighbourhoodDraft, title: string): string {
  const [W, D] = [d.width, d.depth];
  const poly = (pts: readonly [number, number][]) => pts.map(([x, z]) => `${x.toFixed(1)},${z.toFixed(1)}`).join(' ');
  const parts: string[] = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-10 -30 ${W + 20} ${D + 40}" width="${Math.round((W + 20) * 1.1)}" height="${Math.round((D + 40) * 1.1)}">`,
    `<g transform="matrix(1 0 0 -1 0 ${D})">`,
    `<rect x="0" y="0" width="${W}" height="${D}" fill="#e9efdc"/>`,
  ];
  const fills: Record<string, string> = { water: '#9cc3e6', woods: '#9dbf8a', field: '#e6dca4', meadow: '#dbe8c4' };
  for (const z of d.open?.zones ?? []) if (fills[z.kind]) parts.push(`<polygon points="${poly(z.poly)}" fill="${fills[z.kind]}" fill-rule="evenodd" stroke="none"/>`);
  for (const a of d.open?.areas ?? []) parts.push(`<rect x="${a.x}" y="${a.z}" width="${a.w}" height="${a.d}" fill="none" stroke="#c88" stroke-dasharray="6 4"/>`);
  for (const s of d.slots) {
    const r = plotRect(templates, s);
    parts.push(`<rect x="${r.x}" y="${r.z}" width="${r.w}" height="${r.d}" fill="#d8b9a0" stroke="#a87" stroke-width="0.4"/>`);
  }
  const line = (r: RoadPlan, colour: string, width: number, extra = '') => `<polyline points="${poly(r.points)}" fill="none" stroke="${colour}" stroke-width="${width}" stroke-linecap="round" stroke-linejoin="round"${extra}/>`;
  const order = (r: RoadPlan) => ['trail', 'lane', 'street', 'avenue', 'country', 'ramp', 'motorway'].indexOf(r.kind ?? 'street');
  for (const r of [...d.roads].sort((a, b) => order(a) - order(b))) {
    const [colour, k] = KIND_STYLE[r.kind ?? 'street'] ?? ['#555', 1];
    parts.push(line(r, colour, r.width * k));
    // (Its decks: overpasses orange, bridges dark.)
    const ls = lengths(r.points);
    for (const [a, b] of deckSpans(r)) {
      const pts = r.points.filter((_, i) => ls[i] >= a && ls[i] <= b);
      if (pts.length > 1) parts.push(line({ ...r, points: pts }, (r.raised ?? []).some(([x, y]) => x <= a + 0.01 && y >= b - 0.01) ? '#f08a24' : '#333', r.width * 0.5));
    }
    for (const c of r.crosses ?? []) {
      const { p } = pointAt(r.points, c.s, ls);
      parts.push(`<circle cx="${p[0].toFixed(1)}" cy="${p[1].toFixed(1)}" r="3" fill="none" stroke="#d22" stroke-width="1"/>`);
    }
  }
  parts.push('</g>', `<text x="0" y="-12" font-family="sans-serif" font-size="14">${title.replace(/&/g, '&amp;')}</text>`, '</svg>');
  return parts.join('\n');
}

/** What a world's motorway is: a line. */
function judged(d: NeighbourhoodDraft): { fits: boolean; line: string; problems: string[] } {
  const roads = d.roads;
  const ways = roads.filter((r) => r.kind === 'motorway');
  if (!ways.length) return { fits: false, line: `no motorway (${d.diagnostics?.motorway ?? 'no note'})`, problems: [] };
  const problems = motorwayProblems(roads);
  // (Every crossing under a deck, on both.)
  roads.forEach((w, wi) => {
    if (w.kind !== 'motorway') return;
    for (const c of w.crosses ?? []) {
      if (!deckSpans(w).some(([a, b]) => c.s > a && c.s < b)) problems.push(`${w.name}: crossing of ${roads[c.road]?.name} at ${c.s.toFixed(0)} m not under a deck`);
      if (!roads[c.road]?.crosses?.some((x) => x.road === wi)) problems.push(`${roads[c.road]?.name} doesn't list ${w.name}`);
    }
  });
  const ramps = roads.filter((r) => r.kind === 'ramp').map((r) => lengths(r.points).at(-1)!.toFixed(0));
  const n = ways[0].crosses?.length ?? 0;
  const L = lengths(ways[0].points).at(-1)!;
  return { fits: true, line: `motorway ${L.toFixed(0)} m, ${n} crossing${n === 1 ? '' : 's'}, ${ways[0].bridge ? 'a bridge, ' : ''}ramps ${ramps.join('/')} m`, problems };
}

const rows: { kind: string; size: string; fits: boolean; villages: number }[] = [];
const allProblems: string[] = [];
const runs: { kind: string; size: string; seed: number; make: () => NeighbourhoodDraft }[] = [];
for (const kind of kinds) {
  for (const size of sizes) {
    for (let i = 0; i < (oneSeed ? 1 : perKind); i++) {
      const seed = oneSeed ? Number(oneSeed) : (i + 1) * 104729 + kind.length + seedBase * 7919;
      runs.push({ kind, size, seed, make: () => layOn(content, templates, landDraft(content, templates, kind, { ...(size === ('own' as LandSize) ? {} : { size }), ...forced }, seed), {}, { waterKept: true }) });
    }
  }
}
if (classic) for (const kind of Object.keys(CLASSIC)) for (const seed of [11, 22]) runs.push({ kind: `classic ${kind}`, size: 'classic', seed, make: () => generateFromSpec(content, templates, CLASSIC[kind as keyof typeof CLASSIC], seed) });
for (const { kind, size, seed, make } of runs) {
  const t0 = performance.now();
  let d: NeighbourhoodDraft;
  try {
    d = make();
  } catch (e) {
    console.log(`${kind} ${size} ${seed}: no world (${(e as Error).message})`);
    continue;
  }
  const ms = performance.now() - t0;
  const { fits, line, problems } = judged(d);
  const tag = `${kind} ${size} ${seed}`;
  const villages = (d.open?.areas.length ?? 1) - 1;
  const left = d.diagnostics?.villages?.left;
  console.log(`${tag}: ${line}, ${villages} village${villages === 1 ? '' : 's'}${left ? ` (${left} short)` : ''} (${d.width}×${d.depth} m, ${ms.toFixed(0)} ms)${problems.length ? `\n  ${problems.join('\n  ')}` : ''}`);
  for (const p of problems) allProblems.push(`${tag}: ${p}`);
  for (const p of d.diagnostics?.roads ?? []) console.log(`  road fault: ${p}`);
  rows.push({ kind, size, fits, villages });
  if (spotsFile && fits) spots.push({ kind, size, seed, ...spotsOf(d) });
  if (maps) writeFileSync(`${out}/${kind.replace(/ /g, '-')}-${size}-${seed}.svg`, svg(d, `${tag}: ${line}`));
}
if (spotsFile) writeFileSync(spotsFile, JSON.stringify(spots, null, 1));

/** Where world `d`'s junction and its overpass are, from its main town's middle (`townCentre` of the world drawn). */
function spotsOf(d: NeighbourhoodDraft): Record<string, unknown> {
  const [cx, , cz] = townCentre(previewWorld(content, templates, d, null));
  const ramp = d.roads.find((r) => r.kind === 'ramp')!;
  const T = ramp.ends[0] === 'join' ? ramp.points[0] : ramp.points[ramp.points.length - 1];
  const way = d.roads.find((r) => r.kind === 'motorway')!;
  const c = way.crosses?.[0];
  const under = c ? pointAt(way.points, c.s).p : T;
  const off = (p: [number, number]) => ({ tx: Math.round(p[0] - cx), tz: Math.round(p[1] - cz) });
  return { centre: [Math.round(cx), Math.round(cz)], junction: off(T), overpass: off(under) };
}

const share = (list: typeof rows) => `${list.filter((r) => r.fits).length}/${list.length}, villages ${list.reduce((n, r) => n + r.villages, 0)}`;
console.log('\nMotorway fits, by place:');
for (const k of new Set(rows.map((r) => r.kind))) console.log(`  ${k}: ${share(rows.filter((r) => r.kind === k))}`);
console.log('By size:');
for (const s of new Set(rows.map((r) => r.size))) console.log(`  ${s}: ${share(rows.filter((r) => r.size === s))}`);
console.log(`All: ${share(rows)}`);
if (allProblems.length) {
  console.log(`\n${allProblems.length} problems.`);
  process.exitCode = 1;
}
