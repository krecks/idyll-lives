/**
 * Draws the sample worlds (`tools/worlds/sample.ts`'s output) from above, one SVG each, to see
 * how a kind of world is laid out: its streets, the lots (public ones blue, farms' fields
 * yellow), garden paths and driveways, the ground floor's walls and the things standing about.
 *
 * Run: bun tools/worlds/map.ts worlds.json out-dir
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

type Rect = { x: number; z: number; w: number; d: number };
type Town = {
  width: number;
  depth: number;
  walls: [number, number, number, number][];
  objects: { x: number; z: number }[];
  plots: (Rect & { name: string; public: boolean })[];
  exits?: [number, number][];
  meta?: { streets?: Rect[]; paths?: Rect[]; world?: { fields?: Rect[] } };
};

const [input = 'worlds.json', dir = 'maps'] = process.argv.slice(2);
const worlds = JSON.parse(readFileSync(input, 'utf8')) as { kind: string; seed: number; name: string; town: Town }[];
mkdirSync(dir, { recursive: true });
const rect = (r: Rect, style: string) => `<rect x="${r.x}" y="${r.z}" width="${r.w}" height="${r.d}" ${style}/>`;
for (const { kind, seed, name, town } of worlds) {
  // Only the ground floor (the storeys above and the basement are on rows further down).
  const ground = (z: number) => z <= town.depth;
  const parts = [
    `<rect width="${town.width}" height="${town.depth}" fill="#9fc27a"/>`,
    ...(town.meta?.world?.fields ?? []).map((r) => rect(r, 'fill="#e2c86b"')),
    ...town.plots.map((p) => rect(p, `fill="${p.public ? '#a9c7e8' : '#b9d98f'}" stroke="#4d6b33" stroke-width="0.3"`)),
    ...(town.meta?.streets ?? []).map((r) => rect(r, 'fill="#6b6b6b"')),
    ...(town.meta?.paths ?? []).map((r) => rect(r, 'fill="#d8d2c4"')),
    ...town.objects.filter((o) => ground(o.z)).map((o) => `<circle cx="${o.x + 0.5}" cy="${o.z + 0.5}" r="0.35" fill="#5a4a3a"/>`),
    ...town.walls.filter(([, z0, , z1]) => ground(z0) && ground(z1)).map(([x0, z0, x1, z1]) => `<line x1="${x0}" y1="${z0}" x2="${x1}" y2="${z1}" stroke="#7a2e1f" stroke-width="0.35"/>`),
    ...(town.exits ?? []).map(([x, z]) => `<circle cx="${x}" cy="${z}" r="1.5" fill="#d33"/>`),
    `<text x="2" y="5" font-size="4" font-family="sans-serif">${kind} ${seed}: ${name} (${town.width}×${town.depth} m)</text>`,
  ];
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${town.width} ${town.depth}" width="${town.width * 5}" height="${town.depth * 5}">${parts.join('')}</svg>`;
  writeFileSync(`${dir}/${kind}-${seed}.svg`, svg);
}
console.log(`${worlds.length} maps → ${dir}`);
