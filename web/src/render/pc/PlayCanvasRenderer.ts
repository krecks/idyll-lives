/**
 * The game's renderer on PlayCanvas (`Renderer`, `render/types.ts`).
 *
 * Built up stage by stage from the Babylon renderer it replaces (`render/babylon/BabylonRenderer.ts`,
 * kept as the reference until the port is done; see the plan in docs). What this one does so far:
 * the device (WebGPU, else WebGL2), frames paced in whole display refreshes, the game camera (the
 * player's orbit input, the director's shots) and the menus' overview camera, the time of day's
 * sun, sky fill, environment lighting and fog, picking (`picking.ts`: the ground, objects and items,
 * analytic as Babylon's), projecting points onto the screen, and the stats. The viewed lot: its house on every storey (`house.ts`: walls, floors,
 * roofs, doors and windows, the cutaway and the wall modes, the storey in view, a basement with the
 * ground dug away over it, rooms without windows kept out of daylight, doors and gates swinging for
 * residents going through), its fences (`fences.ts`), roads and paths, and the street round it
 * (`street.ts`: the neighbours' houses, sidewalks, street furniture, gardens). Its residents
 * (`characters.ts`): posed each frame, with their contact shadows, the chairs they pull up to
 * tables, their sun shadows, the selection ring, speech, and the camera following one. The rest is
 * stubbed where the port reaches it.
 *
 * Game space is the Babylon renderer's left-handed one: everything in the world hangs under a root
 * mirrored in z (`space.ts`), and only cameras, lights and rays are converted.
 */

import * as pc from './engine';
import { BuildPreviews, mergeSurfaces } from './buildPreviews'; // (Stage 8C.)
import { GameCamera } from './camera';
import { Characters, MAX_CHARACTERS } from './characters';
import { createApp, createDevice } from './device';
import { buildFences, swingGate, type Fences, type GateLeaf } from './fences';
import { gardenPool, GardenPools } from './gardenPools';
import { HoverTile, ResizeHandles } from './handles';
import { HouseBuilder, type HouseDoor } from './house';
import { Instances } from './instancing';
import { Lamps } from './lamps';
import { contactBlob, LotContents } from './lotContents';
import { ENV_UNITS, SKY_UNITS, SUN_UNITS } from './lightUnits';
import { MaterialLibrary, SUN_LIGHT_MASK } from './materials';
import { drawMesh, meshFrom } from './meshes';
import { buildModel, onModelFileArrived, type ModelTemplate } from './models';
import { natureDecor, windStill } from './nature';
import { Placing } from './placing'; // (Stage 8B.)
import { Picker } from './picking';
import { gameLook, gamePostOptions, PostProcessing } from './post';
import { diagViewActive } from './shaders/diagView';
import { shadingFromSettings, softLookFromURL } from './shaders/softLook';
import { drawRoads } from './roads';
import { createWorldRoot, toEngine } from './space';
import { Street } from './street';
import { SelectionMarker } from './selectionMarker';
import { SunShadows } from './sunShadows';
import { TemplateCache } from './templateCache';
import { Thumbnails } from './thumbnail'; // (Stage 8D.)
import { benchDiag } from '../../debug/benchFlags';
import { LOOK } from '../look';
import type { RenderQuality } from '../quality';
import { GameCameraRig } from '../shared/cameraRig';
import { DOOR_SWING, ease, goingThrough } from '../shared/doors';
import { createLighting, lightingAt, type RGB } from '../shared/environment';
import { GAME_RATE } from '../shared/gameSpeed';
import { Geo } from '../shared/geometry';
import { houseObjectKey, placeLights, WALL_HEIGHT, WALL_STUB, type HouseSurfaces, type RoomLight, type RoomTiles } from '../shared/house';
import { compose, quatYawPitchRoll, toLinear, type Quat } from '../shared/math';
import { OrbitControls } from '../shared/orbit';
import { ModelRefresh } from '../shared/modelRefresh'; // (Stage 8B.)
import { OverviewCamera } from '../shared/overviewCamera';
import { FramePacer } from '../shared/pacing';
import { standInOf } from '../shared/modelEntry';
import { renderScale, watchPixelRatio } from '../shared/resolution';
import { fieldLayers } from '../shared/street';
import { darkTiles, windowlessRooms, type DarkTiles, type Windowless } from '../shared/windowless';
import type { FrameState } from '../../core/bridge';
import { Pose } from '../../core/snapshot';
import { lowestStorey, storeyOfRow, storeyRow, storeyShown, type FenceEdge, type ItemPlacement, type ObjectPlacement, type StoreyLayout, type WorldStructure } from '../../core/protocol';
import type { VisemeTrack } from '../../voice/visemes';
import type {
  BuildEffect,
  CameraPose,
  EdgePreview,
  FrameCounts,
  FrameProbe,
  GameShot,
  LiveRenderOptions,
  LotHighlight,
  ObjectBackgroundClips,
  ObjectClipInfo,
  PaintPreviewFace,
  PickResult,
  PlacementGhost,
  RenderFailure,
  RenderPassInfo,
  Renderer,
  RendererDebugHooks,
  RendererDeps,
  RenderStats,
  ResizeHandle,
  RoomOverlayTile,
  SeenBird,
  TownShot,
  ViewRect,
  Shading,
  WallMode,
} from '../types';

/** The game camera's zoom and tilt limits (as the Babylon renderer's). */
const RADIUS_LIMITS = [6, 95] as const;
const BETA_LIMITS = [0.3, 1.35] as const;
/** Its lens: vertical field of view (a slightly long lens flattens the perspective: the dollhouse look), near and far (m). */
const LENS = { fov: 0.7, near: 0.3, far: 1600 };
/** Frames per second at most in the menus. */
const MENU_FPS = 30;
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
/** Metres the sun's shadow square reaches past the viewed lot. */
const SHADOW_MARGIN = 10;
/** How far the road (and garden paths) is drawn beyond the viewed lot: the whole street. */
const STREET_MARGIN = 500;
/** The basement's pit (and the ground dug away over it) reaches this far past the plot (m), clear of walls on its edge. */
const PIT_MARGIN = 0.3;
/** Camera distance below which the viewed lot's roof is hidden even with walls up. */
const ROOF_MIN_DISTANCE = 15;
/** Main-thread time (ms) a world build runs before the frames it runs behind get one (`pace`). */
const BUILD_SLICE_MS = 8;
/**
 * A basement in view gets no sun, only this much of the sky fill and the environment (as seen,
 * intensity times exposure: the same by day and by night).
 */
const BASEMENT_SKY = 0.12;
const BASEMENT_ENV = 0.12;

/** Residents this far (m) outside the viewed lot are still drawn (walking up to it). */
const SIM_VIEW_MARGIN = 2;

/** A directional light's way before it's turned (PlayCanvas shines it along its entity's -y). */
const DOWN = new pc.Vec3(0, -1, 0);

/** No neighbours' houses (to look through). */
const NO_HOUSES: readonly (readonly [number, number, number, number])[] = [];

/**
 * Everything an object is drawn from, but its open state and growth (drawn without a build: see
 * `patchable`). (Wear, quality and what it's worth don't change the model.)
 */
function objectKey(o: ObjectPlacement): string {
  return JSON.stringify([o.id, o.def, o.x, o.z, o.rot, o.turn, o.offset, o.size, o.tall, o.lift, o.w, o.d, o.colors]);
}

/** The same part of the town in view (the messages make a new rect each time). */
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

/** A hex colour (sRGB) in linear light, as Babylon's `toLinearSpace`. */
function linearHex(hex: string): RGB {
  const v = parseInt(hex.slice(1), 16);
  return { r: toLinear(((v >> 16) & 255) / 255), g: toLinear(((v >> 8) & 255) / 255), b: toLinear((v & 255) / 255) };
}

/** A Babylon linear colour (the look's keys are used as linear there) as PlayCanvas takes it (sRGB, linearised when lit). */
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
  private camera!: GameCamera;
  /** (Stage 6A.) Babylon's image processing, FXAA and night grade on the camera (`post.ts`). */
  private post: PostProcessing | null = null;
  private sun!: pc.Entity;
  private sunShadows!: SunShadows;
  private disposed = false;

  private readonly diag = benchDiag();
  private readonly debug = new URLSearchParams(location.search).has('debug');
  /** `?hour=`: the light held at that hour. */
  private readonly fixedHour: number | null;

  /** The player's camera input and the game camera's pose (the director's rig drives it too). */
  private readonly orbit = new OrbitControls({ alpha: -Math.PI * 0.62, beta: 0.86, radius: 24 }, { radius: RADIUS_LIMITS, beta: BETA_LIMITS });
  private detachOrbit: (() => void) | null = null;
  private readonly gameRig: GameCameraRig;
  /** The overview camera's pose (`OverviewCamera` writes it; the camera takes it each town frame). */
  private readonly townPose = { alpha: 0, beta: 0.9, radius: 60, target: { x: 0, y: 0, z: 0 }, targetScreenOffset: { x: 0, y: 0 }, fov: LENS.fov };
  private readonly townRig = new OverviewCamera(this.townPose);

  private readonly pacer = new FramePacer();
  private options: LiveRenderOptions = { resolution: 'auto', cameraSensitivity: 1, fpsCap: 0 };
  private active = false;
  private covered = false;
  private coveredFrames = 0;
  private readonly frameWaiters: { left: number; resolve: () => void }[] = [];
  private onFrame: ((now: number) => void) | null = null;
  private onView: (() => void) | null = null;
  private townListener: (() => void) | null = null;
  private lastTime = 0;
  private frameDt = 1000 / 60;
  private failedFrames = 0;

  private townMode = false;
  private flyingIn = false;
  private townMinute = 12 * 60;
  private townMinuteGoal = 12 * 60;
  private lastMinute = 12 * 60;
  private lastLightMinute = -1;
  private lastLightAt = 0;
  private readonly lighting = createLighting();

  private ground: pc.Entity | null = null;
  private readonly materials = new Map<string, pc.StandardMaterial>();
  /** The surfaces, the look and the environment lighting (`materials.ts`). */
  private lib!: MaterialLibrary;
  /** The shading the materials are drawn in (`shaders/softLook.ts`); the setting's at start. */
  private shading: Shading = shadingFromSettings();
  private wallMode: WallMode = 'down';
  /** The cutaway's look bits last given to the walls (-1: none yet). */
  private cutLook = -1;

  // The world as built (`setWorld`).
  /** The world last asked for (built again when the storey in view changes), the one built, and the one waiting. */
  private lastWorld: WorldStructure | null = null;
  private builtWorld: WorldStructure | null = null;
  private builtStorey = 0;
  private nextWorld: { world: WorldStructure; view: ViewRect | null } | null = null;
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
  /** Build mode's helpers (stage 8) work in lot rows: this lifts them to the storey in view. */
  private storeyLayer!: pc.Entity;
  /** (Stage 8C: Build mode's previews and overlays on it, the tile grid, `buildPreviews.ts`.) */
  private previews!: BuildPreviews;
  /** The viewed lot's house, the street's houses (`house.ts`), and the street (`street.ts`). */
  private house!: HouseBuilder;
  private street!: Street;
  /** What grows and stands on the lot: the landscape and sky, objects and their clips, items (stage 5, `lotContents.ts`). */
  private lot!: LotContents;
  /** The viewed house's interior lights (Stage 6C, `lamps.ts`). */
  private lamps!: Lamps;
  /** (Stage 8B: buy mode's thing in hand, the object picked up, the effects, `placing.ts`; the rebuilds files that came set off, gathered, `shared/modelRefresh.ts`.) */
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
  /** The simulation's plain walls, when the house builder can't read the layout (walls up, and the low ones). */
  private walls: pc.Entity | null = null;
  private wallsLow: pc.Entity | null = null;
  /** Rooms of the house (where its lamps' and fills' lights go). */
  private lotRooms: RoomTiles | null = null;
  /** Openings in the ground floor over stairs down to the basement (ground rows), seen from the ground up. */
  private stairwells: ViewRect[] = [];
  /** The house's garden trees and shrubs (`placeDecor`): a draw per model mesh, hidden while its basement is in view. */
  private decorTemplates!: TemplateCache<ModelTemplate>;
  private readonly decor = new Map<string, { template: ModelTemplate; draws: Instances[] }>();
  /** The viewed lot's door leaves that swing, each with how far open it stands (1 open, as drawn; -1 not placed yet). */
  private doors: { leaf: HouseDoor; open: number }[] = [];
  /** Gates in the fences' meshes, each with how far open it stands (0 shut, as drawn) and the side it opens to. */
  private gates: { fences: Fences; leaf: GateLeaf; open: number; towards: number }[] = [];
  /** Pools of lamplight on the lawn under the garden's lamps. */
  private gardenPools: GardenPools | null = null;
  private readonly poolLampOn = (lamp: number | undefined) => this.lamps.lampOn(lamp); // (Stage 6C: as its lamp fades, `lamps.ts`.)
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
  private simShadow!: Instances;
  private readonly simShadowMatrices = new Float32Array(MAX_CHARACTERS * 16);
  private simCount = 0;
  private marker!: SelectionMarker;
  private selectedSim: number | null = null;
  private lastFrame: FrameState | null = null;
  private readonly markerHead = { x: 0, y: 0, z: 0 };
  private followIndex: number | null = null;
  private readonly followHead = { x: 0, y: 0, z: 0 };
  private framedView = '';
  private readonly simInView = (x: number, z: number) => !this.diag.has('nosims') && this.inView(x, z, SIM_VIEW_MARGIN);

  private readonly cssSize = { width: 1, height: 1 };
  private sizeWatch: ResizeObserver | null = null;
  private unwatchPixelRatio: (() => void) | null = null;
  private renderScaleNow = 1;

  /** Picking (`picking.ts`: the last pick's ray, for `pickPlane` & co.), the resize arrows and the hover tile (`handles.ts`). */
  private picker!: Picker;
  private handles!: ResizeHandles;
  private hoverTile!: HoverTile;
  private readonly eyeTmp = { x: 0, y: 0, z: 0 };
  private readonly vTmp = new pc.Vec3();
  private readonly vTmp2 = new pc.Vec3();
  private readonly qTmp = new pc.Quat();
  private readonly colorTmp = new pc.Color();
  /** `?lightmix=sun,sky,env` (dev server): each part of the daylight scaled, to compare them one by one with the Babylon renderer's. */
  private readonly lightMix = import.meta.env.DEV ? lightMixFromURL() : null;

  private probe: FrameProbe | null = null;
  private readonly counts: FrameCounts = { drawCalls: 0, triangles: 0, shadowDrawCalls: 0, shadowTriangles: 0, mirrorDrawCalls: 0, mirrorTriangles: 0, lightingUpdates: 0, gpuFrames: 0, gpuMs: 0, residentsMs: 0 };
  private frameMs = 1000 / 60;
  private cpuMs = 0;
  private drawCalls = 0;
  private readonly gpuWanted = new Set<'overlay' | 'bench'>();

  constructor(
    private readonly deps: RendererDeps,
    private readonly quality: RenderQuality,
  ) {
    const params = new URLSearchParams(location.search);
    const hour = Number(params.get('hour'));
    this.fixedHour = Number.isFinite(hour) && params.has('hour') ? hour : null;
    this.gameRig = new GameCameraRig(this.orbit, { radius: RADIUS_LIMITS, beta: BETA_LIMITS }, (i, out) => this.simHead(i, out), () => this.orbit.stopInertia());
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
    this.app = createApp(canvas, this.device);
    // A GPU reset (driver update, a laptop switching GPUs, a crashed GPU process): what was on the
    // GPU is gone, and the game's own buffers can't be rebuilt in place.
    this.device.on('devicelost', () => this.onFailure?.('device'));
    this.applyResolution();
    this.unwatchPixelRatio = watchPixelRatio(this.onResize);
    window.addEventListener('resize', this.onResize);

    this.lib = MaterialLibrary.for(this.app).setAssets(this.deps.assets);
    this.lib.anisotropy = this.quality.anisotropy;
    this.lib.setShading(this.shading);

    const scene = this.app.scene;
    // The sky is the game's own (its dome, later): the environment only lights.
    scene.ambientLight = new pc.Color(0, 0, 0);
    // Distance fog in the horizon colour makes the hills fade into the sky (aerial haze).
    scene.fog.type = pc.FOG_EXP2;
    scene.fog.density = LOOK.fog;

    this.root = createWorldRoot(this.app);
    this.camera = new GameCamera(this.app.root, LENS);
    const cam = this.camera.camera;
    // (Not PlayCanvas's skybox: the environment atlas would draw as one.)
    cam.layers = cam.layers.filter((id) => id !== pc.LAYERID_SKYBOX);
    // (Stage 6A.) The picture after the light: multisampling or FXAA, Babylon's image processing
    // and the night grade (`post.ts`; the diag views and `?diag=linear` without them).
    this.post = new PostProcessing(this.app, cam, gamePostOptions(this.quality.msaaSamples, this.diag, diagViewActive));
    this.post.setLook(gameLook(LOOK));

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
    this.sunShadows = new SunShadows(this.app, this.sun, { size: this.quality.shadowMapSize, rate: this.quality.shadowRate });
    // (Stage 5, lotContents.ts: the landscape, sky, objects and items.)
    this.lot = new LotContents(this.app, this.root, this.sun, this.camera.entity, this.deps, this.quality, { storey: () => this.storey, storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), inView: (x, z, m) => this.inView(x, z, m) }, { pace: () => this.pace(), shadowsMoved: (urgent) => this.sunShadows.moved(urgent), landscapeChanged: () => this.lamps.exclude(this.lot.landscapeDraws()) /* (Stage 6C: lamps.ts) */ });

    // The residents (`characters.ts`), on moving seats with them, and their contact shadows.
    this.characters = new Characters(this.app, this.root, this.deps.assets, this.lib, this.deps.content, { refreshProps: true, castsShadows: this.quality.characterShadows, camera: () => this.camera.camera });
    this.characters.setRide((id) => this.lot.ride(id));
    this.characters.setLoop((id) => this.lot.loopPhase(id));
    this.characters.setSun(() => this.lighting.sunDirection);
    const blob = contactBlob(this.app);
    this.simShadow = new Instances(this.device, this.root, blob.mesh, blob.material, { name: 'simShadows', castShadows: false, receiveShadows: false, capacity: MAX_CHARACTERS, cull: false });
    this.marker = new SelectionMarker(this.app, this.root, this.deps.assets);

    // The viewed lot's house and the street round it (`house.ts`, `street.ts`).
    this.house = new HouseBuilder(this.app);
    const c = this.deps.content;
    this.house.setLooks({ coverings: c.wallCoverings, floors: c.floorCoverings, doors: c.doorStyles, windows: c.windowStyles, roofs: c.roofStyles, roofColors: c.roofColors });
    // Public places seen from another lot show their things in the open (as the overview does).
    this.street = new Street(this.app, this.root, this.deps.assets, this.house, {
      lived: true,
      look: (def) => {
        const d = c.object(def);
        return d && { model: d.model ?? `model.${def}`, footprint: d.footprint ?? [1, 1], standIn: standInOf(c, d) };
      },
      lod: this.quality.streetLod,
      shadows: this.quality.streetShadows,
    });
    this.decorTemplates = new TemplateCache(this.app);
    this.storeyLayer = new pc.Entity('storeyLayer');
    this.root.addChild(this.storeyLayer);
    // Picking, the resize arrows (in render space) and the hover tile (in lot rows, on the storey layer).
    this.picker = new Picker(this.camera, this.lot, { storey: () => this.storey, storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), size: this.cssSize });
    this.handles = new ResizeHandles(this.device, this.root);
    this.hoverTile = new HoverTile(this.device, this.storeyLayer);
    // (Stage 8C: the wall tools' previews, the Paint and Floor tools', the room overlay and the grid.)
    this.previews = new BuildPreviews(this.app, this.storeyLayer, this.house, { content: this.deps.content, world: () => this.lastWorld, view: () => this.view, storey: () => this.storey, rowOf: (k) => this.rowOf(k), wallMode: () => this.wallMode });
    if (!this.diag.has('nolamps')) this.gardenPools = new GardenPools(this.app, this.root);
    // (Stage 6C: the interior lights, `lamps.ts`.)
    this.lamps = new Lamps(this.app, this.quality, { storey: () => this.storey, storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), town: () => this.townMode }, { house: this.house, lot: this.lot, pools: () => this.applyGardenPools() });
    // (Stage 8B: placing things, `placing.ts`; after the lamps, whose lights light the thing in hand.)
    this.placing = new Placing(this.app, this.root, this.storeyLayer, this.camera.camera, this.deps, this.lot, { storeyOf: (z) => this.storeyOf(z), rowOf: (k) => this.rowOf(k), town: () => this.townMode, shadowsMoved: () => this.sunShadows.moved() });
    // A file that failed has come: the thing in hand and the landscape's plants are built with it now, the rest
    // once the files settle (Stage 8B: `shared/modelRefresh.ts`, held while the town streams: `holdModelRefresh`).
    this.unlistenFiles = onModelFileArrived(this.app, (url) => {
      this.placing.fileArrived(url);
      this.lot.refreshNature(url, () => this.disposed); // (Stage 5, lotContents.ts: the landscape's plants built again in place.)
      this.modelRefresh.fileArrived(url);
    });
    this.applyWallMode();

    const o = this.orbit;
    o.wheelDeltaPercentage = 0.012;
    o.panningSensibility = 90;
    o.mapPanning = true;
    o.inertia = 0.82;
    o.panningInertia = 0.82;
    this.detachOrbit = o.attach(canvas);

    if (this.debug) Object.assign(window, { __scene: this.app.scene, __app: this.app, __renderer: this });
    this.applyLighting(this.lastMinute);
    this.applyCamera();
    void this.characters.init();
    await this.setupEnvironment();
  }

  /** Image-based lighting from the sky (manifest key `environment.sky`; its PlayCanvas copy, `env.ts`). */
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

  private readonly frame = (time: number): void => {
    if (!this.active || this.disposed) return;
    this.pacer.requestAnimationFrame(this.frame);
    // Behind a full-screen menu: its last frames, then only those something waits for.
    if (this.covered) {
      if (this.coveredFrames > 0) this.coveredFrames--;
      else if (!this.frameWaiters.length) return;
    }
    const start = performance.now();
    const frozen = this.diag.has('freeze');
    const dt = (this.frameDt = frozen ? 0 : Math.min(this.lastTime ? time - this.lastTime : 1000 / 60, MAX_FRAME_MS));
    this.lastTime = time;
    const probe = this.probe;
    let updated = start;
    try {
      probe?.before(start);
      this.onFrame?.(this.pacer.frameTime || performance.now());
      updated = performance.now();
      if (this.townMode) this.townFrame(dt);
      else this.orbit.step(dt, Math.max(0.05, (dt * 60) / 1000));
      this.sunShadows.frame();
      this.applyCamera();
      this.lot.frame(dt); // (Stage 5: the sky round the camera.)
      if (!this.townMode) this.updateCutaway();
      // (The marker faces the camera as it is now, after this frame's input.)
      this.placeMarker();
      this.handles.frame(this.camera.eyeInGame(this.eyeTmp));
      if (this.townMode) this.townListener?.();
      else this.onView?.();
      this.app.update(dt / 1000);
      this.app.render();
      this.drawCalls = this.device._drawCallsPerFrame;
      this.device._drawCallsPerFrame = 0;
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
    const end = performance.now();
    this.cpuMs += (end - start - this.cpuMs) * 0.1;
    if (dt > 0) this.frameMs += (dt - this.frameMs) * 0.1;
    if (probe) {
      const c = this.counts;
      c.drawCalls = this.drawCalls;
      const gpu = this.device.gpuProfiler;
      c.gpuMs = gpu?.enabled ? (gpu.frameTime ?? 0) : 0;
      if (c.gpuMs > 0) c.gpuFrames++;
      probe.after(start, updated, end, c);
    }
  };

  // --- the camera ----------------------------------------------------------------------

  /** Puts the camera where the game (or the overview) has it this frame. */
  private applyCamera(): void {
    if (this.townMode) {
      const p = this.townPose;
      this.camera.setPose(p.alpha, p.beta, p.radius, p.target.x, p.target.y, p.target.z, p.targetScreenOffset.x, p.targetScreenOffset.y);
    } else {
      const o = this.orbit;
      this.camera.setPose(o.alpha, o.beta, o.radius, o.target.x, o.target.y, o.target.z);
    }
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
    // Placed: the next world doesn't turn it to its lot (as Babylon's `setCameraPose` had it).
    this.cameraPlaced = true;
  }

  focus(x: number, z: number): void {
    // Keep the viewing angle; move the camera with its target.
    const o = this.orbit;
    o.target.x = x;
    o.target.y = 0;
    o.target.z = z;
    o.radius = Math.min(o.radius, 32);
  }

  setGameShot(shot: GameShot | null, cut = false): void {
    if (!shot) return this.gameRig.release();
    this.followIndex = null;
    this.gameRig.setShot(shot, cut || this.townMode);
  }

  followSim(index: number | null): void {
    this.followIndex = index;
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
  }

  setLeftDragCamera(on: boolean): void {
    this.orbit.buttons = on ? [0, 1, 2] : [1, 2];
  }

  cameraInfo(): { fov: number; radius: readonly [number, number]; beta: readonly [number, number] } {
    return { fov: LENS.fov, radius: RADIUS_LIMITS, beta: BETA_LIMITS };
  }

  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean {
    return this.camera.project(x, y, z, this.cssSize.width, this.cssSize.height, out);
  }

  // --- the world -----------------------------------------------------------------------

  setWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    if (this.rebuildNext) this.rebuildNext = false;
    else if (this.patchable(world, view)) {
      this.patchWorld(world);
      return Promise.resolve();
    }
    this.lastWorld = world;
    this.nextWorld = { world, view };
    return (this.worldBuilds ??= this.buildWorlds());
  }

  /**
   * Whether `world` differs from the built one only in ways what's on screen can take: no new
   * walls, floors, furniture or clothes, the same storey in view. (The simulation sends a world
   * message for every opened door and every growth stage, `structure_version` in
   * `crates/sim-core/src/world.rs`.)
   */
  private patchable(world: WorldStructure, view: ViewRect | null): boolean {
    const why = this.unpatchable(world, view);
    if (!why) return true;
    if (this.debug) console.info('[render] world built:', why);
    return false;
  }

  /** What keeps `world` from being patched into what's on screen (null: nothing, it can be). */
  private unpatchable(world: WorldStructure, view: ViewRect | null): string | null {
    const built = this.builtWorld;
    if (!built) return 'never built';
    if (this.worldBuilding) return 'building';
    if (this.townMode) return 'town';
    if (this.storey !== this.builtStorey) return `storey ${this.storey} built ${this.builtStorey}`;
    if (!sameView(view, this.view)) return 'view';
    if (built.width !== world.width || built.depth !== world.depth || built.storeys !== world.storeys) return 'size';
    if (built.storeyDepth !== world.storeyDepth || built.basement !== world.basement) return 'storeys';
    // The lot itself (walls, rooms, floors, fences) is what costs a build; a lean message never has it.
    if (built.lotVersion !== world.lotVersion) return 'lot';
    if (built.objects.length !== world.objects.length) return 'object count';
    if (built.sims.length !== world.sims.length) return 'sim count';
    for (let i = 0; i < world.objects.length; i++) {
      if (objectKey(built.objects[i]) !== objectKey(world.objects[i])) return `object ${world.objects[i].id} ${world.objects[i].def}`;
    }
    for (let i = 0; i < world.sims.length; i++) {
      const [a, b] = [built.sims[i], world.sims[i]];
      if (a.id !== b.id || a.gender !== b.gender || a.stage !== b.stage || a.name !== b.name) return `sim ${b.id}`;
      if (JSON.stringify(a.appearance) !== JSON.stringify(b.appearance)) return `sim look ${b.id}`;
    }
    return null;
  }

  /** Applies a `patchable` world in place (what's drawn of an object, its open state and growth). */
  private patchWorld(world: WorldStructure): void {
    this.lot.patch(world); // (Stage 5: objects' clips and picking boxes.)
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
    }
  }

  /** Everything the lot layer (roads and paths, house, fences) is built from. */
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
    this.placing.worldBuilding(); // (Stage 8B: the hand and the pops let go of the buffers this build replaces.)
    this.view = view;
    const { width, depth } = world;
    const [cx, cz] = [view ? view.x + view.w / 2 : width / 2, view ? view.z + view.d / 2 : depth / 2];
    // The sun's shadows cover a fixed square round the lot (`sunShadows.ts`).
    this.sunShadows.setSquare(cx, cz, Math.hypot(view?.w ?? width, view?.d ?? depth) / 2 + SHADOW_MARGIN);
    if (!this.cameraPlaced) {
      this.aimAt(cx, cz);
      this.cameraPlaced = true;
    }
    // (Stage 5, lotContents.ts: the landscape, with its country roads, in place of the plane.)
    if (this.ground) this.ground.enabled = false;
    await this.lot.buildNature(world, (roads, parent) => drawRoads(this.app, roads, parent));

    const lotKey = this.lotKeyOf(world, view);
    const lotBuilt = lotKey !== this.lotKey;
    if (lotBuilt) {
      this.clearLot();
      this.lotKey = lotKey;
      this.buildStreets(world);
      await this.pace();
      await this.buildLot(world);
      await this.pace();
      // The viewed lot's fences stand on the ground (hidden with it while the basement is in view); other lots' stay.
      const v = view;
      const onLot = (f: FenceEdge) => !!v && f.x >= v.x && f.z >= v.z && f.x + (f.axis === 'h' ? 1 : 0) <= v.x + v.w && f.z + (f.axis === 'v' ? 1 : 0) <= v.z + v.d;
      for (const [list, storey] of [[(world.fences ?? []).filter(onLot), 0], [(world.fences ?? []).filter((f) => !onLot(f)), undefined]] as const) {
        const fences = buildFences(this.app, this.root, list, this.deps.content.fenceStyles);
        if (!fences) continue;
        this.lotEntities.push({ entity: fences.entity, storey });
        for (const leaf of fences.gates) this.gates.push({ fences, leaf, open: 0, towards: 1 });
      }
    }
    this.placeRoomLights(world, view);
    await this.pace();
    await this.street.build(world, this.view, () => this.pace());
    await this.lot.buildObjects(world); // (Stage 5: the objects, their clips, contact shadows, items.)
    this.placing.objectsBuilt(); // (Stage 8B: the thing in hand hidden in them.)
    await this.buildSims(world);
    this.applyWallMode();
    this.applyStoreys();
    this.lastLightMinute = -1;
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
    await this.characters.build(world, 'residents');
    const v = this.view;
    const key = v ? `${v.x},${v.z},${v.w},${v.d}` : '';
    const framing = !!v && key !== this.framedView;
    if (framing) this.framedView = key;
    if (framing && this.followIndex === null) {
      const entry = world.plots.find((p) => p.x === v.x && p.z === v.z)?.entry;
      const dx = entry ? entry[0] - (v.x + v.w / 2) : 0;
      const dz = entry ? entry[1] - (v.z + v.d / 2) : 0;
      if (Math.hypot(dx, dz) > 1) this.orbit.alpha = Math.atan2(dz, dx) - Math.PI * 0.12;
    }
    const emotions = this.deps.content.emotions.map((e) => e.id);
    this.characters.setEmotions(emotions);
    this.marker.setEmotions(emotions);
    this.sunShadows.moved();
  }

  /** Turns the camera to (`x`, `z`) on the ground from where it is, as Babylon's `ArcRotateCamera.setTarget` re-aims it. */
  private aimAt(x: number, z: number): void {
    const o = this.orbit;
    const sb = Math.sin(o.beta);
    const eye = [o.target.x + o.radius * Math.cos(o.alpha) * sb, o.target.y + o.radius * Math.cos(o.beta), o.target.z + o.radius * Math.sin(o.alpha) * sb];
    const [dx, dy, dz] = [eye[0] - x, eye[1], eye[2] - z];
    const r = Math.hypot(dx, dy, dz) || o.radius;
    o.target.x = x;
    o.target.y = 0;
    o.target.z = z;
    o.radius = r;
    o.alpha = Math.atan2(dz, dx);
    o.beta = Math.acos(Math.max(-1, Math.min(1, dy / r)));
    o.checkLimits();
  }

  /** A world build gives the frames it runs behind one every `BUILD_SLICE_MS` of main-thread time. */
  private async pace(): Promise<void> {
    if (!this.worldBuilding || performance.now() - this.sliceStart < BUILD_SLICE_MS) return;
    await new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
    this.sliceStart = performance.now();
  }

  /** Roads, garden paths and farm fields from the town metadata, each a flat mesh. */
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
      const geo = new Geo();
      for (const r of rects) geo.poly([[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]], [0, 1, 0], [[0, 0], [1, 0], [1, 1], [0, 1]]);
      const mesh = meshFrom(this.device, geo.meshData());
      if (mesh) this.lotEntities.push({ entity: drawMesh(this.root, name, mesh, this.lotMaterial(material)) });
    }
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
    // The ground first (its garden dressing is the lot's), then upstairs, then the basement.
    const order = [...Array.from({ length: count }, (_, k) => k), ...(lowest < 0 ? [-1] : [])];
    let topped = false;
    let ground: { shrubs: [number, number, number, number][]; trees: [number, number, number, number][] } | null = null;
    const lotRooms = new Map<number, { n: number; sx: number; sz: number }>();
    const surfaces: HouseSurfaces[] = []; // (Stage 8C: where the Paint and Floor tools lay their previews.)
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
      this.root.addChild(storey);
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
      surfaces.push(built.surfaces); // (Stage 8C.)
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
      this.previews.setSurfaces(null); // (Stage 8C.)
      // Unknown wall layout: the simulation's plain meshes.
      const add = (name: string, data: WorldStructure['meshes']['walls'], material: pc.StandardMaterial) => {
        const mesh = meshFrom(this.device, data);
        if (!mesh) return null;
        const entity = drawMesh(this.root, name, mesh, material, { castShadows: true });
        this.lotEntities.push({ entity });
        return entity;
      };
      add('floors', world.meshes.floors, this.lotMaterial('material.floor'));
      const wall = this.lotMaterial('material.wall');
      this.walls = add('walls', world.meshes.walls, wall);
      this.wallsLow = add('wallsLow', world.meshes.wallsLow, wall);
      return;
    }
    // Garden dressing split into species (hedges or mixed shrub borders, varied lot trees).
    const decor = [...natureDecor('model.bush', ground.shrubs), ...natureDecor('model.tree', ground.trees, world, this.view)];
    for (const [key, items] of decor) {
      await this.pace();
      void this.placeDecor(key, items);
    }
    this.lotRooms = lotRooms;
    this.previews.setSurfaces(mergeSurfaces(surfaces)); // (Stage 8C.)
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
        const d = new Instances(this.device, this.root, m.mesh, m.material, { name: m.name, castShadows: true, receiveShadows: true });
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
    const geo = new Geo().color('#6B5440');
    geo.poly([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0, 1, 0]);
    geo.color('#4A3A2C');
    geo.poly([[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]], [0, 0, 1]);
    geo.poly([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0, 0, -1]);
    geo.poly([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [1, 0, 0]);
    geo.poly([[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]], [-1, 0, 0]);
    const mesh = meshFrom(this.device, geo.meshData({ colors: true }));
    if (!mesh) return;
    const entity = drawMesh(this.root, 'basementPit', mesh, this.lib.surface('material.soil', { vertexColors: true, doubleSided: true }), { castShadows: true });
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
      const row = z - this.rowOf(this.storeyOf(z));
      return !view || (x >= view.x && row >= view.z && x < view.x + view.w && row < view.z + view.d);
    };
    this.markWindowless(world, view);
    const lights: RoomLight[] = this.lotRooms ? placeLights(world, inView, this.lotRooms, (def) => this.deps.content.object(def)?.light).filter((l) => shown(l.z)) : [];
    this.gardenPools?.set(lights.filter((l) => l.kind === 'lamp' && l.room === 0 && this.storeyOf(l.z) === 0).map((l) => gardenPool(l, this.rowOf(0))));
    this.applyGardenPools();
    // (Stage 6C: the interior lights and the rooms they keep to, `lamps.ts`.)
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
    this.lamps.setDimmed(dark.dimmed); // (Stage 6C: their lights on by day, `lamps.ts`.)
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

  /** The garden's pools of lamplight, as bright as its lamps are now. */
  private applyGardenPools(): void {
    this.gardenPools?.update(this.glowLevel, this.poolLampOn, this.lampColor);
  }

  /** Model files that failed have come (gathered, Stage 8B: `modelRefresh`): the world draws again what it drew without them (once no build runs). */
  private refreshStreet(urls: readonly string[]): void {
    let drawn = false;
    for (const url of urls) {
      for (const [, p] of this.decor) if (p.template.failed?.includes(url)) this.lotKey = '';
      this.decorTemplates.invalidate(url);
      // (Stage 5, lotContents.ts: the objects drawn without it with the world.)
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
      this.lot.disposeStale(); // (Stage 5.)
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
    this.characters.clear();
    this.simShadowMatrices.fill(0);
    this.simShadow.setCount(0);
    this.simShadow.update(false);
    this.simCount = 0;
    this.selectedSim = null;
    this.lastFrame = null;
    this.marker.place(null, 0, this.camera.entity, 0, true);
    this.hoverTile.set(null);
    this.previews.clear(); // (Stage 8C.)
    this.framedView = '';
    this.placing.clear(); // (Stage 8B.)
    this.lot.clear(); // (Stage 5.)
    this.clearLot();
    this.lotParts = null;
    this.street.clear();
    this.windowless = { rooms: new Set(), of: [] };
    this.dimming = null;
    this.applyDimming();
    this.gardenPools?.set([]);
    this.lamps.clear(); // (Stage 6C.)
    for (const p of this.decor.values()) for (const d of p.draws) d.enabled = false;
    this.lastWorld = this.builtWorld = null;
    this.cameraPlaced = false;
    this.movers.length = 0;
    this.darkRooms.clear();
    this.awakeRooms.clear();
    this.occupiedRooms.clear();
  }

  /** Takes down the lot layer (it's built again for the next lot). */
  private clearLot(): void {
    for (const { entity } of this.lotEntities) entity.destroy();
    this.lotEntities = [];
    this.lotKey = '';
    this.roofs = [];
    this.doors = [];
    this.gates = [];
    this.stairwells = [];
    this.lotRooms = null;
    this.walls = this.wallsLow = null;
  }

  private lotMaterial(key: string): pc.StandardMaterial {
    // Generated geometry has explicit normals; don't depend on triangle winding.
    return this.lib.surface(key, { doubleSided: true });
  }

  /** Walls up, cut away or down: the cutaway state the wall materials read, the roofs, and the fallback's full or low walls. */
  private applyWallMode(): void {
    const mode = this.wallMode;
    if (this.walls) this.walls.enabled = mode === 'up';
    if (this.wallsLow) this.wallsLow.enabled = mode !== 'up';
    this.roofShown = this.roofVisible();
    this.showRoofs();
    this.lib.wallCut.params.x = mode === 'down' ? 1 : 0;
    this.lib.wallCut.params.y = WALL_STUB;
    this.cutLook = -1;
    this.updateCutaway();
  }

  /** The viewed lot's roof: only with walls up, and not when zoomed in (it would fill the view). */
  private roofVisible(): boolean {
    return this.wallMode === 'up' && this.orbit.radius > ROOF_MIN_DISTANCE;
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

  /** Storeys above the one in view are hidden (walls, floors, roofs), and the garden while the basement is. */
  private applyStoreys(): void {
    for (const { entity, storey } of this.lotEntities) if (storey !== undefined) entity.enabled = storeyShown(storey, this.storey);
    for (const p of this.decor.values()) for (const d of p.draws) d.enabled = this.storey >= 0 && d.count > 0;
    this.applyDig();
    this.sunShadows.moved();
  }

  /** Cutaway: walls facing the camera drop when the view direction crosses a quadrant. */
  private updateCutaway(): void {
    let bits = 0;
    if (this.wallMode === 'cutaway') {
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
    this.placing.cutWallItems(bits, this.wallMode === 'down'); // (Stage 5: things on the walls go with them; Stage 8B: but the one in hand, and none while something for a wall is.)
    this.lib.wallCut.look.set(bits & 1 ? 1 : 0, bits & 2 ? 1 : 0, bits & 4 ? 1 : 0, bits & 8 ? 1 : 0);
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
    const houses = this.townMode || this.lampsLevel <= 0 ? NO_HOUSES : this.street.houses;
    const up = this.neighboursUp;
    up.length = houses.length;
    up.fill(false);
    const babies = this.babyStages();
    const count = curr[layout.header.simCount];
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      if (curr[o + k.away] !== 0) continue;
      const px = curr[o + k.x];
      const pz = curr[o + k.z];
      if (curr[o + k.moving] !== 0) {
        // (An older layout without the way ahead: where they are, so only a doorway they're in opens.)
        const wx = k.wayX;
        const wz = k.wayZ;
        const ax = k.aheadX;
        const az = k.aheadZ;
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
      for (let h = 0; h < houses.length; h++) {
        const b = houses[h];
        if (px >= b[0] && px <= b[2] && gz >= b[1] && gz <= b[3]) {
          up[h] = true;
          break;
        }
      }
    }
    for (const room of awake) dark.delete(room);
    if (houses.length) this.street.setUp(up);
  }

  /** Life stage ids of babies (content `baby`: they don't count as someone up, as in sim-core). */
  private babyStages(): ReadonlySet<string> {
    const stages = this.deps.content.lifeStages;
    if (this.babies?.of !== stages) this.babies = { of: stages, ids: new Set(stages.filter((s) => s.baby).map((s) => s.id)) };
    return this.babies.ids;
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
    let moved = false;
    for (const gate of this.gates) {
      const way = goingThrough(movers, gate.leaf.axis, gate.leaf.centre, gate.open === 1);
      // It opens away from the one coming through, and shuts the way it opened.
      if (way && gate.open === 0) gate.towards = way;
      const target = way ? 1 : 0;
      const open = target > gate.open ? Math.min(target, gate.open + step) : Math.max(target, gate.open - step);
      if (open === gate.open) continue;
      gate.open = open;
      swingGate(gate.fences, gate.leaf, ease(open), gate.towards);
      moved = true;
    }
    for (const door of this.doors) {
      const { leaf } = door;
      // (Both in lot rows: the storey is in the row.)
      const going = goingThrough(movers, leaf.axis, leaf.centre, door.open === 1) !== 0;
      const engaged = leaf.bath !== 0 && this.occupiedRooms.has(leaf.bath);
      const target = going || !(leaf.front || engaged) ? 1 : 0;
      const open = door.open < 0 ? target : target > door.open ? Math.min(target, door.open + step) : Math.max(target, door.open - step);
      if (open === door.open) continue;
      door.open = open;
      // Eased: quick off the frame, slowing as it meets the jamb or the wall. One that slides stays
      // turned shut and slides into the wall (from where it was built).
      const k = ease(open);
      const { slide, base } = leaf;
      if (slide) leaf.pivot.setLocalPosition(base.x + slide[0] * k, base.y, base.z + slide[1] * k);
      leaf.pivot.setLocalEulerAngles(0, ((slide ? leaf.close : leaf.close * (1 - leaf.opens * k)) * 180) / Math.PI, 0);
      moved = true;
    }
    if (moved) this.sunShadows.moved();
  }

  /** A flat matte material in an sRGB colour, two-sided (the simulation's meshes don't keep a winding). */
  private flat(key: string, hex: string): pc.StandardMaterial {
    let m = this.materials.get(key);
    if (!m) {
      m = new pc.StandardMaterial();
      m.diffuse = new pc.Color().fromString(hex);
      m.useMetalness = true;
      m.metalness = 0;
      m.gloss = 0.15;
      m.cull = pc.CULLFACE_NONE;
      // (In the look, the diag views and the environment's light like the library's own.)
      this.lib.adopt(m);
      m.update();
      this.materials.set(key, m);
    }
    return m;
  }

  /** The ground round the lot: a plain plane for now (the landscape comes with stage 5). */
  private showGround(width: number, depth: number): void {
    if (!this.ground) {
      this.ground = new pc.Entity('ground');
      this.ground.addComponent('render', { type: 'plane', material: this.flat('grass', '#5E8A3A'), castShadows: false, receiveShadows: true });
      this.root.addChild(this.ground);
    }
    const size = Math.max(width, depth) + 200;
    this.ground.setLocalPosition(width / 2, -0.01, depth / 2);
    this.ground.setLocalScale(size, 1, size);
  }

  update(frame: FrameState): void {
    const { curr, layout } = frame;
    const minute = this.fixedHour !== null ? this.fixedHour * 60 : (curr[layout.header.minute] ?? this.lastMinute);
    this.lastMinute = minute;
    if (this.townMode) return;
    const h = layout.header;
    // At speed 3 and up, and in quiet hours, the preset's step (Low's ten minutes: each update re-colours the sun, sky and lamps).
    const fast = curr[h.speed] >= 3 || (h.calm !== undefined && curr[h.calm] > 0);
    const step = fast ? Math.max(LIGHT_STEP, this.quality.lightingStep) : LIGHT_STEP;
    const due = this.lastLightMinute < 0 || frame.now - this.lastLightAt >= LIGHT_MIN_MS;
    if (due && Math.abs(minute - this.lastLightMinute) >= step) {
      this.applyLighting(minute);
      this.lastLightMinute = minute;
      this.lastLightAt = frame.now;
    }
    // Who is where: doors open for those going through, neighbours' windows glow while someone is up.
    this.findDarkRooms(frame);
    this.lamps.frame(this.frameDt / 1000, this.darkRooms, this.awakeRooms); // (Stage 6C: lamps and glows fading, `lamps.ts`.)
    this.swingDoors(GAME_RATE[curr[h.speed] | 0] ?? 1);
    this.steerCamera(this.frameDt / 1000);
    this.lot.update(frame); // (Stage 5: the objects' clips, the screens.)
    this.placing.frame(this.frameDt); // (Stage 8B: the effects, the thing in hand.)
    this.updateSims(frame);
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
    const count = (this.simCount = Math.min(curr[layout.header.simCount], MAX_CHARACTERS));
    const shadows = this.simShadowMatrices;
    this.characters.update(frame, this.simInView, shadows, this.storey);
    this.lot.pullChairs(this.characters.pulled, this.placing.held); // (Stage 8B: not the one in hand.)
    if (this.characters.posed && this.quality.characterShadows) this.sunShadows.moved(false);
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
    this.marker.place(shown ? this.markerHead : null, emotion, this.camera.entity, frame?.now ?? 0, !!this.options.reducedMotion);
  }

  warmGame(_frame: FrameState): Promise<void> {
    return this.framesRendered(1);
  }

  setItems(items: readonly ItemPlacement[]): void {
    this.lot.setItems(items); // (Stage 5.)
  }
  setLampsOff(off: ReadonlySet<number>): void {
    this.lot.setLampsOff(off); // (Stage 5: fans and fires.)
    this.lamps.setLampsOff(off); // (Stage 6C: their light and fades, `lamps.ts`.)
    this.applyGardenPools();
  }
  holdModelRefresh(on: boolean): void {
    this.modelRefresh.hold(on); // (Stage 8B.)
  }

  // --- light ---------------------------------------------------------------------------

  /**
   * The time of day (`shared/environment.ts`): the sun, the sky's fill, the environment's light,
   * the fog and the clear colour, in PlayCanvas's units (`lightUnits.ts`). No allocations.
   */
  private applyLighting(minute: number): void {
    this.counts.lightingUpdates++;
    const l = lightingAt(minute, LOOK, this.lighting);
    const mix = this.lightMix;
    if (mix) {
      l.sunIntensity *= mix[0];
      l.skyIntensity *= mix[1];
      l.envIntensity *= mix[2];
    }
    // A directional light shines along its entity's -y: turned from straight down to the sun's way (game space to PlayCanvas).
    const to = toEngine(l.sunDirection.x, l.sunDirection.y, l.sunDirection.z, this.vTmp).normalize();
    const axis = this.vTmp2.cross(DOWN, to);
    const sin = axis.length();
    const angle = Math.atan2(sin, DOWN.dot(to)) * pc.math.RAD_TO_DEG;
    this.sun.setRotation(sin > 1e-6 ? this.qTmp.setFromAxisAngle(axis.mulScalar(1 / sin), angle) : this.qTmp.set(0, 0, 0, 1));
    // The look's colours are linear as Babylon used them; PlayCanvas takes sRGB and linearises it,
    // so the strength goes into the colour (an intensity below 1 would be linearised with it).
    this.sun.light!.color = gammaColor(l.sunColor, l.sunIntensity * SUN_UNITS, this.colorTmp);
    this.sunShadows.moved();
    // The sky's hemispheric fill (Babylon's hemispheric light) and its image-based light.
    this.lib.setSkyFill(l.skyColor, l.groundColor, l.skyIntensity * SKY_UNITS);
    this.lib.setEnvironmentIntensity(l.envIntensity * ENV_UNITS);
    // The haze in the horizon's colour as Babylon's PBR had it: linearised twice (the renderer's
    // `toLinearSpace`, then its materials': a deeper blue than the horizon), so it goes to PlayCanvas
    // (which linearises what it's given) linearised once. The clear colour (behind the sky's dome)
    // is drawn as it's given: linear, as Babylon's.
    const c = l.clearColor;
    this.app.scene.fog.color.set(toLinear(c.r), toLinear(c.g), toLinear(c.b), 1);
    this.camera.camera.clearColor = this.colorTmp.set(toLinear(c.r), toLinear(c.g), toLinear(c.b), 1);
    // (Stage 6A.) Babylon's exposure for the hour and the moonlight grade (`post.ts`: uniforms).
    if (this.post) {
      this.post.exposure = l.exposure * LOOK.exposure;
      this.post.night = l.night * LOOK.nightGrade;
    }
    // Under the ground (a basement in view, see `applyDig`) daylight doesn't reach.
    const seen = Math.max(l.exposure, 1e-3);
    this.lib.dig.lit.set(0, Math.min(1, BASEMENT_SKY / Math.max(l.skyIntensity * seen, 1e-3)), Math.min(1, BASEMENT_ENV / Math.max(l.envIntensity * seen, 1e-3)), 0);
    // Glowing windows outside at night, cool daylight in the panes by day; the street's lamps; the garden's pools.
    this.lampsLevel = l.lamps;
    this.lot.applyLighting(l); // (Stage 5: the sky, the night clips, the shades' glow.)
    this.placing?.light(l); // (Stage 8B: the red ghost's light, as Babylon's lights had it.)
    this.lamps.applyLighting(l); // (Stage 6C: the interior lights and the glows, `lamps.ts`.)
    const glow = Math.min(1, Math.max(0, (l.lamps - 0.35) / 0.5));
    const g = this.glowColor;
    const k = glow * glow * 1.6;
    this.lib.setEmissive(this.lib.surface('material.window', { vertexColors: true, cutaway: true }), this.emissiveTmp.set(g.r * k, g.g * k, g.b * k));
    const inner = (1 - l.lamps) * 0.9 + 0.05;
    this.lib.setEmissive(this.lib.surface('material.window.inner', { vertexColors: true, cutaway: true }), this.emissiveTmp.set(toLinear(c.r) * inner, toLinear(c.g) * inner, toLinear(c.b) * inner));
    this.street?.setNight(l.lamps, this.glowColor);
    this.applyGardenPools();
  }

  // --- the menus' overview -------------------------------------------------------------

  get townShown(): boolean {
    return this.townMode;
  }

  showTown(world: WorldStructure, minute: number): Promise<void> {
    if (!this.townMode) {
      const o = this.orbit;
      const p = this.townPose;
      Object.assign(p, { alpha: o.alpha, beta: o.beta, radius: o.radius });
      Object.assign(p.target, o.target);
      this.townRig.syncFromCamera();
      this.orbit.enabled = false;
      this.orbit.stopInertia();
      this.townMinute = minute;
    }
    this.townMode = true;
    this.applyDig(); // (Stage 6C: no basement or windowless rooms kept dark in the town, as Babylon's enterTown.)
    this.sunShadows.setOverview(true);
    this.hoverTile.set(null);
    this.townMinuteGoal = minute;
    this.lastLightMinute = -1;
    this.applyPacing();
    // (Stage 3: the town's land as a plane; the overview of its lots comes with stage 9.)
    this.showGround(world.width, world.depth);
    return Promise.resolve();
  }

  hideTown(glide = 0): Promise<void> {
    if (!this.townMode) return Promise.resolve();
    return new Promise((resolve) => {
      const done = () => {
        this.townMode = false;
        this.applyDig(); // (Stage 6C: the basement and windowless rooms dark again, as Babylon's leaveTown.)
        this.sunShadows.setOverview(false);
        this.flyingIn = false;
        this.orbit.enabled = true;
        this.orbit.stopInertia();
        this.lastLightMinute = -1;
        this.app.scene.fog.density = LOOK.fog;
        this.applyLighting(this.lastMinute);
        this.lastLightMinute = this.lastMinute;
        this.applyPacing();
        resolve();
      };
      if (glide > 0) {
        const o = this.orbit;
        this.flyingIn = true;
        this.applyPacing();
        this.townRig.glideTo({ alpha: o.alpha, beta: o.beta, radius: o.radius, x: o.target.x, z: o.target.z }, glide, done);
      } else done();
    });
  }

  setTownShot(shot: TownShot, cut = false): void {
    this.townRig.setShot(shot, cut);
    if (shot.minute !== undefined) this.townMinuteGoal = shot.minute;
  }

  nudgeTown(dAlpha: number, zoom: number): void {
    this.townRig.nudge(dAlpha, zoom);
  }

  setLotHighlight(_highlight: LotHighlight): void {}

  onTownFrame(fn: (() => void) | null): void {
    this.townListener = fn;
  }

  private townFrame(dt: number): void {
    const w = this.cssSize.width || 1;
    const h = this.cssSize.height || 1;
    this.townRig.setViewport(w, h);
    this.townRig.frame(Math.min(0.1, dt / 1000), w / h);
    // The light eases between the screens' hours (about two seconds).
    const goal = this.townMinuteGoal;
    if (this.townMinute !== goal) {
      const step = Math.max(0.5, Math.abs(goal - this.townMinute) * Math.min(1, dt / 600));
      this.townMinute = Math.abs(goal - this.townMinute) <= step ? goal : this.townMinute + Math.sign(goal - this.townMinute) * step;
    }
    if (this.lastLightMinute !== this.townMinute) {
      this.applyLighting(this.townMinute);
      this.lastLightMinute = this.townMinute;
    }
    // Haze is tuned for the game's close camera; from high above it would wash the town out.
    const fog = LOOK.fog * Math.min(1, Math.max(0.3, 55 / this.townPose.radius));
    if (Math.abs(fog - this.app.scene.fog.density) > LOOK.fog * 0.02) this.app.scene.fog.density = fog;
  }

  // --- picking -------------------------------------------------------------------------

  pick(x: number, y: number): PickResult {
    return this.picker.pick(x, y);
  }

  pickPlane(height: number): { x: number; z: number } | null {
    return this.picker.pickPlane(height);
  }

  pickVertical(x: number, z: number): number | null {
    return this.picker.pickVertical(x, z);
  }

  pickOnWall(axis: 'h' | 'v', line: number, storeyRow: number): { along: number; y: number } | null {
    return this.picker.pickOnWall(axis, line, storeyRow);
  }

  objectBox(id: number): { height: number; bottom: number } | null {
    return this.picker.objectBox(id);
  }

  setResizeHandles(handles: readonly ResizeHandle[] | null, hot: string | null = null): void {
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
    // Taken before the merge, which would otherwise hide the change. (`?look=` on the dev server wins over the setting, as Babylon's plugin had it.)
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
    this.lot.setReducedMotion(!!options.reducedMotion); // (Stage 5: the wind, the clouds.)
    this.characters.reducedMotion = !!options.reducedMotion;
    this.placing.reducedMotion = !!options.reducedMotion; // (Stage 8B.)
    this.applyPacing();
    this.applyResolution();
  }

  /** The frame rate cap, and in the menus at most `MENU_FPS` (not while the camera flies into the game). */
  private applyPacing(): void {
    const cap = this.options.fpsCap;
    const menu = this.townMode && !this.flyingIn;
    this.pacer.setCap(menu ? (cap > 0 ? Math.min(cap, MENU_FPS) : MENU_FPS) : cap);
  }

  private readonly onResize = () => this.applyResolution();

  /** Sizes the drawing to the canvas, the device pixel ratio and the resolution setting (`shared/resolution.ts`). */
  private applyResolution(): void {
    if (!this.app) return;
    const dpr = window.devicePixelRatio || 1;
    const w = this.cssSize.width * dpr;
    const h = this.cssSize.height * dpr;
    const scale = (this.renderScaleNow = renderScale(w, h, this.options.resolution));
    this.app.setCanvasResolution(pc.RESOLUTION_FIXED, Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale)));
  }

  stats(): RenderStats {
    const gpu = this.device.gpuProfiler;
    return {
      backend: this.device.isWebGPU ? 'WebGPU' : 'WebGL2',
      preset: this.quality.preset,
      fps: this.frameMs > 0 ? 1000 / this.frameMs : 0,
      frameMs: this.frameMs,
      cpuMs: this.cpuMs,
      drawCalls: this.drawCalls,
      holdStillFrames: false,
      renderWidth: this.device.width,
      renderHeight: this.device.height,
      renderScale: this.renderScaleNow,
      refreshHz: this.pacer.refreshHz,
      refreshDivisor: this.pacer.divisor,
      gpuMs: this.gpuWanted.size && gpu?.enabled && (gpu.frameTime ?? 0) > 0 ? (gpu.frameTime ?? null) : null,
      residentsMs: this.characters?.updateMs ?? 0,
    };
  }

  setFrameProbe(probe: FrameProbe | null): void {
    this.probe = probe;
  }

  measureGpu(on: boolean, who: 'overlay' | 'bench' = 'overlay'): void {
    if (on) this.gpuWanted.add(who);
    else this.gpuWanted.delete(who);
    if (this.device.gpuProfiler) this.device.gpuProfiler.enabled = this.gpuWanted.size > 0;
  }

  gpuPasses(_reset?: boolean): { name: string; ms: number; frames: number }[] {
    return [];
  }

  benchInfo(): Record<string, unknown> & { passes: RenderPassInfo[] } {
    // (The benchmark's counts so far; the rest comes with the bench support, stage 11.)
    return { engine: `PlayCanvas ${pc.version}`, backend: this.device.isWebGPU ? 'webgpu' : 'webgl2', preset: this.quality.preset, passes: [], scene: { lightingUpdates: this.counts.lightingUpdates, mirrorDraws: 0, residents: this.simCount, residentsDrawn: this.residentsDrawn() } };
  }

  /** Residents drawn now (the benchmark's). */
  private residentsDrawn(): number {
    let n = 0;
    for (let i = 0; i < this.simCount; i++) if (this.characters.visible[i]) n++;
    return n;
  }

  // (Stage 8D: the save's picture, `thumbnail.ts`; made at the first, gone with the app.)
  private thumbnails: Thumbnails | null = null;

  captureThumbnail(width: number, height: number): Promise<string | null> {
    if (this.disposed || !this.app) return Promise.resolve(null);
    this.thumbnails ??= new Thumbnails(this.app, this.camera.camera, this.post, { wantFrame: () => void this.framesRendered(1), shadowsMoved: () => this.sunShadows.moved() });
    return this.thumbnails.capture(width, height);
  }

  // --- residents, birds, voices (stage 7) ----------------------------------------------

  simHead(index: number, out: { x: number; y: number; z: number }): boolean {
    return index < this.simCount && !!this.characters?.head(index, out);
  }

  birds(): readonly SeenBird[] {
    return [];
  }

  birdSings(_id: number, _seconds: number): void {}
  setSelectedSim(id: number | null): void {
    this.selectedSim = id;
    this.characters?.select(id);
  }

  setSpeech(index: number, track: VisemeTrack | null, startTime: number): void {
    this.characters?.setSpeech(index, track, startTime);
  }

  // --- build and buy (stage 8) ---------------------------------------------------------

  setHoverTile(tile: { x: number; z: number } | null): void {
    this.hoverTile.set(tile);
  }

  setWallMode(mode: WallMode): void {
    this.wallMode = mode;
    this.applyWallMode();
  }

  setStorey(storey: number): void {
    const k = Math.max(this.storeys.lowest, Math.min(this.storeys.count - 1, Math.round(storey)));
    if (k === this.storey) return;
    this.storey = k;
    this.storeyLayer?.setLocalPosition(0, k * WALL_HEIGHT, -this.rowOf(k));
    this.applyStoreys();
    // The lights go to the storey now in view (also while a build is running, which skips the rebuild below).
    if (this.lastWorld) this.placeRoomLights(this.lastWorld, this.view);
    // Furniture and residents above it hide; the lot (every storey) is kept.
    if (this.lastWorld && !this.worldBuilding) void this.setWorld(this.lastWorld, this.view);
  }
  // (Stage 8B: placing things, `placing.ts`.)
  setPlacementGhost(ghost: PlacementGhost | null): void {
    this.placing.setPlacementGhost(ghost);
  }

  setHeldObject(id: number | null, moved = false): void {
    this.placing.setHeldObject(id, moved);
  }

  // (Stage 8C: Build mode's previews and overlays, `buildPreviews.ts`.)
  setEdgePreview(edges: readonly EdgePreview[], valid: boolean): void {
    this.previews.setEdgePreview(edges, valid);
  }

  setPaintPreview(faces: readonly PaintPreviewFace[], cover: number | null): void {
    this.previews.setPaintPreview(faces, cover);
  }

  setFloorPreview(tiles: readonly { x: number; z: number }[], floor: number | null): void {
    this.previews.setFloorPreview(tiles, floor);
  }

  setRoomOverlay(tiles: readonly RoomOverlayTile[]): void {
    this.previews.setRoomOverlay(tiles);
  }

  setBuildGrid(rect: ViewRect | null): void {
    this.previews.setBuildGrid(rect);
  }

  buildEffect(fx: BuildEffect): void {
    this.placing.buildEffect(fx); // (Stage 8B.)
  }

  // --- dev tools (`RendererDebugHooks`) ------------------------------------------------

  // (Stage 5, lotContents.ts.)
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
    this.characters?.dispose();
    this.simShadow?.destroy();
    this.marker?.destroy();
    this.handles?.destroy();
    this.hoverTile?.destroy();
    this.previews?.destroy(); // (Stage 8C.)
    this.placing?.dispose(); // (Stage 8B.)
    this.modelRefresh.dispose();
    this.lot?.dispose(); // (Stage 5.)
    this.setActive(false);
    this.onFailure = null;
    this.onFrame = null;
    this.onView = null;
    this.townListener = null;
    this.detachOrbit?.();
    this.detachOrbit = null;
    this.sizeWatch?.disconnect();
    this.unwatchPixelRatio?.();
    window.removeEventListener('resize', this.onResize);
    const debug = window as unknown as { __renderer?: unknown; __scene?: unknown; __app?: unknown };
    if (debug.__renderer === this) {
      delete debug.__renderer;
      delete debug.__scene;
      delete debug.__app;
    }
    for (const w of this.frameWaiters.splice(0)) w.resolve();
    this.unlistenFiles?.();
    this.unlistenFiles = null;
    this.street?.destroy();
    this.gardenPools?.destroy();
    for (const p of this.decor.values()) for (const d of p.draws) d.destroy();
    this.decor.clear();
    this.post?.destroy(); // (Stage 6A.)
    this.post = null;
    this.app?.destroy();
  }
}
