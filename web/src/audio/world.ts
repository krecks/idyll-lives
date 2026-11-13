/**
 * Sounds of the world, synthesised like the interface sounds (no sound files):
 * - ambience by the game clock: a soft outdoor bed, crickets at night, and by day a few neighbourhood
 *   birds of different species, each with its own song, singing in bouts (most in the dawn chorus);
 * - things in use on the lot being looked at: a sizzling pan, a shower, a tap, washing up, a bath, the
 *   washing machine and the dryer, the dishwasher being loaded, a TV, music, drums, and the machines,
 *   games and tools of `things.ts` (a treadmill, a blender, darts, a cello...). One loop per kind of
 *   sound, louder the more residents make it, placed left or right like voices; the water sounds are
 *   made in `water.ts`, the things' in `things.ts`, those of one channel (a pan, a knife, the TV,
 *   music, drums, the outdoor bed, crickets, the knock) in `mono.ts`; the level of each is in
 *   `catalog.ts`. What a use sounds like comes from the thing
 *   (content `sound` and `sounds`), else from its animation (`ACTION_SOUNDS`);
 * - things that sound by themselves (content `ambient`: a fountain; a machine running on its own, its
 *   `runs.sound`: the washer loaded, nobody at it), while their lot is looked at, some only while a use
 *   under them sounds (`ambientWhile`: a cooker hood over a hob in use);
 * - sounds as something happens: a fridge door as its use starts (a `sound` played once), a toilet's
 *   flush as its use ends (content `endSound`), a visitor's knock at the front door.
 *
 * A thing's sound is placed (`space.ts`): left or right by where it is on screen, in the reverb of the
 * room it is made in (dry outdoors), behind a wall when the camera is in on another room, and further
 * off as the camera zooms out (`acoustics.ts`). The outdoors (the bed, crickets, birds) is all round.
 *
 * Presentation only. Reads the snapshot about ten times a second; residents the renderer doesn't
 * draw (other lots, at work) make no sound.
 */

import type { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import { storeyOfRow, storeyRow, type StoreyLayout, type WorldStructure } from '../core/protocol';
import { townView } from '../render/shared/townView';
import { birdActivity, STOREY_HEIGHT, type Renderer, type ViewRect } from '../render/types';
import { game } from '../ui/state.svelte';
import { acoustics } from './acoustics';
import { songSamples, SPECIES, VERSIONS, type Species, type Voice } from './birdsong';
import { AMBIENCE, LOOPS, ONCE_KINDS, ONCE_LEVEL, type LoopKind, type OnceKind, type SoundKind } from './catalog';
import { audioContext, bus, space, updateVolumes } from './mixer';
import { MONO, type MonoKind } from './mono';
import type { RoomSize } from './rooms';
import { Placed, type Place, type Space } from './space';
import { MADE, type MadeKind } from './things';
import { random, WATER, type Stereo } from './water';
import type { SoundReply, SoundRequest } from './water.worker';

const TICK_MS = 100;
/** Updates come with drawn frames: this long without one (the picture stopped), everything fades out. */
const STALE_MS = 2000;
/** Seconds for a loop to fade in or out. */
const FADE = 0.4;
/** Seconds a bath runs before the water's turned off and the bather settles in. */
const BATH_FILL = 7;

/** What each action sounds like (content animation tags; unknown tags are silent; see also `soundOf`). */
const ACTION_SOUNDS: Record<string, LoopKind> = {
  cook: 'sizzle',
  // The cooking steps (content `recipes`): a knife on a board, a pan, the grill.
  chop: 'chop',
  stir: 'sizzle',
  flip: 'sizzle',
  shower: 'shower',
  bath: 'bathFill',
  wash: 'tap',
  watch: 'tv',
  music: 'music',
  listen: 'music',
  dance: 'music',
  // A cooking step in an oven: its fan (a pizza oven's fire, a microwave's hum: by the object's `sounds`).
  bake: 'ovenFan',
};

/** Content `sound` that silences a use whose animation would otherwise make one (toast isn't fried). */
const SILENT = 'none';

/**
 * What a resident's action sounds like, by what they use: the use's own `sound` (not a recipe's,
 * whose steps sound by where they're done), else the cooking step's (`recipe`: the snapshot's,
 * index + 1; a bake), else the object's for that animation (`sounds`: an oven's `stir` is silent),
 * else by the animation (`ACTION_SOUNDS`), with washing by what's washed. A sound played once (a
 * fridge door) is played as the action starts. `def` and `interaction` are the snapshot's (-1: none).
 */
export function soundOf(content: Content, tag: string, def: number, interaction: number, recipe = 0): SoundKind | undefined {
  const object = def >= 0 ? content.objectList[def] : undefined;
  const use = interaction >= 0 ? object?.interactions[interaction] : undefined;
  const step = recipe > 0 ? content.recipes[recipe - 1]?.steps.find((s) => s.anim === tag && s.sound !== undefined) : undefined;
  const own = (use?.recipe ? undefined : use?.sound) ?? step?.sound ?? object?.sounds?.[tag];
  if (own !== undefined) return soundKind(own);
  if (tag === 'wash') {
    // Clearing away after a meal washes up at the sink without an interaction of its own; at a
    // dishwasher (no washing interaction) it's loaded without a sound.
    if (use?.id === 'washUp' || !object) return 'washUp';
    if (!use) return object.interactions.some((i) => i.anim === 'wash') ? 'washUp' : undefined;
    // Getting ready at a dressing table washes without a tap.
    return use.tags?.includes('handwash') ? 'tap' : undefined;
  }
  return ACTION_SOUNDS[tag];
}

/** The sound played once as the use ends (content `endSound`: a toilet's flush), if any. */
export function endSoundOf(content: Content, def: number, interaction: number): OnceKind | undefined {
  const key = def >= 0 && interaction >= 0 ? content.objectList[def]?.interactions[interaction]?.endSound : undefined;
  return key === undefined ? undefined : isOnce(key) ? key : unknownSound(key);
}

/** A content sound key as a world sound (`none`: silence); unknown keys are silent (and said once). */
function soundKind(key: string): SoundKind | undefined {
  if (key === SILENT) return undefined;
  return key in LOOPS || isOnce(key) ? (key as SoundKind) : unknownSound(key);
}

function isOnce(key: string): key is OnceKind {
  return ONCE_KINDS.includes(key);
}

const unknown = new Set<string>();
function unknownSound(key: string): undefined {
  if (!unknown.has(key)) console.warn(`content: no world sound "${key}"`);
  unknown.add(key);
  return undefined;
}

/** What sounded on an update (`WorldSound.playing`): residents and things per loop and where each loop is heard from, the sounds played once, and the camera's distance (m). */
interface Heard {
  loops: Partial<Record<LoopKind, number>>;
  places: Partial<Record<LoopKind, Place>>;
  once: { kind: OnceKind; at: number }[];
  radius: number;
}

/** Something that sounds by itself (content `ambient`), where it stands (`row`: its z in the world's rows, its storey's). */
interface Ambient {
  kind: LoopKind;
  x: number;
  y: number;
  z: number;
  row: number;
  /** Only while a use under it (on these tiles, rows as the world's) sounds like this (content `ambientWhile`); and whether one does now. */
  while?: { kind: SoundKind; tiles: Tiles; on: boolean };
}

interface Loop {
  source: AudioBufferSourceNode;
  gain: GainNode;
  panner: StereoPannerNode;
}

/** A thing's loop, placed in the world's space. */
interface PlacedLoop {
  source: AudioBufferSourceNode;
  gain: GainNode;
  placed: Placed;
}

/** Who makes a loop this tick: how many, and where they are on average (left to right, behind a wall, in rooms of which sizes). */
interface Users {
  n: number;
  pan: number;
  muffle: number;
  rooms: Record<RoomSize, number>;
}

/** One more user of a loop at `pan`, at tile point (`x`, `z`: the world's rows). */
function join(users: Map<LoopKind, Users>, kind: LoopKind, pan: number, x: number, z: number): void {
  let u = users.get(kind);
  if (!u) {
    u = { n: 0, pan: 0, muffle: 0, rooms: { small: 0, medium: 0, large: 0 } };
    users.set(kind, u);
  }
  u.pan = (u.pan * u.n + pan) / (u.n + 1);
  u.muffle = (u.muffle * u.n + acoustics.muffleAt(x, z)) / (u.n + 1);
  u.n++;
  const size = acoustics.sizeAt(x, z);
  if (size) u.rooms[size]++;
}

/** Where a sound made at tile point (`x`, `z`: the world's rows) is, on screen at `pan`. */
function placeAt(pan: number, x: number, z: number): Place {
  const size = acoustics.sizeAt(x, z);
  return { pan, muffle: acoustics.muffleAt(x, z), rooms: size ? { [size]: 1 } : undefined };
}

export class WorldSound {
  private lastTick = 0;
  private readonly loops = new Map<LoopKind, PlacedLoop>();
  /** The world's storeys (to find the rows of the one in view). */
  private storeys: StoreyLayout | null = null;
  private bed: Loop | null = null;
  private crickets: Loop | null = null;
  private readonly made = new WorkerSounds();
  private readonly birds = new Birds(this.made);
  /** Residents knocking last tick (by id), and when each knocks again (ms). */
  private readonly knocking = new Map<number, number>();
  /** Residents in the bath (by id), and when they got in (ms). */
  private readonly bathers = new Map<number, number>();
  /** Residents in a use with an `endSound` (by id): the use (`def:interaction`) and the sound. */
  private readonly ending = new Map<number, { use: number; kind: OnceKind }>();
  /** What each resident was doing (by id: the object, use and animation), to play a sound once as something starts. */
  private readonly doing = new Map<number, string>();
  /** Things sounding by themselves on the lot in view. */
  private ambient: Ambient[] = [];
  /** The sounds played once by the things on the lot in view: made before they're wanted. */
  private onceHere = new Set<OnceKind>();
  /** The definitions (index into `content.objectList`) of the world's objects, by their stable id. */
  private readonly defOf = new Map<number, number>();
  /** The world's objects' tiles, by their stable id. */
  private readonly tilesOf = new Map<number, Tiles>();
  /** What sounded on the last update (`playing`). */
  private heard: Heard = { loops: {}, places: {}, once: [], radius: 0 };
  private readonly head = { x: 0, y: 0, z: 0 };
  private readonly screen = { x: 0, y: 0 };
  private disposed = false;
  /** Faded out for want of updates (until the next one). */
  private hushed = false;
  private readonly watchdog = setInterval(() => this.hush(), STALE_MS / 2);

  constructor(
    private readonly renderer: Renderer,
    private readonly content: Content,
  ) {}

  /** The world's objects, and the lot in view (`view`; null: none): the ones there that sound by themselves. */
  setWorld(world: WorldStructure, view: ViewRect | null): void {
    this.ambient = [];
    this.onceHere = new Set();
    this.defOf.clear();
    this.tilesOf.clear();
    this.storeys = world;
    acoustics.setWorld(world);
    const index = new Map(this.content.objectList.map((o, i) => [o.id, i]));
    for (const o of world.objects) {
      this.defOf.set(o.id, index.get(o.def) ?? -1);
      this.tilesOf.set(o.id, { x: o.x, z: o.z, w: o.w, d: o.d });
    }
    if (!view) return;
    for (const o of world.objects) {
      const def = this.content.object(o.def);
      const storey = storeyOfRow(world, o.z);
      const x = o.x + o.w / 2;
      const z = o.z - storeyRow(world, storey) + o.d / 2;
      if (!def || x < view.x || x > view.x + view.w || z < view.z || z > view.z + view.d) continue;
      for (const key of [...Object.values(def.sounds ?? {}), ...def.interactions.flatMap((i) => [i.sound, i.endSound])]) if (key !== undefined && isOnce(key)) this.onceHere.add(key);
      const key = def.ambient ?? (o.running ? def.runs?.sound : undefined);
      const kind = key === undefined ? undefined : soundKind(key);
      // Broken (worn out), it's still.
      if (!kind || isOnce(kind) || (o.wear ?? 0) >= 1) continue;
      const under = def.ambientWhile === undefined ? undefined : soundKind(def.ambientWhile);
      const tiles = { x: o.x, z: o.z, w: o.w, d: o.d };
      this.ambient.push({ kind, x, y: storey * STOREY_HEIGHT + (under ? 1.8 : 0.6), z, row: o.z + o.d / 2, while: under && { kind: under, tiles, on: false } });
    }
  }

  /** What sounded on the last update: residents and things per loop, and the sounds played once (tests). */
  playing(): Heard {
    return this.heard;
  }

  update(frame: FrameState): void {
    const now = frame.now;
    if (now - this.lastTick < TICK_MS || this.disposed) return;
    this.lastTick = now;
    this.hushed = false;
    updateVolumes();
    // Where the camera looks from (the voices are placed by it too, with the world's sounds off).
    acoustics.look(townView.target.x, townView.target.z + (this.storeys ? storeyRow(this.storeys, game.storey) : 0), townView.radius);
    const ac = audioContext();
    const out = bus('world');
    const around = space('world');
    if (!ac || !out || !around || out.gain.value === 0) return;
    const quiet = document.hidden;
    const t = ac.currentTime;
    around.distance(acoustics.radius, t);

    // Ambience by the time of day.
    const { curr, layout } = frame;
    const minute = curr[layout.header.minute];
    const hour = (minute / 60) % 24;
    const day = daylight(hour);
    const night = 1 - smooth(4.5, 6, hour) * (1 - smooth(19.5, 21, hour));
    this.bed ??= this.loop(ac, out, mono(ac, 'bed'));
    this.crickets ??= this.loop(ac, out, mono(ac, 'crickets'));
    fadeTo(this.bed.gain, quiet ? 0 : AMBIENCE.bed.night + AMBIENCE.bed.day * day, t);
    fadeTo(this.crickets.gain, quiet ? 0 : AMBIENCE.crickets * night, t);
    this.birds.update(ac, out, now, hour, quiet, this.renderer);

    // Things in use on the lot in view, and knocks at the door.
    const k = layout.sim;
    const count = curr[layout.header.simCount];
    const paused = game.speed === 0;
    const users = new Map<LoopKind, Users>();
    const seen = new Set<number>();
    const bathing = new Set<number>();
    const once: Heard['once'] = [];
    const here = new Set<number>();
    for (const kind of this.onceHere) this.made.sound(kind, ac.sampleRate);
    for (const a of this.ambient) if (a.while) a.while.on = false;
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      const action = curr[o + k.action];
      const tag = action >= 0 ? layout.actions[action] : undefined;
      const id = curr[o + k.id];
      here.add(id);
      // Where they stand (the world's rows: their storey's).
      const sx = curr[o + k.x];
      const sz = curr[o + k.z];
      // Cooking's clearing away and repairs are at an object without a use of it: by what that object is.
      const object = curr[o + k.object];
      const def = k.objectDef !== undefined && curr[o + k.objectDef] >= 0 ? curr[o + k.objectDef] : object >= 0 ? (this.defOf.get(object) ?? -1) : -1;
      const interaction = k.interaction === undefined ? -1 : curr[o + k.interaction];
      // A use that sounds as it ends (the flush): heard once they've moved on from it.
      const use = tag && def >= 0 && interaction >= 0 ? def * 1024 + interaction : -1;
      const was = this.ending.get(id);
      if (was && was.use !== use) {
        this.ending.delete(id);
        if (!quiet && this.renderer.simHead(i, this.head) && this.once(ac, around, was.kind, placeAt(this.panAt(), sx, sz))) once.push({ kind: was.kind, at: now });
      }
      const end = use >= 0 ? endSoundOf(this.content, def, interaction) : undefined;
      if (end) {
        this.ending.set(id, { use, kind: end });
        // Made while the use lasts, so it's there as it ends.
        this.made.sound(end, ac.sampleRate);
      }
      const drawn = tag !== undefined && this.renderer.simHead(i, this.head);
      let kind = tag === undefined ? undefined : soundOf(this.content, tag, def, interaction, k.recipe === undefined ? 0 : curr[o + k.recipe]);
      // A sound played once as they start something (a fridge door): heard if they're drawn, once it's made.
      const doing = tag === undefined ? '' : `${object}:${interaction}:${tag}`;
      const before = this.doing.get(id);
      if (before !== doing) {
        if (before === undefined || !kind || !isOnce(kind) || !drawn || quiet) this.doing.set(id, doing);
        else if (!paused && this.once(ac, around, kind, placeAt(this.panAt(), sx, sz))) {
          this.doing.set(id, doing);
          once.push({ kind, at: now });
        }
      }
      if (!tag || !drawn) continue;
      const pan = this.panAt();
      if (tag === 'knock') {
        seen.add(id);
        const next = this.knocking.get(id) ?? 0;
        if (!quiet && !paused && now >= next) play(ac, around, mono(ac, 'knock'), AMBIENCE.knock, placeAt(pan, sx, sz));
        if (now >= next) this.knocking.set(id, now + 2600 + Math.random() * 1200);
        continue;
      }
      if (!kind || isOnce(kind)) continue;
      // A cooker hood over a hob in use draws the steam off.
      const tiles = this.tilesOf.get(object);
      for (const a of this.ambient) if (a.while && a.while.kind === kind && tiles && overlaps(a.while.tiles, tiles)) a.while.on = true;
      if (kind === 'bathFill') {
        // The bath runs while they get in, then it's just the water lapping.
        bathing.add(id);
        const since = this.bathers.get(id) ?? now;
        this.bathers.set(id, since);
        if (now - since > BATH_FILL * 1000) kind = 'bathSoak';
      }
      join(users, kind, pan, sx, sz);
    }
    // Things sounding by themselves, unless the town is shown instead of the lot.
    if (!game.townOpen) {
      for (const a of this.ambient) {
        if (a.while && !a.while.on) continue;
        this.head.x = a.x;
        this.head.y = a.y;
        this.head.z = a.z;
        join(users, a.kind, this.panAt(), a.x, a.row);
      }
    }
    for (const id of this.ending.keys()) if (!here.has(id)) this.ending.delete(id);
    for (const id of this.doing.keys()) if (!here.has(id)) this.doing.delete(id);
    this.heard = {
      loops: Object.fromEntries([...users].map(([kind, u]) => [kind, u.n])),
      places: Object.fromEntries([...users].map(([kind, u]) => [kind, placeOf(u)])),
      once: [...this.heard.once.filter((s) => now - s.at < 10_000), ...once],
      radius: acoustics.radius,
    };
    for (const id of this.knocking.keys()) if (!seen.has(id)) this.knocking.delete(id);
    for (const id of this.bathers.keys()) if (!bathing.has(id)) this.bathers.delete(id);
    for (const kind of Object.keys(LOOPS) as LoopKind[]) {
      const u = users.get(kind);
      const def = LOOPS[kind];
      const target = u && !quiet && !paused ? def.level * Math.min(1.6, Math.sqrt(u.n)) : 0;
      let loop = this.loops.get(kind);
      if (!loop) {
        if (target === 0 || !u) continue;
        const made = kind in MONO ? mono(ac, kind as MonoKind) : this.made.sound(kind as MadeKind, ac.sampleRate);
        // Still being made: it starts when it's ready.
        if (!made) continue;
        loop = this.placedLoop(ac, around, made, placeOf(u));
        this.loops.set(kind, loop);
        if (def.drift) loop.source.playbackRate.value = 1 + def.drift * (Math.random() * 2 - 1);
      }
      fadeTo(loop.gain, target, t);
      if (u) loop.placed.place(placeOf(u), t);
      if (def.drift && target > 0 && Math.random() < 0.05) loop.source.playbackRate.setTargetAtTime(1 + def.drift * (Math.random() * 2 - 1), t, 1.5);
    }
  }

  /** Plays a sound once at `pan`; false while it's still being made. */
  private once(ac: AudioContext, around: Space, kind: OnceKind, place: Place): boolean {
    const made = this.made.sound(kind, ac.sampleRate);
    if (!made) return false;
    play(ac, around, made, ONCE_LEVEL[kind], place);
    return true;
  }

  /** No updates for a while (drawing stopped): loops would hold their last level, so they fade out (back on the next update). */
  private hush(): void {
    if (this.hushed || this.disposed || performance.now() - this.lastTick < STALE_MS) return;
    const ac = audioContext();
    if (!ac) return;
    this.hushed = true;
    for (const loop of [...this.loops.values(), this.bed, this.crickets]) if (loop) fadeTo(loop.gain, 0, ac.currentTime);
    this.birds.hush(ac.currentTime);
  }

  dispose(): void {
    this.disposed = true;
    clearInterval(this.watchdog);
    for (const loop of [...this.loops.values(), this.bed, this.crickets]) {
      if (!loop) continue;
      try {
        loop.source.stop();
      } catch {
        // Never started.
      }
      if ('placed' in loop) loop.placed.disconnect();
      else loop.panner.disconnect();
    }
    this.loops.clear();
    this.birds.dispose();
    this.made.dispose();
  }

  /** Stereo position from where the resident is on screen; off screen counts as far to the side. */
  private panAt(): number {
    return this.renderer.project(this.head.x, this.head.y, this.head.z, this.screen) ? screenPan(this.screen.x, 0.7) : 0;
  }

  /** A thing's loop at its place, silent until faded in; it starts somewhere along the loop, so it never sounds the same twice. */
  private placedLoop(ac: AudioContext, around: Space, made: AudioBuffer, place: Place): PlacedLoop {
    const source = ac.createBufferSource();
    source.buffer = made;
    source.loop = true;
    const gain = ac.createGain();
    gain.gain.value = 0;
    const placed = new Placed(ac, around, place);
    source.connect(gain).connect(placed.input);
    source.start(0, Math.random() * made.duration);
    return { source, gain, placed };
  }

  /** A looping buffer all round (the outdoors), silent until faded in; it starts somewhere along the loop, so it never sounds the same twice. */
  private loop(ac: AudioContext, out: AudioNode, made: AudioBuffer): Loop {
    const source = ac.createBufferSource();
    source.buffer = made;
    source.loop = true;
    const gain = ac.createGain();
    gain.gain.value = 0;
    const panner = ac.createStereoPanner();
    source.connect(gain).connect(panner).connect(out);
    source.start(0, Math.random() * made.duration);
    return { source, gain, panner };
  }
}

/**
 * Sounds that take a moment to make, made by a worker as audio buffers: the water sounds (the one
 * first wanted, then the rest one after another), the things' sounds and the birds' songs (each
 * when first wanted). Made on the spot if workers aren't available.
 */
class WorkerSounds {
  private worker: Worker | null = null;
  private rate = 0;
  private readonly ready = new Map<MadeKind, AudioBuffer>();
  private waterAsked = false;
  /** The things' sounds asked for. */
  private readonly asked = new Set<MadeKind>();
  /** Bird songs by `songKey`: made, or asked for (null). */
  private readonly songs = new Map<string, AudioBuffer | null>();

  /** The sound at the sample rate, or null while it's still being made. */
  sound(kind: MadeKind, rate: number): AudioBuffer | null {
    this.at(rate);
    let b = this.ready.get(kind);
    if (!b && !this.worker) {
      b = stereoBuffer(MADE[kind](rate), rate);
      this.ready.set(kind, b);
    } else if (!b && kind in WATER && !this.waterAsked) {
      this.waterAsked = true;
      const kinds = Object.keys(WATER) as MadeKind[];
      for (const k of [kind, ...kinds.filter((k) => k !== kind)]) this.worker!.postMessage({ kind: k, rate } satisfies SoundRequest);
    } else if (!b && !(kind in WATER) && !this.asked.has(kind)) {
      this.asked.add(kind);
      this.worker!.postMessage({ kind, rate } satisfies SoundRequest);
    }
    return b ?? null;
  }

  /** A version of a bird's song at the sample rate, or null while it's still being made. */
  song(bird: Voice, version: number, rate: number): AudioBuffer | null {
    this.at(rate);
    const key = songKey(bird, version);
    let b = this.songs.get(key);
    if (!b && !this.worker) {
      b = monoBuffer(songSamples(bird, version, rate), rate);
      this.songs.set(key, b);
    } else if (b === undefined) {
      this.songs.set(key, null);
      this.worker!.postMessage({ bird, version, rate } satisfies SoundRequest);
    }
    return b ?? null;
  }

  dispose(): void {
    this.worker?.terminate();
    this.worker = null;
  }

  /** Starts over at another sample rate (the worker starts the first time). */
  private at(rate: number): void {
    if (rate === this.rate) return;
    this.rate = rate;
    this.ready.clear();
    this.songs.clear();
    this.waterAsked = false;
    this.asked.clear();
    if (this.worker) return;
    try {
      this.worker = new Worker(new URL('./water.worker.ts', import.meta.url), { type: 'module' });
    } catch {
      return;
    }
    this.worker.onmessage = (e: MessageEvent<SoundReply>) => {
      const r = e.data;
      if (r.rate !== this.rate) return;
      if ('bird' in r) this.songs.set(songKey(r.bird, r.version), monoBuffer(r.samples, r.rate));
      else this.ready.set(r.kind, stereoBuffer(r, r.rate));
    };
    this.worker.onerror = () => this.dispose();
  }
}

const songKey = (bird: Voice, version: number) => `${bird.seed}:${version}`;

function stereoBuffer(s: Stereo, rate: number): AudioBuffer {
  const b = new AudioBuffer({ numberOfChannels: 2, length: s.left.length, sampleRate: rate });
  b.copyToChannel(s.left as Float32Array<ArrayBuffer>, 0);
  b.copyToChannel(s.right as Float32Array<ArrayBuffer>, 1);
  return b;
}

function monoBuffer(samples: Float32Array, rate: number): AudioBuffer {
  const b = new AudioBuffer({ numberOfChannels: 1, length: samples.length, sampleRate: rate });
  b.copyToChannel(samples as Float32Array<ArrayBuffer>, 0);
  return b;
}

/** An object's tiles (rows as the world's, so storeys apart don't overlap). */
interface Tiles {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** Whether two sets of tiles overlap. */
function overlaps(a: Tiles, b: Tiles): boolean {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.z < b.z + b.d && b.z < a.z + a.d;
}

function fadeTo(param: GainNode, value: number, t: number): void {
  param.gain.setTargetAtTime(value, t, FADE / 3);
}

/** 0 before `a`, 1 after `b`, a smooth step between. */
function smooth(a: number, b: number, x: number): number {
  const u = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return u * u * (3 - 2 * u);
}

/** How much of a bird day it is: up at dawn, gone after dusk. */
function daylight(hour: number): number {
  return smooth(4.5, 6, hour) * (1 - smooth(19, 20.5, hour));
}

// Birds: the garden birds in view sing from where they sit; a few further off, unseen, make a soft
// chorus behind them. Each of a species with its own song, singing in bouts by day.

/** The unseen birds further off: their species, where they are left to right, and the three that only join the dawn chorus. */
const FLOCK: { species: string; pan: number; dawn?: boolean }[] = [
  { species: 'tit', pan: -0.55 },
  { species: 'finch', pan: 0.5 },
  { species: 'warbler', pan: -0.15 },
  { species: 'dove', pan: 0.3 },
  { species: 'blackbird', pan: -0.7, dawn: true },
  { species: 'tit', pan: 0.8, dawn: true },
  { species: 'warbler', pan: -0.8, dawn: true },
];
/** Overall level of the birds. */
const BIRD_LEVEL = AMBIENCE.birds;
/** The unseen birds' level while birds in view sing (they're further off). */
const FAR_LEVEL = AMBIENCE.far;
/** A bird in view sings only this near the camera (m); its level falls off with distance. */
const HEARD_RANGE = 45;
/** A bird whose song is still being made tries again after this long (ms). */
const SONG_WAIT_MS = 250;

/** One of the birds heard. */
interface Bird {
  species: Species;
  /** Its own pitch, pace and nearness: what its songs are made from. */
  voice: Voice;
  /** Only sings in the dawn chorus. */
  dawn: boolean;
  /** Its place left to right. */
  pan: number;
  /** Songs left in its bout, when it sings next and when its song ends (ms, 0 = not yet set). */
  left: number;
  next: number;
  until: number;
  /** Its level and place, made on its first song. */
  out: { gain: GainNode; panner: StereoPannerNode } | null;
  /** A bird in view (the renderer's `SeenBird.id`): its level by distance, whether it may sing now, and the tick it was last in view. */
  id?: number;
  near?: number;
  free?: boolean;
  seenAt?: number;
}

/** A bird of `species` heard, its songs its own (pitch and pace from `r`, made from `seed`); not yet placed. */
function hearBird(species: Species, seed: number, r: () => number): Bird {
  const voice = { species: species.name, seed, pitch: between(r, species.pitch), pace: 0.92 + r() * 0.16, near: 1 };
  return { species, voice, dawn: false, pan: 0, left: 0, next: 0, until: 0, out: null };
}

/** Stereo position from where something is on screen (`x`, px), `scale` of the way to the sides. */
function screenPan(x: number, scale: number): number {
  const pan = Math.max(-1, Math.min(1, (x / (window.innerWidth || 1)) * 2 - 1)) * scale;
  // Web Audio throws on a non-finite value (and that would stop the frame).
  return Number.isFinite(pan) ? pan : 0;
}

/**
 * The birds heard: those the renderer draws near the camera (each sings from where it sits, louder
 * nearer, and lifts its head as it does) and a few unseen ones further off, the same every time.
 * Each sings its song a few times with pauses, then rests; when one sings, another sometimes
 * answers. Busiest in the dawn chorus, quieter around midday, some evening song, none at night
 * (the drawn birds roost then).
 */
class Birds {
  private readonly flock: Bird[];
  /** The birds in view, by their id; and everyone heard (the unseen and those), made again as those change. */
  private readonly seen = new Map<number, Bird>();
  private everyone: Bird[];
  private tick = 0;
  private readonly screen = { x: 0, y: 0 };

  /** `made` makes their songs (off the main thread). */
  constructor(private readonly made: WorkerSounds) {
    const r = random(0x5eed1e55);
    this.flock = FLOCK.map(({ species: name, pan, dawn = false }, i) => {
      const b = hearBird(SPECIES.find((sp) => sp.name === name) ?? SPECIES[0], i, r);
      b.pan = pan + (r() - 0.5) * 0.15;
      b.voice.near = dawn ? 0.3 + r() * 0.15 : 0.45 + r() * 0.25;
      b.dawn = dawn;
      return b;
    });
    this.everyone = this.flock;
  }

  update(ac: AudioContext, out: AudioNode, now: number, hour: number, quiet: boolean, renderer: Renderer): void {
    const day = daylight(hour);
    const chorus = smooth(4.5, 5.5, hour) * (1 - smooth(7, 8.5, hour));
    // How busy the birds are (as the birds drawn): most at dawn and in the early evening, least around midday.
    const busy = day * birdActivity(hour);
    const t = ac.currentTime;
    const heard = this.look(renderer, now);
    const all = this.everyone;
    // Birds mostly take turns; in the dawn chorus up to three sing at once.
    const crowd = 1 + Math.round(2 * chorus);
    let singing = 0;
    for (const b of all) if (b.species.turns && b.until > now) singing++;
    const far = heard ? FAR_LEVEL : 1;
    for (const b of all) {
      const sp = b.species;
      const inView = b.id !== undefined;
      const active = inView ? (b.free ? busy : 0) : b.dawn ? day * chorus : busy;
      if (quiet && b.out) fadeTo(b.out.gain, 0, t);
      // One that flies off (or out of view) stops singing.
      if (inView && !b.free && b.until > now && b.out) {
        fadeTo(b.out.gain, 0, t);
        b.until = now;
      }
      if (now < b.next) continue;
      if (quiet || active < 0.05 || b.next === 0) {
        // Silent for now; when it may sing again, it starts after a while.
        b.left = 0;
        b.next = now + Math.random() * sp.rest[0] * (inView ? 100 : 300);
        continue;
      }
      if (sp.turns && singing >= crowd) {
        b.next = now + 300 + Math.random() * 1200;
        continue;
      }
      // A version of its song (made off the main thread the first time: it sings once that's done).
      const song = this.made.song(b.voice, Math.floor(Math.random() * VERSIONS), ac.sampleRate);
      if (!song) {
        b.next = now + SONG_WAIT_MS;
        continue;
      }
      if (b.left <= 0) b.left = Math.round(between(Math.random, sp.bout) * (1 + 0.5 * chorus));
      const level = BIRD_LEVEL * sp.level * b.voice.near * (0.5 + 0.5 * day) * (inView ? (b.near ?? 1) : far);
      this.sing(ac, out, b, song, t + 0.03 + Math.random() * 0.05, level);
      b.until = now + song.duration * 1000;
      if (inView) renderer.birdSings(b.id!, song.duration + 0.08);
      if (sp.turns) singing++;
      b.left--;
      if (b.left > 0) b.next = b.until + between(Math.random, sp.pause) * b.voice.pace * (1.4 - 0.4 * active) * 1000;
      else b.next = b.until + (between(Math.random, sp.rest) * 1000) / active;
      // Sometimes another bird answers.
      const other = all[Math.floor(Math.random() * all.length)];
      if (sp.turns && other !== b && other.species.turns && other.left <= 0 && Math.random() < 0.15 + 0.25 * active) {
        other.next = Math.min(other.next, b.until + 500 + Math.random() * 1500);
      }
    }
  }

  /** Fades out whoever is singing. */
  hush(t: number): void {
    for (const b of this.everyone) if (b.out) fadeTo(b.out.gain, 0, t);
  }

  dispose(): void {
    for (const b of this.everyone) b.out?.panner.disconnect();
  }

  /**
   * The birds in view near the camera, as the renderer draws them now: a new one gets a voice of its
   * own (from its id), one gone is forgotten once its song has faded; each one's place left to right
   * and its level by distance. Whether any is in view.
   */
  private look(renderer: Renderer, now: number): boolean {
    const width = window.innerWidth || 1;
    const height = window.innerHeight || 1;
    const tick = ++this.tick;
    let here = 0;
    let changed = false;
    for (const s of renderer.birds()) {
      const on = s.distance < HEARD_RANGE && renderer.project(s.x, s.y, s.z, this.screen) && this.screen.x >= 0 && this.screen.x <= width && this.screen.y >= 0 && this.screen.y <= height;
      if (!on) continue;
      let b = this.seen.get(s.id);
      if (!b) {
        const species = SPECIES.find((sp) => sp.name === s.song);
        if (!species) continue;
        // (Seeds past the unseen birds': songs of its own.)
        b = hearBird(species, 1000 + (s.id % 1_000_000), random(s.id));
        b.voice.near = 0.9;
        b.id = s.id;
        this.seen.set(s.id, b);
        changed = true;
      }
      b.seenAt = tick;
      here++;
      b.free = s.perched;
      b.pan = screenPan(this.screen.x, 0.8);
      b.near = Math.min(1.3, 8 / (s.distance + 4));
    }
    for (const [id, b] of this.seen) {
      if (b.seenAt === tick) continue;
      b.free = false;
      if (b.until > now - 2000) continue;
      b.out?.panner.disconnect();
      this.seen.delete(id);
      changed = true;
    }
    if (changed) this.everyone = this.seen.size ? [...this.flock, ...this.seen.values()] : this.flock;
    return here > 0;
  }

  /** Plays one version of the bird's song, from where it is. */
  private sing(ac: AudioContext, out: AudioNode, b: Bird, song: AudioBuffer, at: number, level: number): void {
    if (!b.out) {
      const gain = ac.createGain();
      const panner = ac.createStereoPanner();
      panner.pan.value = b.pan;
      gain.connect(panner).connect(out);
      b.out = { gain, panner };
    }
    if (b.id !== undefined) b.out.panner.pan.setValueAtTime(b.pan, at);
    const source = ac.createBufferSource();
    source.buffer = song;
    source.connect(b.out.gain);
    b.out.gain.gain.setValueAtTime(level, at);
    source.onended = () => source.disconnect();
    source.start(at);
  }
}

function between(r: () => number, [a, b]: [number, number]): number {
  return a + r() * (b - a);
}

/** Plays a sound once at `level`, at its place. */
function play(ac: AudioContext, around: Space, made: AudioBuffer, level: number, place: Place): void {
  const source = ac.createBufferSource();
  source.buffer = made;
  const gain = ac.createGain();
  gain.gain.value = level;
  const placed = new Placed(ac, around, place);
  source.connect(gain).connect(placed.input);
  // (Its room goes on a moment after it: the longest response is under a second.)
  source.onended = () => setTimeout(() => placed.disconnect(), 1000);
  source.start();
}

/** Where a loop's users are on average, as its place: the share of them in rooms of each size (the rest outdoors). */
function placeOf(u: Users): Place {
  return { pan: u.pan, muffle: u.muffle, rooms: { small: u.rooms.small / u.n, medium: u.rooms.medium / u.n, large: u.rooms.large / u.n } };
}

/** The sounds of one channel (`mono.ts`) as audio buffers, made once per sample rate. */
const monos = new Map<MonoKind, AudioBuffer>();

function mono(ac: AudioContext, kind: MonoKind): AudioBuffer {
  let b = monos.get(kind);
  if (!b || b.sampleRate !== ac.sampleRate) {
    b = monoBuffer(MONO[kind](ac.sampleRate), ac.sampleRate);
    monos.set(kind, b);
  }
  return b;
}
