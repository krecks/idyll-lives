// Residents on the land (`residents.ts` `setGround`): out on a big world's hills between its hamlets a resident walks
// on the ground there, not at the flat 0 of what's built (sunk into a hill, floating over a hollow), and with no land
// given (or flat land) nothing changes. `bun test`.
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';
import { TICK_RATES, TICKS_PER_SECOND } from './snapshotLayout';

const WEB = new URL('../', import.meta.url).pathname;

describe('residents on the land', async () => {
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
  type View = import('../src/render/shared/characters/view').ResidentView<object, object, import('../src/render/shared/characters/view').ResidentProp>;
  type SnapshotLayout = import('../src/core/snapshot').SnapshotLayout;
  type FrameState = import('../src/core/bridge').FrameState;
  type SimInfo = import('../src/core/protocol').SimInfo;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

  const view: View = {
    init: () => {},
    setMorphs: () => {},
    uploadPoses: () => {},
    uploadPoseRows: () => {},
    createPart: () => ({}),
    partEnabled: () => {},
    setPartRows: () => {},
    partRows: () => {},
    quiltMaterial: () => {},
    createQuilt: () => ({}),
    quiltRows: () => {},
    quiltEnabled: () => {},
    createRing: () => {},
    ringMoved: () => {},
    props: new TemplateCache<import('../src/render/shared/characters/view').ResidentProp | null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
    async buildProp(key) {
      return { key, matrices: new Float32Array(MAX_CHARACTERS * 16), top: [0, 0, 0] as const };
    },
    showProp: () => {},
    onModelFile: () => () => {},
    fileColour: async () => undefined,
    camera: () => false,
    onDispose: () => {},
  };

  const sims: SimInfo[] = [
    {
      id: 0,
      name: 'walker',
      household: 0,
      gender: 'female',
      attractedTo: [],
      appearance: { body: '#5F86C9', skin: '#C08A64', hair: '#2B2320', hairStyle: 'short', height: 1, build: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers' },
      traits: [],
      perks: [],
    },
  ];
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

  /** Walking speed (m/s) along +x from x = 0, and how long (s). */
  const SPEED = 1.3;
  const SECONDS = 4;

  function write(snap: Float32Array, n: number): void {
    const o = layout.headerLen;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = 1;
    snap[o + k.id] = 0;
    snap[o + k.x] = (SPEED * n) / TICKS_PER_SECOND;
    snap[o + k.z] = 10;
    snap[o + k.yaw] = Math.PI / 2;
    snap[o + k.pose] = Pose.Stand;
    snap[o + k.moving] = 1;
    snap[o + k.partner] = -1;
    snap[o + k.emotion] = 1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood] = 0.5;
  }

  /** The head's height over the land under it, frame by frame, walking up `land` (none: flat). */
  async function walk(land: ((x: number, z: number) => number) | null): Promise<{ over: number[]; floors: number[] }> {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: false });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [] });
      if (land) characters.setGround(land);
      const rig = (characters as unknown as { sims: { floorY: number }[] }).sims[0];
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      write(prev, 0);
      write(curr, 0);
      const head = { x: 0, y: 0, z: 0 };
      const over: number[] = [];
      const floors: number[] = [];
      let tick = 0;
      const FPS = 60;
      for (let n = 0; n < SECONDS * FPS; n++) {
        const t = n / FPS;
        const due = Math.floor(t * TICKS_PER_SECOND);
        while (tick < due) {
          tick++;
          prev.set(curr);
          write(curr, tick);
        }
        frame.alpha = Math.min(1, t * TICKS_PER_SECOND - tick);
        now = t * 1000;
        frame.now = now;
        characters.update(frame, () => true, shadows);
        // (Past the first half second: walking, not setting off.)
        if (t > 0.5 && characters.head(0, head)) {
          over.push(head.y - (land ? land(head.x, head.z) : 0));
          floors.push(rig.floorY);
        }
      }
      return { over, floors };
    } finally {
      performance.now = realNow;
    }
  }

  test('walks on a hill, as tall over it as on flat ground', async () => {
    // A slope of 1 in 5, rising along the way they walk.
    const slope = (x: number) => 2 + 0.2 * x;
    const flat = await walk(null);
    const hill = await walk((x) => slope(x));
    expect(flat.over.length).toBeGreaterThan(100);
    expect(hill.over.length).toBe(flat.over.length);
    const mean = (a: number[]) => a.reduce((s, v) => s + v, 0) / a.length;
    // Flat: the floor at 0. On the hill: the floor rises with it, over 2 m.
    expect(Math.max(...flat.floors.map(Math.abs))).toBe(0);
    expect(Math.min(...hill.floors)).toBeGreaterThan(2);
    // The head over the land under it: as on flat ground (within the head's lead over the feet on a 1-in-5 slope).
    expect(Math.abs(mean(hill.over) - mean(flat.over))).toBeLessThan(0.06);
    // Never sunk into the hill: the head always well above the ground under it.
    expect(Math.min(...hill.over)).toBeGreaterThan(1.2);
  }, 30_000);

  test('flat land changes nothing', async () => {
    const none = await walk(null);
    const flat = await walk(() => 0);
    expect(flat.over).toEqual(none.over);
  }, 30_000);
});
