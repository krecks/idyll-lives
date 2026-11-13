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
import { MADE, ONCE_THINGS, type MadeKind } from './things';
import { ONCE, type Stereo } from './water';

/** A sound played once (`ONCE`, `ONCE_THINGS`), not looped. */
export type OnceKind = (typeof ONCE)[number] | (typeof ONCE_THINGS)[number];
export const ONCE_KINDS: readonly string[] = [...ONCE, ...ONCE_THINGS];
/** A sound that loops while something is in use (or runs by itself). */
export type LoopKind = 'sizzle' | 'chop' | 'tv' | 'music' | 'drums' | Exclude<MadeKind, OnceKind>;
/** A world sound: a loop, or one played once (as a use starts, when it's a use's sound). */
export type SoundKind = LoopKind | OnceKind;

export interface LoopDef {
  level: number;
  /** How far the speed (and with it the pitch) wanders either way, so a loop doesn't sound the same each time round. */
  drift?: number;
}

/** The loops of things in use, by the key the content names them with (`sound`, `sounds`, `ambient`). */
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
  music: { level: 0.16 },
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

/** Levels of the sounds played once. */
export const ONCE_LEVEL: Record<OnceKind, number> = { flush: 0.3, fridgeDoor: 0.27, binLid: 0.29, vendSnack: 0.25, vendCan: 0.25, phoneBox: 0.23, hangUp: 0.23, ping: 0.21, toastPop: 0.27, pour: 0.21 };

/**
 * The outdoors: the bed of wind (its level at night, and how much the day adds), the crickets at night, the
 * birds (times each species' own level and how near the bird is; `far`: the unseen birds' share while birds in
 * view sing) and a visitor's knock at the door.
 */
export const AMBIENCE = { bed: { night: 0.05, day: 0.03 }, crickets: 0.1, birds: 0.05, far: 0.55, knock: 1 };

/** The interface's sounds are quiet by design: the level of their bus (`mixer.ts`). */
export const EFFECTS_LEVEL = 0.5;

/** What a sound belongs to: a thing in the world, the outdoors, or the interface. */
export type Group = 'things' | 'ambience' | 'interface';

/**
 * Where each group's sounds sit as the game plays them (LUFS, `loudness.ts`; `bun run sounds` lists them):
 * things in use between a pottery wheel's hum (-35) and a shower (-20), the outdoors under most of them (it is
 * always there: -40 to -32), the interface's blips from a tab's tick (-45) to a wall's thump (-28; they are a
 * fraction of a second, measured over their own length). A sound outside its group's range is a level left off, or a sound made much louder or quieter than
 * it was without the mix having been looked at.
 */
export const GUARD: Record<Group, readonly [number, number]> = {
  things: [-40, -18],
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
    const heard = name in MONO ? (rate: number) => centred(MONO[name as MonoKind](rate), level) : (rate: number) => scaled(MADE[name as MadeKind](rate), level);
    list.push({ name, group: 'things', plays: once.has(name) ? 'once' : 'loop', level, heard });
  }
  list.push({ name: 'knock', group: 'things', plays: 'once', level: AMBIENCE.knock, heard: (rate) => centred(MONO.knock(rate), AMBIENCE.knock) });
  const day = AMBIENCE.bed.night + AMBIENCE.bed.day;
  list.push({ name: 'bed', group: 'ambience', plays: 'loop', level: day, note: `by day; ${AMBIENCE.bed.night} at night`, heard: (rate) => centred(MONO.bed(rate), day) });
  list.push({ name: 'crickets', group: 'ambience', plays: 'loop', level: AMBIENCE.crickets, note: 'at night', heard: (rate) => centred(MONO.crickets(rate), AMBIENCE.crickets) });
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
