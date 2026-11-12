// A bike left out by a place (sim-core `ride.rs`, `Sim::bike_parked`; the snapshot's `ride` -2 less its stable id and
// `parkX`/`parkZ`/`parkYaw`): got off there, it slides from under the rider to where it's left, not home; it's in
// `parked` there (for `ParkedRides` to draw) once they're off it, and in `ridden` (not drawn at home) all along; got on
// again, it slides from there to under them, and once they're home it's back at its place. And `placements`: what
// `ParkedRides` draws where. `bun test`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { TICK_RATES, TICKS_PER_SECOND } from './snapshotLayout';

const WEB = new URL('../', import.meta.url).pathname;

describe('a bike left out by a place', async () => {
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
  const { MOUNTS, MOUNT_SECONDS } = await import('../src/render/shared/characters/vehicles');
  const { placements } = await import('../src/render/shared/parked');
  type Prop = import('../src/render/shared/characters/view').ResidentProp;
  type View = import('../src/render/shared/characters/view').ResidentView<object, object, Prop>;
  type SnapshotLayout = import('../src/core/snapshot').SnapshotLayout;
  type FrameState = import('../src/core/bridge').FrameState;
  type SimInfo = import('../src/core/protocol').SimInfo;
  type ObjectPlacement = import('../src/core/protocol').ObjectPlacement;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');
  const bike = MOUNTS.bicycle;

  /** The props as built: each node's pivot where the model has it (its axle; the frame at the origin). */
  const built = new Map<string, Prop>();
  const pivotOf = (key: string): Float32Array => {
    const node = key.slice(key.indexOf('#node:') + 6);
    const at = bike.wheels.find((w) => w.node === node) ?? (bike.cranks?.node === node ? bike.cranks : null);
    const m = new Float32Array(16);
    [m[0], m[5], m[10], m[15]] = [1, 1, 1, 1];
    if (at) [m[12], m[13]] = [at.x, at.h];
    return m;
  };
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
    props: new TemplateCache<Prop | null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
    async buildProp(key) {
      const p = { matrices: new Float32Array(MAX_CHARACTERS * 16), top: [0, 0, 0] as const, pivot: pivotOf(key) };
      built.set(key, p);
      return p;
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
      name: 'rider',
      household: 0,
      gender: 'male',
      attractedTo: [],
      appearance: { body: '#5F86C9', skin: '#C08A64', hair: '#2B2320', hairStyle: 'short', height: 1, build: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers' },
      traits: [],
      perks: [],
    },
  ];
  /** The bike: 2 x 1 tiles at (10, 10), turned a quarter (its front wheel towards -z). */
  const ID = 42;
  const parked: ObjectPlacement = { id: ID, def: 'bicycle', x: 10, z: 10, rot: 1, w: 1, d: 2, quality: 0, sellValue: null };
  const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'held', 'ride', 'parkX', 'parkZ', 'parkYaw'] as const;
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

  /**
   * The script (s): riding from home (10, 10) along +z to the café, off it there at `OFF` and left at `PARK` (beside
   * them), inside till `ON`, on it again and riding back home, off it at its place at `HOME`.
   */
  const TICKS = TICKS_PER_SECOND;
  const SPEED = 4;
  const OFF = 2;
  const ON = 5;
  const HOME = 7;
  const END = 9;
  const SPOT = { x: 11.5, z: 11.5 };
  /** Where they get off at the café, and where the bike is left beside them (its middle, facing a quarter turn). */
  const AT = { x: SPOT.x, z: SPOT.z + OFF * SPEED };
  const PARK = { x: AT.x + 1, z: AT.z + 0.5, yaw: Math.PI / 2 };

  function write(snap: Float32Array, n: number): void {
    const t = n / TICKS;
    const out = t < OFF;
    const back = t >= ON && t < HOME;
    const way = out ? t * SPEED : t < ON ? OFF * SPEED : back ? (HOME - t) * SPEED : 0;
    const parked = t >= OFF && t < ON;
    const o = layout.headerLen;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = 1;
    snap[o + k.id] = 0;
    snap[o + k.x] = SPOT.x;
    snap[o + k.z] = SPOT.z + Math.min(way, OFF * SPEED);
    snap[o + k.yaw] = back ? Math.PI : 0;
    snap[o + k.pose] = Pose.Stand;
    snap[o + k.moving] = (out && t > 0.2) || back ? 1 : 0;
    snap[o + k.partner] = -1;
    snap[o + k.emotion] = 1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood] = 0.5;
    snap[o + k.away] = 0;
    snap[o + k.ride] = out || back ? ID : parked ? -2 - ID : -1;
    snap[o + k.parkX] = parked ? PARK.x : 0;
    snap[o + k.parkZ] = parked ? PARK.z : 0;
    snap[o + k.parkYaw] = parked ? PARK.yaw : 0;
  }

  test('slid off to where it is left and drawn there; slid back on from there; home, at its place', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: false });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [parked] });
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      const frameKey = 'model.bicycle#node:frame';
      let tick = 0;
      write(prev, 0);
      write(curr, 0);
      const FPS = 60;
      let jump = 0;
      let last: { x: number; z: number } | null = null;
      let leftAt: { x: number; z: number } | null = null;
      let parkedThere = true;
      let parkedEarly = false;
      let parkedAfter = false;
      let hiddenInside = true;
      let hiddenHome = false;
      let fromThere: { x: number; z: number } | null = null;
      for (let n = 0; n < END * FPS; n++) {
        const t = n / FPS;
        const due = Math.floor(t * TICKS);
        while (tick < due) {
          tick++;
          prev.set(curr);
          write(curr, tick);
        }
        frame.alpha = Math.min(1, t * TICKS - tick);
        now = t * 1000;
        frame.now = now;
        await new Promise((resolve) => setTimeout(resolve, 0));
        characters.update(frame, () => true, shadows);
        const at = characters.parked.get(ID);
        // (Drawn there by the renderer from once they're off it till they're on it again.)
        if (t > OFF + MOUNT_SECONDS + 0.2 && t < ON - 0.1) parkedThere &&= !!at && Math.hypot(at.x - PARK.x, at.z - PARK.z) < 1e-4 && Math.abs(at.yaw - PARK.yaw) < 1e-4;
        if (t < OFF) parkedEarly ||= !!at;
        if (t > ON + 0.2) parkedAfter ||= !!at;
        if (t > OFF + 0.1 && t < ON) hiddenInside &&= characters.ridden.has(ID);
        if (t > HOME + MOUNT_SECONDS + 0.3) hiddenHome ||= characters.ridden.has(ID);
        const m = built.get(frameKey)?.matrices;
        if (!m || m[15] === 0) {
          last = null;
          continue;
        }
        const x = m[12];
        const z = m[14];
        if (last) jump = Math.max(jump, Math.hypot(x - last.x, z - last.z));
        last = { x, z };
        // (Its last place drawn under the rider as they got off, and its first as they get on again.)
        if (t > OFF && t < ON) leftAt = { x, z };
        if (t >= ON && !fromThere) fromThere = { x, z };
      }
      expect(parkedEarly).toBe(false);
      expect(parkedThere).toBe(true);
      expect(parkedAfter).toBe(false);
      expect(hiddenInside).toBe(true);
      expect(hiddenHome).toBe(false);
      // Slid to where it's left (its frame's origin within its length of there), not towards home; back on from there.
      expect(leftAt).not.toBeNull();
      expect(Math.hypot(leftAt!.x - PARK.x, leftAt!.z - PARK.z)).toBeLessThan(0.6);
      expect(fromThere).not.toBeNull();
      expect(Math.hypot(fromThere!.x - PARK.x, fromThere!.z - PARK.z)).toBeLessThan(0.6);
      // (Never jumping: under 12 m/s at 60 frames a second, the slide over and the rider's own way together.)
      expect(jump).toBeLessThan(0.2);
    } finally {
      performance.now = realNow;
    }
  }, 30_000);

  test('placements: left out, drawn there wherever it is from; else at home when off the viewed lot and not ridden', () => {
    const list = [
      { id: 1, x: 0, z: 0, home: true },
      { id: 2, x: 5, z: 5, home: false },
      { id: 3, x: 9, z: 9, home: true },
    ];
    const away = new Map([[2, { x: 40, z: 40, yaw: 1 }]]);
    const drawn = placements(list, away, new Set([2, 3])).map(({ p, at }) => [p.id, at ? at.x : null]);
    expect(drawn).toEqual([
      [1, null],
      [2, 40],
    ]);
  });
});
