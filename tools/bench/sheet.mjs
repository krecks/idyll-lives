/**
 * Pictures of the same name from several folders tiled into one sheet each, every picture under its folder's
 * name: before and after a change, two looks, a few variants of one setting (`looks.mjs`, `play.mjs`).
 * Two go side by side, three or four two by two, more three to a row.
 */

import { mkdirSync, readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const LABEL = 44;
const BACK = '#1b1d21';

/** Tiles the PNGs every folder of `dirs` has, in their order, under `names`; returns how many sheets. */
export async function tile(dirs, names, out) {
  const sharp = (await import(createRequire(join(ROOT, 'web/package.json')).resolve('sharp'))).default;
  mkdirSync(out, { recursive: true });
  const others = dirs.slice(1).map((d) => new Set(readdirSync(d)));
  const shared = readdirSync(dirs[0]).filter((f) => f.endsWith('.png') && others.every((s) => s.has(f))).sort();
  const cols = dirs.length <= 2 ? dirs.length : dirs.length <= 4 ? 2 : 3;
  const rows = Math.ceil(dirs.length / cols);
  for (const name of shared) {
    const pictures = dirs.map((d) => sharp(join(d, name)));
    const sizes = await Promise.all(pictures.map((p) => p.metadata()));
    const w = Math.max(...sizes.map((m) => m.width));
    const h = Math.max(...sizes.map((m) => m.height));
    const cell = (i) => ({ left: (i % cols) * w, top: Math.floor(i / cols) * (h + LABEL) });
    const texts = names.map((label, i) => `<text x="${cell(i).left + w / 2}" y="${cell(i).top + LABEL - 13}" font-family="Helvetica, Arial, sans-serif" font-size="26" fill="#fff" text-anchor="middle">${String(label).replace(/[<&]/g, '')}</text>`);
    const labels = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${cols * w}" height="${rows * (h + LABEL)}">${texts.join('')}</svg>`);
    await sharp({ create: { width: cols * w, height: rows * (h + LABEL), channels: 3, background: BACK } })
      .composite([
        ...(await Promise.all(pictures.map(async (p, i) => ({ input: await p.toBuffer(), left: cell(i).left, top: cell(i).top + LABEL })))),
        { input: labels, left: 0, top: 0 },
      ])
      .png()
      .toFile(join(out, name));
  }
  return shared.length;
}
