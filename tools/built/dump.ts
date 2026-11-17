/**
 * Dumps the built world's samples (`samples.ts`: houses near and far, the backdrop's buildings, a street, fences,
 * cars, each made by the game's own generators) for their gate: a `.glb` a sample and `built.json`, which lists them.
 *
 * Run: bun tools/built/dump.ts <out dir>, then cargo check-glb --built <out dir>
 */

import { mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { glbOf } from './glb';
import { index, samples } from './samples';

const out = process.argv[2];
if (!out) {
  console.error('usage: bun tools/built/dump.ts <out dir>');
  process.exit(2);
}
mkdirSync(out, { recursive: true });
// (A sample gone from the set leaves no file behind for the gate to find.)
for (const f of readdirSync(out)) if (f.endsWith('.glb') || f === 'built.json') rmSync(join(out, f));
const list = await samples();
let [triangles, bytes] = [0, 0];
for (const s of list) {
  const glb = glbOf(s.meshes);
  writeFileSync(join(out, `${s.key}.glb`), glb);
  bytes += glb.length;
  triangles += s.meshes.reduce((n, m) => n + m.indices.length / 3, 0);
}
writeFileSync(join(out, 'built.json'), index(list));
console.log(`${list.length} samples, ${triangles} triangles, ${(bytes / 1e6).toFixed(1)} MB -> ${out}`);
