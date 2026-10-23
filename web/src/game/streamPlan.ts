/**
 * What a game session will download: first the files the viewed house needs to be shown (its
 * furniture, the garden, the residents, the textures), then the rest of the town and the catalog.
 * `stream.planStream` turns this into the HUD's progress; the loading screens fetch `home` now,
 * `reveal` streams `rest` in the background. The main menu already streams what doesn't depend on
 * a world (`gameDataUrls`).
 */

import type { AssetRegistry } from '../assets/registry';
import { optimizedUrl } from '../assets/registry';
import { withBuild } from '../assets/build';
import type { Content } from '../content/content';
import type { WorldStructure } from '../core/protocol';

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** Garden dressing `natureDecor` may pick (see `render/babylon/nature.ts`). */
const NATURE = [
  'model.tree',
  'model.tree.maple',
  'model.tree.blossom',
  'model.tree.fruit',
  'model.tree.birch',
  'model.tree.magnolia',
  'model.tree.japaneseMaple',
  'model.tree.willow',
  'model.bush',
  'model.bush.flowering',
  'model.bush.small',
  'model.hedge',
  'model.garden.hydrangea',
  'model.garden.rosebush',
  'model.garden.lavender',
  'model.garden.boxwood',
];

/** Street props (`render/babylon/street.ts`). */
const STREET = [
  'model.street.bench',
  'model.street.planter',
  'model.street.flowerTub',
  'model.street.sign',
  'model.street.busSign',
  'model.busStop',
  'model.street.pot',
  'model.street.pathLight',
  'model.litterBin',
  'model.street.flowers',
];

function inView(o: { x: number; z: number; w: number; d: number }, view: Rect | null): boolean {
  if (!view) return true;
  const x = o.x + o.w / 2;
  const z = o.z + o.d / 2;
  return x >= view.x && z >= view.z && x <= view.x + view.w && z <= view.z + view.d;
}

function modelUrl(assets: AssetRegistry, key: string): string | null {
  if (!assets.has(key, 'model')) return null;
  const entry = assets.get(key, 'model');
  return entry?.url ? optimizedUrl(entry.url) : null;
}

/** Icons and character sets the menus and character creation show (boot's first loading screen). */
export function menuAssetUrls(assets: AssetRegistry): string[] {
  const urls = new Set<string>();
  for (const key of assets.keys('icon')) {
    const url = assets.icon(key).url;
    if (url && !url.startsWith('data:')) urls.add(optimizedUrl(url));
  }
  for (const key of assets.keys('character')) {
    const url = assets.get(key, 'character')?.url;
    if (url) urls.add(optimizedUrl(url));
  }
  return [...urls];
}

/** Every game's files, whatever its world: surfaces, the garden, the street, the residents and the catalog's and held items' models. */
export function gameDataUrls(content: Content, assets: AssetRegistry): string[] {
  const { home, rest } = gameAssetPlan(content, assets, { objects: [] }, null);
  return [...home, ...rest];
}

/** The viewed house's files, and everything else this session may fetch. */
export function gameAssetPlan(
  content: Content,
  assets: AssetRegistry,
  world: Pick<WorldStructure, 'objects'>,
  view: Rect | null,
): { home: string[]; rest: string[] } {
  const home = new Set<string>();
  const all = new Set<string>();

  const add = (url: string | null, set: Set<string> = all) => {
    if (url) set.add(url);
  };
  const addHome = (url: string | null) => {
    add(url);
    add(url, home);
  };
  const addModel = (key: string) => {
    addHome(modelUrl(assets, key));
  };

  // Surfaces (walls, floors, furniture finishes) and the residents. Normal and ARM maps sit
  // next to the albedo and are only made absolute here, as `materials.ts` does.
  for (const key of assets.keys('material')) {
    const entry = assets.get(key, 'material');
    if (!entry?.texture) continue;
    for (const url of [entry.texture, entry.normal && new URL(entry.normal, entry.texture).href, entry.arm && new URL(entry.arm, entry.texture).href]) {
      if (!url) continue;
      const file = optimizedUrl(withBuild(url));
      all.add(file);
      home.add(file);
    }
  }
  for (const key of NATURE) addModel(key);
  for (const key of STREET) addModel(key);
  // The rig and every character set are `character` entries, not models (`model.sim` is one too).
  for (const key of assets.keys('character')) addHome(assets.get(key, 'character')?.url ?? null);

  // Furniture in the viewed lot (the rest of the town streams after Play).
  for (const o of world.objects) {
    const url = modelUrl(assets, content.object(o.def)?.model ?? `model.${o.def}`);
    if (!url) continue;
    all.add(url);
    if (inView(o, view)) home.add(url);
  }

  // The catalog's pictures are built from these models after the stream settles, and cooking
  // brings held items in from theirs.
  for (const def of content.shop) add(modelUrl(assets, def.model));
  for (const item of content.items) if (item.model) add(modelUrl(assets, item.model));

  return { home: [...home], rest: [...all].filter((url) => !home.has(url)) };
}
