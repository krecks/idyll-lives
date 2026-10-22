/**
 * Garden birds about the viewed lot: blue tits, chaffinches, warblers, blackbirds and collared doves
 * living in its trees, bushes and hedges (and the street's and the woods' near it). Presentation
 * only: nothing of them is in the simulation or the save.
 *
 * - Each bird belongs to a home plant, made from the plant's position (the same birds in the same
 *   trees after every rebuild), as many as the plant's size and the preset (`QualitySettings.birds`)
 *   allow, of species by habitat: tits, finches and warblers in broadleaved trees and bushes,
 *   blackbirds low in bushes and on the lawn, collared doves in pairs, high up, on the roof ridge and
 *   walking the lawn.
 * - Perched on a perch point of its plant (manifest `perches`, model space; swaying with the branch
 *   as the wind bends it, `windBend`): small jerky glances of the head, a flick of the tail, preening
 *   now and then, a hop to a nearby twig, a turn on the spot. Small birds fidget; doves sit long.
 * - Flying: short hops within a plant, flights between plants, to the lawn and the roof, along a
 *   curve kept clear of the houses (their walls and roofs, a height per tile): a take-off burst; tits,
 *   finches and warblers bound (flapping on the rise, wings shut on the dip), blackbirds fly low and
 *   direct, doves flap steadily with glides; banking into turns, a flare to land.
 * - On the lawn (blackbirds, chaffinches, doves): hopping, running or walking (a dove's head bobs),
 *   pecking, looking up; never on paths, patios, in rooms or under furniture.
 * - Wary: a resident within a few metres, or the camera diving close, sends them up into the
 *   nearest plant away from it; they keep off the lawn for a while after.
 * - The day: busiest at dawn and early evening, quieter at midday; at dusk each flies home to roost
 *   (head turned back into its feathers, puffed up) and wakes at first light.
 * - Time: the birds' clock runs with the game's (paused: they hold still; at speeds 3-5 they hurry
 *   only a little, `RATE`). Reduce motion: no flights or hops but the one home; slow, eased head turns.
 *
 * Drawn as thin instances, a buffer per species and part (body, head, tail, the two spread wings);
 * only the birds that moved are written each frame, nothing is allocated per frame.
 */

import { Color3, Matrix, Mesh, MeshBuilder, Quaternion, StandardMaterial, Vector3, VertexBuffer, type Scene } from './core';
import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry, Vec3 } from '../../assets/types';
import { groundDepth, type WorldStructure } from '../../core/protocol';
import type { SeenBird, ViewRect } from '../types';
import { buildModel, buildModelParts, type ModelTemplate } from './models';
import { windBend } from './nature';

export type BirdSpecies = 'blueTit' | 'chaffinch' | 'warbler' | 'blackbird' | 'collaredDove';

/** The model's nodes, in the order of the parts drawn. */
const PARTS = ['body', 'head', 'tail', 'wing.L', 'wing.R'] as const;
const BODY = 0;
const HEAD = 1;
const TAIL = 2;
const WING_L = 3;
const WING_R = 4;

/** How a species looks and behaves. Times in the birds' seconds, distances in metres, angles in radians. */
interface Species {
  id: BirdSpecies;
  /** Its song (`audio/birdsong.ts` species). */
  song: string;
  /** Size when the manifest doesn't say (`bird.length`, `bird.wingspan`). */
  length: number;
  wingspan: number;
  /** `bounding`: flap bursts with shut wings between (an undulating line); `direct`: low and straight, short glides; `steady`: even flapping with glides. */
  flight: 'bounding' | 'direct' | 'steady';
  /** Cruising speed (m/s) and wingbeats a second (kept low enough to read at 60 fps). */
  speed: number;
  flapHz: number;
  /** Seconds between glances, and how fast the head snaps round (1/s). */
  glance: [number, number];
  snap: number;
  /** Seconds between tail flicks (null: none) and how far the tail flicks up. */
  flick: [number, number] | null;
  flickAmp: number;
  preen: [number, number];
  /** Seconds between hops within its plant and between flights away, at the busiest time of day. */
  hop: [number, number];
  fly: [number, number];
  /** Share of its flights that go down to the lawn, and how long it stays there. */
  ground: number;
  groundStay: [number, number];
  gait: 'hop' | 'run' | 'walk';
  /** Share of its flights to the roof ridge. */
  roof: number;
  /** How close (m) a resident may come before it flies up (on the lawn; half that perched low). */
  wary: number;
  /** Hours it wakes and goes to roost. */
  wake: number;
  roost: number;
  /** Lives in pairs (the second follows the first about). */
  pair: boolean;
  /** Where in its plant it likes to sit: 0 low, 1 the top. */
  height: number;
  /** Where it lives: weights in broadleaved trees, conifers and bushes. */
  habitat: [number, number, number];
  /** The stand-in's colours (`?debug&birdStandIn`): body, head, wings, tail, beak. */
  standIn: [string, string, string, string, string];
}

const SPECIES: Species[] = [
  {
    id: 'blueTit', song: 'tit', length: 0.115, wingspan: 0.19, flight: 'bounding', speed: 5.5, flapHz: 15,
    glance: [0.25, 1.0], snap: 40, flick: [1, 3], flickAmp: 0.35, preen: [20, 50], hop: [1.5, 5], fly: [12, 35],
    ground: 0, groundStay: [0, 0], gait: 'hop', roof: 0, wary: 2.5, wake: 5.2, roost: 20.4, pair: false, height: 0.6,
    habitat: [3, 2, 2], standIn: ['#9DB35A', '#3F7CC8', '#4B78B8', '#3E5F9C', '#2A2A2A'],
  },
  {
    id: 'chaffinch', song: 'finch', length: 0.15, wingspan: 0.26, flight: 'bounding', speed: 6.5, flapHz: 13,
    glance: [0.4, 1.6], snap: 30, flick: [2, 6], flickAmp: 0.25, preen: [25, 60], hop: [3, 9], fly: [15, 45],
    ground: 0.45, groundStay: [10, 40], gait: 'hop', roof: 0, wary: 3, wake: 5.1, roost: 20.5, pair: false, height: 0.5,
    habitat: [3, 2, 1.5], standIn: ['#C47A5A', '#6A7E99', '#5A4A3A', '#4A4A44', '#B8B8B0'],
  },
  {
    id: 'warbler', song: 'warbler', length: 0.12, wingspan: 0.19, flight: 'bounding', speed: 5, flapHz: 15,
    glance: [0.2, 0.8], snap: 40, flick: [1, 3], flickAmp: 0.3, preen: [20, 40], hop: [0.8, 3], fly: [10, 30],
    ground: 0, groundStay: [0, 0], gait: 'hop', roof: 0, wary: 2.5, wake: 5.1, roost: 20.5, pair: false, height: 0.45,
    habitat: [2, 0.5, 2.5], standIn: ['#A9A86A', '#9C9B60', '#827F52', '#7E7B4E', '#6A5A40'],
  },
  {
    id: 'blackbird', song: 'blackbird', length: 0.25, wingspan: 0.36, flight: 'direct', speed: 7, flapHz: 11,
    glance: [0.6, 2.2], snap: 22, flick: [3, 8], flickAmp: 0.5, preen: [30, 80], hop: [6, 15], fly: [15, 40],
    ground: 0.65, groundStay: [15, 60], gait: 'run', roof: 0, wary: 3.5, wake: 4.9, roost: 21, pair: false, height: 0.25,
    habitat: [1.5, 1, 3], standIn: ['#1E1E20', '#1E1E20', '#18181A', '#151517', '#E8A020'],
  },
  {
    id: 'collaredDove', song: 'dove', length: 0.32, wingspan: 0.5, flight: 'steady', speed: 9, flapHz: 7,
    glance: [1.5, 5], snap: 8, flick: null, flickAmp: 0, preen: [40, 120], hop: [15, 40], fly: [30, 90],
    ground: 0.4, groundStay: [20, 70], gait: 'walk', roof: 0.35, wary: 4, wake: 5.6, roost: 20.1, pair: true, height: 0.85,
    habitat: [2, 1.5, 0.2], standIn: ['#C9B9A6', '#C2B2A0', '#B5A390', '#A89684', '#3A3A3A'],
  },
];

/**
 * How much faster the birds' clock runs at each game speed (`GAME_RATE`'s index): they keep to the
 * game (still while it's paused) but don't go frantic in a time-lapse.
 */
const RATE = [0, 1, 1.25, 1.5, 1.75, 1.75];
/** Longest step of the birds' clock in a frame (s). */
const MAX_STEP = 0.1;
/** Clearance kept over roofs and walls in flight (m), and round them (m). */
const CLEAR = 0.7;
const CLEAR_SIDE = 0.45;
/** Height of a house above its storeys' walls, for one whose roof isn't known (m). */
const ROOF_RISE = 3.4;
const STOREY = 3;
/** A flight shorter than this is a hop (m). */
const HOP_MAX = 1.6;
/** The camera this close (m) scares a bird up. */
const CAMERA_WARY = 1.8;

/** A tree, bush or hedge as the renderer placed it: its model, where it stands, its scale (x, up, z) and turn. */
export interface PlantPlacement {
  key: string;
  x: number;
  y: number;
  z: number;
  sx: number;
  sy: number;
  sz: number;
  yaw: number;
}

/** Where the birds live (`Birds.setHabitat`). */
export interface BirdHabitat {
  world: WorldStructure;
  view: ViewRect | null;
  plants: PlantPlacement[];
  /** Points along the viewed house's roof ridge (x, y, z each), where doves sit while the roof shows. */
  ridge: number[];
  /** Top of the viewed house's roof (m), if known. */
  roofTop: number | null;
}

/** The birds' preset knob (`QualitySettings.birds`). */
export interface BirdQuality {
  /** Most birds about. */
  count: number;
  /** How far from the viewed lot (m) their home plants may be. */
  range: number;
}

/** What `Birds.update` needs of a frame. */
export interface BirdFrame {
  /** Real seconds since the last frame, and the game's speed (`GAME_RATE` index). */
  dt: number;
  speed: number;
  hour: number;
  camera: Vector3;
  /** Residents' heads (x, y, z each), how many, and which are drawn. */
  residents: Float32Array;
  residentCount: number;
  visible: Uint8Array;
  /** Reduce motion. */
  calm: boolean;
  /** The viewed house's roof is drawn (doves may sit on its ridge). */
  roof: boolean;
}

interface Plant {
  key: string;
  kind: 'tree' | 'bush' | 'roof';
  conifer: boolean;
  x: number;
  y: number;
  z: number;
  /** Perch points at rest (x, y, z each, world) and each one's height in its model (for the wind's bend; 0: still). */
  perches: Float32Array;
  heights: Float32Array;
  top: number;
  radius: number;
  /** Its perches deepest in the crown first, where its birds roost. */
  roosts: number[];
  /** Fixed by where it stands. */
  seed: number;
}

type State = 'perch' | 'fly' | 'ground' | 'roost';

interface Bird {
  /** Fixed by its home plant's position and its place among the plant's birds. */
  key: string;
  id: number;
  sp: Species;
  /** Index of its species' look (`looks`), and its slot in that look's buffers. */
  look: number;
  slot: number;
  size: number;
  home: number;
  partner: Bird | null;
  /** Its own hours (a little off its species'), and the perch of its home it roosts on. */
  wake: number;
  roost: number;
  roostPerch: number;
  state: State;
  /** The perch it's on or flying to (plant, perch index; -1: the lawn). */
  plant: number;
  perch: number;
  x: number;
  y: number;
  z: number;
  yaw: number;
  pitch: number;
  roll: number;
  /** Head yaw, pitch, roll and bob (m forward), and where it's turning to. */
  hy: number;
  hp: number;
  hr: number;
  bob: number;
  hyTo: number;
  hpTo: number;
  hrTo: number;
  tail: number;
  /** Wings: how far spread (0: shut, the spread wings hidden), the beat (up +) and its phase. */
  spread: number;
  flap: number;
  phase: number;
  puff: number;
  /** When it next glances, flicks its tail, preens, hops and flies off (birds' clock, s); preening until; tail flick start. */
  nextGlance: number;
  nextFlick: number;
  nextPreen: number;
  preenUntil: number;
  flickAt: number;
  nextHop: number;
  nextFly: number;
  /** Sings until (birds' clock), as the world's sound says. */
  singUntil: number;
  /** Keeps off the lawn and low perches until. */
  waryUntil: number;
  /** Flight: control points (p0, c1, c2, p1: x, y, z each), duration, time flown, bounds, its kind. */
  path: Float32Array;
  dur: number;
  t: number;
  bounds: number;
  bound: number;
  hop: boolean;
  /** Ground: when it leaves, its next move, and the move under way (from, to, time, length). */
  leaveAt: number;
  nextStep: number;
  step: Float32Array;
  stepT: number;
  stepDur: number;
  steps: number;
  peck: number;
  /** Where it was last frame (its heading and climb in flight). */
  px: number;
  py: number;
  pz: number;
  /** Its matrices need writing. */
  dirty: boolean;
  shown: boolean;
}

/** A species' model as drawn: its parts' meshes and their buffers, and the nodes' pivots. */
interface Look {
  sp: Species;
  meshes: Mesh[][];
  buffers: Float32Array[];
  pivots: Vector3[];
  /** Feet to the body's middle (m, model) and the perched body's nose-up angle (rad). */
  centre: number;
  pitch: number;
  length: number;
  capacity: number;
  count: number;
  /** A part's buffer was written this frame. */
  dirty: boolean;
  /** The model's file (to build again when it comes after failing), or the stand-in. */
  url: string | null;
}

const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const between = (r: () => number, [a, b]: readonly [number, number]) => a + r() * (b - a);
const rnd = Math.random;
/** Shortest turn from angle `a` to `b`. */
const turn = (a: number, b: number) => {
  let d = (b - a) % (Math.PI * 2);
  if (d > Math.PI) d -= Math.PI * 2;
  if (d < -Math.PI) d += Math.PI * 2;
  return d;
};

function hash(x: number, z: number, salt: number): number {
  let h = (Math.imul(Math.round(x * 10) | 0, 374761393) + Math.imul(Math.round(z * 10) | 0, 668265263) + Math.imul(salt, 2147483647)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

function seeded(seed: number): () => number {
  let a = Math.floor(seed * 4294967296) >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** How busy the birds are at an hour (0..1): most at dawn and in the early evening, least round midday. */
function activity(hour: number): number {
  const dawn = Math.exp(-(((hour - 6.5) / 1.6) ** 2));
  const evening = Math.exp(-(((hour - 18) / 1.4) ** 2));
  const midday = Math.exp(-(((hour - 13) / 1.8) ** 2));
  return Math.min(1, 0.5 + 0.5 * Math.max(dawn, 0.85 * evening) - 0.2 * midday);
}

type BirdExtras = ModelEntry & { perches?: Vec3[]; bird?: { length?: number; wingspan?: number; perchHeight?: number; pitch?: number } };

/** A model entry's own placement (`scale`, `rotationY`, `offset`) applied to a point of its model. */
function inSpace(entry: ModelEntry, [x, y, z]: Vec3, out: Vector3): Vector3 {
  const s = entry.scale ?? 1;
  const a = ((entry.rotationY ?? 0) * Math.PI) / 180;
  const [c, n] = [Math.cos(a), Math.sin(a)];
  const [ox, oy, oz] = entry.offset ?? [0, 0, 0];
  return out.set((x * c + z * n) * s + ox, y * s + oy, (-x * n + z * c) * s + oz);
}

export class Birds {
  private birds: Bird[] = [];
  private plants: Plant[] = [];
  private readonly looks: (Look | null)[] = SPECIES.map(() => null);
  private readonly building: (Promise<Look | null> | null)[] = SPECIES.map(() => null);
  private quality: BirdQuality = { count: 0, range: 0 };
  private habitat: BirdHabitat | null = null;
  /** The town's ground tiles: 1 where no bird forages (paths, rooms, patios, furniture, houses). */
  private blocked = new Uint8Array(0);
  /** The houses: x0, z0, x1, z1 (their walls) and the roof's top (m) each. */
  private boxes = new Float32Array(0);
  private width = 0;
  private depth = 0;
  private clock = 0;
  /** Birds' seconds per real second now (`RATE`), and a slow-motion factor for checks (`?debug`'s `__birdTime`). */
  private rate = 1;
  timeScale = 1;
  private hour = 12;
  private calm = false;
  private roofOn = false;
  private visible = true;
  private ticket = 0;
  /** Smoothed main-thread time of `update` (ms), and what the birds did, for `__birds`: flights, hops, visits to the lawn and the roof, scares (by a resident, the camera), and frames a bird in flight was inside a house's walls or roof. */
  private cost = 0;
  private readonly debugInside: unknown[] = [];
  private readonly stats = { flights: 0, hops: 0, lawn: 0, roof: 0, scaredBy: 0, scaredCamera: 0, inside: 0 };
  private readonly seen: SeenBird[] = [];
  /** Where the camera was last frame. */
  private readonly eye = new Vector3();
  /** Perch points made up from a plant model's leaves for the stand-in, by model key (`?debug&birdStandIn`). */
  private readonly madePerches = new Map<string, Promise<Vec3[]>>();

  // Scratch, so frames allocate nothing.
  private readonly bend = { x: 0, z: 0 };
  private readonly q = new Quaternion();
  private readonly rot = new Matrix();
  private readonly world = new Matrix();
  private readonly pose = new Float32Array(16);
  private readonly base = new Float32Array(16);
  private readonly vS = new Vector3();
  private readonly vT = new Vector3();
  private readonly v = new Vector3();
  private readonly at = new Float32Array(3);

  constructor(
    private readonly scene: Scene,
    private readonly assets: AssetRegistry,
    /** `?debug&birdStandIn`: species without a model drawn as plain shapes, plants without perches given some (testing only). */
    private readonly standIn = false,
  ) {}

  setQuality(quality: BirdQuality): void {
    this.quality = quality;
    if (this.habitat) void this.setHabitat(this.habitat);
  }

  /** Hidden while the basement is in view (the garden is dug away) or behind the overview. */
  setVisible(visible: boolean): void {
    if (visible === this.visible) return;
    this.visible = visible;
    for (const look of this.looks) for (const list of look?.meshes ?? []) for (const mesh of list) mesh.setEnabled(visible && look!.count > 0);
  }

  /** A model file that had failed has come: a bird model built without it is built again. */
  fileArrived(url: string): void {
    const i = this.looks.findIndex((l) => l?.url === url);
    if (i < 0 || !this.habitat) return;
    this.dropLook(i);
    void this.setHabitat(this.habitat);
  }

  /**
   * Where the birds live now (the viewed lot built, or anything on it changed): the plants within
   * range get their birds again; a bird whose home is still there carries on as it was.
   */
  async setHabitat(habitat: BirdHabitat): Promise<void> {
    const ticket = ++this.ticket;
    this.habitat = habitat;
    const { world, view } = habitat;
    this.grids(world, habitat.roofTop);
    const range = this.quality.range;
    const near = (x: number, z: number) => {
      if (!view) return false;
      const dx = Math.max(view.x - x, 0, x - (view.x + view.w));
      const dz = Math.max(view.z - z, 0, z - (view.z + view.d));
      return Math.hypot(dx, dz) <= range;
    };
    const placements = this.quality.count > 0 ? habitat.plants.filter((p) => near(p.x, p.z)) : [];
    const plants: Plant[] = [];
    for (const p of placements) {
      const plant = await this.plantOf(p);
      if (ticket !== this.ticket) return;
      if (plant) plants.push(plant);
    }
    if (habitat.ridge.length >= 3) {
      const perches = Float32Array.from(habitat.ridge);
      const n = perches.length / 3;
      const mid = Math.floor(n / 2);
      plants.push({ key: 'roof', kind: 'roof', conifer: false, x: perches[mid * 3], y: 0, z: perches[mid * 3 + 2], perches, heights: new Float32Array(n), top: perches[1], radius: 0, roosts: [mid], seed: hash(perches[0], perches[2], 7) });
    }
    // The species there may be (a model, or the stand-in).
    const available = SPECIES.map((sp) => this.standIn || this.assets.has(`model.bird.${sp.id}`, 'model'));
    const looks = await Promise.all(SPECIES.map((_, i) => (available[i] && plants.length ? this.lookOf(i) : Promise.resolve(null))));
    if (ticket !== this.ticket) return;
    const usable = looks.map((l) => !!l);
    const old = new Map(this.birds.map((b) => [b.key, b]));
    const birds: Bird[] = [];
    // Plants nearest the lot first, so a smaller preset keeps the birds nearest (always the same ones).
    const centre = view ? [view.x + view.w / 2, view.z + view.d / 2] : [0, 0];
    const order = plants.map((_, i) => i).filter((i) => plants[i].kind !== 'roof');
    order.sort((a, b) => Math.hypot(plants[a].x - centre[0], plants[a].z - centre[1]) - Math.hypot(plants[b].x - centre[0], plants[b].z - centre[1]));
    for (const i of order) {
      if (birds.length >= this.quality.count) break;
      const plant = plants[i];
      const r = seeded(plant.seed);
      // A big tree two or three, a small one one or two; a bush or a stretch of hedge now and then.
      const n = plant.kind === 'tree' ? (plant.top > 6 ? 2 + Math.floor(r() * 2) : 1 + Math.floor(r() * 2)) : r() < (plant.top > 1.2 ? 0.25 : 0.12) ? 1 : 0;
      let roosting = 0;
      const roostOf = () => plant.roosts[roosting++ % plant.roosts.length];
      for (let k = 0; k < n && birds.length < this.quality.count; k++) {
        const sp = this.pickSpecies(plant, r(), usable);
        if (!sp) break;
        const key = `${Math.round(plant.x * 10)},${Math.round(plant.z * 10)},${k}`;
        const first = this.makeBird(old.get(key), key, sp, i, plants, roostOf());
        birds.push(first);
        if (sp.pair && birds.length < this.quality.count) {
          const mate = this.makeBird(old.get(`${key}m`), `${key}m`, sp, i, plants, roostOf());
          mate.partner = first;
          first.partner = mate;
          birds.push(mate);
        }
      }
    }
    this.plants = plants;
    this.birds = birds;
    if (this.standIn) console.info(`[birds] ${plants.length} plants (${placements.length} placed), ${birds.length} birds: ${SPECIES.map((sp) => `${sp.id} ${birds.filter((b) => b.sp === sp).length}`).join(', ')}`);
    // Each species' buffers, sized to its birds.
    for (let i = 0; i < SPECIES.length; i++) {
      const look = looks[i];
      if (!look) continue;
      const mine = birds.filter((b) => b.sp === SPECIES[i]);
      mine.forEach((b, slot) => {
        b.look = i;
        b.slot = slot;
        b.dirty = true;
      });
      this.size(look, mine.length);
    }
    for (let i = 0; i < SPECIES.length; i++) if (this.looks[i] && !birds.some((b) => b.look === i)) this.size(this.looks[i]!, 0);
  }

  /** The birds as the world's sound hears them: where each is, and whether it may sing (perched, awake). */
  heard(): readonly SeenBird[] {
    const out = this.seen;
    out.length = this.birds.length;
    for (let i = 0; i < this.birds.length; i++) {
      const b = this.birds[i];
      const s = (out[i] ??= { id: 0, song: '', x: 0, y: 0, z: 0, perched: false, distance: 0 });
      s.id = b.id;
      s.song = b.sp.song;
      s.x = b.x;
      s.y = b.y + 0.1;
      s.z = b.z;
      s.perched = this.visible && b.state === 'perch' && b.preenUntil < this.clock;
      s.distance = Math.hypot(b.x - this.eye.x, b.y - this.eye.y, b.z - this.eye.z);
    }
    return out;
  }

  /** Bird `id` sings for `seconds`: it lifts its head and holds still meanwhile. */
  sings(id: number, seconds: number): void {
    const b = this.birds.find((x) => x.id === id);
    if (b) b.singUntil = this.clock + seconds * this.rate;
  }

  /** `?debug`'s `__birds()`: each bird's species, state and place, and the cost of a frame. */
  debug(): { ms: number; stats: Record<string, number>; inside: unknown[]; birds: { id: number; species: BirdSpecies; state: string; x: number; y: number; z: number; home: string; drawn: boolean }[] } {
    return {
      ms: Math.round(this.cost * 1000) / 1000,
      stats: { ...this.stats },
      inside: this.debugInside,
      birds: this.birds.map((b) => ({
        id: b.id,
        species: b.sp.id,
        state: b.state === 'fly' ? (b.hop ? 'hop' : 'fly') : b.state,
        x: Math.round(b.x * 100) / 100,
        y: Math.round(b.y * 100) / 100,
        z: Math.round(b.z * 100) / 100,
        home: this.plants[b.home]?.key ?? '',
        drawn: this.visible && !!this.looks[b.look],
      })),
    };
  }

  /** `?debug`'s `__birdFly(id, where)`: sends a bird off now (to another plant, the lawn or the roof), for checks. */
  debugFly(id: number, where: 'plant' | 'ground' | 'roof' | 'home'): boolean {
    const b = this.birds.find((x) => x.id === id);
    if (!b || b.state === 'fly') return false;
    if (where === 'ground') {
      const spot = this.lawnSpot(b, this.plants[b.home]);
      if (spot) this.flyDown(b, spot[0], spot[1], 1);
    } else if (where === 'roof') {
      const roof = this.plants.findIndex((p) => p.kind === 'roof');
      if (roof >= 0) this.flyTo(b, roof, Math.floor(rnd() * (this.plants[roof].perches.length / 3)), 1);
    } else if (where === 'home') this.flyHome(b);
    else this.flyOff(b, true);
    return (b.state as State) === 'fly';
  }

  /** Moves the birds a frame on; true if any moved (the snapshot is recorded again). */
  update(f: BirdFrame): boolean {
    if (!this.birds.length) return false;
    const started = performance.now();
    this.rate = (RATE[f.speed | 0] ?? RATE[RATE.length - 1]) * this.timeScale;
    const dt = Math.min(MAX_STEP, f.dt * this.rate);
    this.clock += dt;
    this.hour = f.hour;
    this.calm = f.calm;
    this.eye.copyFrom(f.camera);
    if (f.roof !== this.roofOn) this.roofOn = f.roof;
    let moved = false;
    for (const b of this.birds) {
      b.dirty = false;
      if (dt > 0) this.think(b, f);
      if (b.state === 'perch' || b.state === 'roost') this.sit(b, dt);
      else if (b.state === 'fly') this.fly(b, dt);
      else this.forage(b, dt);
      if (b.dirty && this.visible) {
        this.write(b);
        moved = true;
      }
    }
    for (const look of this.looks) {
      if (!look?.dirty) continue;
      look.dirty = false;
      for (const list of look.meshes) for (const mesh of list) mesh.thinInstanceBufferUpdated('matrix');
    }
    this.cost += (performance.now() - started - this.cost) * 0.05;
    return moved;
  }

  dispose(): void {
    this.ticket++;
    for (let i = 0; i < this.looks.length; i++) this.dropLook(i);
    this.birds = [];
    this.plants = [];
  }

  // ---- habitat ------------------------------------------------------------------------------

  /** The town's lawn mask per ground tile, and the houses as boxes. */
  private grids(world: WorldStructure, roofTop: number | null): void {
    const w = (this.width = world.width);
    const d = (this.depth = groundDepth(world));
    const blocked = (this.blocked = new Uint8Array(w * d));
    const fill = (x0: number, z0: number, x1: number, z1: number) => {
      for (let z = Math.max(0, Math.floor(z0)); z < Math.min(d, Math.ceil(z1)); z++) for (let x = Math.max(0, Math.floor(x0)); x < Math.min(w, Math.ceil(x1)); x++) blocked[z * w + x] = 1;
    };
    for (let i = 0; i < w * d; i++) if (world.rooms[i] !== 0) blocked[i] = 1;
    for (const r of [...(world.meta?.streets ?? []), ...(world.meta?.paths ?? []), ...(world.meta?.world?.fields ?? [])]) fill(r.x, r.z, r.x + r.w, r.z + r.d);
    for (const [x, z] of world.floors ?? []) if (z < d) blocked[z * w + x] = 1;
    for (const o of world.objects) if (o.z < d) fill(o.x, o.z, o.x + o.w, o.z + o.d);
    // Houses: their walls up to their roofs' tops (the viewed one's roof as built, if known).
    const boxes: number[] = [];
    const view = this.habitat?.view;
    for (const p of world.plots) {
      if (!p.house) continue;
      const [x0, z0, x1, z1] = p.house;
      const viewed = !!view && p.x === view.x && p.z === view.z;
      boxes.push(x0, z0, x1, z1, viewed && roofTop !== null ? roofTop : (p.storeys ?? 1) * STOREY + ROOF_RISE);
      fill(x0 - 0.3, z0 - 0.3, x1 + 0.3, z1 + 0.3);
    }
    // The viewed house as it is now (rooms built past its box).
    if (view) {
      let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
      for (let z = view.z; z < Math.min(d, view.z + view.d); z++) {
        for (let x = view.x; x < Math.min(w, view.x + view.w); x++) {
          if (world.rooms[z * w + x] === 0) continue;
          [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x + 1), Math.max(z1, z + 1)];
        }
      }
      if (x1 > x0) boxes.push(x0, z0, x1, z1, roofTop ?? (world.storeys ?? 1) * STOREY + ROOF_RISE);
    }
    this.boxes = Float32Array.from(boxes);
  }

  /** Whether a bird may stand on the ground here (lawn in the town, clear of everything). */
  private lawn(x: number, z: number): boolean {
    const m = 0.18;
    for (const [dx, dz] of [[0, 0], [m, m], [-m, m], [m, -m], [-m, -m]] as const) {
      const tx = Math.floor(x + dx);
      const tz = Math.floor(z + dz);
      if (tx < 0 || tz < 0 || tx >= this.width || tz >= this.depth || this.blocked[tz * this.width + tx]) return false;
    }
    return true;
  }

  /**
   * The top of the house a point is in (m; 0 in the open), its walls widened by `margin` (m) but for
   * the houses `except` says (bit per box: one a flight starts or ends right by).
   */
  private topAt(x: number, z: number, margin = 0, except = 0): number {
    const B = this.boxes;
    let top = 0;
    for (let i = 0, k = 0; i < B.length; i += 5, k++) {
      const m = except & (1 << k) ? 0 : margin;
      if (x > B[i] - m && x < B[i + 2] + m && z > B[i + 1] - m && z < B[i + 3] + m && B[i + 4] > top) top = B[i + 4];
    }
    return top;
  }

  /** The houses (bits, as `topAt`'s `except`) within `margin` of a point. */
  private housesBy(x: number, z: number, margin: number): number {
    const B = this.boxes;
    let bits = 0;
    for (let i = 0, k = 0; i < B.length && k < 31; i += 5, k++) if (x > B[i] - margin && x < B[i + 2] + margin && z > B[i + 1] - margin && z < B[i + 3] + margin) bits |= 1 << k;
    return bits;
  }

  /** A placed plant's perches in the world, if its model has any. */
  private async plantOf(p: PlantPlacement): Promise<Plant | null> {
    const entry = this.assets.get(p.key, 'model') as BirdExtras | undefined;
    if (!entry) return null;
    let points = entry.perches;
    // (Made-up ones are from the model as built: in its entry's placement already.)
    const made = !points?.length;
    if (made && this.standIn && /(^|\.)(tree|bush|hedge|spruce|pine|cypress|willow|rosebush|hydrangea|boxwood|lavender)/i.test(p.key)) points = await this.makePerches(p.key, entry);
    if (!points?.length) return null;
    const perches = new Float32Array(points.length * 3);
    const heights = new Float32Array(points.length);
    const [c, s] = [Math.cos(p.yaw), Math.sin(p.yaw)];
    let top = 0;
    let radius = 0;
    const depth: number[] = [];
    points.forEach((point, i) => {
      const m = made ? this.v.set(point[0], point[1], point[2]) : inSpace(entry, point, this.v);
      const [lx, ly, lz] = [m.x * p.sx, m.y * p.sy, m.z * p.sz];
      perches[i * 3] = p.x + lx * c + lz * s;
      perches[i * 3 + 1] = p.y + ly;
      perches[i * 3 + 2] = p.z - lx * s + lz * c;
      heights[i] = m.y;
      top = Math.max(top, ly);
      const out = Math.hypot(lx, lz);
      radius = Math.max(radius, out);
      // Roost deep in the crown, well up.
      depth.push(out - ly * 0.3);
    });
    const roosts = points.map((_, i) => i).sort((a, b) => depth[a] - depth[b]);
    const conifer = /spruce|pine|cypress|fir/i.test(p.key);
    const tree = /(^|\.)(tree|spruce|pine|cypress|willow|palm)/i.test(p.key) || top > 2.6;
    return { key: p.key, kind: tree ? 'tree' : 'bush', conifer, x: p.x, y: p.y, z: p.z, perches, heights, top, radius, roosts, seed: hash(p.x, p.z, 0xb1d) };
  }

  /**
   * Stand-in perches for a plant model without any (testing only, `?debug&birdStandIn`): points on
   * the top side of its leaves, spread out.
   */
  private makePerches(key: string, entry: ModelEntry): Promise<Vec3[]> {
    let made = this.madePerches.get(key);
    if (!made) {
      made = buildModel(this.scene, `birds:perches:${key}`, entry, [1, 1]).then((t) => {
        // The leaves' outer shell: the highest point in each 30 cm column, and the outermost in each
        // sector and band of height (upper part of the plant only).
        const tops = new Map<string, Vec3>();
        const sides = new Map<string, Vec3>();
        for (const mesh of t.meshes) {
          const leafy = /foliage|leaf|needle|hedge|bush|crown/i.test(mesh.material?.name ?? '');
          const pos = mesh.getVerticesData(VertexBuffer.PositionKind);
          if (leafy && pos) {
            for (let i = 0; i < pos.length; i += 3) {
              const [x, y, z] = [pos[i], pos[i + 1], pos[i + 2]];
              if (y < t.height * 0.35) continue;
              const cell = `${Math.floor(x / 0.3)},${Math.floor(z / 0.3)}`;
              if (!tops.has(cell) || tops.get(cell)![1] < y) tops.set(cell, [x, y, z]);
              if (y < t.height * 0.6) continue;
              const band = `${Math.floor(((Math.atan2(z, x) + Math.PI) / (Math.PI * 2)) * 12)},${Math.floor(y / 0.5)}`;
              const was = sides.get(band);
              if (!was || Math.hypot(was[0], was[2]) < Math.hypot(x, z)) sides.set(band, [x, y, z]);
            }
          }
          mesh.dispose(false, false);
        }
        for (const g of t.glows ?? []) g.mesh.dispose(false, false);
        // (The upper half of them: on the crown, not down between its lumps.)
        const all = [...tops.values(), ...sides.values()].sort((a, b) => b[1] - a[1]);
        // (A little over the highest corners: the faces between them stand higher.)
        const points = all.slice(0, Math.ceil(all.length / 2)).map(([x, y, z]): Vec3 => [x, y + 0.05, z]);
        // Farthest-point sampling: a dozen spread over the crown.
        const picked: Vec3[] = [];
        if (points.length) picked.push(points.reduce((a, b) => (b[1] > a[1] ? b : a)));
        while (picked.length < Math.min(14, points.length)) {
          let best = points[0];
          let far = -1;
          for (const p of points) {
            const d = Math.min(...picked.map((q) => Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2])));
            if (d > far) [far, best] = [d, p];
          }
          picked.push(best);
        }
        return picked;
      });
      this.madePerches.set(key, made);
    }
    return made;
  }

  private pickSpecies(plant: Plant, roll: number, usable: boolean[]): Species | null {
    const h = plant.kind === 'bush' ? 2 : plant.conifer ? 1 : 0;
    let sum = 0;
    for (let i = 0; i < SPECIES.length; i++) if (usable[i]) sum += SPECIES[i].habitat[h];
    let at = roll * sum;
    for (let i = 0; i < SPECIES.length; i++) {
      if (!usable[i]) continue;
      at -= SPECIES[i].habitat[h];
      if (at <= 0) return SPECIES[i];
    }
    return null;
  }

  private makeBird(old: Bird | undefined, key: string, sp: Species, home: number, plants: Plant[], roostPerch: number): Bird {
    let id = 7;
    for (let i = 0; i < key.length; i++) id = (Math.imul(id, 31) + key.charCodeAt(i)) >>> 0;
    // Its own randoms (the plant's only choose how many live there, and which).
    const r = seeded(id / 4294967296);
    const size = 0.94 + r() * 0.12;
    const wake = sp.wake + (r() - 0.5) * 0.5;
    const roost = sp.roost + (r() - 0.5) * 0.5;
    if (old && old.sp === sp) {
      // Carries on where it is: on the same plant if that's still there (found by where it stands), else home.
      old.home = home;
      old.roostPerch = roostPerch;
      if (old.plant >= 0 || old.state === 'fly') {
        const was = this.plants[old.plant];
        const same = was ? plants.findIndex((p) => p.kind === was.kind && Math.abs(p.x - was.x) < 0.05 && Math.abs(p.z - was.z) < 0.05) : -1;
        old.plant = same >= 0 ? same : home;
        old.perch = Math.min(old.perch < 0 ? 0 : old.perch, plants[old.plant].perches.length / 3 - 1);
        if (same < 0) old.perch = old.state === 'roost' ? roostPerch : this.perchFor(old, plants[home]);
        if (old.state === 'fly' && same < 0) old.state = 'perch';
        if (old.state === 'ground') old.state = 'perch';
      } else if (!this.lawn(old.x, old.z)) {
        old.plant = home;
        old.perch = this.perchFor(old, plants[home]);
        old.state = 'perch';
      }
      old.partner = null;
      old.dirty = true;
      return old;
    }
    const plant = plants[home];
    const b: Bird = {
      key, id, sp, look: -1, slot: 0, size, home, partner: null, wake, roost, roostPerch,
      state: 'perch', plant: home, perch: 0, x: plant.x, y: plant.y, z: plant.z, yaw: r() * Math.PI * 2, pitch: 0, roll: 0,
      hy: 0, hp: 0, hr: 0, bob: 0, hyTo: 0, hpTo: 0, hrTo: 0, tail: 0, spread: 0, flap: 0, phase: 0, puff: 1,
      nextGlance: 0, nextFlick: r() * 3, nextPreen: between(r, sp.preen), preenUntil: -1, flickAt: -10, nextHop: between(r, sp.hop), nextFly: between(r, sp.fly),
      singUntil: -1, waryUntil: -1, path: new Float32Array(12), dur: 0, t: 0, bounds: 0, bound: 0, hop: false,
      leaveAt: 0, nextStep: 0, step: new Float32Array(4), stepT: 0, stepDur: 0, steps: 0, peck: -1, px: plant.x, py: plant.y, pz: plant.z,
      dirty: true, shown: false,
    };
    // Asleep on its roost when the game starts at night.
    const asleep = !this.awake(b);
    b.perch = asleep ? roostPerch : this.perchFor(b, plant, r);
    if (asleep) b.state = 'roost';
    this.perchPoint(plant, b.perch, this.at);
    [b.x, b.y, b.z] = [this.at[0], this.at[1], this.at[2]];
    return b;
  }

  // ---- behaviour ----------------------------------------------------------------------------

  private awake(b: Bird): boolean {
    return this.hour >= b.wake && this.hour < b.roost;
  }

  /** Decides what a bird does next. */
  private think(b: Bird, f: BirdFrame): void {
    const now = this.clock;
    const awake = this.awake(b);
    if (b.state === 'roost') {
      if (awake) {
        b.state = 'perch';
        // Wakes with a shake and a preen.
        b.preenUntil = now + 1.5 + rnd() * 2;
        b.nextFly = now + between(rnd, b.sp.fly) * 0.5;
        b.dirty = true;
      }
      return;
    }
    if (b.state === 'fly') return;
    // Dusk: home to roost.
    if (!awake) {
      if (b.state === 'perch' && b.plant === b.home && b.perch === b.roostPerch) {
        b.state = 'roost';
        b.dirty = true;
      } else this.flyTo(b, b.home, b.roostPerch, 0.8);
      return;
    }
    if (this.scared(b, f)) return;
    // Someone else on the roof ridge's perch when the roof goes: off it.
    if (b.state === 'perch' && this.plants[b.plant]?.kind === 'roof' && !this.roofOn) {
      this.flyAway(b, b.x, b.z);
      return;
    }
    if (f.calm) {
      // Calm: whoever's on the lawn goes home, slowly; nobody else flies about.
      if (b.state === 'ground' && now >= b.leaveAt) this.flyTo(b, b.home, this.perchFor(b, this.plants[b.home]), 0.6);
      return;
    }
    const busy = activity(this.hour);
    // A dove's mate follows it about.
    const mate = b.partner;
    const follows = !!mate && b.key.endsWith('m') && mate.state !== 'fly';
    if (b.state === 'ground') {
      if (follows && mate!.state !== 'ground') b.leaveAt = Math.min(b.leaveAt, now + 0.5 + rnd());
      if (now >= b.leaveAt) {
        if (follows && mate!.plant >= 0) this.flyTo(b, mate!.plant, this.nearPerch(this.plants[mate!.plant], mate!.perch, b), 1);
        else this.flyHome(b);
      }
      return;
    }
    if (b.singUntil > now || b.preenUntil > now) return;
    if (follows && (mate!.plant !== b.plant || mate!.state === 'ground')) {
      if (b.nextFly > now + 1.5) b.nextFly = now + 0.4 + rnd() * 1.2;
    }
    if (now >= b.nextFly) {
      b.nextFly = now + between(rnd, b.sp.fly) / (0.4 + busy);
      if (follows) {
        if (mate!.state === 'ground') {
          const a = rnd() * Math.PI * 2;
          const r = 0.4 + rnd() * 0.5;
          this.flyDown(b, mate!.x + Math.cos(a) * r, mate!.z + Math.sin(a) * r, 0.7);
        } else this.flyTo(b, mate!.plant, this.nearPerch(this.plants[mate!.plant], mate!.perch, b), 1);
      } else this.flyOff(b);
      return;
    }
    if (now >= b.nextHop) {
      b.nextHop = now + between(rnd, b.sp.hop) / (0.5 + busy * 0.5);
      const plant = this.plants[b.plant];
      if (!plant || plant.kind === 'roof') {
        // Along the ridge a little, or a turn.
        if (plant) this.flyTo(b, b.plant, Math.max(0, Math.min(plant.perches.length / 3 - 1, b.perch + (rnd() < 0.5 ? -1 : 1))), 1);
        return;
      }
      const next = this.nearPerch(plant, b.perch, b);
      if (next !== b.perch) this.flyTo(b, b.plant, next, 1);
      else b.yaw += (rnd() - 0.5) * 2.4;
      b.dirty = true;
    }
  }

  /** A resident or the camera close: up into the nearest plant away from them. True if it fled. */
  private scared(b: Bird, f: BirdFrame): boolean {
    if (f.calm && b.state !== 'ground') return false;
    const low = b.state === 'ground' || b.y - this.groundY() < 2.2;
    let threat = -1;
    const reach = b.state === 'ground' ? b.sp.wary : b.sp.wary * 0.6;
    if (low) {
      for (let i = 0; i < f.residentCount; i++) {
        if (!f.visible[i]) continue;
        const [hx, hy, hz] = [f.residents[i * 3], f.residents[i * 3 + 1], f.residents[i * 3 + 2]];
        // Upstairs, or indoors behind the walls: not a threat.
        if (hy > 2.6 || !this.lawnOrOpen(hx, hz)) continue;
        if (Math.hypot(hx - b.x, hz - b.z) < reach) threat = i;
      }
    }
    const c = f.camera;
    const camera = Math.hypot(c.x - b.x, c.y - b.y, c.z - b.z) < CAMERA_WARY;
    if (threat < 0 && !camera) return false;
    const [tx, tz] = threat >= 0 ? [f.residents[threat * 3], f.residents[threat * 3 + 2]] : [c.x, c.z];
    if (threat >= 0) this.stats.scaredBy++;
    else this.stats.scaredCamera++;
    b.waryUntil = this.clock + 20 + rnd() * 40;
    this.flyAway(b, tx, tz);
    return true;
  }

  /** Outdoors (a resident there can scare the birds). */
  private lawnOrOpen(x: number, z: number): boolean {
    const tx = Math.floor(x);
    const tz = Math.floor(z);
    if (tx < 0 || tz < 0 || tx >= this.width || tz >= this.depth) return true;
    return this.topAt(x, z) === 0;
  }

  private groundY(): number {
    return 0;
  }

  /** Off to somewhere else: another plant, the lawn or the roof (by species and the time of day). */
  private flyOff(b: Bird, plantOnly = false): void {
    const sp = b.sp;
    const wary = b.waryUntil > this.clock;
    const roll = plantOnly ? 0.5 : rnd();
    if (!wary && !plantOnly && sp.ground > 0 && roll < sp.ground) {
      const spot = this.lawnSpot(b, this.plants[b.home]) ?? this.lawnSpot(b, this.plants[b.plant]);
      if (spot) {
        this.flyDown(b, spot[0], spot[1], 1);
        return;
      }
    }
    if (!plantOnly && sp.roof > 0 && this.roofOn && roll > 1 - sp.roof) {
      const roof = this.plants.findIndex((p) => p.kind === 'roof');
      if (roof >= 0) {
        this.flyTo(b, roof, Math.floor(rnd() * (this.plants[roof].perches.length / 3)), 1);
        return;
      }
    }
    // Another plant nearby (mostly back home when away), not too near nor far.
    const from = this.plants[b.plant] ?? this.plants[b.home];
    if (b.plant !== b.home && !plantOnly && rnd() < 0.55) {
      this.flyTo(b, b.home, this.perchFor(b, this.plants[b.home]), 1);
      return;
    }
    let pick = -1;
    let best = -Infinity;
    for (let i = 0; i < this.plants.length; i++) {
      const p = this.plants[i];
      if (i === b.plant || p.kind === 'roof') continue;
      if (p.kind === 'bush' && sp.height > 0.7) continue;
      const d = Math.hypot(p.x - from.x, p.z - from.z);
      if (d > 30) continue;
      const score = -Math.abs(d - 8) + rnd() * 8 - Math.hypot(p.x - this.plants[b.home].x, p.z - this.plants[b.home].z) * 0.25;
      if (score > best) [best, pick] = [score, i];
    }
    if (pick >= 0) this.flyTo(b, pick, this.perchFor(b, this.plants[pick]), 1);
  }

  /** Home from the lawn (or the nearest plant to it, if home is far). */
  private flyHome(b: Bird): void {
    const home = this.plants[b.home];
    let pick = b.home;
    if (Math.hypot(home.x - b.x, home.z - b.z) > 10) pick = this.nearestPlant(b.x, b.z, -1, b);
    this.flyTo(b, pick, this.perchFor(b, this.plants[pick]), 1);
  }

  /** Up and away from a threat at (tx, tz): the nearest plant at least 4 m from it, well up. */
  private flyAway(b: Bird, tx: number, tz: number): void {
    let pick = -1;
    let best = Infinity;
    for (let i = 0; i < this.plants.length; i++) {
      const p = this.plants[i];
      if (p.kind === 'roof' && !this.roofOn) continue;
      if (Math.hypot(p.x - tx, p.z - tz) < 4 + p.radius || p.top < 1.5) continue;
      const d = Math.hypot(p.x - b.x, p.z - b.z) - (i === b.home ? 2 : 0);
      if (d < best) [best, pick] = [d, i];
    }
    if (pick < 0) pick = b.home;
    const plant = this.plants[pick];
    // The highest perch there.
    let perch = 0;
    for (let k = 0; k < plant.perches.length / 3; k++) if (plant.perches[k * 3 + 1] > plant.perches[perch * 3 + 1] - rnd() * 0.6) perch = k;
    this.flyTo(b, pick, perch, 1.15);
  }

  private nearestPlant(x: number, z: number, except: number, b: Bird): number {
    let pick = b.home;
    let best = Infinity;
    for (let i = 0; i < this.plants.length; i++) {
      if (i === except || this.plants[i].kind === 'roof') continue;
      const d = Math.hypot(this.plants[i].x - x, this.plants[i].z - z);
      if (d < best) [best, pick] = [d, i];
    }
    return pick;
  }

  /** A perch of `plant` this bird likes (by its species' height; off low perches while wary). */
  private perchFor(b: Bird, plant: Plant, r: () => number = rnd): number {
    const n = plant.perches.length / 3;
    let pick = 0;
    let best = -Infinity;
    const wary = b.waryUntil > this.clock;
    for (let k = 0; k < n; k++) {
      const h = plant.top > 0 ? (plant.perches[k * 3 + 1] - plant.y) / plant.top : 0;
      const score = -Math.abs(h - (wary ? 1 : b.sp.height)) + r() * 0.5;
      if (score > best) [best, pick] = [score, k];
    }
    return pick;
  }

  /** A perch of `plant` near `perch` (within a hop), or `perch` itself. */
  private nearPerch(plant: Plant, perch: number, b: Bird): number {
    const p = plant.perches;
    const n = p.length / 3;
    let pick = perch;
    let best = Infinity;
    for (let k = 0; k < n; k++) {
      if (k === perch || this.taken(plant, k, b)) continue;
      const d = Math.hypot(p[k * 3] - p[perch * 3], (p[k * 3 + 1] - p[perch * 3 + 1]) * 1.5, p[k * 3 + 2] - p[perch * 3 + 2]);
      const score = d + rnd() * 0.6;
      if (d < HOP_MAX && score < best) [best, pick] = [score, k];
    }
    return pick;
  }

  /** Another bird sits on (or is flying to) that perch. */
  private taken(plant: Plant, perch: number, b: Bird): boolean {
    const i = this.plants.indexOf(plant);
    return this.birds.some((o) => o !== b && o.plant === i && o.perch === perch && o.state !== 'ground');
  }

  /** A spot on the lawn near `plant`, clear of people. */
  private lawnSpot(b: Bird, plant: Plant | undefined): [number, number] | null {
    if (!plant) return null;
    for (let k = 0; k < 10; k++) {
      const a = rnd() * Math.PI * 2;
      const r = plant.radius + 0.6 + rnd() * (b.sp.gait === 'walk' ? 5 : 3.5);
      const x = plant.x + Math.cos(a) * r;
      const z = plant.z + Math.sin(a) * r;
      if (this.lawn(x, z)) return [x, z];
    }
    return null;
  }

  private perchPoint(plant: Plant, perch: number, out: Float32Array): void {
    const p = plant.perches;
    const h = plant.heights[perch];
    if (h > 0) windBend(this.scene, plant.x, plant.z, h, this.bend);
    else this.bend.x = this.bend.z = 0;
    out[0] = p[perch * 3] + this.bend.x;
    out[1] = p[perch * 3 + 1];
    out[2] = p[perch * 3 + 2] + this.bend.z;
  }

  /** Sets off to a perch (speed: times the species' cruise). */
  private flyTo(b: Bird, plant: number, perch: number, speed: number): void {
    const target = this.plants[plant];
    if (!target) return;
    this.perchPoint(target, perch, this.at);
    if (!this.plan(b, this.at[0], this.at[1], this.at[2], speed, false)) return;
    b.plant = plant;
    b.perch = perch;
  }

  /** Sets off down to the lawn at (x, z). */
  private flyDown(b: Bird, x: number, z: number, speed: number): void {
    if (!this.lawn(x, z)) return;
    if (!this.plan(b, x, this.groundY(), z, speed, true)) return;
    b.plant = -1;
    b.perch = -1;
  }

  /**
   * The flight's curve from where the bird is to (x, y, z): out and up, then in to land, lifted
   * over any house in the way. False (it stays) if the end is inside one.
   */
  private plan(b: Bird, x: number, y: number, z: number, speed: number, down: boolean): boolean {
    const sp = b.sp;
    const d = Math.hypot(x - b.x, z - b.z);
    // Never to a spot inside a house (its roof ridge is on it).
    if (this.topAt(x, z) > y + 0.2) return false;
    const P = b.path;
    const hop = d < HOP_MAX && Math.abs(y - b.y) < 1.2;
    const [ux, uz] = d > 1e-3 ? [(x - b.x) / d, (z - b.z) / d] : [Math.sin(b.yaw), Math.cos(b.yaw)];
    let lift0: number;
    let lift1: number;
    if (hop) {
      lift0 = lift1 = 0.08 + d * 0.25;
    } else if (sp.flight === 'direct') {
      // Low and straight: just above the lawn if it can.
      lift0 = 0.3 + d * 0.03;
      lift1 = down ? 0.35 : 0.15;
    } else if (sp.flight === 'steady') {
      lift0 = 0.8 + d * 0.12;
      lift1 = down ? 1.2 : 0.4;
    } else {
      lift0 = 0.5 + d * 0.1;
      lift1 = down ? 0.8 : 0.25;
    }
    P[0] = b.x;
    P[1] = b.y;
    P[2] = b.z;
    P[3] = b.x + ux * d * 0.3;
    P[4] = b.y + lift0;
    P[5] = b.z + uz * d * 0.3;
    P[6] = x - ux * d * 0.3;
    P[7] = y + lift1;
    P[8] = z - uz * d * 0.3;
    P[9] = x;
    P[10] = y;
    P[11] = z;
    // Up over the houses: where the curve dips under a roof (with room to spare), both middle points
    // rise. A house it leaves or lands by (a shrub by its wall, its ridge) is kept to its walls.
    const by = this.housesBy(b.x, b.z, CLEAR_SIDE + 0.6) | this.housesBy(x, z, CLEAR_SIDE + 0.6);
    let raised = 0;
    for (let pass = 0; pass < 4; pass++) {
      let raise = 0;
      for (let k = 1; k < 16; k++) {
        const s = k / 16;
        this.bezier(P, s, this.at);
        const need = this.topAt(this.at[0], this.at[2], CLEAR_SIDE, by);
        if (need <= 0) continue;
        const w = 3 * s * (1 - s);
        if (this.at[1] < need + CLEAR) raise = Math.max(raise, (need + CLEAR - this.at[1]) / Math.max(0.2, w));
      }
      if (raise <= 0) break;
      P[4] += raise;
      P[7] += raise;
      raised += raise;
    }
    // Not straight up a wall and over: somewhere else instead (birds go round, or not at all).
    if (raised > Math.max(4, d * 0.6)) return false;
    const length = this.arc(P);
    const v = (hop ? 2.2 : sp.speed) * speed * (this.calm ? 0.6 : 1);
    b.dur = hop ? 0.22 + d * 0.16 : length / v + 0.35;
    b.t = 0;
    b.hop = hop;
    // Bounding flight: a bound every couple of metres.
    b.bounds = sp.flight === 'bounding' && !hop ? Math.max(1, Math.round(length / 2.2)) : sp.flight === 'direct' && !hop ? Math.max(1, Math.round(length / 3.5)) : 0;
    b.bound = 0;
    b.state = 'fly';
    if (hop) this.stats.hops++;
    else this.stats.flights++;
    if (down) this.stats.lawn++;
    else if (this.topAt(x, z) > 0) this.stats.roof++;
    b.singUntil = -1;
    b.preenUntil = -1;
    b.dirty = true;
    return true;
  }

  private bezier(P: Float32Array, s: number, out: Float32Array): void {
    const u = 1 - s;
    const [a, c, e, f] = [u * u * u, 3 * u * u * s, 3 * u * s * s, s * s * s];
    out[0] = a * P[0] + c * P[3] + e * P[6] + f * P[9];
    out[1] = a * P[1] + c * P[4] + e * P[7] + f * P[10];
    out[2] = a * P[2] + c * P[5] + e * P[8] + f * P[11];
  }

  private arc(P: Float32Array): number {
    let length = 0;
    let [x, y, z] = [P[0], P[1], P[2]];
    for (let k = 1; k <= 12; k++) {
      this.bezier(P, k / 12, this.at);
      length += Math.hypot(this.at[0] - x, this.at[1] - y, this.at[2] - z);
      [x, y, z] = [this.at[0], this.at[1], this.at[2]];
    }
    return length;
  }

  /** In flight: along the curve, wings beating, banking, landing. */
  private fly(b: Bird, dt: number): void {
    if (dt <= 0) return;
    const sp = b.sp;
    b.t = Math.min(b.dur, b.t + dt);
    const tau = b.t / b.dur;
    // Quick away, slowing to land.
    const s = 0.35 * tau + 0.65 * tau * tau * (3 - 2 * tau);
    const P = b.path;
    // The end moves with its branch.
    if (b.plant >= 0) {
      this.perchPoint(this.plants[b.plant], b.perch, this.at);
      P[9] = this.at[0];
      P[10] = this.at[1];
      P[11] = this.at[2];
    }
    this.bezier(P, s, this.at);
    b.px = b.x;
    b.py = b.y;
    b.pz = b.z;
    let [x, y, z] = [this.at[0], this.at[1], this.at[2]];
    // Bounding: rising while it flaps, falling with its wings shut.
    let flapping = true;
    if (b.bounds > 0) {
      const cycle = tau * b.bounds;
      const amp = (sp.flight === 'direct' ? 0.06 : 0.16) * Math.sin(Math.PI * tau);
      y += amp * Math.sin(cycle * Math.PI * 2);
      const rising = Math.cos(cycle * Math.PI * 2) > (sp.flight === 'direct' ? -0.6 : -0.1);
      flapping = rising || tau < 0.15 || tau > 0.82;
    }
    b.x = x;
    b.y = y;
    b.z = z;
    // (A check: never through a house, short of perching on its ridge.)
    if (y < this.topAt(x, z) - 0.3) {
      this.stats.inside++;
      if (this.debugInside.length < 20) this.debugInside.push({ species: b.sp.id, at: [x, y, z].map((v) => Math.round(v * 100) / 100), top: this.topAt(x, z), path: Array.from(b.path, (v) => Math.round(v * 100) / 100), hop: b.hop });
    }
    // Heading and bank from the way it goes.
    const vx = (b.x - b.px) / dt;
    const vy = (b.y - b.py) / dt;
    const vz = (b.z - b.pz) / dt;
    const vh = Math.hypot(vx, vz);
    if (vh > 0.2) {
      const want = Math.atan2(vx, vz);
      const dy = turn(b.yaw, want);
      const rate = b.hop ? 18 : 7;
      const step = dy * Math.min(1, dt * rate);
      b.yaw += step;
      b.roll = lerp(b.roll, Math.max(-0.7, Math.min(0.7, (-step / dt) * 0.12 * Math.min(1, vh / 4))), Math.min(1, dt * 6));
    }
    const flare = smooth(0.78, 1, tau) * (b.hop ? 0.4 : 1);
    const climb = -Math.atan2(vy, Math.max(0.5, vh)) * 0.5;
    const level = this.looks[b.look]?.pitch ?? 0;
    // Level in flight (the model sits nose-up), nose up to land.
    b.pitch = (b.hop ? level * 0.4 : level) * (1 - flare) + climb * (1 - flare) - flare * 0.65;
    // Wings: a burst to take off, beats (or shut, bounding), held out to glide, spread wide to land.
    const hz = Math.min(15, sp.flapHz * (tau < 0.15 ? 1.25 : 1));
    if (b.hop) {
      b.spread = b.t < b.dur * 0.85 ? 1 : 0;
      b.phase += dt * Math.PI * 2 * hz;
      b.flap = 0.3 + 0.8 * Math.sin(b.phase);
    } else if (flare > 0.05) {
      b.spread = 1;
      b.phase += dt * Math.PI * 2 * hz * 0.8;
      b.flap = 0.35 + 0.9 * Math.sin(b.phase) * (0.5 + 0.5 * flare);
    } else if (sp.flight === 'steady' && Math.sin(tau * b.dur * 1.6) > 0.55 && tau > 0.2) {
      // A dove's glide, wings held a little up.
      b.spread = 1;
      b.flap = lerp(b.flap, 0.18, Math.min(1, dt * 10));
    } else if (!flapping) {
      b.spread = 0;
      b.flap = 0;
    } else {
      b.spread = 1;
      b.phase += dt * Math.PI * 2 * hz;
      b.flap = 0.15 + 0.95 * Math.sin(b.phase);
    }
    b.hy = lerp(b.hy, 0, Math.min(1, dt * 12));
    b.hp = lerp(b.hp, -b.pitch * 0.5, Math.min(1, dt * 12));
    b.hr = 0;
    b.tail = lerp(b.tail, flare * 0.5 + (flapping ? 0.05 : -0.1), Math.min(1, dt * 10));
    b.puff = 1;
    b.dirty = true;
    if (b.t >= b.dur) this.land(b);
  }

  private land(b: Bird): void {
    b.pitch = 0;
    b.roll = 0;
    b.spread = 0;
    b.flap = 0;
    b.hp = 0;
    // Blackbirds raise the tail as they land; others flick it.
    b.flickAt = this.clock;
    if (b.plant >= 0) {
      b.state = 'perch';
      b.nextHop = this.clock + between(rnd, b.sp.hop);
    } else {
      b.state = 'ground';
      b.y = this.groundY();
      b.leaveAt = this.clock + between(rnd, b.sp.groundStay) * (b.waryUntil > this.clock ? 0.3 : 1);
      b.nextStep = this.clock + 0.4 + rnd();
      b.steps = 0;
      b.stepT = b.stepDur = 0;
    }
    b.dirty = true;
  }

  /** Perched or roosting: on its branch as it sways, glancing, flicking, preening, singing, asleep. */
  private sit(b: Bird, dt: number): void {
    const plant = this.plants[b.plant];
    if (!plant) return;
    this.perchPoint(plant, b.perch, this.at);
    if (this.at[0] !== b.x || this.at[1] !== b.y || this.at[2] !== b.z) {
      b.x = this.at[0];
      b.y = this.at[1];
      b.z = this.at[2];
      b.dirty = true;
    }
    if (dt <= 0) return;
    const now = this.clock;
    const sp = b.sp;
    if (b.state === 'roost') {
      // Asleep: head turned back into its feathers, puffed up, still.
      this.ease(b, 2.5, 0.45, 0, dt, 3);
      b.puff = lerp(b.puff, 1.1, Math.min(1, dt * 0.5));
      b.tail = lerp(b.tail, -0.05, Math.min(1, dt));
      b.dirty = true;
      return;
    }
    b.puff = lerp(b.puff, b.singUntil > now ? 1.04 : 1, Math.min(1, dt * 2));
    if (b.singUntil > now) {
      // Singing: head up, a little sway of it.
      this.ease(b, Math.sin(now * 3 + b.id) * 0.25, -0.45, 0, dt, 10);
    } else if (b.preenUntil > now) {
      // Preening: head round to its back and wings, nibbling.
      const side = b.id % 2 ? 1 : -1;
      this.ease(b, side * 2.2 + Math.sin(now * 17) * 0.12, 0.55 + Math.sin(now * 23) * 0.1, side * 0.3, dt, 8);
    } else {
      if (now >= b.nextGlance) {
        const calm = this.calm ? 3 : 1;
        b.nextGlance = now + between(rnd, sp.glance) * calm;
        const back = rnd() < 0.25;
        b.hyTo = back ? 0 : (rnd() - 0.5) * 2.2;
        b.hpTo = back ? 0 : (rnd() - 0.6) * 0.5;
        // Small birds cock the head to look up and down.
        b.hrTo = sp.length < 0.2 && rnd() < 0.3 ? (rnd() - 0.5) * 0.8 : 0;
      }
      this.ease(b, b.hyTo, b.hpTo, b.hrTo, dt, this.calm ? 3 : sp.snap);
      if (now >= b.nextPreen) {
        b.nextPreen = now + between(rnd, sp.preen);
        b.preenUntil = now + 1.5 + rnd() * 2.5;
      }
    }
    this.flick(b, now);
    b.dirty = true;
  }

  /** The tail's flick (up fast, down slower; a blackbird holds it up longer). */
  private flick(b: Bird, now: number): void {
    const sp = b.sp;
    if (sp.flick && !this.calm && now >= b.nextFlick) {
      b.nextFlick = now + between(rnd, sp.flick);
      b.flickAt = now;
    }
    const t = now - b.flickAt;
    const hold = sp.id === 'blackbird' ? 0.35 : 0.12;
    b.tail = t < 0 || t > 2 ? 0 : (t < 0.05 ? t / 0.05 : Math.exp(-(t - 0.05) / hold)) * (sp.flickAmp || 0.15);
  }

  /** Turns the head towards a pose, `rate` per second (high: a snap). */
  private ease(b: Bird, y: number, p: number, r: number, dt: number, rate: number): void {
    const k = Math.min(1, dt * rate);
    b.hy += (y - b.hy) * k;
    b.hp += (p - b.hp) * k;
    b.hr += (r - b.hr) * k;
    b.bob = 0;
  }

  /** On the lawn: hopping, running or walking about, pecking, looking up. */
  private forage(b: Bird, dt: number): void {
    if (dt <= 0) return;
    const now = this.clock;
    const sp = b.sp;
    if (b.stepT < b.stepDur) {
      // A move under way.
      b.stepT = Math.min(b.stepDur, b.stepT + dt);
      const s = b.stepT / b.stepDur;
      const S = b.step;
      if (sp.gait === 'walk') {
        // A dove walks, its head bobbing: thrust forward, then held still as the body catches up.
        const e = s;
        b.x = lerp(S[0], S[2], e);
        b.z = lerp(S[1], S[3], e);
        const pace = (s * b.steps) % 1;
        b.bob = (pace < 0.4 ? pace / 0.4 : 1 - (pace - 0.4) / 0.6) * 0.025 * (this.looks[b.look]?.length ?? 0.3) * 3;
        b.y = this.groundY();
      } else {
        const hops = Math.max(1, b.steps);
        const u = s * hops;
        const k = Math.min(hops - 1, Math.floor(u));
        const f = u - k;
        const e = (k + f * f * (3 - 2 * f)) / hops;
        b.x = lerp(S[0], S[2], e);
        b.z = lerp(S[1], S[3], e);
        b.y = this.groundY() + Math.sin(Math.PI * f) * (sp.gait === 'run' ? 0.015 : 0.035);
      }
      b.yaw += turn(b.yaw, Math.atan2(S[2] - S[0], S[3] - S[1])) * Math.min(1, dt * 20);
      this.ease(b, 0, sp.gait === 'run' ? -0.1 : 0.1, 0, dt, 12);
      b.dirty = true;
      return;
    }
    b.bob = lerp(b.bob, 0, Math.min(1, dt * 8));
    if (b.peck >= 0) {
      // Pecks: head down and up, quickly.
      b.peck += dt;
      const n = Math.floor(b.peck / 0.22);
      const f = (b.peck % 0.22) / 0.22;
      b.hp = (f < 0.4 ? f / 0.4 : 1 - (f - 0.4) / 0.6) * 1.0;
      b.hy = lerp(b.hy, 0, Math.min(1, dt * 10));
      b.pitch = lerp(b.pitch, 0.2, Math.min(1, dt * 8));
      if (n >= b.steps) {
        b.peck = -1;
        b.pitch = 0;
      }
      b.dirty = true;
      return;
    }
    b.pitch = lerp(b.pitch, 0, Math.min(1, dt * 8));
    if (now < b.nextStep) {
      // Looking about (a blackbird stands tall and cocks its head to listen).
      if (now >= b.nextGlance) {
        b.nextGlance = now + between(rnd, sp.glance) * (this.calm ? 3 : 1);
        b.hyTo = (rnd() - 0.5) * 1.6;
        b.hpTo = sp.gait === 'run' ? -0.25 : (rnd() - 0.6) * 0.4;
        b.hrTo = sp.gait === 'run' && rnd() < 0.5 ? (rnd() - 0.5) * 0.9 : 0;
      }
      this.ease(b, b.hyTo, b.hpTo, b.hrTo, dt, this.calm ? 3 : sp.snap);
      this.flick(b, now);
      b.dirty = true;
      return;
    }
    // Next: a peck or two, or a move.
    if (rnd() < (sp.gait === 'walk' ? 0.55 : 0.45)) {
      b.peck = 0;
      b.steps = 1 + Math.floor(rnd() * (sp.gait === 'walk' ? 4 : 3));
      b.nextStep = now + 0.3 + rnd() * 0.6;
      return;
    }
    const [hops, len, dur] = sp.gait === 'walk' ? [3 + Math.floor(rnd() * 5), 0.09, 0.32] : sp.gait === 'run' ? [4 + Math.floor(rnd() * 6), 0.13, 0.09] : [1 + Math.floor(rnd() * 3), 0.11, 0.15];
    // Keep near where it came down: wander, turning back towards its tree's side as it strays.
    const home = this.plants[b.home];
    const away = Math.hypot(b.x - home.x, b.z - home.z);
    let dir = b.yaw + (rnd() - 0.5) * 2.2;
    if (away > home.radius + 6) dir = Math.atan2(home.x - b.x, home.z - b.z) + (rnd() - 0.5);
    for (let k = 0; k < 6; k++) {
      const tx = b.x + Math.sin(dir) * hops * len;
      const tz = b.z + Math.cos(dir) * hops * len;
      if (this.lawn(tx, tz)) {
        const S = b.step;
        [S[0], S[1], S[2], S[3]] = [b.x, b.z, tx, tz];
        b.steps = hops;
        b.stepT = 0;
        b.stepDur = hops * dur;
        break;
      }
      dir += Math.PI / 3 + rnd();
    }
    b.nextStep = now + b.stepDur + between(rnd, sp.gait === 'run' ? [0.8, 2.5] : [0.3, 1.5]);
  }

  // ---- drawing ------------------------------------------------------------------------------

  /** Writes a bird's matrices into its species' buffers. */
  private write(b: Bird): void {
    const look = this.looks[b.look];
    if (!look || b.slot >= look.capacity) return;
    const o = b.slot * 16;
    const s = b.size;
    // Placed by the body's middle (it pitches and rolls about that), feet down.
    Quaternion.RotationYawPitchRollToRef(b.yaw, b.pitch, b.roll, this.q);
    Matrix.ComposeToRef(this.vS.set(s, s, s), this.q, this.vT.set(b.x, b.y + look.centre * s, b.z), this.world);
    const W = this.world.m;
    const base = this.base;
    for (let i = 0; i < 16; i++) base[i] = W[i];
    base[12] -= look.centre * W[4];
    base[13] -= look.centre * W[5];
    base[14] -= look.centre * W[6];
    // Body (puffed up about its middle).
    const P = this.pose;
    this.identity(P, b.puff);
    this.pivot(P, 0, look.centre, 0);
    mul(P, base, look.buffers[BODY], o);
    // Head: yaw, pitch and a cock of it about the neck, and a dove's bob.
    Matrix.RotationYawPitchRollToRef(b.hy, b.hp, b.hr, this.rot);
    this.fromRot(this.rot, 1);
    const n = look.pivots[HEAD];
    this.pivot(P, n.x, n.y, n.z);
    P[14] += b.bob;
    mul(P, base, look.buffers[HEAD], o);
    // Tail: up and down about its base.
    Matrix.RotationXToRef(b.tail, this.rot);
    this.fromRot(this.rot, 1);
    const t = look.pivots[TAIL];
    this.pivot(P, t.x, t.y, t.z);
    mul(P, base, look.buffers[TAIL], o);
    // Spread wings: beating about the shoulders (up: the tips rise), hidden while shut.
    for (const [part, sign] of [[WING_L, -1], [WING_R, 1]] as const) {
      Matrix.RotationZToRef(sign * b.flap, this.rot);
      this.fromRot(this.rot, b.spread);
      const w = look.pivots[part];
      this.pivot(P, w.x, w.y, w.z);
      mul(P, base, look.buffers[part], o);
    }
    look.dirty = true;
  }

  private identity(P: Float32Array, s: number): void {
    P.fill(0);
    P[0] = P[5] = P[10] = s;
    P[15] = 1;
  }

  /** The pose's rotation part from `m`, scaled by `s`. */
  private fromRot(m: Matrix, s: number): void {
    const R = m.m;
    const P = this.pose;
    for (let i = 0; i < 12; i++) P[i] = (i + 1) % 4 === 0 ? 0 : R[i] * s;
    P[15] = 1;
  }

  /** Makes the pose turn about (x, y, z) rather than the origin. */
  private pivot(P: Float32Array, x: number, y: number, z: number): void {
    P[12] = x - (x * P[0] + y * P[4] + z * P[8]);
    P[13] = y - (x * P[1] + y * P[5] + z * P[9]);
    P[14] = z - (x * P[2] + y * P[6] + z * P[10]);
  }

  /** The species' model: its file (nodes as parts) or, testing, the stand-in. */
  private lookOf(i: number): Promise<Look | null> {
    const done = this.looks[i];
    if (done) return Promise.resolve(done);
    const building = this.building[i];
    if (building) return building;
    const sp = SPECIES[i];
    const key = `model.bird.${sp.id}`;
    const entry = this.assets.get(key, 'model') as BirdExtras | undefined;
    const made = (async (): Promise<Look | null> => {
      const template = entry ? await buildModelParts(this.scene, `bird:${sp.id}`, entry, PARTS) : null;
      if (template?.meshes.length && template.motion) return this.lookFrom(sp, entry!, template);
      if (this.standIn) return this.standInLook(sp);
      for (const mesh of template?.meshes ?? []) mesh.dispose(false, false);
      return null;
    })().then((look) => {
      this.building[i] = null;
      this.looks[i] = look;
      return look;
    });
    this.building[i] = made;
    return made;
  }

  private lookFrom(sp: Species, entry: BirdExtras, template: ModelTemplate): Look {
    const meshes: Mesh[][] = PARTS.map(() => []);
    template.meshes.forEach((mesh, k) => meshes[Math.max(0, template.motion!.parts[k])].push(mesh));
    const pivots = PARTS.map((_, k) => template.motion!.rests?.[k]?.getTranslation() ?? Vector3.Zero());
    const bird = entry.bird ?? {};
    return this.ready({
      sp, meshes, pivots, buffers: [], capacity: 0, count: 0, dirty: false, url: entry.url ?? null,
      centre: bird.perchHeight ?? template.height * 0.45,
      pitch: ((bird.pitch ?? 0) * Math.PI) / 180,
      length: bird.length ?? sp.length,
    });
  }

  /** Plain shapes in the species' colours, to the model's contract (testing only). */
  private standInLook(sp: Species): Look {
    const L = sp.length;
    const mat = (hex: string) => {
      const m = new StandardMaterial(`birdStandIn:${hex}`, this.scene);
      m.diffuseColor = Color3.FromHexString(hex).toLinearSpace();
      m.specularColor = Color3.Black();
      return m;
    };
    const [cBody, cHead, cWing, cTail, cBeak] = sp.standIn.map(mat);
    const h = L * 0.42;
    const part = (mesh: Mesh, material: StandardMaterial) => {
      mesh.material = material;
      mesh.bakeCurrentTransformIntoVertices();
      return mesh;
    };
    const body = part(MeshBuilder.CreateSphere('birdBody', { segments: 8, diameterX: L * 0.36, diameterY: L * 0.36, diameterZ: L * 0.55 }, this.scene), cBody);
    body.position.set(0, h, 0);
    body.bakeCurrentTransformIntoVertices();
    const legs = part(MeshBuilder.CreateBox('birdLegs', { width: L * 0.08, height: h * 0.7, depth: L * 0.04 }, this.scene), cBeak);
    legs.position.set(0, h * 0.35, 0);
    legs.bakeCurrentTransformIntoVertices();
    const neck = new Vector3(0, h + L * 0.1, L * 0.2);
    const head = part(MeshBuilder.CreateSphere('birdHead', { segments: 8, diameter: L * 0.26 }, this.scene), cHead);
    head.position.set(0, neck.y + L * 0.08, neck.z + L * 0.05);
    head.bakeCurrentTransformIntoVertices();
    const beak = part(MeshBuilder.CreateCylinder('birdBeak', { height: L * 0.12, diameterTop: 0, diameterBottom: L * 0.06, tessellation: 6 }, this.scene), cBeak);
    beak.rotation.x = Math.PI / 2;
    beak.position.set(0, neck.y + L * 0.07, neck.z + L * 0.22);
    beak.bakeCurrentTransformIntoVertices();
    const tailAt = new Vector3(0, h, -L * 0.22);
    const tail = part(MeshBuilder.CreateBox('birdTail', { width: L * 0.14, height: L * 0.03, depth: L * 0.32 }, this.scene), cTail);
    tail.rotation.x = -0.25;
    tail.position.set(0, h + L * 0.02, tailAt.z - L * 0.15);
    tail.bakeCurrentTransformIntoVertices();
    const span = sp.wingspan / 2;
    const wings = [-1, 1].map((side) => {
      const wing = part(MeshBuilder.CreateBox('birdWing', { width: span, height: L * 0.02, depth: L * 0.3 }, this.scene), cWing);
      wing.position.set((side * span) / 2, h + L * 0.08, 0);
      wing.bakeCurrentTransformIntoVertices();
      return wing;
    });
    const shoulder = (side: number) => new Vector3(side * L * 0.1, h + L * 0.08, 0);
    const bodyMesh = Mesh.MergeMeshes([body, legs], true, true, undefined, false, true)!;
    const headMesh = Mesh.MergeMeshes([head, beak], true, true, undefined, false, true)!;
    return this.ready({
      sp, meshes: [[bodyMesh], [headMesh], [tail], [wings[0]], [wings[1]]], pivots: [Vector3.Zero(), neck, tailAt, shoulder(-1), shoulder(1)], buffers: [], capacity: 0, count: 0, dirty: false, url: null,
      centre: h, pitch: 0, length: L,
    });
  }

  private ready(look: Look): Look {
    for (const list of look.meshes) {
      for (const mesh of list) {
        mesh.isPickable = false;
        mesh.receiveShadows = true;
        mesh.alwaysSelectAsActiveMesh = true;
        mesh.doNotSyncBoundingInfo = true;
        mesh.setEnabled(false);
      }
    }
    return look;
  }

  /** Gives a species' parts room for `count` birds (each written when it's next placed). */
  private size(look: Look, count: number): void {
    look.count = count;
    if (count > look.capacity || count === 0) {
      look.capacity = Math.max(1, count);
      look.buffers = PARTS.map(() => new Float32Array(look.capacity * 16));
    }
    look.meshes.forEach((list, k) => {
      for (const mesh of list) {
        mesh.thinInstanceSetBuffer('matrix', look.buffers[k], 16, false);
        mesh.thinInstanceCount = count;
        mesh.setEnabled(this.visible && count > 0);
      }
    });
    for (const b of this.birds) if (b.look >= 0 && this.looks[b.look] === look) this.write(b);
    look.dirty = true;
  }

  private dropLook(i: number): void {
    const look = this.looks[i];
    this.looks[i] = null;
    for (const list of look?.meshes ?? []) for (const mesh of list) mesh.dispose(false, look!.url === null);
  }
}

/** out[o..o+16] = a × b (row vectors, as Babylon's matrices). */
function mul(a: Float32Array, b: Float32Array, out: Float32Array, o: number): void {
  for (let r = 0; r < 4; r++) {
    const a0 = a[r * 4];
    const a1 = a[r * 4 + 1];
    const a2 = a[r * 4 + 2];
    const a3 = a[r * 4 + 3];
    out[o + r * 4] = a0 * b[0] + a1 * b[4] + a2 * b[8] + a3 * b[12];
    out[o + r * 4 + 1] = a0 * b[1] + a1 * b[5] + a2 * b[9] + a3 * b[13];
    out[o + r * 4 + 2] = a0 * b[2] + a1 * b[6] + a2 * b[10] + a3 * b[14];
    out[o + r * 4 + 3] = a0 * b[3] + a1 * b[7] + a2 * b[11] + a3 * b[15];
  }
}
