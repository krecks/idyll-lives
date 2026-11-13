/**
 * Every sound the game makes and the level it plays each at: what `world.ts` and `ui/sfx.ts` play by,
 * what `bun run sounds` (`tools/audio/listen.ts`) renders and what the tests hold to the mix
 * (`web/tests/sounds.test.ts`). No DOM or audio context here: a sound is samples made at a sample rate.
 *
 * The level is a plain gain on the sound as made (the water's and the things' are made to an average
 * level each, `water.ts` `setLevel`; the others to a peak of 1). A sound of one channel goes through a
 * stereo panner, which in the middle of the picture gives each side 0.71 of it: `heard` is what comes
 * out there, so one sound against another reads here as it does in the game.
 */

import { songSamples, SPECIES } from './birdsong';
import { BLIPS, blipSamples, type Blip } from './blips';
import { MONO, ONCE_MONO, type MonoKind } from './mono';
import { MUSIC, type MusicKind } from './music';
import { OUTDOORS } from './outdoors';
import { FOOTWEAR, stepSamples, SURFACES, type Footwear, type Surface } from './steps';
import { MADE, ONCE_THINGS, type MadeKind } from './things';
import { ONCE, type Stereo } from './water';

/** A sound played once (`ONCE`, `ONCE_THINGS`), not looped. */
export type OnceKind = (typeof ONCE)[number] | (typeof ONCE_THINGS)[number];
export const ONCE_KINDS: readonly string[] = [...ONCE, ...ONCE_THINGS];
/** A sound that loops while something is in use (or runs by itself). */
export type LoopKind = 'sizzle' | 'chop' | 'tv' | 'music' | 'drums' | MusicKind | Exclude<MadeKind, OnceKind>;
/** A world sound: a loop, or one played once (as a use starts, when it's a use's sound). */
export type SoundKind = LoopKind | OnceKind;

export interface LoopDef {
  level: number;
  /** How far the speed (and with it the pitch) wanders either way, so a loop doesn't sound the same each time round. */
  drift?: number;
}

/** The music's level, whichever piece it is. */
const MUSIC_LEVEL = 0.235;

/**
 * The loops of things in use, by the key the content names them with (`sound`, `sounds`, `ambient`). `music` is
 * whichever piece suits the hour and the dancing (`pieceAt`); a thing can name a piece of its own (`musicSlow`
 * for a gramophone's waltz).
 */
export const LOOPS: Record<LoopKind, LoopDef> = {
  sizzle: { level: 0.22 },
  chop: { level: 0.3 },
  shower: { level: 0.28, drift: 0.04 },
  tap: { level: 0.22, drift: 0.04 },
  washUp: { level: 0.24, drift: 0.03 },
  bathFill: { level: 0.26, drift: 0.04 },
  bathSoak: { level: 0.35, drift: 0.03 },
  washer: { level: 0.2, drift: 0.02 },
  dryer: { level: 0.18, drift: 0.02 },
  dishwasher: { level: 0.16, drift: 0.02 },
  crockery: { level: 0.35 },
  fountain: { level: 0.2, drift: 0.03 },
  tv: { level: 0.35 },
  music: { level: MUSIC_LEVEL },
  musicFolk: { level: MUSIC_LEVEL },
  musicDance: { level: MUSIC_LEVEL },
  musicSlow: { level: MUSIC_LEVEL },
  drums: { level: 0.2 },
  treadmill: { level: 0.21, drift: 0.02 },
  bike: { level: 0.2, drift: 0.04 },
  tools: { level: 0.27 },
  bikeTools: { level: 0.27 },
  pool: { level: 0.27 },
  foosball: { level: 0.27 },
  pingPong: { level: 0.27 },
  darts: { level: 0.27 },
  arcade: { level: 0.25 },
  sewing: { level: 0.33 },
  potteryWheel: { level: 0.15, drift: 0.02 },
  bubbling: { level: 0.25, drift: 0.04 },
  espresso: { level: 0.195 },
  blender: { level: 0.28, drift: 0.03 },
  mixer: { level: 0.22, drift: 0.02 },
  microwave: { level: 0.24 },
  toaster: { level: 0.37 },
  ovenFan: { level: 0.275 },
  hood: { level: 0.23, drift: 0.02 },
  fire: { level: 0.21, drift: 0.04 },
  basketball: { level: 0.27 },
  singing: { level: 0.26 },
  cello: { level: 0.29 },
  punchBag: { level: 0.33 },
  smash: { level: 0.33 },
};

/**
 * Which piece the music is (`music.ts`): a dance while someone dances; else a slow one from `evening` o'clock
 * to `morning`, and a folk tune by day.
 */
export const PLAYLIST = { evening: 20, morning: 6 };

/** The piece that plays at `hour` (0..24), `dancing` or not. */
export function pieceAt(hour: number, dancing: boolean): MusicKind {
  if (dancing) return 'musicDance';
  return hour >= PLAYLIST.evening || hour < PLAYLIST.morning ? 'musicSlow' : 'musicFolk';
}

/** Levels of the sounds played once. */
export const ONCE_LEVEL: Record<OnceKind, number> = { flush: 0.3, fridgeDoor: 0.27, binLid: 0.29, vendSnack: 0.25, vendCan: 0.25, phoneBox: 0.23, hangUp: 0.23, ping: 0.21, toastPop: 0.27, pour: 0.21 };

/**
 * The outdoors: the bed of wind (its level at night, and how much the day adds), the crickets at night, the
 * birds (times each species' own level and how near the bird is; `far`: the unseen birds' share while birds in
 * view sing) and a visitor's knock at the door.
 */
export const AMBIENCE = { bed: { night: 0.05, day: 0.03 }, crickets: 0.1, birds: 0.05, far: 0.55, knock: 1 };

/**
 * Footsteps (`footsteps.ts`): the level of a step by what is walked on and in what (shoes on tiles the
 * loudest, bare feet on carpet next to nothing), what a jog and a run add (`gear`, by gait), how the level of
 * each falls with the walkers in view (`crowd` over the square root of how many), how many can sound at once
 * (`voices`) and start on one look (`perLook`), how far ahead a step is started (`ahead` s) and how close two of
 * one walker's may fall (`apart` s), and the least of the camera's distance gain at which steps are still made
 * (`farthest`: to about 68 m).
 */
export const STEPS = {
  level: {
    wood: { shoe: 0.09, bare: 0.057 },
    tile: { shoe: 0.16, bare: 0.077 },
    carpet: { shoe: 0.045, bare: 0.03 },
    grass: { shoe: 0.038, bare: 0.031 },
    paving: { shoe: 0.094, bare: 0.061 },
  } satisfies Record<Surface, Record<Footwear, number>>,
  gear: [1, 1.25, 1.5],
  crowd: 1.5,
  voices: 8,
  perLook: 4,
  ahead: 0.18,
  apart: 0.15,
  farthest: 0.35,
};

/**
 * The street and the weather (`outside.ts`, the loops of `outdoors.ts`).
 * - `car`: a car driving past: its engine's and its tyres' level right by the middle of the picture at `full` m/s
 *   on screen or more (a street's 32 km/h on the traffic's clock, four times as fast as real time: 35 m/s; stood at a light the engine idles at `idle` of that, the tyres are silent); how near it
 *   counts as (`reach` of the camera's distance, `least` m at least: half the level that far from where the
 *   camera looks, nothing under `drop` of it); and how many cars sound at once (`voices`: the nearest).
 * - `town`: the town from above, at the full town view by day (`night` of it at night); nothing at the play
 *   camera's distances.
 * - `leaves`: the wind in the leaves and the grass at a full gust (`calm` of it between gusts), `indoors` of it
 *   with the camera in on a room, `far` of it from the full town view.
 */
export const OUTSIDE = {
  car: { engine: 0.2, tyres: 0.12, idle: 0.35, full: 30, reach: 0.9, least: 14, drop: 0.06, voices: 3 },
  town: { level: 0.13, night: 0.6 },
  leaves: { level: 0.045, calm: 0.15, indoors: 0.3, far: 0.5 },
};

/** The interface's sounds are quiet by design: the level of their bus (`mixer.ts`). */
export const EFFECTS_LEVEL = 0.5;

/** What a sound belongs to: a thing in the world, a resident's feet, the outdoors, or the interface. */
export type Group = 'things' | 'steps' | 'ambience' | 'interface';

/**
 * Where each group's sounds sit as the game plays them (LUFS, `loudness.ts`; `bun run sounds` lists them):
 * things in use between a pottery wheel's hum (-35) and a shower (-20), one footstep under the quietest of
 * them (shoes on tiles -38, bare feet on carpet -48: they are many), the outdoors under most of them (it is
 * always there: -40 to -32), the interface's blips from a tab's tick (-45) to a wall's thump (-28; they are a
 * fraction of a second, measured over their own length). A sound outside its group's range is a level left off, or a sound made much louder or quieter than
 * it was without the mix having been looked at.
 */
export const GUARD: Record<Group, readonly [number, number]> = {
  things: [-40, -18],
  steps: [-52, -34],
  ambience: [-46, -28],
  interface: [-50, -24],
};

/** A sound as `bun run sounds` renders it and the tests measure it. */
export interface Listed {
  name: string;
  group: Group;
  plays: 'loop' | 'once';
  /** The level the game plays it at (a gain on the sound as made); `note` says where that varies. */
  level: number;
  note?: string;
  /** Who sounds it, where no content names it and its group doesn't say. */
  who?: string;
  /** What comes out in the middle of the picture at that level. */
  heard: (rate: number) => Stereo;
}

/** A sound of one channel through a stereo panner in the middle: 0.71 of it each side. */
function centred(samples: Float32Array, level: number): Stereo {
  const left = samples.map((v) => v * level * Math.SQRT1_2);
  return { left, right: left.slice() };
}

function scaled(s: Stereo, level: number): Stereo {
  return { left: s.left.map((v) => v * level), right: s.right.map((v) => v * level) };
}

/** A typical unseen bird of a species (`world.ts` `FLOCK`): the middle of its pitch range, a garden away. */
const FLOCK_NEAR = 0.575;

/** Every sound the game makes, in the order `bun run sounds` lists them. */
export function everySound(): Listed[] {
  const list: Listed[] = [];
  const once = new Set<string>([...ONCE_KINDS, ...ONCE_MONO]);
  const things = [...new Set<string>([...Object.keys(LOOPS), ...Object.keys(ONCE_LEVEL)])];
  for (const name of things) {
    const level = name in LOOPS ? LOOPS[name as LoopKind].level : ONCE_LEVEL[name as OnceKind];
    // (`music` is one of the three pieces, listed by their own names.)
    if (name === 'music') continue;
    if (name in MUSIC) {
      const when: Record<MusicKind, string> = { musicFolk: 'by day', musicDance: 'while someone dances', musicSlow: `from ${PLAYLIST.evening} to ${PLAYLIST.morning} o'clock` };
      list.push({ name, group: 'things', plays: 'loop', level, note: `the music ${when[name as MusicKind]} (\`music\`: a stereo, a radio, an instrument played), and where a thing names it`, heard: (rate) => scaled(MUSIC[name as MusicKind](rate), level) });
      continue;
    }
    const heard = name in MONO ? (rate: number) => centred(MONO[name as MonoKind](rate), level) : (rate: number) => scaled(MADE[name as MadeKind](rate), level);
    list.push({ name, group: 'things', plays: once.has(name) ? 'once' : 'loop', level, heard });
  }
  list.push({ name: 'knock', group: 'things', plays: 'once', level: AMBIENCE.knock, who: 'a visitor at the door', heard: (rate) => centred(MONO.knock(rate), AMBIENCE.knock) });
  for (const surface of SURFACES) {
    for (const wear of FOOTWEAR) {
      const level = STEPS.level[surface][wear];
      list.push({ name: `step-${surface}-${wear}`, group: 'steps', plays: 'once', level, note: 'one step at a walk, one walker', heard: (rate) => centred(stepSamples(surface, wear, 0, rate), level) });
    }
  }
  const mix = (a: Float32Array, b: Float32Array) => a.map((v, i) => v + b[i]);
  list.push({
    name: 'car',
    group: 'things',
    plays: 'loop',
    level: OUTSIDE.car.engine,
    who: 'a car or the work taxi driving past',
    note: `a car passing right by at speed: its engine, and its tyres at ${OUTSIDE.car.tyres}; the game plays both higher the faster it goes`,
    heard: (rate) => centred(mix(OUTDOORS.carEngine(rate).map((v) => v * OUTSIDE.car.engine), OUTDOORS.carTyres(rate).map((v) => v * OUTSIDE.car.tyres)), 1),
  });
  const day = AMBIENCE.bed.night + AMBIENCE.bed.day;
  list.push({ name: 'bed', group: 'ambience', plays: 'loop', level: day, note: `by day; ${AMBIENCE.bed.night} at night`, heard: (rate) => centred(MONO.bed(rate), day) });
  list.push({ name: 'crickets', group: 'ambience', plays: 'loop', level: AMBIENCE.crickets, note: 'at night', heard: (rate) => centred(MONO.crickets(rate), AMBIENCE.crickets) });
  list.push({ name: 'town', group: 'ambience', plays: 'loop', level: OUTSIDE.town.level, note: `from the full town view by day; ${OUTSIDE.town.night} of it at night, nothing from the play camera`, heard: (rate) => scaled(OUTDOORS.town(rate), OUTSIDE.town.level) });
  list.push({ name: 'leaves', group: 'ambience', plays: 'loop', level: OUTSIDE.leaves.level, note: `at a full gust; ${OUTSIDE.leaves.calm} of it between gusts`, heard: (rate) => scaled(OUTDOORS.leaves(rate), OUTSIDE.leaves.level) });
  SPECIES.forEach((sp, i) => {
    const level = AMBIENCE.birds * sp.level * FLOCK_NEAR;
    const voice = { species: sp.name, seed: i, pitch: (sp.pitch[0] + sp.pitch[1]) / 2, pace: 1, near: FLOCK_NEAR };
    list.push({ name: `bird-${sp.name}`, group: 'ambience', plays: 'once', level, note: 'an unseen bird by day; one in view up to twice that', heard: (rate) => centred(songSamples(voice, 0, rate), level) });
  });
  for (const name of Object.keys(BLIPS) as Blip[]) {
    // (No panner: both sides get all of it.)
    list.push({ name: `ui-${name}`, group: 'interface', plays: 'once', level: EFFECTS_LEVEL, heard: (rate) => scaled({ left: blipSamples(name, rate), right: blipSamples(name, rate) }, EFFECTS_LEVEL) });
  }
  return list;
}
