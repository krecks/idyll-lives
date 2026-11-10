// A rider coming into the sim's full-detail area (`Residents.setDetailArea`: a big world's chunks streamed in round
// the camera; sim-core `lod.rs`): out of it the sim moves a resident only now and then and catches them up in one go
// when it reaches them (a bike's rider out of it three game minutes jumped 92 m in one frame, the camera on them). They
// aren't drawn out of it, and come into view only `REVEAL_MS` after it reaches them, where they are, riding on
// smoothly from there. `bun test`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { TICK_RATES, TICKS_PER_SECOND } from './snapshotLayout';

const WEB = new URL('../', import.meta.url).pathname;

describe('a rider coming into the full-detail area', async () => {
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
  const { Characters, MAX_CHARACTERS, REVEAL_MS } = await import('../src/render/shared/characters/residents');
  const { TemplateCache } = await import('../src/render/shared/templateCache');
  const { Pose, simFields } = await import('../src/core/snapshot');
  const { MOUNTS } = await import('../src/render/shared/characters/vehicles');
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

  /**
   * The script: riding along +x at `SPEED` m/s from `START` the whole time. Out of the sim's full-detail area (as a
   * big world's chunks out of the camera's stream) they take a turn now and then (sim-core `lod.rs`): the snapshot
   * keeps them where they were (`STALE`) until the area comes to them (`ARRIVE`: the camera there) and the sim's next
   * tick catches them up on what they owe in one go (`CATCH`), 90 m on, then on along at full detail.
   */
  const TICKS = TICKS_PER_SECOND;
  const SPEED = 18;
  const START = 40;
  const ARRIVE = 1;
  const CATCH = 1.08;
  const END = 2.5;
  const STALE = START;
  const truth = (t: number) => START + 90 + SPEED * (t - CATCH);

  function write(snap: Float32Array, n: number): void {
    const t = n / TICKS;
    const o = layout.headerLen;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = 1;
    snap[o + k.id] = 0;
    snap[o + k.x] = t < CATCH ? STALE : truth(t);
    snap[o + k.z] = 20.5;
    snap[o + k.yaw] = Math.PI / 2;
    snap[o + k.pose] = Pose.Stand;
    snap[o + k.moving] = 1;
    snap[o + k.partner] = -1;
    snap[o + k.emotion] = 1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood] = 0.5;
    snap[o + k.away] = 0;
    snap[o + k.ride] = ID;
  }

  /** Every frame at 60 a second (`detail`: the area set at `ARRIVE`, none before; null: no area at all): where the rider is drawn, and when (NaN: not drawn). */
  async function run(detail: boolean): Promise<{ t: number; x: number }[]> {
    const realNow = performance.now;
    let now = 0;
    performance.now = () => now;
    try {
      const characters = new Characters(view, assets, content, { castsShadows: false });
      expect(await characters.init()).toBe(true);
      await characters.build({ sims, objects: [parked] });
      const rig = (characters as unknown as { sims: { x: number }[] }).sims[0];
      const visible = (characters as unknown as { visible: Uint8Array }).visible;
      const prev = new Float32Array(layout.capacity);
      const curr = new Float32Array(layout.capacity);
      const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
      const shadows = new Float32Array(MAX_CHARACTERS * 16);
      let tick = 0;
      write(prev, 0);
      write(curr, 0);
      // The camera's stream far off at first (the area doesn't reach them), there from `ARRIVE`.
      if (detail) characters.setDetailArea(() => false);
      const out: { t: number; x: number }[] = [];
      const FPS = 60;
      for (let n = 0; n < END * FPS; n++) {
        const t = n / FPS;
        const due = Math.floor(t * TICKS);
        while (tick < due) {
          tick++;
          prev.set(curr);
          write(curr, tick);
        }
        if (detail && n === ARRIVE * FPS) characters.setDetailArea(() => true);
        frame.alpha = Math.min(1, t * TICKS - tick);
        now = t * 1000;
        frame.now = now;
        await new Promise((resolve) => setTimeout(resolve, 0));
        characters.update(frame, () => true, shadows);
        out.push({ t, x: visible[0] ? rig.x : NaN });
      }
      return out;
    } finally {
      performance.now = realNow;
    }
  }

  test('appears where they are, a moment after the area comes to them, and rides on smoothly', async () => {
    const frames = await run(true);
    const drawn = frames.filter((f) => !Number.isNaN(f.x));
    // Not drawn out of the area, nor in it before `REVEAL_MS` (the sim's catching up meanwhile).
    expect(drawn.length).toBeGreaterThan(0);
    expect(drawn[0].t).toBeGreaterThanOrEqual(ARRIVE + REVEAL_MS / 1000 - 1e-9);
    expect(drawn[0].t).toBeLessThan(ARRIVE + REVEAL_MS / 1000 + 0.05);
    // Never where the snapshot had them before (`STALE`): first drawn where they are, within a step.
    expect(drawn.every((f) => f.x > STALE + 80)).toBe(true);
    expect(Math.abs(drawn[0].x - truth(drawn[0].t))).toBeLessThan(SPEED / TICKS + 0.05);
    // Then along smoothly: no frame further than riding goes in one (with a tick's slack), no slide.
    let worst = 0;
    for (let i = 1; i < drawn.length; i++) worst = Math.max(worst, Math.abs(drawn[i].x - drawn[i - 1].x));
    expect(worst).toBeLessThan((SPEED * 2) / 60);
  }, 30_000);

  test('(with no area, as before: drawn where the snapshot was, then a jump when the sim catches them up)', async () => {
    const frames = await run(false);
    let worst = 0;
    for (let i = 1; i < frames.length; i++) worst = Math.max(worst, Math.abs(frames[i].x - frames[i - 1].x));
    expect(frames[0].x).toBeCloseTo(STALE, 3);
    expect(worst).toBeGreaterThan(80);
  }, 30_000);
});
