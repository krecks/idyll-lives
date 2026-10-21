/**
 * Dev-only pose bench (`pnpm dev`, then /poses.html): Sims at real furniture, driven through the
 * game's own `Characters` with a synthetic snapshot, to check activity animations without
 * setting up a game. Not part of the build.
 *
 * Each station is one object and one Sim (two for social interactions) doing an activity tag;
 * a station with a `script` cycles through poses (sitting down, getting into bed, ...).
 *
 * Objects are as the game has them: every content file `base.json` includes, with their footprints,
 * slots and models.
 *
 * URL: `?only=bed,water` (stations whose label, object id or tag is one of these, exactly), `?cols=6`,
 * `?gap=3.4`, `?focus=<n | label | object id>` (frame that station close up; one that isn't there
 * says so on the page and in `__poses.error`), `?alpha=…&beta=…&radius=…`, `?labels=0`,
 * `?manual` (time only advances via `__poses.advance(seconds)`, for screenshots), `?seats=sofa,toilet`
 * (one resident a slot on each, men and women in turn, sitting; an object that only opens to be used, a
 * wall bed or a futon, open), `?seats=sofaBed:doze,chessTable:play` (doing that use, by interaction id
 * or animation tag: its pose, its clip, the object's clips for it held open, and open if the use opens
 * it), `?beds=wallBed,sofaBed:doze` (the same, the first use lying down unless one is named),
 * `?clip=walk` (every Sim plays that clip: `__characters.force`), `?clip=walk&strip=8` (a filmstrip:
 * the clip held at 8 times through it, on a man, a woman, a child, a teen and an elder; no stations),
 * `?bottom=bottom.skirt` (everyone who can wears that bottom),
 * `?stage=teen` (a life stage: baby, child, teen, youngAdult, adult or elder) everyone placed is at,
 * where sim-core lets that stage take the place (a baby only lies in a crib; children and teens
 * don't care for a baby, flirt or kiss), with the game's look for it (stoop, grey hair); with
 * `strip`, the rows are a man and a woman at that stage.
 */

import {
  ArcRotateCamera,
  Color3,
  Color4,
  DirectionalLight,
  Engine,
  HemisphericLight,
  ImageProcessingConfiguration,
  Matrix,
  MeshBuilder,
  PBRMaterial,
  Scene,
  ShadowGenerator,
  Vector3,
} from '../render/babylon/core';
import { AssetRegistry } from '../assets/registry';
import { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import type { ObjectPlacement, SimInfo } from '../core/protocol';
import { Pose, type SnapshotLayout } from '../core/snapshot';
import { lookAtStage, palette } from '../game/household';
import { Characters, MAX_CHARACTERS } from '../render/babylon/characters';
import { MaterialLibrary } from '../render/babylon/materials';
import { buildModel, motionsOf, partPose } from '../render/babylon/models';
import { slotPlace } from '../render/babylon/objectMotion';
import { installNature } from '../render/babylon/nature';

const P = Pose;
/** Conversation animation codes (`social::ANIMATIONS` + 1). */
const SOCIAL = { talk: 1, laugh: 2, flirt: 3, argue: 4, fight: 5, hug: 6, kiss: 7, wave: 8 } as const;

interface Step {
  seconds: number;
  pose: number;
  /** Activity tag while in this step ('' = none). */
  tag?: string;
}
interface Station {
  label: string;
  def?: string;
  tag?: string;
  pose?: number;
  social?: keyof typeof SOCIAL;
  /** Cycles through these steps (overrides `pose` / `tag`). */
  script?: Step[];
  female?: boolean;
  /** Filmstrip frame: the clip time as a share of its length (`?strip`), and the life stage. */
  strip?: number;
  stage?: string;
  /** Carries this item (content `items` id). */
  held?: string;
  /** Walks to and fro across the cell. */
  walk?: boolean;
  /** A baby lies (awake, or 'asleep') in the object's first slot (a crib); the station's Sim sees to them from the front. */
  baby?: boolean | 'asleep';
  /** Only the baby: no grown-up at the crib. */
  alone?: boolean;
  /** A second Sim (a woman) does the same in the object's second slot (two in a double bed). */
  pair?: boolean;
  /** One Sim a slot of the object (`?seats`, `?beds`), a man, then a woman, and so on. */
  seated?: boolean;
  /** The object drawn open (a wall bed folded down), and its clips for this use (interaction id or tag) held open. */
  open?: boolean;
  clips?: string[];
  /** The station's Sim's height and build (appearance; 1 = average). */
  height?: number;
  build?: number;
}

/** At the crib: a cuddle (as long as the content's, 15 minutes: picked up, held, put back), and standing by. */
const CUDDLING: Step[] = [
  { seconds: 3, pose: P.Stand, tag: 'idle' },
  { seconds: 15, pose: P.Stand, tag: 'cuddle' },
  { seconds: 3, pose: P.Stand, tag: 'idle' },
];

const STATIONS: Station[] = [
  // Gardening
  { label: 'water flowers', def: 'garden.tulips', tag: 'water' },
  { label: 'water houseplant', def: 'garden.monstera', tag: 'water' },
  { label: 'tend veg (weed)', def: 'garden.vegPatch', tag: 'tend' },
  { label: 'harvest tomatoes', def: 'garden.tomatoes', tag: 'harvest' },
  { label: 'prune roses', def: 'garden.roses', tag: 'prune' },
  { label: 'trim hedge', def: 'garden.hedge', tag: 'prune' },
  { label: 'garden (legacy tag)', def: 'flowerbed', tag: 'garden' },
  // Looking
  { label: 'smell flowers', def: 'garden.tulips', tag: 'smell', female: true },
  { label: 'admire blossom', def: 'garden.cherry', tag: 'look' },
  { label: 'look birdbath', def: 'garden.birdbath', tag: 'look', female: true },
  { label: 'talk to plant', def: 'garden.monstera', tag: 'talk', female: true },
  // Lying
  { label: 'sleep', def: 'bed', tag: 'sleep', pose: P.Lie },
  { label: 'sleep two', def: 'bedDouble', tag: 'sleep', pose: P.Lie, pair: true },
  { label: 'nap hammock', def: 'natureLover.gardenHammock', tag: 'nap', pose: P.Lie, female: true },
  { label: 'swing hammock', def: 'cheerful.rainbowHammock', tag: 'relax', pose: P.Lie },
  { label: 'bench press', def: 'weightBench', tag: 'lift', pose: P.Lie },
  { label: 'hammock', def: 'hammock', tag: 'relax', pose: P.Lie, female: true },
  { label: 'car bed', def: 'kidsBedCar', tag: 'sleep', pose: P.Lie, stage: 'child' },
  { label: 'kids bed', def: 'kidsBed', tag: 'sleep', pose: P.Lie, stage: 'child' },
  // The baby's crib: on their own (asleep, and awake), feeding (kneeling at it), and a cuddle (picked up, rocked, put back).
  { label: 'baby asleep in crib', def: 'crib', baby: 'asleep', alone: true },
  { label: 'baby awake in crib', def: 'crib', baby: true, alone: true },
  { label: 'feed baby', def: 'crib', tag: 'tend', baby: true, female: true },
  { label: 'cuddle baby', def: 'crib', script: CUDDLING, baby: true, female: true },
  { label: 'cuddle baby tall broad', def: 'crib', script: CUDDLING, baby: true, height: 1.15, build: 1.2 },
  { label: 'cuddle baby short slim', def: 'crib', script: CUDDLING, baby: true, female: true, height: 0.85, build: 0.85 },
  { label: 'cuddle baby elder', def: 'crib', script: CUDDLING, baby: true, stage: 'elder' },
  // Cut short (the sim ends it early): the baby settles back into the crib.
  {
    label: 'cuddle baby cut short',
    def: 'crib',
    baby: true,
    script: [
      { seconds: 2, pose: P.Stand, tag: 'idle' },
      { seconds: 7, pose: P.Stand, tag: 'cuddle' },
      { seconds: 3, pose: P.Stand, tag: 'idle' },
    ],
  },
  {
    label: 'get into / out of bed',
    def: 'bed',
    female: true,
    script: [
      { seconds: 2.5, pose: P.Stand },
      { seconds: 6, pose: P.Lie, tag: 'sleep' },
      { seconds: 4, pose: P.Stand },
    ],
  },
  {
    label: 'sit down / stand up',
    def: 'armchair',
    script: [
      { seconds: 2.5, pose: P.Stand },
      { seconds: 5, pose: P.Sit, tag: 'sit' },
      { seconds: 4, pose: P.Stand },
    ],
  },
  // Seats
  { label: 'sofa', def: 'sofa', tag: 'sit', pose: P.Sit },
  { label: 'tv sofa', def: 'sofaThree', tag: 'watch', pose: P.Sit, female: true },
  { label: 'dining chair', def: 'diningChair', tag: 'sit', pose: P.Sit, female: true },
  { label: 'bar stool', def: 'barStool', tag: 'sit', pose: P.Sit },
  { label: 'office chair', def: 'officeChair', tag: 'sit', pose: P.Sit, female: true },
  { label: 'eat dining table', def: 'diningTable', tag: 'eat', pose: P.Sit },
  { label: 'eat dining oak', def: 'diningOak', tag: 'eat', pose: P.Sit, female: true },
  { label: 'child at desk', def: 'kidsDesk', tag: 'write', pose: P.Sit, stage: 'child' },
  { label: 'elder armchair', def: 'armchair', tag: 'relax', pose: P.Sit, stage: 'elder', female: true },
  // Kitchen and carrying
  { label: 'chop', def: 'counter', tag: 'chop' },
  { label: 'stir', def: 'stoveGas', tag: 'stir', female: true },
  { label: 'flip', def: 'grill', tag: 'flip' },
  { label: 'carry plate', held: 'mealPlate', walk: true, female: true },
  { label: 'carry pot', held: 'mealPot', walk: true },
  { label: 'hold ingredients', held: 'ingredients' },
  { label: 'child carries', held: 'sandwich', walk: true, stage: 'child' },
  { label: 'walk elder', walk: true, stage: 'elder' },
  // Eating / drinking
  { label: 'eat (fridge)', def: 'fridge', tag: 'eat' },
  { label: 'drink (espresso)', def: 'foodie.espressoBar', tag: 'drink', female: true },
  { label: 'eat seated', def: 'romantic.candlelitTable', tag: 'eat', pose: P.Sit },
  // Seated
  { label: 'relax recliner', def: 'couchPotato.recliner', tag: 'relax', pose: P.Sit },
  { label: 'bath', def: 'whirlpoolTub', tag: 'bath', pose: P.Sit, female: true },
  { label: 'toilet', def: 'toilet', tag: 'toilet', pose: P.Sit },
  { label: 'meditate', def: 'hotHeaded.coolDownCushion', tag: 'meditate', pose: P.Sit, female: true },
  { label: 'read seated', def: 'bookworm.readingNook', tag: 'read', pose: P.Sit },
  { label: 'type', def: 'computerDesk', tag: 'type', pose: P.Sit, female: true },
  { label: 'piano', def: 'piano', tag: 'music', pose: P.Sit },
  { label: 'chaise feet up', def: 'chaise', tag: 'relax', pose: P.Sit, female: true },
  { label: 'exercise bike', def: 'exerciseBike', tag: 'exercise', pose: P.Sit },
  { label: 'cello', def: 'gloomy.cello', tag: 'music', pose: P.Sit, female: true },
  { label: 'drums', def: 'hotHeaded.thunderDrums', tag: 'music', pose: P.Sit },
  // Standing work
  { label: 'cook', def: 'chefStove', tag: 'cook', female: true },
  { label: 'wash (sink)', def: 'sink', tag: 'wash' },
  { label: 'paint', def: 'easel', tag: 'paint', female: true },
  { label: 'tinker', def: 'workbench', tag: 'tinker' },
  { label: 'read bookshelf', def: 'bookshelf', tag: 'read', female: true },
  { label: 'mop', def: 'neat.steamMopDock', tag: 'clean' },
  { label: 'laundry', def: 'neat.laundryCenter', tag: 'clean', female: true },
  { label: 'sing', def: 'cheerful.karaokeMachine', tag: 'sing', female: true },
  // Showers (used standing inside)
  { label: 'shower', def: 'shower', tag: 'shower' },
  { label: 'shower stall', def: 'showerStall', tag: 'shower', female: true },
  { label: 'shower walk-in', def: 'showerWalkIn', tag: 'shower' },
  { label: 'shower cedar', def: 'natureLover.cedarShower', tag: 'shower', female: true },
  { label: 'shower contrast', def: 'energetic.contrastShower', tag: 'shower' },
  { label: 'shower rain tower', def: 'neat.rainShowerTower', tag: 'shower', female: true },
  // Exercise
  { label: 'trampoline', def: 'energetic.trampoline', tag: 'exercise' },
  { label: 'treadmill', def: 'treadmill', tag: 'run', female: true },
  { label: 'dumbbells', def: 'dumbbells', tag: 'lift' },
  { label: 'guitar acoustic', def: 'guitarAcoustic', tag: 'music', female: true },
  { label: 'guitar electric', def: 'guitarElectric', tag: 'music' },
  { label: 'ukulele', def: 'ukulele', tag: 'music', stage: 'child' },
  { label: 'ukulele grown-up', def: 'ukulele', tag: 'music', female: true },
  { label: 'bouldering', def: 'energetic.boulderingWall', tag: 'exercise', female: true },
  { label: 'vibe plate', def: 'lazy.vibePlate', tag: 'exercise' },
  { label: 'heavy bag', def: 'hotHeaded.heavyBag', tag: 'exercise' },
  { label: 'streetball', def: 'energetic.streetballHoop', tag: 'exercise', female: true },
  { label: 'sunrise lamp', def: 'cheerful.sunriseLamp', tag: 'exercise' },
  { label: 'bouldering child', def: 'energetic.boulderingWall', tag: 'exercise', stage: 'child' },
  { label: 'bouldering teen', def: 'energetic.boulderingWall', tag: 'exercise', stage: 'teen' },
  { label: 'smash plates', def: 'hotHeaded.smashBin', tag: 'exercise' },
  { label: 'smash plates child', def: 'hotHeaded.smashBin', tag: 'exercise', stage: 'child', female: true },
  { label: 'yoga', def: 'yogaMat', tag: 'exercise', female: true },
  { label: 'yoga elder', def: 'yogaMat', tag: 'exercise', stage: 'elder' },
  // Games
  { label: 'darts', def: 'dartboard', tag: 'play' },
  { label: 'pool', def: 'poolTable', tag: 'play', female: true },
  { label: 'pool child', def: 'poolTable', tag: 'play', stage: 'child' },
  { label: 'foosball', def: 'foosball', tag: 'play' },
  { label: 'foosball child', def: 'foosball', tag: 'play', stage: 'child', female: true },
  { label: 'ping-pong', def: 'pingPongTable', tag: 'play', female: true },
  // Socials
  { label: 'hug', social: 'hug' },
  { label: 'kiss', social: 'kiss' },
  { label: 'wave goodbye', social: 'wave' },
  { label: 'laugh', social: 'laugh' },
  { label: 'flirt', social: 'flirt' },
  { label: 'argue', social: 'argue' },
  { label: 'fight', social: 'fight' },
];

const params = new URLSearchParams(location.search);
const only = params.get('only')?.split(',').filter(Boolean);
const forced = params.get('clip');
const strip = Number(params.get('strip')) || 0;
const stage = params.get('stage');
const STRIP_ROWS: { label: string; female: boolean; stage?: string }[] = stage
  ? [
      { label: `${stage} man`, female: false, stage },
      { label: `${stage} woman`, female: true, stage },
    ]
  : [
      { label: 'man', female: false },
      { label: 'woman', female: true },
      { label: 'child', female: false, stage: 'child' },
      { label: 'teen', female: true, stage: 'teen' },
      { label: 'elder', female: false, stage: 'elder' },
    ];
/** Whether sim-core lets a resident at life stage `st` take actor `a`'s place at station `s`. */
const allows = (s: Station, a: Actor, st: string): boolean => {
  if (a.baby || st === 'baby') return !!a.baby && st === 'baby';
  const grownUp = st !== 'child' && st !== 'teen';
  return grownUp || (!s.baby && s.social !== 'flirt' && s.social !== 'kiss');
};
// The content as the game loads it (`base.json` and every file it includes): object defs (model,
// footprint, slots, uses), the seats it describes, the items Sims carry and the life stages.
const base = import.meta.env.BASE_URL;
const content = await Content.load(`${base}content/base.json`);
/** What the page couldn't do as asked (an object or station that isn't there), shown on it and in `__poses.error`. */
const problems: string[] = [];

/**
 * `?seats=<def>[:<use>]` (`lie` false) or `?beds=…` (true): one Sim a slot of the object, doing the
 * use named (an interaction id, else an animation tag), or else sitting (`seats`) or its first use
 * lying down (`beds`). Drawn open if that use opens it, or, sitting, if only a use that opens it
 * puts anyone on it (a futon).
 */
function seatStation(spec: string, lie: boolean): Station {
  const [id, use] = spec.split(':');
  const def = content.object(id);
  if (!def) problems.push(`no object '${id}'`);
  const uses = def?.interactions ?? [];
  const it = use ? uses.find((u) => u.id === use) : lie ? uses.find((u) => u.pose === 'lie') : undefined;
  if (lie && !use && !it) problems.push(`${id}: no use lying down`);
  if (use && !it && !content.animations.includes(use)) problems.push(`${spec}: neither a use of ${id} nor an animation tag`);
  const pose = it ? (it.pose === 'lie' ? P.Lie : it.pose === 'sit' ? P.Sit : P.Stand) : lie ? P.Lie : P.Sit;
  const tag = it ? (it.anim ?? (pose === P.Lie ? 'sleep' : pose === P.Sit ? 'sit' : '')) : (use ?? (lie ? 'sleep' : 'sit'));
  // Whether it is open while this goes on: the use's own `leaves` / `when`, or for the plain sit, that of the way it is sat on (or else lain on).
  const way = it ?? uses.find((u) => u.pose === 'sit') ?? uses.find((u) => u.pose === 'lie');
  const open = way?.leaves === 'open' || way?.when === 'open';
  return { label: spec, def: id, tag, pose, seated: true, open, clips: it || use ? [it?.id ?? use, tag] : undefined };
}
const seats = params.get('seats')?.split(',').filter(Boolean);
const beds = params.get('beds')?.split(',').filter(Boolean);
const bottom = params.get('bottom');
/** `?only=`: stations whose label, object id or activity tag is one of these. */
const named = (o: string, s: Station) => o === s.label || o === s.def || o === s.tag;
const stations: Station[] = strip
  ? STRIP_ROWS.flatMap((r) => Array.from({ length: strip }, (_, k) => ({ label: `${r.label} ${k}/${strip}`, female: r.female, stage: r.stage, strip: k / strip })))
  : seats || beds
    ? [...(seats ?? []).map((s) => seatStation(s, false)), ...(beds ?? []).map((s) => seatStation(s, true))]
    : only
      ? STATIONS.filter((s) => only.some((o) => named(o, s)))
      : STATIONS;
for (const o of only && !strip && !seats && !beds ? only : []) {
  if (!STATIONS.some((s) => named(o, s))) problems.push(`only=${o}: no station with that label, object id or tag`);
}

const canvas = document.getElementById('c') as HTMLCanvasElement;
const engine = new Engine(canvas, true, { preserveDrawingBuffer: true, stencil: true });
const scene = new Scene(engine);
scene.clearColor = new Color4(0.62, 0.77, 0.9, 1);
scene.imageProcessingConfiguration.toneMappingEnabled = true;
scene.imageProcessingConfiguration.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
scene.imageProcessingConfiguration.exposure = 1.1;
installNature(scene);
const sky = new HemisphericLight('sky', new Vector3(0, 1, 0), scene);
sky.intensity = 0.6;
sky.groundColor = new Color3(0.32, 0.36, 0.25);
const sun = new DirectionalLight('sun', new Vector3(-0.45, -1, -0.35), scene);
sun.intensity = 3;
sun.diffuse = new Color3(1, 0.96, 0.88);
const shadowGen = new ShadowGenerator(2048, sun);
shadowGen.usePercentageCloserFiltering = true;
shadowGen.bias = 0.002;

const assets = await AssetRegistry.load(`${base}assets/manifest.json`, params.getAll('pack'));
const lib = MaterialLibrary.for(scene).setAssets(assets);
const env = assets.get('environment.sky', 'image');
if (env?.url) await lib.loadEnvironment(env.url);

// The animation tags, and those the stations use that the content doesn't list.
const actions = [...content.animations];
for (const s of stations) for (const t of [s.tag, ...(s.script ?? []).map((x) => x.tag)]) if (t && !actions.includes(t)) actions.push(t);

// Layout: stations in a grid, each object at its cell's centre facing +z.
const cols = Number(params.get('cols')) || (strip ? strip : Math.ceil(Math.sqrt(stations.length * 1.4)));
// (Seats and beds: room past the largest, a wall bed folded down clear of the next.)
const largest = Math.max(0, ...stations.filter((s) => s.seated).map((s) => Math.max(...(content.object(s.def!)?.footprint ?? [1, 1]))));
const gap = Number(params.get('gap')) || (strip ? 1.1 : Math.max(3.4, largest + 1.6));
const rows = Math.ceil(stations.length / cols);
const ground = MeshBuilder.CreateGround('ground', { width: cols * gap + 6, height: rows * gap + 6 }, scene);
const gm = new PBRMaterial('ground', scene);
gm.albedoColor = Color3.FromHexString('#8E9A7A').toLinearSpace();
gm.roughness = 1;
gm.metallic = 0;
ground.material = gm;
ground.receiveShadows = true;

interface Actor {
  station: number;
  x: number;
  z: number;
  yaw: number;
  object: number;
  partner: number;
  role: number;
  /** Standing use: the front tile's centre, facing the object; or its slot's own place (content `places`), facing as it says. */
  frontX?: number;
  frontZ?: number;
  frontYaw?: number;
  /** Used standing inside (a shower): on the slot, facing out, like sitting. */
  inside?: boolean;
  /** The baby lying in the station's crib. */
  baby?: boolean;
  /** A woman: the station's second Sim (`Station.pair`), or every other one of `Station.seated`. */
  female?: boolean;
}
const objects: ObjectPlacement[] = [];
const actors: Actor[] = [];
const labels = document.getElementById('labels')!;
const spots: { at: Vector3; el: HTMLSpanElement }[] = [];
const centres: Vector3[] = [];
const posed = new Float32Array(16);
const look = (id: number, female: boolean, stage?: string, height = 1, build = 1): SimInfo => ({
  id,
  name: '',
  household: 0,
  gender: female ? 'female' : 'male',
  attractedTo: [],
  appearance: {
    body: female ? '#C9725F' : '#5F86C9',
    skin: ['#C08A64', '#8D5A3B', '#E8C0A0', '#6B4430'][id % 4],
    hair: ['#2B2320', '#7A4A2A', '#C9A15A', '#1A1A1A'][id % 4],
    hairStyle: female ? 'long' : 'short',
    height,
    build,
    ...(stage ? { stage } : {}),
    ...(bottom ? { bottom } : {}),
  },
  traits: [],
  perks: [],
});

await Promise.all(
  stations.map(async (s, n) => {
    const cx = ((n % cols) - (cols - 1) / 2) * gap;
    const cz = -(Math.floor(n / cols) - (rows - 1) / 2) * gap;
    centres.push(new Vector3(cx, 0, cz));
    // Its label on the floor just in front of it: of its object, else of where its Sims stand.
    let labelAt = new Vector3(cx, 0, cz + 1);
    if (s.strip !== undefined) {
      actors.push({ station: n, x: cx, z: cz, yaw: 0, object: -1, partner: -1, role: 0 });
    } else if (s.social) {
      actors.push({ station: n, x: cx, z: cz - 0.5, yaw: 0, object: -1, partner: 0, role: 1 });
      actors.push({ station: n, x: cx, z: cz + 0.5, yaw: Math.PI, object: -1, partner: 0, role: 2 });
    } else if (s.def) {
      const def = content.object(s.def);
      const [w, d] = def?.footprint ?? [1, 1];
      const ox = Math.round(cx - w / 2);
      const oz = Math.round(cz - d / 2);
      const id = objects.length;
      objects.push({ id, def: s.def, x: ox, z: oz, rot: 0, w, d, quality: 0, sellValue: null, ...(s.open ? { open: true } : {}) });
      const key = def?.model ?? `model.${s.def}`;
      const entry = assets.get(key, 'model');
      const template = await buildModel(scene, key, entry, [w, d]);
      // Open (a wall bed folded down) and the use's own clips held open, as the game holds them.
      const motions = motionsOf(entry);
      // (Each clip once: a sofa bed's `open` is its pull-out use's clip too.)
      const held = new Set([...(s.open && motions?.open ? [motions.open] : []), ...(s.clips ?? []).flatMap((c) => (motions?.uses[c] ?? []).map((u) => u.clip))]);
      const playing = [...held].flatMap((name) => {
        const clip = template.motion?.clips.get(name);
        return clip ? [{ clip, time: clip.duration, kind: 'use' }] : [];
      });
      const place = Matrix.Translation(ox + w / 2, 0, oz + d / 2);
      for (const [i, mesh] of template.meshes.entries()) {
        const part = template.motion ? template.motion.parts[i] : -1;
        const m = part >= 0 && playing.length && partPose(playing, part, posed) ? Matrix.FromArray(posed).multiply(place) : place;
        mesh.thinInstanceSetBuffer('matrix', m.asArray() as unknown as Float32Array, 16, true);
        mesh.receiveShadows = true;
        shadowGen.addShadowCaster(mesh);
      }
      // Standing use: centred on the front tile, facing the object (or at its slot's own place); sitting /
      // lying (or standing inside, a shower): on the slot, facing out or as its place says (as sim-core
      // places them: `slotPlace`).
      const slots = def?.slots ?? 1;
      const slot = (k: number) => slotPlace(objects[id], def, k);
      const inside = def?.interactions?.some((it) => it.inside) ?? false;
      const front = (k: number) => {
        if (!def?.places) return { frontX: ox + Math.floor((w - 1) / 2) + 0.5, frontZ: oz + d + 0.5 };
        const p = slotPlace(objects[id], def, k);
        return { frontX: p.x, frontZ: p.z, frontYaw: p.yaw };
      };
      if (s.seated) {
        for (let k = 0; k < slots; k++) actors.push({ station: n, ...slot(k), object: id, partner: -1, role: 0, ...front(k), inside, female: k % 2 === 1 });
      } else if (!s.alone) {
        actors.push({ station: n, ...slot(0), object: id, partner: -1, role: 0, ...front(0), inside });
      }
      if (s.baby) actors.push({ station: n, ...slot(0), object: id, partner: -1, role: 0, baby: true });
      if (s.pair) actors.push({ station: n, ...slot(1), object: id, partner: -1, role: 0, female: true });
      labelAt = new Vector3(ox + w / 2, 0, oz + d + 0.1);
    } else {
      actors.push({ station: n, x: cx, z: cz, yaw: 0, object: -1, partner: -1, role: 0 });
    }
    if (params.get('labels') !== '0') {
      const el = document.createElement('span');
      el.textContent = `${n}: ${s.label}`;
      labels.append(el);
      spots.push({ at: labelAt, el });
    }
  }),
);
actors.sort((a, b) => a.station - b.station);
const hairPalette = palette(assets, 'palette.hair');
// Stations load in any order: row i is actor i after sorting (a station's own actors keep their order).
const actorSims = actors.map((a, i) => {
  const s = stations[a.station];
  const sim = a.baby ? look(i, i % 2 === 0, 'baby') : look(i, s.social ? a.role === 2 : !!s.female || !!a.female, s.stage, s.height, s.build);
  // `?stage` and the strip's rows take the game's look for the stage (the stations' own stages keep theirs).
  const st = stage && allows(s, a, stage) ? stage : s.strip !== undefined ? s.stage : undefined;
  if (st) sim.appearance = lookAtStage(sim.appearance, content.lifeStages.find((x) => x.id === st), hairPalette);
  return sim;
});
for (let i = 0; i < actors.length; i++) {
  const a = actors[i];
  if (a.partner >= 0) a.partner = actors[i].role === 1 ? i + 1 : i - 1;
}

const characters = new Characters(scene, assets, lib, content);
if (!(await characters.init())) throw new Error('characters unavailable');
await characters.build({ sims: actorSims, objects });
for (const m of characters.casters()) shadowGen.addShadowCaster(m);
if (strip) {
  // Each frame of the strip holds the clip at its share of the clip's length.
  const clip = forced ?? 'idle';
  actors.forEach((a, i) => (characters.directions[i] = { clip, still: true, time: (stations[a.station].strip ?? 0) * characters.clipDuration(clip) }));
} else if (forced) {
  characters.force = forced;
}

const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'held'] as const;
const items = content.items.map((it) => it.id);
const HEADER_LEN = 6;
const STRIDE = FIELDS.length;
const layout: SnapshotLayout = {
  capacity: HEADER_LEN + STRIDE * MAX_CHARACTERS,
  headerLen: HEADER_LEN,
  simStride: STRIDE,
  maxSims: MAX_CHARACTERS,
  ticksPerSecond: 20,
  header: { tick: 0, day: 1, minute: 2, speed: 3, simCount: 4, structureVersion: 5 },
  sim: Object.fromEntries(FIELDS.map((f, i) => [f, i])) as unknown as SnapshotLayout['sim'],
  actions,
  items,
};
const snap = new Float32Array(layout.capacity);
const frame: FrameState = { prev: snap, curr: snap, alpha: 0, steps: 1, layout, now: 0 };
const shadowMatrices = new Float32Array(MAX_CHARACTERS * 16);

let clock = 0;
function writeSnapshot(): void {
  const h = layout.header;
  const k = layout.sim;
  snap[h.speed] = 1;
  snap[h.simCount] = actors.length;
  actors.forEach((a, i) => {
    const s = stations[a.station];
    const o = HEADER_LEN + i * STRIDE;
    let pose = s.pose ?? P.Stand;
    let tag = s.tag ?? '';
    if (s.script) {
      const total = s.script.reduce((t, x) => t + x.seconds, 0);
      let t = clock % total;
      const step = s.script.find((x) => (t -= x.seconds) < 0)!;
      pose = step.pose;
      tag = step.tag ?? '';
    }
    if (a.baby) {
      // Lying in the crib: asleep (the crib's `sleep`), or awake (sim-core shows no action for a baby awake in it).
      pose = P.Lie;
      tag = s.baby === 'asleep' ? 'sleep' : '';
    }
    const standing = pose === P.Stand && a.object >= 0 && !a.inside;
    snap[o + k.id] = i;
    snap[o + k.x] = standing ? a.frontX! : a.x;
    snap[o + k.z] = standing ? a.frontZ! : a.z;
    // Scripted stations stand up facing away, as sim-core leaves them (it keeps the yaw), unless the step uses the object.
    snap[o + k.yaw] = standing && (!s.script || tag) ? (a.frontYaw ?? Math.PI) : a.yaw;
    snap[o + k.pose] = pose;
    snap[o + k.moving] = 0;
    if (s.walk) {
      // To and fro along the cell, turning at the ends.
      const t = (clock * 0.45) % 2;
      const back = t > 1;
      snap[o + k.x] = a.x;
      snap[o + k.z] = a.z - 0.9 + 1.8 * (back ? 2 - t : t);
      snap[o + k.yaw] = back ? Math.PI : 0;
      snap[o + k.moving] = 1;
    }
    snap[o + k.held!] = s.held && !a.baby ? items.indexOf(s.held) + 1 : 0;
    const social = s.social ? SOCIAL[s.social] : 0;
    snap[o + k.social] = social ? 1 : 0;
    snap[o + k.anim] = social;
    snap[o + k.outcome] = 0;
    snap[o + k.partner] = a.partner;
    snap[o + k.role] = a.role;
    snap[o + k.emotion] = 0;
    snap[o + k.away] = 0;
    snap[o + k.object] = s.script && pose === P.Stand && !tag ? -1 : a.object;
    snap[o + k.action] = social ? actions.indexOf('talk') : tag ? actions.indexOf(tag) : -1;
    snap[o + k.mood!] = 0.8;
  });
}

/** A station by its index, label or object id (the first with it); -1 if there's none. */
function stationOf(at: number | string): number {
  const n = typeof at === 'number' || /^\d+$/.test(at) ? Number(at) : stations.findIndex((s) => s.label === at || s.def === at);
  return n >= 0 && n < stations.length ? n : -1;
}
const focusParam = params.get('focus');
const focus = focusParam === null ? -1 : stationOf(focusParam);
if (focusParam !== null && focus < 0) problems.push(`focus=${focusParam}: no such station (an index 0–${stations.length - 1}, a label or an object id)`);
const target = focus >= 0 ? centres[focus].add(new Vector3(0.3, 0.7, 0.75)) : Vector3.Zero();
const camera = new ArcRotateCamera(
  'cam',
  Number(params.get('alpha') ?? Math.PI / 2 + 0.5),
  Number(params.get('beta') ?? 1.15),
  Number(params.get('radius')) || (focus >= 0 ? 4 : Math.max(cols, rows) * gap * 1.05),
  target,
  scene,
);
camera.attachControl(canvas, true);
camera.minZ = 0.05;
camera.wheelPrecision = 30;

scene.onAfterRenderObservable.add(() => {
  const vp = camera.viewport.toGlobal(engine.getRenderWidth(), engine.getRenderHeight());
  const m = scene.getTransformMatrix();
  for (const s of spots) {
    const p = Vector3.Project(s.at, Matrix.IdentityReadOnly, m, vp);
    s.el.style.left = `${(p.x / engine.getRenderWidth()) * canvas.clientWidth}px`;
    s.el.style.top = `${(p.y / engine.getRenderHeight()) * canvas.clientHeight}px`;
  }
});

if (problems.length) {
  const el = document.createElement('span');
  el.textContent = problems.join('; ');
  Object.assign(el.style, { left: '50%', top: '8px', background: '#a00d', fontSize: '14px' });
  labels.append(el);
  console.warn(`poses: ${el.textContent}`);
}

const manual = params.has('manual');
function step(dt: number): void {
  clock += dt;
  writeSnapshot();
  frame.now = clock * 1000;
  characters.update(frame, () => true, shadowMatrices);
}
let last = performance.now();
engine.runRenderLoop(() => {
  if (!manual) {
    const now = performance.now();
    step(Math.min(0.1, (now - last) / 1000));
    last = now;
  }
  scene.render();
});
addEventListener('resize', () => engine.resize());

Object.assign(window, {
  __poses: {
    stations: stations.map((s) => s.label),
    /** Advances the bench by `seconds` in 1/30 s steps (manual mode). */
    advance(seconds: number) {
      for (let t = 0; t < seconds - 1e-6; t += 1 / 30) step(1 / 30);
    },
    /** Frames a station (index, label or object id) close up; throws if there's no such station. */
    focus(at: number | string, radius = 4, alpha?: number, beta?: number) {
      const n = stationOf(at);
      if (n < 0) throw new Error(`no station ${at} (an index 0–${stations.length - 1}, a label or an object id)`);
      camera.target = centres[n].add(new Vector3(0.3, 0.7, 0.75));
      camera.radius = radius;
      if (alpha !== undefined) camera.alpha = alpha;
      if (beta !== undefined) camera.beta = beta;
    },
    /** What the page couldn't do as asked (also shown on it), or null. */
    error: problems.length ? problems.join('; ') : null,
    characters,
    camera,
  },
  __ready: true,
});
