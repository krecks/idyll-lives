/**
 * A big world, laid out from its `WorldSpec` (`world.ts`): its land (896 to 1024 m of it in the
 * classic worlds) with a main town (the spec's shape, its streets reaching all of its box), its
 * hamlets (five to seven in the classic worlds) 170 m or more from each other and spread over the
 * whole world, country roads between
 * them and out of the world with farmsteads and cottages along them, and countryside: woods in
 * copses, meadows, hills, water, farmland and orchards round the hamlets and farmsteads, and
 * outdoor places (a picnic meadow, a fishing jetty, a lookout, a campfire, an orchard, a hammock
 * grove) reached by trails from the nearest hamlet.
 *
 * A place's land comes first (`planLand`, `landDraft`: step 1), from streams of its own, with no
 * town on it: its size, its water, its mountains and the main town's site; its town is laid on it
 * after (step 2), the same way whether the player shaped it or not.
 *
 * A place's water may be a river across the world (its town on a bank, a bridge from the town's
 * waterfront street, one or two more where it runs straight; country roads keep to one bank but
 * over a bridge) or the sea round an island (causeways out over it are the world's ways out). A
 * bridge or a causeway is a straight country road whose deck over the water is `RoadPlan.bridge`;
 * the sim's walkers walk its deck dry (`deckOf`). A place's world carries its relief (`reliefOf`).
 *
 * The land is a cost grid of 8 m cells rolled from noise (how steep, how wooded, where the water
 * is); the areas go where it is flat and dry, far enough apart (a Poisson disc); country roads
 * leave an area by a street's end straight on or by a T square off a street, and are paths over
 * the grid that turn as a road does (`steer`: wide bends, never back on themselves), smoothed
 * into curves with no hairpins (`COUNTRY_TURN`); one whose path follows a road laid already shares
 * it, joining it and leaving it in a T, rather than running beside it (`alongExisting`); trails
 * are A* paths on a finer grid round the lots. One `Ground` covers the whole world, and each
 * area's lots go along its own roads, nearest its middle first (`town.ts` `LotPlacer`).
 */

import type { CarPark, LandSize, ParkingBay, Rle, TownArea, TownPlace, WorldMeta, WorldZone } from '../core/protocol';
import { type Centre } from './centre';
import { type Land as LandField } from './land';
import { type GentlePlane } from './landFollow';
import type { UseLayer } from './uses';
import { type Pt } from './roads';
import { type WorldKind } from './world';


export type Rand = () => number;

/** A big world's areas, land and parking, as the town file carries them (`assembleTown`). */
export interface OpenWorldDraft {
  /** Area 0 is the main town. */
  areas: TownArea[];
  zones: WorldZone[];
  /** The water's tiles (1 m, row-major over the world), run-length (`Rle`). */
  water: Rle;
  /** The height (m) each outdoor plot is levelled to, by plot index. */
  pads: Record<number, number>;
  /** The public parking's bays (`parking.ts`): where a car parked in each stands and faces, by area. */
  parking: ParkingBay[];
  /** The car parks the bays make up (`parking.ts` `carParkAt`): by the square, by the big places, pairs by the verge. */
  carParks?: CarPark[];
  /** The places the tags name, and the main town's centre (its outline and its square). */
  places?: TownPlace[];
  centre?: { area: number; poly: Pt[]; square?: Pt[] }[];
  /** Each centre's high street (`centre.ts` `Centre`), to narrow its outline again once lots are given up (`landFollow.ts`); not in the town file. */
  highStreets?: Centre[];
  /** The centres' squares (kept clear of lots: `withGarden`); not in the town file. */
  squares?: Pt[][];
  /** How high the land stands over the level the towns are built at (`reliefOf`); none in a classic world. */
  relief?: WorldMeta['relief'];
  /**
   * The land's height (m) per 8 m cell, read at the cells' middles, over the level the towns are
   * built at (`heightsOf`: below it too, unlike `relief`), row-major from the world's corner. What
   * a place's land as a height field is seeded from (`landFollow.ts` `withLand`); none in a classic
   * world or one laid again on a shaped land, and not in the town file.
   */
  heights?: { cell: number; cols: number; rows: number; h: Float32Array };
  /**
   * The land as a height field (`land.ts`), once the player has shaped it (step 1's brushes,
   * `landEdit.ts` `applyLand`) or the town was laid again on it (`generateFromSpec`'s `shaped`):
   * the town file carries it as `meta.land` once it is `edited`.
   */
  land?: LandField;
  /** How the place's land was planned (`LandPlan`), carried with the land so the town laid on it finds its water. */
  plan?: LandPlan;
  /** Where the main town was laid (the middle of its box, m): the plan's site, or where the player moved it, as the shore and the dry land let it. */
  site?: Pt;
  /**
   * The gentle ground the town was laid with, an area's plane each (`landFollow.ts` `GentlePlane`):
   * step 2's settles ease the land to them again, so an edit moves the land only round itself. Not
   * in the town file nor the world's code.
   */
  gentle?: GentlePlane[];
  /**
   * What the player painted the land to be used for in step 2 (`uses.ts`), while the new game is set
   * up: Build makes the town from it. Not in the town file nor the world's code.
   */
  uses?: UseLayer;
}

/**
 * A place's land before any town is laid on it, as its own seed plans it (`planLand`): its size,
 * the side its water is on (the main town's, as it faces it), the main town's site (the middle of
 * its box: on flat dry ground, by the shore), the level the towns are built at (the land's height
 * there, m: the land field's 0), the mountains' height, and its water: a river across it (where
 * it runs: its middle `c`, how it tilts, how far its meanders swing, their noise's offset `k`, and
 * the calm reach by the site), a lake, an island's coast, ponds. A town laid on a land whose water
 * is as planned uses these, so its bridges and causeways are as a generated world's.
 */
export interface LandPlan {
  v: 1;
  kind: WorldKind;
  seed: number;
  size: LandSize;
  side: 'zmin' | 'zmax';
  site: Pt;
  level: number;
  massif: number;
  river?: { c: number; tilt: number; amp: number; k: number; reach: [number, number] };
  lake?: { x: number; z: number; rx: number; rz: number };
  coast?: Pt[];
  ponds?: { x: number; z: number; r: number }[];
}

/**
 * How a town is laid on a land (`generateFromSpec`'s `lay`): from its own seed (`townSeed`; else
 * the land's), with its main town's box round `site` (its middle; else the plan's), on the land
 * `plan` planned (`landDraft`'s `open.plan`; none: a land from elsewhere, its town where it fits),
 * and `waterKept` when the land's water is as its plan has it (the town then uses the plan's river
 * for its bridges, and may go on either bank).
 */
export interface LayOptions {
  townSeed?: number;
  site?: Pt;
  waterKept?: boolean;
  plan?: LandPlan;
}

/**
 * A place's land, with no town on it (step 1): `kind` with `options` (the place's own where not
 * given) from `seed`, as `planLand` plans it and `landFollow.ts` `seedBare` seeds it: no lots,
 * roads, greens, places or areas; its land (`open.land`: heights, water, woods, meadows)
 * and its plan (`open.plan`), the water and zones from them, its hills' relief over its level. Its
 * name and tagline are the ones its town takes when laid from the same seed.
 */
