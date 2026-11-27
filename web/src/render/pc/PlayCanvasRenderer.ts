/**
 * The game's renderer on PlayCanvas (`Renderer`, `render/types.ts`).
 *
 * The device (WebGPU, else WebGL2), frames paced in whole display refreshes, the game camera (the
 * player's orbit input, kept above the land, the director's shots), the menus' town overview
 * (`overview.ts`, on a layer of its own, with its camera, its still frames held: `heldFrames.ts`),
 * the time of day's sun and its shadows (`sunShadows.ts`), sky fill, environment lighting and fog,
 * the lamps (`lamps.ts`), the picture chain (`post.ts`), picking (`picking.ts`: the ground, objects
 * and items, analytic), projecting points onto the screen, and the stats and the benchmark's
 * measures (`gpuTiming.ts`). The viewed lot: its house on every storey (`house.ts`: walls, floors,
 * roofs, doors and windows, the cutaway and the wall modes, the storey in view, a basement with the
 * ground dug away over it, rooms without windows kept out of daylight, doors and gates swinging for
 * residents going through), its fences (`fences.ts`), roads and paths, the street round it
 * (`street.ts`: the neighbours' houses, sidewalks, street furniture, gardens), its things and the
 * landscape (`lotContents.ts`), its mirrors (`mirrors.ts`) and the garden birds (`birds.ts`). Its
 * residents (`characters.ts`): posed each frame, with their contact shadows, the chairs they pull up
 * to tables, their sun shadows, the selection ring, speech, and the camera following one. Build mode
 * (`placing.ts`, `buildPreviews.ts`, `handles.ts`) and the save's picture (`thumbnail.ts`). The
 * warm-up behind the loading veil (`warmGame`) makes every pipeline the game draws with before play.
 *
 * Game space is left-handed, y up (the space the game's geometry, models and numbers are made in):
 * everything in the world hangs under a root mirrored in z (`space.ts`), and only cameras, lights and
 * rays are converted.
 */

import * as pc from './engine';
import { BuildPreviews, mergeSurfaces } from './buildPreviews';
import { GameCamera } from './camera';
import { Characters, MAX_CHARACTERS } from './characters';
import { createApp, createDevice, destroyApp } from './device';
import { buildFences, regroundGates, swingGate, type Fences, type GateLeaf } from './fences';
import { fenceHedges, wallCorners } from '../shared/fences';
import { shownFences } from '../shared/shownFences';
import { backEdges, curtainsLetThrough, gardenPool, GardenPools, windowPools, type GardenPool } from './gardenPools';
import { HoverTile, ResizeHandles } from './handles';
import { HeldFrames } from './heldFrames';
import { HouseBuilder, type HouseDoor } from './house';
import { Instances } from './instancing';
import { deviceDriverInfo } from './internals';
import { overLayer } from './layers';
import { Lamps } from './lamps';
import { contactBlob, LotContents } from './lotContents';
import { ENV_UNITS, SKY_UNITS, SUN_UNITS } from './lightUnits';
import { MaterialLibrary, SUN_LIGHT_MASK } from './materials';
import { drawMesh, meshFrom } from './meshes';
import { Mirrors } from './mirrors';
import { buildModel, fileImported, importAhead, modelFileCensus, modelFilesSettled, onModelFileArrived, type ModelTemplate } from './models';
import { memoryCensus } from './memoryCensus';
import { GameBirds } from './birds';
import { colorBytes, depthBytes, DrawCounts, GpuTiming, ShaderWatch, type PassKind } from './gpuTiming';
import { ParallelPipelines } from './parallelPipelines';
import { SamplerCheck } from './samplerCheck';
import { steadyShaderKeys } from './shaderKeys';
import { natureDecor, windGust, windStill } from './nature';
import { PETALS } from '../shared/nature';
import { TownOverview, townLayer } from './overview';
import { Placing } from './placing';
import { Picker } from './picking';
import { gameLook, gamePostOptions, PostProcessing } from './post';
import { diagViewActive } from './shaders/diagView';
import { shadingFromSettings, softLookFromURL } from './shaders/softLook';
import { drawRoads } from './roads';
import { createWorldRoot, setDatum, toEngine } from './space';
import { Street } from './street';
import { Signals } from './signals';
import { Cars } from './cars';
import { ParkedRides } from './parkedRides';
import { SelectionMarker } from './selectionMarker';
import { CASTER_REACH, SunShadows } from './sunShadows';
import { TemplateCache } from './templateCache';
import { Thumbnails } from './thumbnail';
import { benchDiag } from '../../debug/benchFlags';
import { perfEnd, perfStart, perfTime, perfTimeAsync } from '../../debug/perf';
import { fetched, whenStreamSettled } from '../../assets/stream';
import { both, within } from '../../core/async';
import { LOOK } from '../look';
import type { RenderQuality } from '../quality';
import { GameCameraRig, keepUp } from '../shared/cameraRig';
import { BETA_LIMITS, DEFAULT_ORBIT, LENS, RADIUS_LIMITS, TOWN_FROM, townBlend } from '../shared/lens';
import { edgeFog, landEdge, menuFog, menuFogFrom, TOWN_RADIUS_LIMITS, townArrival, townClearance, townFar, townFog, townFogFrom, townNear, townOrbit } from '../shared/townCamera';
import { townView } from '../shared/townView';
import { landscapeRing, WATER_Y } from '../shared/nature';
import { setFogFrom } from './shaders/fog';
import { applyLandFixture, LAND_FIXTURE } from '../landFixture';
import { lotBuilding, lotUp, MOVER_GROW, TownBuildings, TownMovers, type Mover, type TownLotHouse } from './townView';
import { STOREY as TOWN_STOREY, type TownBuilding } from '../shared/townBuildings';
import { BoxGrid, type Box } from '../shared/boxGrid';
import { lotWallsHeight, rayPrism, rectFootprint } from '../shared/buildingPick';
import { closedParts, flatLids, inClosed, LID_THICK, viewedFlats, viewedLot, type ClosedPart } from '../shared/flatLids';
import { TownGroundMap } from './groundTiles';
import { LandHaze } from './landHaze';
import { cutInView } from '../shared/landVeil';
import { anyoneNear, DOOR_SWING, doorCell, doorLead, goingThrough, moverCells } from '../shared/doors';
import { smooth } from '../../game/noise';
import { withoutCloudShadows } from './cloudShadows';
import { createLighting, lightingAt, sunTurned, type Lighting, type RGB } from '../shared/environment';
import { playbackRate } from '../shared/gameSpeed';
import { Geo } from '../shared/geometry';
import { isPaths, pathLayers, pavedRects } from '../shared/paving';
import { pavingLayers } from '../shared/streetGround';
import { GrassPush } from '../shared/grass';
import { idleMoment } from '../shared/idle';
import { coveringSurface, HOUSE_MATERIALS, houseObjectKey, placeLights, WALL_STUB, type HouseSurfaces, type RoomLight, type RoomTiles } from '../shared/house';
import { WALL_HEIGHT } from '../shared/walls';
import { compose, linearHex, quatYawPitchRoll, toLinear, type Quat } from '../shared/math';
import { OrbitControls } from '../shared/orbit';
import { Flight, panBounds, Roam, WARM_LOTS, warmLots, type FocusPlot } from '../shared/focus';
import { CHUNK, chunkCoords, distanceToRect, RectGrid } from '../shared/chunks';
import { ModelRefresh } from '../shared/modelRefresh';
import { OverviewCamera, viewShift } from '../shared/overviewCamera';
import { FramePacer } from '../shared/pacing';
import { standInOf } from '../shared/modelEntry';
import { DEFAULT_RESOLUTION, renderScale, watchPixelRatio } from '../shared/resolution';
import { fieldLayers } from '../shared/street';
import { roadHeightsOf } from '../shared/roads';
import { darkTiles, windowlessRooms, type DarkTiles, type Windowless } from '../shared/windowless';
import type { FrameState } from '../../core/bridge';
import { Pose } from '../../core/snapshot';
import { groundDepth, lowestStorey, storeyOfRow, storeyRow, storeyShown, type FenceEdge, type FlatInfo, type GarageInfo, type ItemPlacement, type ObjectPlacement, type RoadPlan, type StoreyLayout, type WorldStructure } from '../../core/protocol';
import type { VisemeTrack } from '../../voice/visemes';
import type {
  BuildEffect,
  CameraPose,
  EdgePreview,
  FrameCounts,
  FrameProbe,
  GameShot,
  LandCursor,
  LiveRenderOptions,
  LotHighlight,
  ObjectBackgroundClips,
  ObjectClipInfo,
  GaragePreview,
  PorchPreview,
  PaintPreviewFace,
  PickResult,
  BuildingHit,
  DoorSound,
  PlacementGhost,
  RenderFailure,
  RenderPassInfo,
  Renderer,
  RendererDebugHooks,
  RendererDeps,
  RenderStats,
  RenderVram,
  MemoryCensus,
  ResizeHandle,
  RoomOverlayTile,
  SeenBird,
  SimStride,
  TownBuildStep,
  TownGhosts,
  TownShot,
  ViewRect,
  Shading,
  WallMode,
} from '../types';
import { LAND_CELL, NONE, type Land, type LandRect } from '../../game/land';
import type { UseLayer } from '../../game/uses';
import { bakeLand, copyLand, fullRect, marchToGround, sameLand } from '../shared/landBake';
import { OffThreadLand } from '../shared/renderData';
import { LandCursorDraw } from './landCursor';
import { TownGhostDraw, type Drape } from './townGhosts';
import { UseOverlayDraw } from './useOverlay';
import { installLandDebug } from './landDebug';
import type { Landscape } from './nature';
import { fixFrameClock, restartDice } from '../shared/frameClock';

/** The land's work after a stroke runs in slices this long (ms) a frame (`landPace`). */
const LAND_SLICE_MS = 6;

declare global {
  interface Window {
    /** The renderer's own hook for the dev tools and the benches (`?debug`); see `render/types.ts`. */
    __renderer?: unknown;
    __scene?: unknown;
    __app?: unknown;
  }
}

/** The town view's buildings fade in over this much of its blend (a dither: nothing pops in past `TOWN_FROM`). */
const TOWN_FADE = 0.03;
/** A far bike's frame (sRGB). */
const BIKE: readonly [number, number, number] = [0.22, 0.22, 0.24];

/** How often (ms) the flats of the building in view are asked again whether they may be shown open (`setFlatsOpen`). */
const FLATS_ASK_MS = 250;
/** How far (m) under the floor over it a closed part's lid lies while a storey over it is in view (`placeLidHeights`). */
const LID_UNDER = 0.02;

/** Longest wait (ms) for the birds of a game's first build (their models built): they'd pop in in play otherwise. */
const BIRDS_WAIT_MS = 3000;
/** Idle time (ms) a frame must have left for a model file to be read ahead of time (`parseAhead`). */
const PARSE_IDLE_MS = 8;
/** How long (ms) a catalog card must be pointed at before its model is read (`prepareAsset`). */
const PREPARE_AFTER_MS = 150;
/** Light is recomputed when the game's clock moved this many minutes (fast: the preset's `lightingStep`), at most every `LIGHT_MIN_MS`. */
const LIGHT_STEP = 0.5;
const LIGHT_MIN_MS = 100;
/** Frame after frame failing this many times is drawing broken for good. */
const FAILED_FRAMES_LIMIT = 60;
/** Nothing waits for frames longer than this (ms). */
const FRAME_WAIT_LIMIT_MS = 15_000;
/** The longest step a frame takes (ms): a stall isn't a jump. */
const MAX_FRAME_MS = 100;
/** Frames still drawn once a full-screen menu covers the view (its frosted glass shows the last). */
const HOLD_FRAMES = 2;
/**
 * The paused game's frames go on being drawn this long (ms) after a change before the picture is held
 * (`heldFrames.ts`): longer than the slowest rate anything is drawn again at (the sun's map and the lamps'
 * shadows 15 a second on Low, the reflections 10).
 */
const GAME_SETTLE_MS = 250;
/** While water is in view, a view that would be held is drawn this often (ms: 30 a second) so it goes on flowing. */
const WATER_FRAME_MS = 1000 / 30;
/** Metres the sun's shadow square reaches past the viewed lot. */
const SHADOW_MARGIN = 10;
/**
 * Metres past where a resident could cast into the sun's map that their shadow-only copy is still drawn
 * (`Characters.setCastTest`): the square moves in 8 m steps as the camera roams, and those it takes in are listed
 * the frame after it moved, so they are listed a step or two before.
 */
const CAST_SLACK = 16;
/** The game camera's target eases to the ground under it over this (s): `settleTarget`. */
const TARGET_EASE = 0.25;
/** The basement's pit (and the ground dug away over it) reaches this far past the plot (m), clear of walls on its edge. */
const PIT_MARGIN = 0.3;
/** The doors and gates opening or falling shut kept for the world's sound between its looks (`doorSounds`). */
const DOOR_SOUNDS = 16;
/** Camera distance below which the viewed lot's roof is hidden even with walls up. */
const ROOF_MIN_DISTANCE = 15;
/** A roof's height (m) over a house's top storey, at most, for a click on it (`buildingAt`). */
const PICK_ROOF = 4;
/** Main-thread time (ms) a world build runs before the frames it runs behind get one (`pace`). */
const BUILD_SLICE_MS = 8;
/**
 * How long after the game comes on screen the menus' town is let go (`releaseTown`): its destroy
 * (some 20 ms) after the reveal's frames and the fly-in's last ones.
 */
const TOWN_RELEASE_MS = 1000;
/** The day of the week the menus' town is shown on (0 Monday): a working day, its shops and offices open by day. */
const MENU_WEEKDAY = 2;
/** The same while the camera roams (a big world, or once it has settled on a lot of its own). */
const ROAM_SLICE_MS = 4;
/**
 * The same behind the loading screen (`setCovered`, and the warm-up): nobody sees the frames, and
 * each one given costs the build a frame's wait. At 4 ms a slice a game's first build ran a quarter of
 * the time; at this it runs most of it, and the loading screen's bar still moves (20 frames a second).
 */
const COVERED_SLICE_MS = 40;
/**
 * The same while the menus' town is built again over the one on screen (step 2's edits): the old
 * town stays drawn and the camera still, so a frame given up now and then shows nothing, and at 8 ms
 * a slice an edit's town took twice as long to come.
 */
const AGAIN_SLICE_MS = 32;
/**
 * Daylight doesn't reach under the ground or into a room without windows: no sun, only this much
 * of the sky fill and the environment (as seen, intensity times exposure: the same by day and by
 * night). A windowless room up here keeps that little, so its lamps tell. The basement in view
 * gets more, a soft fill to see it by: all its rooms are windowless, and residents switch an empty
 * room's lamps off (sim-core `lights.rs`), so it would be lit only where someone is. Still well
 * under daylight, so a lit room shows warm against the rest. By night its fill is as bright as by
 * day (lamps or not): the night's dim, deep blue sky lifted to it, its colour's brightness against
 * noon's too (`skyLuma`), up to `BASEMENT_NIGHT_GAIN` times; a windowless room's share stops at the
 * night's own.
 */
const WINDOWLESS_SKY = 0.12;
const WINDOWLESS_ENV = 0.12;
const BASEMENT_SKY = 0.35;
const BASEMENT_ENV = 0.35;
const BASEMENT_NIGHT_GAIN = 12;
/** How much of the moonlight grade (`nightGrade.ts`) is left while the basement is in view. */
const BASEMENT_NIGHT_GRADE = 0.3;
/** How bright the sky fill's colours are at noon (`skyLuma`): the basement's fill is matched to it. */
const NOON_SKY_LUMA = skyLuma(lightingAt(13 * 60, LOOK, createLighting()));

/** Residents this far (m) outside the viewed lot are still drawn (walking up to it). */
const SIM_VIEW_MARGIN = 2;
/** How far round the watch rectangle the sim keeps residents in full detail (m: sim-core `lod::VIEW_MARGIN`; `reportWatch`). */
const DETAIL_MARGIN = 5;
/**
 * At super speed (`setDetailTight`), the sim's full-detail area is the streamed chunks within this
 * many chunks of the one the camera looks at, not all of them: a resident out of it takes a turn
 * every game minute, which at super speed is every step or two anyway, and the town ticks faster.
 */
const TIGHT_CHUNKS = 1;
/** Residents further than this (m) from the camera's target, off the viewed lot, have no contact shadow. */
const FAR_SHADOW = 40;
/** `?residents=lot`: residents drawn only on the viewed lot, as before life on every lot. */
const RESIDENTS_LOT_ONLY = typeof location !== 'undefined' && new URLSearchParams(location.search).get('residents') === 'lot';

/** The camera's layers besides the world's that it keeps while the menus' town is shown (`showLayers`). */
const TOWN_CAMERA_LAYERS: readonly number[] = [pc.LAYERID_DEPTH, pc.LAYERID_IMMEDIATE, pc.LAYERID_UI];

/** A directional light's way before it's turned (PlayCanvas shines it along its entity's -y). */
const DOWN = new pc.Vec3(0, -1, 0);
/** (the warm-up's red copies, `drawRedCopies`.) */
const IDENTITY = Float32Array.of(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);

/** No neighbours' houses (to look through). */
const NO_HOUSES: readonly (readonly [number, number, number, number])[] = [];
/** How often (ms) the neighbours' houses are looked at for who is up in them at night (`findDarkRooms`). */
const NEIGHBOURS_EVERY_MS = 250;

/** Whether two worlds' garages are the same (field by field, as `sameObject`). */
function sameGarages(a: readonly GarageInfo[] | undefined, b: readonly GarageInfo[] | undefined): boolean {
  if (a === b) return true;
  if (!a || !b || a.length !== b.length) return false;
  return a.every((g, i) => {
    const h = b[i];
    return g.plot === h.plot && g.x0 === h.x0 && g.z0 === h.z0 && g.x1 === h.x1 && g.z1 === h.z1 && g.rot === h.rot;
  });
}

/** Whether two lists of flat records are the same, field by field (the porches, the porch pieces). */
function sameList<T extends object>(a: readonly T[] | undefined, b: readonly T[] | undefined): boolean {
  if (a === b) return true;
  if ((a?.length ?? 0) !== (b?.length ?? 0)) return false;
  if (!a || !b) return true;
  return a.every((p, i) => {
    const q = b[i] as Record<string, unknown>;
    return Object.entries(p).every(([k, v]) => q[k] === v);
  });
}

/**
 * Whether two placements draw the same: everything an object is drawn from, but its open state and
 * growth (drawn without a build: see `patchable`). (Wear, quality and what it's worth don't change the model.)
 */
function sameObject(a: ObjectPlacement, b: ObjectPlacement): boolean {
  // (Field by field: a big world's lot sends 6 000 objects with every opened door, and their keys as JSON
  // took 4-5 ms of each such message.)
  const pair = (u: readonly number[] | undefined, v: readonly number[] | undefined) => u === v || (!!u && !!v && u[0] === v[0] && u[1] === v[1]);
  return (
    a.id === b.id &&
    a.def === b.def &&
    a.x === b.x &&
    a.z === b.z &&
    a.rot === b.rot &&
    a.turn === b.turn &&
    pair(a.offset, b.offset) &&
    pair(a.size, b.size) &&
    a.tall === b.tall &&
    a.lift === b.lift &&
    a.w === b.w &&
    a.d === b.d &&
    (a.colors === b.colors || (!!a.colors && !!b.colors && JSON.stringify(a.colors) === JSON.stringify(b.colors)))
  );
}

/** `?stream=1`: the street streams its chunks round the camera in any town (else only in a big world's). */
const STREAM_FLAG = typeof location !== 'undefined' && new URLSearchParams(location.search).get('stream') === '1';

/** The plot (by index) under a ground point, else the nearest within `reach` m (every plot looked at: see `PlotFinder` for each frame's). */
function plotNear(world: WorldStructure, x: number, z: number, reach: number): FocusPlot | null {
  let best: FocusPlot | null = null;
  let bestD = reach;
  world.plots.forEach((p, i) => {
    const d = distanceToRect(x, z, p.x, p.z, p.w, p.d);
    if (d < bestD || (d === 0 && !best)) [best, bestD] = [{ id: i, x: p.x, z: p.z, w: p.w, d: p.d }, d];
  });
  return best;
}

/**
 * `plotNear` for the roaming camera, each frame: the plots in a grid of 16 m cells (made once a
 * world), and the last answer kept while the point, the reach and the world are the same (a still
 * camera asks nothing of the grid). The same plot as `plotNear` gives for a finite reach.
 */
class PlotFinder {
  private world: WorldStructure | null = null;
  private grid = new RectGrid<{ x: number; z: number; w: number; d: number; i: number }>(16);
  private readonly asked = { x: NaN, z: NaN, reach: NaN };
  private answer: FocusPlot | null = null;

  find(world: WorldStructure, x: number, z: number, reach: number): FocusPlot | null {
    if (world !== this.world) {
      this.world = world;
      this.grid = new RectGrid(16);
      world.plots.forEach((p, i) => this.grid.add({ x: p.x, z: p.z, w: p.w, d: p.d, i }));
      this.asked.x = NaN;
    }
    const a = this.asked;
    if (x === a.x && z === a.z && reach === a.reach) return this.answer;
    a.x = x;
    a.z = z;
    a.reach = reach;
    if (!Number.isFinite(reach)) return (this.answer = plotNear(world, x, z, reach));
    // (In plot order, as `plotNear` goes: the first of plots as near wins.)
    let best = -1;
    let bestD = reach;
    for (const c of this.grid.near({ x, z, w: 0, d: 0 }, reach)) {
      const d = distanceToRect(x, z, c.x, c.z, c.w, c.d);
      if (d < bestD || (d === bestD && c.i < best) || (d === 0 && best < 0)) [best, bestD] = [c.i, d];
    }
    const p = best >= 0 ? world.plots[best] : null;
    if (p?.x === this.answer?.x && p?.z === this.answer?.z && this.answer?.id === best) return this.answer;
    return (this.answer = p && { id: best, x: p.x, z: p.z, w: p.w, d: p.d });
  }
}

/** A world build stopped because the roaming camera settled on another lot before it was done (`pace`). */
class BuildSuperseded extends Error {}

/** The same part of the town in view (the messages make a new rect each time). */
/** Where step 2's ghosts lie on a landscape (`townGhosts.ts`): its ground, or its water's top where it is wet (a bridge's ghost over a river). */
function groundOrWater(scape: Landscape): Drape {
  const field = scape.heightField;
  return (x, z) => {
    const h = scape.heightAt(x, z);
    if (!field) return scape.waterEdge(x, z) > 0 ? Math.max(h, WATER_Y) : h;
    const [i, j] = [Math.round(x / LAND_CELL), Math.round(z / LAND_CELL)];
    if (i < 0 || j < 0 || i >= field.cols || j >= field.rows) return h;
    const k = j * field.cols + i;
    return field.wl[k] !== NONE && field.wl[k] > field.h[k] ? Math.max(h, field.wl[k] / 100) : h;
  };
}

/** A number of each landscape's own, for what is made from it (`LandHaze.set`). */
const landscapeIds = new WeakMap<object, number>();
let landscapesSeen = 0;
function landscapeId(scape: object): number {
  const id = landscapeIds.get(scape) ?? ++landscapesSeen;
  landscapeIds.set(scape, id);
  return id;
}

function sameView(a: ViewRect | null, b: ViewRect | null): boolean {
  return a === b || (!!a && !!b && a.x === b.x && a.z === b.z && a.w === b.w && a.d === b.d);
}

/** `?lightmix=sun,sky,env`: scales of the look's sun, sky fill and environment light (1 each by default). */
function lightMixFromURL(): [number, number, number] | null {
  const v = new URLSearchParams(location.search).get('lightmix');
  if (!v) return null;
  const [sun = 1, sky = 1, env = 1] = v.split(',').map(Number);
  return [sun, sky, env].map((x) => (Number.isFinite(x) ? x : 1)) as [number, number, number];
}

/** How bright a sky fill of `l`'s colours is, for an intensity of 1: the luminance of its sky and ground, averaged (linear). */
function skyLuma(l: Lighting): number {
  const luma = (c: RGB) => 0.2126 * toLinear(c.r) + 0.7152 * toLinear(c.g) + 0.0722 * toLinear(c.b);
  return Math.max((luma(l.skyColor) + luma(l.groundColor)) / 2, 1e-3);
}

/** A linear colour (the look's keys are used as linear) as PlayCanvas takes it (sRGB, linearised when lit). */
function gammaColor(c: RGB, scale = 1, out = new pc.Color()): pc.Color {
  const g = (v: number) => Math.pow(Math.max(0, v * scale), 1 / 2.2);
  return out.set(g(c.r), g(c.g), g(c.b), 1);
}

export class PlayCanvasRenderer implements Renderer, RendererDebugHooks {
  onFailure: ((why: RenderFailure, error?: unknown) => void) | null = null;

  private device!: pc.GraphicsDevice;
  private app!: pc.AppBase;
  /** The world, in game space (`space.ts`). */
  private root!: pc.Entity;
  /** The viewed lot's layers (its house, things, fences, paths, the tools' previews), raised by the datum over `root` (`space.ts` `setDatum`). */
  private lotRoot!: pc.Entity;
  /**
   * The datum (m, game space): on a land the town follows, the viewed lot's pad, else 0. The world is drawn this
   * much lower (the viewed lot at the engine's 0); the heights the renderer hands out are relative to it. The
   * menus' town draws at 0 (`enterTown`).
   */
  private datum = 0;
  private camera!: GameCamera;
  /** The image processing, FXAA and night grade on the camera (`post.ts`). */
  private post: PostProcessing | null = null;
  private sun!: pc.Entity;
  private sunShadows!: SunShadows;
  private disposed = false;

  private readonly diag = benchDiag();
  private readonly debug = new URLSearchParams(location.search).has('debug');
  /** `?hour=`: the light held at that hour (`setFixedHour` changes it). */
  private fixedHour: number | null;
  /** `?diag=freeze`: time stands still (`setFrozen` changes it). */
  private frozen = this.diag.has('freeze');

  /**
   * The player's camera input and the game camera's pose (the director's rig drives it too). One zoom from the play
   * camera out to the town view: its radius goes to `TOWN_MAX`, and past `TOWN_FROM`
   * its `shape` (`shared/townCamera.ts`, set with the world) eases the tilt, the pan bounds and the zoom's pace.
   */
  private readonly orbit = new OrbitControls({ ...DEFAULT_ORBIT }, { radius: TOWN_RADIUS_LIMITS, beta: BETA_LIMITS });
  private detachOrbit: (() => void) | null = null;
  private readonly gameRig: GameCameraRig;
  /** The overview camera's pose (`OverviewCamera` writes it; the camera takes it each town frame). */
  private readonly townPose = { alpha: 0, beta: 0.9, radius: 60, target: { x: 0, y: 0, z: 0 }, targetScreenOffset: { x: 0, y: 0 }, fov: LENS.fov };
  private readonly townRig = new OverviewCamera(this.townPose);

  private readonly pacer = new FramePacer();
  private options: LiveRenderOptions = { resolution: DEFAULT_RESOLUTION, cameraSensitivity: 1, fpsCap: 0 };
  private active = false;
  private covered = false;
  private coveredFrames = 0;
  private readonly frameWaiters: { left: number; resolve: () => void }[] = [];
  /** Frames drawn so far (`drawFrame` tells whether one was). */
  private framesDrawn = 0;
  private onFrame: ((now: number) => void) | null = null;
  private onView: (() => void) | null = null;
  private townListener: (() => void) | null = null;
  private lastTime = 0;
  private frameDt = 1000 / 60;
  private failedFrames = 0;

  private townMode = false;
  /** The camera glides down into the game: the town isn't held meanwhile. */
  private flyingIn = false;
  private townMinute = 12 * 60;
  private townMinuteGoal = 12 * 60;
  private lastMinute = 12 * 60;
  private lastLightMinute = -1;
  private lastLightAt = 0;
  private readonly lighting = createLighting();
  /**
   * The menus' town (`overview.ts`, on its layer; made at the first `showTown`), the shot
   * and highlights asked for, the town last asked to show, the builds of it queued (only the newest
   * is built: `townTicket`) and running, and the game camera's and the sun's layers while the town's
   * are on (`showLayers`).
   */
  private overview: TownOverview | null = null;
  /** The haze the menus' land fades into at its edge, and the curtain closing its cut (`landHaze.ts`); made with the first town shown. */
  private landHaze: LandHaze | null = null;
  /**
   * The game's: made the first time its camera is past its landscape (`hazeWorld`), for the landscape and the land as
   * they were then (`worldHazeFor`, its `landVersion`), and whether it shows now.
   */
  private worldHaze: LandHaze | null = null;
  private worldHazeFor: { scape: Landscape | null; version: number } = { scape: null, version: -1 };
  private worldHazeShown = false;
  private townShot: TownShot | null = null;
  private townHighlight: LotHighlight | null = null;
  private townWorld: WorldStructure | null = null;
  private townTicket = 0;
  private townQueue: Promise<void> = Promise.resolve();
  private townBuilds = 0;
  /** The town being built replaces one on screen (`showTown` while in the town's mode): see `AGAIN_SLICE_MS`. */
  private townAgain = false;
  private gameLayers: { camera: number[]; sun: number[] } | null = null;
  /** The surfaces, the look and the environment lighting (`materials.ts`). */
  private lib!: MaterialLibrary;
  /** The shading the materials are drawn in (`shaders/softLook.ts`); the setting's at start. */
  private shading: Shading = shadingFromSettings();
  private wallMode: WallMode = 'down';
  /** Something for a wall is in hand: the walls stand whatever `wallMode` says (`placing.ts`). */
  private wallsRaised = false;
  /** The camera is in the town view (past `TOWN_FROM`): the viewed lot's house stands whole (`shownWallMode`, `roofVisible`). */
  private townWhole = false;
  /** The cutaway's look bits last given to the walls (-1: none yet). */
  private cutLook = -1;

  // The world as built (`setWorld`).
  /** The world last asked for (built again when the storey in view changes), the one built, and the one waiting. */
  private lastWorld: WorldStructure | null = null;
  private builtWorld: WorldStructure | null = null;
  private builtStorey = 0;
  /** The world to build next, and whether only its residents are (`sims`: the rest is as built, patched). */
  private nextWorld: { world: WorldStructure; view: ViewRect | null; sims?: boolean } | null = null;
  /** The builds running (one after another), and whether the next is a build whatever it is (a file came: `refreshStreet`). */
  private worldBuilds: Promise<void> | null = null;
  private worldBuilding = false;
  private rebuildNext = false;
  /** When the running slice of a world build began (see `pace`). */
  private sliceStart = 0;
  /** Lot being shown; null = everything. */
  private view: ViewRect | null = null;
  /** The camera has been turned to a lot once (the first world's). */
  private cameraPlaced = false;
  /**
   * Storeys (sim-core `storeys.rs`): how many at and above the ground, the lowest (-1 with a
   * basement), where their rows are, and the one in view (see `storeyShown`: higher ones are
   * hidden, and in the basement only it shows).
   */
  private storeys = { count: 1, lowest: 0, layout: { depth: 0 } as StoreyLayout };
  private storey = 0;
  /** Build mode's helpers work in lot rows: this lifts them to the storey in view. */
  private storeyLayer!: pc.Entity;
  /** (Build mode's previews and overlays on it, the tile grid, `buildPreviews.ts`.) */
  private previews!: BuildPreviews;
  /** The viewed lot's house, the street's houses (`house.ts`), and the street (`street.ts`). */
  private house!: HouseBuilder;
  private street!: Street;
  /** Households' cars, parked and on the road (`cars.ts`). */
  private cars!: Cars;
  /** Other lots' bikes and e-scooters (the street leaves them out: they go off with their riders). */
  private parkedRides!: ParkedRides;
  /**
   * The town view's drawing (`townView.ts`, `groundTiles.ts`), all of it only while
   * `townView.blend` is over 0: the far lots' houses, the movers past the near drawing, the ground's colour map. What
   * the far lots were chosen by (the chunks streamed in, the street's list of houses).
   */
  private townBuildings!: TownBuildings;
  private townMovers!: TownMovers;
  /** The eye (game space) this frame, for the woods' impostors (`Landscape.frameTiles`). */
  private readonly eyeNow = { x: 0, y: 0, z: 0 };
  private townGround!: TownGroundMap;
  private townLotsBy: { chunks: unknown; houses: readonly TownLotHouse[] | null } = { chunks: null, houses: null };
  /** The far lots' houses drawn (`frameTownView`), their parts by lot id: lit in place as their people get up and go to bed (`findTownUp`). */
  private townLotParts = new Map<number, TownBuilding[]>();
  /** The far lots with someone up in their house (`findTownUp`: by lot id, and as a key), and when they were last looked for. */
  private townUp: ReadonlySet<number> = new Set();
  private townUpAt = 0;
  /** The night's scans' houses bucketed, by their list (`gridOf`). */
  private readonly grids = new WeakMap<object, { boxes: readonly Box[]; grid: BoxGrid }>();
  /** When the neighbours' houses were last looked at for who is up in them (`findDarkRooms`). */
  private neighboursAt = -Infinity;
  /** The weekday of the newest snapshot (0 = Monday: sim-core `clock::weekday`), for the town's opening hours. */
  private lastWeekday = 0;
  /** The town view's movers this frame (reused, a pool), and their colours by hex. */
  private readonly townMoverList: Mover[] = [];
  private readonly moverColors = new Map<string, readonly [number, number, number]>();
  private readonly moverAt = { x: 0, y: 0, z: 0 };
  /** The traffic lights at the lit junctions (sim-core `traffic.rs`, `signals.ts`). */
  private signals!: Signals;
  /** What grows and stands on the lot: the landscape and sky, objects and their clips, items (`lotContents.ts`). */
  private lot!: LotContents;
  /** The viewed house's interior lights (`lamps.ts`). */
  private lamps!: Lamps;
  /** (the mirrors' reflections, `mirrors.ts`; the menus' town held while it stands still, `heldFrames.ts`.) */
  private mirrors!: Mirrors;
  private held!: HeldFrames;
  /** The game is paused (the snapshot's speed 0): its picture may be held while nothing in it changes. */
  private gamePaused = false;
  /** The clock (ms) of what moves only for the eye (the wind, the clouds, the selection marker): still while the frame is held. */
  private shownMs = 0;
  /** (buy mode's thing in hand, the object picked up, the effects, `placing.ts`; the rebuilds files that came set off, gathered, `shared/modelRefresh.ts`.) */
  private placing!: Placing;
  private readonly modelRefresh = new ModelRefresh((urls) => this.refreshStreet(urls));
  /**
   * The lot layer (roads and paths, the house on every storey, the basement's pit, the fences):
   * built again only when what it is made of changes (`lotKeyOf`), so buying, selling and moving
   * furniture keep it. Its entities, the storey each is on (none: every storey), and its roofs.
   */
  private lotKey = '';
  private lotParts: { refs: unknown[]; key: string } | null = null;
  private lotEntities: { entity: pc.Entity; storey?: number }[] = [];
  private roofs: pc.Entity[] = [];
  private roofShown = false;
  /** A check keeps the roof on close up (`keepRoof`). */
  private roofKept = false;
  /** The simulation's plain walls, when the house builder can't read the layout (walls up, and the low ones). */
  private walls: pc.Entity | null = null;
  private wallsLow: pc.Entity | null = null;
  /** Rooms of the house (where its lamps' and fills' lights go). */
  private lotRooms: RoomTiles | null = null;
  /** Openings in the ground floor over stairs down to the basement (ground rows), seen from the ground up. */
  private stairwells: ViewRect[] = [];
  /**
   * The flats of the building in view (`WorldStructure.flats` of the viewed lot), whether each may be shown open
   * (`setFlatsOpen`), the parts that may not now (its flats, and a place's own ground floor under them: `closedParts`;
   * their plot ids as a key), when that was last asked, and their lids (one draw a storey, `placeLids`).
   */
  private viewFlats: readonly FlatInfo[] = [];
  private flatsOpen: ((plot: number) => boolean) | null = null;
  private closedParts: readonly ClosedPart[] = [];
  private closedKey = '';
  private flatsAskedAt = 0;
  private lids: { entity: pc.Entity; storey: number }[] = [];
  /** The house's garden trees and shrubs (`placeDecor`): a draw per model mesh, hidden while its basement is in view. */
  private decorTemplates!: TemplateCache<ModelTemplate>;
  /** The viewed lot's hedges (its fences in the hedge style), planted with its garden's shrubs ([x, z, scale, yaw]). */
  private lotHedges: [number, number, number, number][] = [];
  private readonly decor = new Map<string, { template: ModelTemplate; draws: Instances[] }>();
  /** The viewed lot's door leaves that swing, each with how far open it stands (1 open, as drawn; -1 not placed yet). */
  private doors: { leaf: HouseDoor; open: number }[] = [];
  /** Gates in the fences' meshes, each with how far open it stands (0 shut, as drawn) and the side it opens to. */
  private gates: { fences: Fences; leaf: GateLeaf; open: number; towards: number }[] = [];
  /** The other lots' gates (they swing: drawn here, not by the street), on their pads where the town follows the land. */
  private townGates: Fences | null = null;
  /** The doors and gates that opened or fell shut since `doorSounds` was last called (at most `DOOR_SOUNDS`), and the list handed out then. */
  private doorsHeard: DoorSound[] = [];
  private doorsHanded: DoorSound[] = [];
  /**
   * How far out each resident's hand reaches for a door's handle this frame, 0..1 (indexed by their
   * row; `goingThrough`'s `hand`), 0 where they go through no leaf that moves. The arm goes out with
   * the leaf's swing (`residents.ts`).
   */
  private readonly doorHand = new Float32Array(MAX_CHARACTERS);
  /** Pools of lamplight on the lawn under the garden's lamps. */
  private gardenPools: GardenPools | null = null;
  private readonly poolOn = (pool: GardenPool) => (pool.room !== undefined ? this.lamps.windowLight(pool.room) * curtainsLetThrough(pool, this.curtainsShut) : this.lamps.lampOn(pool.lamp)); // (as its lamp fades, or its room's lights, `lamps.ts`; less through curtains drawn.)
  /** The curtains drawn shut (the sim's `curtainsShut`): their windows' pools dim. */
  private curtainsShut: ReadonlySet<number> = new Set();
  /** Night level of the lamps at the last lighting update (`Lighting.lamps`), the lamps' colour and the windows' glow (linear). */
  private lampsLevel = 0;
  private readonly lampColor = linearHex(LOOK.lampColor);
  private readonly glowColor = linearHex(LOOK.windowGlow);
  private readonly emissiveTmp = new pc.Color();
  /** Rooms without windows in the whole town (`shared/windowless.ts`), and the tiles of those of the storey in view, kept out of daylight. */
  private windowless: Windowless = { rooms: new Set(), of: [] };
  private dimming: DarkTiles | null = null;
  /**
   * Rooms where everyone is asleep (lights out), rooms with someone awake (babies don't count, as
   * in sim-core `lights`), and rooms with anyone in them; where residents on the move are and
   * where their way goes (`MOVER` floats each, in lot rows); all as of this frame (`findDarkRooms`).
   */
  private readonly darkRooms = new Set<number>();
  private readonly awakeRooms = new Set<number>();
  private readonly occupiedRooms = new Set<number>();
  private readonly movers: number[] = [];
  /** The cells (`doorCell` m: `swingDoors`) someone on the move is in this frame. */
  private readonly moverCells = new Set<number>();
  /** Per neighbour's house (`Street.houses`): someone is up in there (scratch). */
  private readonly neighboursUp: boolean[] = [];
  /** Indices of the snapshot layout's sleeping action tags, and the life stages of babies. */
  private sleepActions: { tags: readonly string[]; ids: Set<number> } | null = null;
  private babies: { of: readonly unknown[]; ids: ReadonlySet<string> } | null = null;
  private unlistenFiles: (() => void) | null = null;
  private readonly decorScale = new Float32Array(16);
  private readonly decorQ: Quat = [0, 0, 0, 1];

  /**
   * The residents (`characters.ts`): posed each frame into the pose texture, with their contact shadows
   * (a matrix each, `simShadowMatrices`); how many the snapshot has, who is selected (the marker over
   * their head, `selectionMarker.ts`), who the camera keeps in the middle (`followSim`), the frame last
   * posed, and the lot whose arrival the camera was turned to (`buildSims`).
   */
  private characters!: Characters;
  /** The garden birds (`birds.ts`). */
  private birdLife!: GameBirds;
  private simShadow!: Instances;
  private readonly simShadowMatrices = new Float32Array(MAX_CHARACTERS * 16);
  private simCount = 0;
  private marker!: SelectionMarker;
  private selectedSim: number | null = null;
  private lastFrame: FrameState | null = null;
  private readonly markerHead = { x: 0, y: 0, z: 0 };
  private followIndex: number | null = null;
  /** The roaming camera's stream of chunks and its focus lot (`shared/focus.ts`), a flight across the world (`flyTo`). */
  private readonly roam = new Roam();
  /** The plot under the roaming camera's target (`roamFrame`). */
  private readonly plots = new PlotFinder();
  private readonly flight = new Flight();
  private readonly flightPose = { x: 0, z: 0, radius: 0 };
  /** A glide to a town point (`glideTo`): the resident it lands on (it goes where they go meanwhile) and what to do once there. */
  private glide: { follow: number | null; done: (() => void) | null } | null = null;
  private readonly glideSpot = { x: 0, y: 0, z: 0 };
  /** The far plane as the world's size and the preset set it (`buildWorld`): the play camera's, which the town view moves out from. */
  private playFar: number = LENS.far;
  /** The land and the landscape round it (m): what the town view's far plane reaches past. */
  private readonly worldReach = { x0: 0, z0: 0, x1: 0, z1: 0 };
  /** The menus' town's land and the landscape round it (`worldReach`'s, for the town `showTown` was given last). */
  private readonly townReach = { x0: 0, z0: 0, x1: 0, z1: 0 };
  /** Where the haze begins (m from the eye) as last set (`shaders/fog.ts` `setFogFrom`). */
  private fogFromSet = 0;
  /** How far into the town view the camera was drawn last (`applyTownCamera`: its planes and the sky are put back once it is out of it). */
  private townBlendDrawn = 0;
  /** The game camera's view-projection as this frame is drawn with it (`townView.project`). */
  private readonly townViewProj = new Float32Array(16);
  /** `townView.project`: a town point on the canvas (CSS pixels, and its depth in m), for the town tags. */
  private readonly projectTown = (x: number, y: number, z: number): [number, number, number] | null =>
    this.townMode || !this.active ? null : GameCamera.projectWith(this.townViewProj, x, y, z, this.cssSize.width, this.cssSize.height);
  private focusLotListener: ((plot: number) => boolean) | null = null;
  /** The lot (index) the roaming camera settled on last and the game took (it is not framed: `buildSims`). */
  private roamedTo: number | null = null;
  /** When the roaming camera's last focus lot was taken, and how long each took to be built in full (ms; the bench reads them). */
  private focusAsked: { plot: number; at: number } | null = null;
  private readonly focusSwitches: number[] = [];
  private watchListener: ((rect: [number, number, number, number]) => void) | null = null;
  /** The full-detail area kept to the chunks round the camera's (`setDetailTight`), and that chunk as last reported. */
  private detailTight = false;
  private detailChunk = '';
  /** The street building the chunks streamed in, between world builds; another stream change waits for it. */
  private streaming: Promise<void> | null = null;
  private streamDue = false;
  private streamSlice = 0;
  /** Whether the stream limits what the street draws (a big world, or `?stream=1`); else the whole town is drawn, as it always was. */
  private streamOn = false;
  /** Whether the camera has caught up with the resident it follows (`keepUp`: then it stays near them). */
  private followCaught = false;
  private readonly followHead = { x: 0, y: 0, z: 0 };
  private framedView = '';
  /**
   * Whether a resident at ground point (x, z) on storey `k` is drawn: on the viewed lot (and just round
   * it) as ever, but not in a flat of it that may not be shown open (`closedFlats`); elsewhere outdoors
   * within the preset's `residentRadius` of the camera's target and in the view (life on every lot),
   * not indoors on a lot not in view (its house is a silhouette there). `?residents=lot`: only the
   * viewed lot's, as before.
   */
  private readonly simInView = (x: number, z: number, k = 0): boolean => {
    if (this.diag.has('nosims') || inClosed(this.closedParts, x, z, k)) return false;
    if (this.inView(x, z, SIM_VIEW_MARGIN)) return true;
    const vp = this.simViewProj;
    const world = this.lastWorld;
    if (RESIDENTS_LOT_ONLY || !vp || !world) return false;
    const t = this.orbit.target;
    const r = this.quality.residentRadius;
    const dx = x - t.x;
    const dz = z - t.z;
    if (dx * dx + dz * dz > r * r) return false;
    const ix = Math.floor(x);
    const iz = Math.floor(z);
    if (ix >= 0 && iz >= 0 && ix < world.width && iz < groundDepth(world) && world.rooms[iz * world.width + ix]) return false;
    // In the view (a point a metre up, with some slack round the edges for the body).
    const y = 1;
    const wz = -z;
    const cw = vp[3] * x + vp[7] * y + vp[11] * wz + vp[15];
    if (!(cw > 0)) return false;
    const cx = vp[0] * x + vp[4] * y + vp[8] * wz + vp[12];
    const cy = vp[1] * x + vp[5] * y + vp[9] * wz + vp[13];
    return Math.abs(cx) <= cw * 1.15 && Math.abs(cy) <= cw * 1.3;
  };
  /** The camera's view-projection for this frame's `simInView` (null: none yet). */
  private simViewProj: Float32Array | null = null;

  private readonly cssSize = { width: 1, height: 1 };
  private sizeWatch: ResizeObserver | null = null;
  private unwatchPixelRatio: (() => void) | null = null;
  private renderScaleNow = 1;

  /** Picking (`picking.ts`: the last pick's ray, for `pickPlane` & co.), the resize arrows and the hover tile (`handles.ts`). */
  private picker!: Picker;
  private handles!: ResizeHandles;
  private hoverTile!: HoverTile;
  private readonly eyeTmp = { x: 0, y: 0, z: 0 };
  /** The residents pushing the lawn's grass aside (`pushGrass`). */
  private readonly grassPush = new GrassPush();
  private readonly grassHead = { x: 0, y: 0, z: 0 };
  private readonly vTmp = new pc.Vec3();
  private readonly vTmp2 = new pc.Vec3();
  private readonly qTmp = new pc.Quat();
  /** The sun's way it was last turned to (PlayCanvas space; `applyLighting`): none yet. */
  private readonly sunWay = new pc.Vec3(NaN, NaN, NaN);
  private readonly colorTmp = new pc.Color();
  /** `?lightmix=sun,sky,env` (dev server): each part of the daylight scaled, to compare them one by one with reference pictures. */
  private readonly lightMix = import.meta.env.DEV ? lightMixFromURL() : null;

  /** The frame probes (`addFrameProbe`), in the order they were added. */
  private readonly probes: FrameProbe[] = [];
  private readonly counts: FrameCounts = { drawCalls: 0, triangles: 0, shadowDrawCalls: 0, shadowTriangles: 0, mirrorDrawCalls: 0, mirrorTriangles: 0, lightingUpdates: 0, gpuFrames: 0, gpuMs: 0, residentsMs: 0, sunShadows: { whole: 0, moving: 0, why: {} } };
  private frameMs = 1000 / 60;
  private cpuMs = 0;
  private drawCalls = 0;
  private readonly gpuWanted = new Set<'overlay' | 'bench'>();
  /** (the benchmark's measures, `gpuTiming.ts`.) GPU time, draws per pass, shaders made. */
  private gpuTiming!: GpuTiming;
  private drawCounts!: DrawCounts;
  private shaderWatch!: ShaderWatch;
  /** The warm-up's pipelines made side by side (`warmDraw`). */
  private parallelPipelines!: ParallelPipelines;
  /** In dev, WebGL2's texture units each program uses (`samplerCheck.ts`; `window.__samplers()`). */
  private samplerCheck: SamplerCheck | null = null;
  /** The warm-up's frame, drawn next (`warmGame`). */
  private warmNext = false;
  /** The loading screen's listener (`onLoadStep`); the warm-up's own world builds (its lots) don't report. */
  private loadListener: ((label: string, done: number) => void) | null = null;
  private warming = false;
  /** The loading screen covers the view (`setCovered`): world builds run in longer slices. */
  private loadingCover = false;
  /** The birds of this game's first build have been waited for (`buildWorld`). */
  private birdsWaited = false;
  /** Model files to read ahead of time, nearest first (`parseAhead`), and whether that's under way. */
  private parseQueue: { url: string; file: string }[] = [];
  private parsing = false;
  /** The asset a catalog card points at, read after a moment (`prepareAsset`). */
  private prepareTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(
    private readonly deps: RendererDeps,
    private readonly quality: RenderQuality,
  ) {
    const params = new URLSearchParams(location.search);
    const hour = Number(params.get('hour'));
    this.fixedHour = Number.isFinite(hour) && params.has('hour') ? hour : null;
    this.gameRig = new GameCameraRig(this.orbit, { radius: RADIUS_LIMITS, beta: BETA_LIMITS }, (i, out) => this.simHead(i, out), () => this.orbit.stopInertia());
    this.gameRig.ground = (x, z) => this.targetGround(x, z);
  }

  async init(canvas: HTMLCanvasElement): Promise<void> {
    const measure = () => {
      this.cssSize.width = canvas.clientWidth || 1;
      this.cssSize.height = canvas.clientHeight || 1;
    };
    measure();
    this.sizeWatch = new ResizeObserver(() => {
      measure();
      this.applyResolution();
    });
    this.sizeWatch.observe(canvas);
    const forceWebGL = this.deps.backend === 'webgl' || new URLSearchParams(location.search).get('renderer') === 'webgl';
    this.device = await createDevice(canvas, forceWebGL);
    // (Before the first material is made: Low's shaders are built without the clouds' shadows.)
    if (!this.quality.cloudShadows) withoutCloudShadows(this.device);
    steadyShaderKeys();
    // Before anything compiles: every shader and pipeline is counted (`?diag=shaders` logs them).
    this.shaderWatch = new ShaderWatch(this.device, this.diag.has('shaders'));
    this.parallelPipelines = new ParallelPipelines(this.device);
    if (import.meta.env.DEV) {
      const check = (this.samplerCheck = new SamplerCheck(this.device));
      (window as unknown as { __samplers?: () => unknown }).__samplers = () => check.report();
      // (The bridge decks the street draws now, and whether it stands on the game's landscape's surface: `land.mjs`.)
      (window as unknown as { __streetBridges?: () => unknown }).__streetBridges = () => ({ decks: this.street.layout?.bridgeCount() ?? 0, onLand: !!this.lot.landscape?.surface && this.street.layout?.standsOn(this.lot.landscape.surface) });
      // (How far the street's heights stand off the game's landscape's surface at the points, m, the most: 0 when it stands
      // on the land as drawn; null with no surface: `land.mjs`.)
      (window as unknown as { __streetGap?: (points: [number, number][]) => number | null }).__streetGap = (points) => {
        const surface = this.lot.landscape?.surface;
        const street = this.street.layout;
        if (!surface || !street) return null;
        let most = 0;
        for (const [x, z] of points) most = Math.max(most, Math.abs((street.levelAt(x, z) ?? 0) - surface.at(x, z)));
        return most;
      };
      // (The game's ground as drawn at each point: what is built there (a road's height on a road), the terrain and the grass: `land.mjs`.)
      (window as unknown as { __groundDrawn?: (points: [number, number][]) => unknown }).__groundDrawn = (points) => {
        const scape = this.lot.landscape;
        return points.map(([x, z]) => ({ built: scape?.surface?.at(x, z) ?? null, ...(scape?.drawnAt(x, z) ?? { terrain: null, grass: null }) }));
      };
    }
    this.app = createApp(canvas, this.device);
    this.gpuTiming = new GpuTiming(this.device, this.passLabel);
    this.drawCounts = new DrawCounts(this.device);
    // A GPU reset (driver update, a laptop switching GPUs, a crashed GPU process): what was on the
    // GPU is gone, and the game's own buffers can't be rebuilt in place.
    this.device.on('devicelost', () => this.onFailure?.('device'));
    this.applyResolution();
    this.unwatchPixelRatio = watchPixelRatio(this.onResize);
    window.addEventListener('resize', this.onResize);

    this.lib = MaterialLibrary.for(this.app).setAssets(this.deps.assets);
    this.lib.anisotropy = this.quality.anisotropy;
    this.lib.finishLayers = this.quality.finishLayers && !this.diag.has('nolayers');
    this.lib.setShading(this.shading);

    const scene = this.app.scene;
    // The sky is the game's own (its dome, later): the environment only lights.
    scene.ambientLight = new pc.Color(0, 0, 0);
    // Distance fog in the horizon colour makes the hills fade into the sky (aerial haze).
    scene.fog.type = pc.FOG_EXP2;
    scene.fog.density = LOOK.fog;

    this.root = createWorldRoot(this.app);
    // The viewed lot's layers, raised by the datum over the world root (`space.ts` `setDatum`).
    this.lotRoot = new pc.Entity('lot');
    this.root.addChild(this.lotRoot);
    this.camera = new GameCamera(this.app.root, LENS);
    const cam = this.camera.camera;
    // (Not PlayCanvas's skybox: the environment atlas would draw as one.)
    cam.layers = cam.layers.filter((id) => id !== pc.LAYERID_SKYBOX);
    // The picture after the light: multisampling or FXAA, the image processing
    // and the night grade (`post.ts`; the diag views and `?diag=linear` without them).
    this.post = new PostProcessing(this.app, cam, gamePostOptions(this.samples, this.diag, diagViewActive, this.quality.leanTargets, this.quality.contactShading));
    this.post.setLook(gameLook(LOOK));
    // Leaf cards soften their edges with the picture's samples (none on Low, nor without the post-processing).
    this.lib.setCutoutCoverage(this.post.samples > 1);

    this.sun = new pc.Entity('sun');
    this.sun.addComponent('light', {
      type: 'directional',
      castShadows: !this.diag.has('noshadows'),
      // (Its strength is in its colour, `applyLighting`.)
      intensity: 1,
      // (The menus' town: fitted round this much of the view.)
      shadowDistance: 400,
      // Meshes left out of the lamps keep the sun (`MaterialLibrary.excludeFromLights`).
      mask: SUN_LIGHT_MASK,
    });
    this.app.root.addChild(this.sun);
    // The map's size, filtering and bias are the preset's (`sunShadows.ts`).
    this.sunShadows = new SunShadows(this.app, this.sun, { size: this.quality.shadowMapSize, rate: this.quality.shadowRate, still: !this.diag.has('wholeshadows') });
    // (lotContents.ts: the landscape, sky, objects and items.)
    this.lot = new LotContents(this.app, this.root, this.sun, this.camera.entity, this.deps, this.quality, { storey: () => this.storey, storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), inView: (x, z, m) => this.inView(x, z, m), closed: (x, z, k) => inClosed(this.closedParts, x, z, k) }, { pace: () => this.pace(), shadowsMoved: (urgent) => this.castersMoved(urgent), partsMoved: () => this.partsMoved(), landscapeChanged: () => this.lamps.exclude(this.lot.landscapeDraws()) /* (lamps.ts) */ });
    // (the menus' town on a layer of its own, `overview.ts`, the landscape's draws for it too; the sky on both.)
    this.lot.objectsRoot = this.lotRoot;
    const town = townLayer(this.app).id;
    this.lot.townLayer = town;
    for (const r of this.lot.sky.dome.findComponents('render') as pc.RenderComponent[]) r.layers = [pc.LAYERID_WORLD, town];

    // The residents (`characters.ts`), on moving seats with them, and their contact shadows.
    this.characters = new Characters(this.app, this.root, this.deps.assets, this.lib, this.deps.content, { refreshProps: true, castsShadows: this.quality.characterShadows, camera: () => this.camera.camera });
    this.characters.setRide((id) => this.lot.ride(id));
    this.characters.setLoop((id) => this.lot.loopPhase(id));
    this.characters.setSun(() => this.lighting.sunDirection);
    // (A redraw of the sun's map for their poses alone draws only them and the moving parts over the rest's picture: `SunShadows.posed`.)
    if (this.quality.characterShadows)
      this.sunShadows.movers = (out) => {
        this.characters.shadowCasters(out);
        this.lot.movingCasters(out);
        this.cars.movingCasters(out);
      };
    this.birdLife = new GameBirds(this.app, this.root, this.deps.assets, this.deps.content, this.quality.birds, { eye: (out) => this.camera.eyeInGame(out), storey: () => this.storey, roofShown: () => this.roofShown, calm: () => !!this.options.reducedMotion }, this.debug); // (the garden birds.)
    const blob = contactBlob(this.app);
    this.simShadow = new Instances(this.device, this.root, blob.mesh, blob.material, { name: 'simShadows', castShadows: false, receiveShadows: false, capacity: MAX_CHARACTERS, cull: false });
    this.marker = new SelectionMarker(this.app, this.root, this.deps.assets);

    // The viewed lot's house and the street round it (`house.ts`, `street.ts`).
    this.house = new HouseBuilder(this.app);
    const c = this.deps.content;
    this.house.setLooks({ coverings: c.wallCoverings, floors: c.floorCoverings, doors: c.doorStyles, windows: c.windowStyles, roofs: c.roofStyles, roofColors: c.roofColors });
    // Other lots show their things in the open, public places' and neighbours' gardens' (as the overview does).
    this.street = new Street(this.app, this.root, this.deps.assets, this.house, {
      lived: true,
      // (Not the bikes and e-scooters: they go off with their riders, `parkedRides.ts`.)
      look: (def) => {
        const d = c.object(def);
        return d && !d.vehicle ? { model: d.model ?? `model.${def}`, footprint: d.footprint ?? [1, 1], standIn: standInOf(c, d) } : undefined;
      },
      lod: this.quality.streetLod,
      shadows: this.quality.streetShadows,
      ground: true,
      fenceStyles: c.fenceStyles,
      farCull: this.quality.openWorld.farCull,
    });
    this.roam.streamScale = this.quality.openWorld.stream;
    this.cars = new Cars(this.app, this.root, this.quality.streetShadows);
    this.townBuildings = new TownBuildings(this.app, this.root);
    this.townMovers = new TownMovers(this.app, this.root);
    this.townGround = new TownGroundMap(this.app.graphicsDevice);
    this.parkedRides = new ParkedRides(this.app, this.root, this.deps.assets, c, this.quality.streetShadows);
    this.characters.setCarSeat((x, z) => this.cars.seatAt(x, z));
    // Only cars and residents moving where they cast into the sun's map draw it again.
    const inMap = (x: number, z: number) => this.sunShadows.reaches(x, z);
    this.cars.setShadowTest(inMap);
    this.signals = new Signals(this.app, this.root, this.deps.assets, { shadows: this.quality.streetShadows, lod: this.quality.streetLod });
    this.characters.setShadowTest(inMap);
    // (Their shadow-only copies only where they may cast: near the sun's square, and on the lot in view, whose rooms'
    // lamps cast too.)
    this.characters.setCastTest((x, z) => {
      const v = this.view;
      const r = CASTER_REACH;
      return this.sunShadows.reaches(x, z, CAST_SLACK) || (!!v && x >= v.x - r && x <= v.x + v.w + r && z >= v.z - r && z <= v.z + v.d + r);
    });
    this.characters.setCarRide((frame, x, z, yaw, aboard, out) => this.cars.ride(frame, x, z, yaw, aboard, out));
    // (Out on the hills between hamlets, on the land, not sunk into it: `Characters.setGround`. On a land the town
    // follows, on its roads and its lots' pads too.)
    this.characters.setGround((x, z) => this.lot.landscape?.heightAt(x, z) ?? 0);
    // (The cars, the parked rides and the traffic lights only on a land the town follows: elsewhere at 0, as ever.)
    this.cars.setGround(this.builtGround);
    this.parkedRides.setGround(this.builtGround);
    this.signals.setGround(this.builtGround);
    this.decorTemplates = new TemplateCache(this.app);
    this.storeyLayer = new pc.Entity('storeyLayer');
    this.lotRoot.addChild(this.storeyLayer);
    // Picking, the resize arrows (in render space) and the hover tile (in lot rows, on the storey layer).
    this.picker = new Picker(this.camera, this.lot, { storey: () => this.storey, storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), size: this.cssSize });
    this.handles = new ResizeHandles(this.device, this.lotRoot);
    this.hoverTile = new HoverTile(this.device, this.storeyLayer);
    // (the wall tools' previews, the Paint and Floor tools', the room overlay and the grid.)
    this.previews = new BuildPreviews(this.app, this.storeyLayer, this.house, { content: this.deps.content, world: () => this.lastWorld, view: () => this.view, storey: () => this.storey, rowOf: (k) => this.rowOf(k), wallMode: () => this.wallMode });
    if (!this.diag.has('nolamps')) this.gardenPools = new GardenPools(this.app, this.lotRoot);
    // (the interior lights, `lamps.ts`.)
    this.lamps = new Lamps(this.app, this.quality, { storey: () => this.storey, storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), town: () => this.townMode, whole: () => this.townWhole }, { house: this.house, lot: this.lot, pools: () => this.applyGardenPools(), casters: () => this.lampCasters() });
    // (the mirrors, `mirrors.ts`, their held reflections drawn again when the lamps, the sun or what the map shows changed; held frames, `heldFrames.ts`.)
    const mq = this.quality;
    this.mirrors = new Mirrors(this.app, this.lotRoot, this.camera.camera, this.sun.light!, { count: this.diag.has('nomirrors') ? 0 : mq.mirrors, scale: mq.mirrorScale, samples: this.samples, rate: mq.mirrorRate }, (mi, bits) => this.characters.drawsRows(mi, bits));
    this.lamps.onChanged = () => {
      this.mirrors.changed();
      this.pictureChanged();
    };
    this.sunShadows.onMoved = (urgent, casters) => {
      // (Whatever moves in the sun's map moves in the picture: a held game frame is drawn again.)
      this.pictureChanged();
      if (urgent) this.mirrors.changed();
      // (The lamps' shadows, `lampShadows.ts`: what moves on its own is told to them where it moves, `update`.)
      if (urgent && casters) this.lamps.castersMoved(true);
    };
    this.held = new HeldFrames(this.app, this.camera.camera, this.sun, mq.holdStillFrames);
    // (placing things, `placing.ts`; after the lamps, whose lights light the thing in hand.)
    this.placing = new Placing(this.app, this.lotRoot, this.storeyLayer, this.camera.camera, this.deps, this.lot, { storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), town: () => this.townMode, shadowsMoved: () => this.sunShadows.moved(), raiseWalls: (on) => this.raiseWalls(on) });
    // A file that failed has come: the thing in hand and the landscape's plants are built with it now, the rest
    // once the files settle (`shared/modelRefresh.ts`, held while the town streams: `holdModelRefresh`).
    this.unlistenFiles = onModelFileArrived(this.app, (url) => {
      this.placing.fileArrived(url);
      this.lot.refreshNature(url, () => this.disposed); // (lotContents.ts: the landscape's plants built again in place.)
      this.modelRefresh.fileArrived(url);
      this.birdLife.fileArrived(url);
      this.signals.fileArrived(url);
    });
    this.applyWallMode();

    const o = this.orbit;
    o.wheelDeltaPercentage = 0.012;
    o.panningSensibility = 90;
    o.mapPanning = true;
    o.panRadius = 24;
    o.inertia = 0.82;
    o.panningInertia = 0.82;
    this.detachOrbit = o.attach(canvas);

    if (this.debug) Object.assign(window, { __scene: this.app.scene, __app: this.app, __renderer: this });
    // (The land's own debug hook, `landDebug.ts`: scripted strokes through the land methods.)
    if (this.debug && import.meta.env.DEV) installLandDebug(this, () => this.landQueue);
    this.applyLighting(this.lastMinute);
    this.applyCamera();
    townView.project = this.projectTown;
    void this.characters.init();
    await this.setupEnvironment();
  }

  /** Image-based lighting from the sky (manifest key `environment.sky`, a `.pcenv` read by `env.ts`). */
  private async setupEnvironment(): Promise<void> {
    const entry = this.deps.assets.get('environment.sky', 'image');
    if (entry) await this.lib.loadEnvironment(entry.url);
  }

  // --- the frame loop ------------------------------------------------------------------

  run(onFrame: (now: number) => void, onView?: () => void): void {
    this.onFrame = onFrame;
    this.onView = onView ?? null;
    this.setActive(true);
  }

  setActive(active: boolean): void {
    if (active === this.active) return;
    this.active = active;
    if (active) {
      this.lastTime = 0;
      this.pacer.requestAnimationFrame(this.frame);
    } else this.pacer.cancelAnimationFrame();
  }

  hold(on: boolean): void {
    if (on === this.covered) return;
    this.covered = on;
    this.coveredFrames = HOLD_FRAMES;
  }

  framesOnDemand(on: boolean): void {
    if (on) this.restartClocks();
    this.pacer.manual(on);
    if (!on) {
      fixFrameClock(null);
      restartDice(false);
    }
  }

  restartClocks(): void {
    // What moves by a clock of its own (the residents' and the things' clips, the birds, the wind, the clouds) starts
    // again from 0, and the playback's dice from their start: the same frames from the same state draw the same
    // picture, however long the page has been open or the pose took to settle.
    this.shownMs = 0;
    this.characters.restartClock();
    this.lot.restartClocks();
    this.birdLife.life.restartClock();
    restartDice(true);
  }

  async restartLife(): Promise<void> {
    const world = this.builtWorld;
    if (!world) return;
    await this.characters.build(world.staffSims?.length ? { ...world, sims: [...world.sims, ...world.staffSims] } : world, false);
    // (The clocks first: the birds' timers are set by theirs as they perch, from the playback's dice.)
    this.restartClocks();
    if (this.lastWorld && !this.townMode) await this.settleBirds(this.lastWorld, this.view, this.lastMinute);
  }

  drawFrame(time: number): boolean {
    const drawn = this.framesDrawn;
    // (What is drawn at a rate of its own goes by the frame's time, until the next frame's.)
    fixFrameClock(time);
    this.pacer.release(time);
    return this.framesDrawn !== drawn;
  }

  framesRendered(frames: number): Promise<void> {
    return new Promise((resolve) => {
      const waiter = { left: frames, resolve };
      this.frameWaiters.push(waiter);
      setTimeout(() => {
        const i = this.frameWaiters.indexOf(waiter);
        if (i < 0) return;
        this.frameWaiters.splice(i, 1);
        resolve();
      }, FRAME_WAIT_LIMIT_MS);
    });
  }

  async texturesReady(): Promise<void> {
    // (A model file's textures load with it; a surface's on their own, and a model may ask for surfaces.)
    await modelFilesSettled(this.app);
    await this.lib.texturesReady();
  }

  async whenSettled(limitMs: number): Promise<void> {
    const until = performance.now() + limitMs;
    const left = () => Math.max(0, until - performance.now());
    await within(whenStreamSettled(), left(), undefined);
    await this.streamSettled(left());
    while (!this.disposed && this.building() && performance.now() < until) await new Promise((resolve) => setTimeout(resolve, 100));
  }

  building(): string {
    return [this.worldBuilding ? 'world' : '', this.streaming ? 'stream' : ''].filter(Boolean).join('+');
  }

  async streamSettled(limitMs: number): Promise<void> {
    const until = performance.now() + limitMs;
    // (A frame first: the camera's last turn may stream more chunks in.)
    await within(this.framesRendered(1), Math.min(limitMs, 250), undefined);
    while (!this.disposed && (this.streaming || this.streamDue || this.worldBuilds) && performance.now() < until) {
      await new Promise((resolve) => setTimeout(resolve, 30));
    }
  }

  setCovered(on: boolean): void {
    this.loadingCover = on;
  }

  onLoadStep(fn: ((label: string, done: number) => void) | null): void {
    this.loadListener = fn;
  }

  /** A world build's step for the loading screen (not the warm-up's own builds of other lots). */
  private loadStep(label: string, done: number): void {
    if (!this.warming) this.loadListener?.(label, done);
  }

  parseAhead(files: readonly { url: string; file: string }[]): void {
    this.parseQueue = files.filter((f) => !fileImported(this.app, f.url));
    void this.parseFiles();
  }

  /**
   * Reads the files `parseAhead` asked for, one per idle moment with time to spare, each once it's
   * downloaded: not while a world builds or the street streams, whose slices have the frames' spare time.
   */
  private async parseFiles(): Promise<void> {
    if (this.parsing) return;
    this.parsing = true;
    try {
      while (!this.disposed && this.parseQueue.length) {
        const spare = (await idleMoment()).timeRemaining();
        if (this.disposed) return;
        const held = this.worldBuilding || this.streaming || this.townMode;
        if (!held && spare < PARSE_IDLE_MS) continue;
        const i = held ? -1 : this.parseQueue.findIndex((f) => fetched(f.file));
        // (Held, or none downloaded yet, the stream bringing them: look again in a while.)
        if (i < 0) {
          await new Promise((resolve) => setTimeout(resolve, 1000));
          continue;
        }
        const [next] = this.parseQueue.splice(i, 1);
        if (!fileImported(this.app, next.url)) await importAhead(this.app, next.url);
      }
    } finally {
      this.parsing = false;
    }
  }

  prepareAsset(key: string, kind: 'model' | 'wall' | 'floor'): void {
    clearTimeout(this.prepareTimer);
    this.prepareTimer = setTimeout(() => {
      if (this.disposed) return;
      if (kind !== 'model') {
        // Its surface made (its textures loading), as the house will draw it.
        const surface = coveringSurface(kind, key);
        if (surface) this.house.material(surface.key, true, surface.pattern);
        return;
      }
      const url = this.deps.assets.has(key, 'model') ? this.deps.assets.get(key, 'model')?.url : undefined;
      if (url && !fileImported(this.app, url)) void importAhead(this.app, url);
    }, PREPARE_AFTER_MS);
  }

  private readonly frame = (time: number): void => {
    if (!this.active || this.disposed) return;
    this.pacer.requestAnimationFrame(this.frame);
    // Behind a full-screen menu: its last frames, then only those something waits for.
    if (this.covered) {
      if (this.coveredFrames > 0) this.coveredFrames--;
      else if (!this.frameWaiters.length) return;
    }
    const start = performance.now();
    // (Never less than none: the frames after fixed-step playback, back on the wall clock, may be timed before its last.)
    const dt = (this.frameDt = this.frozen ? 0 : Math.min(this.lastTime ? Math.max(0, time - this.lastTime) : 1000 / 60, MAX_FRAME_MS));
    this.lastTime = time;
    // (What moves only for the eye, the wind, the clouds and the selection marker's bob, stands still while the frame is
    // held, and goes on from there when it is drawn again: `heldFrames.ts`.)
    const shownDt = this.held.holding ? 0 : dt;
    this.shownMs += shownDt;
    const probes = this.probes;
    let updated = start;
    if (probes.length) this.drawCounts.reset();
    // (This frame's draws only: those made between frames, a held frame's reflections or a save's picture, went into
    // the next one's count, F3 showing 1,483 where the frames drew about 1,000.)
    this.device._drawCallsPerFrame = 0;
    try {
      for (const probe of probes) probe.before(start);
      this.onFrame?.(this.pacer.frameTime || performance.now());
      updated = performance.now();
      if (this.townMode) this.townFrame(dt);
      else {
        this.orbit.step(dt, Math.max(0.05, (dt * 60) / 1000));
        this.roamFrame(dt);
      }
      this.sunShadows.frame();
      this.applyCamera();
      this.pushGrass(dt);
      this.lot.frame(shownDt, this.townMode); // (the sky round the camera, the clouds' shadows, the grass in view.)
      if (!this.townMode) this.updateCutaway();
      // (The marker faces the camera as it is now, after this frame's input.)
      this.placeMarker();
      this.handles.frame(this.camera.eyeInGame(this.eyeTmp));
      if (this.townMode) this.townListener?.();
      else this.onView?.();
      this.app.update(shownDt / 1000);
      // (the reflections drawn this frame, `mirrors.ts`; the menus' town held while it stands still, and the game while
      // it is paused and nothing in it changes, `heldFrames.ts`.)
      this.mirrors.frame(this.characters, this.simCount, this.townMode);
      this.holdIfStill();
      // (the warm-up's frame, `warmGame`, undone once drawn.)
      const warm = this.warmNext ? this.warmFrame() : null;
      this.warmNext = false;
      try {
        if (this.held.draw(this.frameWaiters.length > 0 || !!warm)) {
          this.app.render();
          this.drawCalls = this.device._drawCallsPerFrame;
          this.device._drawCallsPerFrame = 0;
        } else this.drawCalls = 0;
      } finally {
        warm?.();
      }
      this.failedFrames = 0;
    } catch (err) {
      if (this.failedFrames === 0) console.error('[render] a frame failed', err);
      if (++this.failedFrames === FAILED_FRAMES_LIMIT) this.onFailure?.('frames', err);
      return;
    }
    for (let i = this.frameWaiters.length - 1; i >= 0; i--) {
      const w = this.frameWaiters[i];
      if (--w.left <= 0) {
        this.frameWaiters.splice(i, 1);
        w.resolve();
      }
    }
    this.framesDrawn++;
    const end = performance.now();
    this.cpuMs += (end - start - this.cpuMs) * 0.1;
    if (dt > 0) this.frameMs += (dt - this.frameMs) * 0.1;
    if (probes.length) {
      // (`gpuTiming.ts`.)
      const c = this.counts;
      const d = this.drawCounts;
      c.drawCalls = this.drawCalls;
      c.triangles = d.allTriangles;
      c.shadowDrawCalls = d.draws[1];
      c.shadowTriangles = d.triangles[1];
      c.mirrorDrawCalls = d.draws[2];
      c.mirrorTriangles = d.triangles[2];
      c.gpuFrames = this.gpuTiming.frames;
      c.gpuMs = this.gpuTiming.ms;
      c.sunShadows = this.sunShadows.draws;
      for (const probe of probes) probe.after(start, updated, end, c);
    }
  };

  /**
   * Whether this frame may be held (`heldFrames.ts`): the menus' town standing still, nothing of it being built; the
   * game paused (residents, objects, birds and cars stand still then), nothing being built or streamed in. In the
   * game, what changes the picture in place is told (`pictureChanged`), and frames are drawn for `GAME_SETTLE_MS`
   * after it (what is drawn at a rate of its own: the sun's map, the reflections, the lamps' shadows). With water in
   * view, a held view goes on being drawn `WATER_FRAME_MS` apart, so the water goes on flowing.
   */
  private holdIfStill(): void {
    const held = this.held;
    if (this.townMode) {
      held.settleMs = 0;
      held.active = !this.flyingIn && !this.worldBuilding && this.townBuilds === 0; // (nor while the overview builds.)
      held.motionMs = held.active && this.waterInView() ? WATER_FRAME_MS : 0;
      return;
    }
    held.settleMs = GAME_SETTLE_MS;
    held.active = this.gamePaused && !this.worldBuilding && !this.worldBuilds && !this.streaming && !this.streamDue && !this.warming;
    held.motionMs = held.active && this.waterInView() ? WATER_FRAME_MS : 0;
    if (!held.active) return;
    // What moves in place while the game is paused: a resident posed (a voice's lips, a look changed), the thing in
    // hand and the build effects, a shadow map's redraw still to come.
    if (this.characters.posed || this.placing.animating || this.sunShadows.waiting) held.changed();
  }

  /** Whether the water of the land in view flows on screen (`Landscape.waterInView`): a held view goes on being drawn, slower. */
  private waterInView(): boolean {
    const camera = this.camera.camera;
    return !!camera && !!this.landInView()?.scape.waterInView(camera.frustum);
  }

  /** Something in the game's picture changed that the held frame can't see (`heldFrames.ts`): it is drawn again. */
  private pictureChanged(): void {
    if (!this.townMode) this.held?.changed();
  }

  // --- the camera ----------------------------------------------------------------------

  /** The menus' camera's view offset in game space (`viewShift`), worked out in place. */
  private readonly townShift: [number, number, number] = [0, 0, 0];

  /** The game's built surface at a point (`BuiltSurface.at`), 0 off a land the town follows: what the cars, the parked rides and the traffic lights stand on. */
  private readonly builtGround = (x: number, z: number): number => this.lot.landscape?.surface?.at(x, z) ?? 0;

  /** The menus' land's height at a point (0 before it has one): for the camera's planes (`applyTownCamera`). */
  private readonly townHeight = (x: number, z: number): number => this.lot.landscapeOf('town')?.heightAt(x, z) ?? 0;

  /** Puts the camera where the game (or the overview) has it this frame. */
  private applyCamera(): void {
    if (this.townMode) {
      const p = this.townPose;
      // (The menus' town has the full town view's planes, sky and haze at any distance: `applyTownCamera`.)
      const flat = Math.sin(p.beta) * p.radius;
      const [ex, ey, ez] = [p.target.x + Math.cos(p.alpha) * flat, p.target.y + Math.cos(p.beta) * p.radius, p.target.z + Math.sin(p.alpha) * flat];
      // (Where the camera is drawn from: moved by the view offset, tens of metres from far out. The near plane's
      // clearance and the land's veil go by it.)
      const shift = viewShift(p.alpha, p.beta, p.targetScreenOffset.x, p.targetScreenOffset.y, this.townShift);
      this.applyTownCamera(p.radius, p.alpha, p.beta, ex, ey, ez, true, shift);
      this.landHaze?.show(p.radius, ex + shift[0], ez + shift[2]);
      // (The contact shading faded out by the distance as the game's town view fades it, `frameTownView`.)
      this.post?.setContact(1 - townBlend(p.radius));
      this.camera.setPose(p.alpha, p.beta, p.radius, p.target.x, p.target.y, p.target.z, p.targetScreenOffset.x, p.targetScreenOffset.y);
    } else {
      const o = this.orbit;
      // (The eye: target + radius × (cos α sin β, cos β, sin α sin β), game space.)
      const flat = Math.sin(o.beta) * o.radius;
      const [ex, ey, ez] = [o.target.x + Math.cos(o.alpha) * flat, o.target.y + Math.cos(o.beta) * o.radius, o.target.z + Math.sin(o.alpha) * flat];
      this.applyTownCamera(o.radius, o.alpha, o.beta, ex, ey, ez, false);
      this.hazeWorld(o.alpha, o.beta, ex, ey, ez);
      this.camera.setPose(o.alpha, o.beta, o.radius, o.target.x, o.target.y, o.target.z);
    }
    this.camera.viewProjectionInto(this.townViewProj);
    const o = this.orbit;
    townView.blend = this.townMode ? 0 : townBlend(o.radius);
    townView.radius = o.radius;
    townView.target.x = o.target.x;
    townView.target.z = o.target.z;
  }

  /**
   * The town view's share of the camera (`shared/townCamera.ts`), before it is posed:
   * for an orbit `radius` m out along `alpha`, tilted `beta`, with its eye at (`ex`, `ey`, `ez`), the near plane
   * moved out (the roads keep their depth from 900 m) and the far plane past the whole world, the sky's dome out
   * with it, and the haze thinned, begun further out and thick enough where the land ends in the picture to fade
   * its edge. Up to `TOWN_FROM` all of them are the play camera's own: nothing is set that wasn't. `menus`: the menus' town is up, which is fully in
   * the town view at any distance (its land, `townReach`), with a haze of its own (`menuFog`). With the play
   * camera's planes there, its sky's dome (700 m out, drawn over whatever is further) hid all but the near town
   * from the main menu's low shot, and from the new-game screens zoomed out the whole picture was the dome's flat
   * underside, the lots' tags floating on grey.
   */
  private applyTownCamera(radius: number, alpha: number, beta: number, ex: number, ey: number, ez: number, menus: boolean, shift?: readonly [number, number, number]): void {
    const blend = menus ? 1 : townBlend(radius);
    const r = menus ? this.townReach : this.worldReach;
    if (blend > 0) {
      // (The menus' land round the eye too, as far as the near plane reaches, from where the camera is drawn (`shift`):
      // `townClearance`.)
      const clearance = menus ? townClearance(ex + (shift?.[0] ?? 0), ey + (shift?.[1] ?? 0), ez + (shift?.[2] ?? 0), radius, this.townHeight) : ey - this.groundY(ex, ez);
      const far = townFar(radius, this.playFar, ex, ey, ez, r.x0, r.z0, r.x1, r.z1, blend);
      this.camera.setPlanes(townNear(radius, clearance, blend), far);
      this.lot.skyReach(far, blend);
    } else if (this.townBlendDrawn > 0) {
      this.camera.setPlanes(0, this.playFar);
      this.lot.skyReach(this.playFar, 0);
    }
    this.townBlendDrawn = blend;
    // The haze: the play camera's up to `TOWN_FROM`. From higher up it would wash the town out (and the mountains
    // round a mountain town, which are meant to be seen): thinner, from well past the town, and where the land ends
    // in the picture still enough to fade its edge into the sky.
    const mountains = this.lot.landscapeOf(menus ? 'town' : 'world')?.mountainous ? 0.35 : 1;
    const from = menus ? menuFogFrom(radius) : townFogFrom(radius);
    const own = LOOK.fog * (menus ? menuFog(radius, mountains < 1) : townFog(radius));
    let fog = own;
    if (blend > 0) {
      const spread = Math.atan(Math.tan(LENS.fov / 2) * (this.cssSize.width / Math.max(1, this.cssSize.height)));
      fog = Math.max(own, edgeFog(radius, from, landEdge(ex, ey, ez, alpha, spread, r.x0, r.z0, r.x1, r.z1), beta) * blend * mountains);
    }
    if (Math.abs(fog - this.app.scene.fog.density) > fog * 0.01) this.app.scene.fog.density = fog;
    this.setFogFrom(from);
  }

  /**
   * The game's landscape's cut edge closed in the haze's colour and its land faded into the haze at its edge, as the
   * menus' are (`landHaze.ts` `hold`), while that edge is in the picture (`cutInView`): from the town view far out
   * over a small land's corner, the eye past the landscape looks back at its cut, the ground tiles' skirts and the sea's
   * sheet under the hills. With the eye (at `ex`, `ey`, `ez`, looking along `alpha`, `beta`) over the landscape, as it
   * nearly always is, nothing is worked out, drawn or told.
   */
  private hazeWorld(alpha: number, beta: number, ex: number, ey: number, ez: number): void {
    const r = this.worldReach;
    const scape = this.lot.landscapeOf('world');
    if (!scape || (ex >= r.x0 && ex <= r.x1 && ez >= r.z0 && ez <= r.z1)) {
      if (this.worldHazeShown) this.worldHaze?.hold(0);
      this.worldHazeShown = false;
      return;
    }
    const haze = (this.worldHaze ??= new LandHaze(this.app, this.root));
    const made = this.worldHazeFor;
    if (made.scape !== scape || made.version !== scape.landVersion) {
      [made.scape, made.version] = [scape, scape.landVersion];
      haze.set(r, (x, z) => scape.heightAt(x, z), pc.LAYERID_WORLD, `${landscapeId(scape)}|${scape.landVersion}`);
    }
    // (The eye in game space: the camera's heights are the engine's, the datum below the land's.)
    const k = cutInView(r, haze.positions, ex, ey + this.datum, ez, alpha, beta, LENS.fov, this.cssSize.width / Math.max(1, this.cssSize.height));
    if (k > 0 || this.worldHazeShown) haze.hold(k);
    this.worldHazeShown = k > 0;
  }

  /** Where the haze begins (m from the eye), told to the shaders only when it moved (`shaders/fog.ts`). */
  private setFogFrom(metres: number): void {
    if (Math.abs(metres - this.fogFromSet) < 0.05) return;
    this.fogFromSet = metres;
    setFogFrom(this.device, metres);
  }

  cameraPose(): CameraPose {
    const o = this.orbit;
    return { alpha: o.alpha, beta: o.beta, radius: o.radius, target: [o.target.x, o.target.y, o.target.z] };
  }

  setCameraPose(pose: CameraPose): void {
    const o = this.orbit;
    [o.target.x, o.target.y, o.target.z] = pose.target;
    o.alpha = pose.alpha;
    o.beta = pose.beta;
    o.radius = pose.radius;
    o.checkLimits();
    // Placed: the next world doesn't turn it to its lot.
    this.cameraPlaced = true;
  }

  focus(x: number, z: number): void {
    // Keep the viewing angle; move the camera with its target.
    const o = this.orbit;
    this.flight.stop();
    this.glide = null;
    o.target.x = x;
    o.target.y = this.targetGround(x, z);
    o.target.z = z;
    o.radius = Math.min(o.radius, 32);
  }

  flyTo(x: number, z: number): void {
    const o = this.orbit;
    o.stopInertia();
    this.glide = null;
    this.flight.start(o.target.x, o.target.z, o.radius, x, z, Math.min(Math.max(o.radius, 24), 40), !!this.options.reducedMotion);
  }

  glideTo(x: number, z: number, radius?: number, follow?: number, done?: () => void): void {
    const o = this.orbit;
    o.stopInertia();
    o.takeMoved();
    const limits = o.limits.radius;
    const to = Math.min(limits[1], Math.max(limits[0], radius ?? townArrival(o.radius, follow !== undefined)));
    this.glide = { follow: follow ?? null, done: done ?? null };
    this.flight.start(o.target.x, o.target.z, o.radius, x, z, to, !!this.options.reducedMotion);
  }

  onFocusLot(listener: ((plot: number) => boolean) | null): void {
    this.focusLotListener = listener;
  }

  onWatch(listener: ((rect: [number, number, number, number]) => void) | null): void {
    this.watchListener = listener;
  }

  setDetailTight(tight: boolean): void {
    if (tight === this.detailTight) return;
    this.detailTight = tight;
    this.reportWatch();
  }

  /**
   * Each frame, after the camera's input: a flight under way, then the chunks round the camera
   * (streamed into the street) and the lot it has settled on (`onFocusLot`).
   */
  private roamFrame(dt: number): void {
    const o = this.orbit;
    const glide = this.glide;
    if (glide && this.flight.active) {
      // The player's own pan or zoom takes the camera back: the glide ends where it is (and selects nobody).
      if (o.takeMoved()) {
        this.flight.stop();
        this.glide = null;
      } else if (glide.follow !== null && glide.follow < this.simCount && this.characters.spot(glide.follow, this.glideSpot)) {
        // (Who it lands on walks on meanwhile.)
        this.flight.aim(this.glideSpot.x, this.glideSpot.z);
      }
    }
    if (this.flight.step(dt / 1000, this.flightPose)) {
      o.target.x = this.flightPose.x;
      o.target.z = this.flightPose.z;
      o.radius = this.flightPose.radius;
      o.checkLimits();
      if (!this.flight.active && this.glide) {
        const done = this.glide.done;
        this.glide = null;
        done?.();
      }
    }
    const world = this.builtWorld;
    if (!world || !this.street.layout) return;
    this.followShadows();
    this.frameTownView();
    this.street.frame(o.target.x, o.target.z);
    this.lot.landscape?.frameTiles(o.target.x, o.target.z, this.diag.has('notownwoods') ? 0 : this.townBlend, o.radius, this.diag.has('noimpostors') ? undefined : this.camera.eyeInGame(this.eyeNow));
    this.roam.pinned = this.followIndex !== null || this.gameRig.active || this.flight.active;
    const aspect = this.cssSize.width / Math.max(1, this.cssSize.height);
    const fovH = 2 * Math.atan(Math.tan(LENS.fov / 2) * aspect);
    const changed = this.roam.update({ x: o.target.x, z: o.target.z, radius: o.radius, alpha: o.alpha, fovH }, performance.now(), (x, z, reach) => this.plots.find(world, x, z, reach), (plot) => {
      const taken = !!this.focusLotListener?.(plot.id);
      if (taken) {
        this.roamedTo = plot.id;
        this.focusAsked = { plot: plot.id, at: performance.now() };
      }
      return taken;
    });
    // (Kept to the chunks round the camera's: it moved to another.)
    if (!changed && this.detailTight && this.detailChunk !== `${Math.floor(o.target.x / CHUNK)},${Math.floor(o.target.z / CHUNK)}`) this.reportWatch();
    // (The street's chunks built since: what stands on their roads shown in them.)
    if (!changed && this.streamOn && this.street.shownChunks !== this.onRoadsIn) this.viewOnRoads(o.target.x, o.target.z);
    if (!changed) return;
    this.pictureChanged();
    if (this.streamOn && this.street.setChunks(this.roam.chunks)) this.streamStreet();
    this.viewOnRoads(o.target.x, o.target.z);
    this.reportWatch();
  }

  /** How far into the town view the drawing is (`townView.blend`; 0 with `?diag=notown`: the drawing as before it). */
  private get townBlend(): number {
    return this.diag.has('notown') ? 0 : townView.blend;
  }

  /**
   * The town view's drawing for a world (`townView.ts`): the ground's colour map (once per town), and the far lots
   * chosen again on the next frame.
   */
  private buildTownView(world: WorldStructure): void {
    this.townGround.bake(world);
    this.townLotsBy = { chunks: null, houses: null };
  }

  /**
   * Each frame, the town view by its blend (`townView.blend`; nothing at 0: the play camera draws as it always has):
   * the buildings fading in, the far LOD's houses the town view's (the street's own hidden), the far lots those of the
   * chunks not streamed in, and the ground's colour map.
   */
  private frameTownView(): void {
    const blend = this.townBlend;
    this.street.setTownView(blend > 0);
    // (From the full town view the small things in the gardens and streets are a pixel or two: not drawn. The contact
    // shading, a hand's width of shade, fades out with the town view coming in.)
    this.street.setSmallHidden(blend >= 1);
    this.post?.setContact(1 - blend);
    this.townBuildings.show(this.diag.has('notownbuildings') ? 0 : Math.min(1, blend / TOWN_FADE));
    // (The ground tiles' materials asked for only while the map is, or is to be, on them. In with the buildings: what
    // they stand on is in it. The farmland isn't: its own map draws it at every distance, `FieldMap`.)
    const mapped = this.diag.has('notownground') ? 0 : Math.min(1, blend / TOWN_FADE);
    if (mapped > 0 || this.townGround.active) this.townGround.apply(this.lot.landscape?.groundMaterials() ?? [], mapped);
    // (The lots' buildings the street draws closed, their windows lit by the hour.)
    if (this.street.setTime(this.lastMinute, this.lastWeekday, this.lampsLevel > 0)) this.pictureChanged();
    if (blend <= 0) return;
    this.townBuildings.setTime(this.lastMinute, this.lastWeekday);
    const houses = this.street.townHouses();
    const by = this.townLotsBy;
    if (by.chunks === this.roam.chunks && by.houses === houses) return;
    this.townLotsBy = { chunks: this.roam.chunks, houses };
    const up = this.townUp;
    const land = this.lot.landscape;
    const ground = (x: number, z: number) => land?.heightAt(x, z) ?? 0;
    const parts = (this.townLotParts = new Map());
    this.townBuildings.setLots(
      houses
        .filter((h) => !this.street.chunkDrawn(h.chunk))
        .flatMap((h) => {
          const drawn = lotBuilding(h, ground, up.has(h.id));
          // (A lot with a look is lit by the hour, not by who is up.)
          if (!h.look) parts.set(h.id, drawn);
          return drawn;
        }),
    );
  }

  /**
   * Which far lots' houses have someone up in them (`townUp`, by lot id), twice a second while the town view is in
   * and the lamps are on: their windows glow at night only then (`lotBuilding`), as the neighbours' near the camera do
   * (`findDarkRooms`); the houses whose people got up or went to bed lit again in place (`lotUp`). Nothing but the
   * snapshot's positions is read.
   */
  private findTownUp(frame: FrameState): void {
    if (this.townBlend <= 0 || this.lampsLevel <= 0 || frame.now - this.townUpAt < 500) return;
    this.townUpAt = frame.now;
    const { curr, layout } = frame;
    const k = layout.sim;
    const sleeping = this.sleepActions?.ids;
    const houses = this.street.townHouses();
    const { boxes, grid } = this.gridOf(houses, (h) => h.box);
    const babies = this.babyStages();
    const sims = this.lastWorld?.sims ?? [];
    const up = new Set<number>();
    const count = curr[layout.header.simCount];
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      if (curr[o + k.away] !== 0 || babies.has(sims[i]?.stage ?? '')) continue;
      if (curr[o + k.pose] === Pose.Lie && sleeping?.has(curr[o + k.action] | 0)) continue;
      const px = curr[o + k.x];
      const pz = curr[o + k.z];
      // (Upstairs is a storey's own rows: back to the ground's.)
      const gz = pz - this.rowOf(this.storeyOf(Math.floor(pz)));
      const h = grid.find(boxes, px, gz);
      if (h >= 0) up.add(houses[h].id);
    }
    const was = this.townUp;
    let relit = false;
    let changed = false;
    for (const id of new Set([...was, ...up])) {
      if (was.has(id) === up.has(id)) continue;
      changed = true;
      const parts = this.townLotParts.get(id);
      if (!parts) continue;
      lotUp(parts, up.has(id));
      relit = true;
    }
    if (!changed) return;
    this.townUp = up;
    if (relit) this.townBuildings.relit();
    this.pictureChanged();
  }

  /** Boxes bucketed for the night's scans (`BoxGrid`), made again only for another list. */
  private gridOf<T>(list: readonly T[], box: (t: T) => Box): { boxes: readonly Box[]; grid: BoxGrid } {
    let g = this.grids.get(list);
    if (!g) {
      const boxes = list.map(box);
      this.grids.set(list, (g = { boxes, grid: new BoxGrid(boxes) }));
    }
    return g;
  }

  /**
   * The town view's movers (`townView.ts` `TownMovers`), each frame it is in: the cars and taxis the near drawing
   * leaves out (`Cars.forEachFar`), and the residents it doesn't draw who are out of doors (on a bike if they ride
   * one), where the snapshot has them; nothing else of the sim is read.
   */
  private updateTownMovers(frame: FrameState): void {
    const blend = this.townBlend;
    const world = this.builtWorld;
    if (blend <= 0 || !world) return this.townMovers.hide();
    const land = this.lot.landscape;
    const h = (x: number, z: number) => land?.heightAt(x, z) ?? 0;
    const list = this.townMoverList;
    let n = 0;
    const next = (): Mover => {
      const m = (list[n] ??= { kind: 'car', x: 0, y: 0, z: 0, yaw: 0, color: [1, 1, 1] });
      n++;
      return m;
    };
    const color = (hex: string) => {
      let c = this.moverColors.get(hex);
      if (!c) {
        const v = parseInt(hex.slice(1), 16);
        this.moverColors.set(hex, (c = [((v >> 16) & 255) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255]));
      }
      return c;
    };
    const cars0 = n;
    this.cars.forEachFar((x, z, yaw, paint, taxi) => {
      const m = next();
      [m.kind, m.x, m.y, m.z, m.yaw, m.color] = [taxi ? 'taxi' : 'car', x, h(x, z), z, yaw, color(paint)];
    });
    const cars1 = n;
    const { curr, layout } = frame;
    const count = Math.min(this.simCount + (layout.header.staffCount === undefined ? 0 : curr[layout.header.staffCount]), MAX_CHARACTERS);
    const at = this.moverAt;
    const depth = groundDepth(world);
    const sims = world.sims;
    const staff = world.staffSims ?? [];
    for (let i = 0; i < count; i++) {
      if (this.characters.head(i, at) || !this.characters.spot(i, at)) continue;
      const ix = Math.floor(at.x);
      const iz = Math.floor(at.z);
      // (Indoors, upstairs or in a car: not drawn.)
      if (iz >= depth || (ix >= 0 && iz >= 0 && ix < world.width && world.rooms[iz * world.width + ix])) continue;
      let inCar = false;
      for (let k = cars0; k < cars1 && !inCar; k++) inCar = Math.abs(list[k].x - at.x) < 2.6 && Math.abs(list[k].z - at.z) < 2.6;
      if (inCar) continue;
      const o = layout.headerLen + i * layout.simStride;
      const yaw = curr[o + layout.sim.yaw];
      const info = i < sims.length ? sims[i] : staff[i - sims.length];
      const m = next();
      [m.kind, m.x, m.y, m.z, m.yaw, m.color] = ['person', at.x, h(at.x, at.z), at.z, yaw, color(info?.appearance.body ?? '#B0B0B0')];
      if (layout.sim.ride !== undefined && curr[o + layout.sim.ride] >= 0) {
        const b = next();
        [b.kind, b.x, b.y, b.z, b.yaw, b.color] = ['bike', at.x, m.y, at.z, yaw, BIKE];
        m.y += 0.55;
      }
    }
    list.length = n;
    this.townMovers.set(list, 1 + MOVER_GROW * blend);
  }

  /**
   * The sun's shadow square: round the viewed lot while the camera looks at it (`buildWorld`); once
   * it roams off it, round the camera's target, snapped to 8 m (it moves in steps, drawn again
   * each), half a side of `clamp(0.9 · radius + 15, 30, 70)` m.
   */
  private followShadows(): void {
    const v = this.view;
    const t = this.orbit.target;
    if (!v) return;
    const onLot = (t.x >= v.x - 4 && t.z >= v.z - 4 && t.x <= v.x + v.w + 4 && t.z <= v.z + v.d + 4);
    if (onLot) {
      if (this.shadowSquare === 'lot') return;
      this.shadowSquare = 'lot';
      const r = Math.hypot(v.w, v.d) / 2 + SHADOW_MARGIN;
      this.sunShadows.setSquare(v.x + v.w / 2, v.z + v.d / 2, r, 0, this.reliefRound(v.x + v.w / 2, v.z + v.d / 2, r, 0));
      return;
    }
    // (Numbers compared, not a key made: this runs every frame.)
    const x = Math.round(t.x / 8) * 8;
    const z = Math.round(t.z / 8) * 8;
    const half = Math.min(70, Math.max(30, 0.9 * this.orbit.radius + 15));
    const step = Math.round(half / 8);
    const at = this.roamSquare;
    if (this.shadowSquare === 'roam' && at[0] === x && at[1] === z && at[2] === step) return;
    this.shadowSquare = 'roam';
    at[0] = x;
    at[1] = z;
    at[2] = step;
    // (On a land with relief, round the ground there, deep enough for the land's rise and fall in the square.)
    const y = this.targetGround(x, z);
    this.sunShadows.setSquare(x, z, half, y, this.reliefRound(x, z, half, y));
  }

  /** How far (m) the land in a square (`half` m round (`x`, `z`)) rises or falls from `y` (relative to the datum) at most: 0 without a height field. */
  private reliefRound(x: number, z: number, half: number, y: number): number {
    if (!this.lot.landscape?.heightField) return 0;
    let most = 0;
    for (let j = -4; j <= 4; j++) for (let i = -4; i <= 4; i++) most = Math.max(most, Math.abs(this.groundY(x + (i * half) / 4, z + (j * half) / 4) - y));
    return most;
  }
  /** The shadow square round the viewed lot (`lot`; `buildWorld` sets it there) or round the roaming camera's target (`roam`, `roamSquare`). */
  private shadowSquare: 'lot' | 'roam' = 'lot';
  /** The roaming square's middle (8 m steps) and half side (8 m steps), as last set. */
  private readonly roamSquare = new Float64Array(3);

  /**
   * The sim's full-detail area: the chunks streamed in, as a tile rectangle on the ground (`onWatch`); the residents
   * told it (`Residents.setDetailArea`: one it reaches is caught up by the sim, hidden a moment meanwhile).
   */
  private reportWatch(): void {
    const world = this.builtWorld;
    if (!world || !this.watchListener || !this.roam.chunks.size) return;
    let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
    for (const id of this.roam.chunks) {
      const [cx, cz] = chunkCoords(id);
      x0 = Math.min(x0, cx * CHUNK);
      z0 = Math.min(z0, cz * CHUNK);
      x1 = Math.max(x1, (cx + 1) * CHUNK);
      z1 = Math.max(z1, (cz + 1) * CHUNK);
    }
    // (At super speed, only the chunks round the camera's: `TIGHT_CHUNKS`.)
    const [cx, cz] = [Math.floor(this.orbit.target.x / CHUNK), Math.floor(this.orbit.target.z / CHUNK)];
    this.detailChunk = `${cx},${cz}`;
    if (this.detailTight) {
      x0 = Math.max(x0, (cx - TIGHT_CHUNKS) * CHUNK);
      z0 = Math.max(z0, (cz - TIGHT_CHUNKS) * CHUNK);
      x1 = Math.max(x0, Math.min(x1, (cx + TIGHT_CHUNKS + 1) * CHUNK));
      z1 = Math.max(z0, Math.min(z1, (cz + TIGHT_CHUNKS + 1) * CHUNK));
    }
    const d = groundDepth(world);
    const rect: [number, number, number, number] = [Math.max(0, x0), Math.max(0, z0), Math.min(world.width, x1), Math.min(d, z1)];
    this.watchListener(rect);
    // (Out of it the sim moves residents only now and then: one it reaches catches up in a jump, hidden meanwhile.)
    const [a, b, c, e] = [rect[0] - DETAIL_MARGIN, rect[1] - DETAIL_MARGIN, rect[2] + DETAIL_MARGIN, rect[3] + DETAIL_MARGIN];
    this.characters.setDetailArea((x, z) => x >= a && z >= b && x <= c && z <= e);
  }

  /**
   * What stands on the roads but is drawn apart from the street (the traffic lights, the cars driving and parked) drawn
   * in the chunks whose roads are drawn (`Street.shownChunks`; streaming: those the camera asks for come a build
   * later), round the camera's target (`x`, `z`).
   */
  private viewOnRoads(x: number, z: number): void {
    const chunks = this.streamOn ? this.street.shownChunks : null;
    this.onRoadsIn = chunks;
    this.cars.setView(chunks, x, z);
    this.parkedRides.setView(chunks, x, z);
    this.signals.setView(chunks, x, z);
  }
  /** The chunks `viewOnRoads` last gave. */
  private onRoadsIn: ReadonlySet<number> | null = null;

  /** The street builds the chunks streamed in (between world builds; a world build builds them itself). */
  private streamStreet(): void {
    if (this.worldBuilding || this.streaming) {
      this.streamDue = true;
      return;
    }
    const world = this.builtWorld;
    if (!world) return;
    this.streaming = (async () => {
      do {
        this.streamDue = false;
        this.streamSlice = performance.now();
        await this.street.build(world, this.view, () => this.streamPace());
        if (this.disposed) return;
        this.sunShadows.moved();
      } while (this.streamDue && !this.worldBuilding);
    })()
      .catch((err: unknown) => console.error('[render] streaming the street failed', err))
      .finally(() => {
        this.streaming = null;
      });
  }

  /** A stream's build gives the frame back every 4 ms of main-thread time (the camera is moving: a frame it takes shows). */
  private async streamPace(): Promise<void> {
    if (performance.now() - this.streamSlice < 4) return;
    await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
    this.streamSlice = performance.now();
  }

  setGameShot(shot: GameShot | null, cut = false): void {
    this.pictureChanged();
    if (!shot) return this.gameRig.release();
    this.followIndex = null;
    this.gameRig.setShot(shot, cut || this.townMode);
  }

  followSim(index: number | null): void {
    this.pictureChanged();
    if (index !== this.followIndex) this.followCaught = false;
    this.followIndex = index;
  }

  /**
   * Keeps the game camera above the land round the town (mountains rise close to it in some
   * worlds): looking from higher up instead of from inside a hill. The eye at least 4 m over the
   * land under it, the camera tilted up in steps of 0.03 rad.
   */
  private keepCameraClear(): void {
    const land = this.lot.landscape;
    if (this.townMode || !land) return;
    const o = this.orbit;
    for (let k = 0; k < 20 && o.beta > 0.3; k++) {
      // (The arc camera's eye: target + radius × (cos α sin β, cos β, sin α sin β), game space.)
      const s = Math.sin(o.beta) * o.radius;
      const x = o.target.x + Math.cos(o.alpha) * s;
      const z = o.target.z + Math.sin(o.alpha) * s;
      if (o.target.y + Math.cos(o.beta) * o.radius >= land.heightAt(x, z) - this.datum + 4) break;
      o.beta = Math.max(0.3, o.beta - 0.03);
      o.inertialBetaOffset = 0;
    }
  }

  /** The director's shot, or keeping a followed resident in the middle (game world only). */
  private steerCamera(dt: number): void {
    if (this.gameRig.active) return this.gameRig.frame(dt);
    const i = this.followIndex;
    // Off the drawn lot (on the street, or arriving where the next lot is still being built):
    // where they stand, so the camera keeps going with them.
    if (i === null || (!this.simHead(i, this.followHead) && !(i < this.simCount && this.characters.spot(i, this.followHead)))) return;
    const t = this.orbit.target;
    const k = 1 - Math.exp(-dt / 0.5);
    t.x += (this.followHead.x - t.x) * k;
    t.z += (this.followHead.z - t.z) * k;
    // (Never far behind them once caught up, however fast the game runs: at speed 4 the easing alone trails by 22 m.)
    this.followCaught = keepUp(t, this.followHead.x, this.followHead.z, this.followCaught);
  }

  setLeftDragCamera(on: boolean): void {
    this.orbit.buttons = on ? [0, 1, 2] : [1, 2];
  }

  cameraInfo(): { fov: number; radius: readonly [number, number]; beta: readonly [number, number] } {
    return { fov: LENS.fov, radius: this.orbit.limits.radius, beta: BETA_LIMITS };
  }

  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean {
    return this.camera.project(x, y, z, this.cssSize.width, this.cssSize.height, out);
  }

  // --- the world -----------------------------------------------------------------------

  setWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    this.pictureChanged();
    // (`?landFixture=…` on the dev server: a relief grid, a river with bridges or an island's sea for a Big world,
    // until worldgen writes them.)
    if (LAND_FIXTURE.size) applyLandFixture(world);
    return this.placeWorld(world, view);
  }

  private placeWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    // A big world is never built whole in full detail (no view: every house's storeys and every
    // thing, hundreds of MB of buffers; Low's merged object shadows alone 260 MB, which lost the
    // WebGPU device): with no lot given, the focus stays the lot built, else the one nearest the camera.
    if (!view && world.meta?.world?.size === 'big') view = this.view ?? this.nearestLot(world);
    // (Which traffic lights' poles get a pedestrians' head: those by a crosswalk.)
    this.signals.setWorld(world);
    // (Its roads changed under the same town: a road the land's water bridged, or the bridge taken back. Built again:
    // the street laid out on them, the landscape's surface solved with them, `buildWorld`; not patched.)
    const was = this.builtWorld?.meta?.roads;
    const now = world.meta?.roads;
    const decks = (roads: readonly RoadPlan[]) => roads.map((r) => (r.bridges ? r.bridges.join(';') : '')).join('|');
    const roadsChanged = !!was && !!now && now !== was && (now.length !== was.length || decks(now) !== decks(was));
    if (this.rebuildNext) this.rebuildNext = false;
    else if (roadsChanged) {
      // (Falls through to a whole build.)
    } else if (this.patchable(world, view)) {
      this.patchWorld(world);
      return Promise.resolve();
    } else if (!this.worldBuilds && !this.unpatchable(world, view, false)) {
      // Only the residents differ (one came or went, grew up, looks different): they are built again,
      // the rest patched. A big world built the whole lot, street, cars and birds again for it (a
      // town's staff coming on, a visitor): frames of 70-110 ms.
      this.lastWorld = world;
      this.nextWorld = { world, view, sims: true };
      return (this.worldBuilds = this.buildWorlds());
    }
    this.lastWorld = world;
    this.nextWorld = { world, view };
    return (this.worldBuilds ??= this.buildWorlds());
  }

  /** The lot nearest the camera's target (before the camera is placed: the main town's middle), as a view; null without lots. */
  private nearestLot(world: WorldStructure): ViewRect | null {
    const main = world.meta?.areas?.[0];
    const [x, z] = this.cameraPlaced ? [this.orbit.target.x, this.orbit.target.z] : main ? [main.x + main.w / 2, main.z + main.d / 2] : [world.width / 2, groundDepth(world) / 2];
    const p = plotNear(world, x, z, Infinity);
    return p && { x: p.x, z: p.z, w: p.w, d: p.d };
  }

  /**
   * Whether `world` differs from the built one only in ways what's on screen can take: no new
   * walls, floors, furniture or clothes, the same storey in view. (The simulation sends a world
   * message for every opened door and every growth stage, `structure_version` in
   * `crates/sim-core/src/world/mod.rs`.)
   */
  private patchable(world: WorldStructure, view: ViewRect | null): boolean {
    const why = this.unpatchable(world, view);
    if (!why) return true;
    if (this.debug) console.info('[render] world built:', why);
    return false;
  }

  /** What keeps `world` from being patched into what's on screen (null: nothing, it can be); `sims` false: its residents left out. */
  private unpatchable(world: WorldStructure, view: ViewRect | null, sims = true): string | null {
    const built = this.builtWorld;
    if (!built) return 'never built';
    if (this.worldBuilding) return 'building';
    if (this.townMode) return 'town';
    if (this.storey !== this.builtStorey) return `storey ${this.storey} built ${this.builtStorey}`;
    if (!sameView(view, this.view)) return 'view';
    // (A flat of the building in view opened or closed: its things, its residents and its lid, `setFlatsOpen`.)
    if (this.flatsKey() !== this.closedKey) return 'flats';
    if (built.width !== world.width || built.depth !== world.depth || built.storeys !== world.storeys) return 'size';
    if (built.storeyDepth !== world.storeyDepth || built.basement !== world.basement) return 'storeys';
    // The lot itself (walls, rooms, floors, fences) is what costs a build; a lean message never has it.
    if (built.lotVersion !== world.lotVersion) return 'lot';
    // (A garage built or taken down: the street draws it.)
    if (!sameGarages(built.garages, world.garages)) return 'garages';
    // (Nor a porch or a porch piece: the street draws them too.)
    if (!sameList(built.porches, world.porches) || !sameList(built.porchPieces, world.porchPieces)) return 'porches';
    if (built.objects.length !== world.objects.length) return 'object count';
    // (A thing that didn't change is the same object: `bridge.ts` `mergeWorld`.)
    for (let i = 0; i < world.objects.length; i++) {
      const [a, b] = [built.objects[i], world.objects[i]];
      if (a !== b && !sameObject(a, b)) return `object ${b.id} ${b.def}`;
    }
    if (!sims || built.sims === world.sims) return null;
    if (built.sims.length !== world.sims.length) return 'sim count';
    for (let i = 0; i < world.sims.length; i++) {
      const [a, b] = [built.sims[i], world.sims[i]];
      if (a.id !== b.id || a.gender !== b.gender || a.stage !== b.stage || a.name !== b.name) return `sim ${b.id}`;
      // (The session keeps a look that didn't change the same object: compared whole only when it isn't.)
      if (a.appearance !== b.appearance && JSON.stringify(a.appearance) !== JSON.stringify(b.appearance)) return `sim look ${b.id}`;
    }
    return null;
  }

  /** Applies a `patchable` world in place (what's drawn of an object, its open state and growth). */
  private patchWorld(world: WorldStructure): void {
    this.lot.patch(world, this.builtWorld); // (objects' clips and picking boxes.)
    this.lastWorld = world;
    this.builtWorld = world;
    if (this.debug) console.info('[render] world patched (no rebuild)');
  }

  private async buildWorlds(): Promise<void> {
    let error: unknown = null;
    try {
      for (let next = this.nextWorld; next; next = this.nextWorld) {
        this.nextWorld = null;
        try {
          await (next.sims ? this.buildSimsOnly(next.world) : this.buildOne(next.world, next.view));
          error = null;
        } catch (err) {
          // (Stopped for a newer lot: what it left half built is built again, the lot layer first.)
          if (err instanceof BuildSuperseded) {
            this.lotKey = '';
            error = null;
          } else error = err;
        }
      }
    } finally {
      this.worldBuilds = null;
    }
    if (error) throw error;
  }

  private async buildOne(world: WorldStructure, view: ViewRect | null): Promise<void> {
    // (First: a storey in view the world no longer has is changed below, for this build to show.)
    this.worldBuilding = true;
    this.sliceStart = performance.now();
    try {
      this.storeys = { count: world.storeys ?? 1, lowest: lowestStorey(world), layout: { depth: world.depth, storeyDepth: world.storeyDepth, storeys: world.storeys, basement: world.basement } };
      if (this.storey >= this.storeys.count || this.storey < this.storeys.lowest) this.setStorey(this.storey);
      await this.buildWorld(world, view);
      this.builtWorld = world;
      this.builtStorey = this.storey;
    } finally {
      this.worldBuilding = false;
      // (A stream change while it built: the street catches up.)
      if (this.streamDue) this.streamStreet();
    }
  }

  /** A world whose residents only differ from the one built (`setWorld`): the objects patched, the residents built again. */
  private async buildSimsOnly(world: WorldStructure): Promise<void> {
    this.worldBuilding = true;
    this.sliceStart = performance.now();
    try {
      this.lot.patch(world, this.builtWorld);
      await this.buildSims(world);
      this.builtWorld = world;
      if (this.debug) console.info('[render] world patched, residents built again');
    } finally {
      this.worldBuilding = false;
      if (this.streamDue) this.streamStreet();
    }
  }

  /** Everything the lot layer (roads and paths, house, fences) is built from. */
  private lotKeyOf(world: WorldStructure, view: ViewRect | null): string {
    const refs = [world.meta, world.walls, world.openings, world.diagonals, world.floors, world.fences];
    if (!this.lotParts || this.lotParts.refs.some((r, i) => r !== refs[i])) {
      const m = world.meta;
      this.lotParts = { refs, key: JSON.stringify([m?.seed, m?.streets, m?.roads, m?.paths, world.walls, world.openings, world.diagonals, world.floors, world.fences]) };
    }
    // (An older town's fences, planned from what stands round them, are part of the lot's too.)
    const planned = shownFences(world, this.deps.content.fenceStyles);
    return JSON.stringify([
      view,
      world.width,
      world.depth,
      this.lotParts.key,
      planned === (world.fences ?? planned) ? 0 : planned.length,
      world.plots.map((p) => [p.house, p.roof]),
      world.households.find((h) => h.player)?.plot,
      houseObjectKey(world, view, (def) => !!this.deps.content.object(def)?.stairs),
    ]);
  }

  private async buildWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    const started = performance.now();
    this.placing.worldBuilding(); // (the hand and the pops let go of the buffers this build replaces.)
    this.view = view;
    const { width, depth } = world;
    const [cx, cz] = [view ? view.x + view.w / 2 : width / 2, view ? view.z + view.d / 2 : depth / 2];
    // The sun's shadows cover a fixed square round the lot (`sunShadows.ts`).
    this.sunShadows.setSquare(cx, cz, Math.hypot(view?.w ?? width, view?.d ?? depth) / 2 + SHADOW_MARGIN, 0, this.reliefRound(cx, cz, Math.hypot(view?.w ?? width, view?.d ?? depth) / 2 + SHADOW_MARGIN, 0));
    this.shadowSquare = 'lot';
    if (!this.cameraPlaced) {
      this.aimAt(cx, cz);
      this.cameraPlaced = true;
    }
    // The camera roams the land (a big world's, or the town and 60 m round it); the street streams
    // its chunks round it in a big world, else draws the whole town as ever. The lot in view is the focus.
    const pb = (this.orbit.panBounds = panBounds(world));
    // (From the town view the target is held further inside them, the tilt in a bird's-eye range: `shared/townCamera.ts`.)
    const ring = landscapeRing(world);
    this.orbit.shape = townOrbit(width, groundDepth(world), -pb.x0, ring);
    Object.assign(this.worldReach, { x0: -ring, z0: -ring, x1: width + ring, z1: groundDepth(world) + ring });
    this.orbit.checkLimits();
    this.streamOn = world.meta?.world?.size === 'big' || STREAM_FLAG;
    // A big world's land reaches further: its far plane too (the preset's, `OpenWorldQuality.far`: its far LOD's
    // houses to 650 m, the hills beyond). (The play camera's: the town view moves it out from there, `applyTownCamera`.)
    const far = (this.playFar = world.meta?.world?.size === 'big' ? this.quality.openWorld.far : LENS.far);
    if (this.townBlendDrawn === 0 && this.camera.camera.farClip !== far) this.camera.setLens({ ...LENS, far });
    this.roam.setBounds({ x0: pb.x0 - 300, z0: pb.z0 - 300, x1: pb.x1 + 300, z1: pb.z1 + 300 });
    const viewIndex = view ? world.plots.findIndex((p) => p.x === view.x && p.z === view.z && p.w === view.w && p.d === view.d) : -1;
    this.roam.setFocus(viewIndex >= 0 ? viewIndex : null);
    // The flats of the building in view, and which of them (and of a place's ground floor under them) may not be shown
    // open (`setFlatsOpen`).
    this.viewFlats = viewedFlats(world, view);
    this.closedParts = this.partsClosed(world, view);
    this.closedKey = this.closedParts.map((p) => p.plot).join(',');
    if (this.streamOn) {
      const o = this.orbit;
      const aspect = this.cssSize.width / Math.max(1, this.cssSize.height);
      // (The chunks the camera will see once it's turned to frame the lot, `buildSims`: those it looks at now would
      // stream in again behind the turn, the first frames of play.)
      const alpha = this.framingAlpha(world) ?? o.alpha;
      this.roam.update({ x: o.target.x, z: o.target.z, radius: o.radius, alpha, fovH: 2 * Math.atan(Math.tan(LENS.fov / 2) * aspect) }, performance.now(), () => null, () => false);
      this.street.setChunks(this.roam.chunks);
    } else this.street.setChunks(null);
    this.viewOnRoads(this.orbit.target.x, this.orbit.target.z);
    // (lotContents.ts: the landscape, with its country roads.)
    this.loadStep('Laying out the land', 0.05);
    await this.lot.buildNature(world, (roads, parent) => drawRoads(this.app, roads, parent));

    // The datum: on a land the town follows, the viewed lot's pad (the world drawn that much lower, the lot at 0).
    const surface = this.lot.landscape?.surface ?? null;
    const datum = surface && viewIndex >= 0 ? surface.padY(viewIndex) : 0;
    this.applyDatum(datum);
    this.street.setLotY(datum);
    const lotKey = this.lotKeyOf(world, view);
    const lotBuilt = lotKey !== this.lotKey;
    const v = view;
    const onLot = (f: FenceEdge) => !!v && f.x >= v.x && f.z >= v.z && f.x + (f.axis === 'h' ? 1 : 0) <= v.x + v.w && f.z + (f.axis === 'v' ? 1 : 0) <= v.z + v.d;
    const shown = shownFences(world, this.deps.content.fenceStyles);
    const lotFences = lotBuilt ? shown.filter(onLot) : [];
    if (lotBuilt) {
      this.loadStep('Building the house', 0.2);
      this.clearLot();
      this.lotKey = lotKey;
      this.lotHedges = fenceHedges(lotFences, this.deps.content.fenceStyles).map(([x, z, s, , yaw]): [number, number, number, number] => [x, z, s, yaw]);
      this.buildStreets(world);
      await this.pace();
      await this.buildLot(world);
      await this.pace();
      // The viewed lot's fences stand on the ground (hidden with it while the basement is in view, its
      // hedges with the garden's shrubs: `lotHedges`); other lots' are the street's, but for their gates, which swing.
      const walls = wallCorners(world.walls);
      const wallAt = (x: number, z: number) => walls.has(`${x}:${z}`);
      // (The lot's on its own layer; the other lots' gates on the land, on their pads where the town follows it.)
      const pads = this.lot.landscape?.surface ?? null;
      for (const [list, storey] of [[lotFences, 0], [shown.filter((f) => !onLot(f) && f.kind === 'gate'), undefined]] as const) {
        const ours = storey === 0;
        const fences = buildFences(this.app, ours ? this.lotRoot : this.root, list, this.deps.content.fenceStyles, { wallAt, ...(!ours && pads ? { ground: (x: number, z: number) => pads.at(x, z) } : {}) });
        if (!fences) continue;
        if (!ours) this.townGates = fences;
        this.lotEntities.push({ entity: fences.entity, storey });
        for (const leaf of fences.gates) this.gates.push({ fences, leaf, open: 0, towards: 1 });
      }
    }
    this.placeRoomLights(world, view);
    await this.pace();
    this.loadStep('Building the street', 0.35);
    // (The game's world on the landscape the menus built for its preview, or a road the land's water bridged: the game's
    // landscape solves its surface again for this world, the one the street stands on too, and the land drawn again where
    // it moved.)
    const shaped = this.lot.landscape;
    const taken = shaped?.heightField ? shaped.takeRoads(world) : [];
    // (Solved there and then: what was placed on the surface before, the other lots' gates and maybe the traffic
    // lights' poles, stands on it again. The sun's shadows are taken again once everyone is home, `buildSims`.)
    if (taken.length) this.regroundPlaced();
    if (shaped && taken.length) void this.queueLand('land: roads taken', async () => {
      // (Behind the world's build and the warm-up, not in their way: on the way into a game, the land moved by a few
      // centimetres round its junctions.)
      const behind = async () => {
        await this.landPace();
        while ((this.worldBuilding || this.warming) && !this.disposed) await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
      };
      for (const rect of taken) await shaped.landDone(rect, behind, true);
    });
    await this.street.build(world, this.view, () => this.pace());
    // (On land the brushes shaped, the street stands on the landscape's surface, one surface for the land and what
    // stands on it: a street laid out again (roads bridged, a bridge undone or redone) stood on its own copy of the
    // world's land, and the build that took the roads may have been overtaken by the next.)
    if (shaped?.heightField && shaped.surface && !this.street.layout?.standsOn(shaped.surface) && this.street.landMoved(shaped.surface)) this.streamStreet();
    this.cars.build(world);
    this.buildTownView(world);
    this.loadStep('Furnishing the house', 0.45);
    // The objects, their clips, contact shadows and items; other lots' bikes and e-scooters alongside.
    await Promise.all([this.lot.buildObjects(world), this.parkedRides.build(world, (x, z) => this.inView(x, z, 0))]);
    this.mirrors.set(this.lot.mirrors);
    this.placing.objectsBuilt(); // (the thing in hand hidden in them.)
    this.loadStep('Bringing everyone home', 0.55);
    await this.buildSims(world);
    this.placeLids();
    this.applyWallMode();
    this.applyStoreys();
    const birds = this.settleBirds(world, view, this.fixedHour !== null ? this.fixedHour * 60 : this.lastMinute); // (where the birds live.)
    if (!this.birdsWaited && !this.warming) {
      // The game's first build waits for its birds a little, behind the loading screen (their models
      // built): they popped in in the first seconds of play. Later builds let them come when they come.
      this.birdsWaited = true;
      this.loadStep('Finding the birds', 0.6);
      await within(birds.catch((err: unknown) => console.warn('[render] the birds failed to settle', err)), BIRDS_WAIT_MS, undefined);
    } else void birds;
    this.lastLightMinute = -1;
    // A focus lot the roaming camera settled on, now drawn in full.
    const asked = this.focusAsked;
    const p = asked && world.plots[asked.plot];
    if (asked && p && view && p.x === view.x && p.z === view.z) {
      this.focusSwitches.push(performance.now() - asked.at);
      if (this.focusSwitches.length > 100) this.focusSwitches.shift();
      this.focusAsked = null;
    }
    if (this.debug) console.info(`[render] setWorld ${(performance.now() - started).toFixed(1)} ms${lotBuilt ? '' : ' (lot kept)'}`);
  }

  /**
   * The residents of `world`, carrying on as they were (a world message comes with every change in
   * it), pulling their chairs up to the tables the lot found. The first time a lot is shown, the
   * camera looks from its entry's side (the cutaway opens the front wall onto the household
   * arriving), unless it follows someone there.
   */
  private async buildSims(world: WorldStructure): Promise<void> {
    this.characters.setTables(this.lot.tables);
    // (The town's vendors and shopkeepers after the residents: `game/staff.ts`.)
    await this.characters.build(world.staffSims?.length ? { ...world, sims: [...world.sims, ...world.staffSims] } : world, 'residents');
    const v = this.view;
    const alpha = this.framingAlpha(world);
    if (v) this.framedView = `${v.x},${v.z},${v.w},${v.d}`;
    if (alpha !== null) this.orbit.alpha = alpha;
    const emotions = this.deps.content.emotions.map((e) => e.id);
    this.characters.setEmotions(emotions);
    this.marker.setEmotions(emotions);
    this.sunShadows.moved();
  }

  /**
   * The angle the camera turns to the first time the lot in view is shown (`buildSims`): from its
   * entry's side, the cutaway opening the front wall onto the household arriving. Null: it stays as
   * it is (the lot was framed already, the camera follows someone, the roaming camera settled on it,
   * or its entry is at its middle).
   */
  private framingAlpha(world: WorldStructure): number | null {
    const v = this.view;
    if (!v || `${v.x},${v.z},${v.w},${v.d}` === this.framedView || this.followIndex !== null) return null;
    // (Not a lot the roaming camera settled on: the camera stays as the player has it.)
    const roamed = this.roamedTo !== null && world.plots[this.roamedTo]?.x === v.x && world.plots[this.roamedTo]?.z === v.z;
    if (roamed) return null;
    const entry = world.plots.find((p) => p.x === v.x && p.z === v.z)?.entry;
    const dx = entry ? entry[0] - (v.x + v.w / 2) : 0;
    const dz = entry ? entry[1] - (v.z + v.d / 2) : 0;
    return Math.hypot(dx, dz) > 1 ? Math.atan2(dz, dx) - Math.PI * 0.12 : null;
  }

  /** Turns the camera to (`x`, `z`) on the ground from where it is: the eye stays, the radius and angles follow. */
  private aimAt(x: number, z: number): void {
    const o = this.orbit;
    const sb = Math.sin(o.beta);
    const eye = [o.target.x + o.radius * Math.cos(o.alpha) * sb, o.target.y + o.radius * Math.cos(o.beta), o.target.z + o.radius * Math.sin(o.alpha) * sb];
    const y = this.targetGround(x, z);
    const [dx, dy, dz] = [eye[0] - x, eye[1] - y, eye[2] - z];
    const r = Math.hypot(dx, dy, dz) || o.radius;
    o.target.x = x;
    o.target.y = y;
    o.target.z = z;
    o.radius = r;
    o.alpha = Math.atan2(dz, dx);
    o.beta = Math.acos(Math.max(-1, Math.min(1, dy / r)));
    o.checkLimits();
  }

  /** A world build (or the overview's) gives the frames it runs behind one every `BUILD_SLICE_MS` of main-thread time. */
  private async pace(): Promise<void> {
    // (Shorter slices once the camera roams: the frames go on moving behind the build, and one it takes shows.)
    // (Longer ones behind the loading screen: nothing is seen, and nothing roams.)
    const slice = this.loadingCover || this.warming ? COVERED_SLICE_MS : this.townBuilds === 0 && (this.streamOn || this.roamedTo !== null) ? ROAM_SLICE_MS : this.townBuilds > 0 && this.townAgain && !this.worldBuilding ? AGAIN_SLICE_MS : BUILD_SLICE_MS;
    if ((!this.worldBuilding && this.townBuilds === 0) || performance.now() - this.sliceStart < slice) return;
    await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
    this.sliceStart = performance.now();
    // A roaming camera has settled on another lot meanwhile: this lot's build stops here, the next one's starts.
    const next = this.nextWorld;
    if (this.worldBuilding && this.townBuilds === 0 && slice === ROAM_SLICE_MS && next && !sameView(next.view, this.view)) throw new BuildSuperseded();
  }

  /**
   * The viewed lot's garden paths, driveway and a farm's fields, each a flat mesh on its ground:
   * hidden with it while the basement is in view. (The road and the rest of the town's are the
   * street's, chunk by chunk: `pc/street.ts`.)
   */
  private buildStreets(world: WorldStructure): void {
    const paths = world.meta?.paths ?? [];
    const layers: [string, { x: number; z: number; w: number; d: number }[] | undefined, string, number][] = [
      ...pathLayers(paths, ''),
      ...fieldLayers(world.meta?.world?.fields, ''),
    ];
    const v = this.view;
    if (!v) return;
    const onLot = (r: { x: number; z: number; w: number; d: number }) => r.x >= v.x && r.z >= v.z && r.x + r.w <= v.x + v.w && r.z + r.d <= v.z + v.d;
    for (const [name, all, material, y] of layers) {
      const list = all?.filter(onLot) ?? [];
      if (!list.length) continue;
      const geo = new Geo();
      // Paving toned by vertex (darker and mossy at its sides: `shared/paving.ts`), as the street's.
      const toned = isPaths(name, '');
      if (toned) pavedRects(geo, list, paths, y);
      else for (const r of list) geo.poly([[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]], [0, 1, 0], [[0, 0], [1, 0], [1, 1], [0, 1]]);
      const mesh = meshFrom(this.device, geo.meshData({ colors: toned }));
      if (mesh) this.lotEntities.push({ entity: drawMesh(this.lotRoot, name, mesh, toned ? this.lib.surface(material, { doubleSided: true, vertexColors: true }) : this.lotMaterial(material)), storey: 0 });
    }
    // The town's paving on it (a place's court, a yard: `townPaving`), which the street leaves to it.
    for (const { name, material, geo } of pavingLayers(world, v, { tilesOnly: true })) {
      const mesh = meshFrom(this.device, geo.meshData({ colors: true }));
      if (mesh) this.lotEntities.push({ entity: drawMesh(this.lotRoot, name, mesh, this.lib.surface(material, { doubleSided: true, vertexColors: true })), storey: 0 });
    }
  }

  setFlatsOpen(mayOpen: ((plot: number) => boolean) | null): void {
    this.flatsOpen = mayOpen;
    this.askFlats();
  }

  /** Which flats of the building in view may be shown open, asked again: its things, residents and lids drawn again if that changed. */
  private askFlats(): void {
    if (!this.viewFlats.length || !this.lastWorld || this.worldBuilding) return;
    if (this.flatsKey() !== this.closedKey) void this.setWorld(this.lastWorld, this.view);
  }

  /** Which parts of the building in view may not be shown open now, as their plot ids (`closedKey` is the drawn one's). */
  private flatsKey(): string {
    const world = this.lastWorld;
    return world ? this.partsClosed(world, this.view).map((p) => p.plot).join(',') : '';
  }

  /** The parts of the building in view (`view`) in `world` that may not be shown open now (`closedParts`). */
  private partsClosed(world: WorldStructure, view: ViewRect | null): ClosedPart[] {
    if (!this.viewFlats.length) return [];
    return closedParts(world, viewedLot(world, view), this.viewFlats, this.flatsOpen, (k) => this.rowOf(k));
  }

  /**
   * A lid over each flat of the building in view that may not be shown open (`closedFlats`, `flatLids`), so from above
   * it is closed as a neighbour's house is; drawn only with its storey in view (`applyStoreys`).
   */
  private placeLids(): void {
    for (const l of this.lids) l.entity.destroy();
    this.lids = [];
    for (const [k, geo] of flatLids(this.closedParts, WALL_HEIGHT)) {
      const mesh = meshFrom(this.device, geo.meshData({ colors: true }));
      if (mesh) this.lids.push({ entity: drawMesh(this.lotRoot, `flatLids${k}`, mesh, this.lib.surface(HOUSE_MATERIALS.plaster, { vertexColors: true, doubleSided: true }), { castShadows: true }), storey: k });
    }
  }

  /** Whether a world point is inside the viewed lot (expanded by `margin`). */
  private inView(x: number, z: number, margin: number): boolean {
    const v = this.view;
    return !v || (x >= v.x - margin && z >= v.z - margin && x <= v.x + v.w + margin && z <= v.z + v.d + margin);
  }

  /** The viewed house on every storey (walls, trims, windows, floors, roofs), the basement's pit and the garden. */
  private async buildLot(world: WorldStructure): Promise<void> {
    const { count, lowest } = this.storeys;
    const W = world.width;
    const rooms = world.rooms;
    // Rows from storey `k` up to the one above it (not always `storeyDepth`: the basement's rows come last).
    const up = (k: number) => this.rowOf(k + 1) - this.rowOf(k);
    // Stairwells: the tiles above each flight of stairs are open to the storey below (the
    // ground's too, over stairs down to the basement).
    const holes = new Set<number>();
    const landings = new Set<number>();
    for (const o of world.objects) {
      const def = this.deps.content.object(o.def);
      if (!def?.stairs) continue;
      const lift = up(this.storeyOf(o.z));
      for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) holes.add((z + lift) * W + x);
      // The tile past the top step, on the storey above (as sim-core `storeys::stair_of`). A
      // spiral winds round its square footprint and steps off its back left corner, to the left.
      const far = o.w - 1;
      const off: [number, number] =
        def.stair === 'spiral'
          ? ([[-1, 0], [0, far + 1], [far + 1, far], [far, -1]] as [number, number][])[o.rot % 4]!
          : ([
              [Math.floor((o.w - 1) / 2), -1],
              [-1, Math.floor((o.d - 1) / 2)],
              [Math.floor((o.w - 1) / 2), o.d],
              [o.w, Math.floor((o.d - 1) / 2)],
            ] as [number, number][])[o.rot % 4]!;
      landings.add((o.z + off[1] + lift) * W + o.x + off[0]);
    }
    // The ground first (its garden dressing is the lot's), then upstairs, then the basement.
    const order = [...Array.from({ length: count }, (_, k) => k), ...(lowest < 0 ? [-1] : [])];
    let topped = false;
    let ground: { shrubs: [number, number, number, number][]; trees: [number, number, number, number][] } | null = null;
    const lotRooms = new Map<number, { n: number; sx: number; sz: number }>();
    const surfaces: HouseSurfaces[] = []; // (where the Paint and Floor tools lay their previews.)
    for (const k of order) {
      if (k > 0 && topped) continue;
      const shift = this.rowOf(k);
      const view = this.view ? { ...this.view, z: this.view.z + shift } : null;
      // Nothing up (or down) there yet: no storey to build.
      if (k !== 0 && view && !(world.walls ?? []).some((e) => e.z >= view.z && e.z <= view.z + view.d && e.x >= view.x && e.x <= view.x + view.w)) {
        topped ||= k > 0;
        continue;
      }
      // No roof where a room upstairs stands on this one (the ground covers the whole basement).
      const covered = (x: number, z: number) => k < 0 || (k + 1 < count && (rooms[(z + up(k)) * W + x] ?? 0) !== 0);
      if (k !== 0) await this.pace();
      // Lifted (or lowered) to its height, back over the ground rows.
      const storey = new pc.Entity(`storey${k}`);
      storey.setLocalPosition(0, k * WALL_HEIGHT, -shift);
      this.lotRoot.addChild(storey);
      const built = this.house.build(world, view, k === 0 ? { covered, holes, landings } : { upper: true, basement: k < 0, covered, holes, landings }, storey);
      if (!built) {
        storey.destroy();
        if (k === 0) break;
        continue;
      }
      this.lotEntities.push({ entity: storey, storey: k });
      this.roofs.push(...built.roofs.map((m) => m.entity));
      for (const [r, t] of built.rooms) lotRooms.set(r, t);
      for (const leaf of built.doors) this.doors.push({ leaf, open: -1 });
      surfaces.push(built.surfaces);
      ground ??= { shrubs: built.shrubs, trees: built.trees };
    }
    // Stairs down: from the ground, a shaft round each flight (the rest of the basement stays hidden).
    this.stairwells = [];
    for (const o of world.objects) {
      if (!this.deps.content.object(o.def)?.stairs || this.storeyOf(o.z) !== -1 || !this.inView(o.x + o.w / 2, o.z - this.rowOf(-1) + o.d / 2, 0)) continue;
      this.stairwells.push({ x: o.x, z: o.z - this.rowOf(-1), w: o.w, d: o.d });
    }
    if (lowest < 0 && this.view) this.buildPit(this.view);
    for (const r of this.stairwells) this.buildPit(r, 0, 0);
    if (!ground) {
      this.doors = [];
      this.lotRooms = null;
      this.previews.setSurfaces(null);
      // Unknown wall layout: the simulation's plain meshes.
      const add = (name: string, data: WorldStructure['meshes']['walls'], material: pc.StandardMaterial) => {
        const mesh = meshFrom(this.device, data);
        if (!mesh) return null;
        const entity = drawMesh(this.lotRoot, name, mesh, material, { castShadows: true });
        this.lotEntities.push({ entity });
        return entity;
      };
      add('floors', world.meshes.floors, this.lotMaterial('material.floor'));
      const wall = this.lotMaterial('material.wall');
      this.walls = add('walls', world.meshes.walls, wall);
      this.wallsLow = add('wallsLow', world.meshes.wallsLow, wall);
      return;
    }
    // Garden dressing split into species (hedges or mixed shrub borders, varied lot trees), and the lot's hedges round its garden (its fences').
    const decor = [...natureDecor('model.bush', ground.shrubs), ...natureDecor('model.tree', ground.trees, world, this.view)];
    if (this.lotHedges.length) {
      const hedges = decor.find(([k]) => k === 'model.hedge');
      if (hedges) hedges[1] = [...hedges[1], ...this.lotHedges];
      else decor.push(['model.hedge', this.lotHedges]);
    }
    this.birdLife.setDecor(decor); // (the birds live in it.)
    for (const [key, items] of decor) {
      await this.pace();
      void this.placeDecor(key, items);
    }
    this.lotRooms = lotRooms;
    this.previews.setSurfaces(mergeSurfaces(surfaces));
  }

  /** Garden dressing as instances of a cached template (`deco:` keys): a draw per mesh, kept (hidden) for the next lot. */
  private async placeDecor(key: string, items: [number, number, number, number][]): Promise<void> {
    if (!items.length) return;
    const lot = this.lotKey;
    const template = await this.decorTemplates.get(`deco:${key}`, () => buildModel(this.app, key, this.deps.assets.get(key, 'model'), [1, 1]));
    if (this.disposed || lot !== this.lotKey) return;
    let placed = this.decor.get(key);
    if (placed && placed.template !== template) {
      for (const d of placed.draws) d.destroy();
      placed = undefined;
    }
    if (!placed) {
      placed = { template, draws: [] };
      for (const m of template.meshes) {
        const d = new Instances(this.device, this.lotRoot, m.mesh, m.material, { name: m.name, castShadows: key !== PETALS, receiveShadows: true });
        windStill(d.meshInstance, m);
        placed.draws.push(d);
      }
      this.decor.set(key, placed);
    }
    const m = this.decorScale;
    for (const d of placed.draws) {
      d.setCount(items.length);
      items.forEach(([x, z, s, yaw], i) => d.setMatrix(i, compose(s, s, s, quatYawPitchRoll(yaw, 0, 0, this.decorQ), x, 0, z, m)));
      d.update();
      // The garden goes with the ground dug away over the basement.
      d.enabled = this.storey >= 0;
    }
    // (In place of a box set aside for a file that came, if there was one.)
    this.decorTemplates.hideStale(`deco:${key}`);
    this.sunShadows.moved();
  }

  /**
   * The dug-out lot under the ground, seen while its basement is in view (the landscape over it
   * is cut away then, see `applyDig`): earth walls round the plot and a floor below the basement's.
   */
  private buildPit(view: ViewRect, margin = PIT_MARGIN, storey = -1): void {
    const [x0, z0, x1, z1] = [view.x - margin, view.z - margin, view.x + view.w + margin, view.z + view.d + margin];
    const [y0, y1] = [-WALL_HEIGHT - 0.03, 0];
    // (The soil's own colour on the floor, the walls a shade darker: the vertex colours tint it.)
    const geo = new Geo().color('#FFFFFF');
    geo.poly([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0, 1, 0]);
    geo.color('#B4A898');
    geo.poly([[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]], [0, 0, 1]);
    geo.poly([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0, 0, -1]);
    geo.poly([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [1, 0, 0]);
    geo.poly([[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]], [-1, 0, 0]);
    const mesh = meshFrom(this.device, geo.meshData({ colors: true }));
    if (!mesh) return;
    const entity = drawMesh(this.lotRoot, 'basementPit', mesh, this.lib.surface('material.soil', { vertexColors: true, doubleSided: true }), { castShadows: true });
    this.lotEntities.push({ entity, storey });
  }

  /** In the basement, the ground over the viewed lot is dug away (terrain and lawn; see `DigState`). */
  private applyDig(): void {
    this.applyDimming();
    const dig = this.lib.dig;
    const v = this.view;
    // From the ground up, only over the stairs down (the floors round them cover the rest).
    const wells = this.storey < 0 ? [] : this.stairwells;
    const on = !this.townMode && !!v && (this.storey < 0 || wells.length > 0);
    if (wells.length) {
      dig.rect.set(Math.min(...wells.map((r) => r.x)), Math.min(...wells.map((r) => r.z)), Math.max(...wells.map((r) => r.x + r.w)), Math.max(...wells.map((r) => r.z + r.d)));
    } else if (v) dig.rect.set(v.x - PIT_MARGIN, v.z - PIT_MARGIN, v.x + v.w + PIT_MARGIN, v.z + v.d + PIT_MARGIN);
    // The basement itself in view: no daylight down there.
    dig.on = on;
    dig.under = on && this.storey < 0;
    this.applyUnderLight();
  }

  /**
   * The share of daylight under the ground and in rooms without windows (`DigState.lit`), at the
   * last lighting update: the basement's while it is in view (its fill), else the windowless
   * rooms' (the only ones that read it then: `darkTiles` marks none from the basement). With the
   * basement in view the moonlight grade is softened too (`BASEMENT_NIGHT_GRADE`): no moon down there; and the bloom
   * is full at any hour (`PostProcessing.glow`): no daylight down there either.
   */
  private applyUnderLight(): void {
    const l = this.lighting;
    if (this.post) {
      const basement = this.storey < 0 && !this.townMode;
      this.post.night = l.night * LOOK.nightGrade * (basement ? BASEMENT_NIGHT_GRADE : 1);
      this.post.glow = basement ? 1 : l.night;
    }
    const under = this.storey < 0;
    const [sky, env, most] = under ? [BASEMENT_SKY * (NOON_SKY_LUMA / skyLuma(l)), BASEMENT_ENV, BASEMENT_NIGHT_GAIN] : [WINDOWLESS_SKY, WINDOWLESS_ENV, 1];
    const seen = Math.max(l.exposure, 1e-3);
    this.lib.dig.lit.set(0, Math.min(most, sky / Math.max(l.skyIntensity * seen, 1e-3)), Math.min(most, env / Math.max(l.envIntensity * seen, 1e-3)), 0);
  }

  /**
   * The lights of the viewed house on the storeys shown: the pools of lamplight under the garden's
   * lamps (`gardenPools.ts`), the rooms without windows kept out of daylight, and the interior
   * lights with the rooms they keep to (`lamps.ts`).
   */
  private placeRoomLights(world: WorldStructure, view: ViewRect | null): void {
    // Rooms and lamps on every storey shown (in lot rows: lifted to their height here).
    const shown = (z: number) => storeyShown(this.storeyOf(z), this.storey);
    // The viewed lot on every storey at once: each room's lamps (or fill) are placed once.
    const inView = (x: number, z: number) => {
      const k = this.storeyOf(z);
      const row = z - this.rowOf(k);
      return (!view || (x >= view.x && row >= view.z && x < view.x + view.w && row < view.z + view.d)) && !inClosed(this.closedParts, x + 0.5, row + 0.5, k);
    };
    this.markWindowless(world, view);
    const lights: RoomLight[] = this.lotRooms ? placeLights(world, inView, this.lotRooms, (def) => this.deps.content.object(def)?.light).filter((l) => shown(l.z)) : [];
    // (Under the garden's lamps, and outside the windows of the house's rooms: lit as the rooms are.)
    const ground = (z: number) => this.storeyOf(z) === 0;
    const underLamps = lights.filter((l) => l.kind === 'lamp' && l.room === 0 && ground(l.z)).map((l) => gardenPool(l, this.rowOf(0)));
    // (Curtains over the windows, by the edges they hang over: drawn, they keep the light in.)
    const curtains = new Map<string, number>();
    for (const o of world.objects) if (ground(o.z) && inView(o.x, o.z) && this.deps.content.object(o.def)?.overWindow) for (const e of backEdges(o)) curtains.set(e, o.id);
    this.gardenPools?.set(this.storey >= 0 ? [...underLamps, ...windowPools(world, ground, this.rowOf(0), inView, (e) => curtains.get(e))] : underLamps);
    this.applyGardenPools();
    // (the interior lights and the rooms they keep to, `lamps.ts`.)
    this.lamps.markRooms(world, view);
    this.lamps.place(lights);
  }

  /**
   * Finds the rooms without windows (`windowless`), and keeps those of the storey in view out of
   * daylight as the basement is: every one of their tiles in view, on the materials' mask
   * (`DigState.darken`). (Their lamps lit by day too: `lamps.ts`.)
   */
  private markWindowless(world: WorldStructure, view: ViewRect | null): void {
    this.windowless = windowlessRooms(world, this.windowless);
    const dark = darkTiles(world, view, this.storey, this.rowOf(this.storey), this.windowless.rooms);
    this.dimming = dark.dimming;
    this.lamps.setDimmed(dark.dimmed); // (their lights on by day, `lamps.ts`.)
    this.applyDimming();
  }

  /** The windowless rooms' darkness on the materials (`dimming`): none while the menus' town is shown. */
  private applyDimming(): void {
    this.lib.dig.darken(this.townMode ? null : this.dimming);
  }

  /** How far the garden's pools of lamplight are up (0..1): from dusk as the lamps come up, none in town. */
  private get glowLevel(): number {
    return this.townMode ? 0 : Math.min(1, Math.max(0, (this.lampsLevel - 0.35) / 0.5));
  }

  /** The garden's pools of lamplight, as bright as its lamps are now; whether one changed. */
  private applyGardenPools(): boolean {
    return this.gardenPools?.update(this.glowLevel, this.poolOn, this.lampColor) ?? false;
  }

  /** Model files that failed have come (gathered by `modelRefresh`): the world draws again what it drew without them (once no build runs). */
  private refreshStreet(urls: readonly string[]): void {
    this.refreshTown(urls);
    let drawn = false;
    for (const url of urls) {
      for (const [, p] of this.decor) if (p.template.failed?.includes(url)) this.lotKey = '';
      this.decorTemplates.invalidate(url);
      // (lotContents.ts: the objects drawn without it with the world.)
      const objects = this.lot.invalidate(url);
      if (this.street.invalidate(url) || objects) drawn = true;
    }
    if (!drawn && this.lotKey !== '') return;
    void (async () => {
      while (this.worldBuilds) await this.worldBuilds.catch(() => {});
      if (this.disposed || !this.lastWorld || this.townMode) return;
      this.rebuildNext = true;
      await this.setWorld(this.lastWorld, this.view).catch(() => {});
      this.decorTemplates.disposeStale();
      this.lot.disposeStale();
    })();
  }

  /** The storey a lot row is on (-1: the basement). */
  private storeyOf(z: number): number {
    return Math.max(this.storeys.lowest, Math.min(this.storeys.count - 1, storeyOfRow(this.storeys.layout, z)));
  }

  /** The first lot row of storey `k`. */
  private rowOf(k: number): number {
    return storeyRow(this.storeys.layout, k);
  }

  clear(): void {
    this.pictureChanged();
    this.characters.clear();
    this.birdLife.clear();
    this.simShadowMatrices.fill(0);
    this.simShadow.setCount(0);
    this.simShadow.update(false);
    this.simCount = 0;
    this.selectedSim = null;
    this.lastFrame = null;
    this.marker.place(null, 0, this.camera.entity, 0, true);
    this.hoverTile.set(null);
    this.previews.clear();
    this.framedView = '';
    this.placing.clear();
    this.lot.clear();
    this.mirrors.set([]);
    this.clearLot();
    this.lotParts = null;
    this.street.clear();
    this.cars.clear();
    this.parkedRides.clear();
    this.signals.clear();
    this.windowless = { rooms: new Set(), of: [] };
    this.dimming = null;
    this.applyDimming();
    this.gardenPools?.set([]);
    this.lamps.clear();
    this.lastWorld = this.builtWorld = null;
    this.cameraPlaced = false;
    this.birdsWaited = false;
    this.parseQueue = [];
    this.movers.length = 0;
    this.darkRooms.clear();
    this.awakeRooms.clear();
    this.occupiedRooms.clear();
    // The ended game's callbacks: kept, its frame would go on running and its session stay alive behind the menus.
    this.onFrame = this.onView = null;
    this.flatsOpen = null;
  }

  /** Takes down the lot layer (it's built again for the next lot). */
  private clearLot(): void {
    for (const { entity } of this.lotEntities) entity.destroy();
    this.lotEntities = [];
    for (const l of this.lids) l.entity.destroy();
    this.lids = [];
    this.lotKey = '';
    this.roofs = [];
    this.doors = [];
    this.gates = [];
    this.townGates = null;
    this.stairwells = [];
    this.lotRooms = null;
    this.walls = this.wallsLow = null;
    // Every kept garden draw emptied: the next lot places only the kinds it has (`placeDecor`), and `applyStoreys` shows a draw with copies.
    for (const p of this.decor.values())
      for (const d of p.draws) {
        d.setCount(0);
        d.update();
        d.enabled = false;
      }
  }

  private lotMaterial(key: string): pc.StandardMaterial {
    // Generated geometry has explicit normals; don't depend on triangle winding.
    return this.lib.surface(key, { doubleSided: true });
  }

  /** The walls as drawn: up while something for a wall is in hand and from the town view, else as the player has them. */
  private get shownWallMode(): WallMode {
    return this.wallsRaised || this.townWhole ? 'up' : this.wallMode;
  }

  /** Something for a wall in hand (`on`) or not: the walls stand meanwhile, so what hangs on them hangs on them. */
  private raiseWalls(on: boolean): void {
    if (on === this.wallsRaised) return;
    this.wallsRaised = on;
    this.applyWallMode();
  }

  /**
   * Walls up, cut away or down (as shown: `shownWallMode`): the cutaway state the wall materials
   * read, the roofs (the player's walls up only), and the fallback's full or low walls.
   */
  private applyWallMode(): void {
    const mode = this.shownWallMode;
    if (this.walls) this.walls.enabled = mode === 'up';
    if (this.wallsLow) this.wallsLow.enabled = mode !== 'up';
    this.roofShown = this.roofVisible();
    this.showRoofs();
    this.lib.wallCut.params.x = mode === 'down' ? 1 : 0;
    this.lib.wallCut.params.y = WALL_STUB;
    this.cutLook = -1;
    this.updateCutaway();
    this.placeLidHeights();
  }

  /**
   * The closed parts' lids (`placeLids`) as high as the walls round them stand. With the walls up, one on the storey in
   * view at its walls' top, and one on a storey under it just under the floor over it (it shows only where that floor
   * has a gap: a roof's strip beside a flat over a wider place). With the walls cut away or down, each at the cut walls'
   * stub (every storey's walls are cut alike), so no side shows under it.
   */
  private placeLidHeights(): void {
    const upTo = this.townWhole && this.storey >= 0 ? this.storeys.count - 1 : this.storey;
    const low = this.shownWallMode !== 'up';
    for (const l of this.lids) {
      l.entity.enabled = l.storey <= upTo;
      l.entity.setLocalPosition(0, low ? WALL_STUB - WALL_HEIGHT : l.storey === upTo ? 0 : -LID_THICK - LID_UNDER, 0);
    }
  }

  /**
   * The viewed lot's roof: only with walls up, and not when zoomed in (it would fill the view), unless a check keeps
   * it (`keepRoof`). From the town view (`townWhole`) it is on whatever the walls' mode.
   */
  private roofVisible(): boolean {
    return (this.wallMode === 'up' || this.townWhole) && (this.roofKept || this.orbit.radius > ROOF_MIN_DISTANCE);
  }

  keepRoof(on: boolean): void {
    this.pictureChanged();
    this.roofKept = on;
  }

  /**
   * Roofs show with walls up (over the storeys in view: their storey's entity is). The house casts
   * in every mode, its walls at full height (the shadow pass has no cutaway) and its shadow-only
   * ceiling over the rooms: the sun comes in through windows and doorways.
   */
  private showRoofs(): void {
    for (const roof of this.roofs) roof.enabled = this.roofShown;
    this.sunShadows.moved();
  }

  /**
   * Storeys above the one in view are hidden (walls, floors, roofs), and the lot's ground while the
   * basement is: its garden, paths and fences, its garage (the street's, `Street.showLot`) and a
   * car standing on it (`Cars.hideOn`).
   */
  private applyStoreys(): void {
    // (From the town view the house stands whole, `townWhole`: every storey from the ground up, whichever is in view.)
    const upTo = this.townWhole && this.storey >= 0 ? this.storeys.count - 1 : this.storey;
    for (const { entity, storey } of this.lotEntities) if (storey !== undefined) entity.enabled = storeyShown(storey, upTo);
    this.placeLidHeights();
    for (const p of this.decor.values()) for (const d of p.draws) d.enabled = this.storey >= 0 && d.count > 0;
    const v = this.view;
    this.street.showLot(this.storey >= 0);
    const pit = this.storey < 0 && v ? ([v.x - PIT_MARGIN, v.z - PIT_MARGIN, v.x + v.w + PIT_MARGIN, v.z + v.d + PIT_MARGIN] as const) : null;
    this.cars.hideOn(pit);
    this.parkedRides.hideOn(pit);
    this.applyDig();
    this.sunShadows.moved();
  }

  /** Cutaway: walls facing the camera drop when the view direction crosses a quadrant. */
  private updateCutaway(): void {
    let bits = 0;
    if (this.shownWallMode === 'cutaway') {
      // Camera looks from its position towards the target: direction -(cos a, sin a) in xz.
      const dx = -Math.cos(this.orbit.alpha);
      const dz = -Math.sin(this.orbit.alpha);
      if (dx > 0.3) bits |= 1;
      if (dx < -0.3) bits |= 2;
      if (dz > 0.3) bits |= 4;
      if (dz < -0.3) bits |= 8;
    }
    if (bits === this.cutLook) return;
    this.cutLook = bits;
    this.placing.cutWallItems(bits, this.shownWallMode === 'down'); // (things on the walls go with them, but the one in hand.)
    this.street.cutLot(bits, this.shownWallMode === 'down'); // (and the porch's lamps.)
    this.lib.wallCut.look.set(bits & 1 ? 1 : 0, bits & 2 ? 1 : 0, bits & 4 ? 1 : 0, bits & 8 ? 1 : 0);
  }

  /**
   * Where residents are this frame (`darkRooms`, `awakeRooms`, `occupiedRooms`, `movers`; away
   * residents not counted), and which neighbours' houses have someone up in them (`neighboursUp`, every
   * `NEIGHBOURS_EVERY_MS`).
   */
  private findDarkRooms(frame: FrameState): void {
    const { curr, layout } = frame;
    const dark = this.darkRooms;
    const awake = this.awakeRooms;
    const occupied = this.occupiedRooms;
    const movers = this.movers;
    dark.clear();
    awake.clear();
    occupied.clear();
    movers.length = 0;
    const world = this.lastWorld;
    if (!world) return;
    if (this.sleepActions?.tags !== layout.actions) {
      const ids = new Set<number>();
      layout.actions.forEach((tag, i) => (tag === 'sleep' || tag === 'nap') && ids.add(i));
      this.sleepActions = { tags: layout.actions, ids };
    }
    const sleeping = this.sleepActions.ids;
    const k = layout.sim;
    const W = world.width;
    // Neighbours' houses only matter once their windows can glow; who is up in them is looked for a few times a second
    // (the street draws their windows again at most once a second, `Street.setUp`).
    const all = this.townMode || this.lampsLevel <= 0 ? NO_HOUSES : this.street.houses;
    const houses = all.length && frame.now - this.neighboursAt >= NEIGHBOURS_EVERY_MS ? all : NO_HOUSES;
    const up = this.neighboursUp;
    const { boxes, grid } = this.gridOf(houses, (b) => b);
    if (houses.length) {
      this.neighboursAt = frame.now;
      up.length = houses.length;
      up.fill(false);
    }
    const babies = this.babyStages();
    const count = curr[layout.header.simCount];
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      if (curr[o + k.away] !== 0) continue;
      const px = curr[o + k.x];
      const pz = curr[o + k.z];
      if (curr[o + k.moving] !== 0) {
        // (An older layout without the way ahead: where they are, so only a doorway they're in opens.)
        const ax = k.aheadX;
        const az = k.aheadZ;
        if (ax === undefined || az === undefined) movers.push(px, pz, px, pz, i);
        else movers.push(px, pz, curr[o + ax], curr[o + az], i);
      }
      const asleep = curr[o + k.pose] === Pose.Lie && sleeping.has(curr[o + k.action] | 0);
      const baby = babies.has(world.sims[i]?.stage ?? '');
      // `z` is in lot rows: upper storeys have rooms of their own there.
      const x = Math.floor(px);
      const z = Math.floor(pz);
      const room = x >= 0 && z >= 0 && x < W && z < world.depth ? world.rooms[z * W + x] : 0;
      if (room) {
        occupied.add(room);
        if (asleep) dark.add(room);
        else if (!baby) awake.add(room);
      }
      if (asleep || baby || !houses.length) continue;
      // Upstairs is a storey's own rows: back to the ground's.
      const gz = pz - this.rowOf(this.storeyOf(z));
      const h = grid.find(boxes, px, gz);
      if (h >= 0) up[h] = true;
    }
    for (const room of awake) dark.delete(room);
    if (houses.length) this.street.setUp(up);
  }

  /** A thing of the lot that casts changed (`LotContents`): the sun's shadows and the lamps'. */
  private castersMoved(urgent: boolean): void {
    this.sunShadows.moved(urgent);
    // (An urgent change reaches the lamps through the sun's `onMoved`; a moving part only here.)
    if (!urgent) this.lamps.castersMoved(false);
  }

  /** A use moved the lot's moving parts (`LotContents.update`): the sun's shadows as for a pose, and the lamps'. */
  private partsMoved(): void {
    this.sunShadows.posed();
    this.lamps.castersMoved(false);
  }

  /** What casts in a lamp's light now (`lampShadows.ts`): the lot's things and the residents. One list, filled anew. */
  private lampCasters(): pc.MeshInstance[] {
    const out = this.lampCasterList;
    out.length = 0;
    this.lot.lampCasters(out, this.lampWarm);
    this.characters.shadowCasters(out);
    return out;
  }
  private readonly lampCasterList: pc.MeshInstance[] = [];
  /** The warm-up's frame is being drawn (`warmFrame`): everything that can cast in a lamp's light does. */
  private lampWarm = false;

  /** Life stage ids of babies (content `baby`: they don't count as someone up, as in sim-core). */
  private babyStages(): ReadonlySet<string> {
    const stages = this.deps.content.lifeStages;
    if (this.babies?.of !== stages) this.babies = { of: stages, ids: new Set(stages.filter((s) => s.baby).map((s) => s.id)) };
    return this.babies.ids;
  }

  /**
   * Doors (every one, inside and out) and gates stand shut; each opens only for someone going
   * through it (`goingThrough`: not for someone walking past, nor someone waiting outside, as at a
   * bathroom someone's in), and swings back once they're through. It opens at the hand: the leaf starts when the walker's hand
   * could be on its handle (`doorLead`: a hand's reach, or as far ahead as the swing needs at this
   * game speed, so it's open by the time they are), and the resident's arm goes out with it
   * (`doorHand`, `residents.ts`). `rate`: how fast the game runs (`playbackRate`).
   */
  private swingDoors(rate: number): void {
    const hand = this.doorHand;
    hand.fill(0);
    if (!this.doors.length && !this.gates.length) return;
    const movers = this.movers;
    const lead = doorLead(rate);
    const step = Math.min(1, (this.frameDt / 1000 / DOOR_SWING) * Math.min(Math.max(rate, 1), 10));
    let moved = false;
    let urgent = false;
    let swung = false;
    // (Only a leaf where it casts into the sun's map draws the map again: one across town moves the picture alone.)
    const casts = (c: readonly [number, number]) => this.sunShadows.reaches(c[0], c[1] - this.rowOf(this.storeyOf(Math.floor(c[1]))));
    // (A town has a gate in most gardens, a house a door or more a room: one standing shut with
    // nobody within a cell of it isn't asked. The cells as wide as the lead reaches at this speed, at super speed tens of metres.)
    const size = doorCell(lead);
    const near = this.moverCells;
    moverCells(movers, size, near);
    for (const gate of this.gates) {
      if (gate.open === 0 && !anyoneNear(near, size, gate.leaf.centre[0], gate.leaf.centre[1])) continue;
      const way = goingThrough(movers, gate.leaf.axis, gate.leaf.centre, gate.open === 1, lead, hand);
      // It opens away from the one coming through, and shuts the way it opened.
      if (way && gate.open === 0) gate.towards = way;
      const target = way ? 1 : 0;
      const open = target > gate.open ? Math.min(target, gate.open + step) : Math.max(target, gate.open - step);
      if (open === gate.open) continue;
      this.doorHeard(gate.open, open, gate.leaf.centre, true);
      gate.open = open;
      swingGate(gate.fences, gate.leaf, smooth(open), gate.towards);
      swung = true;
      if (casts(gate.leaf.centre)) moved = urgent = true;
    }
    for (const door of this.doors) {
      const { leaf } = door;
      // (Both in lot rows: the storey is in the row.)
      if (door.open === 0 && !anyoneNear(near, size, leaf.centre[0], leaf.centre[1])) continue;
      const target = goingThrough(movers, leaf.axis, leaf.centre, door.open === 1, lead, hand) !== 0 ? 1 : 0;
      const open = door.open < 0 ? target : target > door.open ? Math.min(target, door.open + step) : Math.max(target, door.open - step);
      if (open === door.open) continue;
      // (One first placed, `open` -1, is heard from its next move.)
      this.doorHeard(door.open, open, leaf.centre, false);
      door.open = open;
      // Eased: quick off the frame, slowing as it meets the jamb or the wall. One that slides stays
      // turned shut and slides into the wall (from where it was built).
      const k = smooth(open);
      const { slide, base } = leaf;
      if (slide) leaf.pivot.setLocalPosition(base.x + slide[0] * k, base.y, base.z + slide[1] * k);
      leaf.pivot.setLocalEulerAngles(0, ((slide ? leaf.close : leaf.close * (1 - leaf.opens * k)) * 180) / Math.PI, 0);
      swung = true;
      if (!casts(leaf.centre)) continue;
      moved = true;
      // (A door indoors, where the sun reaches only through the windows, redraws the shadows as a
      // resident's pose does: held to the preset's `shadowRate`.)
      urgent ||= leaf.front;
    }
    // (Doors and gates cast only the sun's shadow. Any that swung shows in the mirrors' held reflections, whether it casts or not.)
    if (swung) this.mirrors.changed();
    if (moved) this.sunShadows.moved(urgent, false);
    else if (swung) this.pictureChanged();
  }

  /** A leaf moved from `was` to `open` (0 shut): one leaving its frame or coming back to it is kept for the world's sound (`doorSounds`). */
  private doorHeard(was: number, open: number, centre: readonly [number, number], gate: boolean): void {
    const opened = was === 0 && open > 0;
    if ((!opened && !(was > 0 && open === 0)) || this.doorsHeard.length >= DOOR_SOUNDS) return;
    this.doorsHeard.push({ x: centre[0], row: centre[1], shut: !opened, gate });
  }

  update(frame: FrameState): void {
    const { curr, layout } = frame;
    const minute = this.fixedHour !== null ? this.fixedHour * 60 : (curr[layout.header.minute] ?? this.lastMinute);
    this.lastMinute = minute;
    this.lastWeekday = (Math.max(1, curr[layout.header.day] | 0) - 1) % 7;
    if (this.townMode) return;
    // Which flats of the building in view may be shown open, asked again a few times a second.
    if (this.viewFlats.length && this.flatsOpen && frame.now - this.flatsAskedAt >= FLATS_ASK_MS) {
      this.flatsAskedAt = frame.now;
      this.askFlats();
    }
    const h = layout.header;
    this.gamePaused = (curr[h.speed] | 0) === 0;
    // At 10× and super speed, the preset's step (Low's ten minutes: each update re-colours the sun, sky and lamps).
    const fast = curr[h.speed] >= 3;
    const step = fast ? Math.max(LIGHT_STEP, this.quality.lightingStep) : LIGHT_STEP;
    const due = this.lastLightMinute < 0 || frame.now - this.lastLightAt >= LIGHT_MIN_MS;
    if (due && Math.abs(minute - this.lastLightMinute) >= step) {
      // (A step of the clock: the sun turned only once it has moved enough, `sunTurned`; lit afresh, at once.)
      this.applyLighting(minute, this.lastLightMinute >= 0);
      this.lastLightMinute = minute;
      this.lastLightAt = frame.now;
    }
    // Who is where: doors open for those going through, neighbours' windows glow while someone is up.
    this.findDarkRooms(frame);
    this.lamps.frame(this.frameDt / 1000, this.darkRooms, this.awakeRooms, this.orbit.target.x, this.orbit.target.z); // (lamps and glows fading, which cast shadows, `lamps.ts`.)
    this.swingDoors(playbackRate(curr, h));
    this.steerCamera(this.frameDt / 1000);
    this.settleTarget(this.frameDt / 1000);
    this.keepCameraClear();
    if (this.lot.update(frame)) {
      this.mirrors.changed(); // (the objects' clips, the screens, the fires' flicker; things moving, in the held reflections.)
      this.pictureChanged();
    }
    this.placing.frame(this.frameDt); // (the effects, the thing in hand.)
    this.updateSims(frame);
    this.cars.update(frame);
    this.updateTownMovers(frame);
    this.findTownUp(frame);
    this.signals.update(frame);
    // (A car's shadow goes with it: `moved` only for a car that moved where it casts into the map, `setShadowTest`; the lights' poles when they're placed.)
    // (Cars that only drove draw over the still casters' picture, as the residents do: `Cars.movingCasters`.)
    if (this.signals.moved || this.cars.stillMoved) this.sunShadows.moved(false);
    else if (this.cars.moved) this.sunShadows.posed();
    if (this.cars.moved || this.signals.moved) this.pictureChanged();
    // (After the residents, who scare them. Paused, the birds keep still but for the wind's sway on their perches, which
    // stands still with the wind's clock while the frame is held: no change of the picture's.)
    this.birdLife.update(frame, minute, this.frameDt, this.characters, this.simCount);
    // From the town view (past `TOWN_FROM`) the house in view stands whole, its walls up and its roof on, like the
    // houses round it: cut away it is a hole in the town, and nobody looks into a room from there.
    const whole = this.orbit.radius > TOWN_FROM;
    if (whole !== this.townWhole) {
      this.townWhole = whole;
      this.applyWallMode();
      // (And every storey of it, under its roof: a lower one in view would leave it open to the sky.)
      if (this.storeys.count > 1) this.applyStoreys();
    }
    if (this.roofs.length && this.roofVisible() !== this.roofShown) {
      this.roofShown = !this.roofShown;
      this.showRoofs();
    }
  }

  /**
   * The residents posed into the pose texture (after the moving parts: residents on a moving seat go
   * with it), the chairs they sit pulled up to a table, their contact shadows, and their shadows in the
   * sun's map drawn again when they moved (at the preset's `shadowRate`).
   */
  private updateSims(frame: FrameState): void {
    const { curr, layout } = frame;
    this.simCount = Math.min(curr[layout.header.simCount], MAX_CHARACTERS);
    // (With the vendors and shopkeepers drawn after them.)
    const count = Math.min(this.simCount + (layout.header.staffCount === undefined ? 0 : curr[layout.header.staffCount]), MAX_CHARACTERS);
    const shadows = this.simShadowMatrices;
    this.simViewProj = this.camera.viewProjection();
    this.characters.update(frame, this.simInView, shadows, this.storey, this.doorHand);
    // Contact shadows only near the camera (the sun's shadow square is no bigger): none past `FAR_SHADOW` m off the viewed lot.
    const t = this.orbit.target;
    for (let i = 0; i < count; i++) {
      const o = i * 16;
      if (shadows[o + 15] === 0) continue;
      const sx = shadows[o + 12];
      const sz = shadows[o + 14];
      if (Math.hypot(sx - t.x, sz - t.z) > FAR_SHADOW && !this.inView(sx, sz, SIM_VIEW_MARGIN)) shadows.fill(0, o, o + 16);
    }
    this.lot.pullChairs(this.characters.pulled, this.placing.held); // (not the one in hand.)
    // (Bikes and e-scooters ridden off: not where they stand.)
    if (this.characters.riddenChanged) {
      this.characters.riddenChanged = false;
      this.lot.hideRidden(this.characters.ridden);
      this.parkedRides.hide(new Set(this.characters.ridden));
    }
    // (And those left out by a place their riders are visiting: drawn there.)
    if (this.characters.parkedChanged) {
      this.characters.parkedChanged = false;
      this.parkedRides.setAway(this.characters.parked);
    }
    this.lot.takeRestProps(this.characters.restTaken);
    // (Only residents posed where they cast into the map: `shadowPosed`, as the cars' `moved`.)
    if (this.quality.characterShadows && this.characters.shadowPosed) {
      this.sunShadows.posed();
      this.lamps.residentsMoved(this.occupiedRooms); // (their shadows in the lamps' light, in the rooms they are in.)
    }
    const blobs = this.simShadow;
    blobs.setCount(count);
    for (let i = 0; i < count; i++) blobs.setMatrix(i, shadows, i * 16);
    blobs.update(false);
    this.counts.residentsMs = this.characters.updateMs;
    this.lastFrame = frame;
  }

  /** The selection marker over the selected resident's head (hidden in the town, or while they aren't drawn). */
  private placeMarker(): void {
    const frame = this.lastFrame;
    const sel = this.selectedSim;
    const shown = !this.townMode && !!frame && sel !== null && sel < this.simCount && this.characters.head(sel, this.markerHead);
    const emotion = shown ? frame.curr[frame.layout.headerLen + sel * frame.layout.simStride + frame.layout.sim.emotion] : 0;
    // (Its bob by the clock of what is shown: still while the frame is held.)
    this.marker.place(shown ? this.markerHead : null, emotion, this.camera.entity, this.shownMs, !!this.options.reducedMotion);
  }

  /**
   * The warm-up behind the loading veil: every pipeline the game draws with is made
   * before it shows, so play doesn't stall on shader compiles (PlayCanvas compiles a shader, and on
   * WebGPU makes a pipeline, in the frame that first draws with it: tens of milliseconds each, and
   * the first frames of play would make hundreds). While the menus' town is up (the game built behind
   * them), the game's own state is set first: its residents posed, its lighting, the dig and the
   * cutaway, in a slice of its own. Then one frame of the game (`warmFrame`), the town back after
   * it. `?diag=shaders` logs what is still made in play.
   */
  async warmGame(frame: FrameState): Promise<void> {
    if (this.disposed) return;
    this.warming = true;
    try {
      await this.warmGameNow(frame);
    } finally {
      this.warming = false;
    }
  }

  /**
   * One frame of the warm-up (`warmFrame`'s), its new pipelines made side by side (`parallelPipelines.ts`: a
   * machine's first game compiles them all, and in turn that was most of a minute), then drawn again in full if
   * any were; how many were.
   */
  private async warmDraw(): Promise<number> {
    this.warmNext = true;
    const made = await this.parallelPipelines.frame(() => this.framesRendered(1));
    if (made && !this.disposed) {
      this.warmNext = true;
      await this.framesRendered(1);
    }
    return made;
  }

  private async warmGameNow(frame: FrameState): Promise<void> {
    this.shaderWatch.phase = 'warm-up';
    this.loadListener?.('Warming up', 0.65);
    if (this.townMode) {
      // (The game's camera is the orbit's parked pose; `update` poses the residents for it.)
      this.townMode = false;
      try {
        this.update(frame);
      } finally {
        this.townMode = true;
        this.lastLightMinute = -1;
      }
    }
    const t0 = performance.now();
    const made = await perfTimeAsync('warm: frame', () => this.warmDraw());
    this.shaderWatch.note(`warm-up frame done in ${Math.round(performance.now() - t0)} ms (${made} pipelines made side by side)`);
    // The woods' impostors baked and their cards drawn once now, not at the town view's first look (`nature.ts`).
    const landscape = this.lot.landscape;
    if (landscape && !this.disposed) {
      this.loadListener?.('Warming up (trees)', 0.7);
      await perfTimeAsync('warm: impostors', () => landscape.warmImpostors(() => this.framesRendered(1)));
      this.warmNext = true;
      await this.framesRendered(1);
      this.shaderWatch.note(`warm-up impostors done in ${Math.round(performance.now() - t0)} ms`);
    }
    // A big world: the camera roams to lots of every kind, each drawn in full once it settles. A few
    // that between them have most of what its houses are made of are built and drawn here too, so
    // those lots make few pipelines in play (each first one costs its frame tens of ms); then the lot
    // in view again. (Load time for smooth roaming: about a second more.)
    const world = this.lastWorld;
    const home = this.view;
    // (`diag=nowarm`: not for a bench that counts draws, which waits long enough for the pipelines anyway.)
    if (world?.meta?.world?.size === 'big' && home && !this.disposed && !this.diag.has('nowarm')) {
      const gains: number[] = [];
      const lots = warmLots(world, home, WARM_LOTS, gains);
      for (const [i, lot] of lots.entries()) {
        this.loadListener?.(`Warming up (lot ${i + 1} of ${lots.length})`, 0.75 + (0.25 * i) / lots.length);
        const span = perfStart(`warm: lot ${i + 1} of ${lots.length}`);
        await this.setWorld(world, lot);
        const made = await this.warmDraw();
        perfEnd(span, `${gains[i]} new kinds, ${made} pipelines`);
      }
      await perfTimeAsync('warm: home again', () => this.setWorld(this.lastWorld ?? world, home));
      this.shaderWatch.note(`warm-up lots done in ${Math.round(performance.now() - t0)} ms`);
    }
    this.shaderWatch.phase = this.townMode ? 'town' : 'game';
    this.loadListener?.('Warming up', 1);
  }

  /**
   * The warm-up's frame, set up in the frame loop right before it draws (`warmGame`); returns the
   * undo, run once it has drawn. The game's view (in place of the town's, if that's up), drawn
   * without culling: everything on the world's layers is drawn, what is behind the camera, in other
   * rooms and beyond the view too, and what is hidden now (`drawHidden`); buy mode's red copy
   * (`drawRedCopies`); the sun's map drawn; every reflection drawn with everything in it
   * (`Mirrors.warm`: their targets and turned faces); the picture chain as the game has it (the
   * multisampled scene, the image processing, Low's FXAA), the passes it leaves out for now too
   * (`PostProcessing.warm`). The frame goes to the canvas, behind the veil; the sun's map, the
   * reflections and the town's held frame are drawn again after.
   *
   * WebGPU makes its pipelines in the frame that first needs them (PlayCanvas has no asynchronous way
   * there); WebGL2 could skip draws whose shaders still compile (`asyncCompile`, the previews' choice),
   * but the game's held drawing (the sun's map, held reflections) would then keep a picture without
   * them: the warm-up makes them all instead. What only a thing never seen before uses (a model bought
   * for the first time) is still made when it first shows.
   */
  private warmFrame(): () => void {
    const town = this.townMode;
    if (town) this.showGame(true);
    const cam = this.camera.camera;
    cam.frustumCulling = false;
    this.sunShadows.moved(true);
    const hidden = this.drawHidden();
    const red = this.drawRedCopies();
    const mirrors = this.mirrors.warm(this.streamOn);
    // (And a lamp's shadow map, where lamps cast: its shadow pass's shaders, `lampShadows.ts`.)
    this.lampWarm = true;
    const lamps = this.lamps.warm(this.orbit.target.x, -this.orbit.target.z);
    // (And the picture chain's passes left out for now: the bloom's by day, `post.ts` `idlePasses`.)
    const post = this.post?.warm();
    return () => {
      post?.();
      lamps();
      this.lampWarm = false;
      mirrors();
      red();
      hidden();
      cam.frustumCulling = true;
      if (town) this.showGame(false);
      // (The map and the reflections drew what was hidden too: drawn again.)
      this.sunShadows.moved();
      this.held.changed();
    };
  }

  /**
   * For the warm-up's frame: what the world has but doesn't draw now drawn too, every mesh, hidden
   * ones too: the storeys out of view, things hidden (the night's garden pools and glows by day, the
   * build tools' helpers, the templates models are copied from), and the instanced draws that have
   * no instances now (the residents' close-up face parts, lamp glows by day) with one (whatever its
   * data: the frame is behind the veil). Entities with lights or cameras in them stay as they are (a
   * light more would change every lit shader). Returns the undo.
   */
  private drawHidden(): () => void {
    const entities: pc.Entity[] = [];
    const visible: pc.MeshInstance[] = [];
    const counts: pc.MeshInstance[] = [];
    this.root.forEach((node) => {
      const e = node as pc.Entity;
      if (e.enabled || !(e instanceof pc.Entity) || e.findComponents('light').length || e.findComponents('camera').length) return;
      e.enabled = true;
      entities.push(e);
    });
    for (const layer of this.app.scene.layers.layerList) {
      for (const mi of layer.meshInstances) {
        if (!mi.visible) {
          mi.visible = true;
          visible.push(mi);
        }
        const data = mi.instancingData;
        if (data && data.count === 0 && (data.vertexBuffer?.numVertices ?? 0) > 0) {
          mi.instancingCount = 1;
          counts.push(mi);
        }
      }
    }
    return () => {
      for (const mi of counts) mi.instancingCount = 0;
      for (const mi of visible) mi.visible = false;
      for (let i = entities.length - 1; i >= 0; i--) entities[i].enabled = false;
    };
  }

  /**
   * For the warm-up's frame: buy mode's red copy of the thing in hand (where it can't go) over a mesh
   * of each vertex layout the world's things have, on the over layer as the thing in hand is (which
   * clears the depth before it, `layers.ts`). The red copy's shader goes with the mesh's layout, and
   * a thing never seen before still makes its own. Returns the undo.
   */
  private drawRedCopies(): () => void {
    const [plain, coloured] = this.placing.ghostMaterials;
    const over = overLayer(this.app, this.camera.camera).id;
    const seen = new Set<number>();
    const made: Instances[] = [];
    for (const mi of this.app.scene.layers.getLayerById(pc.LAYERID_WORLD)?.meshInstances ?? []) {
      const mesh = mi.mesh;
      const format = mesh?.vertexBuffer?.format;
      if (!mi.instancingData || !mesh || !format || seen.has(format.renderingHash)) continue;
      seen.add(format.renderingHash);
      const colours = format.elements.some((e) => e.name === pc.SEMANTIC_COLOR);
      const red = new Instances(this.device, this.storeyLayer, mesh, colours ? coloured : plain, { name: 'warmRedCopy', castShadows: false, receiveShadows: false, layers: [over], cull: false });
      red.setCount(1);
      red.setMatrix(0, IDENTITY);
      red.update(false);
      made.push(red);
    }
    return () => {
      for (const red of made) red.destroy();
    };
  }

  /** For the warm-up's frame in the menus' town: the game's view (`on`), or the town's back. */
  private showGame(on: boolean): void {
    this.townMode = !on;
    this.showLayers(!on);
    this.sunShadows.setOverview(!on);
    this.applyDig();
    if (on) {
      this.applyLighting(this.lastMinute);
      this.cutLook = -1;
      this.updateCutaway();
    } else this.lastLightMinute = -1;
    this.applyCamera();
  }

  setItems(items: readonly ItemPlacement[]): void {
    this.pictureChanged();
    this.lot.setItems(items);
  }
  setLampsOff(off: ReadonlySet<number>): void {
    this.pictureChanged();
    this.lot.setLampsOff(off); // (fans and fires.)
    this.lamps.setLampsOff(off); // (their light and fades, `lamps.ts`.)
    this.applyGardenPools();
  }
  setCurtainsShut(shut: ReadonlySet<number>): void {
    this.pictureChanged();
    this.curtainsShut = shut;
    this.lot.setCurtainsShut(shut); // (drawn, `objectMotion.ts`.)
    this.applyGardenPools(); // (less light through them on the garden.)
  }
  holdModelRefresh(on: boolean): void {
    this.modelRefresh.hold(on);
  }

  // --- light ---------------------------------------------------------------------------

  /**
   * The time of day (`shared/environment.ts`): the sun, the sky's fill, the environment's light,
   * the fog and the clear colour, in PlayCanvas's units (`lightUnits.ts`). `step`: a step of the clock's, whose sun
   * turns only once it has moved far enough (`sunTurned`); otherwise (a new hour, a load, the bench's) exactly. No
   * allocations.
   */
  private applyLighting(minute: number, step = false): void {
    this.counts.lightingUpdates++;
    this.pictureChanged(); // (the sky, its light and the lamps', not all of which a held frame sees.)
    const l = lightingAt(minute, LOOK, this.lighting);
    const mix = this.lightMix;
    if (mix) {
      l.sunIntensity *= mix[0];
      l.skyIntensity *= mix[1];
      l.envIntensity *= mix[2];
    }
    // A directional light shines along its entity's -y: turned from straight down to the sun's way (game space to PlayCanvas).
    const to = toEngine(l.sunDirection.x, l.sunDirection.y, l.sunDirection.z, this.vTmp).normalize();
    // Only a sun that turned draws its map again: a step that only changes the light's colour (every step of the
    // night, whose moon stands still, and most of the day's: `sunTurned`) leaves the shadows as they are; the held
    // reflections take the new light either way.
    this.mirrors.changed();
    if (step ? sunTurned(to, this.sunWay) : !to.equals(this.sunWay)) {
      this.sunWay.copy(to);
      const axis = this.vTmp2.cross(DOWN, to);
      const sin = axis.length();
      const angle = Math.atan2(sin, DOWN.dot(to)) * pc.math.RAD_TO_DEG;
      this.sun.setRotation(sin > 1e-6 ? this.qTmp.setFromAxisAngle(axis.mulScalar(1 / sin), angle) : this.qTmp.set(0, 0, 0, 1));
      this.sunShadows.moved(true, false);
    }
    // The look's colours are used as linear; PlayCanvas takes sRGB and linearises it,
    // so the strength goes into the colour (an intensity below 1 would be linearised with it).
    this.sun.light!.color = gammaColor(l.sunColor, l.sunIntensity * SUN_UNITS, this.colorTmp);
    // The sky's hemispheric fill and its image-based light.
    this.lib.setSkyFill(l.skyColor, l.groundColor, l.skyIntensity * SKY_UNITS);
    this.lib.setEnvironmentIntensity(l.envIntensity * ENV_UNITS);
    // The haze in the horizon's colour as the look was tuned with it: linearised twice (here, then
    // in the materials: a deeper blue than the horizon), so it goes to PlayCanvas (which linearises
    // what it's given) linearised once. The clear colour (behind the sky's dome) is drawn as it's
    // given: linear.
    const c = l.clearColor;
    this.app.scene.fog.color.set(toLinear(c.r), toLinear(c.g), toLinear(c.b), 1);
    this.camera.camera.clearColor = this.colorTmp.set(toLinear(c.r), toLinear(c.g), toLinear(c.b), 1);
    // The look's exposure for the hour and the moonlight grade (`post.ts`: uniforms).
    if (this.post) this.post.exposure = l.exposure * LOOK.exposure;
    this.applyUnderLight(); // (and the moonlight grade.)
    // Glowing windows outside at night, cool daylight in the panes by day; the street's lamps; the garden's pools.
    this.lampsLevel = l.lamps;
    this.lot.applyLighting(l); // (the sky, the night clips, the shades' glow.)
    this.placing?.light(l); // (the red ghost's light: the sun and the sky, `placementGhost.ts`.)
    this.lamps.applyLighting(l); // (the interior lights and the glows, `lamps.ts`.)
    const glow = Math.min(1, Math.max(0, (l.lamps - 0.35) / 0.5));
    const g = this.glowColor;
    const k = glow * glow * 1.6;
    this.lib.setEmissive(this.lib.surface('material.window', { vertexColors: true, cutaway: true }), this.emissiveTmp.set(g.r * k, g.g * k, g.b * k));
    const inner = (1 - l.lamps) * 0.9 + 0.05;
    this.lib.setEmissive(this.lib.surface('material.window.inner', { vertexColors: true, cutaway: true }), this.emissiveTmp.set(toLinear(c.r) * inner, toLinear(c.g) * inner, toLinear(c.b) * inner));
    this.street?.setNight(l.lamps, this.glowColor);
    this.townBuildings?.setNight(l.lamps, this.glowColor);
    this.signals?.setNight(l.lamps); // (the traffic lights' lit lenses.)
    this.overview?.setNight(l.lamps, this.glowColor);
    this.applyGardenPools();
  }

  // --- the menus' overview (`overview.ts`) ----------------------------------

  get townShown(): boolean {
    return this.townMode;
  }

  /** Builds the overview of `world` (if it isn't already), with its highlights, out of the house's lamps. */
  private async buildOverview(overview: TownOverview, world: WorldStructure): Promise<void> {
    if (!(await overview.build(world, () => this.pace()))) return;
    if (this.townHighlight) overview.setHighlight(this.townHighlight);
    // Interior lamps of the game's house never reach the overview (nor are they on its layer: this keeps its shaders simple).
    perfTime('town: lamps kept out', () => this.lamps.exclude(overview.meshInstances()));
    // (The new street's lamps and windows at the hour shown.)
    if (this.townMode) this.lastLightMinute = -1;
  }

  showTown(world: WorldStructure, minute: number, onStep?: (done: TownBuildStep) => void): Promise<void> {
    if (LAND_FIXTURE.size) applyLandFixture(world);
    const ticket = ++this.townTicket;
    this.townWorld = world;
    const run = this.townQueue.then(async () => {
      // Requests queue up while one builds; only the newest is built.
      if (ticket !== this.townTicket || this.disposed) return;
      const overview = (this.overview ??= new TownOverview(this.app, this.root, this.deps.assets, this.deps.content, this.house));
      const ring = landscapeRing(world);
      Object.assign(this.townReach, { x0: -ring, z0: -ring, x1: world.width + ring, z1: groundDepth(world) + ring });
      // (The player's pan keeps the camera's target over the land, the town or none, and its eye near it and its edge in
      // sight over the landscape round it.)
      this.townRig.setBounds({ x0: 0, z0: 0, x1: world.width, z1: groundDepth(world) }, { ...this.townReach });
      if (this.townBuilds++ === 0 && !this.worldBuilding) this.sliceStart = performance.now();
      // (A town on screen already, built again: longer slices, `AGAIN_SLICE_MS`.)
      this.townAgain = this.townMode;
      try {
        // The landscape on the town layer (the game's, when it's the same town). Same seed and size
        // can still be a new layout ("New neighbours"): the lawn fitted again to its houses.
        // The landscape and the town's own (its street, its lots' things) side by side: the landscape
        // waits on the render data workers (the terrain's shading, the grass field) for much of its
        // build, and the overview's planning fills those waits. (Each gives the frames a turn, `pace`.)
        const [nature] = await both(
          this.lot.useNature('town', world, (roads, parent) => drawRoads(this.app, roads, parent), onStep),
          this.buildOverview(overview, world).then(() => onStep?.('town')),
        );
        perfTime('town: lawn', () => nature.landscape.refreshLawn(world));
        // The camera keeps a clear view over the hills (and out of the mountains).
        this.townRig.ground = (x, z) => nature.landscape.heightAt(x, z);
        // (The land faded into the haze past its landscape's edge, from far out.)
        this.landHaze ??= new LandHaze(this.app, this.root);
        this.hazeLand(nature.landscape);
      } finally {
        this.townBuilds--;
      }
      // (Step 2's ghosts and painted uses over the land as it is built now.)
      if (ticket === this.townTicket && !this.disposed) {
        this.drawGhosts();
        this.drawUses();
      }
      if (ticket !== this.townTicket || this.disposed || this.townMode) return;
      // Entering from the game (or at start-up): the shot's hour at once.
      this.townMinute = this.townMinuteGoal = this.townShot?.minute ?? minute;
      this.enterTown();
      this.drawGhosts();
      this.drawUses();
    });
    this.townQueue = run.catch(() => {});
    return run;
  }

  async hideTown(glide = 0): Promise<void> {
    // Let a pending overview build finish first, so it can't switch back to the overview later.
    this.townTicket++;
    await this.townQueue;
    if (!this.townMode) return;
    if (glide > 0 && this.active) {
      // The camera glides from the town down to the game's pose (kept by the orbit meanwhile).
      const o = this.orbit;
      this.flyingIn = true;
      await new Promise<void>((resolve) => {
        const timer = setTimeout(resolve, glide * 1000 + 600);
        this.townRig.glideTo({ alpha: o.alpha, beta: o.beta, radius: o.radius, x: o.target.x, z: o.target.z }, glide, () => {
          clearTimeout(timer);
          resolve();
        });
      });
      this.flyingIn = false;
    }
    if (this.townMode) this.leaveTown();
  }

  setTownShot(shot: TownShot, cut = false): void {
    this.townShot = shot;
    this.townRig.setShot(shot, cut || !this.townMode);
    if (shot.minute !== undefined) {
      this.townMinuteGoal = shot.minute;
      if (cut || !this.townMode) this.townMinute = shot.minute;
    }
  }

  nudgeTown(dAlpha: number, zoom: number, dBeta = 0): void {
    if (this.townMode) this.townRig.nudge(dAlpha, zoom, dBeta);
  }

  zoomTownAt(x: number, y: number, zoom: number): void {
    if (!this.townMode) return;
    // (Over the sky, towards the camera's target: a plain zoom.)
    const at = this.pickLand(x, y);
    if (at) this.townRig.zoomAt(at.x, at.z, zoom);
    else this.townRig.nudge(0, zoom);
  }

  panTown(dx: number, dz: number): void {
    if (this.townMode) this.townRig.pan(dx, dz);
  }

  panTownView(right: number, forward: number): void {
    if (this.townMode) this.townRig.panView(right, forward);
  }

  setTownNearest(metres: number): void {
    this.townRig.setNearest(metres);
  }

  get townDistance(): number {
    return this.townPose.radius;
  }

  dragTown(x0: number, y0: number, x1: number, y1: number): void {
    if (!this.townMode) return;
    // The land under the pointer follows it; where either point is over the sky or past the land, the level of the
    // land under the camera's target does instead (a drag that starts on the horizon still pans).
    const a = this.pickLand(x0, y0);
    const b = a && this.pickLand(x1, y1);
    if (a && b) {
      this.townRig.pan(a.x - b.x, a.z - b.z);
      return;
    }
    const pose = this.townRig.pose;
    const level = this.townRig.ground?.(pose.tx, pose.tz) ?? 0;
    const reach = pose.radius * 4;
    const p = this.rayToLevel(x0, y0, level, reach, this.dragFrom);
    const q = p && this.rayToLevel(x1, y1, level, reach, this.dragTo);
    if (p && q) this.townRig.pan(p.x - q.x, p.z - q.z);
  }

  reframeTown(seconds = 1.2): void {
    this.townRig.reframe(seconds);
  }

  private readonly dragFrom = { x: 0, z: 0 };
  private readonly dragTo = { x: 0, z: 0 };

  /** Where the ray through canvas CSS pixel (`x`, `y`) meets the level `y` m (into `out`), no further than `reach` m along it; null if it rises. */
  private rayToLevel(x: number, y: number, level: number, reach: number, out: { x: number; z: number }): { x: number; z: number } | null {
    const { origin, dir } = this.landRay;
    this.camera.ray(x, y, this.cssSize.width, this.cssSize.height, origin, dir);
    if (dir.y > -1e-3) return null;
    const t = Math.min(reach, (level - origin.y) / dir.y);
    if (t <= 0) return null;
    out.x = origin.x + dir.x * t;
    out.z = origin.z + dir.z * t;
    return out;
  }

  setLotHighlight(highlight: LotHighlight, snap = false): void {
    this.townHighlight = highlight;
    this.overview?.setHighlight(highlight, snap);
  }

  setTownGhosts(ghosts: TownGhosts | null): void {
    this.ghosts = ghosts;
    this.drawGhosts();
  }

  setTownUses(layer: UseLayer | null, hatched?: Uint8Array | null): void {
    this.uses = layer;
    this.usesHatched = hatched ?? null;
    this.drawUses();
  }

  /** Step 2's painted uses (`setTownUses`), and their overlay (made the first time there are any). */
  private uses: UseLayer | null = null;
  private usesHatched: Uint8Array | null = null;
  private useDraw: UseOverlayDraw | null = null;

  /**
   * The uses drawn over the menus' town (only there): the cells changed since the last time, or all of them over
   * another land (a new landscape, or the land drawn again) or at another least width of their edges on screen.
   */
  private drawUses(): void {
    const at = this.townMode || !this.lot.landscape ? this.landInView() : null;
    const uses = at ? this.uses : null;
    if (!uses && !this.useDraw) return;
    if (!this.useDraw) {
      this.useDraw = new UseOverlayDraw(this.app, this.root);
      this.useDraw.setScale(this.townMetresPerPx());
    }
    const layer = this.lot.townLayer ?? pc.LAYERID_WORLD;
    const land = at ? `${landscapeId(at.scape)}|${at.scape.landVersion}` : null;
    this.useDraw.set(uses, this.usesHatched, at ? groundOrWater(at.scape) : () => 0, layer, land);
    this.held?.changed();
  }

  /** Step 2's edits over the town (`setTownGhosts`), and their drawing (made the first time there are any). */
  private ghosts: TownGhosts | null = null;
  private ghostDraw: TownGhostDraw | null = null;

  /** The ghosts drawn again over the land in view (after they changed, or the town under them was built again). */
  private drawGhosts(): void {
    const at = this.townMode || !this.lot.landscape ? this.landInView() : null;
    const ghosts = at ? this.ghosts : null;
    if (!ghosts && !this.ghostDraw) return;
    if (!this.ghostDraw) {
      this.ghostDraw = new TownGhostDraw(this.app, this.root);
      this.ghostDraw.setScale(this.townMetresPerPx());
    }
    const layer = this.lot.townLayer ?? pc.LAYERID_WORLD;
    this.ghostDraw.set(ghosts, at ? groundOrWater(at.scape) : () => 0, layer, this.townWorld, at ? roadHeightsOf(at.world) : null);
    this.held?.changed();
  }

  /** How many m a CSS px is where the overview's camera looks: its distance, lens and the canvas's height. */
  private townMetresPerPx(): number {
    return (2 * this.townPose.radius * Math.tan(this.townPose.fov / 2)) / (this.cssSize.height || 1);
  }

  onTownFrame(fn: (() => void) | null): void {
    this.townListener = fn;
  }

  // --- the land as a height field (`game/land.ts`; `shared/landBake.ts`) ------------------

  /** The land brush's cursor (`setLandBrush`), made the first time it's shown. */
  private landCursor: LandCursorDraw | null = null;
  /** The work of strokes done and fields taken on (`landChanged`, `setLand`), one after another, a slice a frame. */
  private landQueue: Promise<void> = Promise.resolve();
  private landSliceStart = 0;
  private readonly landRay = { origin: { x: 0, y: 0, z: 0 }, dir: { x: 0, y: -1, z: 0 } };

  /** The landscape in view and its world: the menus' town while it is shown (or there is no game world), else the game's. */
  private landInView(): { scape: Landscape; world: WorldStructure } | null {
    const town = this.lot.landscapeOf('town');
    const game = this.lot.landscape;
    if ((this.townMode || !game) && town && this.townWorld) return { scape: town, world: town.builtFor ?? this.townWorld };
    return game && this.builtWorld ? { scape: game, world: game.builtFor ?? this.builtWorld } : null;
  }

  /** The land's work gives the frames a turn every `LAND_SLICE_MS` of main-thread time. */
  private readonly landPace = async (): Promise<void> => {
    if (performance.now() - this.landSliceStart < LAND_SLICE_MS) return;
    await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
    this.landSliceStart = performance.now();
  };

  /** Runs `job` after the land's work before it, in slices (`landPace`); a failure is logged, the queue goes on. */
  private queueLand(name: string, job: () => Promise<void>): Promise<void> {
    const run = this.landQueue.then(async () => {
      if (this.disposed) return;
      this.landSliceStart = performance.now();
      await perfTimeAsync(name, job);
      this.held?.changed();
    });
    this.landQueue = run.catch((err) => console.error(`[render] ${name} failed`, err));
    return this.landQueue;
  }

  /** The menus' land's haze and curtain fitted to the land as it now is. */
  private landMapped(scape: Landscape): void {
    if (scape === this.lot.landscapeOf('town')) this.hazeLand(scape);
  }

  /**
   * The menus' land's haze and the curtain closing its cut edge (`landHaze.ts`) for `scape` as it is now: made again
   * when its land changed (a stroke that raised the edge, or opened a valley past it, `landVersion`).
   */
  private hazeLand(scape: Landscape): void {
    this.landHaze?.set(this.townReach, (x, z) => scape.heightAt(x, z), townLayer(this.app).id, `${landscapeId(scape)}|${scape.landVersion}`);
  }

  landOf(): Promise<Land> {
    const at = this.landInView();
    if (!at) return Promise.reject(new Error('no land in view'));
    // (Drawn from a height field already, the world's own or one given: a copy of it as it is now.)
    const field = at.scape.heightField;
    if (field) return Promise.resolve(copyLand(field));
    return perfTimeAsync('land: survey', async () => {
      const off = OffThreadLand.open(at.world);
      try {
        const made = await off?.bake();
        if (made) {
          perfEnd(Object.assign(perfStart('land: survey (worker)'), { start: performance.now() - made.ms }));
          return made.field;
        }
      } finally {
        off?.close();
      }
      return bakeLand(at.scape, at.world.width, groundDepth(at.world));
    });
  }

  setLand(land: Land | null): void {
    const at = this.landInView();
    if (!at) return;
    const { scape } = at;
    const before = scape.heightField;
    // (The same land again, `landOf`'s copy of it: taken on, nothing drawn again.)
    if (land && before && sameLand(land, before)) {
      scape.setLand(land);
      return;
    }
    if (!land && !before) return;
    scape.setLand(land);
    void this.queueLand('land: drawn again', async () => {
      await scape.redrawLand(this.landPace);
      await this.lot.regrowNature(scape, null, () => this.disposed, this.landPace);
      this.landMapped(scape);
    });
  }

  landChanged(rect: LandRect, done: boolean): void {
    const at = this.landInView();
    const field = at?.scape.heightField;
    if (!at || !field) return;
    const { scape } = at;
    const span = perfStart('land: stamp');
    const tiles = scape.reground(rect);
    perfEnd(span, `${tiles} tiles`);
    this.held?.changed();
    if (!done) return;
    void this.queueLand('land: stroke done', async () => {
      const box = await scape.landDone(rect, this.landPace);
      if (!box) return;
      await this.lot.regrowNature(scape, box, () => this.disposed, this.landPace);
      this.landMapped(scape);
      // (The game's land: what stands on it drawn again on its surface as the world comes back from the stroke, the
      // lots' buildings too; the sun's shadows of the still casters taken again.)
      if (scape === this.lot.landscape) this.landUnderTown(scape);
    });
  }

  settleLand(rect: LandRect, before?: Int16Array): LandRect | null {
    const at = this.landInView();
    if (!at?.scape.heightField) return null;
    return perfTime('land: settle', () => at.scape.settleLand(rect, before));
  }

  /**
   * The game's land shaped (a stroke done, undone or redone): the street stands on the landscape's surface (one surface
   * for the land and what stands on it) and plans again the chunks whose roads or lots moved; the viewed lot follows its
   * pad (the datum); the traffic lights, the cars, the parked rides and the gates stand on it again; the still casters' shadows
   * are taken again.
   */
  private landUnderTown(scape: Landscape): void {
    const surface = scape.surface;
    this.sunShadows.moved();
    if (!surface || !this.builtWorld) return;
    if (this.street.landMoved(surface)) this.streamStreet();
    this.regroundPlaced();
    const view = this.view;
    const at = view ? this.builtWorld.plots.findIndex((p) => p.x === view.x && p.z === view.z && p.w === view.w && p.d === view.d) : -1;
    if (at < 0) return;
    const datum = surface.padY(at);
    this.applyDatum(datum);
    this.street.setLotY(datum);
  }

  /**
   * Those that look the ground up only when placed stand on the surface again (solved again: a land stroke, a world's
   * roads taken on): the traffic lights' poles, the cars standing, the parked rides, the other lots' gates.
   */
  private regroundPlaced(): void {
    this.signals.setGround(this.builtGround);
    this.cars.setGround(this.builtGround);
    this.parkedRides.setGround(this.builtGround);
    if (this.townGates) regroundGates(this.townGates, this.builtGround);
  }

  pickLand(x: number, y: number): { x: number; y: number; z: number } | null {
    const at = this.landInView();
    if (!at) return null;
    const { origin, dir } = this.landRay;
    this.camera.ray(x, y, this.cssSize.width, this.cssSize.height, origin, dir);
    // (The ray in the engine's heights: the land's less the datum.)
    const datum = this.townMode ? 0 : this.datum;
    return marchToGround(origin, dir, (px, pz) => at.scape.heightAt(px, pz) - datum);
  }

  setLandBrush(cursor: LandCursor | null): void {
    const at = this.landInView();
    if (!cursor || !at) {
      this.landCursor?.set(null, () => 0, 0);
      return;
    }
    this.landCursor ??= new LandCursorDraw(this.app, this.root);
    const layer = this.townMode ? (this.lot.townLayer ?? pc.LAYERID_WORLD) : pc.LAYERID_WORLD;
    this.landCursor.set(cursor, (x, z) => at.scape.heightAt(x, z), layer);
    this.held?.changed();
  }

  /** The whole land's samples (`landDebug.ts`). */
  landRect(): LandRect | null {
    const field = this.landInView()?.scape.heightField;
    return field ? fullRect(field) : null;
  }

  /** The menus' town comes on screen: the overview's camera from where the game's is (whose pose the orbit keeps), its layer, its light. */
  private enterTown(): void {
    this.townMode = true;
    // (The game's haze let go: the menus' town has its own.)
    this.worldHaze?.hold(false);
    this.worldHazeShown = false;
    this.shaderWatch.phase = 'town';
    this.birdLife.setTown(true);
    this.applyDig(); // (no basement or windowless rooms kept dark in the town.)
    const o = this.orbit;
    const p = this.townPose;
    Object.assign(p, { alpha: o.alpha, beta: o.beta, radius: o.radius });
    Object.assign(p.target, o.target);
    // (The menus' town at datum 0: the game's pose there in game space.)
    p.target.y += this.datum;
    setDatum(this.root, this.lotRoot, 0);
    this.lot.datum = 0;
    p.targetScreenOffset.x = p.targetScreenOffset.y = 0;
    this.townRig.syncFromCamera();
    if (this.townShot) this.townRig.setShot(this.townShot, true);
    this.orbit.enabled = false;
    this.orbit.stopInertia();
    this.showLayers(true);
    this.sunShadows.setOverview(true);
    this.hoverTile.set(null);
    this.lastLightMinute = -1;
  }

  /** The game comes on screen again (the menus' town is let go a moment later: `releaseTown`). */
  private leaveTown(): void {
    this.townMode = false;
    this.landHaze?.show(false);
    // (The game has no ghosts nor painted uses: drawn again over the town when it is shown again.)
    this.ghostDraw?.set(null, () => 0, 0, null);
    this.useDraw?.clear();
    setDatum(this.root, this.lotRoot, this.datum);
    this.lot.datum = this.datum;
    const ticket = this.townTicket;
    setTimeout(() => this.releaseTown(ticket), TOWN_RELEASE_MS);
    this.shaderWatch.phase = 'game';
    this.birdLife.setTown(false);
    this.applyDig(); // (the basement and windowless rooms dark again.)
    this.showLayers(false);
    this.sunShadows.setOverview(false);
    this.orbit.enabled = true;
    this.orbit.stopInertia();
    this.app.scene.fog.density = LOOK.fog;
    this.applyLighting(this.lastMinute);
    this.lastLightMinute = this.lastMinute;
    this.cutLook = -1;
    this.updateCutaway();
  }

  /**
   * Lets go of the menus' town while the game is on screen: the overview (a Big world's is some
   * 800 MB of heap, 230 MB of buffers and 130 MB of GPU memory) and its landscape, unless it is the
   * game's too. Not if the town was asked for again since (`ticket`): the next `showTown` builds it
   * again (some seconds behind the menus' own backdrop, as a new town's).
   */
  private releaseTown(ticket: number): void {
    const gone = () => this.townMode || this.disposed || ticket !== this.townTicket;
    if (gone()) return;
    // (After a build or a refresh still running: neither shows the town now, `townTicket`.)
    this.townQueue = this.townQueue
      .then(() => {
        if (gone()) return;
        this.overview?.destroy();
        this.overview = null;
        this.townWorld = null;
        this.landHaze?.destroy();
        this.landHaze = null;
        this.townRig.ground = null;
        this.lot.dropNature('town');
      })
      .catch(() => {});
  }

  /**
   * What the camera draws and the sun lights (the camera's layers and the sun's): while the menus'
   * town is shown, its layer in place of the world's and of build mode's (`TOWN_CAMERA_LAYERS` stay),
   * the sun on it alone (its casters are the overview's and the town's landscape); the game's again
   * after, with any layer added to them meanwhile.
   */
  private showLayers(town: boolean): void {
    const cam = this.camera.camera;
    const sun = this.sun.light!;
    const id = townLayer(this.app).id;
    if (town) {
      if (this.gameLayers) return;
      this.gameLayers = { camera: cam.layers.slice(), sun: sun.layers.slice() };
      cam.layers = cam.layers.flatMap((l) => (l === pc.LAYERID_WORLD ? [id] : TOWN_CAMERA_LAYERS.includes(l) ? [l] : []));
      sun.layers = [id];
    } else {
      const kept = this.gameLayers;
      if (!kept) return;
      this.gameLayers = null;
      cam.layers = [...kept.camera, ...cam.layers.filter((l) => l !== id && !kept.camera.includes(l))];
      sun.layers = [...kept.sun, ...sun.layers.filter((l) => l !== id && !kept.sun.includes(l))];
    }
    this.sunShadows.moved();
  }

  /** Overview per frame: the camera rig, the highlights, the light at the menu's hour (its haze: `applyTownCamera`). Allocation-free. */
  private townFrame(dt: number): void {
    const w = this.cssSize.width || 1;
    const h = this.cssSize.height || 1;
    const secs = Math.min(0.1, dt / 1000);
    this.townRig.setViewport(w, h);
    this.townRig.frame(secs, w / h);
    // (Its highlights' colours are written in place: the held frame is drawn again, `heldFrames.ts`. From far out the
    // lit lots are washed over whole.)
    this.overview?.setReach(this.townPose.radius);
    // (Step 2's ghosts drawn again when the zoom crosses a step of their least widths on screen.)
    if (this.ghostDraw?.setScale(this.townMetresPerPx()) && this.ghosts) this.drawGhosts();
    if (this.useDraw?.setScale(this.townMetresPerPx()) && this.uses) this.drawUses();
    if (this.overview?.frame(secs)) this.held.changed();
    // The light eases between the screens' hours (about two seconds).
    const goal = this.townMinuteGoal;
    if (this.townMinute !== goal) {
      const step = Math.max(0.5, Math.abs(goal - this.townMinute) * Math.min(1, dt / 600));
      this.townMinute = Math.abs(goal - this.townMinute) <= step ? goal : this.townMinute + Math.sign(goal - this.townMinute) * step;
    }
    // (The lots' buildings' windows the street drew again: the held frame too.)
    if (this.overview?.frameLights()) this.held.changed();
    if (this.lastLightMinute !== this.townMinute) {
      this.applyLighting(this.townMinute);
      this.lastLightMinute = this.townMinute;
      // (The lots' buildings' windows by the hour: a weekday's, the shops open by day.)
      this.overview?.setTime(this.townMinute, MENU_WEEKDAY);
      // (The streetlights' and windows' glows too, in their materials.)
      this.held.changed();
    }
  }

  /**
   * The overview, between its builds: what was drawn without `urls` is built again now if the overview
   * is still on screen as last asked for (no newer request, not left: no new ticket is taken, so it
   * never comes back after `hideTown`), else the next time it's shown.
   */
  private refreshTown(urls: readonly string[]): void {
    const overview = this.overview;
    if (!overview) return;
    const ticket = this.townTicket;
    this.townQueue = this.townQueue
      .then(async () => {
        const town = urls.map((url) => overview.invalidate(url)).some(Boolean);
        const world = this.townWorld;
        if (!town || !world || !this.townMode || ticket !== this.townTicket || this.disposed) return;
        if (this.townBuilds++ === 0 && !this.worldBuilding) this.sliceStart = performance.now();
        try {
          await this.buildOverview(overview, world);
        } finally {
          this.townBuilds--;
        }
      })
      .catch(() => {});
  }

  // --- picking -------------------------------------------------------------------------

  pick(x: number, y: number): PickResult {
    return this.picker.pick(x, y);
  }

  buildingAt(x: number, y: number): BuildingHit | null {
    const world = this.builtWorld;
    if (!world || this.townMode) return null;
    const { origin, dir } = this.landRay;
    this.camera.ray(x, y, this.cssSize.width, this.cssSize.height, origin, dir);
    const view = this.view;
    let best = Infinity;
    let hit: BuildingHit | null = null;
    const looks = world.meta?.lotLooks;
    for (let i = 0; i < world.plots.length; i++) {
      const p = world.plots[i];
      if (!p.house || (view && view.x === p.x && view.z === p.z)) continue;
      const [x0, z0, x1, z1] = p.house;
      const g = [this.groundY(x0, z0), this.groundY(x1, z0), this.groundY(x1, z1), this.groundY(x0, z1)];
      // (A lot with a look as tall as the street draws it closed: its real storeys, `lotLookBuilding`.)
      const tall = lotWallsHeight(p.storeys, p.public || p.kind === 'flats' ? looks?.[i] : undefined, WALL_HEIGHT, TOWN_STOREY);
      const t = rayPrism(origin, dir, rectFootprint(x0, z0, x1, z1), Math.min(...g) - 0.5, Math.max(...g) + tall + PICK_ROOF);
      if (t < best) [best, hit] = [t, { kind: 'lot', plot: p.id }];
    }
    // (The viewed lot's own ground, nearer than the building: what was meant is there.)
    if (hit && view && dir.y < -1e-6) {
      const t = -origin.y / dir.y;
      const gx = origin.x + dir.x * t;
      const gz = origin.z + dir.z * t;
      if (t < best && gx >= view.x && gx <= view.x + view.w && gz >= view.z && gz <= view.z + view.d) return null;
    }
    return hit;
  }

  pickPlane(height: number): { x: number; z: number } | null {
    return this.picker.pickPlane(height);
  }

  pickOnWall(axis: 'h' | 'v', line: number, storeyRow: number): { along: number; y: number } | null {
    return this.picker.pickOnWall(axis, line, storeyRow);
  }

  objectBox(id: number): { height: number; bottom: number } | null {
    return this.picker.objectBox(id);
  }

  setResizeHandles(handles: readonly ResizeHandle[] | null, hot: string | null = null): void {
    this.pictureChanged();
    // In render space: an upper storey's rows lie over the ground's, a storey's height up.
    this.handles.set(
      handles,
      (h, out) => {
        const k = this.storeyOf(h.z);
        out.set(h.x, k * WALL_HEIGHT + h.y, h.z - this.rowOf(k));
      },
      hot,
    );
  }

  handleAt(x: number, y: number): string | null {
    return this.handles.at(x, y, this.projectPoint);
  }

  private readonly projectPoint = (x: number, y: number, z: number, out: { x: number; y: number }) => this.project(x, y, z, out);

  // --- settings, stats, the benchmark --------------------------------------------------

  configure(options: LiveRenderOptions): void {
    this.pictureChanged();
    // Taken before the merge, which would otherwise hide the change. (`?look=` on the dev server wins over the setting.)
    const shading = options.shading === undefined ? undefined : (softLookFromURL() ?? options.shading);
    this.options = { ...this.options, ...options };
    if (shading !== undefined && shading !== this.shading) {
      // A define on every lit material: the next frames draw the other way (their shaders compile again).
      this.shading = shading;
      this.lib.setShading(shading);
    }
    const s = Math.max(0.25, options.cameraSensitivity);
    this.orbit.angularSensibilityX = this.orbit.angularSensibilityY = 1000 / s;
    this.orbit.panningSensibility = 90 / s;
    this.orbit.wheelDeltaPercentage = 0.012 * s;
    this.lot.setReducedMotion(!!options.reducedMotion); // (the wind, the clouds.)
    this.characters.reducedMotion = !!options.reducedMotion;
    this.placing.reducedMotion = !!options.reducedMotion;
    // The player's frame rate cap, in the menus too: their town, standing still, is held (`heldFrames.ts`), so a cap
    // of their own saved nothing then, and while its camera moved it judders (30 fps: four refreshes a frame at 120 Hz).
    this.pacer.setCap(this.options.fpsCap);
    this.applyResolution();
  }

  private readonly onResize = () => this.applyResolution();

  /** Sizes the drawing to the canvas, the device pixel ratio and the resolution setting (`shared/resolution.ts`). */
  private applyResolution(): void {
    if (!this.app) return;
    const dpr = window.devicePixelRatio || 1;
    const w = this.cssSize.width * dpr;
    const h = this.cssSize.height * dpr;
    const scale = (this.renderScaleNow = renderScale(h, this.options.resolution));
    this.app.setCanvasResolution(pc.RESOLUTION_FIXED, Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale)));
  }

  stats(): RenderStats {
    return {
      backend: this.device.isWebGPU ? 'WebGPU' : 'WebGL2',
      preset: this.quality.preset,
      fps: this.frameMs > 0 ? 1000 / this.frameMs : 0,
      frameMs: this.frameMs,
      cpuMs: this.cpuMs,
      drawCalls: this.drawCalls,
      holdStillFrames: this.held?.holding ?? false,
      renderWidth: this.device.width,
      renderHeight: this.device.height,
      renderScale: this.renderScaleNow,
      refreshHz: this.pacer.refreshHz,
      refreshDivisor: this.pacer.divisor,
      gpuMs: this.gpuWanted.size ? this.gpuTiming.recentMs : null,
      residentsMs: this.characters?.updateMs ?? 0,
      vram: this.vram(),
    };
  }

  memoryCensus(): MemoryCensus | null {
    return this.app ? memoryCensus(this.app, modelFileCensus(this.app)) : null;
  }

  /** The device's own GPU memory counters, bytes; null before the app exists. */
  private vram(): RenderVram | null {
    // (The engine tracks them on the device (`_vram`); its types leave `app.stats.vram` undeclared.)
    const v = (this.app?.stats as { vram?: Partial<Record<'tex' | 'vb' | 'ib' | 'ub' | 'sb', number>> } | undefined)?.vram;
    if (!v) return null;
    const tex = v.tex ?? 0;
    const geom = (v.vb ?? 0) + (v.ib ?? 0);
    return { tex, geom, total: tex + geom + (v.ub ?? 0) + (v.sb ?? 0) };
  }

  /** Where the birds live in `world` round `view` at `minute`'s hour. */
  private settleBirds(world: WorldStructure, view: ViewRect | null, minute: number): Promise<void> {
    return this.birdLife.settle(world, view, minute, { landscape: this.lot.landscape, street: this.street.plants, roofs: this.roofs, houseTop: (plot) => this.house.roofTop(plot) });
  }

  setFixedHour(hour: number | null): Promise<void> {
    this.fixedHour = hour;
    if (this.townMode) {
      this.lastLightMinute = -1;
      return Promise.resolve();
    }
    // Lit now, not at the next frame's update; the birds settled again as a world built at that hour has them.
    const minute = hour !== null ? hour * 60 : this.lastMinute;
    this.lastMinute = minute;
    this.applyLighting(minute);
    this.lastLightMinute = minute;
    const world = this.lastWorld;
    return world ? this.settleBirds(world, this.view, minute) : Promise.resolve();
  }

  setFrozen(on: boolean): void {
    this.frozen = on;
  }

  addFrameProbe(probe: FrameProbe): () => void {
    this.probes.push(probe);
    // Draws and triangles per pass, counted only while something reads them.
    if (this.probes.length === 1) this.drawCounts.install(this.passKind);
    return () => {
      const i = this.probes.indexOf(probe);
      if (i < 0) return;
      this.probes.splice(i, 1);
      if (!this.probes.length) this.drawCounts.remove();
    };
  }

  measureGpu(on: boolean, who: 'overlay' | 'bench' = 'overlay'): void {
    if (on) this.gpuWanted.add(who);
    else this.gpuWanted.delete(who);
    this.gpuTiming.enabled = this.gpuWanted.size > 0;
  }

  gpuPasses(reset = false): { name: string; ms: number; frames: number }[] {
    return this.gpuTiming.passes(reset);
  }

  /** The picture's multisampling on this device (`QualitySettings.msaaSamples`, `webglMsaaSamples` on WebGL2). */
  private get samples(): number {
    return this.device.isWebGPU ? this.quality.msaaSamples : this.quality.webglMsaaSamples;
  }

  /** The sun's shadow map's target (null: none made, or the sun casts none). */
  private shadowTarget(): pc.RenderTarget | null {
    const light = this.sun?.light;
    // (PlayCanvas types the scene light's `shadowMap` as any: a `ShadowMap`, null before the first shadow pass.)
    const map: { renderTargets: pc.RenderTarget[] } | null = light?.castShadows ? light.light.shadowMap : null;
    return map?.renderTargets[0] ?? null;
  }

  /** What a pass into `target` draws, for the benchmark's counts (`DrawCounts`). */
  private readonly passKind = (target: pc.RenderTarget | null): PassKind => (!target ? 0 : this.mirrors.owns(target) ? 2 : target === this.shadowTarget() ? 1 : 0);

  /** The GPU timer's name for a pass into `target` (`GpuTiming`): the target's, as the bench's results name them. */
  private readonly passLabel = (target: pc.RenderTarget | null): string => {
    if (!target) return 'canvas';
    if (target === this.shadowTarget()) return 'shadow';
    const post = this.post?.targets;
    if (target === post?.scene) return 'scene';
    if (target === post?.fxaa) return 'imageProcessing';
    return target.name;
  };

  benchWorld(): WorldStructure | null {
    return this.lastWorld;
  }

  /**
   * What the benchmark reports of the renderer (`debug/bench.ts`), its fields meaning what they mean
   * in the bench's results up to 0.3.2, so runs compare (see `gpuTiming.ts` for the measures):
   *
   * - `passes`: the targets a frame draws into, with their pixels, samples and bytes per sample: the
   *   sun's shadow map (while it has casters, drawn every frame or not), each mirror reflecting now
   *   (held ones too), the scene's target, then the image processing onto the canvas (on Low into the
   *   FXAA pass's input, then the FXAA onto the canvas), under those results' names. The canvas
   *   counts 4 bytes of colour and 4 of depth. The one difference: the shadow map is depth alone,
   *   where up to 0.3.2 it had a 2-byte colour target too.
   * - `scene`: counts of what is drawn now. `meshes` are the draws (mesh instances) on the
   *   composition's layers; up to 0.3.2 they counted meshes (hidden templates too), so those counts
   *   don't compare with these, only runs since (the bench's fingerprint: equal counts, the same
   *   scene). `materials`: theirs, distinct. `shadowCasters`: the draws that cast in the sun's layers,
   *   shown, and `shadowCasterTriangles` their triangles × instances. `natureInstances`: the
   *   landscape's plant and rock instances the game camera's layer has (the woods' shadow-only copies
   *   left out). `residents`, `mirrors`: how many. `shaders`, `pipelines`: made so far (`ShaderWatch`;
   *   none should be made in play).
   */
  benchInfo(): Record<string, unknown> & { passes: RenderPassInfo[] } {
    const d = this.device;
    const passes: RenderPassInfo[] = [];
    const push = (name: string, t: pc.RenderTarget) => passes.push({ name, width: t.width, height: t.height, samples: Math.max(1, t.samples), colorBytes: colorBytes(t), depthBytes: depthBytes(t) });
    const canvas = (name: string) => passes.push({ name, width: d.width, height: d.height, samples: 1, colorBytes: 4, depthBytes: 4 });
    const scene = this.sceneCounts();
    const map = this.shadowTarget();
    if (map && scene.shadowCasters > 0) push('shadow', map);
    this.mirrors.reflecting.forEach((t, i) => push(`mirror${i}`, t));
    const targets = this.post?.targets;
    if (targets?.scene) {
      push('scene', targets.scene);
      if (targets.fxaa) {
        push('imageProcessing', targets.fxaa);
        canvas('fxaa (canvas)');
      } else canvas('imageProcessing (canvas)');
    } else canvas('scene (canvas)');
    const info = deviceDriverInfo(d);
    const adapter = info.gpuAdapter?.info;
    return {
      engine: `PlayCanvas ${pc.version}`,
      backend: d.isWebGPU ? 'WebGPU' : 'WebGL2',
      gpu: adapter ? { vendor: adapter.vendor, architecture: adapter.architecture, device: adapter.device, description: adapter.description } : { vendor: info.unmaskedVendor, renderer: info.unmaskedRenderer },
      timerQuery: this.gpuTiming.available,
      render: { width: d.width, height: d.height, renderScale: this.renderScaleNow, devicePixelRatio: window.devicePixelRatio, canvasMsaa: false },
      quality: { ...this.quality, mirrors: this.mirrors.capacity, clusteredLamps: true, postSamples: this.post?.samples ?? 1, fxaa: this.post?.options.fxaa ?? false },
      diag: [...this.diag],
      passes,
      // The town drawn (the bench's roam loops over it) and the street's chunks.
      world: this.builtWorld ? { width: this.builtWorld.width, depth: groundDepth(this.builtWorld), size: this.builtWorld.meta?.world?.size ?? 'cosy', areas: this.builtWorld.meta?.areas?.map((a) => [a.x + a.w / 2, a.z + a.d / 2]) } : undefined,
      street: this.street.stats(),
      focusSwitches: [...this.focusSwitches],
      // What is being built now (the bench notes it at each slow frame): a world (a focus lot), the street streaming, both or nothing.
      building: this.building(),
      scene: {
        ...scene,
        textures: d.textures.size,
        lights: scene.lights,
        residents: this.simCount,
        residentsDrawn: this.residentsDrawn(),
        mirrors: this.mirrors.count,
        mirrorsLive: this.mirrors.live,
        mirrorSpots: this.mirrors.spots,
        mirrorDraws: this.mirrors.draws,
        sunShadowDraws: ((d) => ({ whole: d.whole, moving: d.moving, why: { ...d.why }, by: Object.fromEntries(d.by) }))(this.sunShadows.draws),
        nightGrade: this.post?.night ?? 0,
        lightingUpdates: this.counts.lightingUpdates,
        shaders: this.shaderWatch.shaders,
        pipelines: this.shaderWatch.pipelines,
      },
    };
  }

  /** The draws on the composition's layers now, and what of them the benchmark counts (`benchInfo`). */
  private sceneCounts(): { meshes: number; enabledMeshes: number; activeMeshes: number; materials: number; lights: number; lampsLit: number; shadowCasters: number; shadowCasterTriangles: number; natureMeshes: number; natureInstances: number } {
    const layers = this.app.scene.layers.layerList;
    const all = new Set<pc.MeshInstance>();
    for (const layer of layers) for (const mi of layer.meshInstances) all.add(mi);
    const triangles = (mi: pc.MeshInstance) => ((mi.mesh?.primitive[0]?.count ?? 0) / 3) * (mi.instancingData ? mi.instancingCount : 1);
    const shown = (mi: pc.MeshInstance) => mi.visible && mi.node.enabled && (!mi.instancingData || mi.instancingCount > 0);
    const materials = new Set<pc.Material>();
    let enabled = 0;
    let active = 0;
    for (const mi of all) {
      if (mi.material) materials.add(mi.material);
      if (shown(mi)) enabled++;
      if (mi.visibleThisFrame) active++;
    }
    // The sun's casters: in its layers, casting, shown.
    const sun = this.sun.light;
    const casters = new Set<pc.MeshInstance>();
    if (sun?.castShadows) {
      for (const id of sun.layers) {
        const layer = this.app.scene.layers.getLayerById(id);
        if (layer) for (const mi of layer.meshInstances) if (mi.castShadow && shown(mi)) casters.add(mi);
      }
    }
    let casterTriangles = 0;
    for (const mi of casters) casterTriangles += triangles(mi);
    // The landscape's instances in the game camera's world layer.
    const world = new Set(this.app.scene.layers.getLayerById(pc.LAYERID_WORLD)?.meshInstances ?? []);
    let natureMeshes = 0;
    let natureInstances = 0;
    for (const mi of this.lot.landscapeDraws()) {
      if (!world.has(mi) || !mi.node.enabled) continue;
      natureMeshes++;
      if (mi.instancingData) natureInstances += mi.instancingCount;
    }
    const lights = this.app.root.findComponents('light') as pc.LightComponent[];
    const on = lights.filter((l) => l.enabled && l.entity.enabled);
    return {
      meshes: all.size,
      enabledMeshes: enabled,
      activeMeshes: active,
      materials: materials.size,
      lights: on.length,
      lampsLit: on.filter((l) => l.type !== 'directional' && l.intensity > 0).length,
      shadowCasters: casters.size,
      shadowCasterTriangles: Math.round(casterTriangles),
      natureMeshes,
      natureInstances,
    };
  }

  /** Residents drawn now (the benchmark's). */
  private residentsDrawn(): number {
    let n = 0;
    for (let i = 0; i < this.simCount; i++) if (this.characters.visible[i]) n++;
    return n;
  }

  // (the save's picture, `thumbnail.ts`; made at the first, gone with the app.)
  private thumbnails: Thumbnails | null = null;

  captureThumbnail(width: number, height: number): Promise<string | null> {
    if (this.disposed || !this.app) return Promise.resolve(null);
    this.thumbnails ??= new Thumbnails(this.app, this.camera.camera, this.post, { wantFrame: () => void this.framesRendered(1), shadowsMoved: () => this.sunShadows.moved() });
    return this.thumbnails.capture(width, height);
  }

  // --- residents, birds, voices ------------------------------------------------------

  /** The residents walking on the ground in view push the grass aside, the nearest the eye first (`dt` ms); none in the menus' town (those pushing ease out). */
  private pushGrass(dt: number): void {
    const push = this.grassPush;
    const eye = this.camera.eyeInGame(this.eyeTmp);
    const head = this.grassHead;
    push.begin();
    for (let i = 0; i < (this.townMode ? 0 : this.simCount); i++) {
      // (Ranked by how far they are from the eye as drawn, on whatever ground they stand.)
      if (!this.characters?.onGround(i) || !this.characters.head(i, head)) continue;
      push.add(i, head.x, head.z, Math.hypot(head.x - eye.x, head.y - eye.y, head.z - eye.z));
    }
    this.lot.pushGrass(push.end(dt / 1000));
  }

  simSpot(index: number, out: { x: number; z: number }): boolean {
    const f = this.lastFrame;
    if (!f || index >= this.simCount) return false;
    const o = f.layout.headerLen + index * f.layout.simStride;
    if (f.curr[o + f.layout.sim.away] !== 0) return false;
    const z = f.curr[o + f.layout.sim.z];
    out.x = f.curr[o + f.layout.sim.x];
    // (Upstairs rows back to the ground's.)
    out.z = z - this.rowOf(this.storeyOf(Math.floor(z)));
    return true;
  }

  heightAt(x: number, z: number): number {
    return this.groundY(x, z);
  }

  simHead(index: number, out: { x: number; y: number; z: number }): boolean {
    if (!(index < this.simCount && !!this.characters?.head(index, out))) return false;
    out.y -= this.datum;
    return true;
  }

  /** The ground (m) at (`x`, `z`) relative to the datum: the engine's height of what stands there (the landscape's `heightAt`). */
  private groundY(x: number, z: number): number {
    return (this.lot.landscape?.heightAt(x, z) ?? 0) - this.datum;
  }

  /**
   * Where the game camera's target stands (m, relative to the datum): on a land drawn from a height field (a land the
   * town follows, or one the brushes shaped) the ground under it, so it never aims into a hill; elsewhere at 0, as ever.
   */
  private targetGround(x: number, z: number): number {
    return this.lot.landscape?.heightField ? this.groundY(x, z) : 0;
  }

  /**
   * The datum taken on (`d`, game space): the world root lowered by it and the viewed lot's layers raised (outside the
   * menus' town, which draws at 0), and the camera's target shifted by the old datum less the new one in the same frame,
   * so nothing jumps (it then eases to the ground under it: `settleTarget`).
   */
  private applyDatum(d: number): void {
    if (d === this.datum) return;
    this.orbit.target.y += this.datum - d;
    this.datum = d;
    if (!this.townMode) {
      setDatum(this.root, this.lotRoot, d);
      this.lot.datum = d;
    }
    this.sunShadows.moved();
  }

  /** The game camera's target eased to the ground under it (`targetGround`; a quarter of a second): no jump as it crosses a step. */
  private settleTarget(dt: number): void {
    if (!this.lot.landscape?.heightField) return;
    const t = this.orbit.target;
    const goal = this.targetGround(t.x, t.z);
    if (Math.abs(goal - t.y) < 1e-4) return;
    t.y += (goal - t.y) * (1 - Math.exp(-dt / TARGET_EASE));
  }

  gust(x: number, z: number): number {
    return this.app ? windGust(this.app, x, z) : 0;
  }

  simStride(index: number, out: SimStride): boolean {
    return index < this.simCount && !!this.characters?.stride(index, out);
  }

  birds(): readonly SeenBird[] {
    return this.birdLife?.heard() ?? []; // (none behind the overview.)
  }

  birdSings(id: number, seconds: number): void {
    this.birdLife?.sings(id, seconds);
  }

  doorSounds(): readonly DoorSound[] {
    const heard = this.doorsHeard;
    this.doorsHeard = this.doorsHanded;
    this.doorsHeard.length = 0;
    this.doorsHanded = heard;
    return heard;
  }
  setSelectedSim(id: number | null): void {
    this.pictureChanged();
    this.selectedSim = id;
    this.characters?.select(id);
  }

  setSpeech(index: number, track: VisemeTrack | null, startTime: number): void {
    this.pictureChanged();
    this.characters?.setSpeech(index, track, startTime);
  }

  // --- build and buy -----------------------------------------------------------------

  setHoverTile(tile: { x: number; z: number } | null): void {
    this.pictureChanged();
    this.hoverTile.set(tile);
  }

  setWallMode(mode: WallMode): void {
    this.pictureChanged();
    this.wallMode = mode;
    this.applyWallMode();
  }

  setStorey(storey: number): void {
    const k = Math.max(this.storeys.lowest, Math.min(this.storeys.count - 1, Math.round(storey)));
    if (k === this.storey) return;
    this.storey = k;
    this.pictureChanged();
    this.storeyLayer?.setLocalPosition(0, k * WALL_HEIGHT, -this.rowOf(k));
    this.applyStoreys();
    // The lights go to the storey now in view (also while a build is running, which skips the rebuild below).
    if (this.lastWorld) this.placeRoomLights(this.lastWorld, this.view);
    // Furniture and residents above it hide; the lot (every storey) is kept.
    if (this.lastWorld && !this.worldBuilding) void this.setWorld(this.lastWorld, this.view);
  }
  // (placing things, `placing.ts`.)
  setPlacementGhost(ghost: PlacementGhost | null): void {
    this.pictureChanged();
    this.placing.setPlacementGhost(ghost);
  }

  setHeldObject(id: number | null, moved = false): void {
    this.placing.setHeldObject(id, moved);
  }

  // (Build mode's previews and overlays, `buildPreviews.ts`.)
  setEdgePreview(edges: readonly EdgePreview[], valid: boolean): void {
    this.pictureChanged();
    this.previews.setEdgePreview(edges, valid);
  }

  setPaintPreview(faces: readonly PaintPreviewFace[], cover: number | null): void {
    this.pictureChanged();
    this.previews.setPaintPreview(faces, cover);
  }

  setFloorPreview(tiles: readonly { x: number; z: number }[], floor: number | null): void {
    this.pictureChanged();
    this.previews.setFloorPreview(tiles, floor);
  }

  setGaragePreview(garage: GaragePreview | null): void {
    this.pictureChanged();
    this.previews.setGaragePreview(garage);
  }

  setPorchPreview(porch: PorchPreview | null): void {
    this.pictureChanged();
    this.previews.setPorchPreview(porch);
  }

  setRoomOverlay(tiles: readonly RoomOverlayTile[]): void {
    this.pictureChanged();
    this.previews.setRoomOverlay(tiles);
  }

  setBuildGrid(rect: ViewRect | null): void {
    this.pictureChanged();
    this.previews.setBuildGrid(rect);
  }

  buildEffect(fx: BuildEffect): void {
    this.pictureChanged();
    this.placing.buildEffect(fx);
  }

  // --- dev tools (`RendererDebugHooks`) ------------------------------------------------

  // (lotContents.ts.)
  objectClips(id: number): ReadonlyMap<string, ObjectClipInfo> | null {
    return this.lot.objectClips(id);
  }

  objectMotions(id: number): ObjectBackgroundClips | null {
    return this.lot.objectMotions(id);
  }

  objectPoints(id: number, max?: number): Float32Array | null {
    return this.lot.objectPoints(id, max);
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    clearTimeout(this.prepareTimer);
    this.parseQueue = [];
    this.characters?.dispose();
    this.birdLife?.dispose();
    this.simShadow?.destroy();
    this.marker?.destroy();
    this.handles?.destroy();
    this.hoverTile?.destroy();
    this.landCursor?.destroy();
    this.useDraw?.destroy();
    this.ghostDraw?.destroy();
    this.previews?.destroy();
    this.placing?.dispose();
    this.modelRefresh.dispose();
    this.lot?.dispose();
    this.mirrors?.destroy();
    this.held?.destroy();
    this.setActive(false);
    this.onFailure = null;
    this.onFrame = null;
    this.onView = null;
    this.townListener = null;
    if (townView.project === this.projectTown) Object.assign(townView, { project: null, blend: 0 });
    this.glide = null;
    this.detachOrbit?.();
    this.detachOrbit = null;
    this.sizeWatch?.disconnect();
    this.unwatchPixelRatio?.();
    window.removeEventListener('resize', this.onResize);
    if (window.__renderer === this) {
      delete window.__renderer;
      delete window.__scene;
      delete window.__app;
    }
    for (const w of this.frameWaiters.splice(0)) w.resolve();
    this.unlistenFiles?.();
    this.unlistenFiles = null;
    this.overview?.destroy();
    this.landHaze?.destroy();
    this.worldHaze?.destroy();
    this.street?.destroy();
    this.townBuildings?.destroy();
    this.townMovers?.destroy();
    this.townGround?.dispose();
    this.cars?.destroy();
    this.signals?.destroy();
    this.gardenPools?.destroy();
    this.sunShadows?.destroy();
    for (const p of this.decor.values()) for (const d of p.draws) d.destroy();
    this.decor.clear();
    this.post?.destroy();
    this.post = null;
    this.drawCounts?.remove();
    this.gpuTiming?.destroy();
    this.samplerCheck?.destroy();
    this.shaderWatch?.destroy();
    if (this.app) destroyApp(this.app);
  }
}
