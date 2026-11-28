/**
 * The bench's scene recipe: the neighbours and the household a scene's new world gets, and the home they move into.
 * `tools/bench/scene.mjs` runs it in a dev page and `tools/bench/state.mjs --scene` without a browser, so the same
 * place, options and seed give the same town, household and home in both.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import { randomHousehold, type HouseholdDraft } from '../game/household';
import { mulberry32 } from '../game/noise';
import { populate } from '../game/people';
import type { NeighbourhoodDraft, Templates } from '../game/town';
import { homesForSale, type HomeOffer, type HomeRef } from '../ui/homes';

/** Where the household moves in: the first house for sale, one in the main town, or a flat. */
type SceneHome = 'first' | 'town' | 'flat';

/** `make` with `Math.random` drawn from `seed` while it runs (as it is, with no seed). */
function seeded<T>(seed: number | null, make: () => T): T {
  if (seed === null) return make();
  const dice = Math.random;
  Math.random = mulberry32(seed ^ 0x9e3779b9);
  try {
    return make();
  } finally {
    Math.random = dice;
  }
}

/**
 * The neighbours of `empty` (a world just generated) and a household of four. With a seed they are the same every
 * time: their dice are the seed's while they are made, the household's its own, however many the neighbours took.
 */
export function sceneTown(content: Content, assets: AssetRegistry, t: Templates, empty: NeighbourhoodDraft, seed: number | null): { town: NeighbourhoodDraft; household: HouseholdDraft } {
  const town = seeded(seed, () => populate(content, assets, t, empty));
  const household = seeded(seed === null ? null : seed + 1, () => randomHousehold(content, assets, 4, town.households.map((h) => h.household.name)));
  return { town, household };
}

/**
 * The home for sale the household moves into: the first house for sale (`first`), the one halfway out among the
 * houses for sale in the main town (`town`), or the flat for sale nearest the main town's middle, upstairs if one is
 * (`flat`; throws when there is none). With its reference as a start request takes it.
 */
export function sceneHome(content: Content, t: Templates, town: NeighbourhoodDraft, kind: SceneHome): { home: HomeOffer; ref: HomeRef } {
  const vacant = homesForSale(content, t, town);
  const houses = vacant.filter((h) => h.flat === undefined);
  let home = houses[0] ?? vacant[0];
  const main = town.open?.areas?.[0];
  const [cx, cz] = main ? [main.x + main.w / 2, main.z + main.d / 2] : [town.width / 2, town.depth / 2];
  const far = (h: HomeOffer) => Math.hypot(h.lot.x - cx, h.lot.z - cz);
  if (kind === 'town') {
    const inMain = houses.filter((h) => (h.lot.area ?? 0) === 0).sort((a, b) => far(a) - far(b) || a.slot - b.slot);
    if (inMain.length) home = inMain[Math.floor(inMain.length / 2)];
  } else if (kind === 'flat') {
    const flats = vacant.filter((h) => h.flat !== undefined).sort((a, b) => Number((b.storey ?? 0) > 0) - Number((a.storey ?? 0) > 0) || far(a) - far(b) || a.key.localeCompare(b.key));
    if (!flats.length) throw new Error('no flat for sale in this world');
    home = flats[0];
  }
  return { home, ref: home.flat === undefined ? { slot: home.slot } : { slot: home.slot, flat: home.flat } };
}
