// `rotationYawPitchRoll` is ported from Babylon.js 9.30 (`Matrix.RotationYawPitchRollToRef`,
// `Matrix.FromQuaternionToRef`). Changed for Idyll Lives: on plain arrays.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
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
 *   as the wind bends it, `windSway`): small jerky glances of the head, a flick of the tail, preening
 *   now and then, a hop to a nearby twig, a turn on the spot. Small birds fidget; doves sit long.
 * - Flying: short hops within a plant, flights between plants, to the lawn and the roof, along a
 *   curve kept clear of the houses (their walls and roofs, a height per tile): a take-off burst; tits,
 *   finches and warblers bound (flapping on the rise, wings shut on the dip), blackbirds fly low and
 *   direct, doves flap steadily with glides; banking into turns, a flare to land.
 * - On the lawn (blackbirds, chaffinches, doves): hopping, running or walking (a dove's head bobs),
 *   pecking, looking up; never on paths, patios, in rooms or under furniture.
 * - Wary: a resident within a few metres, or the camera diving close, sends them up into the
 *   nearest plant away from it; they keep off the lawn for a while after.
 * - Fed: seed scattered from a bench (`BirdFrame.seed`) brings the ground birds within 25 m down round it,
 *   a few at a time; they work towards it, peck more and stay while it's thrown, and don't mind the one
 *   throwing it (someone else coming close still sends them up).
 * - The day: busiest at dawn and early evening, quieter at midday (`birdActivity`, as the songs
 *   heard); at dusk each flies home to roost (head turned back into its feathers, puffed up) and
 *   wakes at first light.
 * - Time: the birds' clock runs with the game's (paused: they hold still; at speeds 3-5 they hurry
 *   only a little, `RATE`). Reduce motion: no flights or hops but the one home; slow, eased head turns.
 *
 * Drawn instanced, a buffer per species and part (body, head, tail, the two spread wings); a bird
 * is written only when its pose has changed by more than a hair since it was last written (one
 * settled asleep on a still branch is not), only the parts that changed (the wings only in flight),
 * and only those parts' buffers go up; nothing is allocated per frame.
 *
 * Engine-neutral: everything above and every number drawn (the parts' matrices, 16 floats each,
 * translation at 12, 13, 14, in game space: `shared/math.ts`, in 32-bit floats) is here. The engine (`BirdView`: `pc/birds.ts`) builds each
 * species' model with its parts as meshes, draws them instanced from the buffers it's handed, and
 * says how far the wind bends a plant.
 */

import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry, Vec3 } from '../../assets/types';
import type { Content } from '../../content/content';
import { groundDepth, objectCentre, type WorldStructure } from '../../core/protocol';
import { birdActivity, type SeenBird, type ViewRect } from '../types';
import { angleDelta as turn } from './angles';
import { compose, multiply, quatYawPitchRoll, rotationX, rotationZ, scaling, transformNormal, transformPoint, translation, type Quat } from './math';
import { entrySpace, placementVariation } from './modelEntry';
import { bareTiles, hash, mulberry32, smoothstep, windDirection, type PlantPlacement } from './nature';
import { WALL_HEIGHT } from './walls';

export type BirdSpecies = 'blueTit' | 'chaffinch' | 'warbler' | 'blackbird' | 'collaredDove';

/** The model's nodes, in the order of the parts drawn. */
export const BIRD_PARTS = ['body', 'head', 'tail', 'wing.L', 'wing.R'] as const;
const PARTS = BIRD_PARTS;
const BODY = 0;
const HEAD = 1;
const TAIL = 2;
const WING_L = 3;
const WING_R = 4;
const ALL_PARTS = (1 << PARTS.length) - 1;

/** How a kind of flight goes: the curve's lift (m) out of the start (plus per metre flown) and into the end (to the lawn, to a perch), and its bounds. */
interface Flight {
  lift: [number, number];
  land: [number, number];
  /** Metres a bound (0: none), how far it rises and falls (m), and the beat's phase above which it flaps (cos). */
  bound: number;
  boundAmp: number;
  flapsAbove: number;
  /** Glides on held wings now and then. */
  glides: boolean;
}

/** `bounding`: flap bursts with shut wings between (an undulating line); `direct`: low and straight, short glides; `steady`: even flapping with glides. */
const FLIGHT: Record<'bounding' | 'direct' | 'steady', Flight> = {
  bounding: { lift: [0.5, 0.1], land: [0.8, 0.25], bound: 2.2, boundAmp: 0.16, flapsAbove: -0.1, glides: false },
  // Low and straight: just above the lawn if it can.
  direct: { lift: [0.3, 0.03], land: [0.35, 0.15], bound: 3.5, boundAmp: 0.06, flapsAbove: -0.6, glides: false },
  steady: { lift: [0.8, 0.12], land: [1.2, 0.4], bound: 0, boundAmp: 0, flapsAbove: 0, glides: true },
};

/** How a species gets about on the lawn. */
interface Gait {
  /** How far from its plant's edge it comes down (m, up to). */
  reach: number;
  /** A move: so many hops or paces (fewest, and up to how many more), each so long (m) and taking so long (s); its hops' height (m; a walk: none, the head bobs). */
  steps: [number, number];
  stride: number;
  pace: number;
  lift: number;
  bobs: boolean;
  /** The head while moving (pitch), and the pause after a move (s). */
  headPitch: number;
  pause: [number, number];
  /** Share of its turns that are pecks, and the most pecks at once. */
  pecks: number;
  peckMax: number;
  /** Stands tall and cocks its head to listen (a blackbird). */
  listens: boolean;
}

const GAIT: Record<'hop' | 'run' | 'walk', Gait> = {
  hop: { reach: 3.5, steps: [1, 3], stride: 0.11, pace: 0.15, lift: 0.035, bobs: false, headPitch: 0.1, pause: [0.3, 1.5], pecks: 0.45, peckMax: 3, listens: false },
  run: { reach: 3.5, steps: [4, 6], stride: 0.13, pace: 0.09, lift: 0.015, bobs: false, headPitch: -0.1, pause: [0.8, 2.5], pecks: 0.45, peckMax: 3, listens: true },
  // A dove walks, its head bobbing: thrust forward, then held still as the body catches up.
  walk: { reach: 5, steps: [3, 5], stride: 0.09, pace: 0.32, lift: 0, bobs: true, headPitch: 0.1, pause: [0.3, 1.5], pecks: 0.55, peckMax: 4, listens: false },
};

/** How a species looks and behaves. Times in the birds' seconds, distances in metres, angles in radians. */
interface Species {
  id: BirdSpecies;
  /** Its song (`audio/birdsong.ts` species). */
  song: string;
  /**
   * Drawn this much larger than life: at real size a tit is 5 px and a blackbird 10 px from the
   * game's usual camera, specks; the small ones grow most (the game's apples and flowers are a
   * little larger than life too), so they still differ in size as they should.
   */
  show: number;
  flight: keyof typeof FLIGHT;
  /** Cruising speed (m/s) and wingbeats a second (kept low enough to read at 60 fps). */
  speed: number;
  flapHz: number;
  /** Seconds between glances, and how fast the head snaps round (1/s); whether it cocks its head to look up and down (the small ones). */
  glance: [number, number];
  snap: number;
  cock: boolean;
  /** Seconds between tail flicks (null: none, but as it lands), how far the tail flicks up and how long it's held there (s). */
  flick: [number, number] | null;
  flickAmp: number;
  flickHold: number;
  preen: [number, number];
  /** Seconds between hops within its plant and between flights away, at the busiest time of day. */
  hop: [number, number];
  fly: [number, number];
  /** Share of its flights that go down to the lawn, and how long it stays there. */
  ground: number;
  groundStay: [number, number];
  gait: keyof typeof GAIT;
  /** Share of its flights to the roof ridge. */
  roof: number;
  /** How close (m) a resident may come before it flies up (on the lawn; 0.6 of that perched low). */
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
}

const SPECIES: Species[] = [
  {
    id: 'blueTit', song: 'tit', show: 1.6, flight: 'bounding', speed: 5.5, flapHz: 15,
    glance: [0.25, 1.0], snap: 40, cock: true, flick: [1, 3], flickAmp: 0.35, flickHold: 0.12, preen: [20, 50], hop: [1.5, 5], fly: [12, 35],
    ground: 0, groundStay: [0, 0], gait: 'hop', roof: 0, wary: 2.5, wake: 5.2, roost: 20.4, pair: false, height: 0.6,
    habitat: [3, 2, 2],
  },
  {
    id: 'chaffinch', song: 'finch', show: 1.5, flight: 'bounding', speed: 6.5, flapHz: 13,
    glance: [0.4, 1.6], snap: 30, cock: true, flick: [2, 6], flickAmp: 0.25, flickHold: 0.12, preen: [25, 60], hop: [3, 9], fly: [15, 45],
    ground: 0.45, groundStay: [10, 40], gait: 'hop', roof: 0, wary: 3, wake: 5.1, roost: 20.5, pair: false, height: 0.5,
    habitat: [3, 2, 1.5],
  },
  {
    id: 'warbler', song: 'warbler', show: 1.6, flight: 'bounding', speed: 5, flapHz: 15,
    glance: [0.2, 0.8], snap: 40, cock: true, flick: [1, 3], flickAmp: 0.3, flickHold: 0.12, preen: [20, 40], hop: [0.8, 3], fly: [10, 30],
    ground: 0, groundStay: [0, 0], gait: 'hop', roof: 0, wary: 2.5, wake: 5.1, roost: 20.5, pair: false, height: 0.45,
    habitat: [2, 0.5, 2.5],
  },
  {
    // (A blackbird raises its tail as it lands, and holds it up longer.)
    id: 'blackbird', song: 'blackbird', show: 1.35, flight: 'direct', speed: 7, flapHz: 11,
    glance: [0.6, 2.2], snap: 22, cock: false, flick: [3, 8], flickAmp: 0.5, flickHold: 0.35, preen: [30, 80], hop: [6, 15], fly: [15, 40],
    ground: 0.65, groundStay: [15, 60], gait: 'run', roof: 0, wary: 3.5, wake: 4.9, roost: 21, pair: false, height: 0.25,
    habitat: [1.5, 1, 3],
  },
  {
    id: 'collaredDove', song: 'dove', show: 1.25, flight: 'steady', speed: 9, flapHz: 7,
    glance: [1.5, 5], snap: 8, cock: false, flick: null, flickAmp: 0.15, flickHold: 0.12, preen: [40, 120], hop: [15, 40], fly: [30, 90],
    ground: 0.4, groundStay: [20, 70], gait: 'walk', roof: 0.35, wary: 4, wake: 5.6, roost: 20.1, pair: true, height: 0.85,
    habitat: [2, 1.5, 0.2],
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
/** Height of the viewed house above its storeys' walls while its roof isn't known (m). */
const ROOF_RISE = 3.4;
/** A flight shorter than this is a hop (m). */
const HOP_MAX = 1.6;
/** The camera this close (m) scares a bird up. */
const CAMERA_WARY = 1.8;
/** A resident higher than this (m: upstairs) doesn't scare the birds; nor one by a bird perched higher. */
const THREAT_HEIGHT = 2.6;
const LOW_PERCH = 2.2;
/** Least change of a bird's pose that's drawn: its place (m), its turns (rad), puff and the spread of its wings. */
const STILL_M = 0.0015;
const STILL_RAD = 0.004;
const STILL_SCALE = 0.002;

/** Where the birds live (`Birds.setHabitat`). */
export interface BirdHabitat {
  world: WorldStructure;
  view: ViewRect | null;
  /**
   * The trees, bushes and hedges, in groups as their owners keep them (the viewed lot's garden, its
   * things outdoors, the street's, the woods'): a group passed again unchanged (the same array) is
   * not looked through again.
   */
  plants: readonly (readonly PlantPlacement[])[];
  /** Points along the viewed house's roof ridge (x, y, z each), where doves sit while the roof shows. */
  ridge: number[];
  /** Top of the viewed house's roof (m), if known. */
  roofTop: number | null;
  /** The top of another house's roof (m), as the street builds it (`HouseBuilder.roofTop`). */
  houseTop: (plot: { x: number; z: number; house: [number, number, number, number]; roof?: [number, number]; storeys?: number }) => number;
  /** The hour now (birds new to the habitat start asleep at night). */
  hour: number;
}

/** The birds' preset knob (`QualitySettings.birds`). */
export interface BirdQuality {
  /** Most birds about. */
  count: number;
  /** How far from the viewed lot (m) their home plants may be. */
  range: number;
}

/** What `Birds.update` needs of a frame (the caller keeps one and fills it in). */
export interface BirdFrame {
  /** Real seconds since the last frame, and the game's speed (`GAME_RATE` index). */
  dt: number;
  speed: number;
  hour: number;
  /** The camera's position (kept, not copied: read as it is then). */
  camera: { readonly x: number; readonly y: number; readonly z: number };
  /** Residents' heads (x, y, z each), how many, and which are drawn. */
  residents: Float32Array;
  residentCount: number;
  visible: Uint8Array;
  /** Seed being scattered: where it lands (x, z) and the row of the resident throwing it, three each; how many. */
  seed: Float32Array;
  seedCount: number;
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
  /** The way the wind bends it (x, z per m of sway, `windDirection`), and its sway now (m per m² of height) as of frame `swayed`. */
  windX: number;
  windZ: number;
  sway: number;
  swayed: number;
}

type State = 'perch' | 'fly' | 'ground' | 'roost';

interface Bird {
  /** Fixed by its home plant's position and its place among the plant's birds. */
  key: string;
  id: number;
  sp: Species;
  /** Its species' look as drawn, and its slot in the look's buffers. */
  look: BirdLook | null;
  slot: number;
  size: number;
  home: number;
  partner: Bird | null;
  /** The second of a pair: it follows its partner about. */
  follower: boolean;
  /** Its own hours (a little off its species'), and the perch of its home it roosts on. */
  wake: number;
  roost: number;
  roostPerch: number;
  state: State;
  /** A flight it couldn't take (a house in the way): it thinks again from then. */
  retryAt: number;
  /** The perch it's on or flying to (plant, perch index; -1: the lawn). */
  plant: number;
  perch: number;
  x: number;
  y: number;
  z: number;
  yaw: number;
  /** Where it turns to on its perch (NaN: not turning). */
  turnTo: number;
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
  hop: boolean;
  /** Ground: when it leaves, its next move, and the move under way (from, to, time, length). */
  leaveAt: number;
  nextStep: number;
  step: Float32Array;
  stepT: number;
  stepDur: number;
  steps: number;
  peck: number;
  /**
   * Not yet moved: made before the frame's hour was known (the habitat's may be the renderer's
   * default), it goes to its roost on the first frame if that says night.
   */
  fresh: boolean;
  /** Its pose as last written (`Pose`; NaN: to be written whole). */
  drawn: Float32Array;
}

/** A bird's pose as written (`Bird.drawn`): what each part depends on. */
const enum Pose {
  X, Y, Z, Yaw, Pitch, Roll, Hy, Hp, Hr, Bob, Tail, Puff, Spread, Flap, Count,
}

/**
 * A species' model as drawn: whether it has meshes (none: its file failed, drawn as nothing until it
 * comes), its parts' buffers (`BIRD_PARTS` order: a matrix per bird, at its slot) and the nodes'
 * pivots (model space). The engine's look adds its meshes.
 */
export interface BirdLook {
  drawn: boolean;
  buffers: Float32Array[];
  pivots: (readonly [number, number, number])[];
  /** Feet to the body's middle (m, model) and the perched body's nose-up angle (rad), its length (m). */
  centre: number;
  pitch: number;
  length: number;
  capacity: number;
  count: number;
  /** The parts whose buffers were written this frame (bits by part). */
  dirty: number;
  failed?: string[];
}

/** A bird's model entry: its size, the height of its body's middle when perched and its nose-up angle there (degrees). */
export type BirdEntry = ModelEntry & { bird: { length: number; perchHeight: number; pitch: number } };
/** A plant's model: its perches (model space) and what they're on (check-glb checks it's one of four; without one, a bush). */
type PlantEntry = ModelEntry & { perches?: Vec3[]; habitat?: 'tree' | 'conifer' | 'bush' | 'hedge'; fit?: unknown };

/**
 * A species' look from its model (`entry`) built with its parts (`BIRD_PARTS`) as moving parts: their
 * nodes at rest (model space, 16 floats each; a missing one at the origin). Its buffers come with birds.
 */
export function birdLook(entry: BirdEntry, rests: readonly (ArrayLike<number> | undefined)[] | undefined): BirdLook {
  return {
    drawn: true, buffers: [], capacity: 0, count: 0, dirty: 0,
    pivots: PARTS.map((_, k) => {
      const r = rests?.[k];
      return r ? [r[12], r[13], r[14]] as const : [0, 0, 0] as const;
    }),
    centre: entry.bird.perchHeight,
    pitch: (entry.bird.pitch * Math.PI) / 180,
    length: entry.bird.length,
  };
}

/** A species' look without its model (no file, or it failed: `failed`, built again when it comes). */
export function noBirdLook(failed?: string[]): BirdLook {
  return { drawn: false, buffers: [], pivots: [], centre: 0, pitch: 0, length: 0, capacity: 0, count: 0, dirty: 0, failed };
}

/**
 * What the birds ask of the engine that draws them (`pc/birds.ts`). Each species'
 * look is built once and kept (`look`); its parts' meshes draw instanced, one instance per bird, from
 * the buffers the birds' code writes in place (`setBuffers`, `updated`). Nothing is called per frame
 * with new arrays.
 */
export interface BirdView<L extends BirdLook> {
  /**
   * The species' look (manifest key `key`), built the first time it's asked for and kept: its model
   * with the parts `BIRD_PARTS` as moving parts (each part's meshes hidden until it has birds), or
   * one without meshes (`noBirdLook`) if it has none or its file failed.
   */
  look(species: BirdSpecies, key: string): Promise<L>;
  /** A file has come: the looks built without it are set aside (the next `look` builds them again); whether any was. */
  invalidate(url: string): boolean;
  /** Disposes the looks set aside. */
  disposeStale(): void;
  /** Shows or hides every look built: each only while it has birds (`count > 0`). */
  show(visible: boolean): void;
  /** `look` has new buffers (`buffers`, `capacity` matrices each) or draws a new count (`count`): its parts draw from them now, shown if `visible` and it has birds. */
  setBuffers(look: L, visible: boolean): void;
  /** Part `part`'s buffer of `look` was written (its first `count` matrices go up). */
  updated(look: L, part: number): void;
  /** How far the wind bends a plant standing at (`x`, `z`) now (`windSwayAt`; 0 before the wind exists). */
  sway(x: number, z: number): number;
  /** Disposes every look, built or set aside. */
  dispose(): void;
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const between = (r: () => number, [a, b]: readonly [number, number]) => a + r() * (b - a);
const rnd = Math.random;
/** A number fixed by a place (to a tenth of a metre). */
const placeHash = (x: number, z: number, salt: number) => hash(Math.round(x * 10), Math.round(z * 10), salt);
/** Randoms from a seed in 0..1. */
const seeded = (seed: number) => mulberry32(Math.floor(seed * 4294967296));
/** Its pose value `a` differs from `b` by more than `still` (always, if it was never written: NaN). */
const changed = (a: number, b: number, still: number) => !(Math.abs(a - b) <= still);

// Scratch, so frames allocate nothing. (Matrices in 32-bit floats, as the game's are.)
const qTurn: Quat = [0, 0, 0, 1];
const mWorld = new Float32Array(16);
const mBase = new Float32Array(16);
const mPose = new Float32Array(16);
const mTurn = new Float32Array(16);
const mOut = new Float32Array(16);
const vPivot = [0, 0, 0];
const vBody: [number, number, number] = [0, 0, 0];

/** Makes a part's pose `m` turn about `p` (its translation part set so `p` stays put), then shifted `dz` forward. */
function about(m: Float32Array, p: readonly [number, number, number], dz = 0): Float32Array {
  transformNormal(m, p[0], p[1], p[2], vPivot);
  m[12] = p[0] - vPivot[0];
  m[13] = p[1] - vPivot[1];
  m[14] = p[2] - vPivot[2] + dz;
  return m;
}

/** A part's pose after the bird's placement (`mBase`), into its buffer at `o`. */
function put(pose: Float32Array, buffer: Float32Array, o: number): void {
  multiply(pose, mBase, mOut);
  for (let k = 0; k < 16; k++) buffer[o + k] = mOut[k];
}

export class Birds<L extends BirdLook = BirdLook> {
  private birds: Bird[] = [];
  private plants: Plant[] = [];
  /** The looks drawing birds now. */
  private inUse: L[] = [];
  private quality: BirdQuality = { count: 0, range: 0 };
  private habitat: BirdHabitat | null = null;
  /** The plants of each group of placements (`BirdHabitat.plants`), as found for the view and range last asked. */
  private readonly groups = new WeakMap<readonly PlantPlacement[], { near: string; plants: Plant[] }>();
  /** The town's ground tiles: 1 where no bird forages (paths, rooms, patios, furniture, houses). */
  private blocked = new Uint8Array(0);
  /** The houses: x0, z0, x1, z1 (their walls) and the roof's top (m) each. */
  private boxes = new Float32Array(0);
  private width = 0;
  private depth = 0;
  private clock = 0;
  private frame = 0;
  /** Birds' seconds per real second now (`RATE`), and a slow-motion factor for checks (`?debug`'s `__birdTime`). */
  private rate = 1;
  timeScale = 1;
  private hour = 12;
  /** How busy the birds are this frame (`birdActivity`). */
  private busy = 1;
  private calm = false;
  private roofOn = false;
  private visible = true;
  private ticket = 0;
  /** Residents outdoors on the ground this frame (x, z each) and how many: they scare the birds near them. */
  private threats = new Float32Array(0);
  private threatCount = 0;
  /** This frame's seed (`BirdFrame.seed`). */
  private seed: Float32Array = new Float32Array(0);
  private seedCount = 0;
  /** Which perches of a plant are taken (`occupy`). */
  private taken = new Uint8Array(32);
  /**
   * Smoothed main-thread time of `update` (ms), and what the birds did, for `__birds`: flights, hops,
   * visits to the lawn and the roof, scares (by a resident, the camera), and (`?debug` only) frames a
   * bird in flight was inside a house's walls or roof.
   */
  private cost = 0;
  private readonly debugInside: unknown[] = [];
  private readonly stats = { flights: 0, hops: 0, lawn: 0, roof: 0, scaredBy: 0, scaredCamera: 0, inside: 0 };
  private readonly seen: SeenBird[] = [];
  /** The camera (`BirdFrame.camera`). */
  private eye: BirdFrame['camera'] = { x: 0, y: 0, z: 0 };
  private readonly at = new Float32Array(3);

  constructor(
    private readonly view: BirdView<L>,
    private readonly assets: AssetRegistry,
    /** `?debug`: the checks `__birds` reports (`inside`, `roof`) are made. */
    private readonly debugging = false,
  ) {}

  setQuality(quality: BirdQuality): void {
    this.quality = quality;
    if (this.habitat) void this.setHabitat(this.habitat);
  }

  /** Hidden while the basement is in view (the garden is dug away) or behind the overview. */
  setVisible(visible: boolean): void {
    if (visible === this.visible) return;
    this.visible = visible;
    this.view.show(visible);
  }

  /** A model file that had failed has come: a bird model built without it is built again. */
  fileArrived(url: string): void {
    if (this.view.invalidate(url) && this.habitat) void this.setHabitat(this.habitat);
  }

  /**
   * Where the birds live now (the viewed lot built, or anything on it changed): the plants within
   * range get their birds again; a bird whose home is still there carries on as it was.
   */
  async setHabitat(habitat: BirdHabitat): Promise<void> {
    const ticket = ++this.ticket;
    this.habitat = habitat;
    this.hour = habitat.hour;
    this.grids(habitat);
    const plants = this.quality.count > 0 ? this.gather(habitat) : [];
    // The species there may be (those with a model, once it's built).
    const looks = await Promise.all(SPECIES.map((sp) => (plants.length && this.assets.has(`model.bird.${sp.id}`, 'model') ? this.lookOf(sp) : null)));
    if (ticket !== this.ticket) return;
    const birds = this.populate(plants, looks.map((l) => !!l?.drawn), habitat.view);
    this.plants = plants;
    this.birds = birds;
    this.assignSlots(birds, looks);
    // (Built without its file and since built again: nothing draws the old one now.)
    this.view.disposeStale();
  }

  /** The birds as the world's sound hears them: where each is, and whether it may sing (perched, awake). */
  heard(): readonly SeenBird[] {
    const out = this.seen;
    const eye = this.eye;
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
      s.distance = Math.hypot(b.x - eye.x, b.y - eye.y, b.z - eye.z);
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
        drawn: this.visible && !!b.look,
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

  /** Moves the birds a frame on; true if any was drawn anew (the snapshot is recorded again). */
  update(f: BirdFrame): boolean {
    if (!this.birds.length) return false;
    const started = performance.now();
    this.rate = (RATE[f.speed | 0] ?? RATE[RATE.length - 1]) * this.timeScale;
    const dt = Math.min(MAX_STEP, f.dt * this.rate);
    this.clock += dt;
    this.frame++;
    this.hour = f.hour;
    this.busy = birdActivity(f.hour);
    this.calm = f.calm;
    this.eye = f.camera;
    this.roofOn = f.roof;
    this.seed = f.seed;
    this.seedCount = f.seedCount;
    this.findThreats(f);
    let moved = false;
    for (const b of this.birds) {
      if (b.fresh) {
        b.fresh = false;
        if (!this.awake(b) && b.state === 'perch') {
          b.state = 'roost';
          b.plant = b.home;
          b.perch = b.roostPerch;
        }
      }
      if (dt > 0) this.think(b);
      if (b.state === 'perch' || b.state === 'roost') this.sit(b, dt);
      else if (b.state === 'fly') this.fly(b, dt);
      else this.forage(b, dt);
      if (this.visible && this.write(b)) moved = true;
    }
    for (const look of this.inUse) {
      if (!look.dirty) continue;
      for (let k = 0; k < PARTS.length; k++) if (look.dirty & (1 << k)) this.view.updated(look, k);
      look.dirty = 0;
    }
    this.cost += (performance.now() - started - this.cost) * 0.05;
    return moved;
  }

  /** The world is gone (`Renderer.clear`): no birds until the next habitat. */
  clear(): void {
    this.ticket++;
    this.habitat = null;
    this.birds = [];
    this.plants = [];
    for (const look of this.inUse) this.size(look, 0);
    this.inUse = [];
  }

  dispose(): void {
    this.ticket++;
    this.view.dispose();
    this.inUse = [];
    this.birds = [];
    this.plants = [];
  }

  // ---- habitat ------------------------------------------------------------------------------

  /** The plants within range of the view with perches (each group's found once for a view and range), and the roof ridge. */
  private gather({ plants: groups, view, ridge }: BirdHabitat): Plant[] {
    const range = this.quality.range;
    const near = view ? `${view.x},${view.z},${view.w},${view.d},${range}` : '';
    const plants: Plant[] = [];
    for (const group of groups) {
      let found = this.groups.get(group);
      if (found?.near !== near) {
        found = { near, plants: [] };
        for (const p of group) {
          if (!view) continue;
          const dx = Math.max(view.x - p.x, 0, p.x - (view.x + view.w));
          const dz = Math.max(view.z - p.z, 0, p.z - (view.z + view.d));
          if (dx * dx + dz * dz > range * range) continue;
          const plant = this.plantOf(p);
          if (plant) found.plants.push(plant);
        }
        this.groups.set(group, found);
      }
      plants.push(...found.plants);
    }
    if (ridge.length >= 3) {
      const perches = Float32Array.from(ridge);
      const n = perches.length / 3;
      const mid = Math.floor(n / 2);
      plants.push({ key: 'roof', kind: 'roof', conifer: false, x: perches[mid * 3], y: 0, z: perches[mid * 3 + 2], perches, heights: new Float32Array(n), top: perches[1], radius: 0, roosts: [mid], seed: placeHash(perches[0], perches[2], 7), windX: 0, windZ: 0, sway: 0, swayed: -1 });
    }
    return plants;
  }

  /** Each plant's birds (nearest the lot first, so a smaller preset keeps the birds nearest: always the same ones), of the species `usable` says. */
  private populate(plants: Plant[], usable: boolean[], view: ViewRect | null): Bird[] {
    const old = new Map(this.birds.map((b) => [b.key, b]));
    const birds: Bird[] = [];
    const [cx, cz] = view ? [view.x + view.w / 2, view.z + view.d / 2] : [0, 0];
    const order = plants.map((_, i) => i).filter((i) => plants[i].kind !== 'roof');
    const far = (i: number) => (plants[i].x - cx) ** 2 + (plants[i].z - cz) ** 2;
    order.sort((a, b) => far(a) - far(b));
    const count = this.quality.count;
    for (const i of order) {
      if (birds.length >= count) break;
      const plant = plants[i];
      const r = seeded(plant.seed);
      // A big tree two or three, a small one one or two; a bush or a stretch of hedge now and then.
      const n = plant.kind === 'tree' ? (plant.top > 6 ? 2 + Math.floor(r() * 2) : 1 + Math.floor(r() * 2)) : r() < (plant.top > 1.2 ? 0.25 : 0.12) ? 1 : 0;
      let roosting = 0;
      const roostOf = () => plant.roosts[roosting++ % plant.roosts.length];
      for (let k = 0; k < n && birds.length < count; k++) {
        const sp = this.pickSpecies(plant, r(), usable);
        if (!sp) break;
        const key = `${Math.round(plant.x * 10)},${Math.round(plant.z * 10)},${k}`;
        const first = this.makeBird(old.get(key), key, sp, i, plants, roostOf());
        birds.push(first);
        if (sp.pair && birds.length < count) {
          const mate = this.makeBird(old.get(`${key}m`), `${key}m`, sp, i, plants, roostOf());
          mate.partner = first;
          mate.follower = true;
          first.partner = mate;
          birds.push(mate);
        }
      }
    }
    return birds;
  }

  /** Each species' buffers, sized to its birds, and each bird's slot in them (written whole next frame). */
  private assignSlots(birds: Bird[], looks: (L | null)[]): void {
    const inUse: L[] = [];
    for (let i = 0; i < SPECIES.length; i++) {
      const look = looks[i]?.drawn ? looks[i] : null;
      let slot = 0;
      for (const b of birds) {
        if (b.sp !== SPECIES[i]) continue;
        b.look = look;
        b.slot = slot++;
        // (Its slot may have held another bird, its wings spread: every part written anew.)
        b.drawn.fill(NaN);
      }
      if (look) inUse.push(look);
      if (look) this.size(look, slot);
    }
    // Species no longer about.
    for (const look of this.inUse) if (!inUse.includes(look)) this.size(look, 0);
    this.inUse = inUse;
  }

  /** The town's lawn mask per ground tile (the landscape's lawn, less the patios and round the houses), and the houses as boxes. */
  private grids({ world, view, roofTop, houseTop }: BirdHabitat): void {
    const w = (this.width = world.width);
    const d = (this.depth = groundDepth(world));
    const blocked = (this.blocked = new Uint8Array(w * d));
    const fill = (x0: number, z0: number, x1: number, z1: number) => {
      for (let z = Math.max(0, Math.floor(z0)); z < Math.min(d, Math.ceil(z1)); z++) for (let x = Math.max(0, Math.floor(x0)); x < Math.min(w, Math.ceil(x1)); x++) blocked[z * w + x] = 1;
    };
    // Where no lawn grows (paved, fields, indoors, under furniture, the patios).
    const bare = bareTiles(world);
    for (let i = 0; i < w * d; i++) if (bare[i]) blocked[i] = 1;
    // Houses: their walls up to their roofs' tops (the viewed one's roof as built, if known).
    const boxes: number[] = [];
    for (const p of world.plots) {
      const house = p.house;
      if (!house) continue;
      const [x0, z0, x1, z1] = house;
      const viewed = !!view && p.x === view.x && p.z === view.z;
      boxes.push(x0, z0, x1, z1, viewed && roofTop !== null ? roofTop : houseTop({ ...p, house }));
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
      if (x1 > x0) boxes.push(x0, z0, x1, z1, roofTop ?? (world.storeys ?? 1) * WALL_HEIGHT + ROOF_RISE);
    }
    this.boxes = Float32Array.from(boxes);
  }

  /** Whether a bird may stand on the ground here (lawn in the town, clear of everything). */
  private lawn(x: number, z: number): boolean {
    for (let k = 0; k < 5; k++) {
      // The point and four round it, a bird's reach away.
      const tx = Math.floor(x + (k === 0 ? 0 : k % 2 ? 0.18 : -0.18));
      const tz = Math.floor(z + (k === 0 ? 0 : k < 3 ? 0.18 : -0.18));
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
  private plantOf(p: PlantPlacement): Plant | null {
    const entry = this.assets.get(p.key, 'model') as PlantEntry | undefined;
    const points = entry?.perches;
    // (A model fitted to a size (`fit`) is placed by its built geometry: its perches can't be.)
    if (!entry || !points?.length || entry.fit) return null;
    const space = entrySpace(entry);
    const perches = new Float32Array(points.length * 3);
    const heights = new Float32Array(points.length);
    const [c, s] = [Math.cos(p.yaw), Math.sin(p.yaw)];
    let top = 0;
    let radius = 0;
    const depth: number[] = [];
    const m = [0, 0, 0];
    points.forEach(([x, y, z], i) => {
      transformPoint(space, x, y, z, m);
      const [lx, ly, lz] = [m[0] * p.sx, m[1] * p.sy, m[2] * p.sz];
      perches[i * 3] = p.x + lx * c + lz * s;
      perches[i * 3 + 1] = p.y + ly;
      perches[i * 3 + 2] = p.z - lx * s + lz * c;
      // (As the shader: nothing below the model's ground bends.)
      heights[i] = Math.max(0, m[1]);
      top = Math.max(top, ly);
      const out = Math.hypot(lx, lz);
      radius = Math.max(radius, out);
      // Roost deep in the crown, well up.
      depth.push(out - ly * 0.3);
    });
    const roosts = points.map((_, i) => i).sort((a, b) => depth[a] - depth[b]);
    const wind = { x: 0, z: 0 };
    windDirection(p.yaw, p.sx, p.sz, wind);
    return { key: p.key, kind: entry.habitat === 'tree' || entry.habitat === 'conifer' ? 'tree' : 'bush', conifer: entry.habitat === 'conifer', x: p.x, y: p.y, z: p.z, perches, heights, top, radius, roosts, seed: placeHash(p.x, p.z, 0xb1d), windX: wind.x, windZ: wind.z, sway: 0, swayed: -1 };
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
    const size = sp.show * (0.95 + r() * 0.1);
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
      old.follower = false;
      return old;
    }
    const plant = plants[home];
    const b: Bird = {
      key, id, sp, look: null, slot: 0, size, home, partner: null, follower: false, wake, roost, roostPerch,
      state: 'perch', retryAt: 0, plant: home, perch: 0, x: plant.x, y: plant.y, z: plant.z, yaw: r() * Math.PI * 2, turnTo: NaN, pitch: 0, roll: 0,
      hy: 0, hp: 0, hr: 0, bob: 0, hyTo: 0, hpTo: 0, hrTo: 0, tail: 0, spread: 0, flap: 0, phase: 0, puff: 1,
      nextGlance: 0, nextFlick: r() * 3, nextPreen: between(r, sp.preen), preenUntil: -1, flickAt: -10, nextHop: between(r, sp.hop), nextFly: between(r, sp.fly),
      singUntil: -1, waryUntil: -1, path: new Float32Array(12), dur: 0, t: 0, bounds: 0, hop: false,
      leaveAt: 0, nextStep: 0, step: new Float32Array(4), stepT: 0, stepDur: 0, steps: 0, peck: -1,
      fresh: true, drawn: new Float32Array(Pose.Count).fill(NaN),
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
  private think(b: Bird): void {
    const now = this.clock;
    if (b.state === 'roost') {
      if (this.awake(b)) {
        b.state = 'perch';
        // Wakes with a shake and a preen.
        b.preenUntil = now + 1.5 + rnd() * 2;
        b.nextFly = now + between(rnd, b.sp.fly) * 0.5;
      }
      return;
    }
    if (b.state === 'fly' || now < b.retryAt) return;
    // Dusk: home to roost; with home out of reach, where it sits, or in the nearest plant.
    if (!this.awake(b)) {
      if (b.state === 'perch' && b.plant === b.home && b.perch === b.roostPerch) b.state = 'roost';
      else if (!this.flyTo(b, b.home, b.roostPerch, 0.8)) {
        const near = this.nearestPlant(b);
        if (b.state === 'perch') b.state = 'roost';
        else if (near !== b.home) this.flyTo(b, near, this.perchFor(b, this.plants[near]), 0.8);
      }
      return;
    }
    if (this.scared(b)) return;
    // Someone else on the roof ridge's perch when the roof goes: off it.
    if (b.state === 'perch' && this.plants[b.plant]?.kind === 'roof' && !this.roofOn) {
      this.flyAway(b, b.x, b.z);
      return;
    }
    if (this.calm) {
      // Calm: whoever's on the lawn goes home, slowly; nobody else flies about.
      if (b.state === 'ground' && now >= b.leaveAt) this.flyTo(b, b.home, this.perchFor(b, this.plants[b.home]), 0.6);
      return;
    }
    if (b.state === 'ground') this.onLawn(b, now);
    else this.perched(b, now);
  }

  /** The partner a dove's mate follows about (not while that one's in the air). */
  private leader(b: Bird): Bird | null {
    return b.follower && b.partner && b.partner.state !== 'fly' ? b.partner : null;
  }

  /** On the lawn: home when its time's up (or soon after its partner left). */
  private onLawn(b: Bird, now: number): void {
    const mate = this.leader(b);
    // While seed is thrown close by, it stays; nearby on the lawn, it goes over to it rather than home.
    const seed = this.seedNear(b, 12);
    if (seed >= 0 && (this.seed[seed * 3] - b.x) ** 2 + (this.seed[seed * 3 + 1] - b.z) ** 2 < 2.5 * 2.5) {
      b.leaveAt = Math.max(b.leaveAt, now + 2 + rnd() * 4);
      return;
    }
    if (mate && mate.state !== 'ground') b.leaveAt = Math.min(b.leaveAt, now + 0.5 + rnd());
    if (now < b.leaveAt) return;
    if (seed >= 0 && this.flyToSeed(b, seed)) return;
    if (mate && mate.plant >= 0) this.flyTo(b, mate.plant, this.nearPerch(mate.plant, mate.perch, b), 1);
    else this.flyHome(b);
  }

  /** Perched: now and then off somewhere (after its partner), a hop to the next twig, a turn. */
  private perched(b: Bird, now: number): void {
    if (b.singUntil > now || b.preenUntil > now) return;
    const mate = this.leader(b);
    if (mate && (mate.plant !== b.plant || mate.state === 'ground') && b.nextFly > now + 1.5) b.nextFly = now + 0.4 + rnd() * 1.2;
    // Seed thrown nearby: down to it before long, a few at a time.
    const seed = this.seedNear(b, 25);
    if (seed >= 0 && b.nextFly > now + 8) b.nextFly = now + 0.5 + rnd() * 7;
    if (now >= b.nextFly) {
      b.nextFly = now + between(rnd, b.sp.fly) / (0.4 + this.busy);
      if (seed >= 0 && this.flyToSeed(b, seed)) return;
      if (mate) this.followMate(b, mate);
      else this.flyOff(b);
      return;
    }
    if (now < b.nextHop) return;
    b.nextHop = now + between(rnd, b.sp.hop) / (0.5 + this.busy * 0.5);
    const plant = this.plants[b.plant];
    if (!plant || plant.kind === 'roof') {
      // Along the ridge a little, or a turn.
      if (plant) this.flyTo(b, b.plant, Math.max(0, Math.min(plant.perches.length / 3 - 1, b.perch + (rnd() < 0.5 ? -1 : 1))), 1);
      return;
    }
    const next = this.nearPerch(b.plant, b.perch, b);
    if (next !== b.perch) this.flyTo(b, b.plant, next, 1);
    // A turn on the spot, in a flick (`sit`).
    else b.turnTo = b.yaw + (rnd() < 0.5 ? -1 : 1) * (0.6 + rnd() * 1.6);
  }

  /** After its partner: down beside it on the lawn, or to a perch near its. */
  private followMate(b: Bird, mate: Bird): void {
    if (mate.state === 'ground') {
      const a = rnd() * Math.PI * 2;
      const r = 0.4 + rnd() * 0.5;
      this.flyDown(b, mate.x + Math.cos(a) * r, mate.z + Math.sin(a) * r, 0.7);
    } else this.flyTo(b, mate.plant, this.nearPerch(mate.plant, mate.perch, b), 1);
  }

  /** The residents outdoors on the ground (not upstairs, not indoors behind the walls), once a frame. */
  private findThreats(f: BirdFrame): void {
    if (this.threats.length < f.residentCount * 2) this.threats = new Float32Array(f.residentCount * 2);
    let n = 0;
    for (let i = 0; i < f.residentCount; i++) {
      if (!f.visible[i] || this.feeding(i)) continue;
      const hx = f.residents[i * 3];
      const hz = f.residents[i * 3 + 2];
      if (f.residents[i * 3 + 1] > THREAT_HEIGHT || this.topAt(hx, hz) > 0) continue;
      this.threats[n * 2] = hx;
      this.threats[n * 2 + 1] = hz;
      n++;
    }
    this.threatCount = n;
  }

  /** Whether resident row `i` is throwing seed now (not a threat). */
  private feeding(i: number): boolean {
    for (let k = 0; k < this.seedCount; k++) if (this.seed[k * 3 + 2] === i) return true;
    return false;
  }

  /** The seed nearest a bird within `range` m (its index in `seed`), -1 for none; only for the ground birds, not when wary. */
  private seedNear(b: Bird, range: number): number {
    if (b.sp.ground <= 0 || b.waryUntil > this.clock) return -1;
    let pick = -1;
    let best = range * range;
    for (let k = 0; k < this.seedCount; k++) {
      const d = (this.seed[k * 3] - b.x) ** 2 + (this.seed[k * 3 + 1] - b.z) ** 2;
      if (d < best) [best, pick] = [d, k];
    }
    return pick;
  }

  /** Down to the seed (`seed[k]`): a spot of lawn round where it lands. False if there's none. */
  private flyToSeed(b: Bird, k: number): boolean {
    const sx = this.seed[k * 3];
    const sz = this.seed[k * 3 + 1];
    for (let n = 0; n < 12; n++) {
      const a = rnd() * Math.PI * 2;
      const r = 0.35 + rnd() * 1.0;
      const x = sx + Math.cos(a) * r;
      const z = sz + Math.sin(a) * r;
      if (!this.lawn(x, z)) continue;
      this.flyDown(b, x, z, 1);
      return b.state === 'fly';
    }
    return false;
  }

  /** A resident or the camera close: up into the nearest plant away from them. True if it fled. */
  private scared(b: Bird): boolean {
    if (this.calm && b.state !== 'ground') return false;
    let threat = -1;
    if (b.state === 'ground' || b.y < LOW_PERCH) {
      const reach = b.state === 'ground' ? b.sp.wary : b.sp.wary * 0.6;
      const T = this.threats;
      for (let k = 0; k < this.threatCount; k++) if ((T[k * 2] - b.x) ** 2 + (T[k * 2 + 1] - b.z) ** 2 < reach * reach) threat = k;
    }
    const c = this.eye;
    const camera = (c.x - b.x) ** 2 + (c.y - b.y) ** 2 + (c.z - b.z) ** 2 < CAMERA_WARY * CAMERA_WARY;
    if (threat < 0 && !camera) return false;
    if (threat >= 0) this.stats.scaredBy++;
    else this.stats.scaredCamera++;
    b.waryUntil = this.clock + 20 + rnd() * 40;
    if (threat >= 0) this.flyAway(b, this.threats[threat * 2], this.threats[threat * 2 + 1]);
    else this.flyAway(b, c.x, c.z);
    return true;
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
      if (roof >= 0 && Math.hypot(this.plants[roof].x - b.x, this.plants[roof].z - b.z) < 25) {
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
    const pick = Math.hypot(home.x - b.x, home.z - b.z) > 10 ? this.nearestPlant(b) : b.home;
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

  /** The plant nearest a bird (its home if there's none). */
  private nearestPlant(b: Bird): number {
    let pick = b.home;
    let best = Infinity;
    for (let i = 0; i < this.plants.length; i++) {
      if (this.plants[i].kind === 'roof') continue;
      const d = Math.hypot(this.plants[i].x - b.x, this.plants[i].z - b.z);
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

  /** A free perch of plant `at` near `perch` (within a hop), or `perch` itself. */
  private nearPerch(at: number, perch: number, b: Bird): number {
    const p = this.plants[at].perches;
    const n = p.length / 3;
    const taken = this.occupy(at, n, b);
    let pick = perch;
    let best = Infinity;
    for (let k = 0; k < n; k++) {
      if (k === perch || taken[k]) continue;
      const d = Math.hypot(p[k * 3] - p[perch * 3], (p[k * 3 + 1] - p[perch * 3 + 1]) * 1.5, p[k * 3 + 2] - p[perch * 3 + 2]);
      const score = d + rnd() * 0.6;
      if (d < HOP_MAX && score < best) [best, pick] = [score, k];
    }
    return pick;
  }

  /** Which of plant `at`'s `n` perches another bird than `b` sits on or is flying to (1 each). */
  private occupy(at: number, n: number, b: Bird): Uint8Array {
    if (this.taken.length < n) this.taken = new Uint8Array(n);
    const taken = this.taken;
    taken.fill(0, 0, n);
    for (const o of this.birds) if (o !== b && o.plant === at && o.state !== 'ground' && o.perch >= 0) taken[o.perch] = 1;
    return taken;
  }

  /** A spot on the lawn near `plant`, clear of people. */
  private lawnSpot(b: Bird, plant: Plant | undefined): [number, number] | null {
    if (!plant) return null;
    for (let k = 0; k < 10; k++) {
      const a = rnd() * Math.PI * 2;
      const r = plant.radius + 0.6 + rnd() * GAIT[b.sp.gait].reach;
      const x = plant.x + Math.cos(a) * r;
      const z = plant.z + Math.sin(a) * r;
      if (this.lawn(x, z)) return [x, z];
    }
    return null;
  }

  /** Where a perch is now, bent with its branch by the wind (each plant's sway found once a frame). */
  private perchPoint(plant: Plant, perch: number, out: Float32Array): void {
    if (plant.swayed !== this.frame) {
      plant.swayed = this.frame;
      plant.sway = plant.windX || plant.windZ ? this.view.sway(plant.x, plant.z) : 0;
    }
    const p = plant.perches;
    const h = plant.heights[perch];
    const bend = plant.sway * h * h;
    out[0] = p[perch * 3] + bend * plant.windX;
    out[1] = p[perch * 3 + 1];
    out[2] = p[perch * 3 + 2] + bend * plant.windZ;
  }

  /** Sets off to a perch (speed: times the species' cruise). */
  /** Sets off for `perch` of `plant`; false if it can't (then it waits a moment before deciding again). */
  private flyTo(b: Bird, plant: number, perch: number, speed: number): boolean {
    const target = this.plants[plant];
    if (!target) return false;
    this.perchPoint(target, perch, this.at);
    if (!this.plan(b, this.at[0], this.at[1], this.at[2], speed, false, plant === b.home)) {
      b.retryAt = this.clock + 1 + rnd() * 1.5;
      return false;
    }
    b.plant = plant;
    b.perch = perch;
    return true;
  }

  /** Sets off down to the lawn at (x, z). */
  private flyDown(b: Bird, x: number, z: number, speed: number): void {
    if (!this.lawn(x, z)) return;
    // (The town's ground is level, at 0.)
    if (!this.plan(b, x, 0, z, speed, true)) return;
    b.plant = -1;
    b.perch = -1;
  }

  /**
   * The flight's curve from where the bird is to (x, y, z): out and up, then in to land, lifted
   * over any house in the way. False (it stays) if the end is inside one.
   */
  private plan(b: Bird, x: number, y: number, z: number, speed: number, down: boolean, home = false): boolean {
    const sp = b.sp;
    const d = Math.hypot(x - b.x, z - b.z);
    // (Nothing further than the next garden or two, but home.)
    if (d > 40 && !home) return false;
    // Never to a spot inside a house (its roof ridge is on it).
    if (this.topAt(x, z) > y + 0.2) return false;
    const P = b.path;
    const hop = d < HOP_MAX && Math.abs(y - b.y) < 1.2;
    const [ux, uz] = d > 1e-3 ? [(x - b.x) / d, (z - b.z) / d] : [Math.sin(b.yaw), Math.cos(b.yaw)];
    const style = FLIGHT[sp.flight];
    const lift0 = hop ? 0.08 + d * 0.25 : style.lift[0] + d * style.lift[1];
    const lift1 = hop ? lift0 : style.land[down ? 0 : 1];
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
    // A bound every couple of metres (bounding; a blackbird's shallower ones).
    b.bounds = style.bound && !hop ? Math.max(1, Math.round(length / style.bound)) : 0;
    b.state = 'fly';
    b.turnTo = NaN;
    if (hop) this.stats.hops++;
    else this.stats.flights++;
    if (down) this.stats.lawn++;
    else if (this.debugging && this.topAt(x, z) > 0) this.stats.roof++;
    b.singUntil = -1;
    b.preenUntil = -1;
    return true;
  }

  private bezier(P: Float32Array, s: number, out: Float32Array): void {
    const u = 1 - s;
    const a = u * u * u;
    const c = 3 * u * u * s;
    const e = 3 * u * s * s;
    const f = s * s * s;
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
    const style = FLIGHT[sp.flight];
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
    // (Where it was: its heading and climb.)
    const px = b.x;
    const py = b.y;
    const pz = b.z;
    const x = this.at[0];
    let y = this.at[1];
    const z = this.at[2];
    // Bounding: rising while it flaps, falling with its wings shut.
    let flapping = true;
    if (b.bounds > 0) {
      const cycle = tau * b.bounds;
      y += style.boundAmp * Math.sin(Math.PI * tau) * Math.sin(cycle * Math.PI * 2);
      flapping = Math.cos(cycle * Math.PI * 2) > style.flapsAbove || tau < 0.15 || tau > 0.82;
    }
    b.x = x;
    b.y = y;
    b.z = z;
    // (A check: never through a house, short of perching on its ridge.)
    if (this.debugging && y < this.topAt(x, z) - 0.3) {
      this.stats.inside++;
      if (this.debugInside.length < 20) this.debugInside.push({ species: b.sp.id, at: [x, y, z].map((v) => Math.round(v * 100) / 100), top: this.topAt(x, z), path: Array.from(b.path, (v) => Math.round(v * 100) / 100), hop: b.hop });
    }
    // Heading and bank from the way it goes.
    const vx = (x - px) / dt;
    const vy = (y - py) / dt;
    const vz = (z - pz) / dt;
    const vh = Math.hypot(vx, vz);
    if (vh > 0.2) {
      const want = Math.atan2(vx, vz);
      const dy = turn(b.yaw, want);
      const rate = b.hop ? 18 : 7;
      const step = dy * Math.min(1, dt * rate);
      b.yaw += step;
      b.roll = lerp(b.roll, Math.max(-0.7, Math.min(0.7, (-step / dt) * 0.12 * Math.min(1, vh / 4))), Math.min(1, dt * 6));
    }
    const flare = smoothstep(0.78, 1, tau) * (b.hop ? 0.4 : 1);
    const climb = -Math.atan2(vy, Math.max(0.5, vh)) * 0.5;
    const level = b.look?.pitch ?? 0;
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
    } else if (style.glides && Math.sin(tau * b.dur * 1.6) > 0.55 && tau > 0.2) {
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
    b.bob = 0;
    b.tail = lerp(b.tail, flare * 0.5 + (flapping ? 0.05 : -0.1), Math.min(1, dt * 10));
    b.puff = 1;
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
      b.y = 0;
      b.leaveAt = this.clock + between(rnd, b.sp.groundStay) * (b.waryUntil > this.clock ? 0.3 : 1);
      b.nextStep = this.clock + 0.4 + rnd();
      b.steps = 0;
      b.stepT = b.stepDur = 0;
    }
  }

  /** Perched or roosting: on its branch as it sways, glancing, flicking, preening, singing, asleep. */
  private sit(b: Bird, dt: number): void {
    const plant = this.plants[b.plant];
    if (!plant) return;
    this.perchPoint(plant, b.perch, this.at);
    b.x = this.at[0];
    b.y = this.at[1];
    b.z = this.at[2];
    if (dt <= 0) return;
    const now = this.clock;
    const sp = b.sp;
    if (b.state === 'roost') {
      // Asleep: head turned back into its feathers, puffed up, still.
      this.ease(b, 2.5, 0.45, 0, dt, 3);
      b.puff = lerp(b.puff, 1.1, Math.min(1, dt * 0.5));
      b.tail = lerp(b.tail, -0.05, Math.min(1, dt));
      return;
    }
    b.puff = lerp(b.puff, b.singUntil > now ? 1.04 : 1, Math.min(1, dt * 2));
    if (!Number.isNaN(b.turnTo)) {
      // Turning round on the twig: quick, with a little lift off it.
      const left = turn(b.yaw, b.turnTo);
      b.yaw += left * Math.min(1, dt * 16);
      b.y += Math.min(0.02, Math.abs(left) * 0.012);
      if (Math.abs(left) < 0.02) b.turnTo = NaN;
    }
    if (b.singUntil > now) {
      // Singing: head up, a little sway of it.
      this.ease(b, Math.sin(now * 3 + b.id) * 0.25, -0.45, 0, dt, 10);
    } else if (b.preenUntil > now) {
      // Preening: head round to its back and wings, nibbling.
      const side = b.id % 2 ? 1 : -1;
      this.ease(b, side * 2.2 + Math.sin(now * 17) * 0.12, 0.55 + Math.sin(now * 23) * 0.1, side * 0.3, dt, 8);
    } else {
      if (now >= b.nextGlance) {
        b.nextGlance = now + between(rnd, sp.glance) * (this.calm ? 3 : 1);
        const back = rnd() < 0.25;
        b.hyTo = back ? 0 : (rnd() - 0.5) * 2.2;
        b.hpTo = back ? 0 : (rnd() - 0.6) * 0.5;
        b.hrTo = sp.cock && rnd() < 0.3 ? (rnd() - 0.5) * 0.8 : 0;
      }
      this.ease(b, b.hyTo, b.hpTo, b.hrTo, dt, this.calm ? 3 : sp.snap);
      if (now >= b.nextPreen) {
        b.nextPreen = now + between(rnd, sp.preen);
        b.preenUntil = now + 1.5 + rnd() * 2.5;
      }
    }
    this.flick(b, now);
  }

  /** The tail's flick (up fast, down slower; a blackbird holds it up longer). */
  private flick(b: Bird, now: number): void {
    const sp = b.sp;
    if (sp.flick && !this.calm && now >= b.nextFlick) {
      b.nextFlick = now + between(rnd, sp.flick);
      b.flickAt = now;
    }
    const t = now - b.flickAt;
    b.tail = t < 0 || t > 2 ? 0 : (t < 0.05 ? t / 0.05 : Math.exp(-(t - 0.05) / sp.flickHold)) * sp.flickAmp;
  }

  /** Turns the head towards a pose, `rate` per second (high: a snap). */
  private ease(b: Bird, y: number, p: number, r: number, dt: number, rate: number): void {
    const k = Math.min(1, dt * rate);
    b.hy += (y - b.hy) * k;
    b.hp += (p - b.hp) * k;
    b.hr += (r - b.hr) * k;
  }

  /** On the lawn: hopping, running or walking about, pecking, looking up. */
  private forage(b: Bird, dt: number): void {
    if (dt <= 0) return;
    const now = this.clock;
    const sp = b.sp;
    const gait = GAIT[sp.gait];
    if (b.stepT < b.stepDur) {
      // A move under way.
      b.stepT = Math.min(b.stepDur, b.stepT + dt);
      const s = b.stepT / b.stepDur;
      const S = b.step;
      if (gait.bobs) {
        // Walking at an even pace, the head thrust forward, then held still as the body catches up.
        b.x = lerp(S[0], S[2], s);
        b.z = lerp(S[1], S[3], s);
        const pace = (s * b.steps) % 1;
        b.bob = (pace < 0.4 ? pace / 0.4 : 1 - (pace - 0.4) / 0.6) * 0.075 * (b.look?.length ?? 0);
        b.y = 0;
      } else {
        const hops = Math.max(1, b.steps);
        const u = s * hops;
        const k = Math.min(hops - 1, Math.floor(u));
        const f = u - k;
        const e = (k + f * f * (3 - 2 * f)) / hops;
        b.x = lerp(S[0], S[2], e);
        b.z = lerp(S[1], S[3], e);
        b.y = Math.sin(Math.PI * f) * gait.lift;
      }
      b.yaw += turn(b.yaw, Math.atan2(S[2] - S[0], S[3] - S[1])) * Math.min(1, dt * 20);
      this.ease(b, 0, gait.headPitch, 0, dt, 12);
      return;
    }
    b.bob = lerp(b.bob, 0, Math.min(1, dt * 8));
    if (b.peck >= 0) {
      // Pecks: head down and up, quickly.
      b.peck += dt;
      const n = Math.floor(b.peck / 0.22);
      const f = (b.peck % 0.22) / 0.22;
      b.hp = f < 0.4 ? f / 0.4 : 1 - (f - 0.4) / 0.6;
      b.hy = lerp(b.hy, 0, Math.min(1, dt * 10));
      b.pitch = lerp(b.pitch, 0.2, Math.min(1, dt * 8));
      if (n >= b.steps) {
        b.peck = -1;
        b.pitch = 0;
      }
      return;
    }
    b.pitch = lerp(b.pitch, 0, Math.min(1, dt * 8));
    if (now < b.nextStep) {
      // Looking about (a blackbird stands tall and cocks its head to listen).
      if (now >= b.nextGlance) {
        b.nextGlance = now + between(rnd, sp.glance) * (this.calm ? 3 : 1);
        b.hyTo = (rnd() - 0.5) * 1.6;
        b.hpTo = gait.listens ? -0.25 : (rnd() - 0.6) * 0.4;
        b.hrTo = gait.listens && rnd() < 0.5 ? (rnd() - 0.5) * 0.9 : 0;
      }
      this.ease(b, b.hyTo, b.hpTo, b.hrTo, dt, this.calm ? 3 : sp.snap);
      this.flick(b, now);
      return;
    }
    // Next: a peck or two, or a move (by the seed, more pecks, and moves towards it).
    const seed = this.seedNear(b, 6);
    const sx = seed >= 0 ? this.seed[seed * 3] : 0;
    const sz = seed >= 0 ? this.seed[seed * 3 + 1] : 0;
    const toSeed = seed >= 0 ? Math.hypot(sx - b.x, sz - b.z) : Infinity;
    if (rnd() < (toSeed < 1.4 ? 0.8 : gait.pecks)) {
      b.peck = 0;
      b.steps = 1 + Math.floor(rnd() * gait.peckMax);
      b.nextStep = now + 0.3 + rnd() * 0.6;
      return;
    }
    const hops = gait.steps[0] + Math.floor(rnd() * gait.steps[1]);
    // Keep near where it came down: wander, turning back towards its tree's side as it strays.
    const home = this.plants[b.home];
    const away = Math.hypot(b.x - home.x, b.z - home.z);
    let dir = b.yaw + (rnd() - 0.5) * 2.2;
    if (away > home.radius + 6) dir = Math.atan2(home.x - b.x, home.z - b.z) + (rnd() - 0.5);
    if (toSeed < Infinity && toSeed > 0.6) dir = Math.atan2(sx - b.x, sz - b.z) + (rnd() - 0.5) * 0.8;
    for (let k = 0; k < 6; k++) {
      const tx = b.x + Math.sin(dir) * hops * gait.stride;
      const tz = b.z + Math.cos(dir) * hops * gait.stride;
      if (this.lawn(tx, tz)) {
        const S = b.step;
        [S[0], S[1], S[2], S[3]] = [b.x, b.z, tx, tz];
        b.steps = hops;
        b.stepT = 0;
        b.stepDur = hops * gait.pace;
        break;
      }
      dir += Math.PI / 3 + rnd();
    }
    b.nextStep = now + b.stepDur + between(rnd, gait.pause);
  }

  // ---- drawing ------------------------------------------------------------------------------

  /**
   * Writes a bird's matrices into its species' buffers: the parts whose pose changed since they were
   * last written (all of them if it moved, the wings only while spread). Whether it wrote any.
   */
  private write(b: Bird): boolean {
    const look = b.look;
    if (!look || b.slot >= look.capacity) return false;
    const w = b.drawn;
    const moved =
      changed(w[Pose.X], b.x, STILL_M) || changed(w[Pose.Y], b.y, STILL_M) || changed(w[Pose.Z], b.z, STILL_M) ||
      changed(w[Pose.Yaw], b.yaw, STILL_RAD) || changed(w[Pose.Pitch], b.pitch, STILL_RAD) || changed(w[Pose.Roll], b.roll, STILL_RAD);
    let parts = moved ? ALL_PARTS : 0;
    if (!moved) {
      if (changed(w[Pose.Puff], b.puff, STILL_SCALE)) parts |= 1 << BODY;
      if (changed(w[Pose.Hy], b.hy, STILL_RAD) || changed(w[Pose.Hp], b.hp, STILL_RAD) || changed(w[Pose.Hr], b.hr, STILL_RAD) || changed(w[Pose.Bob], b.bob, STILL_M)) parts |= 1 << HEAD;
      if (changed(w[Pose.Tail], b.tail, STILL_RAD)) parts |= 1 << TAIL;
    }
    // Shut wings are hidden (scaled to nothing): written as they shut, open, or beat.
    const wings = changed(w[Pose.Spread], b.spread, STILL_SCALE) || (b.spread > 0 && (moved || changed(w[Pose.Flap], b.flap, STILL_RAD)));
    if (wings) parts |= (1 << WING_L) | (1 << WING_R);
    else parts &= ~((1 << WING_L) | (1 << WING_R));
    if (!parts) return false;
    const o = b.slot * 16;
    const s = b.size;
    // Placed by the body's middle (it pitches and rolls about that), feet down.
    quatYawPitchRoll(b.yaw, b.pitch, b.roll, qTurn);
    compose(s, s, s, qTurn, b.x, b.y + look.centre * s, b.z, mWorld);
    multiply(translation(0, -look.centre, 0, mPose), mWorld, mBase);
    if (parts & (1 << BODY)) {
      // Puffed up about its middle.
      scaling(b.puff, b.puff, b.puff, mPose);
      vBody[1] = look.centre;
      put(about(mPose, vBody), look.buffers[BODY], o);
      w[Pose.Puff] = b.puff;
    }
    if (parts & (1 << HEAD)) {
      // Yaw, pitch and a cock of it about the neck, and a dove's bob.
      rotationYawPitchRoll(b.hy, b.hp, b.hr, mPose);
      put(about(mPose, look.pivots[HEAD], b.bob), look.buffers[HEAD], o);
      w[Pose.Hy] = b.hy;
      w[Pose.Hp] = b.hp;
      w[Pose.Hr] = b.hr;
      w[Pose.Bob] = b.bob;
    }
    if (parts & (1 << TAIL)) {
      // Up and down about its base.
      rotationX(b.tail, mPose);
      put(about(mPose, look.pivots[TAIL]), look.buffers[TAIL], o);
      w[Pose.Tail] = b.tail;
    }
    if (wings) {
      // Beating about the shoulders (up: the tips rise), hidden while shut.
      for (let part = WING_L; part <= WING_R; part++) {
        multiply(scaling(b.spread, b.spread, b.spread, mPose), rotationZ((part === WING_L ? -1 : 1) * b.flap, mTurn), mPose);
        put(about(mPose, look.pivots[part]), look.buffers[part], o);
      }
      w[Pose.Spread] = b.spread;
      w[Pose.Flap] = b.flap;
    }
    if (moved) {
      w[Pose.X] = b.x;
      w[Pose.Y] = b.y;
      w[Pose.Z] = b.z;
      w[Pose.Yaw] = b.yaw;
      w[Pose.Pitch] = b.pitch;
      w[Pose.Roll] = b.roll;
    }
    look.dirty |= parts;
    return true;
  }

  /** The species' model, its nodes as parts (no meshes without its file, or if that fails: built again when it comes). */
  private lookOf(sp: Species): Promise<L> {
    return this.view.look(sp.id, `model.bird.${sp.id}`);
  }

  /** Gives a species' parts room for `count` birds (each written whole when it's next placed). */
  private size(look: L, count: number): void {
    look.count = count;
    if (count > look.capacity || count === 0) {
      look.capacity = Math.max(1, count);
      look.buffers = PARTS.map(() => new Float32Array(look.capacity * 16));
    }
    this.view.setBuffers(look, this.visible);
    for (const b of this.birds) if (b.look === look) b.drawn.fill(NaN);
  }
}

/** The rotation matrix of a yaw, pitch and roll: its quaternion's (`quatYawPitchRoll`), as the original `Matrix.RotationYawPitchRollToRef` makes it. */
function rotationYawPitchRoll(yaw: number, pitch: number, roll: number, out: Float32Array): Float32Array {
  const q = quatYawPitchRoll(yaw, pitch, roll, qHead);
  const xx = q[0] * q[0];
  const yy = q[1] * q[1];
  const zz = q[2] * q[2];
  const xy = q[0] * q[1];
  const zw = q[2] * q[3];
  const zx = q[2] * q[0];
  const yw = q[1] * q[3];
  const yz = q[1] * q[2];
  const xw = q[0] * q[3];
  out[0] = 1 - 2 * (yy + zz);
  out[1] = 2 * (xy + zw);
  out[2] = 2 * (zx - yw);
  out[3] = 0;
  out[4] = 2 * (xy - zw);
  out[5] = 1 - 2 * (zz + xx);
  out[6] = 2 * (yz + xw);
  out[7] = 0;
  out[8] = 2 * (zx + yw);
  out[9] = 2 * (yz - xw);
  out[10] = 1 - 2 * (yy + xx);
  out[11] = 0;
  out[12] = out[13] = out[14] = 0;
  out[15] = 1;
  return out;
}
const qHead: Quat = [0, 0, 0, 1];

/**
 * The viewed house's roof ridge as the birds' perches: the roofs' highest points (`each` calls its
 * function with every roof vertex, in the world), a perch every half metre along them from end to
 * end; and the roof's top (m), null without a roof.
 */
export function roofRidge(each: (fn: (x: number, y: number, z: number) => void) => void): { ridge: number[]; top: number | null } {
  let top = -Infinity;
  each((_x, y) => (top = Math.max(top, y)));
  if (top === -Infinity) return { ridge: [], top: null };
  // Its ends: the top points' extent along it (a house's ridge runs along x or z), down its middle.
  let [x0, x1, z0, z1] = [Infinity, -Infinity, Infinity, -Infinity];
  each((x, y, z) => {
    if (y <= top - 0.03) return;
    x0 = Math.min(x0, x);
    x1 = Math.max(x1, x);
    z0 = Math.min(z0, z);
    z1 = Math.max(z1, z);
  });
  const [mx, mz] = [(x0 + x1) / 2, (z0 + z1) / 2];
  const [a, b] = x1 - x0 >= z1 - z0 ? [{ x: x0, y: top, z: mz }, { x: x1, y: top, z: mz }] : [{ x: mx, y: top, z: z0 }, { x: mx, y: top, z: z1 }];
  const ridge: number[] = [];
  const n = Math.floor(Math.hypot(b.x - a.x, b.z - a.z) / 0.5);
  // A short ridge (a hipped roof's): its ends, and the point between (in order along it, as hops along it go).
  if (n < 3) ridge.push(a.x, a.y, a.z, (a.x + b.x) / 2, a.y, (a.z + b.z) / 2, b.x, b.y, b.z);
  else for (let k = 1; k < n; k++) ridge.push(a.x + ((b.x - a.x) * k) / n, a.y, a.z + ((b.z - a.z) * k) / n);
  return { ridge, top: Math.max(0, top) };
}

/** The garden things outdoors that are plants as placed (catalog trees, hedges, roses; a fiddle-leaf fig in the lounge isn't a bird's). */
export function outdoorPlants(world: WorldStructure, content: Content, assets: AssetRegistry): PlantPlacement[] {
  const plants: PlantPlacement[] = [];
  const gd = world.storeyDepth ?? world.depth;
  for (const o of world.objects) {
    if (o.z >= gd) continue;
    const centre = objectCentre(o);
    if (world.rooms[Math.floor(centre.z) * world.width + Math.floor(centre.x)] !== 0) continue;
    const key = content.object(o.def)?.model ?? `model.${o.def}`;
    const entry = assets.get(key, 'model');
    if (!entry) continue;
    const { turn, size } = placementVariation(entry.vary ?? 0, o.x, o.z);
    plants.push({ key, x: centre.x, y: o.lift ?? 0, z: centre.z, sx: size, sy: size, sz: size, yaw: (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn });
  }
  return plants;
}
