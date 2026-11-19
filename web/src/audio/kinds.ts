/**
 * The sounds there are, by name, and what the player needs to know of them to play them: which are played once,
 * the birds' habits, the rooms' sizes and reverb levels. The sounds themselves are made in Rust (`crates/sound`:
 * `synth.ts` asks for them by these names; its tests check every name here is one it makes).
 */

/** Two channels of samples. */
export interface Stereo {
  left: Float32Array;
  right: Float32Array;
}

/** The water's loops (`crates/sound/src/water.rs`); the flush is played once. */
export const WATER = ['shower', 'tap', 'washUp', 'bathFill', 'bathSoak', 'washer', 'dryer', 'dishwasher', 'crockery', 'fountain', 'flush'] as const;
export type WaterKind = (typeof WATER)[number];
export const ONCE = ['flush'] as const satisfies readonly WaterKind[];

/** The things' sounds (`things.rs`); those played once as something happens. */
export const THINGS = ['treadmill', 'bike', 'tools', 'bikeTools', 'pool', 'foosball', 'pingPong', 'darts', 'arcade', 'sewing', 'potteryWheel', 'bubbling', 'espresso', 'blender', 'mixer', 'microwave', 'toaster', 'ovenFan', 'hood', 'fire', 'basketball', 'singing', 'cello', 'punchBag', 'smash', 'fridgeDoor', 'binLid', 'vendSnack', 'vendCan', 'phoneBox', 'hangUp', 'ping', 'toastPop', 'pour'] as const;
export type ThingKind = (typeof THINGS)[number];
export const ONCE_THINGS = ['fridgeDoor', 'binLid', 'vendSnack', 'vendCan', 'phoneBox', 'hangUp', 'ping', 'toastPop', 'pour'] as const satisfies readonly ThingKind[];

/** Every sound made off the main thread: the water's and the things'. */
export const MADE = [...WATER, ...THINGS] as const;
export type MadeKind = (typeof MADE)[number];

/** The outdoors' loops (`outdoors.rs`): one channel for what has a place (a car), two for what is all round. */
export const OUTDOORS = ['carEngine', 'carTyres', 'town', 'leaves'] as const;
export type OutdoorKind = (typeof OUTDOORS)[number];

/** The pieces of music the game plays (`music.rs`). */
export const MUSIC = ['musicFolk', 'musicDance', 'musicSlow'] as const;
export type MusicKind = (typeof MUSIC)[number];

/** What the sound worker makes (`water.worker.ts`). */
export type WorkerKind = MadeKind | OutdoorKind | MusicKind;

/** The sounds of one channel (`mono.rs`), and those played once (the rest loop). */
export const MONO = ['sizzle', 'chop', 'tv', 'drums', 'bed', 'crickets', 'knock'] as const;
export type MonoKind = (typeof MONO)[number];
export const ONCE_MONO = ['knock'] as const satisfies readonly MonoKind[];

/** The interface's blips (`blips.rs`). */
export const BLIPS = ['pick', 'place', 'sell', 'rotate', 'error', 'build', 'remove', 'paint', 'upgrade', 'tab', 'open', 'close', 'menu', 'choose', 'speed', 'pause', 'resume', 'moneyIn', 'moneyOut', 'notify', 'notifyLow'] as const;
export type Blip = (typeof BLIPS)[number];

/** Footsteps (`steps.rs`): what is walked on, in what, and the versions of each step. */
export type Surface = 'wood' | 'tile' | 'carpet' | 'grass' | 'paving';
export type Footwear = 'shoe' | 'bare';
export const SURFACES: readonly Surface[] = ['wood', 'tile', 'carpet', 'grass', 'paving'];
export const FOOTWEAR: readonly Footwear[] = ['shoe', 'bare'];
export const STEP_VERSIONS = 4;

/**
 * A species of bird as the player plays it (its song is made in `birdsong.rs`): its range of an individual's own
 * pitch (Hz), its level, the songs in a bout, the seconds between them and of rest after the bout when birds are
 * busiest, and whether it takes turns with the other birds (waits for a quiet moment, sometimes answers one).
 */
export interface Species {
  name: string;
  pitch: [number, number];
  level: number;
  bout: [number, number];
  pause: [number, number];
  rest: [number, number];
  turns: boolean;
}

export const SPECIES: Species[] = [
  { name: 'tit', pitch: [3700, 4300], level: 0.8, bout: [3, 6], pause: [2, 4], rest: [25, 60], turns: true },
  { name: 'finch', pitch: [3300, 3800], level: 0.75, bout: [4, 8], pause: [5, 9], rest: [30, 80], turns: true },
  { name: 'warbler', pitch: [2900, 3300], level: 0.75, bout: [3, 6], pause: [6, 12], rest: [30, 80], turns: true },
  { name: 'blackbird', pitch: [1900, 2300], level: 0.9, bout: [4, 9], pause: [2.5, 5], rest: [30, 70], turns: true },
  { name: 'dove', pitch: [470, 560], level: 1.6, bout: [3, 6], pause: [1.4, 2.2], rest: [90, 200], turns: false },
];

/** What makes one bird's song its own (sent to the worker that makes it). */
export interface Voice {
  /** Its species' `name`. */
  species: string;
  seed: number;
  /** Its own pitch (Hz), pace (1 = as written) and nearness (1 = nearest). */
  pitch: number;
  pace: number;
  near: number;
}

/** Versions of its song each bird has. */
export const VERSIONS = 3;

/** A room's size (`rooms.rs` makes each one's reverb). */
export type RoomSize = 'small' | 'medium' | 'large';

/** A room up to `area` m² (the game's tiles) is this size; its reverb is `send` of the sound made in it (0.2: 14 dB under). */
export const ROOMS: Record<RoomSize, { area: number; send: number }> = {
  small: { area: 9, send: 0.2 },
  medium: { area: 28, send: 0.18 },
  large: { area: Infinity, send: 0.22 },
};

export const ROOM_SIZES = Object.keys(ROOMS) as RoomSize[];

/** The size of a room of `area` m². */
export function roomSize(area: number): RoomSize {
  return ROOM_SIZES.find((s) => area <= ROOMS[s].area) ?? 'large';
}
