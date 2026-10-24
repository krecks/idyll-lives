// The garden birds' engine-neutral code (`render/shared/birds.ts`) driven through a recording `BirdView`, the way
// an engine draws them: a lot with a house, a street and gardens round it, through a day and a night (game speeds,
// pauses, long frames, reduce motion, the roof shown and hidden, residents walking by, the camera diving close,
// songs, flights sent off, the habitat built again, a smaller preset, a model file arriving late, `clear`), with
// a fixed Math.random, gives the same birds, draws and sounds heard per second as recorded (`bun test`;
// after a change meant to change how the birds behave, `bun test --update-snapshots`). The record was made when
// the code moved out of the Babylon renderer (Stage 9), where a side-by-side run against the Babylon version
// before the move showed no difference in any frame (bench-results/stage9c/equal.ts).
//
// Also: the matrices a bird's parts are drawn with, against Babylon's own maths as the Babylon renderer wrote
// them, number for number (fixtures recorded from Babylon 9.30 for the poses recorded with them,
// `fixtures/babylon/birds.json`, made by `fixtures/record-babylon.ts`, which needs a Babylon reference checkout
// to run again); and no bird ever flies through a house.
import { describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';
import { translation } from '../src/render/shared/math';
import { fixture, numbers as recorded, type Numbers } from './fixtures/babylon';

const WEB = new URL('../', import.meta.url).pathname;

/** A fixed sequence of numbers in 0..1 (the same every run). */
function numbers(seed: number): () => number {
  let s = seed >>> 0;
  return () => (s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32;
}

function hash(a: ArrayLike<number>, from = 0, to = a.length): string {
  const f = new Float32Array(1);
  const u = new Uint32Array(f.buffer);
  let h = 2166136261;
  for (let i = from; i < to; i++) {
    f[0] = a[i];
    h = Math.imul(h ^ u[0], 16777619);
  }
  return (h >>> 0).toString(16);
}

describe('garden birds', async () => {
  // --- the page's world in bun: the assets from web/public; Math.random fixed (the birds' code takes it as it loads) ---
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  const realRandom = Math.random;
  let random = realRandom;
  Math.random = () => random();
  const { Birds, BIRD_PARTS, birdLook, noBirdLook, roofRidge } = await import('../src/render/shared/birds');
  const { AssetRegistry } = await import('../src/assets/registry');
  type BirdLook = import('../src/render/shared/birds').BirdLook;
  type BirdView = import('../src/render/shared/birds').BirdView<Look>;
  type BirdEntry = import('../src/render/shared/birds').BirdEntry;
  type PlantPlacement = import('../src/render/shared/nature').PlantPlacement;
  type World = import('../src/core/protocol').WorldStructure;
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

  // --- the recording view: each species' look with its parts' counts and uploads ---
  interface Look extends BirdLook {
    species: string;
    shown: boolean;
    uploads: number;
  }
  const log: string[] = [];
  let warblerComes = false;
  let wind = 0;
  const built = new Map<string, Look>();
  const view: BirdView = {
    look: async (id, key) => {
      const known = built.get(key);
      if (known) return known;
      const entry = assets.get(key, 'model') as BirdEntry;
      const failed = id === 'warbler' && !warblerComes;
      const rests = BIRD_PARTS.map((_, k) => translation(0.013 * k, 0.081 + 0.011 * k, k === 1 ? 0.043 : -0.031 * k, new Float32Array(16)));
      const look: Look = { ...(failed ? noBirdLook([entry.url]) : birdLook(entry, rests)), species: id, shown: false, uploads: 0 };
      built.set(key, look);
      return look;
    },
    invalidate: (url) => {
      let any = false;
      for (const [key, look] of built) {
        if (!look.failed?.includes(url)) continue;
        built.delete(key);
        any = true;
      }
      return any;
    },
    disposeStale: () => {},
    show: (visible) => {
      for (const look of built.values()) look.shown = visible && look.count > 0;
    },
    setBuffers: (look, visible) => {
      look.shown = visible && look.count > 0;
      log.push(`${look.species} buffers ${look.count}/${look.capacity}`);
    },
    updated: (look) => look.uploads++,
    sway: (x, z) => Math.sin(wind + x * 0.21 + z * 0.17) * 0.0022,
    dispose: () => built.clear(),
  };

  // --- the lot (as bench-results/stage9c/equal.ts) ---
  const W = 56;
  const rooms = new Array(W * W).fill(0);
  for (let z = 22; z < 30; z++) for (let x = 20; x < 30; x++) rooms[z * W + x] = 1;
  const world = {
    version: 1, lotVersion: 1, mode: 'live', width: W, depth: W, storeys: 2,
    objects: [{ id: 1, def: 'chair', x: 16, z: 18, w: 1, d: 1, rot: 0 }],
    sims: [], households: [], exits: [], walls: [], openings: [],
    plots: [
      { x: 16, z: 16, w: 20, d: 20, house: [20, 22, 30, 30] },
      { x: 38, z: 16, w: 16, d: 20, house: [42, 22, 50, 30], roof: [0, 0], storeys: 1 },
    ],
    rooms, floors: [[18, 20, 1]],
    meta: { streets: [{ x: 0, z: 38, w: W, d: 4 }], paths: [{ x: 24, z: 30, w: 2, d: 8 }] },
    meshes: { walls: {}, wallsLow: {}, floors: {} },
  } as unknown as World;
  const lot = { x: 16, z: 16, w: 20, d: 20 };
  const at = (key: string, x: number, z: number, s = 1, yaw = 0, sx = s, sz = s): PlantPlacement => ({ key, x, y: 0, z, sx, sy: s, sz, yaw });
  const garden = [at('model.tree', 18, 18, 1.1, 0.4), at('model.tree.maple', 33, 19, 0.9, 2), at('model.tree.birch', 18, 33, 1, 1.1), at('model.bush', 31, 31, 1, 0.3), at('model.garden.hedge', 21, 34, 1, 0, 2.5, 0.8), at('model.bush.flowering', 34, 27, 1.2, 1)];
  const outdoor = [at('model.garden.rosebush', 17, 25, 1, 0.2), at('model.tree.fruit', 26, 35, 1, 0.6), at('model.garden.lemonTree', 35, 35, 1)];
  const street = [at('model.tree.blossom', 8, 37, 1, 0.1), at('model.tree.magnolia', 46, 37, 1.1, 3), at('model.hedge', 40, 34, 1, 1.57, 3, 0.7), at('model.pine', 52, 12, 1.2), at('model.spruce', 6, 10, 1.3)];
  const woods = [at('model.tree.willow', 3, 50, 1.4), at('model.cypress', 50, 52, 1), at('model.tree.japaneseMaple', 12, 4, 1), at('model.bush.small', 45, 5, 1)];
  // A gabled roof's ridge along x at z 26, 7.35 m up.
  const roof = roofRidge((fn) => {
    for (let x = 20; x <= 30; x += 0.5) for (const [y, z] of [[5.8, 22], [7.35, 26], [5.8, 30]]) fn(x, y, z);
  });
  const houseTop = (plot: { storeys?: number }) => (plot.storeys ?? 2) * 2.8 + 2.6;

  test('the roof ridge: a perch every half metre along the top', () => {
    expect(roof.top).toBe(7.35);
    expect(roof.ridge.length / 3).toBe(19);
    expect(roof.ridge.slice(0, 3)).toEqual([20.5, 7.35, 26]);
    expect(roofRidge(() => {})).toEqual({ ridge: [], top: null });
  });

  test('a day and a night of birds, as recorded', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    random = numbers(9);
    try {
      const birds = new Birds(view, assets, true);
      const inner = birds as unknown as { inUse: Look[] };
      const camera = { x: 26, y: 12, z: 8 };
      const heads = new Float32Array(12);
      const visible = new Uint8Array(4);
      const frame = { dt: 0, speed: 1, hour: 0, camera, residents: heads, residentCount: 4, visible, calm: false, roof: true };
      const habitat = (plants: readonly (readonly PlantPlacement[])[], hour: number) => ({ world, view: lot, plants, ridge: roof.ridge, roofTop: roof.top, houseTop, hour });
      birds.setQuality({ count: 40, range: 50 });
      await birds.setHabitat(habitat([garden, outdoor, street, woods], 4.5));
      const FRAMES = 9000;
      const record: string[] = [];
      let second: string[] = [];
      let forager: { x: number; z: number } | undefined;
      for (let f = 0; f < FRAMES; f++) {
        now += 16.7;
        wind += 1 / 60;
        const u = f / FRAMES;
        Object.assign(frame, {
          hour: (4.6 + u * 27) % 24,
          speed: [1, 1, 2, 3, 1, 0, 4, 5, 1, 1][Math.floor(f / 400) % 10],
          dt: f % 397 === 0 ? 0.4 : f % 213 === 0 ? 0 : 1 / 60,
          calm: u > 0.42 && u < 0.47,
          roof: Math.floor(f / 700) % 3 !== 1,
        });
        const t = f / 600;
        if (f % 300 === 0) forager = birds.debug().birds.find((b) => b.state === 'ground' || (b.state === 'perch' && b.y < 2));
        const near = forager && f % 300 > 120 ? forager : { x: 2, z: 2 };
        heads.set([26 + Math.cos(t) * 9, 1.6, 26 + Math.sin(t) * 9, 25, 1.6, 26, 18, 4.5, 18, near.x + 3 - ((f % 300) - 120) / 60, 1.7, near.z + 0.5]);
        visible.set([f % 4000 < 3000 ? 1 : 0, 1, 1, 1]);
        const heard = birds.heard();
        const dive = f % 900 > 850 && heard.length ? heard[(f >> 4) % heard.length] : undefined;
        if (dive) [camera.x, camera.y, camera.z] = [dive.x + 0.6, dive.y + 0.4, dive.z];
        else [camera.x, camera.y, camera.z] = [26 + Math.cos(t * 0.3) * 22, 14, 26 + Math.sin(t * 0.3) * 22];
        birds.setVisible(!(u > 0.3 && u < 0.32));
        if (f % 211 === 0 && heard.length) birds.sings(heard[(f / 211) % heard.length | 0].id, 2 + (f % 5));
        if (f % 487 === 0) {
          const list = birds.debug().birds;
          if (list.length) birds.debugFly(list[(f / 487) % list.length | 0].id, (['plant', 'ground', 'roof', 'home'] as const)[(f / 487) % 4 | 0]);
        }
        if (f === 1350) await birds.setHabitat(habitat([garden.slice(1), outdoor, street.slice(), woods], frame.hour));
        if (f === 1800) await birds.setQuality({ count: 6, range: 18 });
        if (f === 2250) await birds.setQuality({ count: 40, range: 50 });
        if (f === 3150) {
          warblerComes = true;
          birds.fileArrived((assets.get('model.bird.warbler', 'model') as BirdEntry).url);
          await new Promise((r) => setTimeout(r, 0));
        }
        if (f === 5400) {
          birds.clear();
          await birds.setHabitat(habitat([garden, outdoor, street, woods], frame.hour));
        }
        const moved = birds.update(frame);
        // (Not the matrices' or positions' numbers: the birds' sin, cos and exp differ in their last bits
        // between platforms, macOS on arm64 against Linux on x86-64.)
        const drawn = inner.inUse.map((l) => `${l.species} ${l.count} ${l.shown ? 'shown' : 'hidden'} ${l.uploads}`);
        const sung = birds.heard().map((b) => `${b.id} ${b.song}${b.perched ? ' perched' : ''}`);
        second.push(`${moved}`, ...drawn, ...log.splice(0), sung.join(', '));
        for (const look of inner.inUse) for (const b of look.buffers) for (let i = 0; i < look.count * 16; i++) if (!Number.isFinite(b[i])) throw new Error(`frame ${f}: ${look.species} drawn with ${b[i]}`);
        if (f % 600 === 599) {
          const d = birds.debug();
          const states = d.birds.reduce<Record<string, number>>((a, b) => ((a[b.state] = (a[b.state] ?? 0) + 1), a), {});
          record.push(`${(f + 1) / 60} s: ${d.birds.length} birds ${JSON.stringify(states)} ${hash(new TextEncoder().encode(second.join('\n')))}`);
          second = [];
        }
      }
      const { stats } = birds.debug();
      record.push(JSON.stringify(stats));
      // Through a whole day: flights and hops, visits to the lawn and the roof, scares; never inside a house.
      expect(stats.inside).toBe(0);
      for (const k of ['flights', 'hops', 'lawn', 'roof', 'scaredBy', 'scaredCamera'] as const) expect(stats[k]).toBeGreaterThan(0);
      expect(record).toMatchSnapshot();
      birds.dispose();
    } finally {
      performance.now = realNow;
      random = realRandom;
      Math.random = realRandom;
      globalThis.fetch = realFetch;
    }
  });

  test("a bird's parts drawn as the Babylon renderer computed them, number for number", async () => {
    const B = fixture<{ fields: string[]; cases: number; inputs: Numbers; parts: Numbers }>('birds');
    const inputs = recorded(B.inputs);
    const want = recorded(B.parts);
    random = numbers(3);
    const birds = new Birds(view, assets);
    const restore: (() => void)[] = [];
    try {
      birds.setQuality({ count: 40, range: 50 });
      await birds.setHabitat({ world, view: lot, plants: [garden, outdoor, street, woods], ridge: roof.ridge, roofTop: roof.top, houseTop, hour: 9 });
      const inner = birds as unknown as { birds: Record<string, number | Float32Array | BirdLook>[]; write(b: unknown): boolean };
      for (const look of new Set(inner.birds.map((b) => b.look as BirdLook))) {
        const { centre, pivots } = look;
        restore.push(() => Object.assign(look, { centre, pivots }));
      }
      const bits = (a: ArrayLike<number>) => Array.from(Float32Array.from(a));
      let checked = 0;
      for (let n = 0; n < B.cases; n++) {
        const b = inner.birds[n % inner.birds.length];
        const look = b.look as BirdLook;
        // The recorded pose, the bird's size, and its look's middle and pivots.
        const v = Object.fromEntries(B.fields.map((f, i) => [f, inputs[n * B.fields.length + i]]));
        const { centre, ...pose } = Object.fromEntries(Object.entries(v).filter(([f]) => !f.startsWith('pivot')));
        Object.assign(b, pose);
        look.centre = centre;
        look.pivots = [look.pivots[0], ...[1, 2, 3, 4].map((k) => [v[`pivot${k}x`], v[`pivot${k}y`], v[`pivot${k}z`]] as const)];
        (b.drawn as Float32Array).fill(NaN);
        inner.write(b);
        const o = (b.slot as number) * 16;
        for (let part = 0; part < 5; part++) expect(bits(look.buffers[part].subarray(o, o + 16))).toEqual(Array.from(want.slice((n * 5 + part) * 16, (n * 5 + part + 1) * 16)));
        checked++;
      }
      expect(checked).toBe(300);
    } finally {
      for (const undo of restore) undo();
      birds.dispose();
      random = realRandom;
    }
  });
});
