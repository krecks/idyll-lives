/**
 * Draws the sample worlds (`tools/worlds/sample.ts`'s output) from above, one SVG each, to see
 * how a kind of world is laid out: its roads (or an older town's streets) and greens, the lots
 * (public ones blue, farms' fields yellow, outdoor places orange; a tick on the side they face),
 * garden paths and driveways, the ground floor's walls, the things standing about and the ways
 * out of town (red). A big world also shows its land (water,
 * woods, hills, meadows, fields, orchards), its country roads and trails (dashed), and its areas' names.
 * An index.html shows them all on one page.
 *
 * Run: bun tools/worlds/map.ts worlds.json out-dir [more.json ...]
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

type Rect = { x: number; z: number; w: number; d: number };
type Pt = [number, number];
type Town = {
  size?: string;
  width: number;
  depth: number;
  walls: [number, number, number, number][];
  objects: { x: number; z: number }[];
  plots: (Rect & { name: string; public: boolean; kind?: string })[];
  exits?: (Pt | { x: number; z: number; kind: 'road' })[];
  parking?: { x: number; z: number }[];
  meta?: {
    streets?: Rect[];
    roads?: { points: Pt[]; width: number; kind?: string; name?: string }[];
    greens?: Pt[][];
    paths?: Rect[];
    facing?: number[];
    world?: { fields?: Rect[] };
    zones?: { kind: string; poly: Pt[] }[];
    areas?: (Rect & { name: string })[];
  };
};

const [input = 'worlds.json', dir = 'maps', ...more] = process.argv.slice(2);
const worlds = [input, ...more].flatMap((f) => JSON.parse(readFileSync(f, 'utf8')) as { kind: string; seed: number; name: string; town: Town }[]);
mkdirSync(dir, { recursive: true });
const rect = (r: Rect, style: string) => `<rect x="${r.x}" y="${r.z}" width="${r.w}" height="${r.d}" ${style}/>`;
const line = (pts: Pt[]) => pts.map((p) => p.join(',')).join(' ');
const ZONE: Record<string, string> = { water: '#7fb2d9', woods: '#5d8a46', hill: '#b7c48a', meadow: '#c3d98f', fields: '#e2c86b', orchard: '#7fa653' };
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;');
const cards: string[] = [];
for (const { kind, seed, name, town } of worlds) {
  // 1.5 px a metre, labels in pixels whatever the scale.
  const scale = 1.5;
  const px = (n: number) => n / scale;
  // Only the ground floor (the storeys above and the basement are on rows further down).
  const ground = (z: number) => z <= town.depth;
  const roads = town.meta?.roads ?? [];
  const exits = (town.exits ?? []).map((e) => (Array.isArray(e) ? { x: e[0], z: e[1], kind: 'road' } : e));
  const order = ['meadow', 'fields', 'hill', 'woods', 'orchard', 'water'];
  const zones = [...(town.meta?.zones ?? [])].sort((a, b) => order.indexOf(a.kind) - order.indexOf(b.kind));
  const parts = [
    `<rect width="${town.width}" height="${town.depth}" fill="#9fc27a"/>`,
    ...zones.map((z) => `<polygon points="${line(z.poly)}" fill="${ZONE[z.kind] ?? '#999'}" fill-rule="evenodd" opacity="${z.kind === 'hill' ? 0.6 : 0.85}"/>`),
    ...(town.meta?.areas ?? []).map((a) => rect(a, `fill="none" stroke="#4d6b33" stroke-width="${px(1)}" stroke-dasharray="${px(4)} ${px(4)}"`)),
    ...(town.meta?.world?.fields ?? []).map((r) => rect(r, 'fill="#e2c86b"')),
    ...(town.meta?.greens ?? []).map((g) => `<polygon points="${line(g)}" fill="#7fb35e"/>`),
    ...town.plots.map((p) => rect(p, `fill="${p.kind === 'countryside' ? '#f2b366' : p.public ? '#a9c7e8' : '#b9d98f'}" stroke="#4d6b33" stroke-width="0.3"`)),
    // The side each lot faces (its street's).
    ...town.plots.map((p, i) => {
      const f = town.meta?.facing?.[i];
      const [x, z] = f === 0 ? [p.x + p.w / 2, p.z] : f === 1 ? [p.x, p.z + p.d / 2] : f === 2 ? [p.x + p.w / 2, p.z + p.d] : f === 3 ? [p.x + p.w, p.z + p.d / 2] : [NaN, NaN];
      return Number.isNaN(x) ? '' : `<circle cx="${x}" cy="${z}" r="0.9" fill="#4d6b33"/>`;
    }),
    ...(town.meta?.streets ?? []).map((r) => rect(r, 'fill="#6b6b6b"')),
    // Roads by kind: streets and lanes grey with their sidewalks, country roads brown, trails dashed.
    ...roads.filter((r) => r.kind !== 'trail').map((r) => `<polyline points="${line(r.points)}" fill="none" stroke="${r.kind === 'country' ? '#cdbb98' : '#d9d4c8'}" stroke-width="${r.width}" stroke-linejoin="round"/>`),
    ...roads.filter((r) => r.kind !== 'trail').map((r) => `<polyline points="${line(r.points)}" fill="none" stroke="${r.kind === 'country' ? '#8a7350' : '#6b6b6b'}" stroke-width="${r.width - 2.7}" stroke-linejoin="round"/>`),
    ...roads.filter((r) => r.kind === 'trail').map((r) => `<polyline points="${line(r.points)}" fill="none" stroke="#7a5a32" stroke-width="${Math.max(r.width * 0.6, px(1.6))}" stroke-dasharray="${px(5)} ${px(3)}" stroke-linecap="round"/>`),
    ...(town.meta?.paths ?? []).map((r) => rect(r, 'fill="#d8d2c4"')),
    ...town.objects.filter((o) => ground(o.z)).map((o) => `<circle cx="${o.x + 0.5}" cy="${o.z + 0.5}" r="0.35" fill="#5a4a3a"/>`),
    ...town.walls.filter(([, z0, , z1]) => ground(z0) && ground(z1)).map(([x0, z0, x1, z1]) => `<line x1="${x0}" y1="${z0}" x2="${x1}" y2="${z1}" stroke="#7a2e1f" stroke-width="0.35"/>`),
    ...(town.parking ?? []).map((p) => `<rect x="${p.x - 1.5}" y="${p.z - 1.5}" width="3" height="3" fill="#3f6fb5"/>`),
    ...exits.map((e) => `<circle cx="${e.x}" cy="${e.z}" r="${Math.max(1.5, px(5))}" fill="#d33"/>`),
    // Outdoor places' and areas' names.
    ...town.plots.filter((p) => p.kind === 'countryside').map((p) => `<text x="${p.x + p.w / 2}" y="${p.z - px(4)}" font-size="${px(11)}" text-anchor="middle" font-family="sans-serif" fill="#5a3a12">${esc(p.name)}</text>`),
    ...(town.meta?.areas ?? []).map((a) => `<text x="${a.x + a.w / 2}" y="${a.z + a.d / 2}" font-size="${px(18)}" font-weight="700" text-anchor="middle" font-family="sans-serif" fill="#1d2230" stroke="#fff" stroke-width="${px(3)}" paint-order="stroke">${esc(a.name)}</text>`),
    `<text x="${px(8)}" y="${px(22)}" font-size="${px(16)}" font-family="sans-serif">${kind} ${seed}: ${esc(name)} (${town.width}×${town.depth} m)</text>`,
  ];
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${town.width} ${town.depth}" width="${Math.round(town.width * scale)}" height="${Math.round(town.depth * scale)}">${parts.join('')}</svg>`;
  const file = `${kind}-${seed}.svg`;
  writeFileSync(`${dir}/${file}`, svg);
  cards.push(`<figure><a href="${file}"><img src="${file}" loading="lazy" alt="${esc(name)}"></a><figcaption>${kind} ${seed}: ${esc(name)}, ${town.width}×${town.depth} m, ${town.plots.length} plots</figcaption></figure>`);
}
writeFileSync(
  `${dir}/index.html`,
  `<!doctype html><meta charset="utf-8"><title>Sample worlds</title><style>body{font:13px sans-serif;margin:16px;background:#f4f1ec}main{display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:12px}figure{margin:0;background:#fff;padding:6px;border-radius:6px}img{width:100%;height:auto;display:block}</style><h1>Sample worlds (${worlds.length})</h1><main>${cards.join('')}</main>`,
);
console.log(`${worlds.length} maps → ${dir} (index.html)`);
