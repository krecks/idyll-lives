/**
 * The neighbourhood around the viewed lot, as a lived-in street: the neighbour houses
 * (`HouseBuilder.silhouettes`, with porches and garages), sidewalks with kerbs turning
 * the junctions' corners, the junctions' stop lines and crosswalks (`roads.ts`), streetlights that light up at night (with pools of light on the
 * ground), mailboxes, hydrants and bins along the kerb, cars parked on neighbours' driveways,
 * the fences and hedges round gardens and fields, and flower beds along their house
 * fronts, with pots by the door and little lights along the garden path; the things standing in
 * their gardens. The town's public places seen from another lot: shops as silhouettes with an awning,
 * a lot with a look (`WorldMeta.lotLooks`: a school, a church, a shop on the high street, a flats building) closed as
 * the building its look says, as tall as its real storeys (`buildingDress.ts`, its door where its own is; open, the
 * game draws its real walls), and the park,
 * playground and market things standing in the open. At the ends of the street a taxi
 * rank and street signs; benches, planters and bins along the kerb in front of the public places.
 *
 * Rules:
 * - Visual only, and kept off where Sims walk: front lawns (Sims cut across them on their way
 *   out of town), garden paths, driveway mouths, the road and the middle of the sidewalk. Kerb
 *   furniture stands on the road side of the sidewalk, cars park beside the houses.
 * - The things outdoors on other lots (neighbours' garden furniture and planters, the park's
 *   benches; not what stands indoors, in the silhouettes) in the game, placed, turned and moved off
 *   their tiles' middle as the lot draws them: the lawn is cleared under every lot's things
 *   (`lawnMask`). The menus' town (`pc/overview.ts`) draws every lot's things itself.
 * - Nothing on the viewed lot: the player builds there (the mailbox stands on the sidewalk). In
 *   the menus' town the player's lot (`WorldMeta.home`) gets its house as a shell dressed as the
 *   game draws it (its porch, no garden dressing), and nothing else. Its garage, if the
 *   town gave it one (`WorldStructure.garages`), its house's porches (the simulation's,
 *   `WorldStructure.porches`, as on every silhouette: `porchDoors`) and its porch pieces stand on it
 *   in the game too, drawn apart (`StreetPlan.lot`): they go with the lot's ground while its
 *   basement is in view, each porch with its wall in the cutaway. A house looks the same from the street as in the game:
 *   the lot's look on every storey, the roof the player chose.
 * - Fences: the world's (`shownFences`: the ones a town puts up round its gardens and fields,
 *   `game/gardenFences.ts`, the player's, and in an older town the ones it would have put up) on
 *   other lots, in their styles (`shared/fences.ts`), merged into each chunk's meshes, hedges with
 *   its plants; the game draws the focus lot's the same way, and every gate (they swing). The same
 *   fences whichever lot is in view.
 * - Cars: the world's (`WorldStructure.cars`), parked, in the menus' town; in the game they come
 *   and go, drawn by `pc/cars.ts`. A game saved before there were cars gets them by some neighbours'
 *   driveways (`parkCar`), parked for good.
 * - Static: generated once per town layout and kept across structure versions of the viewed
 *   lot (build-mode edits don't rebuild it), merged into a handful of meshes (one per material)
 *   plus thin-instanced foliage and props (manifest models: `model.street.*` and the town's
 *   things, see `assets/packs/street/manifest.json`), a few dozen draw calls for the whole street.
 * - Generated models (`.glb` files, downloaded like the rest), plus a CC0 concrete texture for the sidewalks.
 *
 * Engine-neutral: `planStreet` lays the street out and writes its geometry and its models'
 * instances (`StreetPlan`); the renderer's `Street` (`pc/street.ts`) draws it.
 */

import { type StandIn } from './modelEntry';

/** Vertex-colour alpha for the plain finish, as the material libraries' `encodeFinish`: [0, 0.5) dielectric, [0.5, 1] metal; roughness scaled within. */
export function encodeFinish(roughness: number, metallic: number): number {
  const r = Math.min(0.999, Math.max(0, roughness)) * 0.5;
  return metallic >= 0.5 ? 0.5 + r : r;
}

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** Paths at least this wide are driveways (garden paths are 1 m; see `game/town.ts`). */
export const DRIVEWAY_MIN_WIDTH = 1.8;
/** How far kerbside furniture reaches from where it stands, towards the road (m: a bench's depth, a lamp's base): on a 6 m lane it stands this far back from what cars pulling over keep clear (`PullOverStreetInfo.keep`). */
export const KERBSIDE_REACH = 0.3;

/** How far either side of a driveway's mouth a car coming or going swings across the sidewalk (m): no kerbside furniture there. */
export const CAR_SWING = 1.2;
/**
 * The taxi ranks at the rows' ends (`TownStreet.ranks`): one a street at most, none nearer another
 * than this (m), the town's own (a lot's `busStop`, the Market Square's) among them.
 */
export const RANK_APART = 60;
/** The rank's shelter: the content object it is, how far out past the row's last lot along the kerb, and how far behind the pavement its middle stands (m). */
export const RANK_OBJECT = 'busStop';
export const RANK_OUT = 4;
export const RANK_BACK = 0.62;
/** Half the shelter's length along the kerb (m, `model.busStop` 2.98 m), and the clear gap every kerbside thing keeps from it and its sign. */
export const RANK_HALF = 1.49;
export const RANK_GAP = 1.2;
/** How far along the kerb from a rank `rankKerb` reports what stands near it (m). */
export const RANK_SPAN = 10;
/** Half wall thickness (as in house.ts). */
export const T = 0.07;

export const MAILBOX = ['#22262A', '#2F4A3A', '#E8E6E0', '#8C2A26', '#2E4568'];
export const BIN = ['#3C6B3A', '#55595E', '#2E5C8A'];

/** Model keys of the thin-instanced foliage (the street and trees packs). */
export const FOLIAGE = { bush: 'model.bush', flowers: 'model.street.flowers', hedge: 'model.hedge' } as const;
export type FoliageKind = keyof typeof FOLIAGE;
/** Props drawn from manifest models (the street and town packs); a missing one is left out. */
export const PROP = {
  bench: 'model.street.bench',
  planter: 'model.street.planter',
  flowerTub: 'model.street.flowerTub',
  sign: 'model.street.sign',
  taxiSign: 'model.street.taxiSign',
  shelter: 'model.busStop',
  fountain: 'model.fountainTown',
  pot: 'model.street.pot',
  pathLight: 'model.street.pathLight',
  litterBin: 'model.litterBin',
} as const;
/** A thin instance: x, z, scale across, scale up, yaw, and how high it stands (a thing on a top; 0 if left out). */
export type Inst = [number, number, number, number, number, number?];
/** Shop awnings: stripe colours. */
export const AWNINGS: [string, string][] = [['#2F6B4F', '#F2F0EA'], ['#B8322B', '#F2F0EA'], ['#2E5C8A', '#F2F0EA'], ['#C98F2E', '#F7EBD3'], ['#6B3E5E', '#F2E6EE']];
/** What a placed object looks like and how big it is (the game's content), for the things on other lots. */
export type ObjectLook = (def: string) => { model: string; footprint: [number, number]; standIn: StandIn } | undefined;

export const overlaps = (a: Rect, b: Rect, m = 0) => a.x - m < b.x + b.w && a.x + a.w + m > b.x && a.z - m < b.z + b.d && a.z + a.d + m > b.z;

/*
 * A plot's dressing is laid out as if its street ran along x (`PlotLayout`'s coordinates): a plot
 * on a street along z is turned a quarter for it, the town's (x, z) at (z, -x), and its dressing
 * turned back onto the town.
 */
export const localRect = (turned: boolean, r: Rect): Rect => (turned ? { x: r.z, z: -(r.x + r.w), w: r.d, d: r.w } : r);
export const townRect = (turned: boolean, r: Rect): Rect => (turned ? { x: -(r.z + r.d), z: r.x, w: r.d, d: r.w } : r);
export const townPoint = (turned: boolean, x: number, z: number): [number, number] => (turned ? [-z, x] : [x, z]);
export const townYaw = (turned: boolean, yaw: number) => (turned ? yaw - Math.PI / 2 : yaw);
/** A box (x0, z0, x1, z1) as laid out, on the town. */
export const townBox = (turned: boolean, b: { x0: number; z0: number; x1: number; z1: number }) => {
  const r = townRect(turned, { x: b.x0, z: b.z0, w: b.x1 - b.x0, d: b.z1 - b.z0 });
  return { x0: r.x, z0: r.z, x1: r.x + r.w, z1: r.z + r.d };
};

/**
 * Writes posed primitives into a `Geo`: a local frame (origin on the ground, +z forward)
 * placed at x/z with a yaw, and a plain finish (colour + roughness/metalness in the vertex
 * alpha, see `encodeFinish`).
 */
