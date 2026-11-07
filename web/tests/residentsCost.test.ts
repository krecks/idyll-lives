// What the residents cost the GPU and the shadow map (`residents.ts` `faceDetail`, `upload`): a resident not drawn
// (hidden, away, out of view) is in no part's rows, the parts drawn whatever the distance and the shadow-only copies
// included, so nothing skins them to nothing; and only residents posed inside the sun's shadow square draw its map
// again (`posed`). And the pose texture's three texels a bone (`layout.ts` `BONE_TEXELS`) are the skinning matrices'
// rows, number for number; and residents far off are posed in full only every few frames, carried along where they
// are between. `bun test`.
//
// (What this caught when written: the hair, the eyes and the shadow-only copies kept every resident built, drawn or
// not, and one hidden counted as far off, so its low-detail body was drawn too; and anyone posed within the preset's
// resident radius, 140 m, drew the sun's map again, though it covers 30–70 m.)
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';
import { TICK_RATES } from './snapshotLayout';

const WEB = new URL('../', import.meta.url).pathname;
/** A shadow test as `SunShadows.reaches` gives it: a square round (x, z), with a long shadow's 8 m round it. */
const square = (x: number, z: number, half: number) => (px: number, pz: number) => Math.abs(px - x) <= half + 8 && Math.abs(pz - z) <= half + 8;

describe('residents not drawn', async () => {
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { Characters, MAX_CHARACTERS } = await import('../src/render/shared/characters/residents');
  const { DEFAULT_ORBIT, LENS, RADIUS_LIMITS } = await import('../src/render/shared/lens');
  const { TemplateCache } = await import('../src/render/shared/templateCache');
  const { Pose, simFields } = await import('../src/core/snapshot');
  const pose = await import('../src/render/shared/characters/pose');
  type ResidentProp = import('../src/render/shared/characters/view').ResidentProp;
  type View = import('../src/render/shared/characters/view').ResidentView<Part, object, ResidentProp>;
  type SnapshotLayout = import('../src/core/snapshot').SnapshotLayout;
  type FrameState = import('../src/core/bridge').FrameState;
  type SimInfo = import('../src/core/protocol').SimInfo;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

  interface Part {
    name: string;
    shadow: boolean;
    rows: Float32Array;
    count: number;
  }
  const parts: Part[] = [];
  /** The camera (x, y, z), looking on: far off, residents are drawn in low detail. */
  const cam = [0, 1.6, 6];
  /** Its drawing's height (px) and vertical field of view (rad). */
  const lens = { height: 720, fov: 0.8 };
  const view: View = {
    init: () => {},
    uploadPoses: () => {},
    uploadPoseRows: () => {},
    createPart(body, part, shadow) {
      const p = { name: `${body.name}:${part}`, shadow, rows: new Float32Array(0), count: 0 };
      parts.push(p);
      return p;
    },
    partEnabled: () => {},
    setPartRows(p, rows) {
      p.rows = rows;
      p.count = rows.length / 4;
    },
    partRows(p, rows, count) {
      p.rows = rows;
      p.count = count;
    },
    quiltMaterial: () => {},
    createQuilt: () => ({}),
    quiltRows: () => {},
    quiltEnabled: () => {},
    createRing: () => {},
    ringMoved: () => {},
    props: new TemplateCache<ResidentProp | null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
    async buildProp(key) {
      return { key, matrices: new Float32Array(MAX_CHARACTERS * 16), top: [0, 0, 0] as const };
    },
    showProp: () => {},
    onModelFile: () => () => {},
    fileColour: async () => undefined,
    camera(out) {
      [out[0], out[1], out[2], out[3], out[4]] = [cam[0], cam[1], cam[2], lens.height, lens.fov];
      return true;
    },
    onDispose: () => {},
  };

  /** Six residents along x, a metre and a half apart. */
  const N = 6;
  const sims: SimInfo[] = Array.from({ length: N }, (_, i) => ({
    id: i,
    name: `sim${i}`,
    household: 0,
    gender: i % 2 ? 'female' : 'male',
    attractedTo: [],
    appearance: { body: '#5F86C9', skin: '#C08A64', hair: '#2B2320', hairStyle: 'short', height: 1, build: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers' },
    traits: [],
    perks: [],
  }));
  const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'held'] as const;
  const layout: SnapshotLayout = {
    capacity: 6 + FIELDS.length * MAX_CHARACTERS,
    headerLen: 6,
    simStride: FIELDS.length,
    maxSims: MAX_CHARACTERS,
    ...TICK_RATES,
    header: { tick: 0, day: 1, minute: 2, speed: 3, simCount: 4, structureVersion: 5 },
    sim: simFields(FIELDS),
    actions: [...content.animations],
    items: content.items.map((it) => it.id),
  };
  const k = layout.sim as Record<(typeof FIELDS)[number], number>;
  function write(snap: Float32Array, away: number): void {
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = N;
    for (let i = 0; i < N; i++) {
      const o = layout.headerLen + i * layout.simStride;
      snap[o + k.id] = i;
      snap[o + k.x] = i * 1.5;
      snap[o + k.z] = 0;
      snap[o + k.pose] = Pose.Stand;
      snap[o + k.partner] = -1;
      snap[o + k.emotion] = 1;
      snap[o + k.object] = -1;
      snap[o + k.action] = -1;
      snap[o + k.mood] = 0.5;
      snap[o + k.away] = i === away ? 1 : 0;
    }
  }
  /** The rows every part draws now, by resident, shadow-only copies apart. */
  const drawnRows = (shadow: boolean) => {
    const ids = new Set<number>();
    for (const p of parts) if (p.shadow === shadow) for (let j = 0; j < p.count; j++) ids.add(p.rows[j * 4]);
    return [...ids].sort((a, b) => a - b);
  };

  test('are in no part, nor in its shadow copy, near or far', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: true });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [] });
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      // Resident 1 away, resident 3 out of view; the camera near, then far off (low detail).
      write(prev, 1);
      write(curr, 1);
      const inView = (x: number) => Math.abs(x - 4.5) > 0.1;
      for (const [far, frames] of [[false, 10], [true, 10], [false, 10]] as const) {
        cam[2] = far ? 120 : 6;
        for (let f = 0; f < frames; f++) {
          now += 1000 / 60;
          frame.now = now;
          characters.update(frame, inView, shadows);
        }
        expect(drawnRows(false)).toEqual([0, 2, 4, 5]);
        expect(drawnRows(true)).toEqual([0, 2, 4, 5]);
      }
      // Back in view and home: drawn again, every part.
      write(prev, -1);
      write(curr, -1);
      for (let f = 0; f < 3; f++) {
        now += 1000 / 60;
        frame.now = now;
        characters.update(frame, () => true, shadows);
      }
      expect(drawnRows(false)).toEqual([0, 1, 2, 3, 4, 5]);
      expect(drawnRows(true)).toEqual([0, 1, 2, 3, 4, 5]);
    } finally {
      performance.now = realNow;
    }
  }, 30_000);

  test('the default camera draws the full body at any size of drawing, and poses them in turn', async () => {
    // The game's lens and the pose a game opens in (`shared/lens.ts`), looking at the residents from its distance, at
    // the drawing's heights a window, the device's pixel ratio and the resolution setting make (720 to 2160 px).
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    const first = parts.length;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: true });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [] });
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      write(prev, -1);
      write(curr, -1);
      const rigs = (characters as unknown as { sims: { lag: number }[] }).sims;
      lens.fov = LENS.fov;
      // The orbit's eye about the residents' middle (`shared/orbit.ts`: target + radius (cos α sin β, cos β, sin α sin β)).
      const mid = ((N - 1) * 1.5) / 2;
      const run = (radius: number, height: number, frames: number) => {
        const { alpha, beta } = DEFAULT_ORBIT;
        [cam[0], cam[1], cam[2]] = [mid + radius * Math.cos(alpha) * Math.sin(beta), radius * Math.cos(beta), radius * Math.sin(alpha) * Math.sin(beta)];
        lens.height = height;
        let lagging = 0;
        for (let f = 0; f < frames; f++) {
          now += 1000 / 60;
          frame.now = now;
          characters.update(frame, () => true, shadows);
          for (let i = 0; i < N; i++) if (rigs[i].lag > 0) lagging++;
        }
        const drawn = (name: string) => parts.slice(first).some((p) => p.name.endsWith(`:${name}`) && !p.shadow && p.count > 0);
        return { body: drawn('body'), lod: drawn('lod.body'), lagging: lagging / (frames * N) };
      };
      for (const height of [720, 1080, 1440, 2160]) {
        const at = run(DEFAULT_ORBIT.radius, height, 6);
        expect([height, at.body, at.lod]).toEqual([height, true, false]);
        // Zoomed all the way out (the far town: the low-detail body), then back: the full body again, whichever way
        // they came to the default view.
        expect(run(RADIUS_LIMITS[1], height, 3)).toMatchObject({ body: false, lod: true });
        expect(run(DEFAULT_ORBIT.radius, height, 3)).toMatchObject({ body: true, lod: false });
        // And the closest camera.
        expect(run(RADIUS_LIMITS[0], height, 3)).toMatchObject({ body: true, lod: false });
      }
      // In the default view, on the full body, still posed in full only every few frames (the pose's distance is not
      // the mesh's); close up, every frame (once the pose carried along from out there has caught up).
      expect(run(DEFAULT_ORBIT.radius, 1080, 60).lagging).toBeGreaterThan(0.3);
      run(RADIUS_LIMITS[0], 1080, 5);
      expect(run(RADIUS_LIMITS[0], 1080, 60).lagging).toBe(0);
    } finally {
      performance.now = realNow;
      [lens.height, lens.fov] = [720, 0.8];
      [cam[0], cam[1], cam[2]] = [0, 1.6, 6];
    }
  }, 30_000);
  test('the body or its low-detail copy is drawn by the distance, and not both', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    // (The parts of this Sim set only: `parts` holds the other tests' too, still with their counts.)
    const first = parts.length;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: true });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [] });
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      write(prev, -1);
      write(curr, -1);
      // This view's camera: 720 px tall at 0.8 rad, so 42.6 px per 5 cm at 1 m.
      const run = (dist: number, frames: number) => {
        cam[2] = dist;
        for (let f = 0; f < frames; f++) {
          now += 1000 / 60;
          frame.now = now;
          characters.update(frame, () => true, shadows);
        }
        const drawn = (name: string) => parts.slice(first).some((p) => p.name.endsWith(`:${name}`) && !p.shadow && p.count > 0);
        return { body: drawn('body'), lod: drawn('lod.body') };
      };
      // Close (about 4 px per 5 cm): the full body; then far off (1 px): its low-detail copy.
      expect(run(10, 3)).toEqual({ body: true, lod: false });
      expect(run(40, 3)).toEqual({ body: false, lod: true });
      expect(run(10, 3)).toEqual({ body: true, lod: false });
    } finally {
      performance.now = realNow;
    }
  }, 30_000);
  test('only those posed in the shadow square draw the sun\'s map again', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: true });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [] });
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      write(prev, -1);
      write(curr, -1);
      const run = (frames: number) => {
        let [posed, shadowPosed] = [0, 0];
        for (let f = 0; f < frames; f++) {
          now += 1000 / 60;
          frame.now = now;
          characters.update(frame, () => true, shadows);
          posed += characters.posed ? 1 : 0;
          shadowPosed += characters.shadowPosed ? 1 : 0;
        }
        return { posed, shadowPosed };
      };
      // Without a square (it covers everything), and with one round them: as they breathe and blink.
      run(10);
      expect(run(30).shadowPosed).toBeGreaterThan(20);
      characters.setShadowTest(square(3, 0, 30));
      expect(run(30).shadowPosed).toBeGreaterThan(20);
      // The square 100 m off: they still pose (a mirror would show them), the map isn't drawn again for them.
      characters.setShadowTest(square(100, 0, 30));
      const off = run(30);
      expect(off.posed).toBeGreaterThan(20);
      expect(off.shadowPosed).toBe(0);
    } finally {
      performance.now = realNow;
    }
  }, 30_000);

  test('far off, posed in full every few frames, and carried along between', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: true });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [] });
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      const rigs = (characters as unknown as { sims: { lag: number }[] }).sims;
      // Walking along x at 2 m/s, a snapshot every third frame, the camera 120 m off (all of them far).
      cam[2] = 120;
      let t = 0;
      const walk = (snap: Float32Array) => {
        write(snap, -1);
        for (let i = 0; i < N; i++) {
          const o = layout.headerLen + i * layout.simStride;
          snap[o + k.x] = i * 1.5 + t * 2;
          snap[o + k.moving] = 1;
          snap[o + k.yaw] = Math.PI / 2;
        }
      };
      walk(prev);
      walk(curr);
      let lagging = 0;
      let worst = 0;
      for (let f = 0; f < 240; f++) {
        if (f % 3 === 0) {
          prev.set(curr);
          t += 0.05;
          walk(curr);
        }
        frame.alpha = (f % 3) / 3;
        now += 1000 / 60;
        frame.now = now;
        characters.update(frame, () => true, shadows);
        if (f < 30) continue;
        for (let i = 0; i < N; i++) {
          const o = layout.headerLen + i * layout.simStride;
          const x = prev[o + k.x] + (curr[o + k.x] - prev[o + k.x]) * frame.alpha;
          // Where the snapshot has them, every frame (their contact shadow under them), posed in full or not.
          worst = Math.max(worst, Math.abs(shadows[i * 16 + 12] - x));
          if (rigs[i].lag > 0) lagging++;
          expect(rigs[i].lag).toBeLessThan(4.5 / 60);
        }
      }
      // Most frames, most of them not posed in full; always where they are.
      expect(lagging / (210 * N)).toBeGreaterThan(0.5);
      expect(worst).toBeLessThan(1e-3);
    } finally {
      performance.now = realNow;
    }
  }, 30_000);

  test("the pose texture's rows are the skinning matrices'", async () => {
    const characters = new Characters(view, assets, content, { castsShadows: true });
    expect(await characters.init()).toBe(true);
    const set = (characters as unknown as { set: import('../src/render/shared/characters/data').CharacterSet }).set;
    const NB = set.bones.length;
    const q = new Float32Array(NB * 4);
    const p = new Float32Array(3);
    pose.sampleClip(set, set.clips.get('walk')!, 0.3, q, p);
    const add = pose.makeAdditive(NB);
    pose.resetAdditive(add);
    const head = new Float32Array(3);
    for (const body of set.bodies.values()) {
      const place = { x: 3, y: 0.2, z: -4, qx: 0, qy: Math.sin(0.4), qz: 0, qw: Math.cos(0.4), scale: 1.05, head: 1.1, build: 1.15 };
      const columns = new Float32Array(NB * 16);
      const rows = new Float32Array(NB * 12 + 8);
      pose.skin(set, body, q, p, add, place, new pose.PoseScratch(NB), columns, 0, head);
      pose.skin(set, body, q, p, add, place, new pose.PoseScratch(NB), rows, 8, head, true);
      const m = new Float32Array(16);
      for (let b = 0; b < NB; b++) expect(Array.from(pose.boneMatrix(rows, 8, b, m))).toEqual(Array.from(columns.subarray(b * 16, b * 16 + 16)));
    }
  });
});
