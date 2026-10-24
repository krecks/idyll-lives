/**
 * Dev-only previews bench (`bun run dev`, then /previews.html): a fixed set of the household
 * creator's portraits and the catalog's thumbnails, drawn by the game's own preview engine
 * (`render/preview`), side by side on one page, to check them without a game and to compare them
 * pixel for pixel between builds (`tools/bench/render.mjs --shots … --pages previews`). Not part
 * of the build.
 *
 * The residents are made from seeds (`lookFromSeed`, the same look every time); the objects are
 * the first `?count=12` objects of the shop in id order. `?renderer=webgl` draws with
 * WebGL2. Sets `window.__ready` once every picture has loaded. `?debug` exposes the residents'
 * previews as `__previews` (`render/preview/index.ts`): the household creator's stage can be
 * attached from a script (`__previews.stage(host)`) and its studio inspected (`__previews.live`).
 * Both preview apps are PlayCanvas apps of their own (`pc/device.ts` `createSideApp`).
 */

import { AssetRegistry } from '../assets/registry';
import { Content } from '../content/content';
import { lookFromSeed } from '../game/household';
import { createSimPreviews } from '../render/preview';
import { createItemPreviews } from '../render/preview/items';

const params = new URLSearchParams(location.search);
const base = import.meta.env.BASE_URL;
const [content, assets] = await Promise.all([Content.load(`${base}content/base.json`), AssetRegistry.load(`${base}assets/manifest.json`)]);
const backend = params.get('renderer') === 'webgl' ? 'webgl' : 'auto';

/** Men and women in turn, from fixed seeds. */
const looks = [1, 2, 3, 4, 5, 6, 7, 8].map((seed) => {
  const gender = seed % 2 ? 'male' : 'female';
  return { gender, appearance: lookFromSeed(assets, gender, seed * 7919) };
});
const count = Number(params.get('count') ?? 12);
const objects = content.shop
  .filter((o) => assets.has(o.model, 'model'))
  .toSorted((a, b) => (a.id < b.id ? -1 : 1))
  .slice(0, count);

const sims = createSimPreviews({ assets, emotions: content.emotions.map((e) => e.id), backend: () => backend });
const items = createItemPreviews(assets);

/** A row of pictures under a heading; each resolves to an `<img>` once loaded (or a gap). */
function row(title: string, pictures: Promise<{ url: string | null; label: string }>[]): Promise<void>[] {
  const section = document.createElement('section');
  section.innerHTML = `<h2>${title}</h2>`;
  const grid = document.createElement('div');
  section.append(grid);
  document.body.append(section);
  return pictures.map(async (picture) => {
    const cell = document.createElement('figure');
    grid.append(cell);
    const { url, label } = await picture;
    cell.innerHTML = `<figcaption>${label}</figcaption>`;
    if (!url) return;
    const img = new Image();
    img.src = url;
    cell.prepend(img);
    await img.decode();
  });
}

await Promise.all([
  ...row(
    'Portraits',
    looks.map(async (look, i) => ({ url: await sims.portrait(look), label: `${look.gender} ${i + 1}` })),
  ),
  ...row(
    'Thumbnails',
    objects.map(async (o) => ({ url: await items.thumbnail(o.model, o.footprint ?? [1, 1], 'now'), label: o.id })),
  ),
]);

Object.assign(window, { __ready: true });
