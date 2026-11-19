/**
 * The town checks (`townCheck.ts` `townProblems`) on a town nobody lives in yet, straight from its
 * draft: the checks read a town file (`CheckedTown`, what `assembleTown` writes), and assembling one
 * means furnishing every house, far too slow after every edit. `checkedTown` writes the part of a
 * town file the checks read as `assembleTown` would write it (the lots' rectangles, kinds and
 * facings, the roads, the ways out, the water, the parking, the zones, the backdrop, the places and
 * the centre), with nobody living there. `editedProblems` is what step 2's `'edit'` job checks.
 */

import type { Content } from '../../content/content';
import { wetTile } from '../landEdit';
import { roadExits } from '../roads';
import { plotRect, type NeighbourhoodDraft, type Templates } from '../town';
import { townProblems, type CheckedTown } from '../townCheck';

/** The checks of how the generator fills a town (the backdrop each area keeps, the centre the densest), which an edited town needn't keep. */
const GENERATED_ONLY: readonly RegExp[] = [/ has \d+ backdrop buildings \(\d+ or more\)$/, /^the centre's open land is /, /^the main town has no centre$/];

/** The town file's kind of a lot (`assembleTown`'s `PlotKind`). */
const plotKind = (s: NeighbourhoodDraft['slots'][number]): string => (s.kind === 'house' ? (s.lot === 'farm' ? 'farm' : 'home') : s.kind === 'shop' ? 'shop' : s.kind === 'outdoor' ? 'countryside' : 'public');

/** The part of `draft`'s town file the checks read (`CheckedTown`), written as `assembleTown` writes it, with nobody living there yet. */
export function checkedTown(draft: NeighbourhoodDraft, t: Templates): CheckedTown {
  const open = draft.open;
  const W = draft.width;
  const D = draft.depth;
  const land = open?.land;
  const plots = draft.slots.map((s) => ({ ...plotRect(t, s), public: s.kind !== 'house', area: s.area ?? 0, kind: plotKind(s) }));
  // (A place's ways out are where its roads, not its footpaths, leave the world; a classic town's, any road's.)
  const exits = open ? roadExits(draft.roads.filter((r) => r.kind !== 'trail'), W, D).map(([x, z]) => ({ x, z, kind: 'road' as const })) : roadExits(draft.roads, W, D);
  return {
    size: 'big',
    width: W,
    depth: D,
    plots,
    exits,
    sims: [],
    households: [],
    roads: draft.roads,
    ...(open ? { areas: open.areas, terrain: { water: open.water }, parking: open.parking, carParks: open.carParks } : {}),
    // (The water as the land has it, which the Street tool bridges: `CheckedTown.landWet`.)
    ...(land ? { landWet: (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && wetTile(land, Math.floor(x), Math.floor(z)) } : {}),
    meta: {
      roads: draft.roads,
      greens: draft.greens,
      facing: draft.slots.map((s) => s.facing),
      ...(open
        ? {
            zones: open.zones,
            ...(open.backdrop ? { backdrop: open.backdrop } : {}),
            ...(open.places ? { places: open.places } : {}),
            ...(open.centre ? { centre: open.centre } : {}),
          }
        : {}),
    },
  };
}

/**
 * What is wrong with an edited town (`townProblems`, `hard` and `soft`): its layout checked as every
 * generated world's is, but nothing asked of how filled it is (its homes, its built land, its
 * quarters, the backdrop each area keeps, its centre: the player's to choose), only that a town or a
 * city has somewhere for every career to work; and fewer than two homes is hard.
 */
export function editedProblems(draft: NeighbourhoodDraft, t: Templates, content: Content): { hard: string[]; soft: string[] } {
  const size = draft.world?.options?.town;
  const expect = { built: 0, empty: null, meadow: false, quarters: 0, homes: 0, mainLeast: 0, careers: size === 'town' || size === 'city', exits: null };
  const found = townProblems(checkedTown(draft, t), expect, { river: false, careers: content.careerCategories.map((c) => c.id), generated: false });
  const homes = draft.slots.filter((s) => s.kind === 'house').length;
  // (How the generator fills a town, which an edited one needn't keep: a village placed by hand has no backdrop, decision 15.)
  const hard = found.hard.filter((line) => !GENERATED_ONLY.some((p) => p.test(line)));
  return { hard: homes < 2 ? [...hard, `fewer than two homes (${homes})`] : hard, soft: found.soft };
}
