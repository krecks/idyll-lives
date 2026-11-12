// A rider the sim catches up (`Residents.setDetailArea`: a big world's chunks streamed in round the camera; sim-core
// `lod.rs`): out of that area the sim moves a resident only now and then and catches them up in one go when it reaches
// them (a bike's rider out of it three game minutes jumped 92 m in one frame, the camera on them). They're drawn out of
// it as ever; as it reaches them, or as they're seen to go further in a step than they could (a catch-up out of it,
// after a pause), they're hidden for `REVEAL_MS` and then drawn where they are, riding on smoothly. `bun test`.
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

  /**
   * The script: riding along +x at `SPEED` m/s. Out of the sim's full-detail area (as a big world's chunks out of the
   * camera's stream) they take a turn now and then (sim-core `lod.rs`): the snapshot keeps them where they were
   * (`STALE`) until the sim's tick catches them up on what they owe in one go (`CATCH` s of game time in: 90 m on),
   * then on along at full detail. Game time `g` stands still while paused.
   */
  const TICKS = TICKS_PER_SECOND;
  const SPEED = 18;
  const STALE = 40;
  const CATCH = 1.08;
  const END = 2.5;
  const truth = (g: number) => STALE + 90 + SPEED * (g - CATCH);

  function write(snap: Float32Array, g: number, paused: boolean): void {
    const o = layout.headerLen;
    snap[layout.header.speed] = paused ? 0 : 1;
    snap[layout.header.simCount] = 1;
    snap[o + k.id] = 0;
    snap[o + k.x] = g < CATCH ? STALE : truth(g);
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

  /**
   * Every frame at 60 a second for `END` s (or `end`): where the rider is drawn (NaN: not drawn). `area(t)`: whether
   * the full-detail area reaches them at time `t` (null: no area at all, as before); `resume`: paused till then;
   * `seen(t)`: whether they're in view.
   */
  async function run(area: ((t: number) => boolean) | null, resume = 0, end = END, seen: (t: number) => boolean = () => true): Promise<{ t: number; x: number; g: number }[]> {
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
      write(prev, 0, resume > 0);
      write(curr, 0, resume > 0);
      const out: { t: number; x: number; g: number }[] = [];
      const FPS = 60;
      let inside: boolean | null = null;
      for (let n = 0; n < end * FPS; n++) {
        const t = n / FPS;
        const paused = t < resume;
        const g = Math.max(0, t - resume);
        const due = Math.floor(g * TICKS);
        while (tick < due) {
          tick++;
          prev.set(curr);
          write(curr, tick / TICKS, false);
        }
        if (paused) write(curr, g, true);
        // (The camera's stream: its area as it is now, set again as it changes.)
        if (area && area(t) !== inside) {
          inside = area(t);
          const now = inside;
          characters.setDetailArea(() => now);
        }
        frame.alpha = paused ? 1 : Math.min(1, g * TICKS - tick);
        now = t * 1000;
        frame.now = now;
        await new Promise((resolve) => setTimeout(resolve, 0));
        characters.update(frame, () => seen(t), shadows);
        out.push({ t, x: visible[0] ? rig.x : NaN, g });
      }
      return out;
    } finally {
      performance.now = realNow;
    }
  }

  /** The longest step between frames drawn one after the other. */
  const worstStep = (frames: { x: number }[]) => {
    let worst = 0;
    for (let i = 1; i < frames.length; i++) if (!Number.isNaN(frames[i].x) && !Number.isNaN(frames[i - 1].x)) worst = Math.max(worst, Math.abs(frames[i].x - frames[i - 1].x));
    return worst;
  };
  const R = REVEAL_MS / 1000;
  /** Drawn where the snapshot has them (within a step of riding), never between there and somewhere else. */
  const inPlace = (f: { x: number; g: number }) => Math.abs(f.x - (f.g < CATCH ? STALE : truth(f.g))) < SPEED / TICKS + 0.05 || Math.abs(f.x - STALE) < 0.05;

  test('drawn where they were out of the area; hidden as it reaches them, then drawn where they are, riding on smoothly', async () => {
    const ARRIVE = 1;
    const frames = await run((t) => t >= ARRIVE);
    // Out of it, as ever: drawn (where the sim has them), once they've been in view a moment (come into view out of
    // it, a resident may be about to be caught up).
    expect(frames.filter((f) => f.t < R - 1e-9).every((f) => Number.isNaN(f.x))).toBe(true);
    expect(frames.filter((f) => f.t >= R + 1e-9 && f.t < ARRIVE).every((f) => Math.abs(f.x - STALE) < 0.05)).toBe(true);
    // Not drawn from the moment it reaches them (the sim's catching up meanwhile) for `REVEAL_MS`.
    expect(frames.filter((f) => f.t >= ARRIVE && f.t < ARRIVE + R - 1e-9).every((f) => Number.isNaN(f.x))).toBe(true);
    const after = frames.filter((f) => f.t >= ARRIVE + R + 1e-9);
    expect(after.every((f) => !Number.isNaN(f.x) && f.x > STALE + 80)).toBe(true);
    expect(after.every(inPlace)).toBe(true);
    expect(worstStep(after)).toBeLessThan((SPEED * 2) / 60);
  }, 30_000);

  test('the camera jumping to them (in view a frame before the area follows it): never drawn where they were', async () => {
    const ARRIVE = 1;
    const frames = await run((t) => t >= ARRIVE + 1 / 60, 0, END, (t) => t >= ARRIVE);
    const drawn = frames.filter((f) => !Number.isNaN(f.x));
    expect(drawn.length).toBeGreaterThan(0);
    expect(drawn[0].t).toBeGreaterThanOrEqual(ARRIVE + R - 1e-9);
    expect(drawn.every((f) => f.x > STALE + 80 && inPlace(f))).toBe(true);
    expect(worstStep(drawn)).toBeLessThan((SPEED * 2) / 60);
  }, 30_000);

  test('caught up out of the area (seen to go further than they could): hidden a moment, then where they are', async () => {
    const frames = await run(() => false);
    const jump = frames.findIndex((f) => f.g >= CATCH);
    expect(frames.slice(0, jump).filter((f) => f.t >= R + 1e-9).every((f) => Math.abs(f.x - STALE) < 0.05)).toBe(true);
    const hidden = frames.filter((f) => f.t >= frames[jump].t && f.t < frames[jump].t + R - 1e-9);
    expect(hidden.every((f) => Number.isNaN(f.x))).toBe(true);
    const after = frames.filter((f) => f.t >= frames[jump].t + R + 1e-9);
    expect(after.length).toBeGreaterThan(0);
    expect(after.every((f) => !Number.isNaN(f.x) && inPlace(f))).toBe(true);
    expect(worstStep(frames)).toBeLessThan((SPEED * 2) / 60);
  }, 30_000);

  test('paused as the area reaches them: drawn where the sim has them, then where it catches them up on resuming, never sliding', async () => {
    const ARRIVE = 0.5;
    const RESUME = 1.5;
    const frames = await run((t) => t >= ARRIVE, RESUME, RESUME + CATCH + 1);
    // Hidden as it reaches them, then shown where they are (paused: the sim can't catch anyone up).
    expect(frames.filter((f) => f.t >= ARRIVE && f.t < ARRIVE + R - 1e-9).every((f) => Number.isNaN(f.x))).toBe(true);
    expect(frames.filter((f) => f.t >= ARRIVE + R + 1e-9 && f.t < RESUME).every((f) => Math.abs(f.x - STALE) < 0.05)).toBe(true);
    // On resuming, caught up: hidden a moment, then where they are; never a frame between.
    expect(frames.every((f) => Number.isNaN(f.x) || inPlace(f))).toBe(true);
    expect(frames.filter((f) => f.g >= CATCH + R + 0.02).every((f) => !Number.isNaN(f.x))).toBe(true);
    expect(worstStep(frames)).toBeLessThan((SPEED * 2) / 60);
  }, 30_000);

  test('(with no area, as before: a jump when the sim catches them up)', async () => {
    const frames = await run(null);
    expect(frames[0].x).toBeCloseTo(STALE, 3);
    expect(worstStep(frames)).toBeGreaterThan(80);
  }, 30_000);
});
