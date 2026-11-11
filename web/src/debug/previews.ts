/**
 * Dev-only previews bench (`bun run dev`, then /previews.html): a fixed set of the household
 * creator's portraits and the catalog's thumbnails, drawn by the game's own preview engine
 * (`render/preview`), side by side on one page, to check them without a game and to compare them
 * pixel for pixel between builds (`tools/bench/render.mjs --shots … --pages previews`). Not part
 * of the build.
 *
 * The residents are made from seeds (`lookFromSeed`, the same look every time); the objects are
 * the first `?count=12` objects of the shop in id order, or those `?ids=tree,garden.apple` names. `?renderer=webgl` draws with
 * WebGL2. Sets `window.__ready` once every picture has loaded. `?stage=male|female` shows the household creator's
 * stage alone instead, over the whole page, with the first of those residents on it (`?zoom=1`: its face view;
 * `?bald`; `?hd=0`: drawn with the body, where the stage draws the close-up body). `?debug` exposes the residents'
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
const ids = params.get('ids')?.split(',');
const objects = content.shop
  .filter((o) => assets.has(o.model, 'model') && (!ids || ids.includes(o.id)))
  .toSorted((a, b) => (a.id < b.id ? -1 : 1))
  .slice(0, ids ? ids.length : count);

const sims = createSimPreviews({ assets, emotions: content.emotions.map((e) => e.id), backend: () => backend, closeUp: params.get('hd') !== '0' });
const items = createItemPreviews(assets);

const staged = params.get('stage');
if (staged) {
  const gender = staged === 'female' ? 'female' : 'male';
  const host = document.createElement('div');
  host.style.cssText = 'position:fixed;inset:0;background:#9cc3e6';
  document.body.append(host);
  const stage = await sims.stage(host);
  if (!stage) throw new Error('The stage is unavailable');
  stage.show({ gender, appearance: { ...lookFromSeed(assets, gender, 7919), ...(params.has('bald') ? { hairStyle: 'none' as const } : {}) } });
  stage.zoomTo(Number(params.get('zoom') ?? 0));
  // (The framing eases in, and the close-up body comes a moment after the body.)
  await new Promise((resolve) => setTimeout(resolve, 4000));
  Object.assign(window, { __ready: true });
}

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

if (!staged) await Promise.all([
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
