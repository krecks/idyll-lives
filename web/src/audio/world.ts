/**
 * Sounds of the world, synthesised like the interface sounds (no sound files):
 * - ambience by the game clock: a soft outdoor bed, crickets at night, and by day a few neighbourhood
 *   birds of different species, each with its own song, singing in bouts (most in the dawn chorus);
 * - things in use on the lot being looked at: a sizzling pan, a shower, a tap, washing up, a bath, the
 *   washing machine and the dryer, the dishwasher being loaded, a TV, music, drums. One loop per kind
 *   of sound, louder the more residents make it, placed left or right like voices; the water sounds
 *   are made in `water.ts`. What a use sounds like comes from the thing (content `sound` and
 *   `sounds`), else from its animation (`ACTION_SOUNDS`);
 * - things that sound by themselves (content `ambient`: a fountain), while their lot is looked at;
 * - sounds as something happens: a toilet's flush as its use ends (content `endSound`), a
 *   visitor's knock at the front door.
 *
 * Presentation only. Reads the snapshot about ten times a second; residents the renderer doesn't
 * draw (other lots, at work) make no sound.
 */

import type { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import { storeyOfRow, storeyRow, type WorldStructure } from '../core/protocol';
import { STOREY_HEIGHT, type Renderer, type ViewRect } from '../render/types';
import { game } from '../ui/state.svelte';
import { songSamples, SPECIES, VERSIONS, type Species, type Voice } from './birdsong';
import { audioContext, bus, updateVolumes } from './mixer';
import { ONCE, WATER, type Stereo, type WaterKind } from './water';
import type { SoundReply, SoundRequest } from './water.worker';

const TICK_MS = 100;
/** Updates come with drawn frames: this long without one (the picture stopped), everything fades out. */
const STALE_MS = 2000;
/** Seconds for a loop to fade in or out. */
const FADE = 0.4;
/** Seconds a bath runs before the water's turned off and the bather settles in. */
const BATH_FILL = 7;

/** A sound played once (`ONCE`), not looped. */
type OnceKind = (typeof ONCE)[number];
export type LoopKind = 'sizzle' | 'chop' | Exclude<WaterKind, OnceKind> | 'tv' | 'music' | 'drums';

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
};

interface LoopDef {
  /** The raw sound, made here; or one of the water sounds, made off the main thread. */
  make?: (ac: AudioContext) => AudioBuffer;
  water?: WaterKind;
  /** A filter on the way out (shapes the raw buffer into the sound). */
  filter?: { type: BiquadFilterType; frequency: number; q: number };
  level: number;
  /** How far the speed (and with it the pitch) wanders either way, so a loop doesn't sound the same each time round. */
  drift?: number;
}

const LOOPS: Record<LoopKind, LoopDef> = {
  sizzle: { make: sizzleBuffer, filter: { type: 'highpass', frequency: 1800, q: 0.5 }, level: 0.22 },
  chop: { make: chopBuffer, filter: { type: 'bandpass', frequency: 1400, q: 0.8 }, level: 0.3 },
  shower: { water: 'shower', level: 0.28, drift: 0.04 },
  tap: { water: 'tap', level: 0.22, drift: 0.04 },
  washUp: { water: 'washUp', level: 0.24, drift: 0.03 },
  bathFill: { water: 'bathFill', level: 0.26, drift: 0.04 },
  bathSoak: { water: 'bathSoak', level: 0.35, drift: 0.03 },
  washer: { water: 'washer', level: 0.2, drift: 0.02 },
  dryer: { water: 'dryer', level: 0.18, drift: 0.02 },
  crockery: { water: 'crockery', level: 0.35 },
  fountain: { water: 'fountain', level: 0.2, drift: 0.03 },
  tv: { make: murmurBuffer, filter: { type: 'bandpass', frequency: 700, q: 1.1 }, level: 0.35 },
  music: { make: musicBuffer, filter: { type: 'lowpass', frequency: 2600, q: 0.7 }, level: 0.16 },
  drums: { make: drumsBuffer, filter: { type: 'lowpass', frequency: 7000, q: 0.7 }, level: 0.2 },
};

/** Levels of the sounds played once. */
const ONCE_LEVEL: Record<OnceKind, number> = { flush: 0.3 };

/** Content `sound` that silences a use whose animation would otherwise make one (toast isn't fried). */
const SILENT = 'none';

/**
 * What a resident's action sounds like, by what they use: the use's own `sound` (not a recipe's,
 * whose steps sound by where they're done), else the cooking step's (`recipe`: the snapshot's,
 * index + 1; a bake), else the object's for that animation (`sounds`: an oven's `stir` is silent),
 * else by the animation (`ACTION_SOUNDS`), with washing by what's washed. `def` and `interaction`
 * are the snapshot's (-1: none).
 */
export function soundOf(content: Content, tag: string, def: number, interaction: number, recipe = 0): LoopKind | undefined {
  const object = def >= 0 ? content.objectList[def] : undefined;
  const use = interaction >= 0 ? object?.interactions[interaction] : undefined;
  const step = recipe > 0 ? content.recipes[recipe - 1]?.steps.find((s) => s.anim === tag && s.sound !== undefined) : undefined;
  const own = (use?.recipe ? undefined : use?.sound) ?? step?.sound ?? object?.sounds?.[tag];
  if (own !== undefined) return loopKind(own);
  if (tag === 'wash') {
    // Clearing away after a meal washes up at the sink without an interaction of its own.
    if (use?.id === 'washUp' || !use) return 'washUp';
    // Getting ready at a dressing table washes without a tap.
    return use?.tags?.includes('handwash') ? 'tap' : undefined;
  }
  return ACTION_SOUNDS[tag];
}

/** The sound played once as the use ends (content `endSound`: a toilet's flush), if any. */
export function endSoundOf(content: Content, def: number, interaction: number): OnceKind | undefined {
  const key = def >= 0 && interaction >= 0 ? content.objectList[def]?.interactions[interaction]?.endSound : undefined;
  return key === undefined ? undefined : (ONCE as readonly string[]).includes(key) ? (key as OnceKind) : unknownSound(key);
}

/** A content sound key as a loop (`none`: silence); unknown keys are silent (and said once). */
function loopKind(key: string): LoopKind | undefined {
  if (key === SILENT) return undefined;
  return key in LOOPS ? (key as LoopKind) : unknownSound(key);
}

const unknown = new Set<string>();
function unknownSound(key: string): undefined {
  if (!unknown.has(key)) console.warn(`content: no world sound "${key}"`);
  unknown.add(key);
  return undefined;
}

/** Something that sounds by itself (content `ambient`), where it stands. */
interface Ambient {
  kind: LoopKind;
  x: number;
  y: number;
  z: number;
}

interface Loop {
  source: AudioBufferSourceNode;
  gain: GainNode;
  panner: StereoPannerNode;
}

export class WorldSound {
  private lastTick = 0;
  private readonly loops = new Map<LoopKind, Loop>();
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
  /** Things sounding by themselves on the lot in view. */
  private ambient: Ambient[] = [];
  /** The definitions (index into `content.objectList`) of the world's objects, by their stable id. */
  private readonly defOf = new Map<number, number>();
  /** What sounded on the last update (`playing`). */
  private heard: { loops: Partial<Record<LoopKind, number>>; once: { kind: OnceKind; at: number }[] } = { loops: {}, once: [] };
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
    this.defOf.clear();
    const index = new Map(this.content.objectList.map((o, i) => [o.id, i]));
    for (const o of world.objects) this.defOf.set(o.id, index.get(o.def) ?? -1);
    if (!view) return;
    for (const o of world.objects) {
      const key = this.content.object(o.def)?.ambient;
      const kind = key === undefined ? undefined : loopKind(key);
      // Broken (worn out), it's still.
      if (!kind || (o.wear ?? 0) >= 1) continue;
      const storey = storeyOfRow(world, o.z);
      const x = o.x + o.w / 2;
      const z = o.z - storeyRow(world, storey) + o.d / 2;
      if (x < view.x || x > view.x + view.w || z < view.z || z > view.z + view.d) continue;
      this.ambient.push({ kind, x, y: storey * STOREY_HEIGHT + 0.6, z });
    }
  }

  /** What sounded on the last update: residents and things per loop, and the sounds played once (tests). */
  playing(): { loops: Partial<Record<LoopKind, number>>; once: { kind: OnceKind; at: number }[] } {
    return this.heard;
  }

  update(frame: FrameState): void {
    const now = frame.now;
    if (now - this.lastTick < TICK_MS || this.disposed) return;
    this.lastTick = now;
    this.hushed = false;
    updateVolumes();
    const ac = audioContext();
    const out = bus('world');
    if (!ac || !out || out.gain.value === 0) return;
    const quiet = document.hidden;
    const t = ac.currentTime;

    // Ambience by the time of day.
    const { curr, layout } = frame;
    const minute = curr[layout.header.minute];
    const hour = (minute / 60) % 24;
    const day = daylight(hour);
    const night = 1 - smooth(4.5, 6, hour) * (1 - smooth(19.5, 21, hour));
    this.bed ??= this.loop(ac, out, brownBuffer, { type: 'lowpass', frequency: 650, q: 0.5 });
    this.crickets ??= this.loop(ac, out, cricketBuffer);
    fadeTo(this.bed.gain, quiet ? 0 : 0.05 + 0.03 * day, t);
    fadeTo(this.crickets.gain, quiet ? 0 : 0.1 * night, t);
    this.birds.update(ac, out, now, hour, quiet);

    // Things in use on the lot in view, and knocks at the door.
    const k = layout.sim;
    const count = curr[layout.header.simCount];
    const paused = game.speed === 0;
    const users = new Map<LoopKind, { n: number; pan: number }>();
    const seen = new Set<number>();
    const bathing = new Set<number>();
    const once: { kind: OnceKind; at: number }[] = [];
    const here = new Set<number>();
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      const action = curr[o + k.action];
      const tag = action >= 0 ? layout.actions[action] : undefined;
      const id = curr[o + k.id];
      here.add(id);
      // Cooking's clearing away and repairs are at an object without a use of it: by what that object is.
      const object = curr[o + k.object];
      const def = k.objectDef !== undefined && curr[o + k.objectDef] >= 0 ? curr[o + k.objectDef] : object >= 0 ? (this.defOf.get(object) ?? -1) : -1;
      const interaction = k.interaction === undefined ? -1 : curr[o + k.interaction];
      // A use that sounds as it ends (the flush): heard once they've moved on from it.
      const use = tag && def >= 0 && interaction >= 0 ? def * 1024 + interaction : -1;
      const was = this.ending.get(id);
      if (was && was.use !== use) {
        this.ending.delete(id);
        if (!quiet && this.renderer.simHead(i, this.head) && this.once(ac, out, was.kind, this.panAt())) once.push({ kind: was.kind, at: now });
      }
      const end = use >= 0 ? endSoundOf(this.content, def, interaction) : undefined;
      if (end) {
        this.ending.set(id, { use, kind: end });
        // Made while the use lasts, so it's there as it ends.
        this.made.water(end, ac.sampleRate);
      }
      if (!tag || !this.renderer.simHead(i, this.head)) continue;
      const pan = this.panAt();
      if (tag === 'knock') {
        seen.add(id);
        const next = this.knocking.get(id) ?? 0;
        if (!quiet && !paused && now >= next) knock(ac, out, pan);
        if (now >= next) this.knocking.set(id, now + 2600 + Math.random() * 1200);
        continue;
      }
      let kind = soundOf(this.content, tag, def, interaction, k.recipe === undefined ? 0 : curr[o + k.recipe]);
      if (!kind) continue;
      if (kind === 'bathFill') {
        // The bath runs while they get in, then it's just the water lapping.
        bathing.add(id);
        const since = this.bathers.get(id) ?? now;
        this.bathers.set(id, since);
        if (now - since > BATH_FILL * 1000) kind = 'bathSoak';
      }
      const u = users.get(kind) ?? { n: 0, pan: 0 };
      u.pan = (u.pan * u.n + pan) / (u.n + 1);
      u.n++;
      users.set(kind, u);
    }
    // Things sounding by themselves, unless the town is shown instead of the lot.
    if (!game.townOpen) {
      for (const a of this.ambient) {
        Object.assign(this.head, a);
        const u = users.get(a.kind) ?? { n: 0, pan: 0 };
        u.pan = (u.pan * u.n + this.panAt()) / (u.n + 1);
        u.n++;
        users.set(a.kind, u);
      }
    }
    for (const id of this.ending.keys()) if (!here.has(id)) this.ending.delete(id);
    this.heard = { loops: Object.fromEntries([...users].map(([kind, u]) => [kind, u.n])), once: [...this.heard.once.filter((s) => now - s.at < 10_000), ...once] };
    for (const id of this.knocking.keys()) if (!seen.has(id)) this.knocking.delete(id);
    for (const id of this.bathers.keys()) if (!bathing.has(id)) this.bathers.delete(id);
    for (const kind of Object.keys(LOOPS) as LoopKind[]) {
      const u = users.get(kind);
      const def = LOOPS[kind];
      const target = u && !quiet && !paused ? def.level * Math.min(1.6, Math.sqrt(u.n)) : 0;
      let loop = this.loops.get(kind);
      if (!loop && target === 0) continue;
      if (!loop) {
        const made = def.water ? this.made.water(def.water, ac.sampleRate) : def.make;
        // Water still being made: it starts when it's ready.
        if (!made) continue;
        loop = this.loop(ac, out, made, def.filter);
        this.loops.set(kind, loop);
        if (def.drift) loop.source.playbackRate.value = 1 + def.drift * (Math.random() * 2 - 1);
      }
      fadeTo(loop.gain, target, t);
      if (u) loop.panner.pan.setTargetAtTime(u.pan, t, 0.1);
      if (def.drift && target > 0 && Math.random() < 0.05) loop.source.playbackRate.setTargetAtTime(1 + def.drift * (Math.random() * 2 - 1), t, 1.5);
    }
  }

  /** Plays a sound once at `pan`; false while it's still being made. */
  private once(ac: AudioContext, out: AudioNode, kind: OnceKind, pan: number): boolean {
    const made = this.made.water(kind, ac.sampleRate);
    if (!made) return false;
    const source = ac.createBufferSource();
    source.buffer = made;
    const gain = ac.createGain();
    gain.gain.value = ONCE_LEVEL[kind];
    const panner = ac.createStereoPanner();
    panner.pan.value = pan;
    source.connect(gain).connect(panner).connect(out);
    source.onended = () => panner.disconnect();
    source.start();
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
      loop.panner.disconnect();
    }
    this.loops.clear();
    this.birds.dispose();
    this.made.dispose();
  }

  /** Stereo position from where the resident is on screen; off screen counts as far to the side. */
  private panAt(): number {
    if (!this.renderer.project(this.head.x, this.head.y, this.head.z, this.screen)) return 0;
    const width = window.innerWidth || 1;
    const pan = Math.max(-1, Math.min(1, (this.screen.x / width) * 2 - 1)) * 0.7;
    // Web Audio throws on a non-finite value (and that would stop the frame).
    return Number.isFinite(pan) ? pan : 0;
  }

  /** A looping buffer, silent until faded in; it starts somewhere along the loop, so it never sounds the same twice. */
  private loop(ac: AudioContext, out: AudioNode, made: AudioBuffer | ((ac: AudioContext) => AudioBuffer), filter?: LoopDef['filter']): Loop {
    const source = ac.createBufferSource();
    source.buffer = typeof made === 'function' ? cached(ac, made) : made;
    source.loop = true;
    const gain = ac.createGain();
    gain.gain.value = 0;
    const panner = ac.createStereoPanner();
    let node: AudioNode = source;
    if (filter) {
      const f = ac.createBiquadFilter();
      f.type = filter.type;
      f.frequency.value = filter.frequency;
      f.Q.value = filter.q;
      node = node.connect(f);
    }
    node.connect(gain).connect(panner).connect(out);
    source.start(0, Math.random() * source.buffer.duration);
    return { source, gain, panner };
  }
}

/**
 * Sounds that take a moment to make, made by a worker as audio buffers: the water sounds (the one
 * first wanted, then the rest one after another) and the birds' songs (each when first wanted).
 * Made on the spot if workers aren't available.
 */
class WorkerSounds {
  private worker: Worker | null = null;
  private rate = 0;
  private readonly ready = new Map<WaterKind, AudioBuffer>();
  private waterAsked = false;
  /** Bird songs by `songKey`: made, or asked for (null). */
  private readonly songs = new Map<string, AudioBuffer | null>();

  /** The water sound at the sample rate, or null while it's still being made. */
  water(kind: WaterKind, rate: number): AudioBuffer | null {
    this.at(rate);
    let b = this.ready.get(kind);
    if (!b && !this.worker) {
      b = stereoBuffer(WATER[kind](rate), rate);
      this.ready.set(kind, b);
    } else if (!b && !this.waterAsked) {
      this.waterAsked = true;
      const kinds = Object.keys(WATER) as WaterKind[];
      for (const k of [kind, ...kinds.filter((k) => k !== kind)]) this.worker!.postMessage({ kind: k, rate } satisfies SoundRequest);
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

// Birds: a few neighbourhood birds, each of a species with its own song, singing in bouts by day.

/** The neighbourhood's birds: their species, where they sit left to right, and the two further off that only join the dawn chorus. */
const FLOCK: { species: string; pan: number; dawn?: boolean }[] = [
  { species: 'tit', pan: -0.55 },
  { species: 'finch', pan: 0.5 },
  { species: 'warbler', pan: -0.15 },
  { species: 'dove', pan: 0.3 },
  { species: 'tit', pan: 0.8, dawn: true },
  { species: 'warbler', pan: -0.8, dawn: true },
];
/** Overall level of the birds. */
const BIRD_LEVEL = 0.05;
/** A bird whose song is still being made tries again after this long (ms). */
const SONG_WAIT_MS = 250;

/** One of the neighbourhood's birds. */
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
}

/**
 * The neighbourhood's birds: the same few every time, of different species. Each sings its song a few
 * times with pauses, then rests; when one sings, another sometimes answers. Busiest in the dawn
 * chorus, quieter around midday, some evening song, none at night.
 */
class Birds {
  private readonly flock: Bird[];

  /** `made` makes their songs (off the main thread). */
  constructor(private readonly made: WorkerSounds) {
    const r = random(0x5eed1e55);
    this.flock = FLOCK.map(({ species: name, pan, dawn = false }, i) => {
      const species = SPECIES.find((sp) => sp.name === name) ?? SPECIES[0];
      const pitch = between(r, species.pitch);
      const pace = 0.92 + r() * 0.16;
      const place = pan + (r() - 0.5) * 0.15;
      const near = dawn ? 0.35 + r() * 0.15 : 0.6 + r() * 0.4;
      return { species, voice: { species: species.name, seed: i, pitch, pace, near }, dawn, pan: place, left: 0, next: 0, until: 0, out: null };
    });
  }

  update(ac: AudioContext, out: AudioNode, now: number, hour: number, quiet: boolean): void {
    const day = daylight(hour);
    const chorus = smooth(4.5, 5.5, hour) * (1 - smooth(7, 8.5, hour));
    const midday = smooth(10.5, 12, hour) * (1 - smooth(14.5, 16.5, hour));
    // How busy the birds are: most in the dawn chorus, least around midday, a little less in the evening.
    const busy = day * Math.min(1, 0.5 - 0.25 * midday - 0.1 * smooth(14, 16, hour) + 0.5 * chorus);
    const t = ac.currentTime;
    // Birds mostly take turns; in the dawn chorus up to three sing at once.
    const crowd = 1 + Math.round(2 * chorus);
    let singing = 0;
    for (const b of this.flock) if (b.species.turns && b.until > now) singing++;
    for (const b of this.flock) {
      const sp = b.species;
      const active = b.dawn ? day * chorus : busy;
      if (quiet && b.out) fadeTo(b.out.gain, 0, t);
      if (now < b.next) continue;
      if (quiet || active < 0.05 || b.next === 0) {
        // Silent for now; when it may sing again, it starts after a while.
        b.left = 0;
        b.next = now + Math.random() * sp.rest[0] * 300;
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
      this.sing(ac, out, b, song, t + 0.03 + Math.random() * 0.05, BIRD_LEVEL * sp.level * b.voice.near * (0.5 + 0.5 * day));
      b.until = now + song.duration * 1000;
      if (sp.turns) singing++;
      b.left--;
      if (b.left > 0) b.next = b.until + between(Math.random, sp.pause) * b.voice.pace * (1.4 - 0.4 * active) * 1000;
      else b.next = b.until + (between(Math.random, sp.rest) * 1000) / active;
      // Sometimes another bird answers.
      const other = this.flock[Math.floor(Math.random() * this.flock.length)];
      if (sp.turns && other !== b && other.species.turns && other.left <= 0 && Math.random() < 0.15 + 0.25 * active) {
        other.next = Math.min(other.next, b.until + 500 + Math.random() * 1500);
      }
    }
  }

  /** Fades out whoever is singing. */
  hush(t: number): void {
    for (const b of this.flock) if (b.out) fadeTo(b.out.gain, 0, t);
  }

  dispose(): void {
    for (const b of this.flock) b.out?.panner.disconnect();
  }

  /** Plays one version of the bird's song. */
  private sing(ac: AudioContext, out: AudioNode, b: Bird, song: AudioBuffer, at: number, level: number): void {
    if (!b.out) {
      const gain = ac.createGain();
      const panner = ac.createStereoPanner();
      panner.pan.value = b.pan;
      gain.connect(panner).connect(out);
      b.out = { gain, panner };
    }
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

/** Three knocks on a wooden door. */
function knock(ac: AudioContext, out: AudioNode, pan: number): void {
  const panner = ac.createStereoPanner();
  panner.pan.value = pan;
  panner.connect(out);
  const t = ac.currentTime + 0.02;
  for (let n = 0; n < 3; n++) {
    const at = t + n * 0.17 + (n === 2 ? 0.02 : 0);
    const osc = ac.createOscillator();
    const env = ac.createGain();
    osc.frequency.setValueAtTime(220, at);
    osc.frequency.exponentialRampToValueAtTime(95, at + 0.08);
    env.gain.setValueAtTime(0.0001, at);
    env.gain.exponentialRampToValueAtTime(0.5, at + 0.004);
    env.gain.exponentialRampToValueAtTime(0.0001, at + 0.11);
    osc.connect(env).connect(panner);
    osc.start(at);
    osc.stop(at + 0.13);
    const click = ac.createBufferSource();
    click.buffer = cached(ac, whiteBuffer);
    const band = ac.createBiquadFilter();
    band.type = 'bandpass';
    band.frequency.value = 1400;
    band.Q.value = 1.2;
    const clickEnv = ac.createGain();
    clickEnv.gain.setValueAtTime(0.25, at);
    clickEnv.gain.exponentialRampToValueAtTime(0.0001, at + 0.03);
    click.connect(band).connect(clickEnv).connect(panner);
    click.start(at, Math.random());
    click.stop(at + 0.04);
  }
  setTimeout(() => panner.disconnect(), 1000);
}

// Buffers, made once per audio context.

const buffers = new Map<(ac: AudioContext) => AudioBuffer, AudioBuffer>();

function cached(ac: AudioContext, make: (ac: AudioContext) => AudioBuffer): AudioBuffer {
  let b = buffers.get(make);
  if (!b || b.sampleRate !== ac.sampleRate) {
    b = make(ac);
    buffers.set(make, b);
  }
  return b;
}

/** A small seeded generator, so the sounds are the same every time. */
function random(seed: number): () => number {
  let s = seed >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    return (s >>> 0) / 4294967296;
  };
}

function buffer(ac: AudioContext, seconds: number, fill: (data: Float32Array, rate: number) => void): AudioBuffer {
  const b = ac.createBuffer(1, Math.round(ac.sampleRate * seconds), ac.sampleRate);
  const data = b.getChannelData(0);
  fill(data, ac.sampleRate);
  let peak = 0;
  for (let i = 0; i < data.length; i++) peak = Math.max(peak, Math.abs(data[i]));
  if (peak > 0) for (let i = 0; i < data.length; i++) data[i] /= peak;
  return b;
}

function whiteBuffer(ac: AudioContext): AudioBuffer {
  const r = random(1);
  return buffer(ac, 1, (d) => {
    for (let i = 0; i < d.length; i++) d[i] = r() * 2 - 1;
  });
}

/** Soft, low noise: wind and a distant street. */
function brownBuffer(ac: AudioContext): AudioBuffer {
  const r = random(2);
  return buffer(ac, 4, (d) => {
    let b = 0;
    for (let i = 0; i < d.length; i++) {
      b = (b + 0.02 * (r() * 2 - 1)) / 1.02;
      d[i] = b;
    }
    // Ease the loop's seam.
    const n = Math.min(2000, d.length >> 2);
    for (let i = 0; i < n; i++) d[d.length - n + i] = d[d.length - n + i] * (1 - i / n) + d[i] * (i / n);
  });
}

/** Two crickets, each chirping in short trills. */
function cricketBuffer(ac: AudioContext): AudioBuffer {
  const r = random(3);
  return buffer(ac, 3, (d, rate) => {
    for (const [freq, level, offset] of [
      [4600, 1, 0],
      [4250, 0.6, 0.21],
    ]) {
      for (let start = offset; start < 2.8; start += 0.42 + r() * 0.12) {
        for (let p = 0; p < 3; p++) {
          const s = Math.round((start + p * 0.032) * rate);
          const len = Math.round(0.014 * rate);
          for (let j = 0; j < len && s + j < d.length; j++) {
            const env = Math.sin((Math.PI * j) / len);
            d[s + j] += level * env * Math.sin((2 * Math.PI * freq * j) / rate);
          }
        }
      }
    }
  });
}

/** A hot pan: hiss with crackles. */
function sizzleBuffer(ac: AudioContext): AudioBuffer {
  const r = random(4);
  return buffer(ac, 2, (d, rate) => {
    let pop = 0;
    let popLevel = 0;
    for (let i = 0; i < d.length; i++) {
      if (pop <= 0 && r() < 60 / rate) {
        pop = Math.round((0.002 + r() * 0.006) * rate);
        popLevel = 0.3 + r() * 0.7;
      }
      const crackle = pop > 0 ? (r() * 2 - 1) * popLevel * (pop-- / (0.008 * rate)) : 0;
      d[i] = (r() * 2 - 1) * 0.18 + crackle;
    }
  });
}

/** A knife on a chopping board: short woody knocks in uneven runs. */
function chopBuffer(ac: AudioContext): AudioBuffer {
  const r = random(9);
  return buffer(ac, 3, (d, rate) => {
    let i = 0;
    while (i < d.length) {
      // A run of quick chops, then a pause to move the food along.
      const chops = 3 + Math.floor(r() * 6);
      for (let c = 0; c < chops && i < d.length; c++) {
        const len = Math.round(0.035 * rate);
        const level = 0.6 + r() * 0.4;
        const tone = 900 + r() * 500;
        for (let j = 0; j < len && i + j < d.length; j++) {
          const decay = Math.exp((-j / rate) * 90);
          d[i + j] = ((r() * 2 - 1) * 0.5 + Math.sin((2 * Math.PI * tone * j) / rate)) * level * decay;
        }
        i += Math.round((0.16 + r() * 0.08) * rate);
      }
      i += Math.round((0.4 + r() * 0.6) * rate);
    }
  });
}

/** Voices through a wall: noise shaped into syllables, phrases and pauses (the TV). */
function murmurBuffer(ac: AudioContext): AudioBuffer {
  const r = random(6);
  return buffer(ac, 5, (d, rate) => {
    let i = 0;
    while (i < d.length) {
      // A phrase of a few syllables, then a pause.
      const syllables = 3 + Math.floor(r() * 7);
      for (let s = 0; s < syllables && i < d.length; s++) {
        const len = Math.round((0.09 + r() * 0.14) * rate);
        const level = 0.5 + r() * 0.5;
        for (let j = 0; j < len && i < d.length; j++, i++) d[i] = (r() * 2 - 1) * level * Math.sin((Math.PI * j) / len);
      }
      i += Math.round((0.15 + r() * 0.5) * rate);
    }
  });
}

/** A little tune on a loop (a radio, a record). */
function musicBuffer(ac: AudioContext): AudioBuffer {
  const r = random(7);
  const scale = [0, 2, 4, 7, 9, 12, 14, 16];
  const root = 261.63;
  return buffer(ac, 4, (d, rate) => {
    const pluck = (at: number, freq: number, level: number, decay: number) => {
      const s = Math.round(at * rate);
      const len = Math.min(d.length - s, Math.round(decay * 4 * rate));
      for (let j = 0; j < len; j++) {
        const t = j / rate;
        d[s + j] += level * Math.exp(-t / decay) * (Math.sin(2 * Math.PI * freq * t) + 0.3 * Math.sin(4 * Math.PI * freq * t));
      }
    };
    for (let n = 0; n < 16; n++) {
      const note = scale[Math.floor(r() * scale.length)];
      if (r() < 0.85) pluck(n * 0.25, root * 2 ** (note / 12), 0.5, 0.18);
    }
    for (let n = 0; n < 4; n++) pluck(n, (root / 2) * 2 ** ([0, 9, 5, 7][n] / 12), 0.6, 0.5);
  });
}

/** A drum kit played: a rock beat (kick, snare on two and four, the hi-hat on the eighths) at 110 bpm, four bars with a crash at the top and a fill on the toms at the end. */
function drumsBuffer(ac: AudioContext): AudioBuffer {
  const r = random(10);
  const beat = 60 / 110;
  return buffer(ac, 16 * beat, (d, rate) => {
    // A hit's tail runs on past the loop's end into its start.
    const hit = (at: number, seconds: number, level: number, f: (t: number, noise: number) => number) => {
      const s = Math.round(at * rate);
      for (let j = 0, last = 0; j < seconds * rate; j++) {
        const noise = r() * 2 - 1;
        d[(s + j) % d.length] += level * (1 + 0.1 * (r() - 0.5)) * f(j / rate, noise - last);
        last = noise;
      }
    };
    // A pitched skin: a sine falling from `from` to `to` Hz as it dies away, with a little noise in its attack.
    const skin = (at: number, from: number, to: number, decay: number, level: number) => {
      let phase = 0;
      hit(at, decay * 5, level, (t, noise) => {
        phase += (2 * Math.PI * (to + (from - to) * Math.exp(-t / 0.03))) / rate;
        return Math.exp(-t / decay) * Math.sin(phase) + noise * 0.15 * Math.exp(-t / 0.004);
      });
    };
    const snare = (at: number, level: number) => {
      let phase = 0;
      hit(at, 0.4, level, (t, noise) => {
        phase += (2 * Math.PI * 190) / rate;
        return 0.5 * noise * Math.exp(-t / 0.08) + 0.6 * Math.sin(phase) * Math.exp(-t / 0.04);
      });
    };
    // Cymbals: high noise (each sample's change, so only the highs), short for the hi-hat, long for the crash.
    const cymbal = (at: number, decay: number, level: number) => hit(at, decay * 5, level, (t, noise) => noise * Math.exp(-t / decay));
    cymbal(0, 0.8, 0.35);
    for (let bar = 0; bar < 4; bar++) {
      const b = bar * 4 * beat;
      const fill = bar === 3;
      for (let e = 0; e < (fill ? 4 : 8); e++) cymbal(b + e * (beat / 2), 0.03, e % 2 ? 0.12 : 0.2);
      for (const k of fill ? [0] : [0, 2, 2.5]) skin(b + k * beat, 120, 48, 0.16, 0.9);
      for (const s of fill ? [1] : [1, 3]) snare(b + s * beat, 0.7);
    }
    // The fill: sixteenths down the toms.
    for (let n = 0; n < 8; n++) {
      const tom = n < 3 ? 210 : n < 6 ? 160 : 115;
      skin(14 * beat + n * (beat / 4), tom * 1.3, tom, 0.12, 0.5 + 0.1 * (n % 2));
    }
  });
}
