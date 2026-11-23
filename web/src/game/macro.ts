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

import type { Content } from '../content/content';
import type { CarPark, LandSize, ParkingBay, Rle, TownArea, TownPlace, TownSize, WorldMeta, WorldOptions, WorldZone } from '../core/protocol';
import { distanceToRect } from '../render/shared/chunks';
import { areaSize, civicTooBig, CIVIC, namerFor, ZONE, type ZoneKind } from './places';
import { chooseCentre, densestCentre, type Centre } from './centre';
import { tilesOf } from './footprints';
import { designs, DensePass, NEAR_CORNER, pooled, raisePlaces, terraceUnits, townPlaces } from './dense';
import { besidePlace, bySquare, clearOfDrives, driveLines, LOOSE_BEND, parkAlong, vergeParking, type CarParkLaid } from './parking';
import { COVER, COVER_CELL, coverSpan, LAND_CELL, NONE, type Land as LandField } from './land';
import { APRON, landZones, Protection, wetTiles, type BuiltLand } from './landEdit';
import { bridgeSites, dryPieces, type Bounds } from './landPieces';
import { GENTLE_TILT, seedBare, smoothHeights, type GentlePlane } from './landFollow';
import { outlines, rasterize, rle, simplify } from './landMasks';
import { between, fbm, mulberry32 } from './noise';
import type { UseLayer } from './uses';
import { along, astar, CELL, Cells, clearOfBoxes, COUNTRY_GRADE, COUNTRY_WIDTH, CountryRouter, CTRL_APART, FACING, ROAD_WIDTH, ROUTE_GRADE, SIDE_COST, TRIES, type Port, type Way } from './countryRouter';
import { centroid, cropRoads, curve, evenly, inside, keepOf, leftOf, lengths, nearest, pointAt, RoadIndex, roadIndex, roadLengths, roadMeets, straightAround, type Box, type Pt, type RoadPlan } from './roads';
import { lotName, placeOfTemplate, placeUnit } from './placeLots';
import { BIG_COURTS, boundsOf, COURTS, frontageOf, Ground, houseFor, joinUp, lotAt, LotPlacer, plotRect, reachesRoad, settleRoads, shuffled, slotDims, turner, wantedLots, type Facing, type Frontage, type NeighbourhoodDraft, type PlotSlot, type PublicTemplate, type Rect, type Templates, type Unplaced } from './town';
import { LAND_SIZE, LEGACY_KINDS, placeOf, specOf, streetNames, worldName, worldTagline, type RoadLayout, type SizeSpec, type WorldKind, type WorldSpec, type WorldStep } from './world';
import * as pmath from './pmath';

type Rand = () => number;

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
export function landDraft(content: Content, _t: Templates, kind: string, options: Partial<WorldOptions> | undefined, seed: number): NeighbourhoodDraft {
  const chosen = { ...options };
  if (Object.hasOwn(LEGACY_KINDS, kind)) chosen.town ??= LEGACY_KINDS[kind].town;
  return bareDraft(content, specOf(placeOf(kind), chosen), seed);
}

/**
 * A town laid on `land` (`landDraft`'s, shaped by the player or not; its own land seed and plan):
 * its place with `options` over the land's own (its size is the land's), from `lay` (the town's
 * own seed, its site, whether the land's water is as planned), named `name` (else as the town's
 * seed names it); `step` told as `generateFromSpec` tells it.
 */
export function layOn(content: Content, t: Templates, land: NeighbourhoodDraft, options: Partial<WorldOptions>, lay: Omit<LayOptions, 'plan'>, name?: string, step?: (done: WorldStep) => void): NeighbourhoodDraft {
  const field = land.open?.land;
  if (!field) throw new Error('The land has no height field to lay a town on.');
  const kind = placeOf(land.world?.kind ?? 'country');
  const spec = specOf(kind, { ...land.world?.options, ...options, ...(land.open?.plan ? { size: land.open.plan.size } : {}) });
  return generateFromSpec(content, t, spec, land.seed ?? land.open?.plan?.seed ?? 1, name, step, field, { ...lay, ...(land.open?.plan ? { plan: land.open.plan } : {}) });
}

/** A place's plan for its land (see `LandPlan`): from its seed, the town and the villages left out of it. */
export function planLand(spec: WorldSpec, seed: number): LandPlan {
  return planned(spec, seed).plan;
}

/** Areas this far apart at least (their middles, m), and as far as this from the one they join; best this far from the nearest. */
export const AREA_APART = 170;
const AREA_REACH = 440;
const AREA_AIM = 230;
/** No area nearer the world's edge than this (m), nor nearer another's box. */
const EDGE = 50;
const BOX_GAP = 24;
/**
 * An area's road out of it ends this far inside the world's edge at least (m), where the country
 * road on from it starts: an area's box keeps `EDGE` off it, but a lane out of it can reach further
 * than that (a farm hamlet's fork, its box's depth long), past the edge, and a country road on
 * from beyond the edge is cut off from it when the roads are cut to the world.
 */
const PORT_IN = EDGE / 2;
/** Trails' width (m). */
export const TRAIL_WIDTH = 2;
/** Houses with no room in any area go along a country road, as far as this from their own area's box (m). */
const SPILL = 140;
/** The main town's terrace rows with no room in it go along the country roads leaving it, as far as this from its box (m). */
const TERRACE_SPILL = 110;
/** Along a country road: no farmstead nearer an area's box than this (m; how often one comes: `WorldSpec.farmEvery`). */
const FARM_OFF = 25;
/** Cottages by the country roads: so many where each leaves an area or meets another, within this of the spot (m). */
const COTTAGES: [number, number] = [2, 4];
const COTTAGE_REACH = 70;
/** Land steeper than this (rise per m) is no place for a farmstead or a cottage; in a place's world (its towns follow its slopes) nor for any lot out along a country road, within `SPILL_LOOK` m of where it goes. */
const STEEP = 0.25;
const PLACE_STEEP = 0.12;
const SPILL_LOOK = 12;
/** An orchard beside a farmstead: this deep from the lot's side (m), a gap of this from it. */
const ORCHARD = 26;
const ORCHARD_GAP = 3;
/**
 * A country road leaves an area by a port within `SWING` (radians) of the way it is going, else
 * swinging round costs it `SWING_COST` m a radian more; a T off a street facing within
 * `SIDE_FACING` of the way costs `SIDE_COST` m (`countryRouter.ts`) more than a port, and keeps
 * `SIDE_APART` m from the area's other roads where it leaves its street and `SIDE_DEPTH` m (a lot's
 * depth) as it crosses.
 */
const SWING = Math.PI / 6;
const SWING_COST = 120;
const SIDE_FACING = Math.PI / 4;
const SIDE_APART = 24;
const SIDE_DEPTH = 16;
/** How many of the likeliest ways out of each area (`waysOut`) a country road between two looks at. */
const WAYS = 5;
/** A country road between two areas goes straight on from a way out of one to a street of the other this near (m), where that street bends less than `STRAIGHT_BEND` (radians) over 28 m. */
const STRAIGHT_ON = 160;
const STRAIGHT_BEND = 0.6;
/** Two areas' ways that face each other within this (radians) of the line between them are joined by it (`headOn`). */
const HEAD_ON = Math.PI / 15;
/** An area's lots keep their middles this far along their road from where it meets another (m), and off where it turns more than `TIGHT_BEND` (radians) over as far (a bend of 20 m radius or less). */
export const CORNER = 18;
export const TIGHT_BEND = 0.9;
/** A home by a country road stands where it turns less than `BEND_MOST` (radians) over `BEND` m either side. */
export const BEND = 16;
export const BEND_MOST = 0.3;
/** A T leaves a street where it turns less than this (radians) over 14 m either side. */
const T_BEND = 0.3;
/**
 * Public parking spaces by the verge nearest an area's middle: at least so many in the main town and
 * in each village, and in a place's areas one for every so many homes if that is more
 * (`parkingFor`). With the town filled, the old four and two had nine spaces in ten taken for seven
 * or eight game hours a day; about twice as many should bring that to two, like a town a third
 * full. A classic world keeps the fixed count.
 */
const PARKING_MAIN = 4;
const PARKING_VILLAGE = 2;
const HOMES_PER_SPACE_MAIN = 25;
const HOMES_PER_SPACE_VILLAGE = 15;
/** A main town's car park by its square (bays, fewest and most), and one by each big place's lot (`BIG_PLACES`). */
export const SQUARE_BAYS: [number, number] = [6, 10];
export const PLACE_BAYS: [number, number] = [4, 6];
/** The places a car park is laid beside (their layouts): the school, the leisure and medical centres, the hotel, the town hall, the garden centre, the church. */
export const BIG_PLACES = new Set(['school', 'leisure', 'medical', 'hotel', 'townHall', 'gardenCentre', 'church']);
/** How far out from its heart a main town is built up before its houses go down (`DensePass.lay`), of the way to its box's corners. */
const INNER = 0.5;
/** The buildings of flats a main town of each size has on its centre's streets at most (`DensePass.centreFlats`). */
const CENTRE_FLATS: Record<TownSize, number> = { hamlet: 1, village: 2, town: 3, city: 4 };
/** How far past an area's box the country roads into it are built along (m: `DensePass.approaches`). */
const APPROACH = 180;

/** The public parking spaces an area with `homes` homes gets (`main`: the main town). */
export function parkingFor(homes: number, main: boolean): number {
  return main ? Math.max(PARKING_MAIN, Math.round(homes / HOMES_PER_SPACE_MAIN)) : Math.max(PARKING_VILLAGE, Math.round(homes / HOMES_PER_SPACE_VILLAGE));
}

const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
const dist = (a: Pt, b: Pt) => pmath.hypot(a[0] - b[0], a[1] - b[1]);
const round2 = (v: number) => Math.round(v * 100) / 100;

/** Among a place's mountains, a hamlet's box no higher or lower than the area it joins by more than this share of a country road's climb (`countryRouter.ts` `COUNTRY_GRADE`) over the gap between them; and how much a metre of height counts against a site there (the valley floor first). */
const REACHABLE = 0.7;
/** What two areas' gentle ground may differ by besides, where their boxes come nearest (m): half the most a road may be cut into its land or stand over it (`builtSurface.ts` `CUT_MAX`) on either side. */
const PLANE_SLACK = 2;
const MOUNTAIN_LOW = 0.3;
/** A road out of a place's world looks for the lowest of an edge this far (m) either way of straight out; a metre more than it may climb counts this much against that edge (m). */
const EDGE_WINDOW = 200;
const OVER_CLIMB = 40;
/** How far `p` is from rectangle `r` (0 inside it). */
const offRect = (r: Rect, [x, z]: Pt) => distanceToRect(x, z, r.x, r.z, r.w, r.d);

/** A `W` × `D` mask of tiles grown by `by` tiles every way (a square round each). */
export function grown(mask: Uint8Array, W: number, D: number, by: number): Uint8Array {
  const across = new Uint8Array(mask.length);
  for (let z = 0; z < D; z++) {
    for (let x = 0, last = -Infinity; x < W; x++) {
      if (mask[z * W + x]) last = x;
      // (The nearest set tile at or before x, within `by`; then the one after.)
      if (x - last <= by) across[z * W + x] = 1;
    }
    for (let x = W - 1, next = Infinity; x >= 0; x--) {
      if (mask[z * W + x]) next = x;
      if (next - x <= by) across[z * W + x] = 1;
    }
  }
  const out = new Uint8Array(mask.length);
  for (let x = 0; x < W; x++) {
    for (let z = 0, last = -Infinity; z < D; z++) {
      if (across[z * W + x]) last = z;
      if (z - last <= by) out[z * W + x] = 1;
    }
    for (let z = D - 1, next = Infinity; z >= 0; z--) {
      if (across[z * W + x]) next = z;
      if (next - z <= by) out[z * W + x] = 1;
    }
  }
  return out;
}

/** An area as it is laid out: its box on the world, its roads (indices into the world's), its middle. */
export interface AreaPlan {
  name: string;
  rect: Rect;
  centre: Pt;
  roads: number[];
  greens: Pt[][];
  spec: SizeSpec;
  /** Its ways on: road ends that led out of its box. */
  ports: Port[];
}

/** A stream of draws of its own for the massif's height (so a spec without one draws as it did). */
const MASSIF = 0x6d617373;
/** A stream of its own for how a place's land looks (`WorldSpec.landscape`): its palette and trees stay as the options change. */
const LOOK = 0x6c6f6f6b;
/** The mountains rise this far from the valley, the shore, the lake (m), past this much of it flat (m); the low-z side (the menus' camera's) to this share of their height. */
const MASSIF_RISE = 260;
const MASSIF_FLAT = 70;
const MASSIF_NEAR = 0.45;
/** The size of world (m) a lake is drawn for: in a smaller one (a place's), smaller in proportion. */
const LAKE_WORLD = 992;
/** A place's outdoor places no further than this from their area's box (m: the sample's check is 150). */
const SPOT_REACH = 130;
/** A place's lots keep this far from the water (m). */
export const SHORE_GAP = 3;
/** Streams of their own for a river's course and its bridges (a place's). */
const RIVER = 0x72697672;
/** Streams of their own for the named places' lots and for their names (so the rest of the town draws as it did). */
const PLACES = 0x706c6163;
const PLACE_NAMES = 0x6e616d65;
/** A named place's lot stands with the middle of its front no further than this from its road's edge (m): as far as a lot's path reaches across the verge to its road (`layoutTown`). */
const PLACE_REACH = 12;

/** Whether named place lot `u` at `at` (`lotAt`, facing `road`) has the middle of its front within `PLACE_REACH` m of the road's edge (it has no driveway: its path crosses the verge). */
export function placeReaches(t: Templates, road: RoadPlan, u: Unplaced, at: { x: number; z: number; facing: Facing }): boolean {
  const r = plotRect(t, { ...u, ...at });
  const [mx, mz] = ([[r.x + r.w / 2, r.z], [r.x, r.z + r.d / 2], [r.x + r.w / 2, r.z + r.d], [r.x + r.w, r.z + r.d / 2]] as const)[at.facing];
  return nearest(road.points, mx, mz).dist - road.width / 2 <= PLACE_REACH;
}
/** A farm shop goes along its farmstead's road within about this of the farm's middle (m). */
export const FARM_SHOP_REACH = 90;
/** A river's width (m), how far its meanders swing (m), how far its mountains stand back from it (m). */
const RIVER_WIDTH: [number, number] = [14, 22];
const MEANDER: [number, number] = [70, 110];
const RIVER_FLAT = 90;
/** An island's coast: a rounded square (`ISLAND` of the world's size from its middle to the coast along each axis), its points round it. */
const ISLAND = 0.44;
const COAST_POINTS = 180;
/** A bridge's ends stand this far inland of the banks (m); a causeway's inland end this far from the coast. Its deck: the water, and this much either side (m). */
export const BRIDGE_IN = 12;
const CAUSEWAY_IN = 30;
export const DECK_PAST = 2;
/** The longest a bridge's deck may be (m); more bridges over a river (few villages: one, many: two), this far from each other at least and from the areas' boxes (m). */
export const BRIDGE_SPAN = 45;
const BRIDGE_APART = 220;
const BRIDGE_CLEAR = 30;
/**
 * On a shaped land cut into pieces by its water (`landPieces.ts`): a bridge between two crosses this
 * much water at most (m: its deck, with `DECK_PAST` either side, within `BRIDGE_SPAN`), is chosen
 * from the best so many crossings to each piece, and keeps this far (m, their middles) from the others.
 */
const PIECE_SPAN = BRIDGE_SPAN - 2 * DECK_PAST - 1;
const PIECE_TRIES = 12;
const PIECE_BRIDGES_APART = 80;
/** The world's edges (north, east, south, west): which way is out over each, and their names. */
const EDGE_OUT: readonly Pt[] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
const EDGE_NAMES = ['North', 'East', 'South', 'West'];

/** How many, in words (the ways out the world wants at least). */
const WORDS = ['no', 'one', 'two', 'three', 'four'];

/**
 * Lays out a big world from `spec` and `seed` (the same spec and seed always give the same world).
 * See the top of this file. `step`: told as each part of it is done (it changes nothing).
 *
 * A place's town is always laid on a land (`shaped`, `land.ts`): step 1's (`landDraft`, with its
 * `lay.plan`), or one planned and seeded bare here from the same seed if none is given. The world
 * is the land's size, and its 8 m cells are filled from it (`fillFrom`: the mean height with the
 * finer hills smoothed out, wet if any sample is, wooded if most of it is); the place's own water
 * and mountains are not laid again. On a land with its plan, the main town goes round the plan's
 * site (or `lay.site`), against the shore if the shore is near, its shape turned over to face its
 * water (`mirrored`), and from its own seed (`lay.townSeed`; the land's noise and look stay the
 * land's). Where the land's water is as planned (`lay.waterKept`), the plan's river is bridged as
 * ever; else (and on a land with no plan: one the player shaped, laid again by the `world` job) the
 * water cuts the land into pieces of dry land (`landPieces.ts` `dryPieces`): the main town goes on
 * the piece under its site (else the largest that holds it), and, with villages, a village goes on
 * every other piece big enough that a bridge of `PIECE_SPAN` m could reach (`bridgeSites`), joined
 * to the town's by a country road over it; the pieces no bridge reaches are told of
 * (`diagnostics.pieces`). The draft carries a copy of the land (`open.land`; flat under what is built if it
 * was drawn so, its cover cleared there and round the town, its farmland on it: `townCover`), its
 * plan and its site, and its water and zones come from it. A classic world is laid as it always was.
 */
export function generateFromSpec(content: Content, t: Templates, spec: WorldSpec, seed: number, name?: string, step?: (done: WorldStep) => void, shaped?: LandField, lay?: LayOptions): NeighbourhoodDraft {
  // A place's town is always laid on a land (step 1's: `landDraft`): planned and seeded bare here if it was given none.
  if (!spec.classic && !shaped) {
    const bare = bareDraft(content, spec, seed);
    return generateFromSpec(content, t, spec, seed, name, step, bare.open!.land!, { ...lay, plan: bare.open!.plan!, waterKept: true });
  }
  const plan = lay?.plan;
  /** The land's water as its plan has it: the plan's river is the town's to bridge, and the town may go on either bank. */
  const kept = !!(plan && lay?.waterKept);
  const rand = mulberry32(lay?.townSeed ?? seed);
  const rolledName = worldName(content, spec.names, rand);
  const tagline = worldTagline(content, spec, rand);
  const rolledW = Math.round(range(rand, ...spec.world) / 64) * 64;
  const W = shaped ? shaped.cols * LAND_CELL : rolledW;
  const D = shaped ? shaped.rows * LAND_CELL : W;
  const land = shaped ? cells8(shaped) : new Cells(W, D);
  const { gw, gd } = land;
  const nseed = (seed % 100000) + 17;
  /** Country roads the layout found no way for (and bridges and causeways). */
  const failed: string[] = [];

  // ---- the land: hills (a valley between the massif's mountains), woods -----------------------
  const valley = !!spec.massif;
  const massif = spec.massif ? range(mulberry32((seed ^ MASSIF) >>> 0), ...spec.massif) : 0;
  const valleyZ = (x: number) => D / 2 + (fbm(x / 420, 3.7, nseed + 3, 3) - 0.47) * 260;
  const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
  for (let i = 0; i < gw * gd && !shaped; i++) {
    const [x, z] = land.centre(i);
    let h = fbm(x / 230, z / 230, nseed, 4) * spec.hills * 2;
    if (valley && spec.classic) h += massif * pmath.pow(clamp01((Math.abs(z - valleyZ(x)) - MASSIF_FLAT) / MASSIF_RISE), 1.4);
    land.height[i] = h;
  }
  /**
   * On a shaped land (its water not as planned), its pieces of dry land (`landPieces.ts`): the main
   * town on one, a village on every other big enough with a bridge to it (`linked`), the country
   * roads within each piece and over those bridges between them. Null otherwise (a classic world,
   * a land whose water is as planned): nothing here is laid as it was before.
   */
  const dry = shaped && !kept ? dryPieces(land) : null;
  /** The plan's river, on a land whose water is as planned. */
  const planRiver = kept && plan?.river ? riverOf(plan.river, W, (plan.seed % 100000) + 17) : null;
  /** Where the main town goes on a planned land: round the site asked for, else the plan's. */
  const siteAt = plan ? (lay?.site ?? plan.site) : null;
  /**
   * A place's mountains (not a classic world's): the massif's height times `m(x, z)`^1.4 (0 where
   * the land stays low, 1 where the mountains stand full height), lower on the low-z side, once
   * the water they rise from is laid.
   */
  const raise = (m: (x: number, z: number) => number) => {
    if (!valley || spec.classic || shaped) return;
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      land.height[i] += massif * pmath.pow(clamp01(m(x, z)), 1.4) * (MASSIF_NEAR + (1 - MASSIF_NEAR) * clamp01(z / (0.6 * D)));
    }
  };

  // ---- the main town: its roads first (which side its water is on), then where it goes -------
  const mainSpec = spec.town;
  const shape = spec.shape(rand, mainSpec.box, mainSpec.scale);
  const joined = settleRoads(joinUp(rand, shape, mainSpec, BIG_COURTS), mainSpec.box);
  // (On a planned land, its water on the side the site has it: the town, as laid out, turned over if it was drawn the other way.)
  const facing = plan && siteAt ? (lay?.site ? sideOf(plan, siteAt, planRiver, D) : plan.side) : undefined;
  const layout = facing && shape.water && shape.water !== facing ? mirrored({ ...shape, roads: joined }, mainSpec.box) : { ...shape, roads: joined };
  const mainRoads = layout.roads;
  const box = mainSpec.box;
  const waterSide = facing ?? layout.water ?? (rand() < 0.5 ? 'zmin' : 'zmax');

  // The sea along one edge, a lake in the middle; the mountains (a place's) inland of them, else either side of the valley.
  if (spec.water === 'sea') {
    const deep = range(rand, 120, 160);
    const shoreOf = (x: number) => deep + (fbm(x / 160, 9.1, nseed + 5, 3) - 0.47) * 90;
    for (let i = 0; i < gw * gd && !shaped; i++) {
      const [x, z] = land.centre(i);
      const shore = shoreOf(x);
      if (waterSide === 'zmax' ? z > D - shore : z < shore) land.water[i] = 1;
    }
    raise((x, z) => ((waterSide === 'zmax' ? D - shoreOf(x) - z : z - shoreOf(x)) - 160) / MASSIF_RISE);
  }
  const lake: Pt = [W / 2 + range(rand, -50, 50), D / 2 + range(rand, -40, 40)];
  if (spec.water === 'lake') {
    let [rx, rz] = [range(rand, 150, 185), range(rand, 95, 120)];
    // (A place's: a lake in proportion to its world, and room for the town between it and the world's edge, on its side of it.)
    if (!spec.classic) {
      [rx, rz] = [rx * Math.min(1, W / LAKE_WORLD), rz * Math.min(1, W / LAKE_WORLD)];
      const room = EDGE / 2 + box.d + 4 + rz * 1.4;
      lake[1] = waterSide === 'zmin' ? Math.min(lake[1], D - room) : Math.max(lake[1], room);
    }
    for (let i = 0; i < gw * gd && !shaped; i++) {
      const [x, z] = land.centre(i);
      const r = pmath.pow((x - lake[0]) / rx, 2) + pmath.pow((z - lake[1]) / rz, 2);
      if (r < 1 + (fbm(x / 90, z / 90, nseed + 6, 3) - 0.47) * 0.7) land.water[i] = 1;
    }
    raise((x, z) => ((pmath.hypot((x - lake[0]) / rx, (z - lake[1]) / rz) - 1.9) * Math.min(rx, rz)) / 200);
  }
  if (plan?.lake) [lake[0], lake[1]] = [plan.lake.x, plan.lake.z];
  // An island (a place's): a rounded square with a ragged coast, the sea all round it to the world's edges; its mountains in its middle.
  const coast: Pt[] = [];
  if (spec.water === 'island') {
    const [ax, az] = [ISLAND * W, ISLAND * D];
    for (let k = 0; k < COAST_POINTS; k++) {
      const a = (k / COAST_POINTS) * Math.PI * 2;
      const [c, s] = [pmath.cos(a), pmath.sin(a)];
      const r0 = 1 / pmath.cbrt(pmath.pow(Math.abs(c / ax), 3) + pmath.pow(Math.abs(s / az), 3));
      const r = r0 * (1 + (fbm((W / 2 + c * r0) / 140, (D / 2 + s * r0) / 140, nseed + 23, 3) - 0.47) * 0.14);
      coast.push([round2(W / 2 + c * r), round2(D / 2 + s * r)]);
    }
    for (let i = 0; i < gw * gd && !shaped; i++) if (!inside(coast, ...land.centre(i))) land.water[i] = 1;
    raise((x, z) => (0.75 - pmath.cbrt(pmath.pow(Math.abs((x - W / 2) / ax), 3) + pmath.pow(Math.abs((z - D / 2) / az), 3))) / 0.45);
  }
  if (spec.water === 'ponds') raise((x, z) => (Math.abs(z - valleyZ(x)) - MASSIF_FLAT) / MASSIF_RISE);
  /** The first dry z going from `from` towards `dir` across x0..x1 (the shore nearest the town). */
  const shoreAt = (x0: number, x1: number, dir: 1 | -1, from: number): number => {
    let best = dir > 0 ? Infinity : -Infinity;
    for (let x = Math.max(0, x0); x <= Math.min(W - 1, x1); x += CELL) {
      for (let z = from; z >= 0 && z < D; z += dir * CELL) {
        if (land.water[land.cell(x, z)]) {
          best = dir > 0 ? Math.min(best, z) : Math.max(best, z);
          break;
        }
      }
    }
    return best;
  };

  // Where the main town's box goes.
  let ox = Math.round(W / 2 - box.w / 2 + range(rand, -0.08, 0.08) * W);
  let oz = Math.round(D / 2 - box.d / 2 + range(rand, -0.08, 0.08) * D);
  if (siteAt) [ox, oz] = [Math.round(siteAt[0] - box.w / 2), Math.round(siteAt[1] - box.d / 2)];
  ox = Math.max(EDGE, Math.min(W - EDGE - box.w, ox));
  // (Room for the town between the river and the world's edge, on its side of it.)
  const room = EDGE / 2 + box.d + 40;
  const river = spec.water !== 'river' ? null : shaped ? planRiver : riverAcross(mulberry32((seed ^ RIVER) >>> 0), W, D, [ox, ox + box.w], nseed, waterSide === 'zmax' ? [room, D] : [0, D - room]);
  if (river) {
    for (let i = 0; i < gw * gd && !shaped; i++) {
      const [x, z] = land.centre(i);
      // (Every cell the river touches.)
      if (Math.abs(z - river.z(x)) < river.half(x) + CELL / 2) land.water[i] = 1;
    }
    raise((x, z) => (Math.abs(z - river.z(x)) - RIVER_FLAT) / MASSIF_RISE);
  }
  /** Which side of the river a point is on (-1 north, 1 south; 0 with no river). */
  const bank = (p: Pt) => (river ? (p[1] < river.z(p[0]) ? -1 : 1) : 0);
  if (siteAt) {
    // On a planned land: round its site, its water side against the shore if the shore is near
    // (looking for the water from the site), then as near that as it stands on dry land (if the
    // water was changed, on the piece under it if that piece holds it, else on whichever does nearest).
    if (spec.water === 'sea' || spec.water === 'lake' || spec.water === 'river' || spec.water === 'island') {
      const dir = waterSide === 'zmax' ? 1 : -1;
      const from = Math.max(0, Math.min(D - 1, siteAt[1]));
      const shore = shoreAt(ox, ox + box.w, dir, from);
      if (Number.isFinite(shore) && Math.abs(shore - from) <= SNAP_REACH) oz = Math.round(dir > 0 ? shore - 4 - box.d : shore + 4);
    }
    oz = Math.max(EDGE / 2, Math.min(D - EDGE / 2 - box.d, oz));
    const bounds: Bounds = [EDGE, EDGE / 2, W - EDGE - box.w, D - EDGE / 2 - box.d];
    const want = { x: ox, z: oz, w: box.w, d: box.d };
    // (Its water changed: on the piece of dry land under the box's middle if that piece holds it, else as near as it fits on any.)
    const under = dry ? dry.of([ox + box.w / 2, oz + box.d / 2]) : -1;
    const at = dry ? ((under >= 0 ? dry.nearest(under, want, bounds) : null) ?? dry.nearestAny(want, bounds)?.at ?? null) : dryLand(land).nearest(want, bounds, 0);
    if (!at) throw new Error(dry ? noRoom(box) : 'There is no dry land big enough for the town.');
    [ox, oz] = at;
  } else if (dry) {
    // On a shaped land: as near where it would go as it fits on the largest piece of dry land that holds it.
    const want = { x: ox, z: oz, w: box.w, d: box.d };
    const bounds: Bounds = [EDGE, EDGE / 2, W - EDGE - box.w, D - EDGE / 2 - box.d];
    let at: Pt | null = null;
    for (const id of dry.bySize) if (dry.sizes[id] * CELL * CELL >= box.w * box.d && (at = dry.nearest(id, want, bounds))) break;
    if (!at) throw new Error(noRoom(box));
    [ox, oz] = at;
  } else if (spec.water === 'sea' || spec.water === 'lake' || spec.water === 'river' || spec.water === 'island') {
    // Its water side on the shore (its first road runs along it): looking for the water from the land beyond the town.
    const dir = waterSide === 'zmax' ? 1 : -1;
    const from = Math.max(0, Math.min(D - 1, spec.water === 'island' ? D / 2 : (spec.water === 'sea' ? D / 2 : spec.water === 'lake' ? lake[1] : river!.z(ox + box.w / 2)) - dir * 260));
    const shore = shoreAt(ox, ox + box.w, dir, from);
    if (Number.isFinite(shore)) oz = Math.round(dir > 0 ? shore - 4 - box.d : shore + 4);
  } else if (valley) oz = Math.round(valleyZ(ox + box.w / 2) - box.d / 2);
  if (!dry && !siteAt) oz = Math.max(EDGE / 2, Math.min(D - EDGE / 2 - box.d, oz));

  const roads: RoadPlan[] = [];
  const plans: AreaPlan[] = [];
  const takenNames: string[] = [];
  const laying: Laying = { W, D, roads, plans };
  putArea(laying, name ?? rolledName, mainRoads, box, ox, oz, false, mainSpec, layout.greens ?? [], layout.centre ?? mainRoads[0].points[Math.floor(mainRoads[0].points.length / 2)]);

  // ---- a river's town bridge, an island's causeways -------------------------------------------
  /** Bridges and causeways laid (their ends, for the hamlets to keep clear of). */
  const crossings: Pt[][] = [];
  /** A straight road from `a` to `b` over the water (`bridge`: its deck, set once the roads are cut to the world). */
  const crossing = (a: Pt, b: Pt, ends: RoadPlan['ends'], roadName: string): RoadPlan => ({
    points: evenly([a, b], 1).map(([x, z]): Pt => [round2(x), round2(z)]),
    width: ROAD_WIDTH,
    ends,
    kind: 'country',
    name: roadName,
    bridge: [0, 0],
  });
  if (river) {
    // From a straight stretch of the waterfront street nearest the town's middle, a T square across the river,
    // on 12 m past its far bank: a way on from the town, as a port.
    const at = plans[0].roads[0];
    const pts = roads[at].points;
    const ls = lengths(pts);
    const L = ls[ls.length - 1];
    const tries: number[] = [];
    for (let s = 30; s <= L - 30; s += 4) if (straightAround(pts, s, 14, T_BEND, ls)) tries.push(s);
    tries.sort((x, y) => Math.abs(pointAt(pts, x, ls).p[0] - plans[0].centre[0]) - Math.abs(pointAt(pts, y, ls).p[0] - plans[0].centre[0]));
    for (const s of tries) {
      const p = pointAt(pts, s, ls).p;
      const span = across(river, p, 160);
      if (!span || span.wet[1] - span.wet[0] + 2 * DECK_PAST > BRIDGE_SPAN || land.wet(...span.far, 2)) continue;
      const laid = roads.push(crossing(p, span.far, ['join', 'join'], `${plans[0].name} Bridge`)) - 1;
      plans[0].ports.unshift({ road: laid, end: 1, area: 0, p: span.far, dir: span.dir, used: false });
      crossings.push([p, span.far]);
      break;
    }
    if (!crossings.length) failed.push(`no bridge from ${plans[0].name} over the river`);
  }
  /** An island's causeways: the inland end of each (where a country road comes to it), the way inland, the edge it leaves by (`FACING`). */
  const causeways: { p: Pt; dir: Pt; side: number; road: number }[] = [];
  if (spec.water === 'island') {
    // On the side of the island nearest the main town but its own shore's; a second (a town's, a city's) opposite.
    const c = plans[0].centre;
    const near = [c[1], W - c[0], D - c[1], c[0]];
    const own = waterSide === 'zmin' ? 0 : 2;
    const first = [0, 1, 2, 3].filter((k) => k !== own).sort((x, y) => near[x] - near[y])[0];
    for (const side of [first, (first + 2) % 4].slice(0, spec.exits[1])) {
      const out = EDGE_OUT[side];
      // Along the middle of that side, as near the town as it may be.
      const along = side % 2 ? Math.max(0.3 * D, Math.min(0.7 * D, c[1])) : Math.max(0.3 * W, Math.min(0.7 * W, c[0]));
      const edge: Pt = side % 2 ? [side === 1 ? W : 0, along] : [along, side === 2 ? D : 0];
      // In from the edge to the coast, then on inland.
      let d = 0;
      while (d < W / 2 && land.water[land.cell(edge[0] - out[0] * (d + 0.5), edge[1] - out[1] * (d + 0.5))]) d += 1;
      const inland: Pt = [round2(edge[0] - out[0] * (d + CAUSEWAY_IN)), round2(edge[1] - out[1] * (d + CAUSEWAY_IN))];
      const beyond: Pt = [edge[0] + out[0] * 40, edge[1] + out[1] * 40];
      const road = roads.push(crossing(inland, beyond, ['join', 'out'], `${EDGE_NAMES[side]} Causeway`)) - 1;
      causeways.push({ p: inland, dir: [-out[0], -out[1]], side, road });
      crossings.push([inland, edge]);
    }
  }

  // ---- hamlets: where it's dry and flat, far enough from the others, near enough to one --------
  const mainCentre = plans[0].centre;
  const hamlets = between(rand, spec.hamlets);
  /**
   * A place among mountains (its towns follow its slopes): a hamlet only where a country road from
   * the area nearest it can reach it within its grade, the two boxes' heights (their land's mean)
   * no further apart than `REACHABLE` of a country road's climb over the gap between them; and the
   * valley floor before the slopes (`MOUNTAIN_LOW`). Where none can go, it is left out (its houses
   * go to the others: `missing`).
   */
  const reachable = !spec.classic && spec.options?.hills === 'mountains';
  const boxHeight = (r: Rect) => {
    let [sum, n] = [0, 0];
    for (let z = Math.max(0, Math.floor(r.z / CELL)); z <= Math.min(gd - 1, Math.floor((r.z + r.d) / CELL)); z++) {
      for (let x = Math.max(0, Math.floor(r.x / CELL)); x <= Math.min(gw - 1, Math.floor((r.x + r.w) / CELL)); x++) {
        sum += land.height[z * gw + x];
        n++;
      }
    }
    return n ? sum / n : 0;
  };
  /**
   * The gentle ground a box's town is eased to (as `landFollow.ts` `gentler` fits it, near enough):
   * the plane through its cells' heights, tilted no more than `GENTLE_TILT`; its height at a point.
   */
  const planeOf = (r: Rect) => {
    const pts: Pt[] = [];
    const hs: number[] = [];
    for (let z = Math.max(0, Math.floor(r.z / CELL)); z <= Math.min(gd - 1, Math.floor((r.z + r.d) / CELL)); z++) {
      for (let x = Math.max(0, Math.floor(r.x / CELL)); x <= Math.min(gw - 1, Math.floor((r.x + r.w) / CELL)); x++) {
        pts.push(land.centre(z * gw + x));
        hs.push(land.height[z * gw + x]);
      }
    }
    const n = pts.length || 1;
    const [cx, cz, mean] = [pts.reduce((t, p) => t + p[0], 0) / n, pts.reduce((t, p) => t + p[1], 0) / n, hs.reduce((t, h) => t + h, 0) / n];
    let [sxx, sxz, szz, sxh, szh] = [0, 0, 0, 0, 0];
    pts.forEach(([x, z], i) => {
      const [dx, dz, dh] = [x - cx, z - cz, hs[i] - mean];
      [sxx, sxz, szz, sxh, szh] = [sxx + dx * dx, sxz + dx * dz, szz + dz * dz, sxh + dx * dh, szh + dz * dh];
    });
    const det = sxx * szz - sxz * sxz;
    let [bx, bz] = Math.abs(det) > 1e-9 ? [(sxh * szz - szh * sxz) / det, (szh * sxx - sxh * sxz) / det] : [0, 0];
    const tilt = pmath.hypot(bx, bz);
    if (tilt > GENTLE_TILT) [bx, bz] = [(bx * GENTLE_TILT) / tilt, (bz * GENTLE_TILT) / tilt];
    return ([x, z]: Pt) => mean + bx * (x - cx) + bz * (z - cz);
  };
  /** Each area's `planeOf` its box, once asked for. */
  const planes: ((p: Pt) => number)[] = [];
  /** The nearest points of two boxes to each other (the middle of where they overlap, along an axis they overlap on). */
  const facingPts = (a: Rect, b: Rect): [Pt, Pt] => {
    const axis = (a0: number, a1: number, b0: number, b1: number): [number, number] => (a1 < b0 ? [a1, b0] : b1 < a0 ? [a0, b1] : [(Math.max(a0, b0) + Math.min(a1, b1)) / 2, (Math.max(a0, b0) + Math.min(a1, b1)) / 2]);
    const [ax, bx] = axis(a.x, a.x + a.w, b.x, b.x + b.w);
    const [az, bz] = axis(a.z, a.z + a.d, b.z, b.z + b.d);
    return [[ax, az], [bx, bz]];
  };
  const rectGap = (a: Rect, b: Rect) => pmath.hypot(Math.max(0, a.x - (b.x + b.w), b.x - (a.x + a.w)), Math.max(0, a.z - (b.z + b.d), b.z - (a.z + a.d)));
  const candidates: Pt[] = Array.from({ length: 700 }, (): Pt => [range(rand, EDGE + 80, W - EDGE - 80), range(rand, EDGE + 60, D - EDGE - 60)]);
  /** Which quarter of the world a point is in: the hamlets go first to those without an area. */
  const quarter = (p: Pt) => (p[0] < W / 2 ? 0 : 1) + (p[1] < D / 2 ? 0 : 2);
  // A shaped land cut into pieces by its water (`dry`): with villages, one first on every piece big
  // enough that a bridge could reach from a piece with an area on it, the largest first
  // (`villagePerPiece`); then the rest of them, on the pieces with an area (`onLinked`).
  /** The land's water (1 m tiles), on a shaped land cut into pieces: where its bridges could go; also its lots' (`waterMask`). */
  const wet1m = dry ? wetTiles(shaped!, W, D) : null;
  /** Where a bridge could cross between two pieces of dry land (`bridgeSites`): its ends `BRIDGE_IN` m past the banks, the piece each is on. */
  const links: { a: Pt; b: Pt; pieces: [number, number] }[] = [];
  /** The pieces of dry land with an area on them (the main town's first): those the rest of the villages may go on. */
  const linked = new Set<number>();
  /** The pieces big enough for a village that got none: no bridge could reach them, or no site on them could be reached (`diagnostics.pieces`). */
  const unreached: string[] = [];
  if (dry) linked.add(dry.of([ox + box.w / 2, oz + box.d / 2]));
  /**
   * A village on every piece of dry land big enough for one (its box and 24 m round it) that a
   * crossing (`links`) reaches from a piece with an area on it, the largest first: its site the best
   * of a 16 m lattice over the piece by how flat and low it is and how near a crossing to it, which
   * stays `BRIDGE_CLEAR` m clear of its box; `AREA_APART` m from the other areas (but not within
   * `AREA_REACH`: the bridge is its way in), and the mountains' and the gentle ground's rules as the
   * other villages keep them. How many it put down.
   */
  function villagePerPiece(): number {
    const pieces = dry!;
    for (const s of bridgeSites(wet1m!, W, D, PIECE_SPAN)) {
      const a: Pt = [round2(s.a[0] - s.u[0] * BRIDGE_IN), round2(s.a[1] - s.u[1] * BRIDGE_IN)];
      const b: Pt = [round2(s.b[0] + s.u[0] * BRIDGE_IN), round2(s.b[1] + s.u[1] * BRIDGE_IN)];
      const [pa, pb] = [pieces.of(a), pieces.of(b)];
      if (pa >= 0 && pb >= 0 && pa !== pb) links.push({ a, b, pieces: [pa, pb] });
    }
    const hb = spec.hamlet.box;
    // (A piece with fewer cells than a village's box and the land round it can't hold one.)
    const least = Math.ceil((hb.w + 48) / CELL) * Math.ceil((hb.d + 48) / CELL);
    const reaches = (id: number) => links.filter((l) => (l.pieces[0] === id && linked.has(l.pieces[1])) || (l.pieces[1] === id && linked.has(l.pieces[0])));
    const tried = new Set<number>();
    let n = 0;
    for (;;) {
      const id = pieces.bySize.find((k) => !tried.has(k) && !linked.has(k) && pieces.sizes[k] >= least && reaches(k).length > 0);
      if (id === undefined) break;
      tried.add(id);
      const ends = reaches(id).map((l) => ({ near: l.pieces[0] === id ? l.a : l.b, line: evenly([l.a, l.b], 4) }));
      const site: HamletSite = { water: spec.water, boxes: plans.map((a) => a.rect), region: { holds: (r, by) => pieces.holds(id, r, by) }, crossings };
      const first = rand() < 0.35;
      let best = null as { p: Pt; swap: boolean; score: number } | null;
      for (const swap of [first, !first]) {
        const [bw, bd] = swap ? [hb.d, hb.w] : [hb.w, hb.d];
        for (let z = EDGE + bd / 2; z <= D - EDGE - bd / 2; z += 16) {
          for (let x = EDGE + bw / 2; x <= W - EDGE - bw / 2; x += 16) {
            const p: Pt = [x, z];
            if (pieces.of(p) !== id || !pieces.holds(id, { x: Math.round(x - bw / 2), z: Math.round(z - bd / 2), w: bw, d: bd }, 24)) continue;
            if (Math.min(...plans.map((a) => dist(a.centre, p))) < AREA_APART) continue;
            const fit = hamletFits(land, p, [bw, bd], site);
            if (!fit) continue;
            const { rect, steep } = fit;
            // (A bridge to it kept clear of its box.)
            const clear = ends.filter((e) => e.line.every((q) => offRect(rect, q) >= BRIDGE_CLEAR));
            if (!clear.length) continue;
            // (Among mountains, the area it joins reachable; and every area's gentle ground met within a road's climb.)
            if (reachable) {
              const joins = plans.reduce((a, b) => (dist(b.centre, p) < dist(a.centre, p) ? b : a));
              if (Math.abs(boxHeight(rect) - boxHeight(joins.rect)) > REACHABLE * COUNTRY_GRADE * rectGap(rect, joins.rect)) continue;
            }
            const own = planeOf(rect);
            const apart = plans.some((o, k) => {
              const [pa, pb] = facingPts(rect, o.rect);
              return Math.abs(own(pa) - (planes[k] ??= planeOf(o.rect))(pb)) > REACHABLE * COUNTRY_GRADE * rectGap(rect, o.rect) + PLANE_SLACK;
            });
            if (apart) continue;
            // Flat, low, and near where a bridge would come to it.
            const score = steep * 40 + land.at(x, z) * (reachable ? MOUNTAIN_LOW : 0.02) + Math.min(...clear.map((e) => offRect(rect, e.near))) / 40;
            if (!best || score < best.score) best = { p, swap, score };
          }
        }
        if (best) break;
      }
      if (!best) continue;
      const [bw, bd] = best.swap ? [hb.d, hb.w] : [hb.w, hb.d];
      const hamlet = hamletOf(content, spec, rand, plans.length - 1, plans.map((a) => a.name));
      putHamlet(laying, hamlet, Math.round(best.p[0] - bw / 2), Math.round(best.p[1] - bd / 2), best.swap);
      linked.add(id);
      n++;
    }
    // (Pieces big enough that no crossing of `PIECE_SPAN` m reaches from a piece with an area, or with no site on them a village could take.)
    for (const id of pieces.bySize) {
      if (linked.has(id) || pieces.sizes[id] < least) continue;
      const why = tried.has(id) ? 'no site on it for a village a bridge could reach' : `no bridge of ${PIECE_SPAN} m or less reaches it`;
      unreached.push(`No village on ${Math.round((pieces.sizes[id] * CELL * CELL) / 10000)} ha of dry land: ${why}.`);
    }
    return n;
  }
  const placed = dry && spec.hamlets[1] > 0 ? villagePerPiece() : 0;
  /** The rest of the villages' land on a shaped land cut into pieces: any piece with an area on it. */
  const onLinked = dry ? { holds: (r: Rect, by: number) => [...linked].some((id) => dry.holds(id, r, by)) } : null;
  for (let n = placed, relax = 0; n < (dry ? Math.max(hamlets, placed) : hamlets); n++) {
    const held = new Set(plans.map((a) => quarter(a.centre)));
    const swap = rand() < 0.35;
    const hb = spec.hamlet.box;
    const [bw, bd] = swap ? [hb.d, hb.w] : [hb.w, hb.d];
    let best: { p: Pt; score: number } | null = null;
    const site: HamletSite = { water: spec.water, boxes: plans.map((a) => a.rect), region: onLinked, crossings, relax };
    // (No room where it should be: a little further, on steeper land.)
    for (const p of relax ? [...candidates].reverse() : candidates) {
      const near = Math.min(...plans.map((a) => dist(a.centre, p)));
      if (near < AREA_APART || near > AREA_REACH + relax * 90) continue;
      // In the world, clear of the other areas' boxes, dry, not too steep (`hamletFits`).
      const fit = hamletFits(land, p, [bw, bd], site);
      if (!fit) continue;
      const { rect, steep } = fit;
      if (reachable) {
        const joins = plans.reduce((a, b) => (dist(b.centre, p) < dist(a.centre, p) ? b : a));
        if (Math.abs(boxHeight(rect) - boxHeight(joins.rect)) > REACHABLE * COUNTRY_GRADE * rectGap(rect, joins.rect)) continue;
      }
      // (Drawn for every site that fits so far: one the next rule turns away leaves the draws after it as they were.)
      const jitter = rand() * 0.5;
      // In a place's world (its towns follow its slopes), only where the gentle ground its town will
      // be eased to meets every other area's within what a road between them can climb: the two
      // at the boxes' nearest points no further apart than `REACHABLE` of a country road's climb
      // over the gap between them, and `PLANE_SLACK` m. (Two boxes close together on a hillside,
      // each eased to its own plane, meet in a cliff of eased land, and a street reaching from one
      // into the other is cut deep into it.)
      if (!spec.classic) {
        const own = planeOf(rect);
        const apart = plans.some((o, k) => {
          const [pa, pb] = facingPts(rect, o.rect);
          return Math.abs(own(pa) - (planes[k] ??= planeOf(o.rect))(pb)) > REACHABLE * COUNTRY_GRADE * rectGap(rect, o.rect) + PLANE_SLACK;
        });
        if (apart) continue;
      }
      let score = Math.abs(near - AREA_AIM) / 40 + steep * 40 + land.at(p[0], p[1]) * (reachable ? MOUNTAIN_LOW : valley ? 0.08 : 0.02) + (held.has(quarter(p)) ? 0 : -5);
      // A fishing cove on the shore; a hamlet across the lake from the town.
      if (n === 0 && spec.water === 'sea') score += land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 70) ? -6 : 6;
      if (n === 0 && river) score += bank(p) !== bank(mainCentre) ? -6 : 6;
      if (n === 0 && spec.water === 'lake') score += ((p[0] - lake[0]) * (mainCentre[0] - lake[0]) + (p[1] - lake[1]) * (mainCentre[1] - lake[1]) < 0 ? -6 : 6) + (land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 60) ? -3 : 3);
      score += jitter;
      if (!best || score < best.score) best = { p, score };
    }
    if (!best) {
      if (relax++ < 1) n--;
      continue;
    }
    const hamlet = hamletOf(content, spec, rand, plans.length - 1, plans.map((a) => a.name));
    putHamlet(laying, hamlet, Math.round(best.p[0] - bw / 2), Math.round(best.p[1] - bd / 2), swap);
  }

  // Fewer hamlets than the world has at least: their houses go to the others (the main town first; `LotPlacer` spills them on).
  const missing = spec.hamlets[0] - (plans.length - 1);
  if (missing > 0) plans[0].spec = { ...mainSpec, lots: mainSpec.lots + missing * spec.hamlet.lots[0] };

  step?.('land');
  // ---- country roads: between the areas (a tree of them and one more for a loop), and out ----
  // (Routed over the land's cells, round the areas' boxes, sharing the roads laid as they go down: `countryRouter.ts`.)
  const router = new CountryRouter(land, plans, roads, spec);
  const { areaBlock, wetNear } = router;
  /** Whether the land within `SPILL_LOOK` m of `p` is gentle enough for a lot out along a country road (any, in a classic world). */
  const gentleBy = (p: Pt) => {
    if (spec.classic) return true;
    for (let dz = -SPILL_LOOK; dz <= SPILL_LOOK; dz += CELL) for (let dx = -SPILL_LOOK; dx <= SPILL_LOOK; dx += CELL) if (land.slope(land.cell(p[0] + dx, p[1] + dz)) > PLACE_STEEP) return false;
    return true;
  };
  /** How far a way heading `dir` from `p` is from heading for `toward`, as metres of road: nothing within `SWING`, then more the more it would have to swing round. */
  const swing = (p: Pt, dir: Pt, toward: Pt) => {
    const l = dist(p, toward) || 1;
    const cos = (dir[0] * (toward[0] - p[0]) + dir[1] * (toward[1] - p[1])) / l;
    return Math.max(0, pmath.acos(Math.max(-1, Math.min(1, cos))) - SWING) * SWING_COST;
  };
  /**
   * Which of `list` (roads) pass within `R` m of a point round `rect` (to 40 m out): -1 none, the
   * road if only one does, -2 if more.
   */
  const roadsNear = (list: readonly number[], rect: Rect, R: number) => {
    const G = 2;
    const [x0, z0] = [rect.x - 40, rect.z - 40];
    const [w, d] = [Math.ceil((rect.w + 80) / G), Math.ceil((rect.d + 80) / G)];
    const grid = new Int32Array(w * d).fill(-1);
    const reach = Math.ceil(R / G);
    const add = (i: number) => {
      const pts = roads[i].points;
      for (let k = 0; k + 1 < pts.length; k++) {
        const steps = Math.max(1, Math.ceil(dist(pts[k], pts[k + 1]) / G));
        for (let j = 0; j <= steps; j++) {
          const [px, pz] = [pts[k][0] + ((pts[k + 1][0] - pts[k][0]) * j) / steps, pts[k][1] + ((pts[k + 1][1] - pts[k][1]) * j) / steps];
          const [cx, cz] = [Math.floor((px - x0) / G), Math.floor((pz - z0) / G)];
          for (let z = Math.max(0, cz - reach); z <= Math.min(d - 1, cz + reach); z++) {
            const dz = z0 + (z + 0.5) * G - pz;
            for (let x = Math.max(0, cx - reach); x <= Math.min(w - 1, cx + reach); x++) {
              const at = z * w + x;
              const dx = x0 + (x + 0.5) * G - px;
              if (grid[at] === i || grid[at] === -2 || dx * dx + dz * dz >= R * R) continue;
              grid[at] = grid[at] === -1 ? i : -2;
            }
          }
        }
      }
    };
    list.forEach(add);
    const at = ([px, pz]: Pt) => {
      const [x, z] = [Math.floor((px - x0) / G), Math.floor((pz - z0) / G)];
      return x < 0 || z < 0 || x >= w || z >= d ? -1 : grid[z * w + x];
    };
    return { at, add };
  };
  /** Per area, `roadsNear` its own roads, and how many of the world's roads it has been told about. */
  const sideGrids = new Map<number, { roads: number; apart: ReturnType<typeof roadsNear>; depth: ReturnType<typeof roadsNear> }>();
  /** Which of area `a`'s roads (and the country roads into it) pass within `SIDE_APART` and `SIDE_DEPTH` of a point. */
  const gridsOf = (a: number) => {
    const r = plans[a].rect;
    let grids = sideGrids.get(a);
    if (!grids) sideGrids.set(a, (grids = { roads: roads.length, apart: roadsNear(plans[a].roads, r, SIDE_APART), depth: roadsNear(plans[a].roads, r, SIDE_DEPTH) }));
    // (Country roads into the area since: from a T.)
    for (; grids.roads < roads.length; grids.roads++) {
      if (roads[grids.roads].kind === 'country' && roads[grids.roads].points.some(([x, z]) => x > r.x - 30 && z > r.z - 30 && x < r.x + r.w + 30 && z < r.z + r.d + 30)) grids.depth.add(grids.roads);
    }
    return { apart: grids.apart.at, depth: grids.depth.at };
  };
  /**
   * Where way `w` (out of another area), straight on, comes to one of area `b`'s roads within
   * `STRAIGHT_ON` m: square to it give or take `SIDE_FACING`, where it runs straight and not by a
   * junction, over dry land and nobody else's area, clear of `b`'s other roads by a lot's depth.
   * The point (on the road's middle) and how far; null if it doesn't.
   */
  const straightOn = (w: Way, b: number): { p: Pt; len: number } | null => {
    const { apart, depth } = gridsOf(b);
    let to = -1;
    for (let d = 2; d <= STRAIGHT_ON; d += 2) {
      const q: Pt = [w.p[0] + w.dir[0] * d, w.p[1] + w.dir[1] * d];
      const owner = areaBlock[land.cell(q[0], q[1])];
      if (land.wet(q[0], q[1], 3) || (owner && owner !== b + 1 && owner !== (w.port?.area ?? -2) + 1)) return null;
      const by = depth(q);
      if (by === -1) continue;
      // (The first of its roads it comes near is the one it meets, and nothing else on the way.)
      if (by === -2 || !plans[b].roads.includes(by) || (to >= 0 && by !== to)) return null;
      to = by;
      const pts = roads[by].points;
      const ls = lengths(pts);
      const at = nearest(pts, q[0], q[1], ls);
      if (at.dist > 2) continue;
      const { p, t } = pointAt(pts, at.s, ls);
      const ok = Math.abs(t[0] * w.dir[0] + t[1] * w.dir[1]) <= pmath.sin(SIDE_FACING) && straightAround(pts, at.s, 14, STRAIGHT_BEND, ls) && [-1, by].includes(apart(p)) && at.s > 20 && at.s < ls[ls.length - 1] - 20;
      return ok ? { p, len: dist(w.p, p) } : null;
    }
    return null;
  };
  /**
   * Whether way `w` out of area `a` and way `v` into area `b` face each other head on (each within
   * `HEAD_ON` of the straight line between them), no further apart than `STRAIGHT_ON`, with nothing
   * between them but land: no water, no other area's box, no road. (Two areas close together, a
   * port of one in line with a T or a port of the other: the road between them is that line, not
   * one routed round and into a far port, through the other's streets.)
   */
  const headOn = (w: Way, v: Way, a: number, b: number): boolean => {
    const l = dist(w.p, v.p);
    if (l < CELL || l > STRAIGHT_ON) return false;
    const u: Pt = [(v.p[0] - w.p[0]) / l, (v.p[1] - w.p[1]) / l];
    if (u[0] * w.dir[0] + u[1] * w.dir[1] < pmath.cos(HEAD_ON) || -(u[0] * v.dir[0] + u[1] * v.dir[1]) < pmath.cos(HEAD_ON)) return false;
    const index = new RoadIndex(roads);
    for (let d = 2; d < l - 1; d += 2) {
      const q: Pt = [w.p[0] + u[0] * d, w.p[1] + u[1] * d];
      const owner = areaBlock[land.cell(q[0], q[1])];
      if (land.wet(q[0], q[1], 3) || (owner && owner !== a + 1 && owner !== b + 1)) return false;
      // (Clear of every road but by its ends, where it meets the roads it leaves and comes to.)
      if (d > ROAD_WIDTH && d < l - ROAD_WIDTH && index.nearest(q[0], q[1], ROAD_WIDTH)) return false;
    }
    return true;
  };
  /**
   * A way out of area `a` towards `toward` by a T off one of its streets or lanes: where it runs
   * straight (and not by a junction), square to it on the side facing `toward` (within
   * `SIDE_FACING`), straight on across the area's land clear of its other roads, its lots' depth
   * from them, and of the water. The nearest such to `toward` with the least to cross first, none
   * two within `SIDE_APART` of each other.
   */
  const sideT = (a: number, toward: Pt): (Way & { score: number })[] => {
    const plan = plans[a];
    const r = plan.rect;
    const near = (p: Pt, m: number) => p[0] > r.x - m && p[1] > r.z - m && p[0] < r.x + r.w + m && p[1] < r.z + r.d + m;
    const { apart, depth } = gridsOf(a);
    const found: (Way & { score: number })[] = [];
    for (const ri of plan.roads) {
      const pts = roads[ri].points;
      const ls = lengths(pts);
      const L = ls[ls.length - 1];
      for (let s = 20; s <= L - 20; s += 4) {
        if (!straightAround(pts, s, 14, T_BEND, ls)) continue;
        const { p, t } = pointAt(pts, s, ls);
        const l = dist(p, toward) || 1;
        const u: Pt = [(toward[0] - p[0]) / l, (toward[1] - p[1]) / l];
        const left = leftOf(t);
        const k = left[0] * u[0] + left[1] * u[1] >= 0 ? 1 : -1;
        const n: Pt = [left[0] * k, left[1] * k];
        if (n[0] * u[0] + n[1] * u[1] < pmath.cos(SIDE_FACING)) continue;
        if (![-1, ri].includes(apart(p))) continue;
        let reach = 0;
        for (let d = 4; d < 200; d += 4) {
          const q: Pt = [p[0] + n[0] * d, p[1] + n[1] * d];
          const by = depth(q);
          if (land.wet(q[0], q[1], 4) || (by !== -1 && (by !== ri || d > SIDE_DEPTH))) break;
          const cell = land.cell(q[0], q[1]);
          if (!near(q, 0) && !areaBlock[cell]) {
            // (Over the land beyond, first.)
            if (router.cost([])(cell) < Infinity) reach = d;
            break;
          }
        }
        if (!reach) continue;
        const out: Pt = [p[0] + n[0] * reach, p[1] + n[1] * reach];
        found.push({ p, dir: n, reach, score: reach + dist(out, toward) + swing(out, n, toward) + SIDE_COST });
      }
    }
    found.sort((x, y) => x.score - y.score);
    return found.filter((w, i) => found.slice(0, i).every((o) => dist(o.p, w.p) >= SIDE_APART));
  };
  /**
   * Whether port `q` is no road end any more: another area's street or lane comes up to it (its
   * middle within half the wider one's width and a metre: the simulation joins them, one street
   * on through). No country road leaves it: it ran on along that street a few metres off before
   * turning away, crossing it (Canalside's Queen'ston Lane, by Foundry Street's end).
   */
  const continued = (q: Port) =>
    roads.some((o) => o.area !== undefined && o.area !== q.area && o.kind !== 'trail' && o.kind !== 'country' && nearest(o.points, q.p[0], q.p[1]).dist <= Math.max(o.width, roads[q.road].width) / 2 + 1);
  /**
   * The way a country road leaves area `a` towards `toward` (or comes into it from there): the
   * port nearest it that faces it (a port with a country road on from it already, off that road),
   * else a T off one of its streets facing it (`sideT`) if that swings round less.
   */
  const waysOut = (a: number, toward: Pt): (Way & { score: number })[] => {
    const ports = plans[a].ports.filter((q) => !continued(q)).map((q) => {
      const w = router.via(q, toward);
      return { ...w, score: dist(w.p, toward) + swing(w.p, w.dir, toward) + router.takenCost(w) };
    });
    return [...ports, ...sideT(a, toward).slice(0, 3)].sort((x, y) => x.score - y.score);
  };
  // (Two areas' streets meeting end to end: their ports are a way on already, kept as they are.)
  for (const q of plans.flatMap((plan) => plan.ports)) if (continued(q)) q.used = true;
  // Prim's tree over the areas, then the shortest edge it left out. (Across a river only the main
  // town and an area on the other bank: over its bridge. On a shaped land cut into pieces, a tree
  // on each piece: the bridges between them come after.)
  /** Each area's piece of dry land (`dry`). */
  const areaPiece = dry ? plans.map((a) => dry.of(a.centre)) : [];
  const apart = (a: number, b: number) => ((river && a && b && bank(plans[a].centre) !== bank(plans[b].centre)) || (dry && areaPiece[a] !== areaPiece[b]) ? Infinity : dist(plans[a].centre, plans[b].centre));
  const inTree = [0];
  const edges: [number, number][] = [];
  while (inTree.length < plans.length) {
    let best: [number, number, number] | null = null;
    for (const a of inTree) for (let b = 0; b < plans.length; b++) if (!inTree.includes(b) && (!best || apart(a, b) < best[2])) best = [a, b, apart(a, b)];
    // (Only areas on other pieces left: the next tree begins.)
    if (dry && best![2] === Infinity) {
      inTree.push(best![1]);
      continue;
    }
    edges.push([best![0], best![1]]);
    inTree.push(best![1]);
  }
  // (A loop or two: more of them in a world of more areas.)
  for (let loops = plans.length >= 6 ? 2 : plans.length >= 3 ? 1 : 0; loops > 0; loops--) {
    let extra: [number, number, number] | null = null;
    for (let a = 0; a < plans.length; a++) {
      for (let b = a + 1; b < plans.length; b++) {
        if (edges.some(([x, y]) => (x === a && y === b) || (x === b && y === a))) continue;
        const d = apart(a, b);
        if (d < 560 && (!extra || d < extra[2])) extra = [a, b, d];
      }
    }
    if (extra) edges.push([extra[0], extra[1]]);
  }
  const roadNames = new Set<string>();
  // (The bridges and causeways laid already have theirs.)
  for (const r of roads) if (r.bridge && r.name) roadNames.add(r.name);
  const uniqueName = (base: string, alt: string) => {
    const pick = !roadNames.has(base) ? base : !roadNames.has(alt) ? alt : `${alt} ${roadNames.size}`;
    roadNames.add(pick);
    return pick;
  };
  for (const [a, b] of edges) {
    // The likeliest ways out of each towards the other, the cheapest way between any of them;
    // if it turns too sharply (its ends, smoothed), again without the way it came in by, then without the one it left by.
    // (By a river, the road keeps to the bank the area other than the main town is on: the main
    // town's way to the far bank is over its bridge.)
    const side = bank(plans[a === 0 ? b : a].centre);
    const onBank = (list: (Way & { score: number })[]) => (river ? list.filter((w) => bank(w.p) === side) : list);
    let outs = onBank(waysOut(a, plans[b].centre)).slice(0, WAYS);
    let ins = onBank(waysOut(b, plans[a].centre)).slice(0, WAYS);
    type Made = { road: RoadPlan; pa: Way; pb: Way; bad: number; cells?: number[] };
    let made = null as Made | null;
    // Near enough, a way out of one straight on to a street of the other, meeting it in a T.
    const direct = [...outs.map((w) => ({ w, to: b })), ...ins.map((w) => ({ w, to: a }))]
      .map(({ w, to }) => ({ w, hit: straightOn(w, to) }))
      .flatMap(({ w, hit }) => (hit ? [{ w, hit, cost: hit.len + router.wayCost(w) * CELL }] : []))
      .sort((x, y) => x.cost - y.cost)[0];
    // (A place's: a way out of one facing a way into the other head on, near enough, straight from one to the other.)
    const faced = spec.classic ? undefined : outs.flatMap((w) => ins.flatMap((v) => (headOn(w, v, a, b) ? [{ w, v, cost: dist(w.p, v.p) + (router.wayCost(w) + router.wayCost(v)) * CELL }] : []))).sort((x, y) => x.cost - y.cost)[0];
    if (faced && (!direct || faced.cost < direct.cost)) {
      const pts = curve(evenly([faced.w.p, faced.v.p], CTRL_APART));
      made = { road: { points: pts, width: COUNTRY_WIDTH, ends: ['join', 'join'], kind: 'country', name: '' }, pa: faced.w, pb: faced.v, bad: 0 };
    } else if (direct) {
      const pts = curve(evenly([direct.w.p, direct.hit.p], CTRL_APART));
      made = { road: { points: pts, width: COUNTRY_WIDTH, ends: ['join', 'join'], kind: 'country', name: '' }, pa: direct.w, pb: { p: direct.hit.p, dir: [-direct.w.dir[0], -direct.w.dir[1]] }, bad: 0 };
    }
    for (let tries = 0; tries < TRIES && outs.length && ins.length && (!made || made.bad > 0); tries++) {
      // (Into the areas by their ports, or off a road by one, if it comes to it.)
      const ends = [...outs, ...ins].flatMap((w) => (w.port ? [w.port.p, w.p] : []));
      const goals = ins.map((w) => ({ w, cell: router.outCell(w), into: router.into(w), extra: router.wayCost(w) }));
      // (Coming in by the cheapest of them that ends there, heading in.)
      const goal = (i: number, h?: number, loose?: boolean) => {
        let best = -1;
        for (const o of goals) if (o.cell === i && o.into(h, loose) && (best < 0 || o.extra < best)) best = o.extra;
        return best;
      };
      const goalAt = goals.map((o) => land.centre(o.cell));
      const guess = (i: number) => {
        const c = land.centre(i);
        let near = Infinity;
        for (const e of goalAt) near = Math.min(near, dist(c, e));
        return near / CELL;
      };
      const span = dist(plans[a].centre, plans[b].centre) || 1;
      const found = router.route(outs, goal, guess, [(plans[b].centre[0] - plans[a].centre[0]) / span, (plans[b].centre[1] - plans[a].centre[1]) / span], router.costs(ends));
      if (!found) break;
      const end = found.cells[found.cells.length - 1];
      const pb = goals.filter((o) => o.cell === end && o.into(found.h, true)).sort((x, y) => x.extra - y.extra)[0]?.w ?? goals.find((o) => o.cell === end)!.w;
      const pa = outs[found.from];
      const road = router.road(pa, found.cells, pb, ['join', 'join'], '');
      const bad = router.fault(road);
      if (!made || bad < made.bad) made = { road, pa, pb, bad, cells: found.cells };
      // (No way it steers between any of them: none without them either.)
      if (!bad || !found.steered) break;
      if (tries % 2 === 0) ins = ins.filter((w) => w !== pb);
      else outs = outs.filter((w) => w !== pa);
    }
    if (!made) {
      failed.push(`no country road between ${plans[a].name} and ${plans[b].name}`);
      continue;
    }
    const far = b === 0 ? a : a === 0 ? b : dist(plans[a].centre, mainCentre) > dist(plans[b].centre, mainCentre) ? a : b;
    const near = far === a ? b : a;
    const name = () => uniqueName(`${plans[far].name} Road`, `${plans[near].name} Lane`);
    // (In pieces where it runs along a road laid already; a straight road onto a street, whole.)
    const { laid, first, last } = router.course(made.road, made.pa, made.cells, made.pb);
    for (const road of laid) roads.push({ ...router.settle(road), name: name() });
    // (Its ends' ports, where a piece of it leaves them.)
    if (first && made.pa.port) made.pa.port.used = true;
    if (last && made.pb.port) made.pb.port.used = true;
  }
  if (dry && links.length) bridgePieces();
  /**
   * A shaped land cut into pieces (`dry`): a bridge to each piece with an area on it from the main
   * town's, Prim's tree over them (the nearest first). For each piece, the best of `PIECE_TRIES`
   * crossings (`links`) to one joined already, by how far their ends are from the areas either
   * side: `BRIDGE_CLEAR` m clear of the areas' boxes (else, all of them failing, 12 m), with no road
   * within 12 m and `PIECE_BRIDGES_APART` m from the other bridges. A bridge is laid only with a
   * country road from each end to the nearest area on its piece (`approach`).
   */
  function bridgePieces(): void {
    const populated = new Set(areaPiece);
    const joined = new Set([areaPiece[0]]);
    const lost = new Set<number>();
    const mid = ([a, b]: readonly Pt[]): Pt => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
    /** The areas on piece `id`, the nearest `p` first. */
    const areasOn = (id: number, p: Pt) => plans.flatMap((_, k) => (areaPiece[k] === id ? [k] : [])).sort((x, y) => offRect(plans[x].rect, p) - offRect(plans[y].rect, p));
    /** A link from a joined piece (`a` its end there) to one not joined yet (`b`), and how far its ends are from the areas. */
    const oriented = () =>
      links.flatMap((l) => {
        const [ja, jb] = [joined.has(l.pieces[0]), joined.has(l.pieces[1])];
        if (ja === jb) return [];
        const [a, b, from, to] = ja ? [l.a, l.b, l.pieces[0], l.pieces[1]] : [l.b, l.a, l.pieces[1], l.pieces[0]];
        if (!populated.has(to) || lost.has(to)) return [];
        return [{ a, b, from, to, score: offRect(plans[areasOn(from, a)[0]].rect, a) + offRect(plans[areasOn(to, b)[0]].rect, b) }];
      });
    for (;;) {
      const index = new RoadIndex(roads);
      const open = (l: { a: Pt; b: Pt }, clear: number) => {
        const pts = evenly([l.a, l.b], 4);
        return pts.every((q) => plans.every((plan) => offRect(plan.rect, q) >= clear) && !index.nearest(q[0], q[1], 12)) && crossings.every((c) => dist(mid(c), mid([l.a, l.b])) >= PIECE_BRIDGES_APART);
      };
      const all = oriented().sort((x, y) => x.score - y.score);
      if (!all.length) break;
      const to = (all.find((l) => open(l, BRIDGE_CLEAR)) ?? all.find((l) => open(l, 12)) ?? all[0]).to;
      const village = plans[areasOn(to, all.find((l) => l.to === to)!.b)[0]].name;
      let done = false;
      for (const clear of [BRIDGE_CLEAR, 12]) {
        for (const l of all.filter((o) => o.to === to && open(o, clear)).slice(0, PIECE_TRIES)) {
          const at = roads.length;
          const u: Pt = [(l.b[0] - l.a[0]) / dist(l.a, l.b), (l.b[1] - l.a[1]) / dist(l.a, l.b)];
          const ka = areasOn(l.from, l.a)[0];
          const kb = areasOn(l.to, l.b)[0];
          const ma = approach({ road: at, end: 0, area: -1, p: l.a, dir: [-u[0], -u[1]], used: false }, ka);
          const mb = ma && approach({ road: at, end: 1, area: -1, p: l.b, dir: u, used: false }, kb);
          if (!ma || !mb) continue;
          roads.push(crossing(l.a, l.b, ['join', 'join'], uniqueName(`${village} Bridge`, `${village} Old Bridge`)));
          crossings.push([l.a, l.b]);
          for (const [made, place] of [
            [ma, plans[kb].name],
            [mb, plans[ka].name],
          ] as const) {
            const name = () => uniqueName(`${place} Road`, `${place} Lane`);
            const { laid, last } = router.course(made.road, made.pa, made.cells, made.pb);
            for (const road of laid) roads.push({ ...router.settle(road), name: name() });
            if (last && made.pb.port) made.pb.port.used = true;
          }
          done = true;
          break;
        }
        if (done) break;
      }
      if (done) joined.add(to);
      else {
        lost.add(to);
        failed.push(`no bridge to ${village}`);
      }
    }
  }
  /**
   * A country road from port `q` (a bridge's end, heading off it) to area `k`: straight on to one
   * of its streets if it comes to one (`straightOn`), else routed into it as the country roads
   * between the areas are (their tries); null if there is no way (or only over the water).
   */
  function approach(q: Port, k: number): { road: RoadPlan; pa: Way; pb: Way; bad: number; cells?: number[] } | null {
    const w: Way = { p: q.p, dir: q.dir, port: q };
    const hit = straightOn(w, k);
    if (hit) return { road: { points: curve(evenly([w.p, hit.p], CTRL_APART)), width: COUNTRY_WIDTH, ends: ['join', 'join'], kind: 'country', name: '' }, pa: w, pb: { p: hit.p, dir: [-w.dir[0], -w.dir[1]] }, bad: 0 };
    let outs = [w];
    let ins = waysOut(k, w.p).slice(0, WAYS);
    let made = null as { road: RoadPlan; pa: Way; pb: Way; bad: number; cells?: number[] } | null;
    for (let tries = 0; tries < TRIES && outs.length && ins.length && (!made || made.bad > 0); tries++) {
      const ends = [...outs, ...ins].flatMap((v) => (v.port ? [v.port.p, v.p] : []));
      const goals = ins.map((v) => ({ w: v, cell: router.outCell(v), into: router.into(v), extra: router.wayCost(v) }));
      const goal = (i: number, h?: number, loose?: boolean) => {
        let best = -1;
        for (const o of goals) if (o.cell === i && o.into(h, loose) && (best < 0 || o.extra < best)) best = o.extra;
        return best;
      };
      const goalAt = goals.map((o) => land.centre(o.cell));
      const guess = (i: number) => {
        const c = land.centre(i);
        let near = Infinity;
        for (const e of goalAt) near = Math.min(near, dist(c, e));
        return near / CELL;
      };
      const span = dist(w.p, plans[k].centre) || 1;
      const found = router.route(outs, goal, guess, [(plans[k].centre[0] - w.p[0]) / span, (plans[k].centre[1] - w.p[1]) / span], router.costs(ends));
      if (!found) break;
      const end = found.cells[found.cells.length - 1];
      const pb = goals.filter((o) => o.cell === end && o.into(found.h, true)).sort((x, y) => x.extra - y.extra)[0]?.w ?? goals.find((o) => o.cell === end)!.w;
      const road = router.road(w, found.cells, pb, ['join', 'join'], '');
      const bad = router.fault(road);
      if (!made || bad < made.bad) made = { road, pa: w, pb, bad, cells: found.cells };
      if (!bad || !found.steered) break;
      if (tries % 2 === 0) ins = ins.filter((v) => v !== pb);
      else outs = outs.filter((v) => v !== w);
    }
    // (Not over the water.)
    return made && made.bad < 3 ? made : null;
  }
  // Ways out of the world (two or three in the classic worlds): from the ports left over (else any), to the nearest edge.
  // (An island's ways out are its causeways: the country roads to them are laid below.)
  const outs = spec.water === 'island' ? 0 : between(rand, spec.exits);
  const edgeName = ['North', 'East', 'South', 'West'];
  const usedEdges = new Set<number>();
  /** Ways out tried that found no way (a port, an edge: across the sea, say). */
  const noWay = new Map<object, Set<number>>();
  let made = 0;
  /** Each edge of the world (north, east, south, west): whether any of it is dry land. */
  const dryEdge = [0, 1, 2, 3].map((side) => {
    for (let k = 0; k < (side % 2 ? gd : gw); k++) {
      const i = [k, k * gw + gw - 1, (gd - 1) * gw + k, k * gw][side];
      if (!wetNear[i]) return true;
    }
    return false;
  });
  /**
   * A place's land: how much more a road from `p` to edge `side` (`d` m off) would climb than a
   * country road may (m, from the lowest of that edge within `EDGE_WINDOW` m either way of straight
   * out), times `OVER_CLIMB`: the ways out go down the valley or through a pass, not over the
   * massif. 0 in a classic world.
   */
  const overClimb = (p: Pt, side: number, d: number) => {
    if (spec.classic) return 0;
    const along = side % 2 ? p[1] : p[0];
    let low = Infinity;
    for (let t = Math.max(0, along - EDGE_WINDOW); t <= Math.min(side % 2 ? D : W, along + EDGE_WINDOW); t += CELL) {
      const [x, z] = side % 2 ? [side === 1 ? W - 1 : 0, t] : [t, side === 2 ? D - 1 : 0];
      low = Math.min(low, land.at(x, z));
    }
    return Math.max(0, Math.abs(low - land.at(p[0], p[1])) - ROUTE_GRADE * d) * OVER_CLIMB;
  };
  for (let tries = 0; made < outs && tries < outs + 12; tries++) {
    const ports = plans
      .flatMap((plan) => plan.ports)
      .filter((q) => !continued(q))
      .sort((x, y) => Number(x.used) - Number(y.used));
    let best: { q: (typeof ports)[number]; side: number; d: number } | null = null;
    for (const q of ports) {
      for (const [side, d] of [q.p[1], W - q.p[0], D - q.p[1], q.p[0]].entries()) {
        if (!dryEdge[side] || noWay.get(q)?.has(side)) continue;
        // (Not across the river to its far edge.)
        if (river && side % 2 === 0 && bank(q.p) !== (side === 0 ? -1 : 1)) continue;
        const facing = FACING[side];
        const score = d - (q.dir[0] * facing[0] + q.dir[1] * facing[1]) * 80 + (q.used ? 400 : 0) + (usedEdges.has(side) ? 500 : 0) + overClimb(q.p, side, d);
        if (!best || score < best.d) best = { q, side, d: score };
      }
    }
    if (!best) break;
    const { q, side } = best;
    const toward: Pt = [q.p[0] + FACING[side][0] * 400, q.p[1] + FACING[side][1] * 400];
    // (A T off a street facing the edge, if the port doesn't.)
    const tee = q.dir[0] * FACING[side][0] + q.dir[1] * FACING[side][1] < pmath.cos(SIDE_FACING) ? sideT(q.area, toward)[0] : undefined;
    const from: Way = tee ?? router.via(q, toward);
    const onEdge = (i: number) => [Math.floor(i / gw) === 0, i % gw === gw - 1, Math.floor(i / gw) === gd - 1, i % gw === 0][side];
    const ends = from.port ? [from.port.p, from.p] : [];
    const found = router.route([from], (i, h) => (onEdge(i) && (h === undefined || along(h, FACING[side]) > 0.9) ? 0 : -1), (i) => [Math.floor(i / gw), gw - 1 - (i % gw), gd - 1 - Math.floor(i / gw), i % gw][side], FACING[side], router.costs(ends));
    if (!found) {
      // (Another port or edge, then.)
      noWay.set(q, (noWay.get(q) ?? new Set()).add(side));
      continue;
    }
    // (In pieces where it runs along a road laid already.)
    const name = () => uniqueName(`${edgeName[side]} Road`, `${edgeName[side]} Lane`);
    const { laid, first } = router.course(router.toEdge(from, found.cells, side, ''), from, found.cells, null, side);
    for (const road of laid) roads.push({ ...router.settle(road), name: name() });
    if (from.port && first) q.used = true;
    usedEdges.add(side);
    made++;
  }
  // An island's causeways: a country road to each from the nearest port (one nobody took first).
  for (const c of causeways) {
    const way: Way = { p: c.p, dir: c.dir };
    const goalCell = router.outCell(way);
    const comesIn = router.into(way);
    const ports = plans
      .flatMap((plan) => plan.ports)
      .filter((q) => !continued(q))
      .sort((x, y) => Number(x.used) - Number(y.used) || dist(x.p, c.p) - dist(y.p, c.p));
    let laid = false;
    for (const q of ports.slice(0, 4)) {
      const from = router.via(q, c.p);
      const ends = [q.p, from.p, c.p];
      const l = dist(from.p, c.p) || 1;
      const found = router.route([from], (i, h, loose) => (i === goalCell && comesIn(h, loose) ? 0 : -1), (i) => dist(land.centre(i), c.p) / CELL, [(c.p[0] - from.p[0]) / l, (c.p[1] - from.p[1]) / l], router.costs(ends));
      if (!found) continue;
      const name = () => uniqueName(`${plans[q.area].name} Road`, `${plans[q.area].name} Lane`);
      const { laid: pieces, first } = router.course(router.road(from, found.cells, way, ['join', 'join'], ''), from, found.cells, way);
      for (const road of pieces) roads.push({ ...router.settle(road), name: name() });
      if (first && from.port) q.used = true;
      laid = true;
      made++;
      break;
    }
    if (!laid) failed.push(`no country road to the ${roads[c.road].name}`);
  }
  if (made < spec.exits[0]) failed.push(`${made} ways out of the world (${WORDS[spec.exits[0]] ?? spec.exits[0]} or more wanted)`);
  // Ports nobody took: a country road on from each, to the nearest country road, another area's
  // port nobody took or the edge of the world, whichever is nearest (a dead end only if none is).
  /**
   * The country road on from port `q` (of area `k`; -1: a bridge's end, no area's), as it would be
   * laid (`CountryRouter.onward`): to the nearest country road (not its own), another area's port
   * nobody took or the edge of the world. Null if there is none.
   */
  const wayOn = (q: Port, k: number) => router.onward(q, plans.flatMap((o, a) => (a === k ? [] : o.ports.filter((x) => !x.used && !continued(x)))));
  /** Lays the country road on from a port (`wayOn`), named for `place`: whether its first piece leaves the port. */
  const layOn = (on: NonNullable<ReturnType<typeof wayOn>>, place: string) => {
    // (In pieces where it runs along a road laid already.)
    const name = () => uniqueName(`${place} Road`, `${place} Lane`);
    const { laid, first, last } = router.layOnward(on);
    for (const road of laid) roads.push({ ...road, name: name() });
    if (on.made.to && last) on.made.to.used = true;
    return first;
  };
  for (const [k, plan] of plans.entries()) {
    for (const q of plan.ports) {
      if (q.used || continued(q)) continue;
      const on = wayOn(q, k);
      if (!on) {
        // (A place's street running out to the shore, with no way round from there: it ends by the water; or up to
        // land no country road could climb: it ends there.)
        const shore = !spec.classic && [8, 16, 24, 32, 40, 48, 56].some((d) => land.wet(q.p[0] + q.dir[0] * d, q.p[1] + q.dir[1] * d, 4));
        if (!shore && !router.steepOn.has(q)) failed.push(`no country road on from ${plan.name}`);
        continue;
      }
      if (layOn(on, plan.name)) q.used = true;
    }
  }
  // More bridges over a river (a few villages round the town: one; many: two): where it runs
  // straight, well apart from the other bridges and clear of the areas and the roads, the most
  // useful first (the nearest the roads either side); each laid only if a country road goes on
  // from both its ends.
  if (river && spec.options) {
    let more = { none: 0, few: 1, many: 2 }[spec.options.villages];
    const index = new RoadIndex(roads);
    const heading = (x: number) => pmath.atan2(river.z(x + 4) - river.z(x - 4), 8);
    const sites: { a: Pt; b: Pt; score: number }[] = [];
    for (let x = 80; x <= W - 80 && more > 0; x += 8) {
      if (Math.abs(heading(x - 24) - heading(x + 24)) > 0.2) continue;
      const slope = (river.z(x + 1) - river.z(x - 1)) / 2;
      const l = pmath.hypot(1, slope);
      const back = river.half(x) / l + BRIDGE_IN;
      const a: Pt = [round2(x + (slope / l) * back), round2(river.z(x) - back / l)];
      const span = across(river, a, BRIDGE_IN + 4);
      if (!span || span.wet[1] - span.wet[0] + 2 * DECK_PAST > BRIDGE_SPAN) continue;
      const b = span.far;
      const pts = evenly([a, b], 2);
      if ([a, b].some((p) => Math.min(p[0], p[1], W - p[0], D - p[1]) < 40 || land.wet(p[0], p[1], 2))) continue;
      if (pts.some((p) => plans.some((plan) => offRect(plan.rect, p) < BRIDGE_CLEAR) || index.nearest(p[0], p[1], 12))) continue;
      const near = (p: Pt) => index.nearest(p[0], p[1], 300)?.dist ?? 300;
      sites.push({ a, b, score: near(a) + near(b) });
    }
    sites.sort((x, y) => x.score - y.score);
    const mid = ([a, b]: readonly Pt[]): Pt => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
    for (const site of sites) {
      if (more <= 0) break;
      if (crossings.some((c) => dist(mid(c), mid([site.a, site.b])) < BRIDGE_APART)) continue;
      const at = roads.length;
      const u: Pt = [(site.b[0] - site.a[0]) / dist(site.a, site.b), (site.b[1] - site.a[1]) / dist(site.a, site.b)];
      const ends: Port[] = [
        { road: at, end: 0, area: -1, p: site.a, dir: [-u[0], -u[1]], used: false },
        { road: at, end: 1, area: -1, p: site.b, dir: u, used: false },
      ];
      const ons = ends.map((q) => wayOn(q, -1));
      if (ons.some((o) => !o)) continue;
      const placeNear = (p: Pt) => plans.filter((plan) => bank(plan.centre) === bank(p)).sort((x, y) => dist(x.centre, p) - dist(y.centre, p))[0] ?? plans[0];
      const named = placeNear(site.a).name;
      roads.push(crossing(site.a, site.b, ['join', 'join'], uniqueName(`${named} Bridge`, `${named} Old Bridge`)));
      crossings.push([site.a, site.b]);
      ons.forEach((o, e) => {
        if (!layOn(o!, placeNear(ends[e].p).name)) failed.push(`no country road on from ${roads[at].name}`);
      });
      more--;
    }
  }
  // Ports still nobody's (no way on from them): their roads end there.
  for (const plan of plans) {
    for (const q of plan.ports) {
      const road = roads[q.road];
      road.ends = q.end === 0 ? [q.used ? 'join' : 'dead', road.ends[1]] : [road.ends[0], q.used ? 'join' : 'dead'];
      if (q.used) continue;
      // (A bridge stays whole: a bridge to nowhere is a fault.)
      if (road.bridge) {
        failed.push(`no country road on from ${road.name}`);
        continue;
      }
      // A dead end stops inside its area (as a town's do), not out in the fields.
      const r = plan.rect;
      const within = ([x, z]: Pt) => x > r.x + 10 && z > r.z + 10 && x < r.x + r.w - 10 && z < r.z + r.d - 10;
      const pts = q.end === 0 ? [...road.points].reverse() : road.points;
      let keep = pts.length;
      while (keep > 2 && !within(pts[keep - 1])) keep--;
      // (Not back past where another road joins it: a road out laid off it in a T would be left on its own.)
      for (const o of roads) {
        if (o === road) continue;
        for (const e of [o.points[0], o.points[o.points.length - 1]]) {
          for (let k = pts.length - 1; k >= keep; k--) {
            if (dist(pts[k], e) > Math.max(o.width, road.width) / 2 + 1) continue;
            keep = k + 1;
            break;
          }
        }
      }
      if (keep < pts.length) road.points = q.end === 0 ? pts.slice(0, keep).reverse() : pts.slice(0, keep);
    }
  }

  // Ponds, out in the countryside.
  if (spec.water === 'ponds') {
    // (A fixed number draws nothing.)
    const ponds = spec.ponds[0] === spec.ponds[1] ? spec.ponds[0] : between(rand, spec.ponds);
    for (let n = 0, tries = 0; n < ponds && tries < 200; tries++) {
      const r = range(rand, 14, 24);
      const c: Pt = [range(rand, 60, W - 60), range(rand, 60, D - 60)];
      if (plans.some((a) => c[0] > a.rect.x - 50 - r && c[0] < a.rect.x + a.rect.w + 50 + r && c[1] > a.rect.z - 50 - r && c[1] < a.rect.z + a.rect.d + 50 + r)) continue;
      if (Math.min(...plans.map((a) => dist(a.centre, c))) > 200) continue;
      // (Clear of the roads.)
      if (roads.some((road) => nearest(road.points, c[0], c[1]).dist < r * 1.4 + road.width / 2 + 6)) continue;
      for (let i = 0; i < gw * gd && !shaped; i++) {
        const [x, z] = land.centre(i);
        if (dist([x, z], c) < r * (0.85 + fbm(x / 20, z / 20, nseed + 8 + n, 2) * 0.5)) land.water[i] = 1;
      }
      n++;
    }
  }

  step?.('roads');
  // ---- water and woods: zones, the mask the lots keep off -------------------------------------
  // (A river's and an island's sea their own outlines: the river smooth, the sea out to the world's edges with the island a hole in it.)
  // (A shaped land's water is its own: its tiles, its outlines; but a river or a coast as planned is the plan's, as it seeded the land.)
  const keptWater = kept && planRiver ? [planRiver.poly] : kept && plan?.coast ? [islandSea(plan.coast, W, D)] : [];
  const waterPolys = shaped ? keptWater : river ? [river.poly] : coast.length ? [islandSea(coast, W, D)] : outlines(land.water, gw, gd, 2);
  const waterMask = shaped && !keptWater.length ? (wet1m ?? wetTiles(shaped, W, D)) : rasterize(waterPolys, W, D);
  // Woods: the most wooded share of the land by noise, not on the areas, the water or the roads.
  const woodsNoise = new Float32Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    woodsNoise[i] = fbm(x / 105, z / 105, nseed + 7, 4) + (valley ? Math.min(0.25, land.height[i] / 400) : 0);
  }
  const cut = [...woodsNoise].sort((a, b) => b - a)[Math.floor(spec.woods * gw * gd)] ?? 1;
  // (A shaped land's woods are its own, from `fillFrom`.)
  for (let i = 0; i < gw * gd && !shaped; i++) land.woods[i] = woodsNoise[i] > cut && !land.water[i] ? 1 : 0;

  // ---- the lots: one ground over the world, each area's along its own roads -------------------
  const worldRoads = roads;
  // No road out over the world's edge but by an end out of it (`PORT_IN`): cut to the world, it would come apart.
  for (const r of roads) {
    const off = r.points.map(([x, z]) => x < 0 || z < 0 || x > W || z > D);
    const [first, last] = [off.indexOf(false), off.lastIndexOf(false)];
    if (first < 0 || (first > 0 && r.ends[0] !== 'out') || (last < off.length - 1 && r.ends[1] !== 'out') || off.slice(first, last).includes(true)) failed.push(`${r.name ?? 'a road'} runs out over the edge of the world`);
  }
  const allGreens = plans.flatMap((p) => p.greens);
  const ground = new Ground(W, D, worldRoads, allGreens);
  // (A place's lots and buildings keep a strip of bank between them and the water: a way round behind the gardens.)
  ground.block(spec.classic ? waterMask : grown(waterMask, W, D, SHORE_GAP));
  // The living town's own draws (the centre, the terraces, the dense pass), apart from the layout's:
  // the rest of the world is laid out as it was.
  const dense = mulberry32(((lay?.townSeed ?? seed) ^ 0x2545f491) >>> 0);
  // (The centre's buildings of flats from a stream of their own, so the rest is drawn as it was.)
  const centreFlats = mulberry32(((lay?.townSeed ?? seed) ^ 0x51ed270b) >>> 0);
  // The main town's centre: its high street and a square off it, kept clear of lots.
  const centre = chooseCentre(worldRoads, plans[0].roads, plans[0].centre, ground, 0, (x, z) => !spec.classic && land.slope(land.cell(x, z)) > PLACE_STEEP);
  const inCentre = (f: Frontage) => !!centre && inside(centre.poly, f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10);
  const lots = new LotPlacer(content, t, ground, worldRoads, spec.gaps, rand, true);
  const names: (string | undefined)[] = worldRoads.map((r) => r.name);
  const fronts = new Map<number, ReturnType<typeof frontageOf>>();
  const left: { u: Parameters<typeof lots.place>[0][number]; from: number }[] = [];
  // No lot's middle by a junction or on a tight bend (`cornersOf`).
  // (Where the roads meet, worked out once for every road this pass asks about: the areas' and all but the trails.)
  const allRoads = worldRoads.flatMap((r, i) => (r.kind === 'trail' ? [] : [i]));
  const areaRoads = plans.flatMap((plan) => plan.roads);
  const meets = roadMeets(worldRoads, [...allRoads, ...areaRoads]);
  const offCorners = cornersOf(worldRoads, areaRoads, CORNER, meets);
  // The dense pass (`dense.ts`): the main town's middle built up before its houses spread out, every street after.
  const areaList = plans.map((p) => ({ name: p.name, rect: p.rect, centre: p.centre, roads: p.roads }));
  const densePass = spec.classic ? null : new DensePass({ content, t, roads: worldRoads, areas: areaList, lots, centres: centre ? [centre] : [], density: spec.density, civic: spec.civic });
  // ---- the public parking's car parks (`parking.ts`): by the square before any lot, by each big place as it goes down ----
  const parks: CarParkLaid[] = [];
  const offAll = cornersOf(worldRoads, allRoads, CORNER, meets);
  const nearCorners = cornersOf(worldRoads, allRoads, NEAR_CORNER, meets);
  /** A car park of `n` (fewest to most) bays along the first of `frontage` it fits, of area `area` and `also` (`parking.ts` `parkAlong`): whether one went down. */
  const parkBy = (frontage: readonly Frontage[], bays: readonly [number, number], area: number, also: Pick<CarPark, 'serves' | 'square'> = {}) => {
    const laid = parkAlong(worldRoads, ground, frontage, bays, offAll, { area, also });
    if (laid) parks.push(laid);
    return !!laid;
  };
  // (The centre's heart: its square, else its high street's middle.)
  const heart0 = centre && (centre.square ? centroid(centre.square) : pointAt(worldRoads[centre.road].points, (centre.s0 + centre.s1) / 2).p);
  if (centre && heart0 && !spec.classic) {
    // By the square: on the streets nearest it.
    const by = bySquare(worldRoads, frontageOf(worldRoads, allRoads, heart0, dense), heart0);
    // (On a side street first: the high street's frontage is the centre's shops'.)
    // (A row where a street runs along the grid, else a pair: `parking.ts` `carParkFits`.)
    const streets = [...by.filter((f) => f.road !== centre.road), ...by.filter((f) => f.road === centre.road)];
    if (!parkBy(streets, SQUARE_BAYS, 0, { square: true })) parkBy(streets, [1, 2], 0, { square: true });
  }
  /** A car park by each big place (`BIG_PLACES`) laid since the last call, on its road beside its lot. */
  let parkedUpTo = 0;
  const parkBigPlaces = () => {
    if (spec.classic) return;
    for (; parkedUpTo < lots.slots.length; parkedUpTo++) {
      const s = lots.slots[parkedUpTo];
      const f = lots.placed.get(s);
      if (!f || !BIG_PLACES.has(s.template)) continue;
      const r = plotRect(t, s);
      const mid: Pt = [r.x + r.w / 2, r.z + r.d / 2];
      const along = besidePlace(frontageOf(worldRoads, [f.road], mid, dense), f.side, r);
      if (!parkBy(along, PLACE_BAYS, s.area ?? 0, { serves: s.index })) parkBy(along, [1, 2], s.area ?? 0, { serves: s.index });
    }
  };
  // The named places' lots, from a stream of their own (`namedLots`): with no room in their area, along the
  // country roads leaving it, where they run straight over gentle land.
  const placeRand = mulberry32(((lay?.townSeed ?? seed) ^ PLACES) >>> 0);
  const ways = worldRoads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : []));
  /**
   * Each area's named places to go down on what its homes leave once every home has found room: the main
   * town's that went in no parade nor its centre (`units`), every other area's (`namedLots`); along its
   * frontage nearest its heart (`near`).
   */
  const later: { units?: Unplaced[]; near: readonly Frontage[]; a: number }[] = [];
  const spillOf = (a: number) => {
    let list: Frontage[] | undefined;
    return () => (list ??= frontageOf(worldRoads, ways, plans[a].centre, placeRand).filter((f) => offRect(plans[a].rect, f.p) < TERRACE_SPILL && straight(worldRoads[f.road].points, f.s, roadLengths(worldRoads[f.road])) && gentleBy(f.p)));
  };
  for (const [a, plan] of plans.entries()) {
    if (a === 0) {
      const wanted = wantedLots(content, t, spec, plan.spec, rand, { park: true, avoid: [] });
      const all = frontageOf(worldRoads, plan.roads, plan.centre, rand);
      fronts.set(a, all.filter(offCorners));
      // The main town: its public places round its middle, on the high street too; then its
      // terraces off the high street, nearest the middle; then its houses, beyond them.
      const homes = all.filter((f) => !inCentre(f));
      const [places, houses] = [wanted.filter((u) => u.kind !== 'house'), wanted.filter((u) => u.kind === 'house')];
      // (Out of its centre, which is its named places' and the dense pass's: `inCentre`.)
      for (const u of lots.place(places, fronts.get(a)!.filter((f) => !inCentre(f)), plan.greens, plan.centre, a, true)) left.push({ u, from: a });
      // Its named places' lots (`namedLots`) round its heart (its square, else its high street's middle; else its
      // middle), its shops and cafés in parades along its high street; then the rest of its centre built up (`dense.ts`).
      const heart = heart0 ?? plan.centre;
      const near = frontageOf(worldRoads, plan.roads, heart, placeRand);
      const street = centre ? near.filter((f) => f.road === centre.road && f.s >= centre.s0 && f.s <= centre.s1) : near;
      // (Its civic places before its homes, nearest its heart; its other places before them only in its centre,
      // which no home fronts: the rest once every home has found room.)
      const inTheCentre = near.filter(inCentre);
      // (A few buildings of flats on its centre's streets first, at the ends of its parades.)
      densePass?.centreFlats(centreFlats, CENTRE_FLATS[spec.civic]);
      const others = namedLots(t, lots, worldRoads, a, spec.civic, true, near, street, placeRand).filter((u) => !putPlace(lots, t, worldRoads, u, inTheCentre));
      parkBigPlaces();
      // (Out of room in the town: on along the country roads leaving it, where they run straight.)
      const countryWays = worldRoads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : []));
      const out = frontageOf(worldRoads, countryWays, plan.centre, dense).filter((f) => offRect(plan.rect, f.p) < TERRACE_SPILL && straight(worldRoads[f.road].points, f.s, roadLengths(worldRoads[f.road])) && gentleBy(f.p));
      terraceRows(t, lots, [homes, out], spec, dense);
      // (Its middle built up: the centre and the streets round it, `INNER` of the way out to its box's corners.)
      const inner = (INNER * pmath.hypot(plan.rect.w, plan.rect.d)) / 2;
      densePass?.lay(dense, [0], (_, p) => dist(p, heart) < inner);
      fronts.set(a, homes.filter(offCorners));
      for (const u of lots.place(houses, fronts.get(a)!, [], plan.centre, a, true)) left.push({ u, from: a });
      later.push({ units: others, near, a });
    } else {
      // A hamlet: its parking kept by its middle before its lots go down (a pair: more by the verge after them), then
      // its lots along its own roads (`hamletLots`).
      if (!spec.classic) {
        const by = frontageOf(worldRoads, plan.roads, plan.centre, dense);
        if (!parkBy(by, [1, 2], a)) parks.push(...vergeParking(worldRoads, ground, by, 1, a, nearCorners, [], LOOSE_BEND));
      }
      const laid = hamletLots(content, t, spec, plan, a, lots, worldRoads, offCorners, rand);
      // (Its named places' lots once every home has found room.)
      later.push({ near: frontageOf(worldRoads, plan.roads, plan.centre, placeRand), a });
      fronts.set(a, laid.fronts);
      for (const u of laid.left) left.push({ u, from: a });
    }
    const streets = streetNames(content, spec.names, rand, plan.roads.length, [...takenNames, ...roadNames], plan.name);
    takenNames.push(...streets);
    plan.roads.forEach((r, k) => (names[r] = streets[k]));
  }
  // Houses with no room in their own area go to the nearest other that has room (homes, not shops).
  // (In a place's world, out along the country roads only where the land by the road is gentle: `PLACE_STEEP`.)
  let country: ReturnType<typeof frontageOf> | undefined;
  for (const { u, from } of left) {
    const others = plans.map((_, a) => a).filter((a) => a !== from && u.kind === 'house').sort((x, y) => dist(plans[x].centre, plans[from].centre) - dist(plans[y].centre, plans[from].centre));
    const placed = others.some((a) => !lots.place([u], fronts.get(a)!, [], plans[a].centre, a, true).length);
    if (placed) continue;
    // Else along a country road, near its own area's end of it.
    if (!country) {
      // (Off the junctions along them, as a town's lots with a driveway keep, `offAll`: a home's driveway where
      // another road ran into a short country road's end, on Fernshaw's, had a car backing out of it and two
      // coming by wait for each other for good.)
      country = frontageOf(worldRoads, worldRoads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : [])), plans[0].centre, rand).filter((f) => straight(worldRoads[f.road].points, f.s, roadLengths(worldRoads[f.road])) && gentleBy(f.p) && offAll(f));
    }
    const r = plans[from].rect;
    const off = (p: Pt) => offRect(r, p);
    const near = country.filter((f) => off(f.p) < SPILL).sort((x, y) => dist(x.p, plans[from].centre) - dist(y.p, plans[from].centre));
    if (u.kind === 'house' && !lots.place([u], near, [], plans[from].centre, from, true).length) continue;
    lots.dropped.push({ name: u.name, kind: u.kind, lot: u.lot });
  }
  const farmsteads = alongCountryRoads(content, t, lots, plans, worldRoads, land, spec.farmEvery, rand, spec.classic ? STEEP : PLACE_STEEP);
  // The named places that did not go down before the homes, on what the homes leave (the main town's in no
  // parade nor its centre; every other area's, its civic places first, its shops and cafés in parades where
  // they fit). A shop, a café and the like with no room in its own area goes to the nearest other with room,
  // a place of that one's then, as a home does. Then a farm shop by every farm.
  const homeless: Unplaced[] = [];
  for (const { units, near, a } of later) {
    const rest = units ?? namedLots(t, lots, worldRoads, a, areaSize(spec.civic, a), false, near, near, placeRand);
    for (const u of rest) {
      if (putPlace(lots, t, worldRoads, u, near) || putPlace(lots, t, worldRoads, u, spillOf(a)())) continue;
      // (A civic place is its own area's.)
      if ('civic' in (placeOfTemplate(u.template) ?? {})) lots.dropped.push({ name: u.name, kind: u.kind, lot: u.lot, place: true });
      else homeless.push(u);
    }
  }
  for (const u of homeless) {
    const from = plans[u.area ?? 0].centre;
    const others = later.filter((l) => l.a !== (u.area ?? 0)).sort((x, y) => dist(plans[x.a].centre, from) - dist(plans[y.a].centre, from));
    if (!others.some((o) => putPlace(lots, t, worldRoads, { ...u, area: o.a }, o.near))) lots.dropped.push({ name: u.name, kind: u.kind, lot: u.lot, place: true });
  }
  farmShops(t, lots, worldRoads, (a) => (a === 0 ? spec.civic : areaSize(spec.civic, a)), placeRand);
  parkBigPlaces();
  const slots = lots.slots;
  const rectOf = (s: PlotSlot) => plotRect(t, s);

  // ---- parking in each area: pairs of bays by the verge nearest its middle till it has its share ----
  // (Its car parks' bays counted; clear of every driveway's mouth.)
  const drives = driveLines(t, slots, worldRoads, (def) => content.object(def)?.footprint ?? [1, 1]);
  for (const [a, plan] of plans.entries()) {
    const homes = spec.classic ? 0 : slots.filter((s) => s.kind === 'house' && (s.area ?? 0) === a).length;
    const spaces = parkingFor(homes, a === 0);
    const frontage = frontageOf(worldRoads, plan.roads, plan.centre, rand);
    const bays = parks.reduce((n, p) => n + (p.park.area === a ? p.bays.length : 0), 0);
    const pairs = vergeParking(worldRoads, ground, frontage, spaces - bays, a, offAll, drives);
    parks.push(...pairs);
    // (An area whose streets bend too much for its share, or meet too often: single bays on looser stretches, nearer a junction.)
    const short = spaces - bays - pairs.reduce((n, p) => n + p.bays.length, 0);
    if (short > 0) parks.push(...vergeParking(worldRoads, ground, frontage, short, a, nearCorners, drives, LOOSE_BEND));
  }

  // (A trail keeps off the water and the car parks.)
  const keepOff = waterMask.slice();
  for (const p of parks) tilesOf(p.park.poly, 0.5, (x, z) => {
    if (x >= 0 && z >= 0 && x < W && z < D) keepOff[z * W + x] = 1;
  });
  // ---- outdoor places, a trail to each from its area's roads ----------------------------------
  const outdoor = new Map(t.parks.filter((p) => p.kind === 'outdoor').map((p) => [p.id, p]));
  const pads: Record<number, number> = {};
  let next = 0;
  for (const [a, plan] of plans.entries()) {
    const count = between(rand, spec.spotsPerArea) + (a === 0 ? 1 : 0);
    for (let n = 0; n < count; n++) {
      const id = spec.spots[next++ % spec.spots.length];
      const place = outdoor.get(id);
      if (!place) continue;
      const spot = siteFor(place, plan, land, ground, waterMask, t, rand, W, D, spec.classic ? Infinity : SPOT_REACH);
      if (!spot) continue;
      const slot = lots.put({ kind: 'outdoor', template: place.id, name: `${plan.name} ${place.name}`, lot: 'medium', w: spot.w, d: spot.d, area: a }, spot, null);
      const r = rectOf(slot);
      pads[slot.index] = Math.round(land.at(r.x + r.w / 2, r.z + r.d / 2) * 10) / 10;
      const trail = trailTo(slot, plan, worldRoads, slots, keepOff, land, t, W, D);
      // (No way to it: no place there, then. It is the last lot put down.)
      if (!trail) {
        slots.pop();
        delete pads[slot.index];
        continue;
      }
      trail.name = uniqueName(`${slot.name} Trail`, `${slot.name} Path`);
      trail.area = a;
      worldRoads.push(trail);
      // (Nothing else goes down on it.)
      for (const [x, z] of trail.points) ground.take({ x: Math.floor(x) - 1, z: Math.floor(z) - 1, w: 3, d: 3 });
    }
  }

  // ---- orchards: rows of fruit trees beside each farm (behind it, else to one side) ------------
  const orchards: Rect[] = [];
  const wetIn = (r: Rect) => [0, 0.5, 1].some((u) => [0, 0.5, 1].some((v) => land.wet(r.x + r.w * u, r.z + r.d * v, 0)));
  for (const s of slots) {
    if (s.lot !== 'farm') continue;
    const r = rectOf(s);
    const [g, o] = [ORCHARD_GAP, ORCHARD];
    // Behind the lot (away from its road: the front is its min z, min x, max z, max x as it faces 0..3), then its sides.
    const back = [{ x: r.x, z: r.z + r.d + g, w: r.w, d: o }, { x: r.x + r.w + g, z: r.z, w: o, d: r.d }, { x: r.x, z: r.z - g - o, w: r.w, d: o }, { x: r.x - g - o, z: r.z, w: o, d: r.d }];
    const tries = [back[s.facing], back[(s.facing + 1) % 4], back[(s.facing + 3) % 4]];
    const at = tries.find((q) => ground.free(q, 8) && !wetIn(q));
    if (!at) continue;
    ground.take(at, 2);
    orchards.push(at);
  }

  // ---- the dense pass: every street built up with real lots, the centre shoulder to shoulder (`dense.ts`) ----
  densePass?.lay(dense);
  // (And out along the country roads into each area, as far as `APPROACH` m past its box, where the road runs straight over gentle land.)
  // (Into a village only inside its box: outskirts past it would tie it to the country road, and step 2 could no longer move it.)
  densePass?.approaches(dense, ways, (a, p) => offRect(plans[a].rect, p) < (a === 0 ? APPROACH : 0.5) && gentleBy(p));
  // (Flats over the shops, cafés and restaurants, real floors over the offices and halls: `dense.ts` `raisePlaces`.)
  if (!spec.classic) raisePlaces(t, lots.slots, centre ? [centre.poly] : [], dense);
  // No lot whose front door gets no path to a road (on a bend, its door looking past the road): left out
  // (`diagnostics.dropped`), the lots after it numbered again with what names them (the pads, the car parks' `serves`).
  const roadIdx = roadIndex(worldRoads);
  const rects = slots.map(rectOf);
  const pathless = new Set(slots.filter((s) => !reachesRoad(content, t, roadIdx, rects, s)).map((s) => s.index));
  if (pathless.size) {
    for (const s of slots) if (pathless.has(s.index)) lots.dropped.push({ name: s.name, kind: s.kind, ...(s.lot ? { lot: s.lot } : {}) });
    const kept = slots.filter((s) => !pathless.has(s.index));
    const now = new Map(kept.map((s, k) => [s.index, k]));
    const was = { ...pads };
    for (const k of Object.keys(pads)) delete pads[Number(k)];
    for (const [k, h] of Object.entries(was)) if (now.has(Number(k))) pads[now.get(Number(k))!] = h;
    for (const p of parks) {
      if (p.park.serves === undefined) continue;
      const to = now.get(p.park.serves);
      if (to === undefined) delete p.park.serves;
      else p.park.serves = to;
    }
    slots.length = 0;
    kept.forEach((s, k) => slots.push(Object.assign(s, { index: k })));
  }
  lots.number(names);
  // The bays, every one clear of every driveway (a lot laid after its car park may have one by it, across the street too).
  const lines = driveLines(t, slots, worldRoads, (def) => content.object(def)?.footprint ?? [1, 1]);
  const { bays: parking, carParks } = clearOfDrives(parks, lines);
  // (An area whose bays all went: a pair, or one, by the verge where none is by a driveway now.)
  for (const [a, plan] of plans.entries()) {
    if (spec.classic || parking.some((b) => b.area === a)) continue;
    for (const p of vergeParking(worldRoads, ground, frontageOf(worldRoads, plan.roads, plan.centre, rand), 2, a, nearCorners, lines, LOOSE_BEND)) {
      parking.push(...p.bays.map((b) => ({ ...b, park: carParks.length })));
      carParks.push(p.park);
    }
  }

  // ---- the named places' names: each lot as its place is called, none twice -------------------
  nameLots(content, slots, plans, worldRoads, spec.civic, mulberry32(((lay?.townSeed ?? seed) ^ PLACE_NAMES) >>> 0));

  // ---- the centre made the densest: narrowed to its high street where the land round it is the emptier ----
  if (centre && !spec.classic) densestCentre(centre, worldRoads, W, D, waterMask, allGreens, slots.map((s) => ({ rect: rectOf(s), built: !!s.dress && (s.kind === 'shop' || s.kind === 'flats') })), plans[0].rect);

  // ---- the places the tags name ---------------------------------------------------------------
  const places = townPlaces(content, t, worldRoads, areaList, slots, centre ? [centre.area] : [], (a) => (a === 0 ? spec.civic : areaSize(spec.civic, a)), dense);

  // ---- zones -------------------------------------------------------------------------------
  const clearAround = (mask: Uint8Array, r: Rect, grow: number) => {
    for (let z = Math.floor((r.z - grow) / CELL); z <= Math.floor((r.z + r.d + grow) / CELL); z++) for (let x = Math.floor((r.x - grow) / CELL); x <= Math.floor((r.x + r.w + grow) / CELL); x++) if (x >= 0 && z >= 0 && x < gw && z < gd) mask[z * gw + x] = 0;
  };
  const woods = land.woods.slice();
  /** Where the town clears the woods (0), for a shaped land's cover. */
  const wild = new Uint8Array(gw * gd).fill(1);
  const clearWoods = (r: Rect, grow: number) => {
    clearAround(woods, r, grow);
    clearAround(wild, r, grow);
  };
  // (Wooded: round a hamlet the woods come up to its roads and lots.)
  for (const [a, plan] of plans.entries()) if (!spec.wildBoxes || (a === 0 && spec.wildBoxes === 'hamlets')) clearWoods(plan.rect, 10);
  for (const s of slots) clearWoods(rectOf(s), 4);
  for (const r of orchards) clearWoods(r, 4);
  // (A motorway's and a ramp's kept land, `keepOf`, wider: its embankment, no trees on it.)
  for (const road of worldRoads) for (let i = 0; i < road.points.length; i += 3) clearWoods({ x: road.points[i][0], z: road.points[i][1], w: 0, d: 0 }, road.width / 2 + Math.max(3, keepOf(road)));
  const heights = [...land.height].sort((x, y) => x - y);
  const hillCut = heights[Math.floor(heights.length * (valley ? 0.62 : 0.82))];
  const hills = new Uint8Array(gw * gd);
  const meadow = new Uint8Array(gw * gd);
  const fields = new Uint8Array(gw * gd);
  /** Farmland: round each area's box (as far as `spec.fields` past it) and each farmstead (a little less from its middle). */
  const reach = spec.fields ?? 0;
  const farmMiddles = farmsteads.map((s): Pt => {
    const r = rectOf(s);
    return [r.x + r.w / 2, r.z + r.d / 2];
  });
  const nearFarmland = (p: Pt) =>
    plans.some((a) => offRect(a.rect, p) < reach) || farmMiddles.some((m) => dist(m, p) < reach * 1.1);
  for (let i = 0; i < gw * gd; i++) {
    if (land.water[i]) continue;
    const [x, z] = land.centre(i);
    if (land.height[i] > hillCut) hills[i] = 1;
    if (!woods[i] && !areaBlock[i] && fbm(x / 70, z / 70, nseed + 11, 3) > (spec.meadowCut ?? 0.58)) meadow[i] = 1;
    if (spec.fields && !woods[i] && !areaBlock[i] && !meadow[i] && nearFarmland([x, z]) && fbm(x / 110, z / 110, nseed + 13, 3) > spec.fieldsCut) fields[i] = 1;
  }
  for (const plan of plans) clearAround(hills, plan.rect, 0);
  for (const road of worldRoads) {
    for (let i = 0; i < road.points.length; i += 3) {
      const at = { x: road.points[i][0], z: road.points[i][1], w: 0, d: 0 };
      clearAround(fields, at, road.width / 2 + Math.max(2, keepOf(road)));
      clearAround(meadow, at, road.width / 2 + Math.max(1, keepOf(road)));
    }
  }
  for (const r of [...slots.map(rectOf), ...orchards, ...carParks.map((p) => boundsOf(p.poly))]) {
    clearAround(fields, r, 2);
    clearAround(meadow, r, 2);
  }
  let zones: WorldZone[] = [
    ...waterPolys.map((poly): WorldZone => ({ kind: 'water', poly })),
    ...outlines(hills, gw, gd, 12).map((poly): WorldZone => ({ kind: 'hill', poly })),
    ...outlines(fields, gw, gd, 10).map((poly): WorldZone => ({ kind: 'fields', poly })),
    ...outlines(meadow, gw, gd, 6).map((poly): WorldZone => ({ kind: 'meadow', poly })),
    ...outlines(woods, gw, gd, 8).map((poly): WorldZone => ({ kind: 'woods', poly })),
    ...orchards.map((r): WorldZone => ({ kind: 'orchard', poly: [[r.x, r.z], [r.x + r.w, r.z], [r.x + r.w, r.z + r.d], [r.x, r.z + r.d]] })),
  ];

  // A shaped land: graded and cleared under what is built, the town's farmland on it, its water and cover the zones (the hills and orchards still the layout's).
  let laid: LandField | undefined;
  if (shaped) {
    laid = settleTown(shaped, { width: W, depth: D, plots: [...slots.map(rectOf), ...orchards], roads: worldRoads });
    townCover(laid, land, { wild, fields, areas: areaBlock });
    const own = keptWater.length ? [...keptWater.map((poly): WorldZone => ({ kind: 'water', poly })), ...landZones(laid, { cover: true })] : landZones(laid);
    zones = (['water', 'hill', 'fields', 'meadow', 'woods', 'orchard'] as const).flatMap((kind) => (kind === 'hill' || kind === 'orchard' ? zones : own).filter((z) => z.kind === kind));
  }

  const areas: TownArea[] = plans.map((p) => ({ name: p.name, kind: 'hamlet', x: p.rect.x, z: p.rect.z, w: p.rect.w, d: p.rect.d }));
  // (The landscape round the world: its sea or lake is in the zones now. A place's from its own
  // stream; its mountains as high as the massif.)
  const { water: _water, ...landscape } = spec.landscape(spec.classic ? rand : mulberry32((seed ^ LOOK) >>> 0), W, D, waterSide);
  if (!spec.classic && massif) landscape.mountains = Math.round(massif);
  const relief = spec.classic ? undefined : reliefOf(land, slots.map(rectOf), shaped ? 0 : undefined);
  const signed = spec.classic || shaped ? undefined : heightsOf(land, slots.map(rectOf), worldRoads);
  // The roads cut to the world; each bridge's and causeway's deck over its water (and a little
  // past it), and the water the sim walks round without the decks.
  const filed = cropRoads(worldRoads, 0, 0, W, D);
  let walkWater = waterMask;
  if (filed.some((r) => r.bridge)) {
    walkWater = waterMask.slice();
    for (const r of filed) if (r.bridge) r.bridge = deckOf(r, waterMask, walkWater, W, D);
  }
  step?.('homes');
  return {
    name: name ?? rolledName,
    width: W,
    depth: D,
    slots,
    roads: filed,
    greens: allGreens.map((g) => g.map(([x, z]): Pt => [round2(x), round2(z)])),
    households: [],
    seed,
    world: {
      kind: spec.kind,
      size: 'big',
      tagline,
      landscape,
      ...(spec.options ? { options: spec.options } : {}),
      ...(lay?.townSeed !== undefined && lay.townSeed !== seed ? { townSeed: lay.townSeed } : {}),
      ...(shaped?.edited ? { shaped: true as const } : {}),
    },
    diagnostics: { dropped: lots.dropped, ...(failed.length ? { roads: failed } : {}), ...(reachable && missing > 0 ? { villages: { left: missing, homes: missing * spec.hamlet.lots[0] } } : {}), ...(unreached.length ? { pieces: unreached } : {}) },
    open: {
      areas,
      zones,
      water: rle(walkWater),
      pads,
      parking,
      ...(carParks.length ? { carParks } : {}),
      places,
      ...(centre ? { centre: [{ area: centre.area, poly: centre.poly, ...(centre.square ? { square: centre.square } : {}) }], squares: centre.square ? [centre.square] : [], highStreets: [centre] } : {}),
      ...(relief ? { relief } : {}),
      ...(signed ? { heights: signed } : {}),
      ...(laid ? { land: laid } : {}),
      ...(plan ? { plan } : {}),
      ...(shaped ? { site: [ox + box.w / 2, oz + box.d / 2] as Pt } : {}),
    },
  };
}

// ---- the generator's pieces a village placed by hand reuses -----------------------------------

/** A world being laid out: its size, its roads and its areas so far (`putArea` adds to both). */
export interface Laying {
  W: number;
  D: number;
  roads: RoadPlan[];
  plans: AreaPlan[];
}

/**
 * An area put on the world being laid (`laying`): its roads (`laid`, laid out in its own box
 * `local`) moved to (`x`, `z`) and turned across if `swap`, each at least `ROAD_WIDTH` wide, its
 * first a street and the rest lanes, all the area's; a road that led out of its box cut back to
 * `PORT_IN` m inside the world's edge and kept as one of its ways on (`ports`). Its roads are added
 * to `laying.roads` and it to `laying.plans` (its index is the number of areas before it).
 */
export function putArea(laying: Laying, areaName: string, laid: readonly RoadPlan[], local: Box, x: number, z: number, swap: boolean, size: SizeSpec, greens: Pt[][], centre: Pt): AreaPlan {
  const { W, D, roads, plans } = laying;
  const onto = ([px, pz]: Pt): Pt => (swap ? [x + pz, z + px] : [x + px, z + pz]);
  const index = plans.length;
  const plan: AreaPlan = {
    name: areaName,
    rect: swap ? { x, z, w: local.d, d: local.w } : { x, z, w: local.w, d: local.d },
    centre: onto(centre),
    roads: [],
    greens: greens.map((g) => g.map(onto)),
    spec: size,
    ports: [],
  };
  /** Whether a point is `PORT_IN` or further inside the world. */
  const inWorld = ([px, pz]: Pt) => Math.min(px, pz, W - px, D - pz) >= PORT_IN;
  laid.forEach((r, k) => {
    // (Each way out cut back from the world's edge.)
    const all = r.points.map(onto);
    let [from, to] = [0, all.length];
    if (r.ends[1] === 'out') while (to - from > 2 && !inWorld(all[to - 1])) to--;
    if (r.ends[0] === 'out') while (to - from > 2 && !inWorld(all[from])) from++;
    const points = all.slice(from, to);
    const road: RoadPlan = { ...r, points, width: Math.max(r.width, ROAD_WIDTH), kind: k === 0 ? 'street' : 'lane', area: index, ...(r.lots && swap ? { lots: (-r.lots) as 1 | -1 } : {}) };
    const at = roads.push(road) - 1;
    plan.roads.push(at);
    for (const e of [0, 1] as const) {
      if (r.ends[e] !== 'out') continue;
      const [p, q] = e === 0 ? [points[0], points[1]] : [points[points.length - 1], points[points.length - 2]];
      const l = dist(p, q) || 1;
      plan.ports.push({ road: at, end: e, area: index, p, dir: [(p[0] - q[0]) / l, (p[1] - q[1]) / l], used: false });
    }
  });
  plans.push(plan);
  return plan;
}

/** A hamlet laid out in its own box (`hamletOf`): its name, its roads, its layout (its greens and middle), its size (the lots it wants). */
export interface Hamlet {
  name: string;
  laid: RoadPlan[];
  layout: RoadLayout;
  size: SizeSpec;
}

/**
 * A hamlet of `spec`'s place, from `rand` in the generator's order: its streets (the `n`th of the
 * place's hamlet shapes, in turn, in its box `spec.hamlet.box`; joined up and settled: `joinUp`,
 * `settleRoads`), its name (drawn again while another area has it, then with "End" after it:
 * `taken`, the other areas' names), and how many lots it wants. Not yet on the world: `putHamlet`.
 */
export function hamletOf(content: Content, spec: WorldSpec, rand: Rand, n: number, taken: readonly string[]): Hamlet {
  const { shapes, scale: hs, fill, extend, box: hb } = spec.hamlet;
  const layout = shapes[n % shapes.length](rand, hb, hs);
  const laid = settleRoads(joinUp(rand, layout, { box: hb, lots: 0, scale: hs, fill, extend }, COURTS), hb);
  let name = worldName(content, spec.names, rand);
  for (let k = 0; k < 12 && taken.includes(name); k++) name = worldName(content, spec.names, rand);
  if (taken.includes(name)) name = `${name} End`;
  const size: SizeSpec = { box: hb, lots: between(rand, spec.hamlet.lots), scale: hs, places: spec.hamlet.places, farms: spec.hamlet.farms ?? [0, 0] };
  return { name, laid, layout, size };
}

/** Hamlet `h` put on the world being laid (`putArea`), its box's corner at (`x`, `z`), turned across if `swap`. */
export function putHamlet(laying: Laying, h: Hamlet, x: number, z: number, swap: boolean): AreaPlan {
  const hb = h.size.box;
  return putArea(laying, h.name, h.laid, hb, x, z, swap, h.size, h.layout.greens ?? [], h.layout.centre ?? [hb.w / 2, hb.d / 2]);
}

/**
 * What a hamlet's site is tested against (`hamletFits`): the world's water (a river's or an
 * island's keeps it 24 m from its box), the other areas' boxes, the pieces of dry land it may go on
 * on a land whose water was changed (`region`: `landPieces.ts`), the bridges' and causeways' ends
 * (`crossings`), and how far the generator has given up on a flat site (`relax`).
 */
export interface HamletSite {
  water: WorldSpec['water'];
  boxes: readonly Rect[];
  region?: { holds: (r: Rect, by: number) => boolean } | null;
  crossings?: readonly (readonly Pt[])[];
  relax?: number;
}

/**
 * Whether a hamlet's box `bw` × `bd` round `p` may go there, as the generator tests a site: in the
 * world by `EDGE`, clear of the other areas' boxes by `BOX_GAP`, dry (on a land whose water was
 * changed, 24 m from any of it and on a piece of dry land it may go on: `site.region`), 50 m clear of the bridges and causeways, and no steeper than 0.35 anywhere
 * in it (unless `site.relax`). Its box (whole metres) and its steepest slope; null if it may not.
 */
export function hamletFits(land: Cells, p: Pt, [bw, bd]: readonly [number, number], site: HamletSite): { rect: Rect; steep: number } | null {
  const { W, D } = land;
  const relax = site.relax ?? 0;
  const rect = { x: Math.round(p[0] - bw / 2), z: Math.round(p[1] - bd / 2), w: bw, d: bd };
  if (rect.x < EDGE || rect.z < EDGE || rect.x + bw > W - EDGE || rect.z + bd > D - EDGE) return null;
  if (site.boxes.some((a) => rect.x < a.x + a.w + BOX_GAP && a.x < rect.x + rect.w + BOX_GAP && rect.z < a.z + a.d + BOX_GAP && a.z < rect.z + rect.d + BOX_GAP)) return null;
  // (On a land whose water was changed, its water is the land's own, a pond as much as a river: 24 m from the box.)
  if (site.region ? land.wetNear(rect, 24) || !site.region.holds(rect, 24) : site.water === 'river' || site.water === 'island' ? land.wetNear(rect, 24) : land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 30)) return null;
  // (Clear of the bridges and causeways.)
  if ((site.crossings ?? []).some(([a, b]) => evenly([a, b], 4).some((q) => offRect(rect, q) < 50))) return null;
  let steep = 0;
  for (let z = rect.z; z < rect.z + bd; z += CELL * 2) for (let x = rect.x; x < rect.x + bw; x += CELL * 2) steep = Math.max(steep, land.slope(land.cell(x, z)));
  if (steep > 0.35 - relax * 0.1 && !relax) return null;
  return { rect, steep };
}

/**
 * Whether a lot's middle may be at frontage `f` on `roads`, as an area's lots keep: `CORNER` m along
 * its road from where it meets another (its joined ends, and where another road's end joins it or
 * another runs by close enough to touch: a car turning out of a driveway there swings across the
 * corner), and off a tight bend (`TIGHT_BEND`). Worked out for the areas' roads `areaRoads`; `corner` m from a junction (`CORNER` unless given).
 */
export function cornersOf(roads: readonly RoadPlan[], areaRoads: Iterable<number>, corner = CORNER, meets = roadMeets(roads, areaRoads)): (f: Frontage) => boolean {
  const own = new Set(areaRoads);
  // (Nor on a tight bend: out of a driveway there a car swings off the road.)
  return (f: Frontage) => {
    if (!((own.has(f.road) && meets.get(f.road)) || []).every((s) => Math.abs(f.s - s) >= corner)) return false;
    return straightAround(roads[f.road].points, f.s, corner / 2, TIGHT_BEND, roadLengths(roads[f.road]));
  };
}

/**
 * Hamlet `plan`'s lots (area `a`), from `rand` in the generator's order: the lots it wants of the
 * place's mix (`wantedLots`: no market), put along its own roads' frontage off the corners
 * (`offCorners`, from `cornersOf`), nearest its middle first, the first on a green if one fits
 * (`LotPlacer.place`). Its frontage, and the lots with no room there.
 */
export function hamletLots(content: Content, t: Templates, spec: WorldSpec, plan: AreaPlan, a: number, lots: LotPlacer, roads: readonly RoadPlan[], offCorners: (f: Frontage) => boolean, rand: Rand): { fronts: Frontage[]; left: Unplaced[] } {
  const wanted = wantedLots(content, t, spec, plan.spec, rand, { park: false, avoid: ['market'] });
  const fronts = frontageOf(roads, plan.roads, plan.centre, rand).filter(offCorners);
  return { fronts, left: lots.place(wanted, fronts, plan.greens, plan.centre, a, true) };
}

/**
 * Named place lot `u` (`placeLots.ts` `placeUnit`) put down along the first of `frontage` it fits, as
 * `lotAt` lays a lot but not snug (it has no driveway whose end must meet the road): the middle of its
 * front within `PLACE_REACH` m of its road's edge, the land a metre round it taken. Its slot, or null.
 */
export function putPlace(lots: LotPlacer, t: Templates, roads: readonly RoadPlan[], u: Unplaced, frontage: readonly Frontage[]): PlotSlot | null {
  const { width, depth } = slotDims(t, u);
  for (const f of frontage) {
    const road = roads[f.road];
    const at = lotAt(lots.ground, road, f, width, depth);
    if (!at || !placeReaches(t, road, u, at)) continue;
    const r = plotRect(t, { ...u, ...at });
    const slot: PlotSlot = { ...u, index: lots.slots.length, ...at };
    lots.slots.push(slot);
    lots.placed.set(slot, f);
    lots.ground.take(r, 1);
    return slot;
  }
  return null;
}

/**
 * Area `a`'s named places as lots (`placeLots.ts`), each a place of `size`'s (its layout and name), drawn
 * from `rand`: its civic places (`CIVIC`: a main town's, `main`, those of its size, `Civic.least`;
 * any other area's a hamlet's) put down along `near` (its frontage nearest its heart first: `putPlace`),
 * and its other places (`ZONE`, on their compact layouts in a main town: of each kind as many as `ZONE.lots` has for a main town of its size,
 * `ZONE.around` for any other area), its shops and cafés in parades along `street` where they fit
 * (`paraded`). Those not put down yet, to go one by one: the civic places first, then a restaurant, the
 * shops and cafés, the offices, the workshops.
 */
export function namedLots(t: Templates, lots: LotPlacer, roads: readonly RoadPlan[], a: number, size: TownSize, main: boolean, near: readonly Frontage[], street: readonly Frontage[], rand: Rand): Unplaced[] {
  const civic = CIVIC.flatMap((c) => (civicTooBig(c, main ? size : 'hamlet') ? [] : [{ ...placeUnit(t, c, size, rand), area: a }])).filter((u) => !putPlace(lots, t, roads, u, near));
  // (A main town's on its compact layouts: its high street built up shoulder to shoulder.)
  const of = (k: ZoneKind) => Array.from({ length: between(rand, main ? ZONE[k].lots[size] : ZONE[k].around) }, (): Unplaced => ({ ...placeUnit(t, k, size, rand, main), area: a }));
  const [shops, cafes, restaurants, offices, workshops] = (['shop', 'cafe', 'restaurant', 'office', 'workshop'] as const).map(of);
  return [...civic, ...restaurants, ...paraded(lots, shuffled([...shops, ...cafes], rand), street, a, rand), ...offices, ...workshops];
}

/**
 * Of `units` (named places' lots of area `a`, all of one depth: shops and cafés), as many as fit put down
 * in parades of two to four along `street` (`LotPlacer.placeRow`: nearest its start first), each as long
 * as `rand` draws else shorter, down to a pair. Those left (the first a parade found no room for on).
 */
export function paraded(lots: LotPlacer, units: readonly Unplaced[], street: readonly Frontage[], a: number, rand: Rand): Unplaced[] {
  const left = units.slice();
  while (left.length >= 2 && street.length) {
    let row: PlotSlot[] = [];
    for (let n = Math.min(left.length, between(rand, [2, 4])); n >= 2 && !row.length; n--) row = lots.placeRow(left.slice(0, n), street, a);
    if (!row.length) break;
    left.splice(0, row.length);
  }
  return left;
}

/**
 * A farm shop (`ZONE.farm`, a place of `sizeOf` its area's size) by every farm lot (a farmstead out in the
 * country, a farm in an area), along its road within about `FARM_SHOP_REACH` m of its middle, called after
 * the farm (`Meadow Farm Shop`), drawn from `rand`. One with no room by its farm is listed in `lots.dropped`.
 */
export function farmShops(t: Templates, lots: LotPlacer, roads: readonly RoadPlan[], sizeOf: (area: number) => TownSize, rand: Rand): void {
  for (const s of lots.slots.filter((q) => q.lot === 'farm')) {
    const f = lots.placed.get(s);
    if (!f) continue;
    const r = plotRect(t, s);
    const near = frontageOf(roads, [f.road], [r.x + r.w / 2, r.z + r.d / 2], rand).filter((g) => g.far < FARM_SHOP_REACH);
    const a = s.area ?? 0;
    const u: Unplaced = { ...placeUnit(t, 'farm', sizeOf(a), rand), area: a, name: `${s.name} Shop` };
    if (!putPlace(lots, t, roads, u, near)) lots.dropped.push({ name: u.name, kind: u.kind, lot: u.lot, place: true });
  }
}

/**
 * Every named place's lot of `slots` called as its place is (`placeLots.ts` `lotName`; a farm shop after its
 * farm), none as another lot is nor twice (`namerFor`: the names from its area's, `plans`, its streets
 * among `roads`, of a place of the main town's size `main`, else of its area's, `areaSize`), drawn from
 * `rand`. Its place is called the same (`dense.ts` `townPlaces`).
 */
export function nameLots(content: Content, slots: readonly PlotSlot[], plans: readonly AreaPlan[], roads: readonly RoadPlan[], main: TownSize, rand: Rand): void {
  const taken = new Set(slots.flatMap((s) => (s.dress ? [] : [s.name])));
  const lastNames = content.names.last.length ? content.names.last : ['Smith'];
  for (const s of slots) {
    if (!s.dress || s.kind !== 'shop') continue;
    const a = s.area ?? 0;
    const streets = plans[a].roads.map((r) => roads[r].name).filter((n): n is string => !!n && !/Trail|Path/.test(n));
    const namer = namerFor(rand, plans[a].name, a === 0 ? main : areaSize(main, a), lastNames, streets, taken);
    const what = placeOfTemplate(s.template);
    s.name = what && 'zone' in what && what.zone === 'farm' ? namer.unique(() => s.name) : (lotName(s, namer) ?? s.name);
  }
}

/** A shaped land's 8 m cells, as the generator and the country roads' router read it (`fillFrom`). */
export function cells8(land: LandField): Cells {
  const cells = new Cells(land.cols * LAND_CELL, land.rows * LAND_CELL);
  fillFrom(cells, land);
  return cells;
}

/**
 * A shaped land's 8 m cells (`generateFromSpec`'s `shaped`): each the mean height of its samples
 * with the finer hills smoothed out (`landFollow.ts` `smoothHeights`: as the land is once it has
 * yielded to a town), water if any of them is wet, wooded if most of its cover is woods.
 */
function fillFrom(land: Cells, field: LandField): void {
  const smooth = smoothHeights(field);
  const k = CELL / LAND_CELL;
  const c = CELL / COVER_CELL;
  const cw = coverSpan(field.cols);
  const cd = coverSpan(field.rows);
  for (let cz = 0; cz < land.gd; cz++) {
    for (let cx = 0; cx < land.gw; cx++) {
      let [sum, n, wet] = [0, 0, 0];
      for (let j = cz * k; j < Math.min(field.rows, cz * k + k); j++) {
        for (let i = cx * k; i < Math.min(field.cols, cx * k + k); i++) {
          const s = j * field.cols + i;
          sum += smooth[s];
          n++;
          if (field.wl[s] !== NONE && field.wl[s] > field.h[s]) wet = 1;
        }
      }
      let [woods, cells] = [0, 0];
      for (let r = cz * c; r < Math.min(cd, cz * c + c); r++) {
        for (let q = cx * c; q < Math.min(cw, cx * c + c); q++) {
          cells++;
          if (field.cover[r * cw + q] === COVER.woods) woods++;
        }
      }
      const i = cz * land.gw + cx;
      land.height[i] = n ? sum / n : 0;
      land.water[i] = wet;
      land.woods[i] = woods * 2 > cells && !wet ? 1 : 0;
    }
  }
}

/** Why a town can't be laid on a shaped land cut into pieces by its water (`dry`): no piece holds its box. */
const noRoom = (box: Box) => `There is no dry land big enough for the town: it needs ${box.w} × ${box.d} m in one piece.`;

/**
 * A land's dry land (8 m cells), for a town on a land whose water is as planned: the place nearest
 * a box's (8 m steps) where it and the land `by` m round it stand all on dry land, within bounds
 * (`nearest`: [x0, z0, x1, z1] of its corner). (A land whose water was changed: `landPieces.ts`.)
 */
function dryLand(land: Cells): { nearest: (r: Rect, bounds: Bounds, by?: number) => Pt | null } {
  const { gw, gd } = land;
  // Wet cells, summed from the corner.
  const sat = new Int32Array((gw + 1) * (gd + 1));
  for (let z = 0; z < gd; z++) {
    let row = 0;
    for (let x = 0; x < gw; x++) {
      row += land.water[z * gw + x] ? 1 : 0;
      sat[(z + 1) * (gw + 1) + x + 1] = sat[z * (gw + 1) + x + 1] + row;
    }
  }
  const holds = (r: Rect, by: number) => {
    const [x0, z0] = [Math.floor((r.x - by) / CELL), Math.floor((r.z - by) / CELL)];
    const [x1, z1] = [Math.floor((r.x + r.w + by) / CELL) + 1, Math.floor((r.z + r.d + by) / CELL) + 1];
    if (x0 < 0 || z0 < 0 || x1 > gw || z1 > gd) return false;
    return sat[z1 * (gw + 1) + x1] - sat[z0 * (gw + 1) + x1] - sat[z1 * (gw + 1) + x0] + sat[z0 * (gw + 1) + x0] === 0;
  };
  const nearest = (r: Rect, [bx0, bz0, bx1, bz1]: Bounds, by = CELL): Pt | null => {
    // (Where it is, if it stands there.)
    if (r.x >= bx0 && r.z >= bz0 && r.x <= bx1 && r.z <= bz1 && holds(r, by)) return [Math.round(r.x), Math.round(r.z)];
    let best: Pt | null = null;
    let far = Infinity;
    for (let z = bz0; z <= bz1; z += CELL) {
      for (let x = bx0; x <= bx1; x += CELL) {
        const d = pmath.hypot(x - r.x, z - r.z);
        if (d >= far || !holds({ x, z, w: r.w, d: r.d }, by)) continue;
        [best, far] = [[Math.round(x), Math.round(z)], d];
      }
    }
    return best;
  };
  return { nearest };
}

/** The ground round what is built rises back to a shaped land's own height over this far (m) past the flat apron. */
const GRADE_RISE = 40;
/** A bank kept this far over the water beside it (cm) where the grading lowers the land. */
const GRADE_BANK = 25;

/**
 * A shaped land with a town laid on it (`generateFromSpec`'s `shaped`): a copy, flat at 0 under
 * what is built and rising back to its own height round it (as the renderer draws a world's
 * land round its towns), but never below a bank over the water beside it; and its cover open
 * within 2 m of anything built. Marked `edited`. A land whose town follows its slopes (`seeded`
 * `'follow'`) keeps its heights (the town is graded into it afterwards: `landFollow.ts`
 * `withLand`) and stays `edited` only if it was (shaped by the player).
 */
function settleTown(field: LandField, built: BuiltLand): LandField {
  const { cols, rows } = field;
  const out: LandField = { ...field, h: field.h.slice(), wl: field.wl.slice(), cover: field.cover.slice(), edited: field.seeded === 'follow' ? field.edited : true };
  const guard = new Protection(cols, rows, built);
  const wet = (k: number) => out.wl[k] !== NONE && out.wl[k] > out.h[k];
  for (let k = 0; k < cols * rows && field.seeded !== 'follow'; k++) {
    if (wet(k)) continue;
    const d = guard.dist[k];
    if (d >= APRON + GRADE_RISE) continue;
    const t = Math.min(1, Math.max(0, (d - APRON) / GRADE_RISE));
    out.h[k] = Math.round(field.h[k] * t * t * (3 - 2 * t));
  }
  // (Beside water, no lower than a bank over it, nor than the land was.)
  for (let j = 0; j < rows && field.seeded !== 'follow'; j++) {
    for (let i = 0; i < cols; i++) {
      const k = j * cols + i;
      if (wet(k) || out.h[k] === field.h[k]) continue;
      let top = -Infinity;
      for (const [ni, nj] of [[i - 1, j], [i + 1, j], [i, j - 1], [i, j + 1]] as const) if (ni >= 0 && nj >= 0 && ni < cols && nj < rows && wet(nj * cols + ni)) top = Math.max(top, out.wl[nj * cols + ni] + GRADE_BANK);
      if (top > out.h[k]) out.h[k] = Math.min(field.h[k], top);
    }
  }
  const cw = coverSpan(cols);
  const c = COVER_CELL / LAND_CELL;
  for (let r = 0; r < coverSpan(rows); r++) for (let q = 0; q < cw; q++) if (guard.dist[Math.min(rows - 1, r * c + 1) * cols + Math.min(cols - 1, q * c + 1)] <= 2) out.cover[r * cw + q] = COVER.open;
  return out;
}

/** A stream of its own for a place's land plan (`planLand`): its water side, its site, the sea's and the lake's draws, its ponds. */
const PLAN = 0x706c616e;
/**
 * The main town a land is planned for before any is laid on it (m), by the land's size: about the
 * biggest box a town that fits on it has (the room by the water and the river's calm reach are kept
 * for it; a bigger one is pushed off the water's edge a little), and the middle of a town, whose
 * ground the site is scored by (`SITE_CORE`).
 */
const SITE_BOX: Record<LandSize, { w: number; d: number }> = { small: { w: 300, d: 240 }, medium: { w: 390, d: 320 }, large: { w: 430, d: 350 } };
const SITE_CORE = { w: 200, d: 160 };
/** A site is looked for this share of the land's size either way of the place's own spot, every `SITE_STEP` m; by water, its middle this far from the shore (m). */
const SITE_LOOK = 0.12;
const SITE_STEP = 16;
const SITE_SHORE = 110;
/** What a site's slope costs it (per 1 of mean rise per m) against a metre from the place's own spot, and a metre of its height among mountains. */
const SITE_STEEP = 1200;
const SITE_HIGH = 2;
/** A pond is looked for within this of the site first (m), on land no steeper than `POND_STEEP` (rise per m), clear of the biggest town's box by this (m), and this far from another (m, between their edges). */
const POND_NEAR = 330;
const POND_STEEP = 0.1;
const POND_CLEAR = 50;
const POND_APART = 80;
/** A main town's box is put against the shore (`SHORE_GAP`-like: 4 m short of it) if the shore is no further than this from its site, towards its water (m). */
const SNAP_REACH = 260;

/** A place's land as `planned` makes it: its plan, its 8 m cells (heights as rolled, water), its water's outlines, and its woods, meadows and hills (8 m masks). */
interface Planned {
  plan: LandPlan;
  land: Cells;
  waterPolys: Pt[][];
  woods: Uint8Array;
  meadow: Uint8Array;
  hills: Uint8Array;
}

/**
 * A place's land from `seed`, with no town on it (`LandPlan`), from streams of its own (the plan's,
 * the river's, the massif's), so the town and the villages chosen change none of it:
 * - its size (`LAND_SIZE`, the spec's `options.size`), its hills (the generator's noise);
 * - which side the main town's water is on, and its water: the sea along that edge, a lake past
 *   the town's room from the edge, an island's coast, or a river across, calm along the site's
 *   reach, with room on the town's bank for the biggest town the land fits (`SITE_BOX`);
 * - the mountains (a massif's), risen from the water or either side of the valley;
 * - the main town's site: of the places within `SITE_LOOK` of the place's own spot (the middle,
 *   give or take; the valley floor; by the water, `SITE_SHORE` in from the shore on the water's
 *   side), the one whose middle (`SITE_CORE`) is dry and least steep, nearest the spot;
 * - ponds, out in the countryside not far from the site, on gentle land, clear of the site;
 * - the level the towns are built at: the height `BUILT_LOW` of the site's middle stands lower than;
 * - its woods (the spec's share by noise), meadows and hills, as the generator has always drawn them.
 */
function planned(spec: WorldSpec, seed: number): Planned {
  const size: LandSize = spec.options?.size ?? 'medium';
  const W = LAND_SIZE[size];
  const D = W;
  const land = new Cells(W, D);
  const { gw, gd } = land;
  const nseed = (seed % 100000) + 17;
  const rand = mulberry32((seed ^ PLAN) >>> 0);
  const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
  const valley = !!spec.massif;
  const massif = spec.massif ? range(mulberry32((seed ^ MASSIF) >>> 0), ...spec.massif) : 0;
  const valleyZ = (x: number) => D / 2 + (fbm(x / 420, 3.7, nseed + 3, 3) - 0.47) * 260;
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    land.height[i] = fbm(x / 230, z / 230, nseed, 4) * spec.hills * 2;
  }
  const raise = (m: (x: number, z: number) => number) => {
    if (!valley) return;
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      land.height[i] += massif * pmath.pow(clamp01(m(x, z)), 1.4) * (MASSIF_NEAR + (1 - MASSIF_NEAR) * clamp01(z / (0.6 * D)));
    }
  };
  const side: 'zmin' | 'zmax' = rand() < 0.5 ? 'zmin' : 'zmax';
  const dir = side === 'zmax' ? 1 : -1;
  const box = SITE_BOX[size];
  const spotX = W / 2 + range(rand, -0.08, 0.08) * W;

  // ---- the water, and the mountains risen from it ----
  let lake: LandPlan['lake'];
  let riverPlan: LandPlan['river'];
  let river: River | null = null;
  const coast: Pt[] = [];
  if (spec.water === 'sea') {
    const deep = range(rand, 120, 160);
    const shoreOf = (x: number) => deep + (fbm(x / 160, 9.1, nseed + 5, 3) - 0.47) * 90;
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      if (side === 'zmax' ? z > D - shoreOf(x) : z < shoreOf(x)) land.water[i] = 1;
    }
    raise((x, z) => ((side === 'zmax' ? D - shoreOf(x) - z : z - shoreOf(x)) - 160) / MASSIF_RISE);
  }
  if (spec.water === 'lake') {
    const at: Pt = [W / 2 + range(rand, -50, 50), D / 2 + range(rand, -40, 40)];
    const [rx, rz] = [range(rand, 150, 185) * Math.min(1, W / LAKE_WORLD), range(rand, 95, 120) * Math.min(1, W / LAKE_WORLD)];
    // (Room for the town between it and the world's edge, on the town's side of it.)
    const room = EDGE / 2 + box.d + 4 + rz * 1.4;
    at[1] = side === 'zmin' ? Math.min(at[1], D - room) : Math.max(at[1], room);
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      if (pmath.pow((x - at[0]) / rx, 2) + pmath.pow((z - at[1]) / rz, 2) < 1 + (fbm(x / 90, z / 90, nseed + 6, 3) - 0.47) * 0.7) land.water[i] = 1;
    }
    raise((x, z) => ((pmath.hypot((x - at[0]) / rx, (z - at[1]) / rz) - 1.9) * Math.min(rx, rz)) / 200);
    lake = { x: round2(at[0]), z: round2(at[1]), rx: round2(rx), rz: round2(rz) };
  }
  if (spec.water === 'island') {
    const [ax, az] = [ISLAND * W, ISLAND * D];
    for (let k = 0; k < COAST_POINTS; k++) {
      const a = (k / COAST_POINTS) * Math.PI * 2;
      const [c, s] = [pmath.cos(a), pmath.sin(a)];
      const r0 = 1 / pmath.cbrt(pmath.pow(Math.abs(c / ax), 3) + pmath.pow(Math.abs(s / az), 3));
      const r = r0 * (1 + (fbm((W / 2 + c * r0) / 140, (D / 2 + s * r0) / 140, nseed + 23, 3) - 0.47) * 0.14);
      coast.push([round2(W / 2 + c * r), round2(D / 2 + s * r)]);
    }
    for (let i = 0; i < gw * gd; i++) if (!inside(coast, ...land.centre(i))) land.water[i] = 1;
    raise((x, z) => (0.75 - pmath.cbrt(pmath.pow(Math.abs((x - W / 2) / ax), 3) + pmath.pow(Math.abs((z - D / 2) / az), 3))) / 0.45);
  }
  if (spec.water === 'ponds') raise((x, z) => (Math.abs(z - valleyZ(x)) - MASSIF_FLAT) / MASSIF_RISE);
  if (spec.water === 'river') {
    // (Calm along the site's reach; room for the town between it and the world's edge, on its bank.)
    const room = EDGE / 2 + box.d + 40;
    riverPlan = { ...riverDrawn(mulberry32((seed ^ RIVER) >>> 0), D, side === 'zmax' ? [room, D] : [0, D - room]), reach: [round2(spotX - box.w / 2), round2(spotX + box.w / 2)] };
    river = riverOf(riverPlan, W, nseed);
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      if (Math.abs(z - river.z(x)) < river.half(x) + CELL / 2) land.water[i] = 1;
    }
    const r = river;
    raise((x, z) => (Math.abs(z - r.z(x)) - RIVER_FLAT) / MASSIF_RISE);
  }

  // ---- the main town's site ----
  const core = (c: Pt): Rect => ({ x: c[0] - SITE_CORE.w / 2, z: c[1] - SITE_CORE.d / 2, w: SITE_CORE.w, d: SITE_CORE.d });
  const cellsOf = (r: Rect, each: (i: number) => void) => {
    for (let z = Math.max(0, Math.floor(r.z / CELL)); z <= Math.min(gd - 1, Math.floor((r.z + r.d) / CELL)); z++) {
      for (let x = Math.max(0, Math.floor(r.x / CELL)); x <= Math.min(gw - 1, Math.floor((r.x + r.w) / CELL)); x++) each(z * gw + x);
    }
  };
  const wetIn = (r: Rect) => {
    let wet = false;
    cellsOf({ x: r.x - CELL, z: r.z - CELL, w: r.w + 2 * CELL, d: r.d + 2 * CELL }, (i) => (wet ||= land.water[i] === 1));
    return wet;
  };
  const steepness = (r: Rect) => {
    let [sum, n] = [0, 0];
    cellsOf(r, (i) => {
      sum += land.slope(i);
      n++;
    });
    return n ? sum / n : 0;
  };
  /** The first wet z going from `from` towards the water across x0..x1 (the shore nearest the site). */
  const shoreAt = (x0: number, x1: number, from: number): number => {
    let best = dir > 0 ? Infinity : -Infinity;
    for (let x = Math.max(0, x0); x <= Math.min(W - 1, x1); x += CELL) {
      for (let z = from; z >= 0 && z < D; z += dir * CELL) {
        if (land.water[land.cell(x, z)]) {
          best = dir > 0 ? Math.min(best, z) : Math.max(best, z);
          break;
        }
      }
    }
    return best;
  };
  const byWater = spec.water === 'sea' || spec.water === 'lake' || spec.water === 'river' || spec.water === 'island';
  /** Where the site would go at `x`: by the water, `SITE_SHORE` in from its shore; in a valley, on its floor; else null (anywhere across). */
  const zAt = (x: number): number | null => {
    if (byWater) {
      const middle = spec.water === 'island' || spec.water === 'sea' ? D / 2 : spec.water === 'lake' ? lake!.z : river!.z(x);
      const from = Math.max(0, Math.min(D - 1, middle - (spec.water === 'island' ? 0 : dir * 260)));
      const shore = shoreAt(x - SITE_CORE.w / 2, x + SITE_CORE.w / 2, from);
      return Number.isFinite(shore) ? shore - dir * SITE_SHORE : D / 2;
    }
    return valley ? valleyZ(x) : null;
  };
  const spot: Pt = [spotX, zAt(spotX) ?? D / 2 + range(rand, -0.08, 0.08) * D];
  const [x0, x1] = [Math.max(EDGE + SITE_CORE.w / 2, spotX - SITE_LOOK * W), Math.min(W - EDGE - SITE_CORE.w / 2, spotX + SITE_LOOK * W)];
  const zLimit = (z: number) => Math.max(EDGE / 2 + SITE_CORE.d / 2, Math.min(D - EDGE / 2 - SITE_CORE.d / 2, z));
  let best: { p: Pt; score: number } | null = null;
  for (let x = x0; x <= x1; x += SITE_STEP) {
    const along = zAt(x);
    const zs = along === null ? Array.from({ length: Math.floor((2 * SITE_LOOK * D) / SITE_STEP) + 1 }, (_, k) => spot[1] - SITE_LOOK * D + k * SITE_STEP) : [along];
    for (const z of zs) {
      const p: Pt = [x, zLimit(z)];
      const r = core(p);
      if (wetIn(r)) continue;
      const score = steepness(r) * SITE_STEEP + dist(p, spot) + (valley ? land.at(p[0], p[1]) * SITE_HIGH : 0);
      if (!best || score < best.score) best = { p, score };
    }
  }
  const site: Pt = best ? [Math.round(best.p[0]), Math.round(best.p[1])] : [Math.round(spot[0]), Math.round(zLimit(spot[1]))];

  // ---- ponds, out in the countryside, clear of the site ----
  const ponds: NonNullable<LandPlan['ponds']> = [];
  if (spec.water === 'ponds') {
    // (A fixed number draws nothing.)
    const count = spec.ponds[0] === spec.ponds[1] ? spec.ponds[0] : between(rand, spec.ponds);
    for (let n = 0, tries = 0; n < count && tries < 200; tries++) {
      const r = range(rand, 14, 24);
      const c: Pt = [range(rand, 60, W - 60), range(rand, 60, D - 60)];
      if (Math.abs(c[0] - site[0]) < box.w / 2 + POND_CLEAR + r && Math.abs(c[1] - site[1]) < box.d / 2 + POND_CLEAR + r) continue;
      if (tries < 100 && dist(c, site) > POND_NEAR + Math.max(box.w, box.d) / 2) continue;
      if (tries < 150 && land.slope(land.cell(c[0], c[1])) > POND_STEEP) continue;
      if (ponds.some((q) => dist([q.x, q.z], c) < q.r + r + POND_APART)) continue;
      for (let i = 0; i < gw * gd; i++) {
        const [x, z] = land.centre(i);
        if (dist([x, z], c) < r * (0.85 + fbm(x / 20, z / 20, nseed + 8 + n, 2) * 0.5)) land.water[i] = 1;
      }
      ponds.push({ x: round2(c[0]), z: round2(c[1]), r: round2(r) });
      n++;
    }
  }

  // ---- the level the towns are built at ----
  const levels: number[] = [];
  cellsOf(core(site), (i) => !land.water[i] && levels.push(land.height[i]));
  levels.sort((a, b) => a - b);
  const level = levels[Math.floor(levels.length * BUILT_LOW)] ?? 0;

  // ---- woods, meadows and hills ----
  const waterPolys = river ? [river.poly] : coast.length ? [islandSea(coast, W, D)] : outlines(land.water, gw, gd, 2);
  const woodsNoise = new Float32Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    woodsNoise[i] = fbm(x / 105, z / 105, nseed + 7, 4) + (valley ? Math.min(0.25, land.height[i] / 400) : 0);
  }
  const cut = [...woodsNoise].sort((a, b) => b - a)[Math.floor(spec.woods * gw * gd)] ?? 1;
  const heights = [...land.height].sort((x, y) => x - y);
  const hillCut = heights[Math.floor(heights.length * (valley ? 0.62 : 0.82))];
  const woods = new Uint8Array(gw * gd);
  const meadow = new Uint8Array(gw * gd);
  const hills = new Uint8Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) {
    if (land.water[i]) continue;
    const [x, z] = land.centre(i);
    woods[i] = woodsNoise[i] > cut ? 1 : 0;
    if (!woods[i] && fbm(x / 70, z / 70, nseed + 11, 3) > (spec.meadowCut ?? 0.58)) meadow[i] = 1;
    if (land.height[i] > hillCut) hills[i] = 1;
  }

  const plan: LandPlan = {
    v: 1,
    kind: spec.kind as WorldKind,
    seed,
    size,
    side,
    site,
    level: round2(level),
    massif: round2(massif),
    ...(riverPlan ? { river: riverPlan } : {}),
    ...(lake ? { lake } : {}),
    ...(coast.length ? { coast } : {}),
    ...(ponds.length ? { ponds } : {}),
  };
  return { plan, land, waterPolys, woods, meadow, hills };
}

/**
 * A place's land as a draft with no town on it (`landDraft`): `planned`, then seeded bare
 * (`landFollow.ts` `seedBare`) from its heights over its level, its water's outlines and its woods
 * and meadows; named and told as the town laid on it from the same seed is (its first draws).
 */
function bareDraft(content: Content, spec: WorldSpec, seed: number): NeighbourhoodDraft {
  const { plan, land, waterPolys, woods, meadow, hills } = planned(spec, seed);
  const { W, D, gw, gd } = land;
  const rand = mulberry32(seed);
  const name = worldName(content, spec.names, rand);
  const tagline = worldTagline(content, spec, rand);
  const { water: _water, ...landscape } = spec.landscape(mulberry32((seed ^ LOOK) >>> 0), W, D, plan.side);
  if (plan.massif) landscape.mountains = Math.round(plan.massif);
  const zones: WorldZone[] = [
    ...waterPolys.map((poly): WorldZone => ({ kind: 'water', poly })),
    ...outlines(hills, gw, gd, 12).map((poly): WorldZone => ({ kind: 'hill', poly })),
    ...outlines(meadow, gw, gd, 6).map((poly): WorldZone => ({ kind: 'meadow', poly })),
    ...outlines(woods, gw, gd, 8).map((poly): WorldZone => ({ kind: 'woods', poly })),
  ];
  const open: OpenWorldDraft = {
    areas: [],
    zones,
    water: rle(rasterize(waterPolys, W, D)),
    pads: {},
    parking: [],
    relief: reliefOf(land, [], plan.level),
    heights: { cell: CELL, cols: gw, rows: gd, h: Float32Array.from(land.height, (h) => h - plan.level) },
    plan,
  };
  const draft: NeighbourhoodDraft = { name, width: W, depth: D, slots: [], roads: [], greens: [], households: [], seed, world: { kind: spec.kind, size: 'big', tagline, landscape, ...(spec.options ? { options: spec.options } : {}) }, open };
  const { heights: _heights, ...rest } = open;
  return { ...draft, open: { ...rest, land: seedBare(draft) } };
}

/** A main town's roads as laid out in `box` (`shape`: joined up and settled), turned over front to back (its water on the other side). */
function mirrored(shape: RoadLayout, box: Box): RoadLayout {
  const flip = ([x, z]: Pt): Pt => [x, box.d - z];
  return {
    ...shape,
    roads: shape.roads.map((r) => ({ ...r, points: r.points.map(flip), ...(r.lots ? { lots: (-r.lots) as 1 | -1 } : {}) })),
    ...(shape.greens ? { greens: shape.greens.map((g) => g.map(flip)) } : {}),
    ...(shape.centre ? { centre: flip(shape.centre) } : {}),
    ...(shape.water ? { water: shape.water === 'zmin' ? 'zmax' : 'zmin' } : {}),
  };
}

/**
 * The side of a town at `at` its water is on (`LandPlan.side` for the plan's own site): the
 * river's (`river`, the plan's kept) or the lake's way from it, the island's nearer coast, else the plan's.
 */
function sideOf(plan: LandPlan, at: Pt, river: River | null, D: number): 'zmin' | 'zmax' {
  if (river) return at[1] < river.z(at[0]) ? 'zmax' : 'zmin';
  if (plan.lake) return at[1] < plan.lake.z ? 'zmax' : 'zmin';
  if (plan.coast) return at[1] < D / 2 ? 'zmin' : 'zmax';
  return plan.side;
}

/**
 * A town's cover on the land it was laid on (`laid`, its copy: `settleTown`'s): the woods cleared
 * where the town clears them (`wild` 0, per 8 m cell of `land`: its boxes, its lots, buildings and
 * roads and a little round them), no meadow in the areas' boxes (`areas`), and its farmland
 * (`fields`, 8 m cells) where the ground is open.
 */
function townCover(laid: LandField, land: Cells, { wild, fields, areas }: { wild: Uint8Array; fields: Uint8Array; areas: Uint8Array }): void {
  const cw = coverSpan(laid.cols);
  const cd = coverSpan(laid.rows);
  for (let r = 0; r < cd; r++) {
    for (let q = 0; q < cw; q++) {
      const i = land.cell((q + 0.5) * COVER_CELL, (r + 0.5) * COVER_CELL);
      const c = r * cw + q;
      if (laid.cover[c] === COVER.woods && !wild[i]) laid.cover[c] = COVER.open;
      if (laid.cover[c] === COVER.meadow && areas[i]) laid.cover[c] = COVER.open;
      if (laid.cover[c] === COVER.open && fields[i]) laid.cover[c] = COVER.fields;
    }
  }
}

/** A river across a world (`riverAcross`). */
interface River {
  /** Its middle line, from west to east, and its half-width there (m, along z: wider where it runs slantwise). */
  z: (x: number) => number;
  half: (x: number) => number;
  /** Its outline, a water zone (on past the world's edges). */
  poly: Pt[];
}

/**
 * A river across a `W` × `D` world from west to east: meandering 70 to 110 m either way, but a
 * fifth as much along `reach` (the main town's stretch of it, from x to x) and a little past it,
 * 14 to 22 m wide, its middle near the world's (within `middle`, from z to z, where the town has room).
 */
function riverAcross(rand: Rand, W: number, D: number, reach: [number, number], nseed: number, middle: [number, number]): River {
  return riverOf({ ...riverDrawn(rand, D, middle), reach }, W, nseed);
}

/** A river's course as drawn from `rand` (`LandPlan.river`, but its calm reach): its middle near the world's (within `middle`), its tilt, its meanders' swing and their noise's offset. */
function riverDrawn(rand: Rand, D: number, middle: [number, number]): Omit<NonNullable<LandPlan['river']>, 'reach'> {
  const c = Math.max(middle[0], Math.min(middle[1], D / 2 + range(rand, -0.04, 0.04) * D));
  const tilt = range(rand, -0.05, 0.05);
  const amp = range(rand, ...MEANDER);
  const k = rand() * 50;
  return { c, tilt, amp, k };
}

/** The river a course (`LandPlan.river`) makes across a world `W` m wide (`nseed`: the world's noise). */
function riverOf({ c, tilt, amp, k, reach }: NonNullable<LandPlan['river']>, W: number, nseed: number): River {
  const calm = (x: number) => {
    const t = Math.min(1, Math.max(reach[0] - 60 - x, x - reach[1] - 60, 0) / 120);
    return 0.2 + 0.8 * t * t * (3 - 2 * t);
  };
  const z = (x: number) => c + tilt * (x - W / 2) + (fbm(x / 260, k, nseed + 21, 3) - 0.5) * 2.5 * amp * calm(x);
  const width = (x: number) => RIVER_WIDTH[0] + (RIVER_WIDTH[1] - RIVER_WIDTH[0]) * Math.min(1, Math.max(0, fbm(x / 120, k + 3, nseed + 22, 2) * 1.6 - 0.3));
  const half = (x: number) => (width(x) / 2) * pmath.hypot(1, (z(x + 1) - z(x - 1)) / 2);
  const xs: number[] = [];
  for (let x = -60; x <= W + 60; x += 4) xs.push(x);
  const poly = [...xs.map((x): Pt => [x, round2(z(x) - half(x))]), ...[...xs].reverse().map((x): Pt => [x, round2(z(x) + half(x))])];
  return { z, half, poly };
}

/**
 * Across `river` from `p`, square to its course: how far along that way the water begins and
 * ends (m), the point `BRIDGE_IN` m past its far bank, and the way on from there (away from the
 * water). Null if the river is more than `max` m off.
 */
function across(river: River, p: Pt, max: number): { wet: [number, number]; far: Pt; dir: Pt } | null {
  const toward = river.z(p[0]) > p[1] ? 1 : -1;
  const t: Pt = [1, (river.z(p[0] + 4) - river.z(p[0] - 4)) / 8];
  const l = pmath.hypot(...t);
  const n: Pt = [(-t[1] / l) * toward, (t[0] / l) * toward];
  let [first, last] = [-1, -1];
  for (let d = 0; d <= max + 80; d += 0.5) {
    const q: Pt = [p[0] + n[0] * d, p[1] + n[1] * d];
    if (Math.abs(q[1] - river.z(q[0])) < river.half(q[0])) {
      if (first < 0) first = d;
      last = d;
    } else if (first >= 0) break;
    if (first < 0 && d > max) return null;
  }
  if (first < 0) return null;
  const far: Pt = [round2(p[0] + n[0] * (last + BRIDGE_IN)), round2(p[1] + n[1] * (last + BRIDGE_IN))];
  return { wet: [first, last], far, dir: n };
}

/**
 * An island's sea: the world's square with the island's `coast` a hole in it, joined to it by a
 * seam (filled even-odd, as `inside` does), touching every edge of the world.
 */
function islandSea(coast: readonly Pt[], W: number, D: number): Pt[] {
  const west = coast.reduce((k, p, i) => (p[0] < coast[k][0] ? i : k), 0);
  const seam: Pt = [0, coast[west][1]];
  return [seam, [0, 0], [W, 0], [W, D], [0, D], seam, ...coast.slice(west), ...coast.slice(0, west + 1)];
}

/**
 * A bridge's or a causeway's deck (`RoadPlan.bridge`): its arc lengths from the first point over
 * `water` (1 m tiles) less `DECK_PAST` to the last plus as much. The tiles of its deck
 * (`deckTiles`) are cleared in `walk` (the water the sim's walkers keep off), so a walker on its
 * verge walks dry.
 */
export function deckOf(road: RoadPlan, water: Uint8Array, walk: Uint8Array, W: number, D: number): [number, number] {
  const ls = lengths(road.points);
  const L = ls[ls.length - 1];
  const wetAt = ([x, z]: Pt) => water[Math.min(D - 1, Math.max(0, Math.floor(z))) * W + Math.min(W - 1, Math.max(0, Math.floor(x)))] === 1;
  const first = road.points.findIndex(wetAt);
  if (first < 0) return [0, 0];
  let last = first;
  for (let i = first; i < road.points.length; i++) if (wetAt(road.points[i])) last = i;
  const span: [number, number] = [round2(Math.max(0, ls[first] - DECK_PAST)), round2(Math.min(L, ls[last] + DECK_PAST))];
  deckTiles(road, span, W, D, (tile) => (walk[tile] = 0));
  return span;
}

/**
 * The 1 m tiles (indices into a `W` × `D` world) of a bridge's or a causeway's deck over `span`
 * (arc lengths along `road`, as `RoadPlan.bridge`): those within half its width and `DECK_PAST`
 * of its middle line there, each passed to `each`. The water under them is walked dry.
 */
export function deckTiles(road: RoadPlan, span: readonly [number, number], W: number, D: number, each: (tile: number) => void): void {
  const ls = lengths(road.points);
  const [a, b] = [pointAt(road.points, span[0], ls).p, pointAt(road.points, span[1], ls).p];
  const R = road.width / 2 + DECK_PAST;
  const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
  const len2 = dx * dx + dz * dz || 1;
  for (let z = Math.max(0, Math.floor(Math.min(a[1], b[1]) - R)); z < Math.min(D, Math.ceil(Math.max(a[1], b[1]) + R)); z++) {
    for (let x = Math.max(0, Math.floor(Math.min(a[0], b[0]) - R)); x < Math.min(W, Math.ceil(Math.max(a[0], b[0]) + R)); x++) {
      const t = Math.max(0, Math.min(1, ((x + 0.5 - a[0]) * dx + (z + 0.5 - a[1]) * dz) / len2));
      if (pmath.hypot(x + 0.5 - a[0] - dx * t, z + 0.5 - a[1] - dz * t) <= R) each(z * W + x);
    }
  }
}

/** The relief's cells (m). */
const RELIEF = 16;

/**
 * How high the land stands over the level the towns are built at (`WorldMeta.relief`): the land's
 * height averaged over 16 m cells, less the middle height (the median) of the cells anything
 * stands on (`built`: the lots; `at` instead, if given: the level the land is
 * planned at), in decimetres, never below 0, and 0 where the cell is mostly water. The renderer
 * draws its hills from it.
 */
function reliefOf(land: Cells, built: readonly Rect[], at?: number): NonNullable<WorldMeta['relief']> {
  const k = RELIEF / CELL;
  const [cols, rows] = [Math.ceil(land.W / RELIEF), Math.ceil(land.D / RELIEF)];
  const mean = new Float32Array(cols * rows);
  const wet = new Uint8Array(cols * rows);
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      let [sum, n, water] = [0, 0, 0];
      for (let dz = 0; dz < k; dz++) {
        for (let dx = 0; dx < k; dx++) {
          const [x, z] = [c * k + dx, r * k + dz];
          if (x >= land.gw || z >= land.gd) continue;
          sum += land.height[z * land.gw + x];
          water += land.water[z * land.gw + x];
          n++;
        }
      }
      mean[r * cols + c] = n ? sum / n : 0;
      wet[r * cols + c] = water * 2 >= n && n ? 1 : 0;
    }
  }
  const on = new Uint8Array(cols * rows);
  for (const b of built) {
    for (let r = Math.max(0, Math.floor(b.z / RELIEF)); r <= Math.min(rows - 1, Math.floor((b.z + b.d) / RELIEF)); r++) {
      for (let c = Math.max(0, Math.floor(b.x / RELIEF)); c <= Math.min(cols - 1, Math.floor((b.x + b.w) / RELIEF)); c++) on[r * cols + c] = 1;
    }
  }
  const levels = [...mean.filter((_, i) => on[i] && !wet[i])].sort((a, b) => a - b);
  const level = at ?? levels[Math.floor(levels.length / 2)] ?? 0;
  return { cell: RELIEF, cols, rows, h: Array.from(mean, (h, i) => (wet[i] ? 0 : Math.max(0, Math.round((h - level) * 10)))) };
}

/** The share of the built cells lower than the level the towns are built at (`heightsOf`). */
const BUILT_LOW = 0.05;

/**
 * The land's signed heights (`OpenWorldDraft.heights`): each 8 m cell's height less the level the
 * towns are built at, the height `BUILT_LOW` of the cells anything is built on stand lower than
 * (the lots and the roads but the trails; not water). Mostly the built land is a
 * little over 0, and the sea (`land.ts` `SEA_CM`) a little under it, as the renderer has always
 * drawn them.
 */
function heightsOf(land: Cells, built: readonly Rect[], roads: readonly RoadPlan[]): NonNullable<OpenWorldDraft['heights']> {
  const { gw, gd } = land;
  const on = new Uint8Array(gw * gd);
  for (const b of built) {
    for (let z = Math.max(0, Math.floor(b.z / CELL)); z <= Math.min(gd - 1, Math.floor((b.z + b.d) / CELL)); z++) {
      for (let x = Math.max(0, Math.floor(b.x / CELL)); x <= Math.min(gw - 1, Math.floor((b.x + b.w) / CELL)); x++) on[z * gw + x] = 1;
    }
  }
  for (const r of roads) if (r.kind !== 'trail') for (const [x, z] of r.points) if (x >= 0 && z >= 0 && x < land.W && z < land.D) on[land.cell(x, z)] = 1;
  const levels: number[] = [];
  for (let i = 0; i < gw * gd; i++) if (on[i] && !land.water[i]) levels.push(land.height[i]);
  levels.sort((a, b) => a - b);
  const level = levels[Math.floor(levels.length * BUILT_LOW)] ?? 0;
  return { cell: CELL, cols: gw, rows: gd, h: Float32Array.from(land.height, (h) => h - level) };
}

/**
 * The main town's terraces (`WorldSpec.terraces` lots, in rows of `WorldSpec.rows`: two, a
 * semi-detached pair), each row along the first stretch of `frontages` it fits on (the town's
 * nearest its middle first, then the next list's: `LotPlacer.placeRow`); a row that fits nowhere is
 * tried shorter, down to a pair. Their houses are drawn from `rand` (the world's own draws are left as
 * they were). The rows placed.
 */
export function terraceRows(t: Templates, lots: LotPlacer, frontages: readonly (readonly Frontage[])[], spec: Pick<WorldSpec, 'terraces' | 'rows'>, rand: Rand): PlotSlot[][] {
  const out: PlotSlot[][] = [];
  let left = between(rand, spec.terraces);
  // (Its houses of a few designs, as a terrace built at once is; each design generated once.)
  const design = pooled(designs(rand), rand);
  // (The longest row that went down last: one longer fits nowhere.)
  let longest = spec.rows[1];
  // (Nowhere for a pair: no more rows.)
  for (let tries = 0; left >= 2 && tries < 400; tries++) {
    let n = Math.min(left, longest, between(rand, spec.rows));
    if (left - n === 1) n = left === 3 && spec.rows[1] >= 3 ? 3 : n - 1;
    const units = terraceUnits(t, Math.max(2, n), 0, design);
    // (No terraced house to be had: none generated, the content has no row class.)
    if (units.length < 2) break;
    let placed: PlotSlot[] = [];
    for (let k = units.length; k >= 2 && !placed.length; k--) {
      for (const frontage of frontages) if (!placed.length) placed = lots.placeRow(units.slice(0, k), frontage, 0);
    }
    if (!placed.length) break;
    if (placed.length < units.length) longest = placed.length;
    out.push(placed);
    left -= placed.length;
  }
  return out;
}

/** A country road runs nearly straight round arc length `s` (within `BEND` m either way): a car turns in off it without swinging off the road. */
const straight = (pts: readonly Pt[], s: number, ls: readonly number[]) => straightAround(pts, s, BEND, BEND_MOST, ls);

/**
 * Homes along the country roads, on purpose: 2 to 4 cottages where each road leaves an area and
 * where it meets another out in the country, then a farmstead (a farm lot: its fields, its barn
 * and yard) every `every` m of road (150 to 200 in the classic worlds), each clear of the areas,
 * the water and steep land, and of the area it is nearest. The farmsteads placed.
 */
export function alongCountryRoads(content: Content, t: Templates, lots: LotPlacer, plans: readonly AreaPlan[], roads: readonly RoadPlan[], land: Cells, every: [number, number], rand: Rand, steepest = STEEP): PlotSlot[] {
  const country = roads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : []));
  const byRoad = new Map<number, Frontage[]>();
  const lens = new Map(country.map((r) => [r, lengths(roads[r].points)]));
  for (const f of frontageOf(roads, country, plans[0].centre, rand)) {
    if (!straight(roads[f.road].points, f.s, lens.get(f.road)!)) continue;
    const list = byRoad.get(f.road);
    if (list) list.push(f);
    else byRoad.set(f.road, [f]);
  }
  const outside = (p: Pt, by: number) => clearOfBoxes(plans, p, by);
  /** Well inside the world (a road out leaves it there). */
  const within = (p: Pt) => Math.min(p[0], p[1], land.W - p[0], land.D - p[1]) > 30;
  const homeArea = (p: Pt) => plans.reduce((k, a, i) => (dist(a.centre, p) < dist(plans[k].centre, p) ? i : k), 0);
  /** Dry and not steep within `r` m of `p`. */
  const buildable = (p: Pt, r: number) => {
    if (land.wet(p[0], p[1], r)) return false;
    for (let dz = -r; dz <= r; dz += CELL * 2) for (let dx = -r; dx <= r; dx += CELL * 2) if (land.slope(land.cell(p[0] + dx, p[1] + dz)) > steepest) return false;
    return true;
  };
  const house = (lot: 'small' | 'farm', name: string, area: number) => ({ kind: 'house' as const, name, lot, area, ...houseFor(t, lot, lot === 'farm' ? 'roomy' : rand() < 0.5 ? 'snug' : 'roomy', rand) });

  // Cottages: where each country road leaves an area (a little way out along it), and where one meets another.
  const spots: { p: Pt; roads: number[] }[] = [];
  for (const r of country) {
    const pts = roads[r].points;
    const L = lengths(pts).at(-1)!;
    for (const e of [0, 1] as const) {
      if (roads[r].ends[e] === 'out') continue;
      const end = e === 0 ? pts[0] : pts[pts.length - 1];
      if (!outside(end, 20)) {
        const s = range(rand, 45, 80);
        if (L > s + 30) spots.push({ p: pointAt(pts, e === 0 ? s : L - s).p, roads: [r] });
      } else {
        const other = country.find((o) => o !== r && nearest(roads[o].points, end[0], end[1]).dist < roads[o].width / 2 + 2);
        spots.push({ p: end, roads: other === undefined ? [r] : [r, other] });
      }
    }
  }
  for (const spot of spots) {
    if (!buildable(spot.p, 24)) continue;
    const fronts = spot.roads
      .flatMap((r) => byRoad.get(r) ?? [])
      .filter((f) => dist(f.p, spot.p) > 10 && dist(f.p, spot.p) < COTTAGE_REACH && outside(f.p, 14) && within(f.p))
      .sort((a, b) => dist(a.p, spot.p) - dist(b.p, spot.p));
    if (!fronts.length) continue;
    const area = homeArea(spot.p);
    for (let k = between(rand, COTTAGES); k > 0; k--) if (lots.place([house('small', '', area)], fronts, [], spot.p, area, true).length) break;
  }

  // Farmsteads along the way, each named for a word of the farmland's (none twice).
  const words = content.names.worlds?.farm?.start ?? ['Meadow', 'Barley', 'Clover'];
  const named = new Set(lots.slots.filter((s) => s.lot === 'farm').map((s) => s.name));
  const farmName = () => {
    for (let k = 0; k < 24; k++) {
      const n = `${words[Math.floor(rand() * words.length)]} Farm`;
      if (!named.has(n)) return n;
    }
    return `${words[named.size % words.length]} Farm ${named.size + 1}`;
  };
  const farms: PlotSlot[] = [];
  for (const r of country) {
    const pts = roads[r].points;
    const L = lengths(pts).at(-1)!;
    const fronts = byRoad.get(r) ?? [];
    for (let s = range(rand, 50, 110); s < L - 30; ) {
      const p = pointAt(pts, s).p;
      if (!outside(p, FARM_OFF) || !within(p) || !buildable(p, 36) || farms.some((f) => dist([f.x, f.z], p) < 110)) {
        s += 16;
        continue;
      }
      // (Farms go on the frontage furthest down the list first: the nearest last here.)
      const near = fronts.filter((f) => Math.abs(f.s - s) < 24).sort((a, b) => Math.abs(b.s - s) - Math.abs(a.s - s));
      const area = homeArea(p);
      const name = farmName();
      if (lots.place([house('farm', name, area)], near, [], p, area, true).length) {
        s += 16;
        continue;
      }
      named.add(name);
      farms.push(lots.slots[lots.slots.length - 1]);
      s += range(rand, ...every);
    }
  }
  return farms;
}

/**
 * Where an outdoor place of area `plan` goes: on its kind of land (by the water, its back to it;
 * up a hill; in the woods; in a meadow), 50 to 140 m from the area's middle, on land nothing else
 * takes, no further than `reach` from the area's box. Its rectangle and which way it faces
 * (towards the area: its trail comes from there).
 */
export function siteFor(place: PublicTemplate, plan: AreaPlan, land: Cells, ground: Ground, water: Uint8Array, t: Templates, rand: Rand, W: number, D: number, reach = Infinity): { x: number; z: number; facing: Facing; w: number; d: number } | null {
  const frame = place.lot ?? t.plot;
  let best: { at: { x: number; z: number; facing: Facing; w: number; d: number }; score: number } | null = null;
  const wetAt = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && water[Math.floor(z) * W + Math.floor(x)] === 1;
  for (let k = 0; k < 260; k++) {
    const angle = rand() * Math.PI * 2;
    const r = range(rand, 50, 140) + Math.max(plan.rect.w, plan.rect.d) * 0.3;
    const c: Pt = [plan.centre[0] + pmath.cos(angle) * r, plan.centre[1] + pmath.sin(angle) * r];
    if (offRect(plan.rect, c) > reach) continue;
    // From the area towards the place (its trail comes from there), or towards the water behind it.
    let n: Pt = [(c[0] - plan.centre[0]) / r, (c[1] - plan.centre[1]) / r];
    let score = 0;
    if (place.site === 'water') {
      let [sx, sz, hits] = [0, 0, 0];
      for (let a = 0; a < 16; a++) {
        const [dx, dz] = [pmath.cos((a / 16) * Math.PI * 2), pmath.sin((a / 16) * Math.PI * 2)];
        if (wetAt(c[0] + dx * 16, c[1] + dz * 16)) [sx, sz, hits] = [sx + dx, sz + dz, hits + 1];
      }
      if (!hits || hits > 9) continue;
      const l = pmath.hypot(sx, sz) || 1;
      n = [sx / l, sz / l];
      score = -hits;
    } else if (place.site === 'hill') score = -land.at(c[0], c[1]) + land.slope(land.cell(c[0], c[1])) * 60;
    else if (place.site === 'woods') {
      let trees = 0;
      for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) trees += land.woods[land.cell(c[0] + dx * 16, c[1] + dz * 16)];
      if (trees < 5) continue;
      score = -trees + land.slope(land.cell(c[0], c[1])) * 60;
    } else score = land.woods[land.cell(c[0], c[1])] * 4 + land.slope(land.cell(c[0], c[1])) * 60;
    const facing = (Math.abs(n[1]) >= Math.abs(n[0]) ? (n[1] > 0 ? 0 : 2) : n[0] > 0 ? 1 : 3) as Facing;
    const [w, d] = facing % 2 ? [frame.depth, frame.width] : [frame.width, frame.depth];
    const rect = { x: Math.round(c[0] - w / 2), z: Math.round(c[1] - d / 2), w, d };
    if (!ground.free({ x: rect.x - 3, z: rect.z - 3, w: rect.w + 6, d: rect.d + 6 }, 8)) continue;
    // Keeps a little way back from the water.
    if (land.wet(c[0], c[1], 0) || [0, 0.5, 1].some((u) => [0, 0.5, 1].some((v) => wetAt(rect.x - 2 + (rect.w + 4) * u, rect.z - 2 + (rect.d + 4) * v)))) continue;
    if (!best || score < best.score) best = { at: { x: rect.x, z: rect.z, facing, w: frame.width, d: frame.depth }, score };
  }
  return best?.at ?? null;
}

/** Trails' grid (m). */
const TRAIL_CELL = 2;

/**
 * A trail to outdoor place `slot` from the nearest of its area's roads, round the lots and the
 * water: A* on a 2 m grid from every road cell of the area to the place's front, smoothed where
 * that keeps it off the lots. Null if there's no way.
 */
export function trailTo(slot: PlotSlot, plan: AreaPlan, roads: readonly RoadPlan[], slots: readonly PlotSlot[], water: Uint8Array, land: Cells, t: Templates, W: number, D: number): RoadPlan | null {
  const rect = plotRect(t, slot);
  const { width, depth } = slotDims(t, slot);
  const turn = turner(slot.x, slot.z, width, depth, slot.facing);
  const front = turn.point([width / 2, 0]);
  const before = turn.point([width / 2, -3]);
  // A window round the area and the place.
  const x0 = Math.max(0, Math.floor(Math.min(plan.rect.x, rect.x) - 40));
  const z0 = Math.max(0, Math.floor(Math.min(plan.rect.z, rect.z) - 40));
  const x1 = Math.min(W, Math.ceil(Math.max(plan.rect.x + plan.rect.w, rect.x + rect.w) + 40));
  const z1 = Math.min(D, Math.ceil(Math.max(plan.rect.z + plan.rect.d, rect.z + rect.d) + 40));
  const gw = Math.ceil((x1 - x0) / TRAIL_CELL);
  const gd = Math.ceil((z1 - z0) / TRAIL_CELL);
  const blocked = new Uint8Array(gw * gd);
  const mark = (r: Rect, grow: number) => {
    for (let z = Math.max(0, Math.floor((r.z - grow - z0) / TRAIL_CELL)); z <= Math.min(gd - 1, Math.floor((r.z + r.d + grow - z0) / TRAIL_CELL)); z++) {
      for (let x = Math.max(0, Math.floor((r.x - grow - x0) / TRAIL_CELL)); x <= Math.min(gw - 1, Math.floor((r.x + r.w + grow - x0) / TRAIL_CELL)); x++) blocked[z * gw + x] = 1;
    }
  };
  for (const s of slots) mark(plotRect(t, s), 0.4);
  for (let z = z0; z < z1; z++) for (let x = x0; x < x1; x++) if (water[z * W + x]) blocked[Math.floor((z - z0) / TRAIL_CELL) * gw + Math.floor((x - x0) / TRAIL_CELL)] = 1;
  const cellOf = ([x, z]: Pt) => Math.max(0, Math.min(gd - 1, Math.floor((z - z0) / TRAIL_CELL))) * gw + Math.max(0, Math.min(gw - 1, Math.floor((x - x0) / TRAIL_CELL)));
  const centre = (i: number): Pt => [x0 + ((i % gw) + 0.5) * TRAIL_CELL, z0 + (Math.floor(i / gw) + 0.5) * TRAIL_CELL];
  const onRoad = new Uint8Array(gw * gd);
  const starts: number[] = [];
  const startPoint = new Map<number, Pt>();
  for (const r of plan.roads) {
    for (const p of roads[r].points) {
      const i = cellOf(p);
      onRoad[i] = 1;
      if (!blocked[i] && !startPoint.has(i)) {
        starts.push(i);
        startPoint.set(i, p);
      }
    }
  }
  const goal = cellOf(before);
  blocked[goal] = 0;
  const cells = astar(gw, gd, (i) => (blocked[i] ? Infinity : onRoad[i] ? 4 : 1 + land.woods[land.cell(...centre(i))] * 0.4), starts, (i) => i === goal, (i) => dist(centre(i), before) / TRAIL_CELL);
  if (!cells) return null;
  const raw: Pt[] = [startPoint.get(cells[0])!, ...cells.slice(1).map(centre), before, front];
  const clear = (pts: readonly Pt[]) =>
    pts.every(([x, z]) => !slots.some((s) => {
      const r = plotRect(t, s);
      return s !== slot && x > r.x + 0.05 && x < r.x + r.w - 0.05 && z > r.z + 0.05 && z < r.z + r.d - 0.05;
    }) && !(x >= 0 && z >= 0 && x < W && z < D && water[Math.floor(z) * W + Math.floor(x)]));
  const smooth = curve(simplify(raw, 0.9));
  const points = clear(smooth) ? smooth : evenly(raw, 1);
  return { points: points.map(([x, z]): Pt => [round2(x), round2(z)]), width: TRAIL_WIDTH, ends: ['join', 'dead'], kind: 'trail' };
}

/** The middles of a big world's areas, and a centroid helper for the menus. */
export function areaCentre(a: TownArea): Pt {
  return centroid([[a.x, a.z], [a.x + a.w, a.z + a.d]]);
}
