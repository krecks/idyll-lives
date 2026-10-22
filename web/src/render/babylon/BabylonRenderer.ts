/**
 * Babylon.js implementation of the Renderer contract.
 *
 * Performance model:
 * - WebGPU first (WebGL2 fallback). With WebGPU, snapshot rendering records the frame's
 *   draw commands once and replays them; it is re-recorded only on structural changes.
 * - Every object type is one mesh drawn with thin instances (one draw call per type).
 * - Sims are skinned characters (characters.ts): thin instances per body part, GPU skinning
 *   from one pose texture that is rewritten each frame (no new draw commands).
 * - The viewed house is generated into one mesh per material (house.ts); the cutaway is done
 *   in the vertex shader from a per-vertex attribute.
 * - No allocations in `update()`.
 *
 * Look: `look.ts` drives lighting keys, sky, fog, post-processing and colour grading.
 */

import {
  ArcRotateCamera,
  type ArcRotateCameraKeyboardMoveInput,
  type ArcRotateCameraPointersInput,
  type Camera,
  Color3,
  Color4,
  ColorCurves,
  Effect,
  Constants,
  DefaultRenderingPipeline,
  DirectionalLight,
  DynamicTexture,
  Engine,
  HemisphericLight,
  ImageProcessingConfiguration,
  Light,
  Material,
  Matrix,
  Mesh,
  MirrorTexture,
  MeshBuilder,
  PBRMaterial,
  Plane,
  SpotLight,
  ClusteredLightContainer,
  Quaternion,
  Scene,
  SceneInstrumentation,
  CreateScreenshotAsync,
  ShadowGenerator,
  StandardMaterial,
  TransformNode,
  Vector3,
  VertexBuffer,
  VertexData,
  Viewport,
  WebGPUEngine,
  WebGPUPerfCounter,
  type AbstractEngine,
  type AbstractMesh,
  type PerformanceMonitor,
  type RenderTargetTexture,
  type RenderTargetWrapper,
} from './core';
import { benchDiag } from '../../debug/benchFlags';
import { perfEnd, perfPhase, perfRecord, perfStart, perfTime, perfTimeAsync } from '../../debug/perf';
import type { FrameState } from '../../core/bridge';
import { Pose } from '../../core/snapshot';
import { colorsKey, lowestStorey, objectCentre, storeyOfRow, storeyRow, storeyShown, type ItemPlacement, type StoreyLayout, type DiagonalWall, type FenceEdge, type MeshArrays, type ObjectPlacement, type Opening, type PartColors, type WallEdge, type WorldStructure } from '../../core/protocol';
import type { RenderQuality } from '../quality';
import type { VisemeTrack } from '../../voice/visemes';
import type { ObjectDef } from '../../content/content';
import { LOOK } from '../look';
import type {
  BuildEffect,
  CameraPose,
  EdgePreview,
  FrameCounts,
  FrameProbe,
  GameShot,
  LiveRenderOptions,
  PaintPreviewFace,
  RoomOverlayTile,
  LotHighlight,
  PickResult,
  ObjectResize,
  RenderPassInfo,
  PlacementGhost,
  ResizeHandle,
  Renderer,
  RendererDeps,
  RenderFailure,
  RenderStats,
  SeenBird,
  Shading,
  TownShot,
  ViewRect,
  WallMode,
} from '../types';
import { resizeKey } from '../types';
import { createLighting, lightingAt } from './environment';
import { buildFences, swingGate, type GateLeaf } from './fences';
import { applyShaderFixes } from './shaderFixes';
import { patchRoomLights, type RoomMask } from './roomLights';
import { HALF_WALL_HEIGHT, HouseBuilder, houseObjectKey, placeLights, WALL_HALF, WALL_HEIGHT, WALL_STUB, type DoorLeaf, type HouseSurfaces, type RoomLight, type RoomTiles } from './house';
import { fieldLayers, Street } from './street';
import { MaterialLibrary, WINDOWLESS_BITS, type DarkTiles } from './materials';
import { refreshSoftLook, shadingFromSettings } from './softLook';
import { Geo, LOOK as CUT } from './geometry';
import { buildModel, motionsOf, onModelFileArrived, paintColors, painted, placementVariation, prefetchModels, standInOf, withGlows, type ModelTemplate, type StandIn } from './models';
import { TemplateCache } from './templateCache';
import { compressedTextureFeatures, noteWebGpuFailed, webGpuFailed } from './loaders';
import { Characters, GAME_RATE, MAX_CHARACTERS } from './characters';
import { ItemLayer } from './items';
import { Mirrors, type MirrorInstance } from './mirrors';
import { ResizeHandles } from './handles';
import { installNature, Landscape, natureDecor, setWind, Sky } from './nature';
import { Birds, type PlantPlacement } from './birds';
import { LAYER_ALL, LAYER_TOWN, LAYER_WORLD } from './layers';
import { NightGrade } from './nightGrade';
import { BuildEffects, writeMoving, type PopTarget } from './buildFx';
import { tableRooms } from './tableTuck';
import { gardenPool, GardenPools } from './gardenPools';
import { ObjectMotion, restPoses, slotOf, type MovingObject } from './objectMotion';
import { TownOverview } from './overview';
import { GameCameraRig } from './cameraRig';
import { renderScale, watchPixelRatio } from './resolution';
import { FramePacer } from './pacing';
import { SunShadows } from './sunShadows';

const ACCENT = Color3.FromHexString('#5B7CFA');
/** Helpers are parked below the ground instead of toggling visibility (keeps snapshots valid). */
const HIDDEN_Y = -100;
/** The basement's pit (and the ground dug away over it) reaches this far past the plot (m), clear of walls on its edge. */
const PIT_MARGIN = 0.3;
/** Height of the fence and gate preview while drawing. */
const FENCE_PREVIEW_HEIGHT = 1.0;
/**
 * Frames per second at most for the menus' town: it moves slowly, and drawn at the display's rate
 * it kept the main thread as busy as play (87 % at 3× CPU slow-down, P2-08). Paced like the cap
 * (`pacing.ts`): 30 on 60 and 120 Hz displays, 36 on 144 Hz.
 */
const MENU_FPS = 30;
/** Game minutes between lighting updates (with / without snapshot rendering; at speed 3 and up and in quiet hours, the quality's `lightingStep`). */
const LIGHT_STEP = 0.5;
const LIGHT_STEP_SNAPSHOT = 5;
/**
 * And real milliseconds at least: at the fastest speeds the minutes passed every snapshot (20 a
 * second), and each update can thaw the materials for a few frames. At normal speed the steps are
 * 500 ms apart anyway.
 */
const LIGHT_MIN_MS = 100;
/**
 * Metres of the game's sun shadow map beyond the lot's corners (nearby trees and the street
 * still shade it), and the outer share of it over which shadows fade out.
 */
const SHADOW_MARGIN = 10;
const SHADOW_EDGE_FADE = 0.15;
/**
 * One body mesh's instances in the merged object shadow mesh (Low): its vertices (`pos`, local) and
 * world matrix, its instances' matrices, and the merged vertex its first instance starts at.
 */
type ShadowPart = { pos: ArrayLike<number>; world: Matrix; matrices: Float32Array; first: number };
/** A body mesh to merge into the object shadow mesh, and its instances' matrices. */
type ShadowSource = { mesh: Mesh; matrices: Float32Array };
/**
 * Interior lights: the household's lamps first, then dim fills for rooms without one. With
 * clustered lighting (WebGPU, and WebGL2 with float blending) they live in one
 * `ClusteredLightContainer` (one light to the materials) and many lamps light at once
 * (quality `clusteredLamps`); otherwise each is a forward light and only a few fit.
 * `?lights=forward|clustered` forces one for testing.
 */
const ROOM_LIGHTS = 4;
const CLUSTERED_LIGHTS = 32;
/** Range (m) of an interior light while it is dark: it then covers next to no cluster tiles (see `applyRoomLights`). */
const DARK_LAMP_RANGE = 0.01;
/** Brightness of a lamp at night (times its content `intensity`). */
const LAMP_POWER = 7;
/**
 * A room light's cone: a whole turn for a lamp lighting all round (the cluster takes a cone whose
 * half-angle's cosine is below 0 as a point light; the glTF falloff, with the inner cone as wide,
 * as no cone), half a turn for one whose shade sends it one way (`RoomLight.dir`): full along it,
 * about half 30–45° off it and next to none side-on.
 */
const ALL_ROUND = Math.PI * 2;
const SHADED = Math.PI;
/**
 * Furthest reach (m) the lamp cluster bins: the largest bought lamp's range. Babylon's own default is
 * 16 km, which puts every lamp in every tile it can see, so each PBR fragment loops over far more
 * lamps than can light it. Room fill lights may reach further; their glow stops at this range.
 */
const LAMP_MAX_RANGE = 7;
/** A room without a lamp gets this share of the light it had before lamps gave light. */
const FILL_POWER = 0.35;
/**
 * A basement in view gets no sun, only this much of the sky fill and the environment (as seen,
 * intensity times exposure: the same by day and by night), and its lamps on as at night.
 */
const BASEMENT_SKY = 0.12;
const BASEMENT_ENV = 0.12;
const NIGHT_EXPOSURE = Math.max(...LOOK.keys.map((k) => k.exposure));
/**
 * Seconds a door or a gate takes to swing open or shut at speed 1 (quicker as the game runs
 * faster, up to 10x, as walkers' turns are). It opens for someone whose way crosses its doorway
 * within `DOOR_AHEAD` metres (sim-core `snapshot::AHEAD`: about a second's walk), or who stands
 * less than `DOORWAY` metres either side of it; open, it waits to shut until nobody walking is
 * within `DOOR_CLEAR` of it (a leaf is 0.83 m long).
 */
const DOOR_SWING = 0.45;
const DOOR_AHEAD = 1.5;
const DOORWAY = 0.35;
const DOOR_CLEAR = 1.1;
/** Floats per resident in `movers`: where they are, the waypoint they walk to, where their way is `DOOR_AHEAD` on. */
const MOVER = 6;
/** Seconds over which a room's lights go out (or come back on) as its residents fall asleep (or wake). */
const LIGHTS_OUT_FADE = 1.2;
/** Seconds a lamp takes to go off or on at its switch. */
const LAMP_SWITCH_FADE = 0.25;
/** Camera distance below which the viewed lot's roof is hidden even with walls up. */
const ROOF_MIN_DISTANCE = 15;
/** Capacity of the wall-tool preview buffers (grown on demand). */
const PREVIEW_CAPACITY = 128;
/**
 * How long (ms) after a failed model file comes the world is built again with it: the wait starts
 * over with each file that comes (after an outage many come one after another, for one rebuild),
 * but lasts at most `FILES_WAIT_MS` from the first.
 */
const FILES_SETTLE_MS = 1500;
const FILES_WAIT_MS = 5000;

interface PlacedObject {
  id: number;
  /** Where it stands, on the storey it's on (`y0`: that storey's floor). */
  y0: number;
  minX: number;
  minZ: number;
  maxX: number;
  maxZ: number;
  height: number;
  /** Bottom of the model above `y0` (things on a wall or the ceiling hang higher). */
  bottom: number;
  /** Things on a wall: the cutaway look bits that drop their wall (`wallCutBits`), to hide them with it. */
  wallCut?: number;
  /** Its instance in the template's matrix buffer (for the placement pop). */
  pop: PopTarget;
  /** Its contact shadow's instance. */
  shadow?: PopTarget;
  /** Things on a top: the object whose top it is (they're in hand with it). */
  host?: number;
}

const MAX_SIMS = MAX_CHARACTERS;

/**
 * Babylon disposes an effect when the last submesh that used it is disposed, even though the engine
 * caches effects by shader and defines and the next world reuses the same variant. The mirror pass
 * compiles a clip-plane variant of every material it reflects, and those are the expensive ones to
 * lose when a household's world is torn down. Keep them alive: the cache entry is still there, so
 * the next world's first reflection reuses it instead of recompiling.
 *
 * Patching the prototype from `init` would close over that renderer and pin it (and its engine) for
 * the life of the page, so the wrapper lives out here where it captures only the original method.
 */
function keepClipPlaneEffects(): void {
  // SAFETY: `keepsClipPlanes` is our own marker on Babylon's runtime Effect class; its typings do not know it.
  const proto = Effect.prototype as unknown as { keepsClipPlanes?: boolean };
  if (proto.keepsClipPlanes) return;
  proto.keepsClipPlanes = true;
  const dispose = Effect.prototype.dispose;
  Effect.prototype.dispose = function (this: Effect, ...args: Parameters<typeof dispose>) {
    if (/^#define CLIPPLANE\d*\b/m.test(this.defines)) return;
    return dispose.apply(this, args);
  };
}

/** How far the road (and garden paths) is drawn beyond the viewed lot: the whole street. */
const STREET_MARGIN = 500;
/** Sims are drawn this far outside the lot (e.g. walking up the garden path). */
const SIM_VIEW_MARGIN = 2;
/** Selection marker colour slot (0 = great .. 4 = bad) per emotion id; no emotion = slot 1. */
const EMOTION_MOOD: Record<string, number> = { happy: 0, flirty: 0, confident: 0, focused: 0, inspired: 0, energized: 0, relaxed: 0, embarrassed: 3, sad: 3, tense: 3, angry: 4 };
/** Game camera zoom limits (the overview lifts them while it drives the camera). */
const RADIUS_LIMITS = [6, 95] as const;
/** Frames in which a game world built behind the overview is also drawn (hidden), so its shaders and pipelines are ready when it is shown. */
const WARM_WORLD_FRAMES = 4;
/** Longest wait for a hidden world's textures before it is warmed up anyway (ms). */
const TEXTURE_WAIT_MS = 5000;
/** Main-thread time (ms) a world build behind the overview runs before the menus' town draws a frame. */
const BUILD_SLICE_MS = 8;
/** Most frames a landscape's plants built again wait for their shaders before they're swapped in anyway (`meshesReady`). */
const MESH_READY_FRAMES = 90;
/** Frames in a row that may fail to draw before drawing counts as broken (`onFailure`). */
const FAILED_FRAMES_LIMIT = 60;
/** Longest wait for `framesRendered` (drawing stopped, a hidden tab): then it goes on without them. */
const FRAME_WAIT_LIMIT_MS = 15_000;
/** Longest frame time (ms) things move by in one frame (the first frame back after a hidden tab spans all of it). */
const MAX_FRAME_MS = 100;
/** Frames still drawn once a full-screen menu covers the view (see `hold`): the paused scene, settled, under its glass. */
const HOLD_FRAMES = 2;

/** A landscape, shared by the game world and the overview when they are the same town. */
interface NatureSet {
  /** Its town's (`Landscape.key`). */
  key: string;
  /** Kept to refresh the lawn when rooms or objects change (build mode), and to build plants again when a file comes. */
  landscape: Landscape;
  meshes: Mesh[];
  casters: Mesh[];
  only: Map<Mesh, 'game' | 'town'>;
  ready: Promise<void>;
  users: Set<'world' | 'town'>;
  pending: number;
  /** Its plants being built again with the files that came (`refreshNature`), one file after another. */
  refreshing: Promise<void>;
  /** Those plants, built and not shown yet. */
  coming: Mesh[];
}

export class BabylonRenderer implements Renderer {
  private engine!: AbstractEngine;
  private webgpu: WebGPUEngine | null = null;
  private scene!: Scene;
  private camera!: ArcRotateCamera;
  private sun!: DirectionalLight;
  private sky!: HemisphericLight;
  private shadows!: ShadowGenerator;
  /** Which casters the shadow map draws, and when (see sunShadows.ts). */
  private sunShadows!: SunShadows;
  private post: DefaultRenderingPipeline | null = null;
  private nightGrade: NightGrade | null = null;
  private instrumentation!: SceneInstrumentation;
  /** Main-thread ms per frame, smoothed (see `stats`). */
  private cpuMs = 0;
  private canvas!: HTMLCanvasElement;
  private lib!: MaterialLibrary;
  private house!: HouseBuilder;
  /** Neighbour houses, sidewalks, streetlights, gardens and cars (street.ts). */
  private street!: Street;

  private worldMeshes: Mesh[] = [];
  /**
   * The viewed lot's streets, house and fences: rebuilt only when what they depend on changes
   * (see `lotKey`), so buying, selling and moving furniture rebuild just the objects.
   */
  private lotMeshes: Mesh[] = [];
  private lotKey = '';
  /** Shadow casters of the cached house, and the garden templates its build uses. */
  private lotCasters: Mesh[] = [];
  private lotTemplates = new Set<string>();
  /** Rooms of the cached house (where its lamps' and fills' lights go). */
  private lotRooms: RoomTiles | null = null;
  /** Where coverings lie in the viewed house (null: plain fallback meshes). */
  private lotSurfaces: HouseSurfaces | null = null;
  /** Openings in the ground floor over stairs down to the basement (ground rows), seen from the ground up. */
  private stairwells: ViewRect[] = [];
  /** The viewed lot's garden trees and shrubs (`placeDecor`): hidden while its basement is in view. */
  private readonly decorMeshes = new Set<Mesh>();
  /** Those trees and shrubs as placed (model, [x, z, scale, yaw] each), where birds live. */
  private decorPlaced: [string, [number, number, number, number][]][] = [];
  /** The garden birds (birds.ts). */
  private birdLife!: Birds;
  private readonly birdCamera = new Vector3();
  /** The Paint and Floor tools' previews in the real look: what they show, and the mesh. */
  private laid = {
    faces: { keys: [] as string[], cover: null as number | null, shown: '', mesh: null as Mesh | null },
    floors: { keys: [] as string[], cover: null as number | null, shown: '', mesh: null as Mesh | null },
  };
  /** Meshes the room lights were last told to leave out (the landscape and sky). */
  private lightExcluded = new Set<Mesh>();
  private onFrame: ((now: number) => void) | null = null;
  /** Per drawn frame once the camera has moved (see `run`), and the frame it last ran in. */
  private onView: (() => void) | null = null;
  private viewFrame = -1;
  /** The frame state last posed (the marker goes up once the camera has moved). */
  private lastFrame: FrameState | null = null;
  private active = false;
  /** A full-screen menu covers the view (see `hold`), and the frames still drawn before it stops. */
  private covered = false;
  private coveredFrames = 0;
  /** This frame's time (ms, clamped to `MAX_FRAME_MS`), for everything in the scene that moves with it. */
  private frameDt = 0;
  /** Reduce motion: plants, clouds, the selection marker, the Build ghost and build effects keep still. */
  private reducedMotion = false;
  /** The arrow keys' camera nudges per frame at 60 fps (see `paceKeys`). */
  private keyNudge = { angularSpeed: 0, panningSensibility: 0, zoomingSensibility: 0 };
  /** The watching director's hold on the game camera (see `setGameShot`). */
  private gameRig!: GameCameraRig;
  /** Resident the camera keeps centred while the player turns and zooms (`followSim`). */
  private followIndex: number | null = null;
  private readonly followHead = { x: 0, y: 0, z: 0 };
  private frameWaiters: { left: number; resolve: () => void }[] = [];
  /** Frames in a row that failed to draw (each is caught, so the loop goes on). */
  private failedFrames = 0;
  onFailure: ((why: RenderFailure, error?: unknown) => void) | null = null;
  /**
   * Built model templates by key, kept across lot switches and sessions (building them
   * is expensive). Templates not used by the current world are disabled, not disposed. One built
   * without its file is built again when the file comes (`refreshModels`).
   */
  private templates!: TemplateCache<ModelTemplate>;
  /** Files that failed and have since come, waiting for `refreshModels` (a moment later: several often come together). */
  private readonly filesArrived = new Set<string>();
  private filesTimer: ReturnType<typeof setTimeout> | null = null;
  /** When the first of the files waiting came (`performance.now()`): the wait for more is capped from then. */
  private filesSince = 0;
  private unlistenFiles: (() => void) | null = null;
  /** `dispose` has run: nothing more is built. */
  private disposed = false;
  private readonly templatesInUse = new Set<string>();
  /** Keys of templates built in a player's colours: once no world uses one, it is disposed rather than kept (a click through the palette builds one per swatch). */
  private readonly paintedTemplates = new Set<string>();
  /** Landscapes by town (seed and size), and which one each layer shows. */
  private readonly natures = new Map<string, NatureSet>();
  private readonly natureIn: Record<'world' | 'town', NatureSet | null> = { world: null, town: null };
  private readonly natureTicket = { world: 0, town: 0 };

  // Town overview (menus), on its own camera layer; see `showTown`.
  private overview: TownOverview | null = null;
  private townMode = false;
  /** The camera is flying from the overview into the game (drawn at the game's frame rate). */
  private flyingIn = false;
  /** Overview lighting: the minute shown and the one it eases towards. */
  private townMinute = 18 * 60;
  private townMinuteGoal = 18 * 60;
  private townShot: TownShot | null = null;
  private townHighlight: LotHighlight | null = null;
  private townListener: (() => void) | null = null;
  private townTicket = 0;
  private townQueue: Promise<void> = Promise.resolve();
  /** The town the overview was last asked to show (built again when a file it was drawn without comes). */
  private townWorld: WorldStructure | null = null;
  /** Holds the game camera's pose while the overview drives the real camera. */
  private parked!: ArcRotateCamera;
  private warmFrames = 0;
  /** The game's own look has been warmed once (the shaders are shared by the households). */
  private gameWarmed = false;
  private mirrorWarmed = false;
  private skyMirrorWarmed = false;
  private worldBuilding = false;
  /** When the running slice of a world build began (see `pace`). */
  private sliceStart = 0;
  /** World/overview builds in progress; snapshot recording waits until there are none. */
  private builds = 0;
  /** Landscapes' plants being built again in play (`rebuildNature`): they're paced too. */
  private refreshes = 0;
  private snapshotGeneration = 0;
  /** A snapshot recording to start, or to make again, before the next frame draws (see `applySnapshotDue`). */
  private snapshotDue: 'record' | 'reset' | null = null;
  /** Drawing to do in the next frame, before the scene is drawn (see `warmGame`). */
  private beforeDraw: (() => void) | null = null;
  private readonly warmPose = { alpha: 0, beta: 0, radius: 0, x: 0, y: 0, z: 0, ox: 0, oy: 0 };
  /** Lot being shown; null = everything. */
  private view: ViewRect | null = null;
  /** Fallback walls (simulation meshes) when the house builder can't read the layout. */
  private walls: Mesh | null = null;
  private wallsLow: Mesh | null = null;
  private roofs: Mesh[] = [];
  private roofShown = false;
  /** Shadow casters without the house walls, and the walls (they cast only with walls up). */
  private baseCasters: Mesh[] = [];
  private wallCasters: Mesh[] = [];
  /** Some object's moving parts cast sun shadows of their own (on Low only see-through ones do): their motion redraws the map. */
  private movingPartsCast = false;
  /** The low preset's one static shadow caster for the lot's objects (null on Medium and High). */
  private objectShadowMerge: Mesh | null = null;
  /** `objectShadowMerge`'s material, one for the renderer (the world's meshes go without theirs). */
  private objectShadowMaterial: StandardMaterial | null = null;
  /** Each body mesh's instances in `objectShadowMerge`, by their matrix buffer: baked again as an instance moves or hides (`writeInstance`). */
  private readonly objectShadowParts = new Map<Float32Array, ShadowPart[]>();
  /** `objectShadowMerge`'s vertex positions (world space). */
  private objectShadowPositions = new Float32Array(0);
  private readonly shadowInstance = new Matrix();
  private wallMode: WallMode = 'down';
  /** Camera look bits for the cutaway (+x, -x, +z, -z). */
  private cutLook = -1;
  private placed: PlacedObject[] = [];
  private characters!: Characters;
  /** Food and dishes (`items.ts`). */
  private items!: ItemLayer;
  /** Live reflections in the mirrors nearest the camera. */
  private mirrors!: Mirrors;
  /** Resize arrows around what's picked in Buy and Build mode. */
  private handles!: ResizeHandles;
  private cursor!: Mesh;
  private selectedSim: number | null = null;
  private cameraPlaced = false;
  /** Lot the camera was last turned towards (see `buildSims`). */
  private framedView = '';

  private options: LiveRenderOptions = { resolution: 'auto', cameraSensitivity: 1, fpsCap: 60 };
  /** The shading the scene's materials were prepared for (`softLook.ts`); the setting's at start. */
  private shading: Shading = shadingFromSettings();
  /** Draws frames in whole display refreshes, at most the frame rate cap (Babylon's frame requester). */
  private readonly pacer = new FramePacer();
  /** Stops watching the device pixel ratio (see `applyResolution`). */
  private unwatchPixelRatio: (() => void) | null = null;
  private skyDome!: Sky;
  private readonly lighting = createLighting();
  private lastLightMinute = -1;
  /** When the lighting was last updated in the game (`FrameState.now`; see `LIGHT_MIN_MS`). */
  private lastLightAt = 0;
  /**
   * Storeys (sim-core `storeys.rs`): how many at and above the ground, the lowest (-1 with a
   * basement), where their rows are, and the one in view (see `storeyShown`: higher ones are
   * hidden, and in the basement only it shows).
   */
  private storeys = { count: 1, lowest: 0, layout: { depth: 0 } as StoreyLayout };
  private storey = 0;
  /** Build helpers (cursor, ghost, grid, previews) work in lot rows; this lifts them to the storey in view. */
  private storeyLayer!: TransformNode;
  /** The world last built (built again when the storey in view changes). */
  private lastWorld: WorldStructure | null = null;
  private lastMinute = 12 * 60;
  private readonly lotCentre = new Vector3();
  /** Radius of the game's sun shadow map around `lotCentre` (see `applyLighting`). */
  private shadowRadius = 30;
  /** `?debug`: expose the scene/renderer on `window` and log build timings. */
  private readonly debug = new URLSearchParams(location.search).has('debug');
  /** `?hour=21` freezes the lighting at that hour (for screenshots and look development). */
  private readonly fixedHour: number | null;
  /** `?diag=…` (benchmarks): features left out of the picture to measure what each costs, or time frozen (`debug/benchFlags.ts`). */
  private readonly diag = benchDiag();
  /** Benchmark hooks around each frame, and what the frame drew (see `setFrameProbe`). */
  private probe: FrameProbe | null = null;
  private probeOff: (() => void)[] = [];
  private readonly counts: FrameCounts = { drawCalls: 0, triangles: 0, shadowDrawCalls: 0, shadowTriangles: 0, mirrorDrawCalls: 0, mirrorTriangles: 0, lightingUpdates: 0, gpuFrames: 0, gpuMs: 0, residentsMs: 0 };
  /** Who wants GPU time measured (the F3 overlay, the benchmark); WebGL2's timer queries on. */
  private readonly gpuWanted = new Set<string>();
  private glTimer = false;
  /** WebGPU: per frame, its first pass's start and last pass's end (ns; see `spanPasses`), and the latest whole frames (ms). */
  private readonly gpuSpans = new Map<number, [bigint, bigint]>();
  private readonly gpuRecent: number[] = [];
  private readonly roomLights: SpotLight[] = [];
  /** Holds the room lights when lighting is clustered (null: forward lights). */
  private lampCluster: ClusteredLightContainer | null = null;
  /** Pools of lamplight on the lawn under the garden's lamps (the ground isn't lit by the lamps themselves). */
  private gardenPools: GardenPools | null = null;
  /** What each interior light is lighting (null: unused). */
  private roomLightUse: (RoomLight | null)[] = [];
  /** Per interior light: 1 on, 0 out (its room is dark), eased between. */
  private roomLightOn: number[] = [];
  /** Per room whose windows are dimmed: 1 glowing, 0 out (its room is dark), eased between; unlisted rooms glow. */
  private readonly windowGlowOn = new Map<number, number>();
  /**
   * Rooms where everyone is asleep (lights out), rooms with someone awake (babies don't count, as
   * in sim-core `lights`), and rooms with anyone in them; where residents on the move are and
   * where their way goes (`MOVER` floats each, in lot rows); all as of this frame (`findDarkRooms`).
   */
  private readonly darkRooms = new Set<number>();
  private readonly awakeRooms = new Set<number>();
  private readonly occupiedRooms = new Set<number>();
  private readonly movers: number[] = [];
  /** Rooms without windows in the whole town (as sim-core `lights::windowless`), for the lot it was found for. */
  private windowless: { rooms: ReadonlySet<number>; of: readonly unknown[] } = { rooms: new Set(), of: [] };
  /**
   * The windowless rooms of the storey in view kept out of daylight (`DigState.mask`): their
   * lamps are lit by day as at night, and a fill (no lamp there) only while someone awake is in.
   */
  private dimmed = new Set<number>();
  /** Their tiles, as `DigState` gets them while the game world is shown (null: none in view). */
  private dimming: DarkTiles | null = null;
  /** The rooms of the lot in view, for keeping each lamp to its own (`markRoomLights`). */
  private roomTiles: RoomMask | null = null;
  /** The viewed lot's door leaves that swing, each with how far open it stands (1 open, as drawn; -1 not placed yet). */
  /** `base`: where the leaf's meshes stand (lifted to their storey), before any slide. */
  private doors: { leaf: DoorLeaf; open: number; base: Vector3 }[] = [];
  /** Gates in the fences' meshes, each with how far open it stands (0 shut, as drawn) and the side it opens to. */
  private gates: { mesh: Mesh; leaf: GateLeaf; open: number; towards: number }[] = [];
  /** Per neighbour's house (`Street.houses`): someone is up in there (scratch). */
  private readonly neighboursUp: boolean[] = [];
  /** The lamps switched off (object ids; sim-core `lights.rs`), and since when nothing fades (`fadeLamps`). */
  private lampsOff: ReadonlySet<number> | null = null;
  private lampsSettled = true;
  /** Per lamp going off, off or coming on (object id): 1 on, 0 off, eased between; unlisted lamps are on. */
  private readonly lampOn = new Map<number, number>();
  /** The viewed lot's lamps (all of them, also those without an interior light to go round) by room. */
  private lotLamps: { room: number; lamp?: number }[] = [];
  /** Rooms whose lamps are all switched off: their windows go dark as when everyone is asleep. */
  private readonly unlitRooms = new Set<number>();
  /** More lights than interior lights to go round (Low): lamps switched off give theirs up (`placeRoomLights`). */
  private lightsShort = false;
  /** Indices of the snapshot layout's sleeping action tags. */
  private sleepActions: { tags: readonly string[]; ids: Set<number> } | null = null;
  /** Night level of the lamps at the last lighting update (`Lighting.lamps`). */
  private lampsLevel = 0;
  /** Objects whose screen is on this frame (a resident is using them), and when it was last set. */
  private screenOn = new Set<number>();
  private screenTime = 0;
  /** The camera's exposure at the last lighting update (without the look's overall scale). */
  private exposure = 1;
  private readonly lampColor = new Color3();
  private readonly glowColor = new Color3();
  private readonly emissiveTmp = new Color3();

  // Selection marker (sparkle star, outline, glow halo) and contact shadows (thin instances
  // updated per frame).
  private marker: Mesh[] = [];
  private markerHalo!: Mesh;
  private readonly markerMatrix = new Float32Array(16);
  private readonly markerColor = new Float32Array(4);
  private readonly markerOutlineColor = new Float32Array(4);
  private readonly markerHaloMatrix = new Float32Array(16);
  private readonly markerHaloColor = new Float32Array(4);
  private markerPalette: number[][] = [];
  /** The star's material and its outline's (`setupMarker`), kept for when it's built again. */
  private markerMats: [PBRMaterial, PBRMaterial] | null = null;
  /** The files the marker's star was built without (`ModelTemplate.failed`): it's built again when one comes. */
  private markerFailed: readonly string[] = [];
  /** The star being built again (`buildMarker`), one file after another. */
  private markerBuilds: Promise<void> = Promise.resolve();
  private unlistenMarker: (() => void) | null = null;
  private emotionMood: number[] = [];
  private simShadow!: Mesh;
  private readonly simShadowMatrices = new Float32Array(MAX_SIMS * 16);
  private objectShadow!: Mesh;

  // Build mode helpers.
  private ghostRoot!: TransformNode;
  private ghostKey = '';
  /**
   * Models whose file failed to load for the ghost, with the files: it's drawn as its box straight
   * away from then on, until one of those files comes (`ghostFileArrived`).
   */
  private readonly ghostUnfiled = new Map<string, string[]>();
  /** What the ghost was last built from (`loadGhost`'s arguments). */
  private ghostArgs: Parameters<BabylonRenderer['loadGhost']> | null = null;
  /** The model in hand (without its resizing). */
  private ghostModel = '';
  private ghostMeshes: Mesh[] = [];
  /** Each ghost mesh's own material: shown where the thing fits; the translucent tint where it can't go. */
  private ghostOwn: (Material | null)[] = [];
  private ghostMaterial!: StandardMaterial;
  /** Flat arrow on the tile the ghost is used from, pointing out of it. */
  private ghostArrow!: Mesh;
  private ghostArrowMaterial!: StandardMaterial;
  private ghostValid = true;
  /** The ghost hangs on a wall (`PlacementGhost.wall`): no thing on a wall is hidden with its wall. */
  private ghostOnWall = false;
  /** The ghost glides to its tile and turns smoothly; it floats a little, as if held. */
  private ghostShown = false;
  private readonly ghostGoal = new Vector3();
  /** Height of the top the ghost is held over (things on tables, counters, shelves). */
  private ghostLift = 0;
  private ghostYawGoal = 0;
  private ghostAge = 0;
  private ghostSpawn = 0;
  /** The placed object in hand, hidden (its instances shrunk to nothing; their matrices kept to restore). */
  private heldId: number | null = null;
  private held: { id: number; saved: { target: PopTarget; matrix: Float32Array }[] } | null = null;
  /** Chairs drawn pulled up to their table (`Characters.pulled`): how far each is in, and its matrices where it stands. */
  private pulledIn = new Map<number, { by: number; saved: { target: PopTarget; matrix: Float32Array }[] }>();
  /** The seats that face a table, and their turns (quarter turns). */
  private seatTurns = new Map<number, number>();
  private heldRelease: ReturnType<typeof setTimeout> | null = null;
  private fx!: BuildEffects;
  /** Placed objects' moving parts, played while residents use them. */
  private readonly motion = new ObjectMotion();
  /** What glows (lamp shades at night, screens while used: a template's `glow`), per model: its instances' strengths, rooms and object ids. */
  private glows: { mesh: Mesh; colors: Float32Array; rooms: number[]; lamps: number[]; ids: number[]; screen: boolean }[] = [];
  private previewMeshes!: Record<'wall' | 'door' | 'window' | 'open', Mesh>;
  private previewBuffers!: Record<'wall' | 'door' | 'window' | 'open', Float32Array>;
  private previewValid = true;
  /** Fences, gates, doors and windows the edge preview shows as built, and what they show. */
  private builtPreview: Mesh[] = [];
  private builtPreviewKey = '';
  /** The next of them, hidden until its shaders are ready, with the stretch of wall it hides (x0, z0, x1, z1, width, y0, y1). */
  private builtPending: { meshes: Mesh[]; hole: number[] | null } | null = null;
  private grid!: Mesh;
  private gridOn = false;
  private paintPreview!: Mesh;
  private paintBuffer = new Float32Array(64 * 16);
  private floorPreview!: Mesh;
  private floorBuffer = new Float32Array(64 * 16);
  private roomOverlay: Mesh | null = null;
  private roomMatrices = new Float32Array(0);
  private roomColors = new Float32Array(0);

  // Scratch objects for allocation-free updates.
  private readonly mOut = new Matrix();
  private simCount = 0;
  private readonly simInView = (x: number, z: number) => this.inView(x, z, SIM_VIEW_MARGIN);
  private readonly residentRows = (mesh: AbstractMesh) => this.characters.rowsOf(mesh);
  private readonly vTmp = new Vector3();
  private readonly vScale = new Vector3();
  private readonly qTmp = new Quaternion();
  private readonly vProj = new Vector3();
  private readonly viewport = new Viewport(0, 0, 1, 1);
  /**
   * The canvas's size on the page (CSS px), kept up to date by `sizeWatch`. `project` runs for
   * every bubble each frame, after the interface's changes: reading `clientWidth` there would
   * make the browser redo style and layout in the middle of the frame.
   */
  private readonly cssSize = { width: 0, height: 0 };
  private sizeWatch: ResizeObserver | null = null;
  /** Camera alpha, beta, radius, target x/y/z and screen offset x/y at the last frame. */
  private readonly lastView = new Float64Array(8);

  constructor(
    private readonly deps: RendererDeps,
    private readonly quality: RenderQuality,
  ) {
    const hour = Number(new URLSearchParams(location.search).get('hour'));
    this.fixedHour = Number.isFinite(hour) && new URLSearchParams(location.search).has('hour') ? hour : null;
  }

  async init(canvas: HTMLCanvasElement): Promise<void> {
    this.canvas = canvas;
    const measure = () => {
      this.cssSize.width = this.canvas.clientWidth;
      this.cssSize.height = this.canvas.clientHeight;
    };
    measure();
    this.sizeWatch = new ResizeObserver(measure);
    this.sizeWatch.observe(canvas);
    const forceWebGL = this.deps.backend === 'webgl' || new URLSearchParams(location.search).get('renderer') === 'webgl';
    // No multisampling on the canvas: only the last full-screen pass draws to it, so it would only
    // cost memory traffic (66 MB a frame at 1080p; screenshots without it are the same, P1-06).
    // The scene itself is smoothed in the post-processing pipeline (`msaaSamples`, or FXAA).
    if (!forceWebGL && !webGpuFailed() && (await WebGPUEngine.IsSupportedAsync)) {
      const engine = new WebGPUEngine(canvas, { antialias: false, powerPreference: 'high-performance', doNotHandleContextLost: true, ...compressedTextureFeatures() });
      try {
        await engine.initAsync();
        this.engine = this.webgpu = engine;
        this.watchDeviceLoss(engine);
      } catch (err) {
        // An adapter but no device (a blocklisted driver, a device lost while starting): WebGL2,
        // for the rest of the session.
        console.warn('[render] WebGPU failed to start; drawing with WebGL2', err);
        noteWebGpuFailed();
        try {
          engine.dispose();
        } catch {
          // Half started: nothing more to free.
        }
      }
    }
    this.engine ??= new Engine(canvas, false, { powerPreference: 'high-performance' });
    keepClipPlaneEffects();
    if (new URLSearchParams(location.search).has('profile')) {
      // Which shader variant is being compiled (and how long it takes) when a frame is slow.
      const process = Effect.prototype._processShaderCodeAsync;
      const seen = new Map<string, string>();
      let compilingFor = '';
      const ready = PBRMaterial.prototype.isReadyForSubMesh;
      PBRMaterial.prototype.isReadyForSubMesh = function (this: PBRMaterial, ...args: Parameters<typeof ready>) {
        const before = compilingFor;
        compilingFor = `${this.name} on ${args[0]?.name ?? ''}`;
        try {
          return ready.apply(this, args);
        } finally {
          compilingFor = before;
        }
      };
      const readyStd = StandardMaterial.prototype.isReadyForSubMesh;
      StandardMaterial.prototype.isReadyForSubMesh = function (this: StandardMaterial, ...args: Parameters<typeof readyStd>) {
        const before = compilingFor;
        compilingFor = `${this.name} on ${args[0]?.name ?? ''}`;
        try {
          return readyStd.apply(this, args);
        } finally {
          compilingFor = before;
        }
      };
      Effect.prototype._processShaderCodeAsync = async function (this: Effect, ...args: Parameters<typeof process>) {
        const start = performance.now();
        const material = compilingFor;
        try {
          return await process.apply(this, args);
        } finally {
          const ms = performance.now() - start;
          if (ms > 20) {
            // SAFETY: Babylon stores the compiled effect's cache key in the private `_key` field, set before this hook runs.
            const key = (this as unknown as { _key: string })._key ?? '';
            console.log(`[shader] ${ms.toFixed(0)} ms at ${start.toFixed(0)} ${this.name} mat ${material} defs ${this.defines.length} key ${key.slice(0, 140)}`);
            const name = String(this.name);
            const before = seen.get(name);
            if (before) {
              const now = this.defines.split('\n');
              const old = before.split('\n');
              const changed: string[] = [];
              for (let i = 0; i < Math.max(now.length, old.length); i++) if (now[i] !== old[i]) changed.push(`${old[i] ?? ''} => ${now[i] ?? ''}`);
              if (changed.length) console.log(`[shader-diff] ${name} ${changed.slice(0, 8).join(' | ')}`);
            }
            seen.set(name, this.defines);
          }
        }
      };
    }
    if (this.webgpu && new URLSearchParams(location.search).has('profile')) {
      // WebGPU builds a pipeline synchronously on first use: a slow one is a dropped frame, so name it.
      // SAFETY: WebGPUEngine keeps its GPUDevice in the private `_device`; only read under `?profile`.
      const device = (this.webgpu as unknown as { _device?: { createRenderPipeline: (desc: { label?: string }) => unknown } })._device;
      if (device) {
        const create = device.createRenderPipeline.bind(device);
        device.createRenderPipeline = (desc) => {
          const start = performance.now();
          const pipeline = create(desc);
          const ms = performance.now() - start;
          if (ms > 8) console.log(`[pipeline] ${ms.toFixed(0)} ms ${desc.label ?? ''}`);
          return pipeline;
        };
      }
    }
    // The drawing size follows the resolution setting and the device pixel ratio (not Babylon's
    // `adaptToDeviceRatio`, which would scale our own level again when the ratio changes).
    this.applyResolution();
    this.unwatchPixelRatio = watchPixelRatio(this.onResize);
    // Screenshots that compare pixel for pixel: wind, clouds, fades and glides stand still.
    if (this.diag.has('freeze')) this.engine.getDeltaTime = () => 0;
    this.applyPacing();
    this.engine.customAnimationFrameRequester = this.pacer;
    this.timeFramesByRefresh();
    // A GPU reset (driver update, a laptop switching GPUs, a crashed GPU process): what was on the
    // GPU is gone, and the game's own buffers can't be rebuilt in place.
    this.engine.onContextLostObservable.add(() => this.onFailure?.('device'));
    this.engine.onResizeObservable.add(() => this.drawOnce());

    // Babylon's own clock (particles: the build effects' sparkles and dust) steps at most as far as the game's.
    Scene.MaxDeltaTime = MAX_FRAME_MS;
    const scene = (this.scene = new Scene(this.engine));
    this.templates = new TemplateCache(scene);
    if (new URLSearchParams(location.search).has('profile')) {
      // Where a slow frame's time goes: shadow/render targets, active-mesh evaluation, the draw phase.
      const t: Record<string, number> = {};
      scene.onBeforeAnimationsObservable.add(() => { t.anim = performance.now(); });
      scene.onBeforeRenderObservable.add(() => { t.render = performance.now(); });
      scene.onBeforeRenderTargetsRenderObservable.add(() => { t.targets = performance.now(); });
      scene.onAfterRenderTargetsRenderObservable.add(() => { t.targetsEnd = performance.now(); });
      scene.onBeforeDrawPhaseObservable.add(() => { t.draw = performance.now(); });
      scene.onAfterDrawPhaseObservable.add(() => { t.drawEnd = performance.now(); });
      scene.onAfterRenderObservable.add(() => {
        const now = performance.now();
        if (t.anim && now - t.anim > 40) {
          console.log(`[stage] total ${(now - t.anim).toFixed(0)} anim ${(t.render - t.anim).toFixed(0)} targets ${(t.targetsEnd - t.targets).toFixed(0)} draw ${(t.drawEnd - t.draw).toFixed(0)} tail ${(now - t.drawEnd).toFixed(0)}`);
        }
      });
    }
    // Picking is analytic (see `pick`); never let Babylon raycast the scene on pointer moves.
    scene.skipPointerMovePicking = true;
    scene.constantlyUpdateMeshUnderPointer = false;
    this.instrumentation = new SceneInstrumentation(scene);
    // Once the camera has taken this frame's input (it does inside `render`), before anything is drawn.
    scene.onBeforeCameraRenderObservable.add((camera) => this.cameraMoved(camera));
    // `?debug`: expose the scene for inspection from the console / test harness.
    if (this.debug) Object.assign(window, { __scene: scene, __renderer: this });
    // PBR materials and image-based lighting; the environment must exist before materials compile.
    this.lib = MaterialLibrary.for(scene).setAssets(this.deps.assets);
    this.lib.anisotropy = this.quality.anisotropy;
    // Frozen and thawed materials bind differently; a recording made while some were thawed
    // (lighting changes) goes stale once they refreeze, so record again.
    this.lib.onRefrozen = () => this.resetSnapshot();
    this.house = new HouseBuilder(scene);
    const c = this.deps.content;
    this.house.setLooks({
      coverings: c.wallCoverings,
      floors: c.floorCoverings,
      doors: c.doorStyles,
      windows: c.windowStyles,
      roofs: c.roofStyles,
      roofColors: c.roofColors,
    });
    // Public places seen from another lot show their things in the open (as the overview does).
    this.street = new Street(scene, this.deps.assets, this.house, {
      lived: true,
      look: (def) => {
        const d = this.deps.content.object(def);
        return d && { model: d.model ?? `model.${def}`, footprint: d.footprint ?? [1, 1], standIn: this.standInOf(d) };
      },
      lod: this.quality.streetLod,
    });
    // Wind sway and tint variation for foliage materials (hooks them as the models load).
    installNature(scene);
    await this.setupEnvironment();

    this.setupCamera();
    this.setupLights();
    this.skyDome = new Sky(scene, 700, this.deps.assets.has('image.clouds', 'image') ? this.deps.assets.get('image.clouds', 'image')!.url : undefined);
    this.skyDome.setLayerMask(LAYER_ALL);
    // Distance fog in the horizon colour makes the hills fade into the sky (aerial haze).
    scene.fogMode = Scene.FOGMODE_EXP2;
    this.setupPostProcessing();
    this.setupHelpers();
    await this.setupMarker();
    // (Their data loads with the first world that has residents, not with the menus.)
    this.characters = new Characters(scene, this.deps.assets, this.lib, this.deps.content, { refreshProps: true, castsShadows: this.quality.characterShadows });
    this.characters.setRide((id) => this.motion.ride(id));
    this.characters.setLoop((id) => this.motion.loopPhase(id));
    this.characters.setSun(() => this.sun.direction);
    // Garden birds. `?debug`: `__birds()` lists them (species, state, place) with what they did and
    // the cost of a frame; `__birdFly(id, where)` sends one off; `__birdTime(scale)` slows their clock.
    this.birdLife = new Birds(scene, this.deps.assets);
    this.birdLife.setQuality(this.quality.birds);
    if (this.debug) Object.assign(window, { __birds: () => this.birdLife.debug(), __birdFly: (id: number, where: 'plant' | 'ground' | 'roof' | 'home' = 'plant') => this.birdLife.debugFly(id, where), __birdTime: (scale: number) => (this.birdLife.timeScale = scale) });
    this.items = new ItemLayer(scene, this.deps.assets, this.deps.content);
    this.items.onChange = () => this.resetSnapshot();
    // A model file that failed has come: what was drawn without it (a box, or nothing) is built again.
    this.unlistenFiles = onModelFileArrived(scene, (url) => this.fileArrived(url));
    // Reflections show the game world (not the shadow-only copies or the overview).
    const q = this.quality;
    this.mirrors = new Mirrors(scene, { count: this.diag.has('nomirrors') ? 0 : q.mirrors, scale: q.mirrorScale, samples: q.msaaSamples, rate: q.mirrorRate }, (mesh) => (mesh.layerMask & LAYER_WORLD) !== 0);
    if (new URLSearchParams(location.search).has('profile')) {
      const render = MirrorTexture.prototype.render;
      MirrorTexture.prototype.render = function (this: MirrorTexture, ...args: Parameters<typeof render>) {
        const start = performance.now();
        const result = render.apply(this, args);
        const ms = performance.now() - start;
        if (ms > 8) console.log(`[mirror] ${ms.toFixed(0)} ms at ${start.toFixed(0)}-${(start + ms).toFixed(0)} ${this.name} ${this.getSize().width}x${this.getSize().height}`);
        return result;
      };
    }
    this.motion.setAmbientRange(this.quality.ambientRange);
    this.handles = new ResizeHandles(scene, LAYER_WORLD);
    this.applyLook();
    window.addEventListener('resize', this.onResize);
  }

  /** The newest world waiting to be built, and the builds in progress (see `setWorld`). */
  private nextWorld: { world: WorldStructure; view: ViewRect | null } | null = null;
  private worldBuilds: Promise<void> | null = null;

  /**
   * Builds `world`. One build runs at a time: worlds arriving meanwhile (two edits in one frame,
   * a multi-edit build) wait for it, and only the newest of them is built next. Resolves when
   * the newest world is built.
   */
  setWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    this.lastWorld = world;
    this.nextWorld = { world, view };
    return (this.worldBuilds ??= this.buildWorlds());
  }

  private async buildWorlds(): Promise<void> {
    let error: unknown = null;
    try {
      for (let next = this.nextWorld; next; next = this.nextWorld) {
        this.nextWorld = null;
        try {
          await this.buildOne(next.world, next.view);
          error = null;
        } catch (err) {
          error = err;
        }
      }
    } finally {
      this.worldBuilds = null;
    }
    if (error) throw error;
  }

  private async buildOne(world: WorldStructure, view: ViewRect | null): Promise<void> {
    this.beginBuild();
    this.gameWarmed = false;
    this.mirrorWarmed = false;
    // (First: a storey in view the world no longer has is changed below, for this build to show.)
    this.worldBuilding = true;
    this.sliceStart = performance.now();
    const until = this.sliceStart + TEXTURE_WAIT_MS;
    try {
      this.storeys = { count: world.storeys ?? 1, lowest: lowestStorey(world), layout: { depth: world.depth, storeyDepth: world.storeyDepth, storeys: world.storeys, basement: world.basement } };
      if (this.storey >= this.storeys.count || this.storey < this.storeys.lowest) this.setStorey(this.storey);
      perfPhase('render: build world');
      await perfTimeAsync('render: buildWorld', () => this.buildWorld(world, view));
      perfPhase('render: warm shaders');
      await perfTimeAsync('render: warmShaders', () => this.warmShaders(until));
    } finally {
      this.worldBuilding = false;
      this.endBuild();
    }
    // Built behind the overview (prepared in a menu): draw it hidden a few times so it shows without a hitch,
    // once its textures are in (KTX2 ones transcode in the background; a shader waits for its textures).
    if (this.townMode) {
      await Promise.race([this.scene.whenReadyAsync(), new Promise((resolve) => setTimeout(resolve, Math.max(0, until - performance.now())))]);
      this.warmFrames = WARM_WORLD_FRAMES;
    }
  }

  /** The lot arrays the last key was made from, and that key (they arrive unchanged when only furniture moved). */
  private lotParts: { refs: unknown[]; key: string } | null = null;

  /** Everything the lot layer (streets, house, fences) is built from. */
  private lotKeyOf(world: WorldStructure, view: ViewRect | null): string {
    const refs = [world.meta, world.walls, world.openings, world.diagonals, world.floors, world.fences];
    if (!this.lotParts || this.lotParts.refs.some((r, i) => r !== refs[i])) {
      const m = world.meta;
      this.lotParts = { refs, key: JSON.stringify([m?.seed, m?.streets, m?.paths, world.walls, world.openings, world.diagonals, world.floors, world.fences]) };
    }
    return JSON.stringify([
      view,
      world.width,
      world.depth,
      this.lotParts.key,
      world.plots.map((p) => [p.house, p.roof]),
      world.households.find((h) => h.player)?.plot,
      houseObjectKey(world, view, (def) => !!this.deps.content.object(def)?.stairs),
    ]);
  }

  private async buildWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    const started = performance.now();
    for (const mesh of this.worldMeshes) mesh.dispose(false, false);
    this.worldMeshes = [];
    this.placed = [];
    this.wallItemsHidden.clear();
    this.fx.clearPops();
    // The held object's buffers go with the old world; a moved one now shows in its new place.
    this.held = null;
    if (this.heldRelease) clearTimeout(this.heldRelease);
    this.heldRelease = null;
    this.view = view;

    const { width, depth } = world;
    this.lotCentre.set(view ? view.x + view.w / 2 : width / 2, 0, view ? view.z + view.d / 2 : depth / 2);
    this.shadowRadius = Math.hypot(view?.w ?? width, view?.d ?? depth) / 2 + SHADOW_MARGIN;
    if (!this.cameraPlaced) {
      // (Re-aims from the camera's current position; a parked camera computes it first.)
      this.gameCamera.getViewMatrix(true);
      this.gameCamera.setTarget(this.lotCentre.clone());
      this.cameraPlaced = true;
    }

    // The residents' data downloads alongside the lot (it is first needed after it, in `buildSims`).
    void this.characters.init();
    // The landscape depends only on the town, not on which lot is viewed (and may be the overview's).
    // (Its shadow casters join the shadow map's list as it has them, `landscapeCasters`.)
    await perfTimeAsync('render: nature', async () => {
      const n = await this.useNature('world', world);
      // No grass through floors of rooms built (or furniture bought) since the lawn was scattered.
      perfTime('nature: refreshLawn', () => n.landscape.refreshLawn(world));
    });
    this.templatesInUse.clear();
    const lotKey = this.lotKeyOf(world, view);
    const lotBuilt = lotKey !== this.lotKey;
    if (lotBuilt) {
      for (const mesh of this.lotMeshes) mesh.dispose(false, false);
      this.lotKey = lotKey;
      this.roofs = [];
      this.walls = this.wallsLow = null;
      const before = new Set(this.templatesInUse);
      await perfTimeAsync('render: streets', async () => {
        this.buildStreets(world);
        await this.pace();
      });
      this.lotCasters = await perfTimeAsync('render: lot', async () => {
        const casters = await this.buildLot(world);
        await this.pace();
        return casters;
      });
      // The viewed lot's fences stand on the ground (hidden with it while the basement is in view); other lots' stay.
      const v = view;
      const onLot = (f: FenceEdge) => !!v && f.x >= v.x && f.z >= v.z && f.x + (f.axis === 'h' ? 1 : 0) <= v.x + v.w && f.z + (f.axis === 'v' ? 1 : 0) <= v.z + v.d;
      this.gates = [];
      for (const [list, storey] of [[(world.fences ?? []).filter(onLot), 0], [(world.fences ?? []).filter((f) => !onLot(f)), undefined]] as const) {
        const fences = buildFences(this.scene, list, this.deps.content.fenceStyles);
        if (!fences) continue;
        const { mesh } = fences;
        if (storey !== undefined) mesh.metadata = { storey };
        this.worldMeshes.push(mesh);
        for (const leaf of fences.gates) this.gates.push({ mesh, leaf, open: 0, towards: 1 });
      }
      // What was just built belongs to the lot layer; objects follow in `worldMeshes`.
      this.lotMeshes = this.worldMeshes;
      this.worldMeshes = [];
      this.lotTemplates = new Set([...this.templatesInUse].filter((k) => !before.has(k)));
      for (const mesh of this.lotMeshes) mesh.freezeWorldMatrix();
      for (const { leaf } of this.doors) for (const mesh of leaf.meshes) mesh.unfreezeWorldMatrix();
    } else {
      for (const key of this.lotTemplates) this.templatesInUse.add(key);
    }
    const houseCasters = this.lotCasters;
    this.placeRoomLights(world, view);
    await this.pace();
    const streetCasters = await perfTimeAsync('render: street furniture', () => this.street.build(world, this.view, () => this.pace()));
    const merged = await perfTimeAsync('render: objects', async () => {
      const merged = await this.buildObjects(world);
      // What replaces the boxes of files that came is placed now: they're hidden (and go at the end).
      this.templates.hideStale();
      this.pullUpChairs(world);
      this.hideHeld();
      await this.pace();
      return merged;
    });
    await perfTimeAsync('render: sims', () => this.buildSims(world));
    this.retireTemplates(new Set(this.templatesInUse));
    const templates = await perfTimeAsync('render: templates', () => this.templateMeshes());

    const notCasting = new Set(['floors', 'floorsTiled', 'floorsCarpet', 'floorsStone', 'streets', 'paths', 'fields', 'furrows', 'contactShadows', 'objectShadows', 'windows', 'doorLeafGlass', 'windowsInner', 'plinth', 'wallsTiled']);
    this.baseCasters = [
      ...new Set([
        ...[...this.lotMeshes, ...this.worldMeshes].filter((m) => !notCasting.has(m.name) && !m.name.startsWith('silhouette') && !houseCasters.includes(m)),
        ...templates.filter((m) => !merged.has(m)),
        // Sims' small details (eyes, brows) don't cast sun shadows (nothing of them does on Low).
        ...this.characters.casters(),
        ...(this.quality.streetShadows ? streetCasters : []),
      ]),
    ];
    this.wallCasters = houseCasters;
    for (const mesh of [...this.worldMeshes, ...templates]) mesh.freezeWorldMatrix();
    this.applyWallMode();
    this.applyStoreys();
    this.showItems();
    // Built without the boxes of files that came (set aside before this build began, `refreshModels`): they go.
    if (this.templates.disposeStale()) this.resetSnapshot();
    void this.settleBirds(world);
    this.lastLightMinute = -1;
    // Then the other lots' furniture downloads in the background (the camera may follow someone there).
    const elsewhere = world.objects.filter((o) => !this.inView(o.x + o.w / 2, o.z + o.d / 2, 0));
    prefetchModels(elsewhere.map((o) => this.objectModel(o)).filter((key) => this.deps.assets.has(key, 'model')).map((key) => this.deps.assets.get(key, 'model')));
    if (this.debug) console.info(`[render] setWorld ${(performance.now() - started).toFixed(1)} ms${lotBuilt ? '' : ' (lot kept)'}`);
  }

  update(frame: FrameState): void {
    const { curr, layout } = frame;
    // Snapshot rendering bakes light uniforms and the clear colour into the recording, so
    // lighting changes force a re-record; throttle them to keep most frames replayed.
    const minute = this.fixedHour !== null ? this.fixedHour * 60 : curr[layout.header.minute];
    this.lastMinute = minute;
    // Behind the overview the game world waits (nothing of it is drawn).
    if (this.townMode) return;
    const snapshot = this.webgpu?.snapshotRendering ?? false;
    const h = layout.header;
    const fast = curr[h.speed] >= 3 || (h.calm !== undefined && curr[h.calm] > 0);
    const step = snapshot ? LIGHT_STEP_SNAPSHOT : fast ? Math.max(LIGHT_STEP, this.quality.lightingStep) : LIGHT_STEP;
    const due = this.lastLightMinute < 0 || frame.now - this.lastLightAt >= LIGHT_MIN_MS;
    if (due && Math.abs(minute - this.lastLightMinute) >= step) {
      this.applyLighting(minute);
      this.lastLightMinute = minute;
      this.lastLightAt = frame.now;
      this.resetSnapshot();
    }
    // Lamps switched off or on fade out or in.
    const switched = this.fadeLamps(this.frameDt / 1000);
    // Lights go out in rooms where everyone is asleep (and neighbours' windows where they all are).
    this.findDarkRooms(frame);
    this.swingDoors(GAME_RATE[curr[h.speed] | 0] ?? 1);
    if (this.fadeRoomLights(this.frameDt / 1000) && (this.lampsLevel > 0 || this.storey < 0 || this.dimmed.size > 0)) {
      this.applyRoomLights();
      this.resetSnapshot();
    }
    // And their windows (and lamp shades, screens) stop glowing with them.
    if ((this.fadeWindowGlow(this.frameDt / 1000) || switched) && (this.lampsLevel > 0 || this.dimmed.size > 0)) {
      this.house.setWindowGlow(this.windowGlowOn);
      this.applyGlow();
      this.resetSnapshot();
    }
    this.steerCamera(this.frameDt / 1000);
    this.keepCameraClear();
    this.updateCutaway();
    this.followCamera();
    if (!this.reducedMotion) this.skyDome.drift(this.frameDt);
    // Things springing into place cast as they grow.
    if (this.fx.busy) this.sunShadows.moved();
    this.fx.update(this.frameDt);
    if (this.ghostShown) this.updateGhost(this.frameDt);
    if (this.builtPending) this.swapBuilt();
    if (this.roofs.length && this.roofVisible() !== this.roofShown) {
      this.roofShown = !this.roofShown;
      this.showRoofs();
      this.resetSnapshot();
    }

    const count = Math.min(curr[layout.header.simCount], MAX_SIMS);
    this.simCount = count;
    // Doors that open while residents use what they're on, and ambient clips near the camera (the
    // sun's shadows aren't redrawn for those).
    const motion = this.motion.update(frame, layout.actions ?? [], this.camera.position);
    if (motion.moved && this.movingPartsCast) this.sunShadows.moved(false);
    if (motion.moved) this.resetSnapshot();
    // (Ambient clips alone don't make the held mirrors draw again: a fan turns in them when something else changes.)
    else if (motion.ambient && this.webgpu?.snapshotRendering) this.webgpu.snapshotRenderingReset();
    this.animateScreens(frame);
    // Sims: pose every visible Sim into the shared pose texture (characters.ts).
    // (After the moving parts: residents on a moving seat go with it.)
    this.characters.update(frame, this.simInView, this.simShadowMatrices, this.storey);
    this.pullChairs();
    this.updateBirds(frame, minute);

    // Sims who moved (or breathed: anyone in view while the game runs) cast anew.
    if (this.characters.posed && this.quality.characterShadows) this.sunShadows.moved(false);
    // Held reflections are drawn again when a resident in them moved; they draw only the residents they show.
    this.mirrors.update(this.camera, { heads: this.characters.heads, visible: this.characters.visible, count, moved: this.characters.posed, rowsOf: this.residentRows });
    this.handles.update(this.camera.globalPosition);
    this.simShadow.thinInstanceBufferUpdated('matrix');
    this.lastFrame = frame;
  }

  /**
   * Once per drawn frame of the game world, after the camera took this frame's input (dragging,
   * zooming): the selection marker (it faces the camera) and what the session places over the
   * view (`onView`: bubbles, room tags), so they don't trail a frame behind the camera.
   */
  private cameraMoved(camera: Camera): void {
    if (camera !== this.camera || this.townMode || this.viewFrame === this.scene.getFrameId()) return;
    this.viewFrame = this.scene.getFrameId();
    this.placeMarker();
    this.onView?.();
  }

  private placeMarker(): void {
    this.markerMatrix.fill(0);
    this.markerHaloMatrix.fill(0);
    const frame = this.lastFrame;
    const sel = this.selectedSim;
    if (frame && sel !== null && sel < this.simCount && this.characters.visible[sel]) {
      const { curr, layout, now } = frame;
      const h = this.characters.heads;
      const emotion = curr[layout.headerLen + sel * layout.simStride + layout.sim.emotion];
      this.writeMarker(h[sel * 3], h[sel * 3 + 1], h[sel * 3 + 2], emotion, now);
    }
    for (const mesh of this.marker) {
      mesh.thinInstanceBufferUpdated('matrix');
      mesh.thinInstanceBufferUpdated('color');
    }
    this.markerHalo.thinInstanceBufferUpdated('matrix');
    this.markerHalo.thinInstanceBufferUpdated('color');
  }

  /** The ray of the last `pick` (for `pickPlane`). */
  private readonly pickOrigin = new Vector3();
  private readonly pickDir = new Vector3();

  setItems(items: readonly ItemPlacement[]): void {
    this.items.set(items);
  }

  setLampsOff(off: ReadonlySet<number>): void {
    // The first time (a game loading) they're as they are; afterwards a lamp switched off fades out.
    const first = !this.lampsOff;
    for (const id of off) if (!this.lampOn.has(id)) this.lampOn.set(id, first ? 0 : 1);
    this.lampsOff = off;
    this.lampsSettled = false;
    this.motion.setLampsOff(off);
    this.findUnlitRooms();
    if (this.lightsShort && this.lastWorld) {
      this.placeRoomLights(this.lastWorld, this.view);
      this.applyRoomLights();
      this.resetSnapshot();
    }
  }

  /** Eases each lamp towards off or on (`lampsOff`); true if any changed. */
  private fadeLamps(dt: number): boolean {
    if (this.lampsSettled) return false;
    let changed = false;
    const step = Math.min(1, dt / LAMP_SWITCH_FADE);
    for (const [id, on] of this.lampOn) {
      const target = this.lampsOff?.has(id) ? 0 : 1;
      if (on === target && target === 0) continue;
      const next = target > on ? Math.min(target, on + step) : Math.max(target, on - step);
      if (next === 1) this.lampOn.delete(id);
      else this.lampOn.set(id, next);
      changed = true;
    }
    this.lampsSettled = !changed;
    return changed;
  }

  /** The viewed lot's rooms with lamps, all of them switched off (and nothing else lit there, a fire). */
  private findUnlitRooms(): void {
    const lit = new Set<number>();
    this.unlitRooms.clear();
    for (const { room, lamp } of this.lotLamps) {
      if (lamp === undefined || !this.lampsOff?.has(lamp)) lit.add(room);
      else if (room) this.unlitRooms.add(room);
    }
    for (const room of lit) this.unlitRooms.delete(room);
  }

  /** Food and dishes for the lot and storeys in view. */
  private showItems(): void {
    this.items.show({ storey: this.storey, storeyOf: (z) => this.storeyOf(z), shown: (k) => storeyShown(k, this.storey), groundZ: (z) => z - this.rowOf(this.storeyOf(z)), storeyHeight: WALL_HEIGHT, inView: (x, z) => this.inView(x, z, 0) });
  }

  pick(x: number, y: number): PickResult {
    const ray = this.scene.createPickingRay(x, y, null, this.camera);
    const o = ray.origin;
    const d = ray.direction;
    this.pickOrigin.copyFrom(o);
    this.pickDir.copyFrom(d);
    let ground: PickResult['ground'] = null;
    // The floor of the storey in view, answered in its rows of the lot.
    const floor = this.storey * WALL_HEIGHT;
    if (d.y < -1e-6 && o.y > floor) {
      const t = (floor - o.y) / d.y;
      ground = { x: o.x + d.x * t, z: o.z + d.z * t + this.rowOf(this.storey) };
    }
    let objectId: number | null = null;
    let nearest = Infinity;
    for (const p of this.placed) {
      if (this.wallItemsHidden.has(p.id)) continue;
      const t = rayBox(o, d, p.minX, p.y0 + p.bottom, p.minZ, p.maxX, p.y0 + p.height, p.maxZ);
      if (t < nearest) {
        nearest = t;
        objectId = p.id;
      }
    }
    // A plate on a table is picked before the table.
    let itemId: number | null = null;
    let nearestItem = Infinity;
    for (const p of this.items.placed) {
      const t = rayBox(o, d, p.minX, p.y0, p.minZ, p.maxX, p.y0 + p.height, p.maxZ);
      if (t < nearestItem && t <= nearest + 0.3) {
        nearestItem = t;
        itemId = p.id;
      }
    }
    return { ground, objectId, itemId };
  }

  setResizeHandles(handles: readonly ResizeHandle[] | null, hot: string | null = null): void {
    this.handles.set(handles, (h) => {
      const k = this.storeyOf(h.z);
      return new Vector3(h.x, k * WALL_HEIGHT + h.y, h.z - this.rowOf(k));
    }, hot);
  }

  handleAt(x: number, y: number): string | null {
    return this.handles.at(x, y, (p, out) => this.project(p.x, p.y, p.z, out));
  }

  pickVertical(x: number, z: number): number | null {
    const [o, d] = [this.pickOrigin, this.pickDir];
    const k = this.storeyOf(z);
    const pz = z - this.rowOf(k);
    // The upright plane through the point, square to the camera's view across the floor.
    const fx = this.camera.target.x - this.camera.globalPosition.x;
    const fz = this.camera.target.z - this.camera.globalPosition.z;
    const len = Math.hypot(fx, fz);
    if (len < 1e-6) return null;
    const [nx, nz] = [fx / len, fz / len];
    const along = d.x * nx + d.z * nz;
    if (Math.abs(along) < 1e-6) return null;
    const t = ((x - o.x) * nx + (pz - o.z) * nz) / along;
    return o.y + d.y * t - k * WALL_HEIGHT;
  }

  pickOnWall(axis: 'h' | 'v', line: number, storeyRow: number): { along: number; y: number } | null {
    const [o, d] = [this.pickOrigin, this.pickDir];
    const k = this.storeyOf(storeyRow);
    const D = this.rowOf(k);
    // The wall's plane in render space (rows of upper storeys are drawn at the ground's).
    const plane = axis === 'h' ? line - D : line;
    const across = axis === 'h' ? d.z : d.x;
    if (Math.abs(across) < 1e-6) return null;
    const t = (plane - (axis === 'h' ? o.z : o.x)) / across;
    if (t <= 0) return null;
    return { along: axis === 'h' ? o.x + d.x * t : o.z + d.z * t + D, y: o.y + d.y * t - k * WALL_HEIGHT };
  }

  objectBox(id: number): { height: number; bottom: number } | null {
    const p = this.placed.find((p) => p.id === id);
    return p ? { height: p.height, bottom: p.bottom } : null;
  }

  pickPlane(height: number): { x: number; z: number } | null {
    const [o, d] = [this.pickOrigin, this.pickDir];
    const y = this.storey * WALL_HEIGHT + height;
    if (d.y > -1e-6 || o.y <= y) return null;
    const t = (y - o.y) / d.y;
    return { x: o.x + d.x * t, z: o.z + d.z * t + this.rowOf(this.storey) };
  }

  birds(): readonly SeenBird[] {
    // (Behind the overview the game world isn't drawn.)
    return this.townMode ? [] : this.birdLife.heard();
  }

  birdSings(id: number, seconds: number): void {
    this.birdLife.sings(id, seconds);
  }

  /**
   * Where the birds live, now that the lot is built: the viewed lot's garden trees and shrubs, the
   * garden things outdoors (catalog trees, hedges, roses), the street's bushes, hedges and public
   * places' trees, the woods nearby, and the viewed house's roof ridge.
   */
  private async settleBirds(world: WorldStructure): Promise<void> {
    const plants: PlantPlacement[] = [];
    for (const [key, items] of this.decorPlaced) for (const [x, z, s, yaw] of items) plants.push({ key, x, y: 0, z, sx: s, sy: s, sz: s, yaw });
    const gd = world.storeyDepth ?? world.depth;
    for (const o of world.objects) {
      if (o.z >= gd) continue;
      const centre = objectCentre(o);
      // Indoors (a fiddle-leaf fig in the lounge): not a bird's.
      if (world.rooms[Math.floor(centre.z) * world.width + Math.floor(centre.x)] !== 0) continue;
      const key = this.objectModel(o);
      const entry = this.deps.assets.get(key, 'model');
      if (!entry) continue;
      const { turn, size } = placementVariation(entry.vary ?? 0, o.x, o.z);
      plants.push({ key, x: centre.x, y: o.lift ?? 0, z: centre.z, sx: size, sy: size, sz: size, yaw: (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn });
    }
    for (const { key, items, flat } of this.street.placed) for (const [x, z, s, sy, yaw] of items) plants.push({ key, x, y: 0, z, sx: s, sy, sz: flat ? 1 : s, yaw });
    const v = this.view;
    const woods = this.natureIn.world?.landscape;
    if (v && woods) for (const p of woods.woodsNear(v.x + v.w / 2, v.z + v.d / 2, Math.hypot(v.w, v.d) / 2 + this.quality.birds.range)) plants.push({ key: p.key, x: p.x, y: p.y, z: p.z, sx: p.scale, sy: p.up, sz: p.scale, yaw: p.yaw });
    // The roof ridge: the roofs' highest points, a perch every half metre along them.
    let roofTop: number | null = null;
    const ridge: number[] = [];
    const tops: [number, number, number][] = [];
    for (const roof of this.roofs) {
      const pos = roof.getVerticesData(VertexBuffer.PositionKind);
      if (!pos) continue;
      roof.computeWorldMatrix(true);
      const m = roof.getWorldMatrix();
      let top = -Infinity;
      const pts: Vector3[] = [];
      for (let i = 0; i < pos.length; i += 3) {
        const p = Vector3.TransformCoordinates(this.vTmp.set(pos[i], pos[i + 1], pos[i + 2]), m);
        top = Math.max(top, p.y);
        pts.push(p);
      }
      roofTop = Math.max(roofTop ?? 0, top);
      for (const p of pts) if (p.y > top - 0.03) tops.push([p.x, p.y, p.z]);
    }
    if (tops.length >= 2) {
      // Along the ridge from one end to the other.
      let [a, b] = [tops[0], tops[1]];
      let far = 0;
      for (const p of tops) for (const q of tops) if (Math.hypot(p[0] - q[0], p[2] - q[2]) > far) [far, a, b] = [Math.hypot(p[0] - q[0], p[2] - q[2]), p, q];
      const n = Math.floor(far / 0.5);
      for (let k = 1; k < n; k++) ridge.push(a[0] + ((b[0] - a[0]) * k) / n, a[1], a[2] + ((b[2] - a[2]) * k) / n);
      // A short ridge (a hipped roof's): its ends, and the point between.
      if (n < 3) ridge.push(a[0], a[1], a[2], b[0], b[1], b[2]);
    }
    const minute = this.fixedHour !== null ? this.fixedHour * 60 : this.lastMinute;
    await this.birdLife.setHabitat({ world, view: v, plants, ridge, roofTop, hour: (minute / 60) % 24 });
  }

  /** The birds a frame on (with the game's clock; they're hidden with the garden over the basement). */
  private updateBirds(frame: FrameState, minute: number): void {
    const { curr, layout } = frame;
    const h = layout.header;
    const calmHours = h.calm !== undefined && curr[h.calm] > 0;
    this.birdLife.setVisible(this.storey >= 0);
    this.birdCamera.copyFrom(this.camera.globalPosition);
    const moved = this.birdLife.update({
      dt: this.frameDt / 1000,
      speed: calmHours ? 4 : curr[h.speed] | 0,
      hour: (minute / 60) % 24,
      camera: this.birdCamera,
      residents: this.characters.heads,
      residentCount: this.simCount,
      visible: this.characters.visible,
      calm: this.reducedMotion,
      roof: this.roofShown && this.storey >= 0,
    });
    // (As the ambient clips: drawn again, the sun's shadows not.)
    if (moved && this.webgpu?.snapshotRendering) this.webgpu.snapshotRenderingReset();
  }

  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean {
    this.viewport.width = this.cssSize.width;
    this.viewport.height = this.cssSize.height;
    this.vTmp.set(x, y, z);
    // The overview moves the camera just before drawing: project with its current matrices.
    const transform = this.townMode ? this.camera.getTransformationMatrix() : this.scene.getTransformMatrix();
    Vector3.ProjectToRef(this.vTmp, Matrix.IdentityReadOnly, transform, this.viewport, this.vProj);
    // Behind the camera, or not a point at all (a NaN position).
    if (!(this.vProj.z >= 0 && this.vProj.z <= 1 && Number.isFinite(this.vProj.x) && Number.isFinite(this.vProj.y))) return false;
    out.x = this.vProj.x;
    out.y = this.vProj.y;
    return true;
  }

  simHead(index: number, out: { x: number; y: number; z: number }): boolean {
    return index < this.simCount && this.characters.head(index, out);
  }

  focus(x: number, z: number): void {
    // Keep the viewing angle; move the camera with its target (setTarget's default
    // would keep the camera's position and swing the angles instead).
    const cam = this.gameCamera;
    cam.setTarget(new Vector3(x, 0, z), false, false, true);
    cam.radius = Math.min(cam.radius, 32);
    this.cameraPlaced = true;
  }

  setHoverTile(tile: { x: number; z: number } | null): void {
    if (tile) this.cursor.position.set(tile.x + 0.5, 0.025, tile.z + 0.5);
    else this.cursor.position.y = HIDDEN_Y;
  }

  setSelectedSim(id: number | null): void {
    this.selectedSim = id;
    this.characters?.select(id);
  }

  setSpeech(index: number, track: VisemeTrack | null, startTime: number): void {
    this.characters?.setSpeech(index, track, startTime);
  }

  setWallMode(mode: WallMode): void {
    this.wallMode = mode;
    this.applyWallMode();
    this.resetSnapshot();
  }

  configure(options: LiveRenderOptions): void {
    // Taken before the merge, which would otherwise hide the change.
    const shading = options.shading;
    this.options = { ...this.options, ...options };
    if (shading !== undefined && shading !== this.shading) {
      // The plugin sets its define while the shaders are prepared: marking the materials dirty is
      // enough, frozen or not, and the next frames draw the other way.
      this.shading = shading;
      refreshSoftLook();
      this.resetSnapshot();
    }
    const s = Math.max(0.25, options.cameraSensitivity);
    this.camera.angularSensibilityX = this.camera.angularSensibilityY = 1000 / s;
    this.camera.panningSensibility = 90 / s;
    this.camera.wheelDeltaPercentage = 0.012 * s;
    // Frames are paced in whole display refreshes (`pacing.ts`), not by Babylon's `maxFPS`.
    this.applyPacing();
    const still = (this.reducedMotion = !!options.reducedMotion);
    setWind(this.scene, still ? 0 : 1);
    this.characters.reducedMotion = still;
    this.fx.reducedMotion = still;
    this.applyLook();
  }

  // --- build mode ----------------------------------------------------------------------

  setPlacementGhost(ghost: PlacementGhost | null): void {
    // Something for a wall in hand: the things on walls show, so what's in its way does.
    const onWall = !!ghost?.wall;
    if (onWall !== this.ghostOnWall) {
      this.ghostOnWall = onWall;
      this.cutWallItems(Math.max(this.cutLook, 0));
      this.resetSnapshot();
    }
    if (!ghost) {
      this.ghostShown = false;
      if (this.ghostRoot.position.y !== HIDDEN_Y) {
        this.ghostRoot.position.y = HIDDEN_Y;
        this.ghostRoot.computeWorldMatrix(true);
      }
      return;
    }
    // A new thing in hand pops in; the same one resized just changes shape.
    const fresh = !this.ghostShown || ghost.model !== this.ghostModel;
    if (ghostKeyOf(ghost.model, ghost.def, ghost.resize, ghost.colors) !== this.ghostKey) this.loadGhost(ghost.model, ghost.def, ghost.resize, ghost.colors);
    // Rotate about the footprint centre (moved by its offset), like placed objects. The turn takes the short way round.
    const centre = objectCentre(ghost);
    this.ghostGoal.set(centre.x, 0, centre.z);
    this.ghostLift = ghost.lift ?? 0;
    const yaw = (ghost.rot * Math.PI) / 2 + ((ghost.turn ?? 0) * Math.PI) / 180;
    const turn = Math.atan2(Math.sin(yaw - this.ghostYawGoal), Math.cos(yaw - this.ghostYawGoal));
    this.ghostYawGoal += turn;
    if (!this.ghostShown) {
      this.ghostRoot.position.copyFrom(this.ghostGoal);
      this.ghostRoot.rotation.y = this.ghostYawGoal = yaw;
    }
    if (fresh) this.ghostSpawn = 0;
    this.ghostShown = true;
    // The arrow rides with the ghost but ignores its free turn: the use tile goes by quarter turns
    // (a seat used from beside or behind: it points away from the seat there).
    const arrow = this.ghostArrow;
    if (arrow.isEnabled() !== !!ghost.front) {
      arrow.setEnabled(!!ghost.front);
      this.resetSnapshot();
    }
    if (ghost.front) {
      const dx = ghost.front.x + 0.5 - this.ghostGoal.x;
      const dz = ghost.front.z + 0.5 - this.ghostGoal.z;
      const c = Math.cos(yaw);
      const s = Math.sin(yaw);
      // World offset into the ghost's turned frame; just above the floor below the floating ghost.
      arrow.position.set(dx * c - dz * s, -0.04, dx * s + dz * c);
      arrow.rotation.y = ((ghost.front.out - ghost.rot) * Math.PI) / 2 - ((ghost.turn ?? 0) * Math.PI) / 180;
    }
    if (ghost.valid !== this.ghostValid) {
      this.ghostValid = ghost.valid;
      this.setGhostTint();
    }
  }

  setHeldObject(id: number | null, moved = false): void {
    if (id === this.heldId) return;
    if (this.heldRelease) clearTimeout(this.heldRelease);
    this.heldRelease = null;
    this.heldId = id;
    if (id === null && moved) {
      // Put down elsewhere: the next world shows it there; a refused move shows it here again.
      this.heldRelease = setTimeout(() => ((this.heldRelease = null), this.showHeld()), 1500);
      return;
    }
    this.hideHeld();
  }

  private hideHeld(): void {
    if (this.held?.id === this.heldId) return;
    this.showHeld();
    const p = this.heldId === null ? undefined : this.placed.find((o) => o.id === this.heldId);
    if (!p) return;
    // What stands on its top goes with it.
    const riders = this.placed.filter((o) => o.host === p.id);
    const targets = [p, ...riders].flatMap((o) => (o.shadow ? [o.pop, o.shadow] : [o.pop]));
    this.held = { id: p.id, saved: targets.map((target) => ({ target, matrix: target.matrices.slice(target.index * 16, target.index * 16 + 16) })) };
    // Scaled to nothing where it stands (an all-zero matrix would put w = 0).
    for (const { target, matrix } of this.held.saved) {
      const m = matrix.slice();
      m.fill(0, 0, 12);
      this.writeInstance(target, m);
    }
  }

  private showHeld(): void {
    if (!this.held) return;
    for (const { target, matrix } of this.held.saved) this.writeInstance(target, matrix);
    this.held = null;
  }

  /** The tables the seats in the new world face (`tableRooms`), for the residents to pull their chairs up to; none pulled in yet. */
  private pullUpChairs(world: WorldStructure): void {
    this.pulledIn.clear();
    const byId = new Map(this.placed.map((p) => [p.id, p]));
    const rooms = tableRooms(world, this.deps.content, (id) => byId.get(id)?.pop, (o) => this.storeyOf(o.z) * WALL_HEIGHT);
    this.seatTurns = new Map(world.objects.filter((o) => rooms.has(o.id)).map((o) => [o.id, o.rot]));
    this.characters.setTables(rooms);
  }

  /** Draws the chairs residents sit pulled up to a table where they are now (`Characters.pulled`), and those let go back where they stand. */
  private pullChairs(): void {
    const pulled = this.characters.pulled;
    for (const [id, was] of this.pulledIn) {
      if (pulled.has(id)) continue;
      for (const { target, matrix } of was.saved) this.writeInstance(target, matrix);
      this.pulledIn.delete(id);
    }
    for (const [id, by] of pulled) {
      const was = this.pulledIn.get(id);
      if ((was && Math.abs(was.by - by) < 0.001) || this.held?.id === id) continue;
      const p = this.placed.find((o) => o.id === id);
      if (!p) continue;
      const targets = p.shadow ? [p.pop, p.shadow] : [p.pop];
      const saved = was?.saved ?? targets.map((target) => ({ target, matrix: target.matrices.slice(target.index * 16, target.index * 16 + 16) }));
      this.pulledIn.set(id, { by, saved });
      // Along its facing (the turn of its placement: the model's z).
      const yaw = ((this.seatTurns.get(id) ?? 0) * Math.PI) / 2;
      for (const { target, matrix } of saved) {
        const m = matrix.slice();
        m[12] += Math.sin(yaw) * by;
        m[14] += Math.cos(yaw) * by;
        this.writeInstance(target, m);
      }
    }
  }

  private writeInstance(target: PopTarget, matrix: Float32Array): void {
    target.matrices.set(matrix, target.index * 16);
    for (const mesh of target.meshes) {
      if (mesh.isDisposed()) continue;
      mesh.thinInstanceAllowAutomaticStaticBufferRecreation = true;
      mesh.thinInstanceBufferUpdated('matrix');
    }
    writeMoving(target);
    this.bakeObjectShadow(target);
    this.sunShadows.moved();
    this.resetSnapshot();
  }

  private updateGhost(dtMs: number): void {
    const dt = Math.min(dtMs, 50) / 1000;
    this.ghostAge += dt;
    this.ghostSpawn += dt;
    const root = this.ghostRoot;
    const glide = 1 - Math.exp(-dt * 22);
    root.position.x += (this.ghostGoal.x - root.position.x) * glide;
    root.position.z += (this.ghostGoal.z - root.position.z) * glide;
    root.rotation.y += (this.ghostYawGoal - root.rotation.y) * (1 - Math.exp(-dt * 16));
    // Held a hand's breadth above the floor (or the top it goes on), bobbing gently; picked up with a springy pop.
    // (Reduce motion: held still, no pop, no pulse.)
    const still = this.reducedMotion;
    root.position.y = this.ghostLift + 0.07 + (still ? 0 : 0.035 * Math.sin(this.ghostAge * 3.4));
    const t = this.ghostSpawn;
    const s = still || t > 0.6 ? 1 : 1 - Math.exp(-8 * t) * Math.cos(14 * t);
    root.scaling.set(s, s, s);
    // Where it can't go, it pulses.
    this.ghostMaterial.alpha = this.ghostValid ? 0.5 : still ? 0.36 : 0.36 + 0.14 * Math.sin(this.ghostAge * 9);
    this.ghostArrowMaterial.alpha = this.ghostMaterial.alpha + 0.25;
  }

  buildEffect(fx: BuildEffect): void {
    if (this.townMode) return;
    const k = this.storeyOf(fx.z);
    this.fx.play({ ...fx, z: fx.z - this.rowOf(k), y: k * WALL_HEIGHT });
    const placed = fx.objectId === undefined ? undefined : this.placed.find((p) => p.id === fx.objectId);
    if (placed && placed.id !== this.heldId) this.fx.pop(placed.pop);
  }

  setEdgePreview(edges: readonly EdgePreview[], valid: boolean): void {
    // Where they can go, walls, doors, windows, fences and gates show as built; removals (and what can't go) as films.
    const built = valid ? edges.filter((e) => e.kind !== 'open') : [];
    this.showBuilt(built);
    const counts = { wall: 0, door: 0, window: 0, open: 0 };
    for (const e of built.length ? edges.filter((e) => !built.includes(e)) : edges) {
      // Fences show as low walls, gates as low doors.
      const kind = e.kind === 'fence' ? 'wall' : e.kind === 'gate' ? 'door' : e.kind;
      let buf = this.previewBuffers[kind];
      const n = counts[kind]++;
      if ((n + 1) * 16 > buf.length) {
        const grown = new Float32Array(buf.length * 2);
        grown.set(buf);
        buf = this.previewBuffers[kind] = grown;
        this.previewMeshes[kind].thinInstanceSetBuffer('matrix', buf, 16, false);
      }
      // Unit slab (1 x 1 x 1 centred box) scaled to the edge: walls full height, doors a
      // door-sized slab, windows a glassy pane between sill and head, removals a red sleeve
      // just around the existing wall.
      const h =
        e.kind === 'fence' || e.kind === 'gate'
          ? FENCE_PREVIEW_HEIGHT
          : e.kind === 'door'
            ? 2.1
            : e.kind === 'window'
              ? 1.2
              : e.kind === 'open'
                ? WALL_HEIGHT + 0.06
                : e.form
                  ? HALF_WALL_HEIGHT
                  : WALL_HEIGHT;
      const y0 = e.kind === 'window' ? 0.9 : 0;
      const t = e.kind === 'open' ? 0.22 : e.kind === 'window' ? 0.2 : e.kind === 'fence' || e.kind === 'gate' ? 0.08 : 0.16;
      const o = n * 16;
      buf.fill(0, o, o + 16);
      if (e.axis === 'dp' || e.axis === 'dn') {
        // Diagonal across the tile: a √2-long slab turned ±45° about its centre (scale, then yaw).
        const c = Math.SQRT1_2;
        const sn = e.axis === 'dp' ? -c : c;
        buf[o] = Math.SQRT2 * c;
        buf[o + 2] = -Math.SQRT2 * sn;
        buf[o + 8] = t * sn;
        buf[o + 10] = t * c;
      } else {
        buf[o] = e.axis === 'h' ? 1 : t;
        buf[o + 10] = e.axis === 'h' ? t : 1;
      }
      const diagonal = e.axis === 'dp' || e.axis === 'dn';
      const cx = diagonal || e.axis === 'h' ? e.x + 0.5 : e.x;
      const cz = diagonal ? e.z + 0.5 : e.axis === 'h' ? e.z : e.z + 0.5;
      buf[o + 5] = h;
      buf[o + 12] = cx;
      buf[o + 13] = y0 + h / 2;
      buf[o + 14] = cz;
      buf[o + 15] = 1;
    }
    for (const kind of ['wall', 'door', 'window', 'open'] as const) {
      const mesh = this.previewMeshes[kind];
      mesh.thinInstanceCount = counts[kind];
      mesh.thinInstanceBufferUpdated('matrix');
      // Empty previews are disabled so they cost no draw call.
      if (mesh.isEnabled(false) !== counts[kind] > 0) {
        mesh.setEnabled(counts[kind] > 0);
        this.resetSnapshot();
      }
    }
    if (valid !== this.previewValid) {
      this.previewValid = valid;
      const tint = valid ? null : Color3.FromHexString('#E5484D');
      const colors = { wall: '#9FC4FF', door: '#B07A4A', window: '#7FE0F2' };
      for (const kind of ['wall', 'door', 'window'] as const) {
        const mat = this.previewMeshes[kind].material as StandardMaterial;
        mat.emissiveColor.copyFrom(tint ?? Color3.FromHexString(colors[kind]));
      }
      this.resetSnapshot();
    }
  }

  /**
   * Fences and gates in their style; walls, doors and windows as the house would build them
   * (standing while walls are down), the stretch of wall they go into hidden meanwhile.
   */
  private showBuilt(edges: readonly EdgePreview[]): void {
    const key = edges.length ? JSON.stringify([this.storey, this.wallMode, edges]) : '';
    if (key === this.builtPreviewKey) return;
    this.builtPreviewKey = key;
    for (const mesh of this.builtPending?.meshes ?? []) mesh.dispose(false, false);
    this.builtPending = null;
    const world = this.lastWorld;
    if (!edges.length || !world) {
      for (const mesh of this.builtPreview) mesh.dispose(false, false);
      this.builtPreview = [];
      this.lib.wallCut.holeSpan.x = 0;
      this.resetSnapshot();
      return;
    }
    // The shown preview stays until the next one can draw (no gap moving from tile to tile).
    const next: Mesh[] = [];
    let hole: number[] | null = null;
    const fences = edges.flatMap((e) => ((e.kind === 'fence' || e.kind === 'gate') && (e.axis === 'h' || e.axis === 'v') ? [{ axis: e.axis, x: e.x, z: e.z, kind: e.kind, style: e.style }] : []));
    const fenceMesh = buildFences(this.scene, fences, this.deps.content.fenceStyles)?.mesh;
    if (fenceMesh) next.push(fenceMesh);
    // A lot with just these walls (in the rooms as they are), the walls they go into keeping their looks.
    const W = world.width;
    const walls: WallEdge[] = [];
    const openings: Opening[] = [];
    const diagonals: DiagonalWall[] = [];
    const standing: EdgePreview[] = [];
    const previewed = new Set(edges.map((e) => `${e.axis}:${e.x}:${e.z}`));
    const opened = new Map((world.openings ?? []).map((o) => [`${o.axis}:${o.x}:${o.z}`, o]));
    const walled = new Map((world.walls ?? []).map((w) => [`${w.axis}:${w.x}:${w.z}`, w]));
    const slanted = new Map((world.diagonals ?? []).map((d) => [`${d.axis}:${d.x}:${d.z}`, d]));
    // With walls cut down, a door or window put into a wall raises the whole of it (the straight
    // run either side, as built) so it shows in the wall it goes into.
    const run: EdgePreview[] = [];
    if (this.wallMode !== 'up') {
      for (const e of edges) {
        if (e.kind !== 'door' && e.kind !== 'window') continue;
        const at = `${e.axis}:${e.x}:${e.z}`;
        if (!walled.has(at) && !slanted.has(at)) continue;
        const [dx, dz] = e.axis === 'h' ? [1, 0] : e.axis === 'v' ? [0, 1] : e.axis === 'dp' ? [1, 1] : [1, -1];
        for (const step of [-1, 1]) {
          for (let k = step; ; k += step) {
            const [x, z] = [e.x + dx * k, e.z + dz * k];
            const along = `${e.axis}:${x}:${z}`;
            if (previewed.has(along)) continue;
            const had = slanted.get(along) ?? opened.get(along) ?? walled.get(along);
            if (!had) break;
            previewed.add(along);
            run.push({ axis: e.axis, x, z, kind: 'kind' in had ? had.kind : 'wall', style: 'style' in had ? had.style : undefined });
          }
        }
      }
    }
    for (const e of [...edges, ...run]) {
      if (e.kind !== 'wall' && e.kind !== 'door' && e.kind !== 'window') continue;
      const faces: [number, number] | undefined = e.cover ? [e.cover, e.cover] : undefined;
      if (e.axis === 'dp' || e.axis === 'dn') {
        const had = slanted.get(`${e.axis}:${e.x}:${e.z}`);
        const r = world.rooms[e.z * W + e.x] ?? 0;
        if (had) standing.push(e);
        diagonals.push({ axis: e.axis, x: e.x, z: e.z, kind: e.kind, rooms: had?.rooms ?? [r, r], faces: faces ?? had?.faces, form: had ? had.form : e.form, style: e.style });
      } else {
        const had = walled.get(`${e.axis}:${e.x}:${e.z}`);
        if (had) standing.push(e);
        walls.push({ axis: e.axis, x: e.x, z: e.z, faces: faces ?? had?.faces, form: e.kind === 'wall' ? (had ? had.form : e.form) : undefined });
        if (e.kind !== 'wall') openings.push({ axis: e.axis, x: e.x, z: e.z, kind: e.kind, style: e.style, sill: e.sill, head: e.head });
      }
    }
    if (walls.length || diagonals.length) {
      const view = this.view ? { ...this.view, z: this.view.z + this.rowOf(this.storey) } : null;
      const house = this.house.build({ ...world, walls, openings, diagonals }, view, { upper: this.storey !== 0, preview: true, standing: world });
      if (house) next.push(...house.meshes);
      hole = this.holeFor(standing);
    }
    for (const mesh of next) {
      mesh.parent = this.storeyLayer;
      mesh.alwaysSelectAsActiveMesh = true;
      mesh.setEnabled(false);
    }
    this.builtPending = { meshes: next, hole };
    this.swapBuilt();
  }

  /** Shows the pending preview (and hides the wall it stands in) once all of it is ready to draw. */
  private swapBuilt(): void {
    const pending = this.builtPending;
    if (!pending || !pending.meshes.every((mesh) => mesh.isReady(true))) return;
    this.builtPending = null;
    for (const mesh of this.builtPreview) mesh.dispose(false, false);
    this.builtPreview = pending.meshes;
    for (const mesh of this.builtPreview) mesh.setEnabled(true);
    const cut = this.lib.wallCut;
    if (pending.hole) {
      const [x0, z0, x1, z1, w, y0, y1] = pending.hole;
      cut.hole.set(x0, z0, x1, z1);
      cut.holeSpan.set(w, y0, y1, 0);
    } else cut.holeSpan.x = 0;
    this.resetSnapshot();
  }

  /** The stretch of wall `edges` cover (in one line: a door or window and the wall it goes into), for `swapBuilt` to hide. */
  private holeFor(edges: readonly EdgePreview[]): number[] | null {
    // In the world, this storey's rows lie over the ground's (see `setStorey`).
    const shift = this.rowOf(this.storey);
    const ends = edges.map((e): [number, number, number, number] => {
      const t = e.axis === 'h' || e.axis === 'v' ? WALL_HALF : WALL_HALF * Math.SQRT1_2;
      const z = e.z - shift;
      if (e.axis === 'h') return [e.x + t, z, e.x + 1 - t, z];
      if (e.axis === 'v') return [e.x, z + t, e.x, z + 1 - t];
      if (e.axis === 'dp') return [e.x + t, z + t, e.x + 1 - t, z + 1 - t];
      return [e.x + t, z + 1 - t, e.x + 1 - t, z + t];
    });
    if (!ends.length) return null;
    // One segment from the first to the farthest end along the line (none for a bent run).
    const [x0, z0, x1, z1] = ends[0];
    const [ux, uz] = [x1 - x0, z1 - z0];
    const along = (x: number, z: number) => (x - x0) * ux + (z - z0) * uz;
    const off = (x: number, z: number) => Math.abs((x - x0) * uz - (z - z0) * ux);
    if (ends.some(([a, b, c, d]) => off(a, b) > 1e-6 || off(c, d) > 1e-6)) return null;
    const points = ends.flatMap(([a, b, c, d]) => [[a, b], [c, d]]);
    const lo = points.reduce((m, p) => (along(p[0], p[1]) < along(m[0], m[1]) ? p : m));
    const hi = points.reduce((m, p) => (along(p[0], p[1]) > along(m[0], m[1]) ? p : m));
    // Within reach of trim and sills (a crossing wall loses no more than that); this storey's walls only.
    const y0 = this.storey * WALL_HEIGHT;
    return [lo[0], lo[1], hi[0], hi[1], WALL_HALF + 0.08, this.storey ? y0 + 0.005 : -1, y0 + WALL_HEIGHT + 0.005];
  }

  setPaintPreview(faces: readonly PaintPreviewFace[], cover: number | null): void {
    const real = this.showLaid('faces', faces.length ? cover : null, faces.map((f) => `${f.axis}:${f.x}:${f.z}:${f.side}`));
    // The automatic look (0) depends on the room, so a white film stands in for it.
    const color = real || cover === null ? null : ((cover ? this.deps.content.wallCoverings[cover - 1]?.color : undefined) ?? '#FFFFFF');
    this.paintFilm(faces, color);
  }

  setFloorPreview(tiles: readonly { x: number; z: number }[], floor: number | null): void {
    const real = this.showLaid('floors', tiles.length ? floor : null, tiles.map((t) => `${t.x}:${t.z}`));
    const color = real || floor === null ? null : ((floor ? this.deps.content.floorCoverings[floor - 1]?.color : undefined) ?? '#FFFFFF');
    this.floorFilm(tiles, color);
  }

  /**
   * Shows `keys` (wall faces or floor tiles in `lotSurfaces`) as covering `cover` lays them, in its
   * real material. False where it can't (no covering, the automatic look, no house surfaces).
   */
  private showLaid(which: 'faces' | 'floors', cover: number | null, keys: string[]): boolean {
    const slot = this.laid[which];
    slot.keys = keys;
    slot.cover = cover;
    const surfaces = this.lotSurfaces?.[which];
    const shown = cover && surfaces ? `${cover}|${keys.join(',')}` : '';
    if (shown !== slot.shown) {
      slot.shown = shown;
      slot.mesh?.dispose();
      slot.mesh = null;
      if (cover && surfaces) {
        const parts = keys.flatMap((k) => surfaces.get(k) ?? []);
        slot.mesh = which === 'faces' ? this.house.coverPreview(parts, cover) : this.house.floorPreview(parts, cover);
        if (slot.mesh) slot.mesh.parent = this.storeyLayer;
      }
      this.resetSnapshot();
    }
    return shown !== '';
  }

  /** The house was rebuilt: lay the shown previews on its new surfaces. */
  private refreshLaid(): void {
    for (const which of ['faces', 'floors'] as const) {
      const slot = this.laid[which];
      if (!slot.shown) continue;
      slot.shown = '';
      this.showLaid(which, slot.cover, slot.keys);
    }
  }

  /** The Paint tool's stand-in: a film of `color` over each face (null: none). */
  private paintFilm(faces: readonly PaintPreviewFace[], color: string | null): void {
    const mesh = this.paintPreview;
    const n = color ? faces.length : 0;
    if (n * 16 > this.paintBuffer.length) {
      this.paintBuffer = new Float32Array(Math.max(n, this.paintBuffer.length / 8) * 32);
      mesh.thinInstanceSetBuffer('matrix', this.paintBuffer, 16, false);
    }
    const buf = this.paintBuffer;
    // A thin film just off each face (walls are 0.14 m thick), as tall as the wall.
    const off = 0.078;
    const t = 0.012;
    faces.slice(0, n).forEach((f, i) => {
      const o = i * 16;
      const h = f.half ? HALF_WALL_HEIGHT : WALL_HEIGHT;
      const s = f.side ? 1 : -1;
      buf.fill(0, o, o + 16);
      let cx: number;
      let cz: number;
      if (f.axis === 'h' || f.axis === 'v') {
        buf[o] = f.axis === 'h' ? 1 : t;
        buf[o + 10] = f.axis === 'h' ? t : 1;
        cx = f.axis === 'h' ? f.x + 0.5 : f.x + s * off;
        cz = f.axis === 'h' ? f.z + s * off : f.z + 0.5;
      } else {
        // As in `setEdgePreview`: a √2-long film turned ±45°, offset towards half 1 (`s` > 0).
        const c = Math.SQRT1_2;
        const sn = f.axis === 'dp' ? -c : c;
        buf[o] = Math.SQRT2 * c;
        buf[o + 2] = -Math.SQRT2 * sn;
        buf[o + 8] = t * sn;
        buf[o + 10] = t * c;
        cx = f.x + 0.5 + s * off * sn;
        cz = f.z + 0.5 + s * off * c;
      }
      buf[o + 5] = h;
      buf[o + 12] = cx;
      buf[o + 13] = h / 2;
      buf[o + 14] = cz;
      buf[o + 15] = 1;
    });
    mesh.thinInstanceCount = n;
    mesh.thinInstanceBufferUpdated('matrix');
    if (color) (mesh.material as StandardMaterial).emissiveColor.copyFrom(Color3.FromHexString(color));
    if (mesh.isEnabled(false) !== n > 0) {
      mesh.setEnabled(n > 0);
      this.resetSnapshot();
    }
  }

  /** The Floor tool's stand-in: a film of `color` over each tile (null: none). */
  private floorFilm(tiles: readonly { x: number; z: number }[], color: string | null): void {
    const mesh = this.floorPreview;
    const n = color ? tiles.length : 0;
    if (n * 16 > this.floorBuffer.length) {
      this.floorBuffer = new Float32Array(Math.max(n, this.floorBuffer.length / 8) * 32);
      mesh.thinInstanceSetBuffer('matrix', this.floorBuffer, 16, false);
    }
    const buf = this.floorBuffer;
    // A film just above the floor (floors lie at 0.01 m), a hair inside each tile so tiles read.
    tiles.slice(0, n).forEach((t, i) => {
      const o = i * 16;
      buf.fill(0, o, o + 16);
      buf[o] = 0.96;
      buf[o + 5] = 1;
      buf[o + 10] = 0.96;
      buf[o + 12] = t.x + 0.5;
      buf[o + 13] = 0.03;
      buf[o + 14] = t.z + 0.5;
      buf[o + 15] = 1;
    });
    mesh.thinInstanceCount = n;
    mesh.thinInstanceBufferUpdated('matrix');
    if (color) (mesh.material as StandardMaterial).emissiveColor.copyFrom(Color3.FromHexString(color));
    if (mesh.isEnabled(false) !== n > 0) {
      mesh.setEnabled(n > 0);
      this.resetSnapshot();
    }
  }

  setRoomOverlay(tiles: readonly RoomOverlayTile[]): void {
    const n = tiles.length;
    if (!this.roomOverlay && n === 0) return;
    let mesh = this.roomOverlay;
    if (!mesh) {
      // One film per tile, tinted per instance; just under the floor tool's film.
      mesh = MeshBuilder.CreateGround('room-overlay', { width: 1, height: 1 }, this.scene);
      const mat = new StandardMaterial('room-overlay', this.scene);
      mat.disableLighting = true;
      mat.emissiveColor = Color3.White();
      mat.alpha = 0.45;
      mesh.material = mat;
      mesh.isPickable = false;
      mesh.alwaysSelectAsActiveMesh = true;
      mesh.parent = this.storeyLayer;
      this.roomOverlay = mesh;
    }
    if (n * 16 > this.roomMatrices.length) {
      const cap = Math.max(n, 64) * 2;
      this.roomMatrices = new Float32Array(cap * 16);
      this.roomColors = new Float32Array(cap * 4);
      mesh.thinInstanceSetBuffer('matrix', this.roomMatrices, 16, false);
      mesh.thinInstanceSetBuffer('color', this.roomColors, 4, false);
    }
    const m = this.roomMatrices;
    const c = this.roomColors;
    tiles.forEach((t, i) => {
      const o = i * 16;
      m.fill(0, o, o + 16);
      m[o] = 1;
      m[o + 5] = 1;
      m[o + 10] = 1;
      m[o + 12] = t.x + 0.5;
      m[o + 13] = 0.025;
      m[o + 14] = t.z + 0.5;
      m[o + 15] = 1;
      c.set([t.rgb[0], t.rgb[1], t.rgb[2], 1], i * 4);
    });
    mesh.thinInstanceCount = n;
    mesh.thinInstanceBufferUpdated('matrix');
    mesh.thinInstanceBufferUpdated('color');
    if (mesh.isEnabled(false) !== n > 0) mesh.setEnabled(n > 0);
    this.resetSnapshot();
  }

  /** Build tools: whether a left-drag turns the camera (off while a wall is being drawn). */
  setLeftDragCamera(on: boolean): void {
    const pointers = this.camera.inputs.attached.pointers as ArcRotateCameraPointersInput | undefined;
    if (pointers) pointers.buttons = on ? [0, 1, 2] : [1, 2];
  }

  setStorey(storey: number): void {
    const k = Math.max(this.storeys.lowest, Math.min(this.storeys.count - 1, Math.round(storey)));
    if (k === this.storey && this.storeyLayer) return;
    this.storey = k;
    this.storeyLayer?.position.set(0, k * WALL_HEIGHT, -this.rowOf(k));
    this.applyStoreys();
    // The interior lights go to the storey now in view (also while a build is running, which skips the rebuild below).
    if (this.lastWorld) {
      this.placeRoomLights(this.lastWorld, this.view);
      this.applyRoomLights();
    }
    // Furniture and residents above it hide; the lot (every storey) is kept.
    if (this.lastWorld && !this.worldBuilding) void this.setWorld(this.lastWorld, this.view);
  }

  /** The storey a lot row is on (-1: the basement). */
  private storeyOf(z: number): number {
    return Math.max(this.storeys.lowest, Math.min(this.storeys.count - 1, storeyOfRow(this.storeys.layout, z)));
  }

  /** The first lot row of storey `k`. */
  private rowOf(k: number): number {
    return storeyRow(this.storeys.layout, k);
  }

  /** Roofs show with walls up, over the storeys in view. */
  private showRoofs(): void {
    for (const roof of this.roofs) roof.setEnabled(this.roofShown && storeyShown((roof.metadata as { storey?: number } | null)?.storey ?? 0, this.storey));
    this.sunShadows.moved();
  }

  /** Storeys above the one in view are hidden (walls, floors, roofs). */
  private applyStoreys(): void {
    for (const mesh of [...this.lotMeshes, ...this.worldMeshes]) {
      const k = (mesh.metadata as { storey?: number } | null)?.storey;
      if (k === undefined) continue;
      const roof = this.roofs.includes(mesh);
      mesh.setEnabled(storeyShown(k, this.storey) && (!roof || this.roofShown));
    }
    for (const mesh of this.decorMeshes) {
      if (mesh.isDisposed()) this.decorMeshes.delete(mesh);
      else mesh.isVisible = this.storey >= 0;
    }
    this.applyDig();
    this.sunShadows.moved();
    this.resetSnapshot();
  }

  setBuildGrid(rect: ViewRect | null): void {
    const on = rect !== null;
    if (rect) {
      this.grid.scaling.set(rect.w, 1, rect.d);
      this.grid.position.set(rect.x + rect.w / 2, 0.03, rect.z + rect.d / 2);
      const tex = (this.grid.material as StandardMaterial).diffuseTexture as DynamicTexture;
      tex.uScale = rect.w;
      tex.vScale = rect.d;
    }
    if (on !== this.gridOn) {
      this.gridOn = on;
      this.grid.setEnabled(on);
      this.resetSnapshot();
    }
  }

  async captureThumbnail(width: number, height: number): Promise<string | null> {
    this.drawOnce();
    try {
      return await CreateScreenshotAsync(this.engine, this.camera, { width, height }, 'image/jpeg', 0.8);
    } catch (err) {
      console.warn('[render] thumbnail capture failed', err);
      return null;
    }
  }

  cameraPose(): CameraPose {
    const cam = this.gameCamera;
    return { alpha: cam.alpha, beta: cam.beta, radius: cam.radius, target: [cam.target.x, cam.target.y, cam.target.z] };
  }

  setCameraPose(pose: CameraPose): void {
    const cam = this.gameCamera;
    cam.setTarget(new Vector3(...pose.target), false, false, true);
    cam.alpha = pose.alpha;
    cam.beta = pose.beta;
    cam.radius = pose.radius;
    this.cameraPlaced = true;
  }

  stats(): RenderStats {
    return {
      backend: `${this.webgpu ? 'WebGPU' : 'WebGL2'}${this.lampCluster ? ' · clustered lamps' : ''}`,
      preset: this.quality.preset,
      fps: this.engine.getFps(),
      frameMs: this.engine.getDeltaTime(),
      cpuMs: this.cpuMs,
      drawCalls: this.instrumentation.drawCallsCounter.current,
      snapshotRendering: this.webgpu?.snapshotRendering ?? false,
      renderWidth: this.engine.getRenderWidth(),
      renderHeight: this.engine.getRenderHeight(),
      renderScale: 1 / (this.engine.getHardwareScalingLevel() * window.devicePixelRatio),
      refreshHz: this.pacer.refreshHz,
      refreshDivisor: this.pacer.divisor,
      gpuMs: this.gpuMs(),
      residentsMs: this.characters?.updateMs ?? 0,
    };
  }

  setFrameProbe(probe: FrameProbe | null): void {
    for (const off of this.probeOff) off();
    this.probeOff = [];
    this.probe = probe;
    if (!probe) return;
    // Draw calls and triangles of the shadow map and the reflections (both counted for the whole frame too).
    const draws = this.instrumentation.drawCallsCounter;
    const watch = (rtt: RenderTargetTexture, kind: 'shadow' | 'mirror') => {
      let calls = 0;
      let indices = 0;
      const before = rtt.onBeforeRenderObservable.add(() => {
        calls = draws.current;
        indices = this.scene.getActiveIndices();
      });
      const after = rtt.onAfterRenderObservable.add(() => {
        this.counts[`${kind}DrawCalls`] += draws.current - calls;
        this.counts[`${kind}Triangles`] += (this.scene.getActiveIndices() - indices) / 3;
      });
      this.probeOff.push(() => (rtt.onBeforeRenderObservable.remove(before), rtt.onAfterRenderObservable.remove(after)));
    };
    const map = this.shadows.getShadowMap();
    if (map) watch(map, 'shadow');
    for (const texture of this.mirrors.textures) watch(texture, 'mirror');
    // (Reflections are made when a mirror first reflects.)
    this.mirrors.onCreated = (texture) => watch(texture, 'mirror');
    this.probeOff.push(() => (this.mirrors.onCreated = null));
  }

  measureGpu(on: boolean, who: 'overlay' | 'bench' = 'overlay'): void {
    if (on) this.gpuWanted.add(who);
    else this.gpuWanted.delete(who);
    const want = this.gpuWanted.size > 0;
    if (this.webgpu) {
      // Needs the device's 'timestamp-query' feature (asked for in `loaders.ts`, where the adapter has it).
      if (!this.engine.getCaps().timerQuery || this.webgpu.enableGPUTimingMeasurements === want) return;
      this.webgpu.enableGPUTimingMeasurements = want;
      this.gpuSpans.clear();
      this.gpuRecent.length = 0;
      if (want) {
        this.timePasses();
        this.spanPasses();
      }
    } else if (want !== this.glTimer) {
      // WebGL2: EXT_disjoint_timer_query_webgl2 where the browser offers it (the query code loads on first use).
      this.glTimer = want;
      void Promise.all([import('@babylonjs/core/Engines/AbstractEngine/abstractEngine.timeQuery'), import('@babylonjs/core/Engines/Extensions/engine.query')]).then(() => {
        // SAFETY: the WebGL2 query extension adds `captureGPUFrameTime` to the engine at runtime.
        const engine = this.engine as unknown as { captureGPUFrameTime(on: boolean): void };
        engine.captureGPUFrameTime(this.glTimer);
      });
    }
  }

  /** GPU time of the whole frame, about the last second's average (ms), while measured. */
  private gpuMs(): number | null {
    if (this.webgpu) return this.gpuRecent.length ? this.gpuRecent.reduce((a, x) => a + x, 0) / this.gpuRecent.length : null;
    // SAFETY: Babylon's WebGL2 timer keeps its running counter in the private `_gpuFrameTime`.
    const counter = this.glTimer ? (this.engine as unknown as { _gpuFrameTime?: { count: number; lastSecAverage: number } })._gpuFrameTime : undefined;
    return counter && counter.count > 0 ? counter.lastSecAverage / 1e6 : null;
  }

  /**
   * WebGPU: the GPU's whole frame is from its first pass's start to its last pass's end. (Babylon
   * times the passes one by one, and its frame timer stamps the command encoders, which on Apple
   * GPUs misses the drawing; their passes overlap, so their own times add up to more than the frame.)
   * Reads each pass's start and end as Babylon's query set resolves them.
   */
  private spanPasses(): void {
    type QuerySet = { readTwoValuesAndSubtract(first: number): Promise<number | null>; _getBuffer(first: number, count: number): GPUBuffer | null; _dstBuffers: GPUBuffer[]; spans?: true };
    const engine = this.webgpu!;
    // SAFETY: WebGPUEngine's timestamp-query state is private (`_timestampQuery`); its shape is stable in 9.x.
    const current = () => (engine as unknown as { _timestampQuery: { _enabled: boolean; _measureDuration?: { _querySet?: QuerySet } } })._timestampQuery;
    const set = current()._measureDuration?._querySet;
    if (!set || set.spans || typeof set._getBuffer !== 'function') return;
    set.spans = true;
    // Babylon reads a pass's two stamps as the pass ends, before the frame's commands are sent: that
    // gets what the stamps' slots held from an earlier frame, from another pass where the passes
    // differ from frame to frame (a reflection drawn now and then). Read once the frame is sent.
    set.readTwoValuesAndSubtract = (first: number) => {
      const frame = engine.frameId;
      return new Promise<number | null>((resolve) => engine.onEndFrameObservable.addOnce(() => resolve(read(first, frame))));
    };
    const read = async (first: number, frame: number) => {
      // (Timing turned off meanwhile: the query set is gone.)
      const timer = current();
      if (!timer._enabled || timer._measureDuration?._querySet !== set) return null;
      const buffer = set._getBuffer(first, 2);
      if (!buffer) return null;
      try {
        await buffer.mapAsync(1 /* GPUMapMode.READ */);
      } catch {
        return 0;
      }
      const [begin, end] = new BigUint64Array(buffer.getMappedRange());
      buffer.unmap();
      set._dstBuffers.push(buffer);
      // (Queries 0 and 1 are the encoder stamps of Babylon's frame timer.)
      if (first >= 2 && end > begin) {
        const span = this.gpuSpans.get(frame);
        if (!span) this.gpuSpans.set(frame, [begin, end]);
        else {
          if (begin < span[0]) span[0] = begin;
          if (end > span[1]) span[1] = end;
        }
      }
      return Number(end - begin);
    };
  }

  /** Frames whose pass times have all come back (a few frames late): their whole spans, ms. */
  private settleSpans(): void {
    const settled = this.engine.frameId - 6;
    for (const [frame, [begin, end]] of this.gpuSpans) {
      if (frame > settled) continue;
      this.gpuSpans.delete(frame);
      const ms = Number(end - begin) / 1e6;
      this.counts.gpuFrames++;
      this.counts.gpuMs = ms;
      this.gpuRecent.push(ms);
      if (this.gpuRecent.length > 60) this.gpuRecent.shift();
    }
  }

  /** Every render target Babylon has made for this engine (its private wrapper cache). */
  private targetWrappers(engine: AbstractEngine): RenderTargetWrapper[] {
    // SAFETY: Babylon keeps every render-target wrapper in the engine's private `_renderTargetWrapperCache`.
    return (engine as unknown as { _renderTargetWrapperCache: RenderTargetWrapper[] })._renderTargetWrapperCache;
  }

  /** WebGPU: a GPU timer on every render target (Babylon gives one only to targets made while timing is on). */
  private timePasses(reset = false): void {
    for (const target of this.targetWrappers(this.engine)) {
      const timed = target as RenderTargetWrapper & { gpuTimeInFrame?: WebGPUPerfCounter };
      if (reset || !timed.gpuTimeInFrame) timed.gpuTimeInFrame = new WebGPUPerfCounter();
    }
    if (reset && this.webgpu) (this.webgpu as { gpuTimeInFrameForMainPass?: WebGPUPerfCounter }).gpuTimeInFrameForMainPass = new WebGPUPerfCounter();
  }

  gpuPasses(reset = false): { name: string; ms: number; frames: number }[] {
    const engine = this.webgpu;
    if (!engine?.enableGPUTimingMeasurements) return [];
    const out: { name: string; ms: number; frames: number }[] = [];
    const add = (name: string, timer: WebGPUPerfCounter | undefined) => {
      const c = timer?.counter;
      if (c && c.count > 0) out.push({ name, ms: c.average / 1e6, frames: c.count });
    };
    add('canvas', engine.gpuTimeInFrameForMainPass);
    for (const target of this.targetWrappers(engine)) {
      add(target.label ?? '?', (target as RenderTargetWrapper & { gpuTimeInFrame?: WebGPUPerfCounter }).gpuTimeInFrame);
    }
    this.timePasses(reset);
    return out;
  }

  benchInfo(): Record<string, unknown> & { passes: RenderPassInfo[] } {
    const engine = this.engine;
    const scene = this.scene;
    const map = this.shadows.getShadowMap();
    const passes: RenderPassInfo[] = [];
    const target = (name: string, rtw: RenderTargetWrapper | null | undefined): RenderPassInfo | null =>
      rtw ? { name, width: rtw.width, height: rtw.height, samples: rtw.samples ?? 1, colorBytes: textureBytes(rtw.texture?.type, rtw.texture?.format), depthBytes: depthBytes(rtw) } : null;
    const push = (pass: RenderPassInfo | null) => pass && passes.push(pass);
    if (this.sun.shadowEnabled && map?.renderList?.length) push(target('shadow', map.renderTarget));
    const live = this.mirrors.live;
    this.mirrors.reflecting.forEach((t, i) => push(target(`mirror${i}`, t.renderTarget)));
    // The scene draws into the first post-process's input, each post-process into the next one's, the last onto the canvas.
    const chain = (this.camera._postProcesses ?? []).filter((pp) => !!pp);
    if (chain.length) push(target('scene', chain[0]!.inputTexture));
    chain.forEach((pp, i) => {
      const next = chain[i + 1];
      if (next) push(target(pp!.name, next.inputTexture));
      else passes.push({ name: `${pp!.name} (canvas)`, width: engine.getRenderWidth(), height: engine.getRenderHeight(), samples: 1, colorBytes: 4, depthBytes: 4 });
    });
    const casters = (map?.renderList ?? []).filter((m) => m.isEnabled() && m.isVisible) as Mesh[];
    const triangles = (m: Mesh) => (m.getTotalIndices() / 3) * (m.hasThinInstances ? m.thinInstanceCount : 1);
    const nature = (this.natureIn.world?.meshes ?? []).filter((m) => m.layerMask & LAYER_WORLD);
    // (GPUAdapterInfo's fields are getters: copied one by one.)
    // SAFETY: WebGPUEngine keeps the adapter's info in the private `_adapterInfo`; its fields are getters.
    const adapter = (this.webgpu as unknown as { _adapterInfo?: Record<string, string> } | null)?._adapterInfo;
    return {
      backend: this.webgpu ? 'WebGPU' : 'WebGL2',
      gpu: adapter ? { vendor: adapter.vendor, architecture: adapter.architecture, device: adapter.device, description: adapter.description } : (engine as Engine).getGlInfo(),
      timerQuery: !!engine.getCaps().timerQuery,
      render: { width: engine.getRenderWidth(), height: engine.getRenderHeight(), hardwareScaling: engine.getHardwareScalingLevel(), devicePixelRatio: window.devicePixelRatio, canvasMsaa: false },
      quality: { ...this.quality, mirrors: this.mirrors.capacity, clusteredLamps: !!this.lampCluster, postSamples: this.post?.samples ?? 1, fxaa: this.post?.fxaaEnabled ?? false },
      diag: [...this.diag],
      passes,
      targets: this.targetWrappers(engine).map((t) => target(t.label ?? '?', t)),
      scene: {
        meshes: scene.meshes.length,
        enabledMeshes: scene.meshes.filter((m) => m.isEnabled()).length,
        activeMeshes: scene.getActiveMeshes().length,
        materials: scene.materials.length,
        textures: scene.textures.length,
        lights: scene.lights.filter((l) => l.isEnabled()).length,
        lampsLit: this.roomLights.filter((l) => l.intensity > 0).length,
        shadowCasters: casters.length,
        shadowCasterTriangles: Math.round(casters.reduce((n, m) => n + triangles(m), 0)),
        natureMeshes: nature.filter((m) => m.isEnabled()).length,
        natureInstances: nature.reduce((n, m) => n + (m.isEnabled() && m.hasThinInstances ? m.thinInstanceCount : 0), 0),
        residents: this.simCount,
        residentsDrawn: this.characters ? this.characters.visible.subarray(0, this.simCount).reduce((n, v) => n + (v ? 1 : 0), 0) : 0,
        mirrors: this.mirrors.count,
        mirrorsLive: live,
        mirrorSpots: this.mirrors.spots,
        mirrorDraws: this.mirrors.draws,
        nightGrade: this.nightGrade?.amount ?? 0,
        lightingUpdates: this.counts.lightingUpdates,
      },
    };
  }

  run(onFrame: (now: number) => void, onView?: () => void): void {
    this.onFrame = onFrame;
    this.onView = onView ?? null;
    this.setActive(true);
  }

  setActive(active: boolean): void {
    if (active === this.active) return;
    this.active = active;
    if (active) this.engine.runRenderLoop(this.renderFrame);
    else this.engine.stopRenderLoop(this.renderFrame);
  }

  hold(on: boolean): void {
    if (on === this.covered) return;
    this.covered = on;
    this.coveredFrames = HOLD_FRAMES;
  }

  /** Behind a full-screen menu, one more frame: a screenshot reads the canvas once drawn, and a resized canvas is blank. */
  private drawOnce(): void {
    if (this.covered) this.coveredFrames = Math.max(this.coveredFrames, 1);
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

  setGameShot(shot: GameShot | null, cut = false): void {
    if (!shot) return this.gameRig.release();
    this.followIndex = null;
    this.gameRig.setShot(shot, cut || this.townMode);
    this.cameraPlaced = true;
  }

  followSim(index: number | null): void {
    this.followIndex = index;
  }

  /**
   * Keeps the game camera above the land round the town (mountains rise close to it in some
   * worlds): looking from higher up instead of from inside a hill.
   */
  private keepCameraClear(): void {
    const land = this.natureIn.world?.landscape;
    if (this.townMode || !land) return;
    const cam = this.camera;
    for (let k = 0; k < 20 && cam.beta > 0.3; k++) {
      const p = cam.position;
      if (p.y >= land.heightAt(p.x, p.z) + 4) break;
      cam.beta = Math.max(0.3, cam.beta - 0.03);
      cam.inertialBetaOffset = 0;
      cam.getViewMatrix(true);
    }
  }

  /** The director's shot, or keeping a followed resident centred (game world only). */
  private steerCamera(dt: number): void {
    if (this.gameRig.active) return this.gameRig.frame(dt);
    const i = this.followIndex;
    // Off the drawn lot (on the street, or arriving where the next lot is still being built):
    // where they stand, so the camera keeps going with them.
    if (i === null || (!this.simHead(i, this.followHead) && !(i < this.simCount && this.characters.spot(i, this.followHead)))) return;
    const t = this.camera.target;
    const k = 1 - Math.exp(-dt / 0.5);
    t.x += (this.followHead.x - t.x) * k;
    t.z += (this.followHead.z - t.z) * k;
  }

  clear(): void {
    this.pauseSnapshot();
    for (const mesh of [...this.worldMeshes, ...this.lotMeshes]) mesh.dispose(false, false);
    this.worldMeshes = [];
    this.lotMeshes = [];
    this.lotKey = '';
    this.lotCasters = [];
    this.doors = [];
    this.gates = [];
    this.dimmed = new Set();
    this.windowless = { rooms: new Set(), of: [] };
    this.dimming = null;
    this.roomTiles = null;
    this.applyDimming();
    this.lotTemplates.clear();
    this.lotRooms = null;
    this.lotParts = null;
    this.roofs = [];
    this.baseCasters = [];
    this.wallCasters = [];
    this.walls = this.wallsLow = null;
    this.retireTemplates(new Set());
    // Those built without their files are built (and their files tried) afresh in the next session,
    // and no session's world is built again for a file that comes meanwhile (`refreshModels`).
    this.templates.dropFailed();
    this.lastWorld = null;
    this.placed = [];
    this.motion.clear();
    this.wallItemsHidden.clear();
    this.held = null;
    this.characters?.clear();
    this.items?.clear();
    this.mirrors?.set([]);
    this.simCount = 0;
    this.selectedSim = null;
    this.onFrame = null;
    this.onView = null;
    this.lastFrame = null;
    this.cursor.position.y = HIDDEN_Y;
    this.markerMatrix.fill(0);
    this.markerHaloMatrix.fill(0);
    this.simShadowMatrices.fill(0);
    this.setPlacementGhost(null);
    this.setEdgePreview([], true);
    this.setPaintPreview([], null);
    this.setFloorPreview([], null);
    this.setRoomOverlay([]);
    this.setBuildGrid(null);
    this.cameraPlaced = false;
    this.framedView = '';
    this.warmFrames = 0;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.onFailure = null;
    this.townTicket++;
    this.townListener = null;
    this.onFrame = null;
    this.onView = null;
    // Babylon's own loss handler keeps the engine (and its renderer) in a promise that may never
    // settle; `watchDeviceLoss` uses a WeakRef instead. Drop the hooks the engine still holds on us.
    this.engine.onContextLostObservable.clear();
    this.engine.customAnimationFrameRequester = null;
    delete (this.engine as unknown as { _measureFps?: () => void })._measureFps;
    const debug = window as unknown as { __renderer?: unknown; __scene?: unknown };
    if (debug.__renderer === this) delete debug.__renderer;
    if (debug.__scene === this.scene) delete debug.__scene;
    window.removeEventListener('resize', this.onResize);
    this.sizeWatch?.disconnect();
    this.unwatchPixelRatio?.();
    this.unlistenFiles?.();
    this.unlistenMarker?.();
    this.birdLife?.dispose();
    if (this.filesTimer) clearTimeout(this.filesTimer);
    this.filesTimer = null;
    this.filesArrived.clear();
    this.characters?.dispose();
    this.items?.dispose();
    this.overview?.dispose();
    // The scene owns the meshes, materials and its own cleanup (file retries, the material
    // library's timer, observers); the engine alone does not dispose it. (The per-scene caches are
    // weak maps: their entries go with the scene.)
    this.scene.dispose();
    // (Babylon's WebGPU error listener on the device goes as the engine destroys it: `loaders`.)
    this.engine.dispose();
  }

  // --- town overview (menus) ---------------------------------------------------------

  get townShown(): boolean {
    return this.townMode;
  }

  /** Builds the overview of `world` (if it isn't already), with its highlights and out of the house's lamps. */
  private async buildOverview(overview: TownOverview, world: WorldStructure): Promise<void> {
    await overview.build(world, () => this.pace());
    if (this.townHighlight) overview.setHighlight(this.townHighlight);
    // Interior lamps of the game's house never reach the overview.
    this.lib.excludeFromLights([...this.roomLights, ...(this.lampCluster ? [this.lampCluster] : [])], new Set<Mesh>(overview.meshes));
  }

  showTown(world: WorldStructure, minute: number): Promise<void> {
    const ticket = ++this.townTicket;
    this.townWorld = world;
    const run = this.townQueue.then(async () => {
      // Requests queue up while one builds; only the newest is built.
      if (ticket !== this.townTicket) return;
      const overview = (this.overview ??= new TownOverview(this.scene, this.deps.assets, this.deps.content, this.house, this.lib, this.camera));
      this.beginBuild();
      try {
        // Same seed and size can still be a new layout ("New neighbours"): re-fit the lawn to its houses.
        perfPhase('render: town nature');
        const nature = await perfTimeAsync('render: town nature', () => this.useNature('town', world));
        perfTime('nature: refreshLawn', () => nature.landscape.refreshLawn(world));
        // The camera keeps a clear view over the hills (and out of the mountains).
        overview.rig.ground = (x, z) => nature.landscape.heightAt(x, z);
        perfPhase('render: town overview');
        await perfTimeAsync('render: town overview', () => this.buildOverview(overview, world));
      } finally {
        this.endBuild();
      }
      if (ticket !== this.townTicket) return;
      if (!this.townMode) {
        // Entering from the game (or at start-up): the shot's hour at once.
        this.townMinute = this.townMinuteGoal = this.townShot?.minute ?? minute;
        this.enterTown();
      }
      else {
        this.updateShadowList();
        this.resetSnapshot();
      }
      // The overview is on screen now, but its effects may still be compiling: the menu's first
      // frames would then create their pipelines and drop frames. Warm them behind the loading
      // screen instead (the build above skipped this, as `townMode` only turns on in `enterTown`).
      this.beginBuild();
      try {
        await perfTimeAsync('render: menu warm shaders', () => this.warmShaders(performance.now() + TEXTURE_WAIT_MS));
        // The mirror pass compiles clip-plane variants of the sky and cloud band, which exist once
        // for the page: draw them into a tiny reflection now, behind the loading screen, so the
        // first household's mirror does not pay that compile on its own loading screen.
        await perfTimeAsync('render: menu warm mirror', () => this.warmSkyMirror());
      } finally {
        this.endBuild();
      }
    });
    this.townQueue = run.catch(() => {});
    return run;
  }

  async hideTown(glide = 0): Promise<void> {
    // Let a pending overview build finish first, so it can't switch back to the overview later.
    this.townTicket++;
    await this.townQueue;
    if (!this.townMode) return;
    const rig = this.overview?.rig;
    if (glide > 0 && rig && this.active) {
      const p = this.parked;
      this.flyingIn = true;
      this.applyPacing();
      await new Promise<void>((resolve) => {
        const timer = setTimeout(resolve, glide * 1000 + 600);
        rig.glideTo({ alpha: p.alpha, beta: p.beta, radius: p.radius, x: p.target.x, z: p.target.z }, glide, () => {
          clearTimeout(timer);
          resolve();
        });
      });
      this.flyingIn = false;
    }
    if (this.townMode) this.leaveTown();
    this.applyPacing();
  }

  setTownShot(shot: TownShot, cut = false): void {
    this.townShot = shot;
    this.overview?.rig.setShot(shot, cut || !this.townMode);
    if (shot.minute !== undefined) {
      this.townMinuteGoal = shot.minute;
      if (cut || !this.townMode) this.townMinute = shot.minute;
    }
  }

  nudgeTown(dAlpha: number, zoom: number): void {
    if (this.townMode) this.overview?.rig.nudge(dAlpha, zoom);
  }

  setLotHighlight(highlight: LotHighlight): void {
    this.townHighlight = highlight;
    this.overview?.setHighlight(highlight);
  }

  onTownFrame(fn: (() => void) | null): void {
    this.townListener = fn;
  }

  /** The camera the game world uses: the real one, or the parked pose while the overview drives it. */
  private get gameCamera(): ArcRotateCamera {
    return this.townMode ? this.parked : this.camera;
  }

  private enterTown(): void {
    const cam = this.camera;
    this.townMode = true;
    this.applyPacing();
    this.applyDig();
    copyPose(cam, this.parked);
    cam.detachControl();
    cam.inertialAlphaOffset = cam.inertialBetaOffset = cam.inertialRadiusOffset = 0;
    cam.inertialPanningX = cam.inertialPanningY = 0;
    cam.lowerRadiusLimit = 2;
    cam.upperRadiusLimit = 1200;
    cam.lowerBetaLimit = 0.05;
    cam.upperBetaLimit = 1.45;
    cam.layerMask = LAYER_TOWN;
    this.sunShadows.setOverview(true);
    const rig = this.overview!.rig;
    rig.syncFromCamera();
    if (this.townShot) rig.setShot(this.townShot, true);
    this.cursor.position.y = HIDDEN_Y;
    this.lastLightMinute = -1;
    this.updateShadowList();
    this.pauseSnapshot();
  }

  private leaveTown(): void {
    const cam = this.camera;
    perfPhase('render: live');
    this.townMode = false;
    this.applyDig();
    this.warmFrames = 0;
    cam.layerMask = LAYER_WORLD;
    this.sunShadows.setOverview(false);
    copyPose(this.parked, cam);
    cam.targetScreenOffset.set(0, 0);
    [cam.lowerRadiusLimit, cam.upperRadiusLimit] = RADIUS_LIMITS;
    cam.lowerBetaLimit = 0.3;
    cam.upperBetaLimit = 1.35;
    cam.inertialAlphaOffset = cam.inertialBetaOffset = cam.inertialRadiusOffset = 0;
    cam.attachControl(true);
    this.updateShadowList();
    this.scene.fogDensity = LOOK.fog;
    this.applyLighting(this.lastMinute);
    this.lastLightMinute = this.lastMinute;
    this.cutLook = -1;
    this.updateCutaway();
    // The game draws live (see `scheduleSnapshot`).
    this.snapshotGeneration++;
    this.snapshotDue = null;
    if (this.webgpu) this.webgpu.snapshotRendering = false;
  }

  /** Overview per frame: camera rig, highlights, lighting at the menu's hour. Allocation-free. */
  private townFrame(dt: number): void {
    const overview = this.overview!;
    const w = this.cssSize.width || 1;
    const h = this.cssSize.height || 1;
    overview.rig.setViewport(w, h);
    overview.frame(Math.min(0.1, dt / 1000), w / h);
    // The light eases between the screens' hours (about two seconds).
    const goal = this.townMinuteGoal;
    if (this.townMinute !== goal) {
      const step = Math.max(0.5, Math.abs(goal - this.townMinute) * Math.min(1, dt / 600));
      this.townMinute = Math.abs(goal - this.townMinute) <= step ? goal : this.townMinute + Math.sign(goal - this.townMinute) * step;
    }
    if (this.lastLightMinute !== this.townMinute) {
      this.applyLighting(this.townMinute);
      this.lastLightMinute = this.townMinute;
      this.resetSnapshot();
    }
    // Haze is tuned for the game's close camera; from high above it would wash the town out
    // (and the mountains round a mountain town, which are meant to be seen).
    const clear = this.natureIn.town?.landscape.mountainous ? 0.35 : 1;
    const fog = LOOK.fog * Math.min(1, Math.max(0.3, 55 / this.camera.radius)) * clear;
    if (Math.abs(fog - this.scene.fogDensity) > LOOK.fog * 0.02) this.scene.fogDensity = fog;
    this.followCamera();
    if (!this.reducedMotion) this.skyDome.drift(dt);
    this.townListener?.();
  }

  /**
   * Draws the game world once more in this frame, from its own (parked) view, before the overview
   * is drawn over it: compiles its shaders and pipelines while it is still hidden.
   */
  private warmWorld(): void {
    const span = perfStart('render: warm world');
    this.warmFrames--;
    const cam = this.camera;
    const p = this.parked;
    const w = this.warmPose;
    [w.alpha, w.beta, w.radius, w.x, w.y, w.z, w.ox, w.oy] = [cam.alpha, cam.beta, cam.radius, cam.target.x, cam.target.y, cam.target.z, cam.targetScreenOffset.x, cam.targetScreenOffset.y];
    const webgpu = this.webgpu;
    const snapshot = webgpu?.snapshotRendering ?? false;
    if (webgpu && snapshot) webgpu.snapshotRendering = false;
    cam.layerMask = LAYER_WORLD;
    copyPose(p, cam);
    cam.targetScreenOffset.set(0, 0);
    const map = this.shadows.getShadowMap()!;
    map.renderList = [...this.baseCasters, ...this.wallCasters, ...this.landscapeCasters()];
    this.scene.render();
    cam.layerMask = LAYER_TOWN;
    cam.alpha = w.alpha;
    cam.beta = w.beta;
    cam.radius = w.radius;
    cam.target.copyFromFloats(w.x, w.y, w.z);
    cam.targetScreenOffset.copyFromFloats(w.ox, w.oy);
    this.updateShadowList();
    if (webgpu && snapshot) webgpu.snapshotRendering = true;
    perfEnd(span);
  }

  /**
   * Runs one game frame while the overview is up (the game is built behind the menus): the game's
   * own lighting, cutaway and poses are applied and its world is drawn once, so the shaders and
   * pipelines it needs are ready before it is first shown. The overview's look is put back after.
   */
  async warmGame(frame: FrameState): Promise<void> {
    if (!this.townMode || this.gameWarmed) return;
    this.gameWarmed = true;
    const cam = this.camera;
    const pose = [cam.alpha, cam.beta, cam.radius, cam.target.x, cam.target.y, cam.target.z, cam.targetScreenOffset.x, cam.targetScreenOffset.y] as const;
    const lightMinute = this.lastLightMinute;
    const minute = this.lastMinute;
    const town = () => {
      this.townMode = true;
      this.sunShadows.setOverview(true);
      [cam.alpha, cam.beta, cam.radius, cam.target.x, cam.target.y, cam.target.z, cam.targetScreenOffset.x, cam.targetScreenOffset.y] = pose;
      cam.layerMask = LAYER_TOWN;
      this.lastMinute = minute;
      this.applyLighting(this.townMinute);
      this.lastLightMinute = lightMinute;
      this.applyDig();
      this.updateCutaway();
      this.updateShadowList();
      this.pauseSnapshot();
    };
    // First the game's own state (lighting, cutaway, the shadow map's culling): the update is the
    // costly part, so it runs in its own slice.
    this.townMode = false;
    // The game's cutaway is worked out from the game's own camera: put it where it will be when the
    // game is shown (the parked pose), not where the overview's camera happens to be.
    copyPose(this.parked, cam);
    perfTime('render: warm game state', () => {
      this.update(frame);
      this.applyDig();
      this.sunShadows.setOverview(false);
      this.updateShadowList();
    });
    const gameMinute = this.lastMinute;
    town();
    // Then draw the world once with that look, in the next frame (a whole render is a slice of its
    // own). Inside the render loop: a WebGPU draw outside a frame targets the canvas texture of the
    // frame before, which is gone by the next submit, and that whole submit is then refused.
    this.beforeDraw = () => {
      if (!this.townMode) return;
      this.townMode = false;
      try {
        copyPose(this.parked, cam);
        // (The game's pose is the one the reflections are culled against, so they are ranked again here.)
        this.mirrors.update(this.camera, null);
        if (!this.mirrorWarmed) {
          this.mirrorWarmed = true;
          cam.layerMask = LAYER_WORLD;
          this.scene.updateTransformMatrix();
          // At its real size: the first live reflection at 512 would otherwise pay the same compile again.
          this.mirrors.warm(0)();
        }
        perfTime('render: warm game draw', () => {
          this.lastMinute = gameMinute;
          this.applyLighting(gameMinute);
          this.applyDig();
          this.updateCutaway();
          this.sunShadows.setOverview(false);
          this.updateShadowList();
          this.warmFrames = Math.max(1, this.warmFrames);
          this.warmWorld();
        });
      } finally {
        town();
      }
    };
    await this.framesRendered(1);
    this.beforeDraw = null;
  }

  /** Shadow casters of what is on screen: the overview, or the game world (the house in every wall mode). */
  private updateShadowList(): void {
    const map = this.shadows.getShadowMap()!;
    if (this.townMode) map.renderList = [...(this.overview?.casters ?? []), ...((this.quality.landscapeShadows && this.natureIn.town?.casters) || [])];
    else map.renderList = [...this.baseCasters, ...this.wallCasters, ...this.landscapeCasters()];
    this.sunShadows.moved();
  }

  /**
   * The landscape of `world` for a layer: shared with the other layer when it is the same town
   * (seed and size), otherwise built; one no layer shows any more is disposed.
   */
  private async useNature(layer: 'world' | 'town', world: WorldStructure): Promise<NatureSet> {
    // The town's own size (the game's lot also holds the storeys above it).
    const key = Landscape.key(world);
    const ticket = ++this.natureTicket[layer];
    let set = this.natures.get(key);
    if (!set) {
      const landscape = new Landscape(world, this.quality.lawnDensity, this.quality.streetLod);
      const created: NatureSet = { key, landscape, meshes: [], casters: [], only: new Map(), users: new Set(), pending: 0, ready: Promise.resolve(), refreshing: Promise.resolve(), coming: [] };
      created.ready = landscape.build(this.scene, this.deps.assets, world, () => this.pace()).then((nature) => {
        perfTime('nature: meshes', () => {
          created.meshes = nature.meshes;
          created.casters = nature.casters;
          created.only = nature.only;
          for (const mesh of nature.meshes) this.prepareNature(mesh);
        });
      });
      this.natures.set(key, (set = created));
    }
    set.pending++;
    try {
      await set.ready;
    } finally {
      set.pending--;
    }
    if (ticket === this.natureTicket[layer]) {
      const old = this.natureIn[layer];
      if (old !== set) old?.users.delete(layer);
      set.users.add(layer);
      this.natureIn[layer] = set;
    }
    // Drop landscapes nobody shows; draw each on the layers that use it.
    for (const [k, n] of this.natures) {
      if (!n.users.size && !n.pending) {
        for (const mesh of n.meshes) mesh.dispose(false, false);
        this.natures.delete(k);
        continue;
      }
      this.showNature(n);
    }
    this.excludeLandscape();
    return set;
  }

  /** A landscape mesh just built: still, on no layer yet (`showNature`), dug away over a basement in view. */
  private prepareNature(mesh: Mesh): void {
    mesh.freezeWorldMatrix();
    mesh.layerMask = 0;
    // Dug away over a basement in view.
    this.lib.digInto(mesh);
    // `?diag=nonature`: without the woods, rocks and lawn (the terrain, water and roads stay).
    if (this.diag.has('nonature') && mesh.hasThinInstances) mesh.setEnabled(false);
  }

  /** Draws a landscape on the layers that use it. */
  private showNature(n: NatureSet): void {
    const mask = (n.users.has('world') ? LAYER_WORLD : 0) | (n.users.has('town') ? LAYER_TOWN : 0);
    // Plants in groups only in the game, whole only in the overview (and in the game's shadow
    // map: the casters); those neither draws are switched off (each mesh costs a little every
    // frame, drawn or not).
    const casters = new Set(this.quality.landscapeShadows ? n.casters : []);
    for (const mesh of n.meshes) {
      const only = n.only.get(mesh);
      if (!only) {
        mesh.layerMask = mask;
        continue;
      }
      mesh.layerMask = mask & (only === 'game' ? LAYER_WORLD : LAYER_TOWN);
      mesh.setEnabled((mesh.layerMask !== 0 || (casters.has(mesh) && n.users.has('world'))) && !this.diag.has('nonature'));
    }
  }

  /**
   * Interior lamps never light the landscapes kept (keeps their shaders simple: lit by the lamp
   * cluster, the ground alone cost 1.1–1.5 ms of GPU a frame at night on Medium), nor the plants
   * being built again for them (`NatureSet.coming`); the garden's lamps light the lawn with pools
   * instead (`placeGardenPools`). Only when that changes: setting the lists marks every material dirty.
   */
  private excludeLandscape(): void {
    const landscape = new Set([...[...this.natures.values()].flatMap((n) => [...n.meshes, ...n.coming]), this.skyDome.mesh]);
    if (landscape.size !== this.lightExcluded.size || [...landscape].some((m) => !this.lightExcluded.has(m))) {
      this.lightExcluded = landscape;
      this.lib.excludeFromLights([...this.roomLights, ...(this.lampCluster ? [this.lampCluster] : [])], landscape);
    }
  }

  /** The landscape's shadow casters in the game (the woods near the town), as the world's landscape has them now. */
  private landscapeCasters(): Mesh[] {
    return (this.quality.landscapeShadows && this.natureIn.world?.casters) || [];
  }

  /** A model file that failed has come: each landscape's plants built without it are built again (`rebuildNature`). */
  private refreshNature(url: string): void {
    for (const set of this.natures.values()) {
      set.refreshing = set.refreshing.then(() => this.rebuildNature(set, url)).catch((err) => console.error(`[render] building the landscape again with ${url} failed`, err));
    }
  }

  /**
   * Builds again the plants of `set` built without the file at `url` (nothing, where it failed;
   * the full model, where a far version did), in slices between frames and unseen, and swaps them
   * in once their shaders are ready for every pass that draws them: the rest of the landscape
   * stays as it is, and what they replace is drawn until then and disposed after. No world is
   * built again for it.
   */
  private async rebuildNature(set: NatureSet, url: string): Promise<void> {
    try {
      await set.ready;
    } catch {
      // (Its first build failed: there's nothing to build again into.)
      return;
    }
    const gone = () => this.disposed || this.natures.get(set.key) !== set;
    if (gone()) return;
    this.refreshes++;
    let built;
    try {
      built = await set.landscape.rebuild(this.scene, this.deps.assets, url, () => this.pace());
    } finally {
      this.refreshes--;
    }
    if (!built) return;
    if (!gone()) {
      for (const mesh of built.meshes) this.prepareNature(mesh);
      set.coming = built.meshes;
      this.excludeLandscape();
      const casters = new Set(built.casters);
      await this.meshesReady(built.meshes, (mesh) => this.quality.landscapeShadows && casters.has(mesh));
      // (A world build takes the landscape as it was: swapped between builds.)
      while (this.worldBuilds) await this.worldBuilds.catch(() => {});
    }
    set.coming = [];
    if (gone()) {
      for (const mesh of built.meshes) mesh.dispose(false, false);
      return;
    }
    built.commit();
    const replaced = new Set(built.replaced);
    set.meshes = [...set.meshes.filter((m) => !replaced.has(m)), ...built.meshes];
    set.casters = [...set.casters.filter((m) => !replaced.has(m)), ...built.casters];
    for (const mesh of replaced) set.only.delete(mesh);
    for (const [mesh, only] of built.only) set.only.set(mesh, only);
    this.showNature(set);
    this.updateShadowList();
    for (const mesh of replaced) {
      mesh.dispose(false, false);
      // (Disposing takes it off the lamps' lists: the new ones are on them already.)
      this.lightExcluded.delete(mesh);
    }
    this.resetSnapshot();
  }

  /**
   * Waits (a frame at a time, a second or two at most) until `meshes`' shaders are ready for every
   * pass that will draw them, so they're drawn whole from the first frame they're shown: the view,
   * the sun's shadow map for those that will cast (`casts`), and the reflections live now.
   */
  private async meshesReady(meshes: readonly Mesh[], casts: (mesh: Mesh) => boolean): Promise<void> {
    for (let k = 0; k < MESH_READY_FRAMES && !this.disposed && meshes.some((m) => !m.isDisposed() && !this.drawReady(m, casts(m))); k++) {
      // (As `pace` waits: a hidden tab draws nothing, so it waits with it.)
      await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
    }
  }

  /**
   * Whether `mesh`'s shaders are ready for the passes that will draw it: the view, the sun's
   * shadow map (`caster`: its depth shader) and each reflection live now (its clip-plane
   * variant). Asking compiles what's missing, before the mesh is shown.
   */
  private drawReady(mesh: Mesh, caster: boolean): boolean {
    if (!mesh.isReady(true)) return false;
    const engine = this.engine;
    const pass = engine.currentRenderPassId;
    const clip = this.scene.clipPlane;
    try {
      if (caster) {
        for (const id of this.shadows.getShadowMap()!.renderPassIds) {
          engine.currentRenderPassId = id;
          for (const sub of mesh.subMeshes) if (!this.shadows.isReady(sub, mesh.hasThinInstances, sub.getMaterial()?.needAlphaBlendingForMesh(mesh) ?? false)) return false;
        }
      }
      for (const texture of this.mirrors.reflecting) {
        this.scene.clipPlane = texture.mirrorPlane;
        for (const id of texture.renderPassIds) {
          engine.currentRenderPassId = id;
          if (!mesh.isReady(true)) return false;
        }
      }
    } finally {
      engine.currentRenderPassId = pass;
      this.scene.clipPlane = clip;
    }
    return true;
  }

  private readonly renderFrame = () => {
    // Behind a full-screen menu: its last frames, then only those something waits for.
    if (this.covered) {
      if (this.coveredFrames > 0) this.coveredFrames--;
      else if (!this.frameWaiters.length) return;
    }
    const start = performance.now();
    const waited = this.frameWaiters.length > 0;
    const frameSpan = waited ? perfStart('render: frame (waited)') : null;
    const probe = this.probe;
    // Babylon asks for the next frame only after this one returns: a frame that throws must not
    // stop drawing for good. The first error is logged; many in a row count as broken.
    let updated = 0;
    try {
      if (probe) {
        const c = this.counts;
        c.shadowDrawCalls = c.shadowTriangles = c.mirrorDrawCalls = c.mirrorTriangles = 0;
        probe.before(start);
      }
      const dt = (this.frameDt = Math.min(this.engine.getDeltaTime(), MAX_FRAME_MS));
      this.paceKeys(dt);
      // The time of the refresh this frame is for: even steps, as the frames will show.
      const tOn = performance.now();
      this.onFrame?.(this.pacer.frameTime || performance.now());
      const onMs = performance.now() - tOn;
      if (onMs > 20) perfRecord('render: onFrame', tOn, onMs, this.townMode ? 'town' : 'world');
      if (probe) updated = performance.now();
      if (this.townMode && this.overview) {
        this.townFrame(dt);
        if (this.warmFrames > 0 && !this.worldBuilding) this.warmWorld();
      }
      const before = this.beforeDraw;
      this.beforeDraw = null;
      before?.();
      this.applySnapshotDue();
      const tScene = performance.now();
      this.scene.render();
      const sceneMs = performance.now() - tScene;
      if (sceneMs > 20) perfRecord('render: scene.render', tScene, sceneMs, this.townMode ? 'town' : 'world');
      this.failedFrames = 0;
    } catch (err) {
      if (this.failedFrames === 0) console.error('[render] a frame failed', err);
      if (++this.failedFrames === FAILED_FRAMES_LIMIT) this.onFailure?.('frames', err);
      return;
    }
    const end = performance.now();
    if (end - start > 40) perfRecord('render: slow frame', start, end - start, this.townMode ? 'town' : 'world');
    this.cpuMs += (end - start - this.cpuMs) * 0.1;
    if (this.gpuWanted.size && this.webgpu) this.settleSpans();
    if (probe) {
      this.counts.drawCalls = this.instrumentation.drawCallsCounter.current;
      this.counts.triangles = this.scene.getActiveIndices() / 3;
      if (!this.webgpu) {
        // SAFETY: Babylon's WebGL2 timer keeps its running counter in the private `_gpuFrameTime`.
        const gpu = this.glTimer ? (this.engine as unknown as { _gpuFrameTime?: { count: number; current: number } })._gpuFrameTime : undefined;
        this.counts.gpuFrames = gpu?.count ?? 0;
        this.counts.gpuMs = gpu ? gpu.current / 1e6 : 0;
      }
      this.counts.residentsMs = this.characters?.updateMs ?? 0;
      probe.after(start, updated, end, this.counts);
    }
    if (frameSpan) perfEnd(frameSpan);
    for (let i = this.frameWaiters.length - 1; i >= 0; i--) {
      const w = this.frameWaiters[i];
      if (--w.left <= 0) {
        this.frameWaiters.splice(i, 1);
        w.resolve();
      }
    }
  };

  /**
   * Babylon's keyboard camera input (arrows turn, Ctrl+arrows pan, +/- zoom) nudges the camera by
   * a fixed amount per drawn frame: scaled by the frame's time, it moves as fast at any frame rate
   * as at 60 fps.
   */
  private paceKeys(dtMs: number): void {
    const keys = this.camera.inputs.attached.keyboard as ArcRotateCameraKeyboardMoveInput | undefined;
    if (!keys) return;
    const k = Math.max(0.05, (dtMs * 60) / 1000);
    const at60 = this.keyNudge;
    keys.angularSpeed = at60.angularSpeed * k;
    keys.panningSensibility = at60.panningSensibility / k;
    keys.zoomingSensibility = at60.zoomingSensibility / k;
  }

  // --- setup -------------------------------------------------------------------------

  private setupCamera(): void {
    const cam = (this.camera = new ArcRotateCamera('camera', -Math.PI * 0.62, 0.86, 24, Vector3.Zero(), this.scene));
    cam.lowerRadiusLimit = 6;
    cam.upperRadiusLimit = 95;
    cam.lowerBetaLimit = 0.3;
    cam.upperBetaLimit = 1.35;
    cam.wheelDeltaPercentage = 0.012;
    cam.panningSensibility = 90;
    cam.mapPanning = true;
    cam.inertia = 0.82;
    cam.panningInertia = 0.82;
    cam.minZ = 0.3;
    cam.maxZ = 1600;
    // A slightly longer lens than the default flattens perspective a little: the dollhouse look.
    cam.fov = 0.7;
    cam.attachControl(true);
    const keys = cam.inputs.attached.keyboard as ArcRotateCameraKeyboardMoveInput | undefined;
    if (keys) this.keyNudge = { angularSpeed: keys.angularSpeed, panningSensibility: keys.panningSensibility, zoomingSensibility: keys.zoomingSensibility };
    this.gameRig = new GameCameraRig(cam, { radius: RADIUS_LIMITS, beta: [0.3, 1.35] }, (i, out) => this.simHead(i, out));
    // Holds the game view while the town overview drives the camera (never drawn).
    this.parked = new ArcRotateCamera('parkedCamera', cam.alpha, cam.beta, cam.radius, Vector3.Zero(), this.scene);
    this.parked.fov = cam.fov;
  }

  private setupLights(): void {
    applyShaderFixes();
    patchRoomLights();
    // Sky ambient and reflections come from the environment map (image-based lighting); the
    // hemispheric light only adds a time-of-day tint (golden hour, blue hour, moonlight).
    this.sky = new HemisphericLight('sky', new Vector3(0.2, 1, -0.3), this.scene);
    this.sky.specular = Color3.Black();
    this.sun = new DirectionalLight('sun', new Vector3(-0.5, -1, 0.4), this.scene);
    this.sun.autoCalcShadowZBounds = true;

    // A red-only colour target (R16F): filtered shadows read only the depth texture beside it, so the
    // RGBA16F one Babylon makes by default was written every frame and never read (P1-05).
    const shadows = (this.shadows = new ShadowGenerator(this.quality.shadowMapSize, this.sun, false, undefined, true));
    shadows.usePercentageCloserFiltering = true;
    shadows.filteringQuality = ShadowGenerator.QUALITY_MEDIUM;
    shadows.bias = 0.0015;
    shadows.normalBias = 0.02;
    // Physically, shadows block all direct sunlight; the environment fills them in.
    shadows.setDarkness(0);
    this.sunShadows = new SunShadows(shadows, this.scene);
    if (new URLSearchParams(location.search).has('profile')) {
      const map = shadows.getShadowMap()!;
      const render = map.render.bind(map);
      map.render = (useCameraPostProcess?: boolean, dumpForDebug?: boolean) => {
        const start = performance.now();
        const result = render(useCameraPostProcess, dumpForDebug);
        const ms = performance.now() - start;
        if (ms > 8) console.log(`[shadow] ${ms.toFixed(0)} ms casters ${map.renderList?.length ?? 0}`);
        return result;
      };
    }
    if (this.diag.has('noshadows')) this.sun.shadowEnabled = false;
    // The game's map is drawn again only when something in it moved (see sunShadows.ts); with WebGPU
    // snapshot rendering (the menus' town) Babylon draws every render target each frame anyway.

    // Warm interior lamps; always enabled (so shaders never recompile) and dimmed to zero by
    // day. Clustered where the engine supports it: then dozens light at once. Spot lights, so a
    // shade can send the light one way (`RoomLight.dir`); those lighting all round have a cone of
    // a whole turn, which the cluster takes as a point light (and the glTF falloff as no cone).
    const wanted = new URLSearchParams(location.search).get('lights');
    const make = (i: number) => {
      const light = new SpotLight(`room${i}`, new Vector3(0, HIDDEN_Y, 0), new Vector3(0, -1, 0), ALL_ROUND, 1, this.scene);
      light.innerAngle = ALL_ROUND;
      light.falloffType = Light.FALLOFF_GLTF;
      light.range = DARK_LAMP_RANGE;
      light.intensity = 0;
      light.specular = new Color3(0.4, 0.35, 0.3);
      return light;
    };
    const first = make(0);
    // Clusters take only the default (inverse-square) falloff.
    first.falloffType = Light.FALLOFF_DEFAULT;
    const clustered = (wanted === 'clustered' || (wanted !== 'forward' && this.quality.clusteredLamps)) && ClusteredLightContainer.IsLightSupported(first);
    if (!clustered) first.falloffType = Light.FALLOFF_GLTF;
    this.roomLights.push(first);
    for (let i = 1; i < (clustered ? CLUSTERED_LIGHTS : ROOM_LIGHTS); i++) {
      const light = make(i);
      if (clustered) light.falloffType = Light.FALLOFF_DEFAULT;
      this.roomLights.push(light);
    }
    if (clustered) {
      this.lampCluster = new ClusteredLightContainer('lamps', this.roomLights, this.scene);
      this.lampCluster.maxRange = LAMP_MAX_RANGE;
    }
    // Each clustered lamp lights only its own room (`roomLights.ts`; `?roomlights=0` turns it off to compare).
    this.lib.rooms.enabled = clustered && new URLSearchParams(location.search).get('roomlights') !== '0';
    if (this.diag.has('nolamps')) for (const light of this.lampCluster ? [this.lampCluster] : this.roomLights) light.setEnabled(false);
    else this.gardenPools = new GardenPools(this.scene);
  }

  private setupPostProcessing(): void {
    const q = this.quality;
    const post = (this.post = new DefaultRenderingPipeline('post', true, this.scene, [this.camera]));
    const aa = !this.diag.has('noaa');
    post.samples = aa ? q.msaaSamples : 1;
    post.fxaaEnabled = aa && q.msaaSamples <= 1;
    post.imageProcessingEnabled = true;
    const ip = post.imageProcessing;
    ip.toneMappingEnabled = true;
    ip.vignetteEnabled = true;
    ip.vignetteStretch = 0.5;
    ip.vignetteColor = new Color4(0.02, 0.02, 0.04, 0);
    ip.colorCurvesEnabled = true;
    ip.colorCurves = new ColorCurves();
    post.bloomEnabled = false;
    post.depthOfFieldEnabled = false;
    if (!this.diag.has('nonight')) this.nightGrade = new NightGrade(this.camera, this.engine, this.webgpu !== null);
  }

  private setupHelpers(): void {
    const cursorMat = new StandardMaterial('cursor', this.scene);
    cursorMat.disableLighting = true;
    cursorMat.emissiveColor = ACCENT;
    cursorMat.alpha = 0.35;
    this.cursor = MeshBuilder.CreateGround('cursor', { width: 0.92, height: 0.92 }, this.scene);
    this.cursor.material = cursorMat;
    this.cursor.position.y = HIDDEN_Y;

    // Soft round contact shadows (Sims: per frame; objects: per world) from one radial texture.
    const blob = new DynamicTexture('blob', 64, this.scene, true);
    const ctx = blob.getContext();
    const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
    g.addColorStop(0, 'rgba(0,0,0,0.55)');
    g.addColorStop(0.55, 'rgba(0,0,0,0.3)');
    g.addColorStop(1, 'rgba(0,0,0,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 64, 64);
    blob.update();
    blob.hasAlpha = true;
    const blobMat = new StandardMaterial('blobShadow', this.scene);
    blobMat.disableLighting = true;
    blobMat.diffuseColor = Color3.Black();
    blobMat.emissiveColor = Color3.Black();
    blobMat.specularColor = Color3.Black();
    blobMat.opacityTexture = blob;
    blobMat.disableDepthWrite = true;
    blobMat.zOffset = -3;
    const makeBlob = (name: string) => {
      const mesh = MeshBuilder.CreateGround(name, { width: 1, height: 1 }, this.scene);
      mesh.material = blobMat;
      mesh.isPickable = false;
      return mesh;
    };
    this.simShadow = makeBlob('simShadows');
    this.simShadow.thinInstanceSetBuffer('matrix', this.simShadowMatrices, 16, false);
    this.simShadow.alwaysSelectAsActiveMesh = true;
    this.objectShadow = makeBlob('objectShadowTemplate');
    this.objectShadow.thinInstanceSetBuffer('matrix', new Float32Array(16), 16, true);
    this.objectShadow.thinInstanceCount = 0;
    this.objectShadow.setEnabled(false);

    // Build mode: placement ghost, wall-tool previews and the tile grid.
    this.ghostRoot = new TransformNode('ghost', this.scene);
    this.ghostRoot.position.y = HIDDEN_Y;
    const ghostMat = (this.ghostMaterial = new StandardMaterial('ghost', this.scene));
    ghostMat.alpha = 0.5;
    ghostMat.diffuseColor = new Color3(1.0, 0.45, 0.42);
    ghostMat.specularColor = Color3.Black();
    ghostMat.emissiveColor = new Color3(0.25, 0.25, 0.25);
    ghostMat.backFaceCulling = false;
    const arrowMat = (this.ghostArrowMaterial = new StandardMaterial('ghostArrow', this.scene));
    arrowMat.disableLighting = true;
    arrowMat.backFaceCulling = false;
    arrowMat.emissiveColor = new Color3(0.55, 1.0, 0.6);
    // A stem and a head lying flat, pointing +z (out of the object's front), within one tile.
    const arrow = (this.ghostArrow = new Mesh('ghostArrow', this.scene));
    const arrowData = new VertexData();
    arrowData.positions = [-0.09, 0, -0.36, 0.09, 0, -0.36, 0.09, 0, 0.04, -0.09, 0, 0.04, -0.26, 0, 0.04, 0.26, 0, 0.04, 0, 0, 0.38];
    arrowData.normals = Array.from({ length: 7 }, () => [0, 1, 0]).flat();
    arrowData.indices = [0, 2, 1, 0, 3, 2, 4, 6, 5];
    arrowData.applyToMesh(arrow, false);
    arrow.material = arrowMat;
    arrow.parent = this.ghostRoot;
    arrow.isPickable = false;
    arrow.renderingGroupId = 1;
    arrow.alwaysSelectAsActiveMesh = true;
    arrow.setEnabled(false);
    this.fx = new BuildEffects(this.scene);

    const preview = (kind: EdgePreview['kind'], color: string, alpha: number) => {
      const mat = new StandardMaterial(`preview-${kind}`, this.scene);
      mat.disableLighting = true;
      mat.emissiveColor = Color3.FromHexString(color);
      mat.alpha = alpha;
      mat.backFaceCulling = false;
      const mesh = MeshBuilder.CreateBox(`preview-${kind}`, { size: 1 }, this.scene);
      mesh.material = mat;
      mesh.isPickable = false;
      mesh.alwaysSelectAsActiveMesh = true;
      const buffer = new Float32Array(PREVIEW_CAPACITY * 16);
      mesh.thinInstanceSetBuffer('matrix', buffer, 16, false);
      mesh.thinInstanceCount = 0;
      mesh.setEnabled(false);
      return [mesh, buffer] as const;
    };
    const [wall, wallBuf] = preview('wall', '#9FC4FF', 0.45);
    const [door, doorBuf] = preview('door', '#B07A4A', 0.6);
    const [pane, paneBuf] = preview('window', '#7FE0F2', 0.55);
    const [open, openBuf] = preview('open', '#E5484D', 0.5);
    // Paint tool: one film per face, tinted with the covering.
    const paint = MeshBuilder.CreateBox('preview-paint', { size: 1 }, this.scene);
    const paintMat = new StandardMaterial('preview-paint', this.scene);
    paintMat.disableLighting = true;
    paintMat.alpha = 0.6;
    paint.material = paintMat;
    paint.isPickable = false;
    paint.alwaysSelectAsActiveMesh = true;
    paint.thinInstanceSetBuffer('matrix', this.paintBuffer, 16, false);
    paint.thinInstanceCount = 0;
    paint.setEnabled(false);
    this.paintPreview = paint;
    // Floor tool: one film per tile, tinted with the covering.
    const floor = MeshBuilder.CreateGround('preview-floor', { width: 1, height: 1 }, this.scene);
    const floorMat = new StandardMaterial('preview-floor', this.scene);
    floorMat.disableLighting = true;
    floorMat.alpha = 0.6;
    floor.material = floorMat;
    floor.isPickable = false;
    floor.alwaysSelectAsActiveMesh = true;
    floor.thinInstanceSetBuffer('matrix', this.floorBuffer, 16, false);
    floor.thinInstanceCount = 0;
    floor.setEnabled(false);
    this.floorPreview = floor;
    this.previewMeshes = { wall, door, window: pane, open };
    this.previewBuffers = { wall: wallBuf, door: doorBuf, window: paneBuf, open: openBuf };

    // Tile grid: white lines on a transparent texture, repeated once per tile, with fainter
    // half- and quarter-tile lines inside for lining up small things (they fade out with distance).
    const gridTex = new DynamicTexture('grid', 128, this.scene, true);
    const gctx = gridTex.getContext();
    gctx.clearRect(0, 0, 128, 128);
    gctx.fillStyle = 'rgba(255,255,255,0.3)';
    for (const p of [32, 96]) {
      gctx.fillRect(0, p, 128, 2);
      gctx.fillRect(p, 0, 2, 128);
    }
    gctx.fillStyle = 'rgba(255,255,255,0.55)';
    gctx.fillRect(0, 64, 128, 2);
    gctx.fillRect(64, 0, 2, 128);
    gctx.fillStyle = 'rgba(255,255,255,1)';
    gctx.fillRect(0, 0, 128, 3);
    gctx.fillRect(0, 0, 3, 128);
    gridTex.update();
    gridTex.hasAlpha = true;
    gridTex.wrapU = gridTex.wrapV = DynamicTexture.WRAP_ADDRESSMODE;
    const gridMat = new StandardMaterial('grid', this.scene);
    gridMat.disableLighting = true;
    gridMat.diffuseTexture = gridTex;
    gridMat.useAlphaFromDiffuseTexture = true;
    gridMat.emissiveColor = new Color3(0.9, 0.95, 1);
    gridMat.alpha = 0.45;
    gridMat.disableDepthWrite = true;
    gridMat.zOffset = -4;
    this.grid = MeshBuilder.CreateGround('buildGrid', { width: 1, height: 1 }, this.scene);
    this.grid.material = gridMat;
    this.grid.isPickable = false;
    this.grid.setEnabled(false);

    // Build helpers work in lot rows; this node lifts them to the storey in view.
    this.storeyLayer = new TransformNode('storeyLayer', this.scene);
    for (const node of [this.cursor, this.ghostRoot, this.grid, this.paintPreview, this.floorPreview, ...Object.values(this.previewMeshes)]) node.parent = this.storeyLayer;
  }

  /**
   * The selection marker over the selected Sim (`model.marker`): a small soft four-point
   * sparkle star, unlit, with a thin deeper-toned outline (so it reads on light floors and
   * bright sky) and a faint additive glow disc behind it. All face the camera.
   */
  private async setupMarker(): Promise<void> {
    const mat = new PBRMaterial('marker', this.scene);
    mat.unlit = true;
    mat.albedoColor = Color3.White();
    // Outline: the star's back faces pushed out along their normals (inverted hull).
    const outlineMat = new PBRMaterial('markerOutline', this.scene);
    outlineMat.unlit = true;
    outlineMat.albedoColor = Color3.White();
    outlineMat.cullBackFaces = false;
    this.markerMats = [mat, outlineMat];
    // Without its file, no star (the glow stays) until the file comes. Listened for before it's
    // built: a scene nobody listens to doesn't try a failed file again.
    this.unlistenMarker = onModelFileArrived(this.scene, (url) => {
      this.markerBuilds = this.markerBuilds.then(() => (this.markerFailed.includes(url) ? this.buildMarker() : undefined)).catch((err) => console.error(`[render] building the marker again with ${url} failed`, err));
    });
    await this.buildMarker();

    // Glow: a radial gradient disc, added on top of the scene (reads at night, subtle by day).
    const glow = new DynamicTexture('markerGlow', 64, this.scene, true);
    const ctx = glow.getContext();
    const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
    g.addColorStop(0, 'rgba(255,255,255,0.9)');
    g.addColorStop(0.25, 'rgba(255,255,255,0.45)');
    g.addColorStop(0.6, 'rgba(255,255,255,0.12)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 64, 64);
    glow.update();
    glow.hasAlpha = true;
    const haloMat = new StandardMaterial('markerHalo', this.scene);
    haloMat.disableLighting = true;
    haloMat.diffuseColor = Color3.Black();
    haloMat.specularColor = Color3.Black();
    haloMat.emissiveColor = Color3.White();
    haloMat.opacityTexture = glow;
    haloMat.alphaMode = Constants.ALPHA_ADD;
    haloMat.disableDepthWrite = true;
    haloMat.backFaceCulling = false;
    const halo = MeshBuilder.CreatePlane('markerHalo', { size: 1 }, this.scene);
    halo.material = haloMat;
    halo.isPickable = false;
    halo.applyFog = false;
    halo.alwaysSelectAsActiveMesh = true;
    halo.thinInstanceSetBuffer('matrix', this.markerHaloMatrix, 16, false);
    halo.thinInstanceSetBuffer('color', this.markerHaloColor, 4, false);
    this.markerHalo = halo;
  }

  /**
   * Builds the marker's star and its outline from `model.marker`, in place of the last ones (the
   * same frame: never both, nor neither): none without the file, until it comes.
   */
  private async buildMarker(): Promise<void> {
    const template = await buildModel(this.scene, 'model.marker', this.deps.assets.get('model.marker', 'model'), [1, 1]);
    if (this.disposed) {
      for (const mesh of template.meshes) mesh.dispose(false, false);
      return;
    }
    const [mat, outlineMat] = this.markerMats!;
    const meshes: Mesh[] = [];
    for (const mesh of template.meshes) {
      const positions = mesh.getVerticesData(VertexBuffer.PositionKind)!;
      const normals = mesh.getVerticesData(VertexBuffer.NormalKind)!;
      const n = normals.length / 3;
      const outline = new Mesh('markerOutline', this.scene);
      const data = new VertexData();
      data.positions = positions.map((p, i) => p + normals[i] * 0.009);
      data.normals = normals.slice();
      data.indices = mesh.getIndices()!.slice();
      data.colors = new Float32Array(n * 4).fill(1);
      data.applyToMesh(outline, false);
      outline.material = outlineMat;
      // Smooth, pillowy shading: a hot core that blooms, brightest on the faces turned to the
      // viewer, softly deeper towards the ray tips.
      const colors = new Float32Array(n * 4);
      for (let i = 0; i < n; i++) {
        const r = Math.hypot(positions[i * 3], positions[i * 3 + 1]);
        const face = Math.abs(normals[i * 3 + 2]) * 0.75 + Math.max(0, normals[i * 3 + 1]) * 0.25;
        const k = (0.7 + 0.3 * face) * (1.45 - 0.6 * Math.min(1, r / 0.16));
        colors.set([k, k, k, 1], i * 4);
      }
      mesh.setVerticesData(VertexBuffer.ColorKind, colors);
      mesh.material = mat;
      for (const [m, color] of [[mesh, this.markerColor], [outline, this.markerOutlineColor]] as const) {
        m.isPickable = false;
        m.applyFog = false;
        m.alwaysSelectAsActiveMesh = true;
        m.thinInstanceSetBuffer('matrix', this.markerMatrix, 16, false);
        m.thinInstanceSetBuffer('color', color, 4, false);
        meshes.push(m);
      }
    }
    for (const mesh of this.marker) mesh.dispose(false, false);
    this.marker = meshes;
    this.markerFailed = template.failed ?? [];
    this.resetSnapshot();
  }

  // --- style -------------------------------------------------------------------------

  /** Applies the look and the live options (post-processing, fog, resolution, ...). */
  private applyLook(): void {
    if (this.disposed) return;
    const post = this.post;
    const ip = post?.imageProcessing;
    if (ip) {
      ip.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
      ip.contrast = LOOK.contrast;
      ip.vignetteWeight = LOOK.vignette;
      const curves = ip.colorCurves!;
      curves.globalSaturation = LOOK.saturation;
      curves.highlightsHue = LOOK.highlightsHue;
      curves.highlightsDensity = LOOK.highlightsDensity;
      curves.shadowsHue = LOOK.shadowsHue;
      curves.shadowsDensity = LOOK.shadowsDensity;
    }
    this.scene.fogDensity = LOOK.fog;
    this.applyResolution();
    this.markerPalette = LOOK.marker.map((hex) => {
      const c = Color3.FromHexString(hex).toLinearSpace();
      return [c.r * 1.2, c.g * 1.2, c.b * 1.2, 1];
    });
    this.lampColor.copyFrom(Color3.FromHexString(LOOK.lampColor).toLinearSpace());
    this.glowColor.copyFrom(Color3.FromHexString(LOOK.windowGlow).toLinearSpace());
    const minute = this.townMode ? this.townMinute : this.lastMinute;
    this.applyLighting(minute);
    this.lastLightMinute = minute;
    this.resetSnapshot();
  }

  // --- world building ----------------------------------------------------------------

  /** Roads and garden paths from the town metadata, each merged into one flat mesh. */
  private buildStreets(world: WorldStructure): void {
    const layers: [string, { x: number; z: number; w: number; d: number }[] | undefined, string, number][] = [
      ['streets', world.meta?.streets, 'material.road', 0.012],
      ['paths', world.meta?.paths, 'material.path', 0.014],
      ...fieldLayers(world.meta?.world?.fields, ''),
    ];
    for (const [name, all, material, y] of layers) {
      // Only the stretch of road in front of the viewed lot.
      const rects = all?.map((r) => this.clipToView(r, STREET_MARGIN)).filter((r) => r !== null);
      if (!rects?.length) continue;
      const parts = rects.map((r) => {
        const m = MeshBuilder.CreateGround(name, { width: r.w, height: r.d }, this.scene);
        m.position.set(r.x + r.w / 2, y, r.z + r.d / 2);
        m.bakeCurrentTransformIntoVertices();
        return m;
      });
      const merged = Mesh.MergeMeshes(parts, true, true);
      if (!merged) continue;
      merged.name = name;
      merged.material = this.lotMaterial(material);
      merged.receiveShadows = true;
      this.worldMeshes.push(merged);
    }
  }

  /** Whether a world point is inside the viewed lot (expanded by `margin`). */
  private inView(x: number, z: number, margin: number): boolean {
    const v = this.view;
    return !v || (x >= v.x - margin && z >= v.z - margin && x <= v.x + v.w + margin && z <= v.z + v.d + margin);
  }

  private clipToView(r: { x: number; z: number; w: number; d: number }, margin: number) {
    const v = this.view;
    if (!v) return r;
    const x0 = Math.max(r.x, v.x - margin);
    const z0 = Math.max(r.z, v.z - margin);
    const x1 = Math.min(r.x + r.w, v.x + v.w + margin);
    const z1 = Math.min(r.z + r.d, v.z + v.d + margin);
    return x1 > x0 && z1 > z0 ? { x: x0, z: z0, w: x1 - x0, d: z1 - z0 } : null;
  }

  /** The viewed house (walls, trims, windows, floors, roof, room lights); returns shadow casters. */
  private async buildLot(world: WorldStructure): Promise<Mesh[]> {
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
      if (!this.deps.content.object(o.def)?.stairs) continue;
      const lift = up(this.storeyOf(o.z));
      for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) holes.add((z + lift) * W + x);
      // The tile past the top step, on the storey above (as sim-core `storeys::stair_of`).
      const [lx, lz] = [
        [o.x + Math.floor((o.w - 1) / 2), o.z - 1],
        [o.x - 1, o.z + Math.floor((o.d - 1) / 2)],
        [o.x + Math.floor((o.w - 1) / 2), o.z + o.d],
        [o.x + o.w, o.z + Math.floor((o.d - 1) / 2)],
      ][o.rot % 4];
      landings.add((lz + lift) * W + lx);
    }
    const builds = [];
    // The ground first (its garden dressing is the lot's), then upstairs, then the basement.
    const order = [...Array.from({ length: count }, (_, k) => k), ...(lowest < 0 ? [-1] : [])];
    let topped = false;
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
      const built = this.house.build(world, view, k === 0 ? { covered, holes, landings } : { upper: true, basement: k < 0, covered, holes, landings });
      if (!built) {
        if (k === 0) break;
        continue;
      }
      // Lifted (or lowered) to its height, back over the ground rows.
      for (const mesh of built.meshes) {
        mesh.position.set(0, k * WALL_HEIGHT, -shift);
        mesh.metadata = { storey: k };
      }
      builds.push(built);
    }
    // Stairs down: from the ground, a shaft round each flight (the rest of the basement stays hidden).
    this.stairwells = [];
    for (const o of world.objects) {
      if (!this.deps.content.object(o.def)?.stairs || this.storeyOf(o.z) !== -1 || !this.inView(o.x + o.w / 2, o.z - this.rowOf(-1) + o.d / 2, 0)) continue;
      this.stairwells.push({ x: o.x, z: o.z - this.rowOf(-1), w: o.w, d: o.d });
    }
    if (lowest < 0 && this.view) this.buildPit(this.view);
    for (const r of this.stairwells) this.buildPit(r, 0, 0);
    const built = builds.length
      ? {
          meshes: builds.flatMap((b) => b.meshes),
          roofs: builds.flatMap((b) => b.roofs),
          casters: builds.flatMap((b) => b.casters),
          shrubs: builds[0].shrubs,
          trees: builds[0].trees,
          rooms: new Map(builds.flatMap((b) => [...b.rooms])),
          doors: builds.flatMap((b) => b.doors),
          surfaces: { faces: new Map(builds.flatMap((b) => [...b.surfaces.faces])), floors: new Map(builds.flatMap((b) => [...b.surfaces.floors])) },
        }
      : null;
    if (!built) {
      this.doors = [];
      // Unknown wall layout: fall back to the simulation's plain meshes.
      this.meshFromArrays('floors', world.meshes.floors, this.lotMaterial('material.floor'));
      const wallMat = this.lotMaterial('material.wall');
      this.walls = this.meshFromArrays('walls', world.meshes.walls, wallMat);
      this.wallsLow = this.meshFromArrays('wallsLow', world.meshes.wallsLow, wallMat);
      this.lotRooms = null;
      this.lotSurfaces = null;
      this.refreshLaid();
      return [];
    }
    this.worldMeshes.push(...built.meshes);
    this.roofs = built.roofs;
    // Roofs start hidden (walls down / cutaway); compile them now so walls-up doesn't hitch.
    for (const roof of built.roofs) {
      await this.pace();
      void roof.material?.forceCompilationAsync(roof).catch(() => {});
    }
    // Garden dressing split into species (hedges or mixed shrub borders, varied lot trees).
    this.decorPlaced = [...natureDecor('model.bush', built.shrubs), ...natureDecor('model.tree', built.trees, world, this.view)];
    for (const [key, items] of this.decorPlaced) {
      await this.pace();
      void this.placeDecor(key, items);
    }
    this.lotRooms = built.rooms;
    this.lotSurfaces = built.surfaces;
    this.doors = built.doors.map((leaf) => ({ leaf, open: -1, base: leaf.meshes[0]?.position.clone() ?? Vector3.Zero() }));
    this.refreshLaid();
    return built.casters;
  }

  /**
   * The dug-out lot under the ground, seen while its basement is in view (the landscape over it
   * is cut away then, see `applyDig`): earth walls round the plot and a floor below the basement's.
   */
  private buildPit(view: ViewRect, margin = PIT_MARGIN, storey = -1): void {
    const [x0, z0, x1, z1] = [view.x - margin, view.z - margin, view.x + view.w + margin, view.z + view.d + margin];
    const [y0, y1] = [-WALL_HEIGHT - 0.03, 0];
    const geo = new Geo().color('#6B5440');
    geo.poly([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0, 1, 0]);
    geo.color('#4A3A2C');
    geo.poly([[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]], [0, 0, 1]);
    geo.poly([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0, 0, -1]);
    geo.poly([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [1, 0, 0]);
    geo.poly([[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]], [-1, 0, 0]);
    const mesh = geo.toMesh('basementPit', this.scene, { colors: true });
    if (!mesh) return;
    mesh.material = this.lib.surface('material.soil', { vertexColors: true, doubleSided: true });
    mesh.receiveShadows = true;
    mesh.metadata = { storey };
    this.worldMeshes.push(mesh);
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
    // The basement itself in view: no daylight down there (its lamps light it, see `applyRoomLights`).
    const under = on && this.storey < 0;
    if (dig.on === on && dig.under === under) return;
    dig.on = on;
    dig.under = under;
    this.resetSnapshot();
  }

  /** The household's lamps (then dim fills for rooms without one) on the interior lights. */
  private placeRoomLights(world: WorldStructure, view: ViewRect | null): void {
    // Rooms and lamps on every storey shown (in lot rows: lifted to their height here).
    const shown = (z: number) => storeyShown(this.storeyOf(z), this.storey);
    // The viewed lot on every storey at once: each room's lamps (or fill) are placed once.
    const inView = (x: number, z: number) => {
      const row = z - this.rowOf(this.storeyOf(z));
      return !view || (x >= view.x && row >= view.z && x < view.x + view.w && row < view.z + view.d);
    };
    this.markWindowless(world, view);
    this.markRoomLights(world, view);
    const lights = this.lotRooms ? placeLights(world, inView, this.lotRooms, (def) => this.deps.content.object(def)?.light).filter((l) => shown(l.z)) : [];
    // With few lights to go round, the storey in view gets them first, then the nearest below it
    // (the sort is stable: within a storey, each room's first light still comes before any second);
    // lamps switched off last of all.
    const away = (z: number) => Math.abs(this.storeyOf(z) - this.storey);
    const off = (l: RoomLight) => (l.lamp !== undefined && this.lampsOff?.has(l.lamp) ? 1 : 0);
    lights.sort((a, b) => off(a) - off(b) || away(a.z) - away(b.z));
    this.lightsShort = lights.length > this.roomLights.length;
    this.lotLamps = lights.filter((l) => l.kind === 'lamp');
    this.gardenPools?.set(lights.filter((l) => l.kind === 'lamp' && l.room === 0 && this.storeyOf(l.z) === 0).map((l) => gardenPool(l, this.rowOf(0))));
    this.applyGardenPools();
    this.findUnlitRooms();
    this.roomLightUse = this.roomLights.map((light, i) => {
      const l = lights[i];
      if (l) light.position.set(l.x, l.y + this.storeyOf(l.z) * WALL_HEIGHT, l.z - this.rowOf(this.storeyOf(l.z)));
      else light.position.y = HIDDEN_Y;
      // A shade's side (`aim`): half a turn about its direction; all round, a whole turn.
      if (l?.dir) light.direction.set(l.dir[0], l.dir[1], l.dir[2]);
      const cone = l?.dir ? SHADED : ALL_ROUND;
      if (light.angle !== cone) {
        light.angle = cone;
        light.innerAngle = l?.dir ? 0 : ALL_ROUND;
      }
      return l ?? null;
    });
    this.roomLightOn = this.roomLightUse.map((use) => (use?.lamp !== undefined ? (this.lampOn.get(use.lamp) ?? 1) : use && this.fillOut(use.room) ? 0 : 1));
  }

  /** A room's fill is out: everyone there is asleep, or nobody awake is in a room without windows. */
  private fillOut(room: number): boolean {
    return this.darkRooms.has(room) || (this.dimmed.has(room) && !this.awakeRooms.has(room));
  }

  /**
   * Finds the rooms without windows (`windowless`), and keeps those of the storey in view out of
   * daylight as the basement is (`dimmed`): every one of their tiles in view, on the materials'
   * mask (`DigState.darken`).
   */
  private markWindowless(world: WorldStructure, view: ViewRect | null): void {
    const W = world.width;
    const room = (x: number, z: number) => (x >= 0 && z >= 0 && x < W && z < world.depth ? world.rooms[z * W + x] : 0);
    // Which rooms have none only changes with the walls (new `rooms` and `openings` arrays).
    const of = [world.rooms, world.openings, world.diagonals];
    if (of.some((a, i) => a !== this.windowless.of[i])) {
      // A window lets daylight into the rooms on both its sides (sim-core `rooms.rs`).
      const lit = new Set<number>();
      for (const o of world.openings ?? []) {
        if (o.kind !== 'window') continue;
        lit.add(room(o.x, o.z));
        lit.add(o.axis === 'h' ? room(o.x, o.z - 1) : room(o.x - 1, o.z));
      }
      for (const d of world.diagonals ?? []) if (d.kind === 'window') for (const r of d.rooms) lit.add(r);
      const rooms = new Set<number>();
      for (const r of world.rooms) if (r !== 0 && !lit.has(r)) rooms.add(r);
      for (const d of world.diagonals ?? []) for (const r of d.rooms) if (r !== 0 && !lit.has(r)) rooms.add(r);
      this.windowless = { rooms, of };
    }
    const dark = this.windowless.rooms;

    const dimmed = new Set<number>();
    let dimming: DarkTiles | null = null;
    if (view && this.storey >= 0 && dark.size) {
      const row = this.rowOf(this.storey);
      const [vw, vd] = [view.w, view.d];
      // The view's tiles: dark whole, or the halves of one split by a diagonal wall that are; and
      // the box round the dark ones (x0, z0, x1, z1, in the view), all the materials are given.
      const tiles = new Uint8Array(vw * vd);
      const box = [vw, vd, 0, 0];
      const mark = (x: number, z: number, bits: number) => {
        tiles[z * vw + x] = bits;
        box.splice(0, 4, Math.min(box[0], x), Math.min(box[1], z), Math.max(box[2], x + 1), Math.max(box[3], z + 1));
      };
      const diagonals = new Map((world.diagonals ?? []).map((d) => [d.z * W + d.x, d]));
      const B = WINDOWLESS_BITS;
      for (let z = 0; z < vd; z++) {
        for (let x = 0; x < vw; x++) {
          const i = (z + view.z + row) * W + x + view.x;
          const d = diagonals.get(i);
          if (!d) {
            if (!dark.has(world.rooms[i])) continue;
            mark(x, z, B.tile);
            dimmed.add(world.rooms[i]);
            continue;
          }
          let bits = 0;
          d.rooms.forEach((r, h) => {
            if (!dark.has(r)) return;
            bits |= h === 0 ? B.half0 : B.half1;
            dimmed.add(r);
          });
          if (bits) mark(x, z, bits | (d.axis === 'dn' ? B.dn : 0));
        }
      }
      if (dimmed.size) {
        const [x0, z0, w, d] = [box[0], box[1], box[2] - box[0], box[3] - box[1]];
        const crop = new Uint8Array(w * d);
        for (let z = 0; z < d; z++) crop.set(tiles.subarray((z0 + z) * vw + x0, (z0 + z) * vw + x0 + w), z * w);
        // From under the floor of the storey in view to just under the next one's.
        dimming = { x0: view.x + x0, z0: view.z + z0, w, d, y0: this.storey * WALL_HEIGHT - 0.5, y1: (this.storey + 1) * WALL_HEIGHT - 0.02, tiles: crop };
      }
    }
    this.dimmed = dimmed;
    this.dimming = dimming;
    this.applyDimming();
  }

  /** Life stage ids of babies (content `baby`: they don't count as someone up, as in sim-core). */
  private babyStages(): ReadonlySet<string> {
    const stages = this.deps.content.lifeStages;
    if (this.babies?.of !== stages) this.babies = { of: stages, ids: new Set(stages.filter((s) => s.baby).map((s) => s.id)) };
    return this.babies.ids;
  }
  private babies: { of: readonly unknown[]; ids: ReadonlySet<string> } | null = null;

  /**
   * The rooms of the lot in view on every storey shown, for keeping each lamp's light in its room
   * (`RoomLightState`): 1 outdoors, 2 and up a room, 0 a tile split by a diagonal wall (lit by all).
   */
  private markRoomLights(world: WorldStructure, view: ViewRect | null): void {
    if (!this.lib.rooms.enabled || !view) {
      this.roomTiles = null;
      this.applyDimming();
      return;
    }
    const W = world.width;
    const [k0, n] = this.storey < 0 ? [this.storey, 1] : [0, this.storey + 1];
    const tiles = new Uint16Array(view.w * view.d * n);
    const ids = new Map<number, number>();
    const diagonals = new Set((world.diagonals ?? []).map((d) => d.z * W + d.x));
    for (let k = 0; k < n; k++) {
      const row = this.rowOf(k0 + k);
      for (let z = 0; z < view.d; z++) {
        for (let x = 0; x < view.w; x++) {
          const i = (z + view.z + row) * W + x + view.x;
          if (z + view.z + row >= world.depth || diagonals.has(i)) continue;
          const r = world.rooms[i] ?? 0;
          let id = ids.get(r);
          if (id === undefined) ids.set(r, (id = r === 0 ? 1 : ids.size + 2));
          tiles[(k * view.d + z) * view.w + x] = id;
        }
      }
    }
    this.roomTiles = { x0: view.x, z0: view.z, w: view.w, d: view.d, storey: k0, storeys: n, tiles };
    this.applyDimming();
  }

  /** The windowless rooms' darkness on the materials (`dimming`), and the rooms lamps keep to (`roomTiles`): none while the menus' town is shown. */
  private applyDimming(): void {
    const darker = this.lib.dig.darken(this.townMode ? null : this.dimming);
    if (this.lib.rooms.show(this.townMode ? null : this.roomTiles) || darker) this.resetSnapshot();
  }

  /**
   * Where residents are this frame (`darkRooms`, `awakeRooms`, `occupiedRooms`, `movers`; away
   * residents not counted), and which neighbours' houses have someone up in them (`neighboursUp`).
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
    // Neighbours' houses only matter once their windows can glow.
    const houses = this.townMode || this.lampsLevel <= 0 ? [] : (this.street?.houses ?? []);
    const up = this.neighboursUp;
    up.length = houses.length;
    up.fill(false);
    const babies = this.babyStages();
    const count = Math.min(curr[layout.header.simCount], MAX_SIMS);
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      if (curr[o + k.away] !== 0) continue;
      const px = curr[o + k.x];
      const pz = curr[o + k.z];
      if (curr[o + k.moving] !== 0) {
        // (An older layout without the way ahead: where they are, so only a doorway they're in opens.)
        const [wx, wz, ax, az] = [k.wayX, k.wayZ, k.aheadX, k.aheadZ];
        if (wx === undefined || wz === undefined || ax === undefined || az === undefined) movers.push(px, pz, px, pz, px, pz);
        else movers.push(px, pz, curr[o + wx], curr[o + wz], curr[o + ax], curr[o + az]);
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
      const h = houses.findIndex(([x0, z0, x1, z1]) => px >= x0 && px <= x1 && gz >= z0 && gz <= z1);
      if (h >= 0) up[h] = true;
    }
    for (const room of awake) dark.delete(room);
    if (houses.length && this.street?.setUp(up)) this.resetSnapshot();
  }

  /**
   * Front doors and gates stand shut, and a bathroom's door while someone is in there; each opens
   * only for someone going through it (`goingThrough`: not for someone walking past), and swings
   * back once they're through. `rate`: how fast the game runs (`GAME_RATE`).
   */
  private swingDoors(rate: number): void {
    if (!this.doors.length && !this.gates.length) return;
    const movers = this.movers;
    const step = Math.min(1, (this.frameDt / 1000 / DOOR_SWING) * Math.min(Math.max(rate, 1), 10));
    const swung = (target: number, at: number) => (target > at ? Math.min(target, at + step) : Math.max(target, at - step));
    for (const gate of this.gates) {
      if (gate.mesh.isDisposed()) continue;
      const way = goingThrough(movers, gate.leaf.axis, gate.leaf.centre, gate.open === 1);
      // It opens away from the one coming through, and shuts the way it opened.
      if (way && gate.open === 0) gate.towards = way;
      const open = swung(way ? 1 : 0, gate.open);
      if (open === gate.open) continue;
      gate.open = open;
      swingGate(gate.mesh, gate.leaf, ease(open), gate.towards);
      this.resetSnapshot();
    }
    let moved = false;
    for (const door of this.doors) {
      const { leaf } = door;
      if (leaf.meshes[0]?.isDisposed()) continue;
      // (Both in lot rows: the storey is in the row.)
      const going = goingThrough(movers, leaf.axis, leaf.centre, door.open === 1) !== 0;
      const engaged = leaf.bath !== 0 && this.occupiedRooms.has(leaf.bath);
      const target = going || !(leaf.front || engaged) ? 1 : 0;
      const open = door.open < 0 ? target : swung(target, door.open);
      if (open === door.open) continue;
      door.open = open;
      // Eased: quick off the frame, slowing as it meets the jamb or the wall. One that slides stays
      // turned shut and slides into the wall (from where its storey has it).
      const k = ease(open);
      const { slide } = leaf;
      for (const mesh of leaf.meshes) {
        if (slide) mesh.position.set(door.base.x + slide[0] * k, door.base.y, door.base.z + slide[1] * k);
        mesh.rotation.y = slide ? leaf.close : leaf.close * (1 - leaf.opens * k);
      }
      moved = true;
    }
    if (!moved) return;
    this.sunShadows.moved();
    this.resetSnapshot();
  }

  /** Eases each interior light towards out (its room dark) or on, a lamp as its switch (`lampOn`); true if any changed. */
  private fadeRoomLights(dt: number): boolean {
    let changed = false;
    const step = Math.min(1, dt / LIGHTS_OUT_FADE);
    this.roomLightUse.forEach((use, i) => {
      const on = this.roomLightOn[i] ?? 1;
      if (use?.lamp !== undefined) {
        const now = this.lampOn.get(use.lamp) ?? 1;
        if (now !== on) changed = true;
        this.roomLightOn[i] = now;
        return;
      }
      const target = use && this.fillOut(use.room) ? 0 : 1;
      if (on === target) return;
      this.roomLightOn[i] = target > on ? Math.min(target, on + step) : Math.max(target, on - step);
      changed = true;
    });
    return changed;
  }

  /** Eases each room's window glow towards out (the room dark, or its lamps all off) or on, as its lights; true if any changed. */
  private fadeWindowGlow(dt: number): boolean {
    let changed = false;
    const step = Math.min(1, dt / LIGHTS_OUT_FADE);
    const glow = this.windowGlowOn;
    for (const room of this.darkRooms) if (!glow.has(room)) glow.set(room, 1);
    for (const room of this.unlitRooms) if (!glow.has(room)) glow.set(room, 1);
    for (const [room, on] of glow) {
      const target = this.darkRooms.has(room) || this.unlitRooms.has(room) ? 0 : 1;
      if (on === target && target === 0) continue;
      const next = target > on ? Math.min(target, on + step) : Math.max(target, on - step);
      if (next === 1) glow.delete(room);
      else glow.set(room, next);
      changed = true;
    }
    return changed;
  }

  private decorModel(key: string): Promise<ModelTemplate> {
    return buildModel(this.scene, key, this.deps.assets.get(key, 'model'), [1, 1]);
  }

  /** Garden dressing as thin instances of a cached template (`deco:` keys). */
  private async placeDecor(key: string, items: [number, number, number, number][]): Promise<void> {
    if (!items.length) return;
    const template = await this.template(`deco:${key}`, () => this.decorModel(key));
    const matrices = new Float32Array(items.length * 16);
    items.forEach(([x, z, s, yaw], i) => {
      Matrix.ComposeToRef(this.vScale.setAll(s), Quaternion.RotationYawPitchRollToRef(yaw, 0, 0, this.qTmp), this.vTmp.set(x, 0, z), this.mOut);
      this.mOut.copyToArray(matrices, i * 16);
    });
    for (const mesh of template.meshes) {
      mesh.thinInstanceSetBuffer('matrix', matrices, 16, true);
      mesh.thinInstanceRefreshBoundingInfo(false);
      mesh.receiveShadows = true;
      // The garden goes with the ground dug away over the basement.
      this.decorMeshes.add(mesh);
      mesh.isVisible = this.storey >= 0;
    }
    // (In place of a box set aside for a file that came, if there was one.)
    this.templates.hideStale(`deco:${key}`);
    this.resetSnapshot();
  }

  private applyWallMode(): void {
    const mode = this.wallMode;
    this.walls?.setEnabled(mode === 'up');
    this.wallsLow?.setEnabled(mode !== 'up');
    this.roofShown = this.roofVisible();
    this.showRoofs();
    // The house casts in every mode, its walls at full height (the shadow pass has no cutaway)
    // and its shadow-only ceiling over the rooms: the sun comes in through windows and doorways.
    this.updateShadowList();
    this.lib.wallCut.params.x = mode === 'down' ? 1 : 0;
    this.lib.wallCut.params.y = WALL_STUB;
    this.cutLook = -1;
    this.updateCutaway();
  }

  /** The viewed lot's roof: only with walls up, and not when zoomed in (it would fill the view). */
  private roofVisible(): boolean {
    return this.wallMode === 'up' && this.gameCamera.radius > ROOF_MIN_DISTANCE;
  }

  /**
   * Snapshot replays were observed to keep the camera of their recording, so a moving camera
   * re-records each frame (normal rendering cost) and replay resumes once it is still.
   */
  private followCamera(): void {
    const c = this.camera;
    const v = this.lastView;
    const t = c.target;
    const o = c.targetScreenOffset;
    if (v[0] === c.alpha && v[1] === c.beta && v[2] === c.radius && v[3] === t.x && v[4] === t.y && v[5] === t.z && v[6] === o.x && v[7] === o.y) return;
    v[0] = c.alpha;
    v[1] = c.beta;
    v[2] = c.radius;
    v[3] = t.x;
    v[4] = t.y;
    v[5] = t.z;
    v[6] = o.x;
    v[7] = o.y;
    this.resetSnapshot();
  }

  /** Cutaway: walls facing the camera drop when the view direction crosses a quadrant. */
  private updateCutaway(): void {
    let bits = 0;
    if (this.wallMode === 'cutaway') {
      // Camera looks from its position towards the target: direction -(cos a, sin a) in xz.
      const dx = -Math.cos(this.gameCamera.alpha);
      const dz = -Math.sin(this.gameCamera.alpha);
      if (dx > 0.3) bits |= 1;
      if (dx < -0.3) bits |= 2;
      if (dz > 0.3) bits |= 4;
      if (dz < -0.3) bits |= 8;
    }
    if (bits === this.cutLook) return;
    this.cutLook = bits;
    this.lib.wallCut.look.set(bits & 1 ? 1 : 0, bits & 2 ? 1 : 0, bits & 4 ? 1 : 0, bits & 8 ? 1 : 0);
    this.cutWallItems(bits);
    this.resetSnapshot();
  }

  /** Things on a wall hidden with the wall they hang on, and their matrices to show them again. */
  private readonly wallItemsHidden = new Map<number, Float32Array>();

  /**
   * Things on a wall go with their wall: all of them with walls down, and in cutaway those on the
   * walls that drop, from either side (`wallCutBits`). Not while something for a wall is held:
   * then they show, as what may be in its way.
   */
  private cutWallItems(bits: number): void {
    const down = this.wallMode === 'down';
    for (const p of this.placed) {
      if (p.wallCut === undefined || p.id === this.held?.id) continue;
      const hide = !this.ghostOnWall && (down || (bits & p.wallCut) !== 0);
      const saved = this.wallItemsHidden.get(p.id);
      if (hide && !saved) {
        const m = p.pop.matrices.slice(p.pop.index * 16, p.pop.index * 16 + 16);
        this.wallItemsHidden.set(p.id, m);
        const gone = m.slice();
        gone.fill(0, 0, 12);
        this.writeInstance(p.pop, gone);
      } else if (!hide && saved) {
        this.wallItemsHidden.delete(p.id);
        this.writeInstance(p.pop, saved);
      }
    }
  }

  /**
   * The cutaway look bits (geometry.ts `LOOK`) that drop the wall a thing on a wall hangs on, as
   * house.ts `edgeMask` gives them: looking towards a room on either side of it (along any of the
   * tiles it hangs across), and the way it faces (its wall then stands between the camera and it,
   * even with no room behind).
   */
  private wallCutBits(world: WorldStructure, o: ObjectPlacement): number {
    const W = world.width;
    const k = this.storeyOf(o.z);
    // A room on the thing's own storey (lot rows run on into the next storey's).
    const room = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < world.depth && this.storeyOf(z) === k && world.rooms[z * W + x] > 0;
    const r = o.rot % 4;
    let bits: number = [CUT.PosZ, CUT.PosX, CUT.NegZ, CUT.NegX][r];
    if (r % 2 === 0) {
      // Its back on the wall along grid line `z`, between rows z - 1 and z.
      const z = r === 0 ? o.z : o.z + o.d;
      for (let x = o.x; x < o.x + o.w; x++) bits |= (room(x, z) ? CUT.PosZ : 0) | (room(x, z - 1) ? CUT.NegZ : 0);
    } else {
      const x = r === 1 ? o.x : o.x + o.w;
      for (let z = o.z; z < o.z + o.d; z++) bits |= (room(x, z) ? CUT.PosX : 0) | (room(x - 1, z) ? CUT.NegX : 0);
    }
    return bits;
  }

  /** How the player resized an object (undefined: as its kind comes). */
  private objectResize(o: ObjectPlacement): ObjectResize | undefined {
    const def = this.deps.content.object(o.def);
    const from = def?.footprint ?? [1, 1];
    const tiles: [number, number] = o.rot % 2 === 0 ? [o.w, o.d] : [o.d, o.w];
    const size: [number, number] = o.size ? [o.size[0] / 100, o.size[1] / 100] : tiles;
    const resize = { from: [from[0], from[1]] as [number, number], size, tall: o.tall ?? 0, ceiling: def?.layer === 'ceiling' };
    return resizeKey(resize) ? resize : undefined;
  }

  /** Model key for an object. */
  private objectModel(o: ObjectPlacement): string {
    return this.deps.content.object(o.def)?.model ?? `model.${o.def}`;
  }

  /** The colours an object's parts are drawn in: the player's, on the parts its model paints. */
  private objectColors(o: ObjectPlacement): PartColors | undefined {
    return paintColors(this.deps.assets.get(this.objectModel(o), 'model'), o.colors);
  }

  /** Returns the body meshes whose sun shadows are in `objectShadowMerge` (Low): they cast no shadow of their own. */
  private async buildObjects(world: WorldStructure): Promise<Set<Mesh>> {
    // Things on the storeys up to the one in view, each lifted to its storey (from its lot rows).
    // From the ground up, stairs down to the basement show in their stairwell too.
    const down = (o: ObjectPlacement) => this.storey >= 0 && this.storeyOf(o.z) === -1 && !!this.deps.content.object(o.def)?.stairs;
    const shown = world.objects.filter((o) => (storeyShown(this.storeyOf(o.z), this.storey) || down(o)) && this.inView(o.x + o.w / 2, o.z - this.rowOf(this.storeyOf(o.z)) + o.d / 2, 0));
    // One template per model and size (things the player resized get their own).
    const byModel = Map.groupBy(shown, (o) => `${o.def}|${this.objectModel(o)}${resizeKey(this.objectResize(o))}${colorsKey(this.objectColors(o))}`);
    const templates = await perfTimeAsync('render: object templates', async () => {
      // Generated models build as they're asked for: one at a time, so a build behind the overview can pause between them.
      const pending: Promise<ModelTemplate>[] = [];
      for (const list of byModel.values()) {
        await this.pace();
        pending.push(this.template(...this.objectTemplate(list[0])));
      }
      return Promise.all(pending);
    });
    // What each tile's top belongs to, for the things standing on tops.
    const tops = new Map<string, number>();
    for (const o of shown) {
      if (!this.deps.content.object(o.def)?.surface) continue;
      for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) tops.set(`${x}:${z}`, o.id);
    }
    // Soft contact shadows under every object, sized to its footprint.
    const blobs = new Float32Array(Math.max(1, shown.length) * 16);
    const mirrors: MirrorInstance[] = [];
    const moving: MovingObject[] = [];
    const glows: typeof this.glows = [];
    // Low draws the objects' sun shadows from one merged mesh (their bodies, at rest) rather than
    // from every template's thin instances: the lot's furniture costs a draw, not a hundred.
    const merge = this.quality.mergedObjectShadows;
    const shadowSources: ShadowSource[] = [];
    const merged = new Set<Mesh>();
    let partsCast = false;
    const instances = perfStart('render: object instances');
    [...byModel.values()].forEach((list, n) => {
      const template = templates[n];
      const matrices = new Float32Array(list.length * 16);
      // Moving parts (a fridge's doors) draw from buffers of their own, each instance posed on its own.
      const motion = template.motion;
      const motions = motion && motionsOf(this.deps.assets.get(this.objectModel(list[0]), 'model'));
      const partCount = motion ? Math.max(-1, ...motion.parts) + 1 : 0;
      // (A part's glow, if any, moves with it.)
      const glowOf = (k: number) => (template.glows ?? []).filter((g) => g.part === k).map((g) => g.mesh);
      const parts = Array.from({ length: partCount }, (_, k) => ({ meshes: [...template.meshes.filter((_, i) => motion!.parts[i] === k), ...glowOf(k)], matrices: new Float32Array(list.length * 16) }));
      const vary = this.deps.assets.get(this.objectModel(list[0]), 'model')?.vary ?? 0;
      list.forEach((o, i) => {
        const { turn, size } = placementVariation(vary, o.x, o.z);
        const k = this.storeyOf(o.z);
        const D = this.rowOf(k);
        // Things on a top stand on it.
        const [y0, z] = [k * WALL_HEIGHT + (o.lift ?? 0), o.z - D];
        // Things that stand anywhere on their tile are moved off its middle (picking and the shadow follow).
        const centre = objectCentre(o);
        const [ox, oz] = [centre.x - o.x - o.w / 2, centre.z - o.z - o.d / 2];
        Quaternion.RotationYawPitchRollToRef((o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn, 0, 0, this.qTmp);
        Matrix.ComposeToRef(this.vScale.setAll(size), this.qTmp, this.vTmp.set(centre.x, y0, centre.z - D), this.mOut);
        this.mOut.copyToArray(matrices, i * 16);
        const def = this.deps.content.object(o.def);
        const layer = def?.layer;
        const bottom = layer === 'wall' || layer === 'ceiling' ? Math.max(0, Math.min(template.bottom ?? 0, template.height - 0.05)) : 0;
        // Something small on a top is picked by its own disc, not the whole tile (the table around it stays clickable).
        // Anything else by what's drawn of it (a pot on a rug leaves the rug around it clickable),
        // never past its tiles; something held open (a wall bed folded down) by its whole tiles.
        let [px, pz] = layer === 'surface' ? [centre.x - this.deps.content.radius(def) * 1.2, centre.z - D - this.deps.content.radius(def) * 1.2] : [o.x + ox, z + oz];
        let [pw, pd] = layer === 'surface' ? [this.deps.content.radius(def) * 2.4, this.deps.content.radius(def) * 2.4] : [o.w, o.d];
        const shape = o.open && motions?.open ? undefined : template.outline;
        if (layer !== 'surface' && shape) {
          const yaw = (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn;
          const [c, s] = [Math.cos(yaw), Math.sin(yaw)];
          // Model x and z turned by the yaw (as `RotationYawPitchRoll`: x -> (c, -s), z -> (s, c)).
          const xs: number[] = [];
          const zs: number[] = [];
          for (const mx of [shape.minX, shape.maxX]) {
            for (const mz of [shape.minZ, shape.maxZ]) {
              xs.push(centre.x + (mx * c + mz * s) * size);
              zs.push(centre.z - D + (-mx * s + mz * c) * size);
            }
          }
          const [x0, x1] = [Math.max(o.x + ox, Math.min(...xs) - 0.03), Math.min(o.x + ox + o.w, Math.max(...xs) + 0.03)];
          const [z0, z1] = [Math.max(z + oz, Math.min(...zs) - 0.03), Math.min(z + oz + o.d, Math.max(...zs) + 0.03)];
          if (x1 > x0 && z1 > z0) [px, pz, pw, pd] = [x0, z0, x1 - x0, z1 - z0];
        }
        this.placed.push({
          id: o.id,
          y0,
          minX: px,
          minZ: pz,
          maxX: px + pw,
          maxZ: pz + pd,
          height: template.height,
          bottom,
          wallCut: layer === 'wall' ? this.wallCutBits(world, o) : undefined,
          host: layer === 'surface' ? tops.get(`${o.x}:${o.z}`) : undefined,
          // (Its glow, if any, goes with the body: springing into place, held in buy mode.)
          pop: { meshes: [...template.meshes.filter((_, j) => !motion || motion.parts[j] < 0), ...glowOf(-1)], matrices, index: i, ...(partCount ? { moving: { parts, poses: restPoses(partCount) } } : {}) },
        });
        if (motion && motions) {
          // A lamp's fan turns while it's on; a fire's flames (its uses' clips) burn while it's lit.
          const light = def?.light;
          const burns = light?.fire ? [...new Set(Object.values(motions.uses).flatMap((clips) => clips.map((c) => c.clip)))] : undefined;
          moving.push({ id: o.id, target: this.placed[this.placed.length - 1].pop, motion, motions, interactions: (def?.interactions ?? []).map((it) => it.id), open: !!o.open, stage: o.growth, slotOf: slotOf(o, def), lamp: !!light && light.switch !== false, burns });
        }
        // Mirrors on a moving part (a mirror cabinet's door) follow that part's buffer.
        for (const [part, surfaces] of Map.groupBy(template.mirrors ?? [], (s) => s.part ?? -1)) {
          mirrors.push({ surfaces, matrices: part >= 0 && parts[part] ? parts[part].matrices : matrices, index: i });
        }
      });
      // At rest, the parts are where the body is.
      for (const part of parts) part.matrices.set(matrices);
      template.meshes.forEach((mesh, j) => {
        const part = motion ? motion.parts[j] : -1;
        mesh.thinInstanceSetBuffer('matrix', part < 0 ? matrices : parts[part].matrices, 16, part < 0);
        mesh.thinInstanceRefreshBoundingInfo(false);
        mesh.receiveShadows = true;
        // (Something see-through still casts from its own thin instances; a moving part does not cast
        // on Low, where the merged mesh is the lot's one object caster.)
        if (merge && !mesh.material?.needAlphaBlending() && !mesh.material?.needAlphaTesting()) {
          merged.add(mesh);
          if (part < 0) shadowSources.push({ mesh, matrices });
        } else if (part >= 0) partsCast = true;
      });
      for (const { mesh, part, screen } of template.glows ?? []) {
        // Strengths per instance (`applyGlow`), by the room each stands in; on a moving part, its buffer.
        const colors = new Float32Array(list.length * 4);
        const still = part < 0 || !parts[part];
        mesh.thinInstanceSetBuffer('matrix', still ? matrices : parts[part].matrices, 16, still);
        mesh.thinInstanceSetBuffer('color', colors, 4, false);
        mesh.thinInstanceRefreshBoundingInfo(false);
        const W = world.width;
        const roomOf = (o: ObjectPlacement) => world.rooms[Math.floor(o.z + o.d / 2) * W + Math.floor(o.x + o.w / 2)] ?? 0;
        // A lamp's shade glows with the lamp (switched on or off), anything else's with its room's lights.
        const light = this.deps.content.object(list[0].def)?.light;
        const lamps = list.map((o) => (light && light.switch !== false ? o.id : -1));
        glows.push({ mesh, colors, rooms: list.map(roomOf), lamps, ids: list.map((o) => o.id), screen: !!screen });
      }
    });
    perfEnd(instances);
    this.movingPartsCast = partsCast;
    const shadowsSpan = perfStart('render: object shadows');
    this.objectShadowParts.clear();
    this.objectShadowMerge = this.mergeObjectShadows(shadowSources);
    if (this.objectShadowMerge) this.worldMeshes.push(this.objectShadowMerge);
    for (const g of this.glows) if (!glows.some((n) => n.mesh === g.mesh)) g.mesh.setEnabled(false);
    this.glows = glows;
    // A night clip that lights a thing glowing with its room (the candles' flames) stands only while the room's lights are on.
    const glowRoom = new Map<number, number>();
    for (const g of glows) if (!g.screen) g.ids.forEach((id, k) => g.lamps[k] < 0 && g.rooms[k] && glowRoom.set(id, g.rooms[k]));
    this.motion.setUnlit((id) => {
      const room = glowRoom.get(id);
      return room !== undefined && (this.windowGlowOn.get(room) ?? 1) < 0.5;
    });
    this.applyGlow();
    this.mirrors.set(mirrors);
    this.motion.set(moving);
    shown.forEach((o, i) => {
      const k = i * 16;
      const s = this.storeyOf(o.z);
      const D = this.rowOf(s);
      // Nothing on a wall, the ceiling or the floor's surface (rugs) sits on a contact shadow;
      // things on a top get a small one there, the size of what they take on it.
      const def = this.deps.content.object(o.def);
      const layer = def?.layer ?? 'floor';
      const flat = layer === 'floor' || layer === 'surface' ? 1 : 0;
      const [sw, sd] = layer === 'surface' ? [this.deps.content.radius(def) * 2.3, this.deps.content.radius(def) * 2.3] : [o.w * 1.15, o.d * 1.15];
      blobs[k] = sw * flat;
      blobs[k + 5] = flat;
      blobs[k + 10] = sd * flat;
      const centre = objectCentre(o);
      blobs[k + 12] = centre.x;
      blobs[k + 13] = 0.018 + s * WALL_HEIGHT + (o.lift ?? 0);
      blobs[k + 14] = centre.z - D;
      blobs[k + 15] = 1;
    });
    if (!shown.length) return merged;
    const shadows = this.objectShadow.clone('objectShadows', null, true, false) as Mesh;
    shadows.setEnabled(true);
    shadows.thinInstanceSetBuffer('matrix', blobs, 16, true);
    const blobIndex = new Map(shown.map((o, i) => [o.id, i]));
    for (const p of this.placed) {
      const index = blobIndex.get(p.id);
      if (index !== undefined) p.shadow = { meshes: [shadows], matrices: blobs, index };
    }
    shadows.thinInstanceCount = shown.length;
    shadows.thinInstanceRefreshBoundingInfo(false);
    this.worldMeshes.push(shadows);
    perfEnd(shadowsSpan);
    return merged;
  }

  /**
   * One mesh of every object body, for the sun's shadow map on Low: a template's thin instances
   * are baked into world-space triangles here, and one again as it hides or moves (`bakeObjectShadow`;
   * the template is left out of the shadow list, see `buildObjects`). Its layer mask keeps it
   * out of the camera's pass.
   */
  private mergeObjectShadows(sources: ShadowSource[]): Mesh | null {
    // (Only read: the template's own arrays, no copies.)
    const parts: (ShadowPart & { idx: ArrayLike<number> })[] = [];
    let vertices = 0;
    let indexCount = 0;
    for (const { mesh, matrices } of sources) {
      const pos = mesh.getVerticesData(VertexBuffer.PositionKind, false, false);
      const idx = mesh.getIndices(false, false);
      if (!pos || !idx) continue;
      const count = matrices.length / 16;
      // A one-part model keeps the part's own transform; a merged one's is already in its vertices.
      parts.push({ pos, idx, world: mesh.computeWorldMatrix(true).clone(), matrices, first: vertices });
      vertices += (pos.length / 3) * count;
      indexCount += idx.length * count;
    }
    if (!indexCount) return null;
    const positions = new Float32Array(vertices * 3);
    const indices = vertices > 65535 ? new Uint32Array(indexCount) : new Uint16Array(indexCount);
    let at = 0;
    for (const part of parts) {
      const n = part.pos.length / 3;
      for (let i = 0; i < part.matrices.length / 16; i++) {
        this.bakeShadowInstance(part, i, positions);
        const base = part.first + i * n;
        for (let k = 0; k < part.idx.length; k++) indices[at++] = base + part.idx[k];
      }
      const list = this.objectShadowParts.get(part.matrices);
      if (list) list.push(part);
      else this.objectShadowParts.set(part.matrices, [part]);
    }
    this.objectShadowPositions = positions;
    const data = new VertexData();
    data.positions = positions;
    data.indices = indices;
    const mesh = new Mesh('objectShadowsMerged', this.scene);
    // (Updatable: an instance hidden or moved is baked again in place, `bakeObjectShadow`.)
    data.applyToMesh(mesh, true);
    if (!this.objectShadowMaterial) {
      this.objectShadowMaterial = new StandardMaterial('objectShadowsMerged', this.scene);
      this.objectShadowMaterial.disableLighting = true;
      this.objectShadowMaterial.backFaceCulling = false;
    }
    mesh.material = this.objectShadowMaterial;
    mesh.isPickable = false;
    mesh.receiveShadows = false;
    // The shadow map's custom list ignores layer masks; the camera's pass does not.
    // (Its world matrix is frozen with the world's meshes, `buildWorld`.)
    mesh.layerMask = 0;
    return mesh;
  }

  /**
   * Instance `i` of a body mesh in the merged shadow mesh's `positions`, as its matrix stands now: a thin
   * instance draws as `instance × mesh world`. (One hidden, scaled to nothing, is every vertex in one
   * point: no triangles, no shadow.) Returns where in `positions` it starts (`part.pos.length` floats).
   */
  private bakeShadowInstance(part: ShadowPart, i: number, positions: Float32Array): number {
    const m = this.mOut;
    const v = this.vTmp;
    Matrix.FromArrayToRef(part.matrices, i * 16, this.shadowInstance);
    this.shadowInstance.multiplyToRef(part.world, m);
    const { pos } = part;
    const start = (part.first + (i * pos.length) / 3) * 3;
    for (let p = 0; p < pos.length; p += 3) {
      Vector3.TransformCoordinatesFromFloatsToRef(pos[p], pos[p + 1], pos[p + 2], m, v);
      positions[start + p] = v.x;
      positions[start + p + 1] = v.y;
      positions[start + p + 2] = v.z;
    }
    return start;
  }

  /** On Low: an object instance written anew (hidden, shown, moved) casts from where it is now in the merged shadow mesh. */
  private bakeObjectShadow(target: PopTarget): void {
    const parts = this.objectShadowParts.get(target.matrices);
    const mesh = this.objectShadowMerge;
    if (!parts || !mesh || mesh.isDisposed()) return;
    const buffer = mesh.getVertexBuffer(VertexBuffer.PositionKind);
    const positions = this.objectShadowPositions;
    for (const part of parts) {
      const start = this.bakeShadowInstance(part, target.index, positions);
      buffer?.updateDirectly(positions.subarray(start, start + part.pos.length), start);
    }
  }

  /** An object's template key, and how to build it. */
  private objectTemplate(o: ObjectPlacement): [string, () => Promise<ModelTemplate>] {
    const key = this.objectModel(o);
    const def = this.deps.content.object(o.def);
    const resize = this.objectResize(o);
    const colors = this.objectColors(o);
    const id = `object:${key}${resizeKey(resize)}${colorsKey(colors)}`;
    if (colors && colorsKey(colors)) this.paintedTemplates.add(id);
    // Without its file, a box in its place, so it can still be seen and clicked, until the file comes
    // (`refreshModels`). Shaped by the first kind built with this model; no two kinds share one today.
    return [id, () => buildModel(this.scene, key, painted(this.deps.assets.get(key, 'model'), colors), def?.footprint ?? [1, 1], { resize, standIn: this.standInOf(def) })];
  }

  /** The box a kind of thing is drawn as without its model's file (`BuildOptions.standIn`). */
  private standInOf(def: ObjectDef | undefined): StandIn {
    return standInOf(this.deps.content, def);
  }

  /** Cached template by key; marks it as used by the world being built. */
  private template(key: string, build: () => Promise<ModelTemplate>): Promise<ModelTemplate> {
    const t = this.templates.get(key, build);
    this.templatesInUse.add(key);
    return t.then((template) => {
      for (const mesh of template.meshes) mesh.setEnabled(true);
      return template;
    });
  }

  /** A model file that failed has come: things drawn without it are built again in a moment (`refreshModels`), the ghost and the landscape's plants (`refreshNature`) now. */
  private fileArrived(url: string): void {
    this.ghostFileArrived(url);
    this.birdLife.fileArrived(url);
    this.refreshNature(url);
    const now = performance.now();
    if (!this.filesArrived.size) this.filesSince = now;
    this.filesArrived.add(url);
    if (this.filesTimer) clearTimeout(this.filesTimer);
    this.filesTimer = setTimeout(
      () => {
        this.filesTimer = null;
        void this.refreshModels();
      },
      Math.max(0, Math.min(FILES_SETTLE_MS, this.filesSince + FILES_WAIT_MS - now)),
    );
  }

  /**
   * Builds again what was drawn without the files that have come: the world (once no build is
   * running: one holds the templates it was given; and only while a session's world is shown), the
   * game's street and the town overview's. The old templates (the boxes) are disposed at the end of
   * the next world build that finishes (`buildWorld`).
   */
  private async refreshModels(): Promise<void> {
    while (this.worldBuilds) await this.worldBuilds.catch(() => {});
    if (this.disposed) return;
    const urls = [...this.filesArrived];
    this.filesArrived.clear();
    if (!urls.length) return;
    // Only what the world draws now makes it build again; the rest is just built afresh when asked for.
    let drawn = false;
    for (const url of urls) {
      const keys = this.templates.invalidate(url);
      // The lot layer (the garden's dressing) is kept while the lot is the same: build that again too.
      if (keys.some((k) => this.lotTemplates.has(k))) this.lotKey = '';
      if (keys.some((k) => this.templatesInUse.has(k))) drawn = true;
      // Those the world doesn't draw (put away, hidden) go now.
      this.templates.disposeStale(keys.filter((k) => !this.templatesInUse.has(k)));
      // (The street builds again with the world, its layout key cleared, if it draws one.)
      if (this.street.invalidate(url)) drawn = true;
    }
    this.refreshTown(urls);
    // (No session: back in the menus, `clear` let the world go, and `templatesInUse` with it.)
    if (drawn && this.lastWorld) void this.setWorld(this.lastWorld, this.view).catch(() => {});
  }

  /**
   * The overview, between its builds: what was drawn without `urls` is built again now if the overview
   * is still on screen as last asked for (no newer request, not left: no new ticket is taken, so it
   * never comes back after `hideTown`), else the next time it's shown.
   */
  private refreshTown(urls: string[]): void {
    const overview = this.overview;
    if (!overview) return;
    const ticket = this.townTicket;
    this.townQueue = this.townQueue
      .then(async () => {
        const town = urls.map((url) => overview.invalidate(url)).some(Boolean);
        const world = this.townWorld;
        if (!town || !world || !this.townMode || ticket !== this.townTicket) return;
        this.beginBuild();
        try {
          await this.buildOverview(overview, world);
        } finally {
          this.endBuild();
        }
        this.updateShadowList();
        this.resetSnapshot();
      })
      .catch(() => {});
  }

  /** Disables cached templates the current world doesn't use, and disposes those in a player's colours (their materials are shared and stay). */
  private retireTemplates(keep: Set<string>): void {
    for (const [key, t] of this.templates) {
      if (keep.has(key)) continue;
      void t.then((template) => {
        if (this.templatesInUse.has(key)) return;
        if (!this.paintedTemplates.has(key) || this.templates.peek(key) !== t) {
          for (const mesh of withGlows(template)) mesh.setEnabled(false);
          return;
        }
        this.templates.delete(key);
        this.paintedTemplates.delete(key);
        for (const mesh of withGlows(template)) mesh.dispose(false, false);
      });
    }
    for (const key of [...this.templatesInUse]) if (!keep.has(key)) this.templatesInUse.delete(key);
  }

  /** Meshes of the templates in use (for shadows and freezing). */
  private async templateMeshes(): Promise<Mesh[]> {
    const lists = await Promise.all([...this.templatesInUse].flatMap((k) => this.templates.peek(k) ?? []));
    return lists.flatMap((t) => t.meshes);
  }

  private async buildSims(world: WorldStructure): Promise<void> {
    // Residents carry on as they were (a world message comes with every change in it: a thing
    // bought, sold, broken or mended, someone arriving or leaving ...).
    await this.characters.build(world, 'residents');
    // Sims arrive at the lot's entry, in front of the door. The first time a lot is shown, view it
    // from that side (three-quarter angle) so the cutaway opens the front wall instead of the
    // full-height back wall hiding the arriving household. (Not when following someone there:
    // the camera goes along as the player left it.)
    const v = this.view;
    const key = v ? `${v.x},${v.z},${v.w},${v.d}` : '';
    const framing = !!v && key !== this.framedView;
    if (framing) this.framedView = key;
    if (framing && this.followIndex === null) {
      const entry = world.plots.find((p) => p.x === v.x && p.z === v.z)?.entry;
      const dx = entry ? entry[0] - (v.x + v.w / 2) : 0;
      const dz = entry ? entry[1] - (v.z + v.d / 2) : 0;
      if (Math.hypot(dx, dz) > 1) this.gameCamera.alpha = Math.atan2(dz, dx) - Math.PI * 0.12;
    }
    // Selection marker colour per emotion code (index + 1; 0 = none).
    this.emotionMood = [1, ...this.deps.content.emotions.map((e) => EMOTION_MOOD[e.id] ?? 1)];
    this.characters.setEmotions(this.deps.content.emotions.map((e) => e.id));
  }

  private meshFromArrays(name: string, arrays: MeshArrays, material: Material): Mesh | null {
    if (arrays.positions.length === 0) return null;
    const mesh = new Mesh(name, this.scene);
    const data = new VertexData();
    data.positions = arrays.positions;
    data.normals = arrays.normals;
    data.uvs = arrays.uvs;
    data.indices = arrays.indices;
    data.applyToMesh(mesh, false);
    mesh.material = material;
    mesh.receiveShadows = true;
    this.worldMeshes.push(mesh);
    return mesh;
  }

  private lotMaterial(key: string): Material {
    // Textured PBR surface from the shared library (cached per key, frozen once loaded).
    // Generated geometry has explicit normals; don't depend on triangle winding.
    return this.lib.surface(key, { doubleSided: true });
  }

  /** Builds (once per model key) the preview meshes for buy mode: the thing itself, held up. */
  private loadGhost(model: string, kind: string | undefined, resize?: ObjectResize, colors?: PartColors): void {
    const key = ghostKeyOf(model, kind, resize, colors);
    this.ghostKey = key;
    // Another thing: the old one goes now. The same thing resized keeps its old shape until the new one is built.
    if (model !== this.ghostModel) {
      for (const mesh of this.ghostMeshes) mesh.dispose(false, false);
      this.ghostMeshes = [];
      this.ghostOwn = [];
    }
    this.ghostModel = model;
    this.ghostArgs = [model, kind, resize, colors];
    // Without its file, the box the placed thing would get. A file that failed for the ghost isn't
    // fetched again for each step of a resize or a colour: the box is built straight away, until the
    // file comes (`ghostFileArrived`).
    const def = kind === undefined ? undefined : this.deps.content.object(kind);
    const unfiled = this.ghostUnfiled.get(model);
    const entry = unfiled ? undefined : painted(this.deps.assets.get(model, 'model'), colors);
    void buildModel(this.scene, `ghost:${key}`, entry, resize?.from ?? def?.footprint ?? [1, 1], { resize, standIn: this.standInOf(def) }).then((template) => {
      if (template.failed) this.ghostUnfiled.set(model, template.failed);
      // The ghost doesn't glow.
      for (const glow of template.glows ?? []) glow.mesh.dispose(false, false);
      if (this.ghostKey !== key) {
        for (const mesh of template.meshes) mesh.dispose(false, false);
        return;
      }
      for (const mesh of this.ghostMeshes) mesh.dispose(false, false);
      for (const mesh of template.meshes) {
        mesh.parent = this.ghostRoot;
        mesh.isPickable = false;
        mesh.renderingGroupId = 1;
        // SAFETY: `Matrix.asArray()` with no destination returns a Float32Array; its typings widen it to the array union.
        mesh.thinInstanceSetBuffer('matrix', Matrix.Identity().asArray() as unknown as Float32Array, 16, true);
        mesh.alwaysSelectAsActiveMesh = true;
      }
      this.ghostMeshes = template.meshes;
      this.ghostOwn = template.meshes.map((mesh) => mesh.material);
      this.setGhostTint();
    });
  }

  /** A file that failed for the ghost has come: its models are built with it again, and the thing in hand, if it's one, now (its box stays until then). */
  private ghostFileArrived(url: string): void {
    // (The thing in hand is a box while its model is unfiled.)
    const inHand = !!this.ghostUnfiled.get(this.ghostModel)?.includes(url);
    for (const [model, files] of this.ghostUnfiled) if (files.includes(url)) this.ghostUnfiled.delete(model);
    if (!inHand || !this.ghostArgs) return;
    if (this.ghostShown) this.loadGhost(...this.ghostArgs);
    // Put away: built afresh when next taken in hand.
    else this.ghostKey = '';
  }

  private setGhostTint(): void {
    // Where it fits, the thing itself (and a green arrow); where it can't go, a red see-through copy.
    if (this.ghostValid) this.ghostArrowMaterial.emissiveColor.set(0.55, 1.0, 0.6);
    else this.ghostArrowMaterial.emissiveColor.set(1.0, 0.45, 0.42);
    this.ghostMeshes.forEach((mesh, i) => (mesh.material = this.ghostValid ? this.ghostOwn[i] : this.ghostMaterial));
    this.resetSnapshot();
  }

  // --- per frame ---------------------------------------------------------------------

  /**
   * The selection marker floats above the selected Sim facing the camera: a slow bob, a soft
   * breathing pulse in size and brightness, and no spin. It grows a little with camera distance
   * so it stays readable when zoomed out. Its colour follows their mood. Allocation-free.
   */
  private writeMarker(x: number, headY: number, z: number, emotion: number, now: number): void {
    // (Reduce motion: no bob, no breathing.)
    const t = this.reducedMotion ? 0 : now * 0.001;
    const breath = Math.sin(t * 2.4);
    const cam = this.camera.globalPosition;
    const dist = Math.hypot(cam.x - x, cam.y - headY, cam.z - z);
    const grow = 1 + Math.min(2.2, Math.max(0, dist - 5) * 0.06);
    const s = grow * (1 + breath * 0.05);
    // Float clear of the head: the lower ray's tip stays ~8 cm above it at any size.
    const by = headY + 0.07 + 0.19 * grow + Math.sin(t * 1.5) * 0.03;
    // Camera right / up / forward in world space: the columns of the view matrix's rotation.
    const v = this.camera.getViewMatrix().m;
    // The halo: same orientation, larger, nudged just behind the star (away from the camera).
    const hs = 0.62 * s * (1 + breath * 0.06);
    const out = this.markerMatrix;
    const halo = this.markerHaloMatrix;
    for (let r = 0; r < 3; r++) {
      for (let k = 0; k < 3; k++) {
        out[r * 4 + k] = v[r + k * 4] * s;
        halo[r * 4 + k] = v[r + k * 4] * hs;
      }
      out[r * 4 + 3] = halo[r * 4 + 3] = 0;
    }
    out[12] = x;
    out[13] = by;
    out[14] = z;
    out[15] = halo[15] = 1;
    for (let k = 0; k < 3; k++) halo[12 + k] = out[12 + k] + v[2 + k * 4] * 0.04;

    const slot = this.emotionMood[emotion | 0] ?? 1;
    const c = this.markerPalette[slot] ?? this.markerPalette[0];
    if (!c) return;
    const glow = 1 + breath * 0.12;
    this.markerColor[0] = c[0] * glow;
    this.markerColor[1] = c[1] * glow;
    this.markerColor[2] = c[2] * glow;
    this.markerColor[3] = 1;
    // Outline: the mood colour, much deeper (amber -> warm brown, periwinkle -> ink blue).
    this.markerOutlineColor[0] = c[0] * 0.16;
    this.markerOutlineColor[1] = c[1] * 0.13;
    this.markerOutlineColor[2] = c[2] * 0.12;
    this.markerOutlineColor[3] = 1;
    const h = 0.22 * (1 + breath * 0.25);
    this.markerHaloColor[0] = c[0] * h;
    this.markerHaloColor[1] = c[1] * h;
    this.markerHaloColor[2] = c[2] * h;
    this.markerHaloColor[3] = 1;
  }

  private applyLighting(minute: number): void {
    this.counts.lightingUpdates++;
    const l = lightingAt(minute, LOOK, this.lighting);
    this.sun.direction.copyFrom(l.sunDirection);
    this.sunShadows.moved();
    const town = this.townMode && this.overview;
    // The game's shadow map covers a fixed circle round the lot: its texels keep their size
    // and place while the sun turns and residents walk, so shadows don't change shape at each
    // lighting step (fitted to the casters, it spanned the town at ~10 cm a texel). The
    // overview fits the whole town.
    const r = this.shadowRadius;
    const distance = town ? 40 : r + 40;
    l.sunDirection.scaleToRef(-distance, this.vTmp);
    this.sun.position.copyFrom(town ? this.overview!.centre : this.lotCentre).addInPlace(this.vTmp);
    this.sun.autoUpdateExtends = this.sun.autoCalcShadowZBounds = !!town;
    this.sun.shadowOrthoScale = town ? 0.1 : 0;
    this.shadows.frustumEdgeFalloff = town ? 0 : SHADOW_EDGE_FADE;
    if (!town) {
      this.sun.orthoLeft = this.sun.orthoBottom = -r;
      this.sun.orthoRight = this.sun.orthoTop = r;
      this.sun.shadowMinZ = 1;
      this.sun.shadowMaxZ = distance + r;
    }
    this.sun.intensity = l.sunIntensity;
    this.sun.diffuse.copyFrom(l.sunColor);
    this.sun.specular.copyFrom(l.sunColor);
    this.sky.intensity = l.skyIntensity;
    this.sky.diffuse.copyFrom(l.skyColor);
    this.sky.groundColor.copyFrom(l.groundColor);
    // Shading is linear (image processing converts to sRGB at the end), so key colours are too.
    l.clearColor.toLinearSpaceToRef(this.scene.fogColor);
    this.scene.clearColor.set(this.scene.fogColor.r, this.scene.fogColor.g, this.scene.fogColor.b, 1);
    this.skyDome.update(l);
    this.lib.setEnvironmentIntensity(l.envIntensity);
    if (this.post) {
      setExposure(this.post.imageProcessing.imageProcessingConfiguration, l.exposure * LOOK.exposure);
      // Moonlit nights: blue-grey dark tones, warm lamps and windows (see nightGrade.ts).
      if (this.nightGrade) {
        this.nightGrade.amount = l.night * LOOK.nightGrade;
        this.nightGrade.update();
      }
    }

    // Under the ground (a basement in view, see `applyDig`) daylight doesn't reach.
    const seen = Math.max(l.exposure, 1e-3);
    this.lib.dig.lit.set(0, Math.min(1, BASEMENT_SKY / Math.max(l.skyIntensity * seen, 1e-3)), Math.min(1, BASEMENT_ENV / Math.max(l.envIntensity * seen, 1e-3)), 0);
    this.exposure = seen;
    // Warm lamps inside, glowing windows outside at night; cool daylight in the panes by day.
    this.lampsLevel = l.lamps;
    this.applyRoomLights();
    // Curtains drawn from dusk to dawn; lamp shades and screens glowing.
    this.motion.setLamps(l.lamps);
    this.applyGlow();
    const glass = this.lib.surface('material.window', { vertexColors: true, cutaway: true }) as PBRMaterial;
    // Windows light up only once it is getting dark outside.
    const glow = Math.min(1, Math.max(0, (l.lamps - 0.35) / 0.5));
    this.glowColor.scaleToRef(glow * glow * 1.6, this.emissiveTmp);
    this.lib.setEmissive(glass, this.emissiveTmp);
    const inner = this.lib.surface('material.window.inner', { vertexColors: true, cutaway: true }) as PBRMaterial;
    l.clearColor.toLinearSpaceToRef(this.emissiveTmp);
    this.lib.setEmissive(inner, this.emissiveTmp.scaleInPlace((1 - l.lamps) * 0.9 + 0.05));
    this.street?.setNight(l.lamps, this.glowColor);
    this.overview?.setNight(l.lamps, this.glowColor);
  }

  /** How far the shades' glow and the garden's pools are up (0..1): from dusk as the lamps come up, none in town. */
  private get glowLevel(): number {
    return this.townMode ? 0 : Math.min(1, Math.max(0, (this.lampsLevel - 0.35) / 0.5));
  }

  /**
   * Lamp shades glow as the windows do: from dusk, as the lamps come up, and out with their room's
   * lights where everyone is asleep (`windowGlowOn`); a lamp's shade with the lamp (`lampOn`: off,
   * no glow). Screens are lit by use instead (see `writeScreenGlow`).
   */
  private applyGlow(): void {
    const level = this.glowLevel;
    this.applyGardenPools();
    for (const g of this.glows) {
      if (g.screen) {
        this.writeScreenGlow(g);
        continue;
      }
      let lit = false;
      g.rooms.forEach((room, i) => {
        const lamp = g.lamps[i];
        // In a room without windows a lamp's shade glows by day too.
        const f = (room && this.dimmed.has(room) ? 1 : level) * (lamp >= 0 ? (this.lampOn.get(lamp) ?? 1) : room ? (this.windowGlowOn.get(room) ?? 1) : 1);
        g.colors.fill(f, i * 4, i * 4 + 3);
        g.colors[i * 4 + 3] = 1;
        lit ||= f > 0;
      });
      if (g.mesh.isDisposed()) continue;
      g.mesh.thinInstanceBufferUpdated('color');
      g.mesh.setEnabled(lit);
    }
  }

  /** The garden's pools of lamplight, as bright as its lamps are now (with the shades' glow). */
  private applyGardenPools(): void {
    this.gardenPools?.update(this.glowLevel, (lamp) => (lamp === undefined ? 1 : (this.lampOn.get(lamp) ?? 1)), this.lampColor);
  }

  /**
   * Screens (a TV, a monitor) are lit only while a resident uses them, and flicker a little while
   * they run; the glow mesh's vertex colour is the screen's, its instance colour the brightness.
   */
  private writeScreenGlow(g: { mesh: Mesh; colors: Float32Array; ids: number[] }): void {
    const level = this.glowLevel;
    const t = this.screenTime / 1000;
    let changed = false;
    g.ids.forEach((id, i) => {
      let f = 0;
      if (!this.townMode && this.screenOn.has(id)) {
        const flicker = this.reducedMotion
          ? 1
          : 0.9 + 0.1 * (0.5 + 0.5 * Math.sin(t * 17.3 + id * 1.7)) * (0.6 + 0.4 * Math.sin(t * 41.1 + id * 2.3));
        f = Math.fround((0.55 + 0.45 * level) * flicker);
      }
      if (g.colors[i * 4] === f) return;
      g.colors.fill(f, i * 4, i * 4 + 3);
      g.colors[i * 4 + 3] = 1;
      changed = true;
    });
    if (g.mesh.isDisposed()) return;
    // Uploaded only when a brightness changed: every screen off, or held steady (reduced motion),
    // leaves the buffer as it is.
    if (changed) g.mesh.thinInstanceBufferUpdated('color');
    // Off (nobody watching) it needn't draw at all; toggled only when that changes, so a still
    // picture isn't re-recorded every frame.
    const on = g.ids.some((id) => this.screenOn.has(id));
    if (g.mesh.isEnabled() !== on) g.mesh.setEnabled(on);
  }

  /** Which screens are on this frame: the objects residents are using (not just walking to). */
  private animateScreens(frame: FrameState): void {
    if (!this.glows.some((g) => g.screen)) return;
    const { curr, layout } = frame;
    const k = layout.sim;
    const on = new Set<number>();
    const count = Math.min(curr[layout.header.simCount], MAX_SIMS);
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      if ((curr[o + k.action] | 0) < 0) continue;
      const id = curr[o + k.object] | 0;
      if (id >= 0) on.add(id);
    }
    this.screenOn = on;
    this.screenTime = frame.now;
    for (const g of this.glows) if (g.screen) this.writeScreenGlow(g);
  }

  /** The interior lights at the night level of the last lighting update, out in rooms where everyone is asleep. */
  private applyRoomLights(): void {
    this.roomLights.forEach((light, i) => {
      // The overview has no interiors: the game house's lamps stay off behind it.
      const use = this.townMode ? null : (this.roomLightUse[i] ?? null);
      // Down in the basement and in rooms without windows they're on by day too, as bright as at
      // night (by day the exposure is lower).
      const dark = this.storey < 0 || (!!use && this.dimmed.has(use.room));
      const lit = (dark ? NIGHT_EXPOSURE / this.exposure : this.lampsLevel) * (this.roomLightOn[i] ?? 1);
      if (!use) {
        light.intensity = 0;
      } else if (use.kind === 'lamp') {
        // A lamp the household bought: a warm pool around it.
        light.intensity = lit * LAMP_POWER * (use.power ?? 1);
        light.range = use.range ?? 4.5;
      } else {
        // A room without a lamp: only a dim glow from the ceiling.
        light.intensity = lit * FILL_POWER * (4 + Math.min(use.area, 30) * 0.35);
        light.range = 4 + Math.sqrt(use.area) * 1.4;
      }
      // A dark one (all of them by day) adds nothing, but the cluster would still sort it into the
      // screen's tiles by its range for every pixel to loop over: next to no range keeps it out.
      if (light.intensity === 0) light.range = DARK_LAMP_RANGE;
      light.diffuse.copyFrom(this.lampColor);
    });
  }

  /**
   * Behind the overview, lets the menus' town draw a frame once the world build has run for a
   * slice: built in one go, it held the main thread for most of a second. In play (and when
   * nothing is waiting to draw) the build runs straight through.
   */
  private async pace(): Promise<void> {
    // A world or the overview is being built (behind a menu), or plants in play: let the frames it runs behind go on.
    if ((!this.worldBuilding && this.builds === 0 && this.refreshes === 0) || performance.now() - this.sliceStart < BUILD_SLICE_MS) return;
    await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
    this.sliceStart = performance.now();
  }

  /** Draws the sky and cloud band once through a tiny mirror, compiling their clip-plane variants. */
  private async warmSkyMirror(): Promise<void> {
    if (this.skyMirrorWarmed) return;
    this.skyMirrorWarmed = true;
    const meshes = [this.skyDome.mesh, this.skyDome.band].filter((m): m is Mesh => !!m);
    if (!meshes.length) return;
    const texture = new MirrorTexture('warmMirror', { ratio: 0.03 }, this.scene, false, Constants.TEXTURETYPE_HALF_FLOAT);
    texture.mirrorPlane = new Plane(0, 1, 0, 0);
    texture.renderList = meshes;
    texture.activeCamera = this.camera;
    texture.refreshRate = 0;
    try {
      texture.render();
    } finally {
      texture.dispose();
    }
  }

  /**
   * Behind the overview, prepares the scene's shaders a few at a time, as the readiness check
   * before the world is shown would (it did them all at once). Materials whose textures are
   * still arriving wait for them, so this goes round again until all are ready (or `until`).
   */
  private async warmShaders(until: number): Promise<void> {
    while (this.townMode && !(await this.warmShadersOnce()) && performance.now() < until) {
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  /** One round of `warmShaders`: whether every shader is ready. */
  private async warmShadersOnce(): Promise<boolean> {
    const engine = this.engine;
    const instancing = !!engine.getCaps().instancedArrays;
    let ready = true;
    // Every mesh, hidden ones too (templates built ahead), as `Scene.isReady` checks them.
    for (const mesh of [...this.scene.meshes]) {
      if (!(mesh instanceof Mesh) || mesh.isDisposed() || !mesh.subMeshes?.length) continue;
      const instanced = mesh.hasThinInstances || (instancing && mesh.instances.length > 0);
      const shadows = mesh.lightSources
        .flatMap((l) => [...(l.getShadowGenerators()?.values() ?? [])])
        .filter((g) => g.getShadowMap()?.renderList?.includes(mesh) ?? true);
      for (const sub of mesh.subMeshes) {
        const mat = sub.getMaterial();
        if (!mat) continue;
        // (Not `mesh.isReady`: it stops at the first material still compiling, which is nearly always the first.)
        // Effects are kept per render pass: the camera's, as the check is made for it.
        const pass = engine.currentRenderPassId;
        engine.currentRenderPassId = this.scene.activeCamera?.renderPassId ?? pass;
        const t0 = performance.now();
        ready = (mat._storeEffectOnSubMeshes ? mat.isReadyForSubMesh(mesh, sub, instanced) : mat.isReady(mesh, instanced)) && ready;
        const ms = performance.now() - t0;
        if (ms > 20) perfRecord(`render: shader ${mat.name}`, t0, ms, mesh.name);
        for (const g of shadows) {
          for (const id of g.getShadowMap()?.renderPassIds ?? [pass]) {
            engine.currentRenderPassId = id;
            ready = g.isReady(sub, instanced, mat.needAlphaBlendingForMesh(mesh)) && ready;
          }
        }
        engine.currentRenderPassId = pass;
        await this.pace();
      }
    }
    return ready;
  }

  /** Something on screen changed: a WebGPU recording is made again, and held reflections are drawn again. */
  private resetSnapshot(): void {
    if (this.webgpu?.snapshotRendering) this.snapshotDue ??= 'reset';
    this.mirrors?.changed();
  }

  /**
   * Starts or restarts a snapshot recording, right before a frame draws. Babylon records from the
   * render pass after the call: one made while a frame draws (the materials' refreeze and the
   * recording's own start run after the scene's render targets) records only the canvas pass, and
   * the replays then draw nothing but the clear colour until the next reset.
   */
  private applySnapshotDue(): void {
    const due = this.snapshotDue;
    const engine = this.webgpu;
    this.snapshotDue = null;
    if (!engine || !due) return;
    if (due === 'record') engine.snapshotRendering = true;
    else if (engine.snapshotRendering) engine.snapshotRenderingReset();
  }

  /** A world or overview build starts: no snapshot recording until every build is done. */
  private beginBuild(): void {
    this.builds++;
    this.snapshotGeneration++;
    this.snapshotDue = null;
    if (this.webgpu) this.webgpu.snapshotRendering = false;
  }

  private endBuild(): void {
    this.builds = Math.max(0, this.builds - 1);
    if (this.builds === 0) this.scheduleSnapshot();
  }

  /** Stops replaying (the scene changed a lot) and records again once it has settled. */
  private pauseSnapshot(): void {
    this.snapshotGeneration++;
    this.snapshotDue = null;
    if (this.webgpu) this.webgpu.snapshotRendering = false;
    if (this.builds === 0) this.scheduleSnapshot();
  }

  /**
   * Starts snapshot recording once the scene and its post-process effects have settled: only for
   * the town overview behind the menus. In play, replays don't pick up the Sims' pose texture
   * updates (Sims froze between re-recordings) and each re-record cost a 50-70 ms hitch, while
   * drawing live costs only ~0.2 ms more per frame.
   */
  private scheduleSnapshot(): void {
    const engine = this.webgpu;
    const generation = ++this.snapshotGeneration;
    if (!engine || !this.quality.snapshotRendering || !this.townMode) return;
    // A newer build (or pause) may have started by the time this runs; it records once it is done.
    const current = () => generation === this.snapshotGeneration && this.builds === 0 && this.townMode;
    this.scene.executeWhenReady(() => {
      let left = 10;
      const observer = this.scene.onAfterRenderObservable.add(() => {
        if (--left > 0) return;
        this.scene.onAfterRenderObservable.remove(observer);
        if (!current()) return;
        engine.snapshotRenderingMode = Constants.SNAPSHOTRENDERING_STANDARD;
        // (This runs as the frame ends: the recording starts with the next one.)
        this.snapshotDue = 'record';
        // Record once more after the first replays: the first recording can capture one-off
        // start-up state (late material freezes) and then stop following the camera.
        let again = 15;
        const second = this.scene.onAfterRenderObservable.add(() => {
          if (--again > 0) return;
          this.scene.onAfterRenderObservable.remove(second);
          if (current()) this.resetSnapshot();
        });
      });
    });
  }

  private watchDeviceLoss(engine: WebGPUEngine): void {
    // Babylon's `device.lost.then(...)` closes over the engine, so a pending promise keeps a disposed
    // renderer alive. Our handler only holds a weak reference; the device can still be watched.
    // SAFETY: WebGPUEngine keeps its GPUDevice in the private `_device`.
    const device = (engine as unknown as { _device?: { lost?: Promise<unknown> } })._device;
    const self = new WeakRef(this);
    void device?.lost?.then(() => {
      const renderer = self.deref();
      if (renderer && !renderer.disposed) renderer.onFailure?.('device');
    });
  }

  private readonly onResize = () => this.applyResolution();

  /**
   * Babylon times each frame when it starts to run (`performance.now()`), which wanders by however
   * long the browser took to get to it. It times them by the refresh they're drawn for instead (the
   * pacer's `frameTime`): the frame time (`getDeltaTime`: camera glides and inertia, clouds, wind,
   * fades) is then whole refreshes, as the frames show.
   */
  private timeFramesByRefresh(): void {
    // SAFETY: Babylon's private frame timer lives on the engine as `_measureFps`, `_performanceMonitor`, `_fps` and `_deltaTime`.
    const engine = this.engine as unknown as { _measureFps?: () => void; _performanceMonitor?: PerformanceMonitor; _fps: number; _deltaTime: number };
    const monitor = engine._performanceMonitor;
    if (typeof engine._measureFps !== 'function' || !monitor) return;
    engine._measureFps = () => {
      monitor.sampleFrame(this.pacer.frameTime || undefined);
      engine._fps = monitor.averageFPS;
      engine._deltaTime = monitor.instantaneousFrameTime || 0;
    };
  }

  /** The frame rate cap, and in the menus at most `MENU_FPS` (not while the camera flies into the game). */
  private applyPacing(): void {
    const cap = this.options.fpsCap;
    const menu = this.townMode && !this.flyingIn;
    this.pacer.setCap(menu ? (cap > 0 ? Math.min(cap, MENU_FPS) : MENU_FPS) : cap);
  }

  /**
   * Sizes the drawing to the canvas, the device pixel ratio and the resolution setting
   * (`resolution.ts`): on start, on settings changes, when the window resizes and when the pixel
   * ratio changes (zoom, another screen).
   */
  private applyResolution(): void {
    const engine = this.engine;
    const dpr = window.devicePixelRatio || 1;
    const scale = renderScale(this.canvas.clientWidth * dpr, this.canvas.clientHeight * dpr, this.options.resolution);
    const level = 1 / (dpr * scale);
    // (Setting the level resizes; the same level still needs a resize for a new canvas size.)
    if (level !== engine.getHardwareScalingLevel()) engine.setHardwareScalingLevel(level);
    else engine.resize();
  }

  /** Image-based lighting from a prefiltered environment (manifest key `environment.sky`). */
  private async setupEnvironment(): Promise<void> {
    const entry = this.deps.assets.get('environment.sky', 'image');
    if (entry) await this.lib.loadEnvironment(entry.url);
  }
}

/** Smoothstep: eases 0..1 in and out. */
function ease(t: number): number {
  return t * t * (3 - 2 * t);
}

/**
 * Whether someone in `movers` (`MOVER` floats each) goes through the 1 m doorway whose middle is
 * `centre`, in a wall or fence along x (`axis` 'h') or z: their way crosses it before `DOOR_AHEAD`
 * metres on, or they're in it. Walking past, or up to it and away, isn't. One that stands `open`
 * also stays so while anyone on the move is still within `DOOR_CLEAR` of it (the leaf would
 * sweep through them shutting). Returns the way they go across (+1 towards +z of an 'h' edge or
 * +x of a 'v' one, -1 the other), 0 for nobody.
 */
function goingThrough(movers: readonly number[], axis: 'h' | 'v', centre: readonly [number, number], open: boolean): number {
  // In the doorway's own terms: `a` along the edge from its middle, `n` across it.
  const h = axis === 'h';
  const [ca, cn] = h ? centre : [centre[1], centre[0]];
  const [ia, iN] = h ? [0, 1] : [1, 0];
  for (let i = 0; i < movers.length; i += MOVER) {
    const pa = movers[i + ia] - ca;
    const pn = movers[i + iN] - cn;
    let wa = movers[i + 2 + ia] - ca;
    let wn = movers[i + 2 + iN] - cn;
    const qa = movers[i + 4 + ia] - ca;
    const qn = movers[i + 4 + iN] - cn;
    if ((Math.abs(pa) <= 0.5 && Math.abs(pn) < DOORWAY) || (open && pa * pa + pn * pn < DOOR_CLEAR * DOOR_CLEAR)) return qn >= pn ? 1 : -1;
    // A waypoint further than the look ahead: where it looks to is on the way there.
    if (Math.hypot(wa - pa, wn - pn) >= DOOR_AHEAD) {
      wa = qa;
      wn = qn;
    }
    const way = crosses(pa, pn, wa, wn) || crosses(wa, wn, qa, qn);
    if (way) return way;
  }
  return 0;
}

/** Whether a step from `s` to `e` (along, across) crosses a 1 m doorway at the origin: its way across (±1), or 0. */
function crosses(sa: number, sn: number, ea: number, en: number): number {
  if (sn * en > 0 || sn === en) return 0;
  return Math.abs(sa + ((ea - sa) * sn) / (sn - en)) <= 0.5 ? Math.sign(en - sn) : 0;
}

/**
 * Sets exposure without the configuration's change notice. Image processing runs as a post
 * process, where exposure is a uniform read each frame, but the notice marks every material dirty
 * (each walking every mesh: tens of ms per lighting step) for nothing; nothing else here listens
 * for it (bloom, which would, is off). The pass has exposure only while it isn't 1 (a define), so
 * crossing 1 still goes through the setter, as does everything should Babylon rename the field.
 */
function setExposure(config: ImageProcessingConfiguration, value: number): void {
  // SAFETY: ImageProcessingConfiguration stores exposure in its private `_exposure` backing field.
  const fields = config as unknown as { _exposure?: number };
  if (typeof fields._exposure === 'number' && (fields._exposure === 1) === (value === 1)) fields._exposure = value;
  else config.exposure = value;
}

/** Bytes per pixel of a colour target of Babylon texture `type` and `format` (benchmark reports; 0: depth only). */
function textureBytes(type: number = Constants.TEXTURETYPE_UNSIGNED_BYTE, format: number = Constants.TEXTUREFORMAT_RGBA): number {
  if (format >= Constants.TEXTUREFORMAT_DEPTH24_STENCIL8 && format <= Constants.TEXTUREFORMAT_STENCIL8) return 0;
  const channels: Record<number, number> = { [Constants.TEXTUREFORMAT_ALPHA]: 1, [Constants.TEXTUREFORMAT_R]: 1, [Constants.TEXTUREFORMAT_RG]: 2 };
  const size: Record<number, number> = { [Constants.TEXTURETYPE_FLOAT]: 4, [Constants.TEXTURETYPE_HALF_FLOAT]: 2 };
  return (channels[format] ?? 4) * (size[type] ?? 1);
}

/** Bytes per pixel of a render target's depth (and stencil); 0 without one. */
function depthBytes(rtw: RenderTargetWrapper): number {
  const texture = rtw._depthStencilTexture ?? (rtw.texture && textureBytes(rtw.texture.type, rtw.texture.format) === 0 ? rtw.texture : null);
  if (!texture) return rtw._generateDepthBuffer || rtw._generateStencilBuffer ? 4 : 0;
  return texture.format === Constants.TEXTUREFORMAT_DEPTH16 ? 2 : texture.format === Constants.TEXTUREFORMAT_DEPTH32FLOAT_STENCIL8 ? 8 : 4;
}

/** Copies an orbit camera's angles, radius and target. */
function copyPose(from: ArcRotateCamera, to: ArcRotateCamera): void {
  to.alpha = from.alpha;
  to.beta = from.beta;
  to.radius = from.radius;
  to.target.copyFrom(from.target);
}

/** Buy mode's ghost's key: its model, kind (the box it gets without its file), size and colours. */
function ghostKeyOf(model: string, kind: string | undefined, resize?: ObjectResize, colors?: PartColors): string {
  return `${model}|${kind ?? ''}${resizeKey(resize)}${colorsKey(colors)}`;
}

/** Ray vs axis-aligned box (slab test). Returns distance or Infinity. */
function rayBox(o: Vector3, d: Vector3, x0: number, y0: number, z0: number, x1: number, y1: number, z1: number): number {
  let tMin = 0;
  let tMax = Infinity;
  const axes: [number, number, number, number][] = [
    [o.x, d.x, x0, x1],
    [o.y, d.y, y0, y1],
    [o.z, d.z, z0, z1],
  ];
  for (const [origin, dir, lo, hi] of axes) {
    if (Math.abs(dir) < 1e-9) {
      if (origin < lo || origin > hi) return Infinity;
      continue;
    }
    let t0 = (lo - origin) / dir;
    let t1 = (hi - origin) / dir;
    if (t0 > t1) [t0, t1] = [t1, t0];
    tMin = Math.max(tMin, t0);
    tMax = Math.min(tMax, t1);
    if (tMin > tMax) return Infinity;
  }
  return tMin;
}
