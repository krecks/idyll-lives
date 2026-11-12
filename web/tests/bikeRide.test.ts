// Riding a bike (`residents.ts` `writeMount`, `shared/characters/vehicles.ts`; sim-core `ride.rs`): a resident whose
// snapshot row says they ride a bike has it drawn under them, its saddle under their hips, facing the way they go,
// its wheels turning forward by the way they've gone; it slides from where it stands to under them as they get on
// and back as they get off, never jumping; and it's in `ridden` (not drawn where it stands) from getting on, away at
// work too, until it's back in its place. `bun test`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { TICK_RATES, TICKS_PER_SECOND } from './snapshotLayout';

const WEB = new URL('../', import.meta.url).pathname;

describe('riding a bike', async () => {
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
  const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'held', 'ride'] as const;
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

  /** The script (s): stood by the bike, on it, riding off along +z at `SPEED` m/s, away at work, back on it, off again. */
  const TICKS = TICKS_PER_SECOND;
  const SPEED = 4;
  const ON = 1;
  const AWAY = 4;
  const BACK = 6;
  const OFF = 8;
  const END = 10;
  /** Where they stand to get on and off (beside it, in front of it: its front tile's middle). */
  const SPOT = { x: 11.5, z: 11.5 };

  function write(snap: Float32Array, n: number): void {
    const t = n / TICKS;
    const riding = t >= ON && t < OFF;
    const away = t >= AWAY && t < BACK;
    // Off from the spot along +z and back to it.
    const way = t < ON ? 0 : t < AWAY ? (t - ON) * SPEED : t < BACK ? 0 : t < OFF ? (OFF - t) * SPEED : 0;
    const o = layout.headerLen;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = 1;
    snap[o + k.id] = 0;
    snap[o + k.x] = SPOT.x;
    snap[o + k.z] = SPOT.z + way;
    snap[o + k.yaw] = t >= BACK ? Math.PI : 0;
    snap[o + k.pose] = Pose.Stand;
    snap[o + k.moving] = riding && !away && (t < AWAY || t >= BACK) && way > 0 ? 1 : 0;
    snap[o + k.partner] = -1;
    snap[o + k.emotion] = 1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood] = 0.5;
    snap[o + k.away] = away ? 1 : 0;
    snap[o + k.ride] = riding ? ID : -1;
  }

  test('drawn under the rider, the wheels turning; slid on and off, hidden where it stands meanwhile', async () => {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: false });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [parked] });
      const rig = (characters as unknown as { sims: { x: number; z: number; yaw: number }[] }).sims[0];
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      const frameKey = 'model.bicycle#node:frame';
      const wheelKey = 'model.bicycle#node:wheel_front';
      let tick = 0;
      write(prev, 0);
      write(curr, 0);
      const FPS = 60;
      let worstSaddle = 0;
      let worstYaw = 0;
      let jump = 0;
      let turned = 0;
      let rolled = 0;
      let lastTurn: number | null = null;
      let last: { x: number; z: number } | null = null;
      let hiddenAway = true;
      let hiddenBefore = false;
      let hiddenAfter = false;
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
        // (The props are built as they're first asked for: at once here.)
        await new Promise((resolve) => setTimeout(resolve, 0));
        characters.update(frame, () => true, shadows);
        const ridden = characters.ridden.has(ID);
        if (t < ON - 0.05) hiddenBefore ||= ridden;
        if (t >= AWAY + 0.1 && t < BACK - 0.1) hiddenAway &&= ridden;
        if (t > OFF + MOUNT_SECONDS + 0.2) hiddenAfter ||= ridden;
        const m = built.get(frameKey)?.matrices;
        if (!m || m[15] === 0) {
          last = null;
          continue;
        }
        // The saddle's top (the model's point under the hips) and which way its front faces.
        const sx = m[0] * bike.seat[0] + m[4] * bike.seat[1] + m[12];
        const sz = m[2] * bike.seat[0] + m[6] * bike.seat[1] + m[14];
        const front = Math.atan2(m[0], m[2]);
        // Riding along, at least a mount's time on: under the hips, facing the way they face.
        if ((t > ON + MOUNT_SECONDS + 0.1 && t < AWAY) || (t > BACK + MOUNT_SECONDS + 0.1 && t < OFF)) {
          worstSaddle = Math.max(worstSaddle, Math.hypot(sx - rig.x, sz - rig.z));
          worstYaw = Math.max(worstYaw, Math.abs(Math.atan2(Math.sin(front - rig.yaw), Math.cos(front - rig.yaw))));
          // The front wheel turned from the frame's upright as it rolls: forward (its top towards its front).
          const w = built.get(wheelKey)?.matrices;
          if (w && t > 2 && t < 2.5) {
            turned = Math.max(turned, Math.acos(Math.min(1, w[4] * m[4] + w[5] * m[5] + w[6] * m[6])));
            const a = Math.atan2(w[4] * m[0] + w[5] * m[1] + w[6] * m[2], w[4] * m[4] + w[5] * m[5] + w[6] * m[6]);
            if (lastTurn !== null) rolled += Math.atan2(Math.sin(a - lastTurn), Math.cos(a - lastTurn)) > 0 ? 1 : -1;
            lastTurn = a;
          }
        }
        // Getting on and off: slid between its place and the rider, never jumping (the rider moves less than 7 cm a frame).
        if (last) jump = Math.max(jump, Math.hypot(sx - last.x, sz - last.z));
        last = { x: sx, z: sz };
      }
      expect(hiddenBefore).toBe(false);
      expect(hiddenAway).toBe(true);
      expect(hiddenAfter).toBe(false);
      expect(worstSaddle).toBeLessThan(0.02);
      expect(worstYaw).toBeLessThan(0.05);
      expect(turned).toBeGreaterThan(0.1);
      expect(rolled).toBeGreaterThan(20);
      expect(jump).toBeLessThan(0.15);
    } finally {
      performance.now = realNow;
    }
  }, 30_000);
});
