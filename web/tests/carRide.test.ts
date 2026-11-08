// Riding in a car (`residents.ts` `setCarRide`, `shared/cars.ts` `carriedAlong`): a resident sat in a car that
// drives goes with it rigidly, every frame, where the car is drawn (between the snapshots, and on past the newest
// while the next is late), turns and all; and from easing into the seat (the car parked) to riding in it and back
// as it sets off and parks, nothing jumps. `bun test`.
//
// (What this caught when written: those riding eased after the car's seat at 9/s, 0.67 m behind it at 6 m/s, and
// at speed 2 further than the snap distance, so they jumped to it and fell behind again; and they turned 6 rad/s at
// most, behind the car on every bend.)
import { afterAll, describe, expect, test } from 'bun:test';
import { readFileSync, existsSync } from 'node:fs';
import { TICK_RATES, TICKS_PER_SECOND } from './snapshotLayout';

const WEB = new URL('../', import.meta.url).pathname;

describe('riding in a car', async () => {
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
  const { CARS, carriedAlong } = await import('../src/render/shared/cars');
  const { lerpAngle, wrapAngle } = await import('../src/render/shared/characters/rig');
  const { maxAlpha } = await import('../src/core/bridge');
  type View = import('../src/render/shared/characters/view').ResidentView<object, object, import('../src/render/shared/characters/view').ResidentProp>;
  type SnapshotLayout = import('../src/core/snapshot').SnapshotLayout;
  type FrameState = import('../src/core/bridge').FrameState;
  type SimInfo = import('../src/core/protocol').SimInfo;
  type CarPose = import('../src/render/shared/cars').CarPose;

  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');

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
      name: 'driver',
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

  const body = CARS[0];
  /** The driver's seat in the car's frame (sim-core `Car::seat_at`: a little back from the front door, on its side). */
  const SEAT = { along: body.len * 0.05, across: body.wid / 2 - 0.5 };
  /** Where the driver gets in and out (beside the front door, outside the car). */
  const DOOR = { along: 0.6, across: body.wid / 2 + 0.6 };
  /** Ticks a second, and the car's speed (m/s) as it drives round a bend (rad/s). */
  const TICKS = TICKS_PER_SECOND;
  const SPEED = 6;
  const BEND = 0.5;
  /** The script (s): stood at the door, sat in, driving off and round, parked again, out. */
  const SAT = 1;
  const DRIVES = 2;
  const PARKS = 6;
  const OUT = 7;
  const END = 8.5;
  const STEP = 0.5;

  /** Where the car is at `t` s (and whether it drives): parked till `DRIVES`, then off along +z bending left, parked again at `PARKS`. */
  function carAt(t: number): CarPose & { driving: boolean } {
    const d = Math.max(0, Math.min(t, PARKS) - DRIVES);
    // Along a circle (yaw grows at `BEND`), starting at the origin facing +z.
    const yaw = BEND * d;
    const r = SPEED / BEND;
    return { x: r * (1 - Math.cos(yaw)), z: r * Math.sin(yaw), yaw, driving: t >= DRIVES && t < PARKS };
  }
  const inCar = (car: CarPose, p: { along: number; across: number }) => ({
    x: car.x + p.across * Math.cos(car.yaw) + p.along * Math.sin(car.yaw),
    z: car.z - p.across * Math.sin(car.yaw) + p.along * Math.cos(car.yaw),
  });

  /** The snapshot at tick `n` (the car's pose kept beside it, as the snapshot's car rows would have it). */
  function write(snap: Float32Array, n: number): CarPose & { driving: boolean } {
    const t = n / TICKS;
    const car = carAt(t);
    const sat = t >= SAT && t < OUT;
    // (Stepping in from the door to the seat, and out, over `STEP` s, as sim-core's `getting` moves them.)
    const u = Math.min(1, Math.max(0, t < OUT ? (t - (SAT - STEP)) / STEP : 1 - (t - OUT) / STEP));
    const [door, seat] = [inCar(car, DOOR), inCar(car, SEAT)];
    const at = { x: door.x + (seat.x - door.x) * u, z: door.z + (seat.z - door.z) * u };
    const o = layout.headerLen;
    snap[layout.header.speed] = 1;
    snap[layout.header.simCount] = 1;
    snap[o + k.id] = 0;
    snap[o + k.x] = at.x;
    snap[o + k.z] = at.z;
    snap[o + k.yaw] = car.yaw;
    snap[o + k.pose] = sat ? Pose.Sit : Pose.Stand;
    snap[o + k.moving] = 0;
    snap[o + k.partner] = -1;
    snap[o + k.emotion] = 1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood] = 0.5;
    return car;
  }

  for (const late of [false, true]) {
    test(late ? 'goes with the car on past a late snapshot' : 'sits in the car where it is drawn, every frame, setting off and parking smoothly', async () => {
      const realNow = performance.now;
      let now = 0;
      performance.now = () => now;
      try {
        const characters = new Characters(view, assets, content, { castsShadows: false });
        expect(await characters.init()).toBe(true);
        await characters.build({ sims, objects: [] });
        // The car's poses in the two snapshots and as drawn (between them, as `pc/cars.ts` `pose` places it).
        let carPrev = carAt(0);
        let carCurr = carAt(0);
        const drawn: CarPose = { x: 0, z: 0, yaw: 0 };
        let rides = 0;
        characters.setCarSeat(() => ({ h: 0.3, fwd: 0 }));
        characters.setCarRide((_frame, x, z, yaw, aboard, out) => {
          if (!(carCurr.driving || carPrev.driving || aboard)) return false;
          rides++;
          return carriedAlong(body, carCurr, drawn, x, z, yaw, out);
        });
        const rig = (characters as unknown as { sims: { x: number; z: number; yaw: number }[] }).sims[0];
        const prev = new Float32Array(layout.capacity);
        const curr = new Float32Array(layout.capacity);
        const frame: FrameState = { prev, curr, alpha: 0, steps: 1, layout, now: 0 };
        const shadows = new Float32Array(MAX_CHARACTERS * 16);
        let tick = 0;
        carPrev = write(prev, 0);
        carCurr = write(curr, 0);
        const FPS = 60;
        let worst = 0;
        let worstYaw = 0;
        let jump = 0;
        let last = { x: NaN, z: NaN };
        let lastSeat = { x: NaN, z: NaN };
        for (let n = 0; n < END * FPS; n++) {
          const t = n / FPS;
          // A snapshot every tick; with `late`, one every quarter of a second comes 15 ms late (shown on past the newest).
          const due = Math.floor(t * TICKS);
          const held = late && due % (TICKS / 4) === 0 && t - due / TICKS < 0.015;
          while (tick < due && !held) {
            tick++;
            prev.set(curr);
            carPrev = carCurr;
            carCurr = write(curr, tick);
          }
          // (Up to the bridge's `MAX_EXTRAPOLATION_MS` past the newest.)
          frame.alpha = Math.min(maxAlpha(1000 / TICKS), t * TICKS - tick);
          now = t * 1000;
          frame.now = now;
          // As `pc/cars.ts` `pose` places it: on past the newest snapshot only while it drives in both.
          const a = carPrev.driving && carCurr.driving ? frame.alpha : Math.min(1, frame.alpha);
          drawn.x = carPrev.x + (carCurr.x - carPrev.x) * a;
          drawn.z = carPrev.z + (carCurr.z - carPrev.z) * a;
          drawn.yaw = lerpAngle(carPrev.yaw, carCurr.yaw, a);
          characters.update(frame, () => true, shadows);
          const seat = inCar(drawn, SEAT);
          if (t > DRIVES + 0.1 && t < PARKS) {
            worst = Math.max(worst, Math.hypot(rig.x - seat.x, rig.z - seat.z));
            worstYaw = Math.max(worstYaw, Math.abs(wrapAngle(rig.yaw - drawn.yaw)));
          }
          // Getting in and out, parked: eased, never a jump.
          // Setting off and parking again: no jump between easing into the seat and riding in it, nor back.
          if (Math.abs(t - DRIVES) < 0.3 || Math.abs(t - PARKS) < 0.3) {
            const move = Math.hypot(rig.x - last.x, rig.z - last.z) - Math.hypot(seat.x - lastSeat.x, seat.z - lastSeat.z);
            if (Number.isFinite(move)) jump = Math.max(jump, Math.abs(move));
          }
          last = { x: rig.x, z: rig.z };
          lastSeat = seat;
        }
        expect(rides).toBeGreaterThan(0);
        // In its seat where the car is drawn (to float precision), facing the way it does.
        expect(worst).toBeLessThan(1e-3);
        expect(worstYaw).toBeLessThan(1e-3);
        expect(jump).toBeLessThan(0.01);
      } finally {
        performance.now = realNow;
      }
    }, 30_000);
  }
});
