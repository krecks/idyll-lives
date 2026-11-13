/**
 * Pictures of the same name from several folders tiled into one sheet each, every picture under its folder's
 * name: before and after a change, two looks, a few variants of one setting (`looks.mjs`, `play.mjs`).
 * Two go side by side, three or four two by two, more three to a row. The repo's own tool does it
 * (`cargo shots sheet`, crates/tools/src/shots.rs); this hands it the folders.
 */

import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));

/** Tiles the PNGs every folder of `dirs` has, in their order, under `names`; returns how many sheets. */
export async function tile(dirs, names, out) {
  const args = ['shots', 'sheet', '--out', out, ...names.flatMap((name) => ['--name', String(name)]), ...dirs];
  const run = spawnSync('cargo', args, { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] });
  if (run.status !== 0) throw new Error(`cargo shots sheet failed (${run.status})`);
  return Number(/^(\d+) sheets/m.exec(run.stdout)?.[1] ?? 0);
}
