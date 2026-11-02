// What the residents cost the GPU and the shadow map (`residents.ts` `faceDetail`, `upload`): a resident not drawn
// (hidden, away, out of view) is in no part's rows, the parts drawn whatever the distance and the shadow-only copies
// included, so nothing skins them to nothing; and only residents posed inside the sun's shadow square draw its map
// again (`posed`). And the pose texture's three texels a bone (`layout.ts` `BONE_TEXELS`) are the skinning matrices'
// rows, number for number. `bun test`.
//
// (What this caught when written: the hair, the eyes and the shadow-only copies kept every resident built, drawn or
// not, and one hidden counted as far off, so its low-detail body was drawn too; and anyone posed within the preset's
// resident radius, 140 m, drew the sun's map again, though it covers 30–70 m.)
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

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
      [out[0], out[1], out[2], out[3], out[4]] = [cam[0], cam[1], cam[2], 720, 0.8];
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
    ticksPerSecond: 20,
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
      characters.setShadowSquare({ x: 3, z: 0, half: 30 });
      expect(run(30).shadowPosed).toBeGreaterThan(20);
      // The square 100 m off: they still pose (a mirror would show them), the map isn't drawn again for them.
      characters.setShadowSquare({ x: 100, z: 0, half: 30 });
      const off = run(30);
      expect(off.posed).toBeGreaterThan(20);
      expect(off.shadowPosed).toBe(0);
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
