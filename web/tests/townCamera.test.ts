/**
 * The town view's camera (`render/shared/townCamera.ts`): one zoom from
 * the play orbit out to `TOWN_MAX` with nothing jumping at `TOWN_FROM`, the tilt eased into the bird's-eye range,
 * the target held over land and given back on the way in, the haze and the planes (the menus' town's too), the
 * flights down from the town view, and the roaming focus left alone up there.
 */
import { describe, expect, test } from 'bun:test';
import { Flight, panBounds, Roam, SETTLE_MS, type FocusPlot } from '../src/render/shared/focus';
import { BETA_LIMITS, LENS, RADIUS_LIMITS, TOWN_FROM, TOWN_FULL, TOWN_MAX, townBlend } from '../src/render/shared/lens';
import { OrbitControls } from '../src/render/shared/orbit';
import { edgeFog, FOG_EDGE, FOG_FROM, landEdge, menuFog, menuFogFrom, TOWN_BETA, TOWN_FOG, TOWN_NEAR_SHARE, TOWN_RADIUS_LIMITS, townArrival, townBeta, townFar, townFog, townFogFrom, townInset, townKeyZoom, townNear, townOrbit, townWheel } from '../src/render/shared/townCamera';

const WORLD = { width: 1024, depth: 1024, meta: { world: { size: 'big' } } };
const RING = 260;

/** The game's orbit over a big world, as the renderer sets it up. */
function orbit(pose = { alpha: 0, beta: 0.86, radius: 24 }): OrbitControls {
  const o = new OrbitControls(pose, { radius: TOWN_RADIUS_LIMITS, beta: BETA_LIMITS });
  o.wheelDeltaPercentage = 0.012;
  o.panningSensibility = 90;
  o.mapPanning = true;
  o.panRadius = 24;
  o.inertia = 0.82;
  o.panningInertia = 0.82;
  o.panBounds = panBounds(WORLD);
  o.shape = townOrbit(WORLD.width, WORLD.depth, 20, RING);
  o.target.x = o.target.z = 512;
  return o;
}

/** The same orbit as the play camera has always been: its own limits, no town view. */
function playOrbit(pose = { alpha: 0, beta: 0.86, radius: 24 }): OrbitControls {
  const o = orbit(pose);
  o.limits = { radius: RADIUS_LIMITS, beta: BETA_LIMITS };
  o.shape = null;
  return o;
}

/** A canvas that hands its listeners back, to send the orbit wheel and key events. */
function canvas() {
  const listeners = new Map<string, (e: unknown) => void>();
  const element = { addEventListener: (type: string, fn: (e: unknown) => void) => listeners.set(type, fn), removeEventListener: () => {}, setPointerCapture: () => {}, releasePointerCapture: () => {} };
  return {
    element: element as unknown as HTMLElement,
    wheel: (deltaY: number) => listeners.get('wheel')!({ deltaY, deltaMode: 0 }),
    key: (keyCode: number, down: boolean, ctrlKey = false) => listeners.get(down ? 'keydown' : 'keyup')!({ keyCode, ctrlKey, altKey: false, metaKey: false }),
    drag: (button: number, dx: number, dy: number) => {
      listeners.get('pointerdown')!({ button, pointerId: 1, clientX: 0, clientY: 0, ctrlKey: false, preventDefault: () => {} });
      listeners.get('pointermove')!({ pointerId: 1, clientX: dx, clientY: dy });
      listeners.get('pointerup')!({ pointerId: 1 });
    },
  };
}

// (The orbit's listeners read these off the globals in a browser.)
const g = globalThis as unknown as { window?: unknown; WheelEvent?: unknown };
g.window ??= { addEventListener: () => {}, removeEventListener: () => {} };
g.WheelEvent ??= { DOM_DELTA_LINE: 1 };

const FRAME = 1000 / 60;

describe('the town view: the orbit', () => {
  test('up to TOWN_FROM nothing differs from the play camera', () => {
    expect(townBlend(TOWN_FROM)).toBe(0);
    const out: [number, number] = [0, 0];
    for (const r of [RADIUS_LIMITS[0], 24, RADIUS_LIMITS[1], TOWN_FROM]) {
      townBeta(r, BETA_LIMITS, out);
      expect(out).toEqual([...BETA_LIMITS]);
      expect(townInset(r, 1024, 1024, 20, RING)).toBe(0);
      expect(townWheel(r)).toBe(1);
      expect(townKeyZoom(r)).toBe(1);
      expect(townFog(r)).toBe(1);
      expect(townFogFrom(r)).toBe(0);
      expect(townNear(r, 1000)).toBe(Math.max(LENS.near, r * LENS.nearShare));
      expect(townFar(r, 2600, 0, 50, 0, -260, -260, 1284, 1284)).toBe(2600);
    }
  });

  test('the same input moves the play camera the same, with the town view or without', () => {
    const [a, b] = [orbit(), playOrbit()];
    const [ca, cb] = [canvas(), canvas()];
    a.attach(ca.element);
    b.attach(cb.element);
    // Wheel out, a turn, a pan, the zoom keys: all inside the play range.
    for (let f = 0; f < 240; f++) {
      for (const c of [ca, cb]) {
        if (f < 12) c.wheel(100);
        if (f === 40) c.drag(0, 180, -90);
        if (f === 70) c.drag(2, -240, 130);
        if (f === 100) c.key(189, true);
        if (f === 130) c.key(189, false);
        if (f === 150) c.key(187, true);
        if (f === 200) c.key(187, false);
      }
      a.step(FRAME);
      b.step(FRAME);
      expect(a.radius).toBeLessThan(TOWN_FROM);
      expect([a.alpha, a.beta, a.radius, a.target.x, a.target.z]).toEqual([b.alpha, b.beta, b.radius, b.target.x, b.target.z]);
    }
    expect(a.radius).not.toBe(24);
  });

  test('the wheel goes out to TOWN_MAX and no further, without a jump on the way', () => {
    const o = orbit({ alpha: 0, beta: 1.3, radius: 24 });
    const c = canvas();
    o.attach(c.element);
    let last = { radius: o.radius, beta: o.beta };
    let passed = false;
    let notches = 0;
    for (let f = 0; f < 4000 && o.radius < TOWN_MAX; f++) {
      if (f % 6 === 0) {
        c.wheel(100);
        notches++;
      }
      o.step(FRAME);
      // (A frame's zoom is a small share of the radius, and the tilt follows it smoothly: no step at TOWN_FROM.)
      expect(o.radius / last.radius).toBeLessThan(1.06);
      expect(Math.abs(o.beta - last.beta)).toBeLessThan(0.02);
      if (last.radius <= TOWN_FROM && o.radius > TOWN_FROM) passed = true;
      last = { radius: o.radius, beta: o.beta };
    }
    expect(passed).toBe(true);
    expect(o.radius).toBe(TOWN_MAX);
    // (From the game's opening distance to the whole town in a few dozen notches, not a hundred.)
    expect(notches).toBeLessThan(60);
    for (let f = 0; f < 60; f++) o.step(FRAME);
    expect(o.radius).toBe(TOWN_MAX);
  });

  test('the tilt eases into the bird’s-eye range and comes back as the camera comes in', () => {
    const o = orbit({ alpha: 0, beta: 1.3, radius: 60 });
    o.step(FRAME);
    expect(o.beta).toBe(1.3);
    o.radius = (TOWN_FROM + TOWN_FULL) / 2;
    o.step(FRAME);
    expect(o.beta).toBeLessThan(1.3);
    expect(o.beta).toBeGreaterThan(TOWN_BETA[1]);
    o.radius = TOWN_FULL;
    o.step(FRAME);
    expect(o.beta).toBeCloseTo(TOWN_BETA[1]);
    o.radius = TOWN_MAX;
    o.step(FRAME);
    expect(o.beta).toBeCloseTo(TOWN_BETA[1]);
    // Back in: the tilt the player had.
    o.radius = 60;
    o.step(FRAME);
    expect(o.beta).toBe(1.3);
    // A steep one is lifted to the range's other end, and comes back too.
    const steep = orbit({ alpha: 0, beta: 0.35, radius: TOWN_MAX });
    steep.step(FRAME);
    expect(steep.beta).toBeCloseTo(TOWN_BETA[0]);
    steep.radius = 40;
    steep.step(FRAME);
    expect(steep.beta).toBe(0.35);
  });

  test('a shape set once the camera is in use starts from the tilt and the target it has', () => {
    // (The renderer sets the shape with the world, after the orbit's first frames.)
    const o = playOrbit({ alpha: 0, beta: 0.86, radius: 24 });
    o.target.x = 700;
    o.target.z = 300;
    for (let f = 0; f < 5; f++) o.step(FRAME);
    o.limits = { radius: TOWN_RADIUS_LIMITS, beta: BETA_LIMITS };
    o.shape = townOrbit(WORLD.width, WORLD.depth, 20, RING);
    o.step(FRAME);
    expect([o.beta, o.target.x, o.target.z]).toEqual([0.86, 700, 300]);
    // And another world's shape, set while the camera is held in the town view, keeps what it holds back.
    o.radius = TOWN_MAX;
    o.target.x = 1000;
    o.step(FRAME);
    const held = o.target.x;
    expect(held).toBeLessThan(1000);
    o.shape = townOrbit(WORLD.width, WORLD.depth, 20, RING);
    o.step(FRAME);
    expect(o.target.x).toBe(held);
    o.radius = 24;
    o.step(FRAME);
    expect(o.target.x).toBe(1000);
  });

  test('a tilt the player sets in the town view is theirs from then on', () => {
    const o = orbit({ alpha: 0, beta: 1.3, radius: TOWN_FULL });
    const c = canvas();
    o.attach(c.element);
    o.step(FRAME);
    expect(o.beta).toBeCloseTo(TOWN_BETA[1]);
    // Dragged steeper (the pointer down the screen tilts the camera up over the town).
    c.drag(0, 0, 250);
    for (let f = 0; f < 120; f++) o.step(FRAME);
    const set = o.beta;
    expect(set).toBeLessThan(TOWN_BETA[1] - 0.1);
    expect(set).toBeGreaterThanOrEqual(TOWN_BETA[0]);
    o.radius = 50;
    o.step(FRAME);
    expect(Math.abs(o.beta - set)).toBeLessThan(0.01);
  });

  test('the target is held over land the further out the camera is, and given back on the way in', () => {
    const o = orbit();
    // Home in a corner of the land.
    o.target.x = 960;
    o.target.z = 40;
    o.step(FRAME);
    expect([o.target.x, o.target.z]).toEqual([960, 40]);
    let last = { x: o.target.x, z: o.target.z, radius: o.radius };
    for (let r = 24; r < TOWN_MAX; r *= 1.02) {
      o.radius = r;
      o.step(FRAME);
      // (It slides in with the zoom: a little a step, never away from the middle.)
      expect(Math.max(last.x - o.target.x, o.target.z - last.z)).toBeLessThan(0.7 * (r - last.radius) + 1e-6);
      expect(o.target.x).toBeLessThanOrEqual(last.x);
      expect(o.target.z).toBeGreaterThanOrEqual(last.z);
      last = { x: o.target.x, z: o.target.z, radius: r };
    }
    o.radius = TOWN_MAX;
    o.step(FRAME);
    // Well inside the land, with room left to roam.
    const inset = townInset(TOWN_MAX, 1024, 1024, 20, RING);
    expect(o.target.x).toBeCloseTo(1044 - inset);
    expect(o.target.z).toBeCloseTo(-20 + inset);
    expect(o.target.x).toBeLessThan(1024 - 200);
    expect(1044 - inset - 512).toBeGreaterThanOrEqual(96 - 1e-9);
    // Back in without a pan: home again.
    o.radius = 24;
    o.step(FRAME);
    expect([o.target.x, o.target.z]).toEqual([960, 40]);
  });

  test('a pan in the town view moves on from where the camera is held, and stays the player’s', () => {
    const o = orbit({ alpha: -Math.PI / 2, beta: 0.86, radius: 700 });
    const c = canvas();
    o.attach(c.element);
    o.target.x = 1000;
    o.target.z = 512;
    o.step(FRAME);
    const held = o.target.x;
    expect(held).toBeLessThan(1000);
    // A drag that moves the view towards the middle.
    c.drag(2, 300, 0);
    for (let f = 0; f < 90; f++) o.step(FRAME);
    const moved = o.target.x;
    expect(Math.abs(moved - held)).toBeGreaterThan(20);
    expect(moved).toBeLessThan(held);
    // Coming in, it stays there: the corner it was held back from is forgotten.
    o.radius = 30;
    o.step(FRAME);
    expect(o.target.x).toBe(moved);
  });

  test('a drag pans the view as far across the screen at any distance', () => {
    const across = (radius: number) => {
      const o = orbit({ alpha: -Math.PI / 2, beta: 0.86, radius });
      const c = canvas();
      o.attach(c.element);
      o.step(FRAME);
      const from = o.target.x;
      c.drag(2, 100, 0);
      for (let f = 0; f < 200; f++) o.step(FRAME);
      // (Metres moved, as a share of what the picture shows across its height at the target.)
      return Math.abs(o.target.x - from) / (2 * radius * Math.tan(LENS.fov / 2));
    };
    const play = across(60);
    expect(play).toBeGreaterThan(0.01);
    for (const r of [TOWN_FROM, 250, TOWN_FULL, 700]) expect(across(r)).toBeCloseTo(play, 6);
  });

  test('the zoom keys keep their pace in the play range and keep up with the distance past it', () => {
    // Seconds to zoom out from `from` to `to` with the key held.
    const secs = (from: number, to: number) => {
      const o = orbit({ alpha: 0, beta: 0.86, radius: from });
      const c = canvas();
      o.attach(c.element);
      c.key(189, true);
      let f = 0;
      for (; f < 60 * 120 && o.radius < to; f++) o.step(FRAME, 1);
      return f / 60;
    };
    const play = secs(24, RADIUS_LIMITS[1]);
    expect(play).toBeGreaterThan(3);
    // (The whole way out to the town takes about as long again, not a minute.)
    const town = secs(TOWN_FROM, TOWN_MAX);
    expect(town).toBeLessThan(12);
    expect(town).toBeGreaterThan(3);
    // No jump in speed at TOWN_FROM.
    expect(townKeyZoom(TOWN_FROM + 0.5)).toBeCloseTo(1, 1);
    expect(townWheel(TOWN_FROM + 0.5)).toBeCloseTo(1, 3);
  });

  test('the player’s pan or zoom is noticed once (a flight under way ends on it), a turn is not', () => {
    const o = orbit();
    const c = canvas();
    o.attach(c.element);
    expect(o.takeMoved()).toBe(false);
    c.drag(0, 50, 10);
    o.step(FRAME);
    expect(o.takeMoved()).toBe(false);
    c.wheel(100);
    expect(o.takeMoved()).toBe(true);
    expect(o.takeMoved()).toBe(false);
    c.drag(2, 30, 0);
    expect(o.takeMoved()).toBe(true);
    c.key(189, true);
    o.step(FRAME);
    c.key(189, false);
    expect(o.takeMoved()).toBe(true);
    expect(o.takeMoved()).toBe(false);
  });
});

describe('the town view: haze and planes', () => {
  test('the haze thins to a quarter by TOWN_FULL, and the middle of the picture never gets hazier on the way out', () => {
    expect(townFog(TOWN_FULL)).toBeCloseTo(TOWN_FOG);
    // (Exponential-squared haze: what is left of a thing `r` m off is exp(-(r · density)²).)
    const clear = (r: number) => Math.exp(-((r * 0.0032 * townFog(r)) ** 2));
    let last = 1;
    for (let r = TOWN_FROM; r <= TOWN_MAX; r += 5) {
      const f = townFog(r);
      expect(f).toBeLessThanOrEqual(last);
      expect(last - f).toBeLessThan(0.05);
      expect(clear(r)).toBeGreaterThanOrEqual(clear(TOWN_FROM) - 1e-9);
      last = f;
    }
    expect(clear(TOWN_MAX)).toBeCloseTo(clear(TOWN_FULL));
    expect(clear(TOWN_MAX)).toBeGreaterThan(0.85);
  });

  test('the haze begins past the town in the town view, moving out smoothly from the play camera’s', () => {
    // (What is left of a thing `d` m off, as `shaders/fog.ts` draws it: the exponential-squared factor from the start, to the 2.2th.)
    const clear = (d: number, r: number) => Math.exp(-2.2 * (Math.max(0, d - townFogFrom(r)) * 0.0032 * townFog(r)) ** 2);
    let last = 0;
    for (let r = TOWN_FROM; r <= TOWN_MAX; r += 5) {
      const from = townFogFrom(r);
      expect(from).toBeGreaterThanOrEqual(last);
      expect(from - last).toBeLessThan(FOG_FROM * 5 * 4);
      // The middle of the picture never hazier than from `TOWN_FROM`, and clear once halfway into the town view.
      expect(clear(r, r)).toBeGreaterThanOrEqual(clear(TOWN_FROM, TOWN_FROM) - 1e-9);
      if (townBlend(r) >= 0.5) expect(clear(r, r)).toBe(1);
      last = from;
    }
    expect(townFogFrom(TOWN_FULL)).toBeCloseTo(FOG_FROM * TOWN_FULL);
    // The town's far side, half the radius past its middle, clear from the full town view out.
    for (const r of [TOWN_FULL, 650, TOWN_MAX]) expect(clear(r * 1.5, r)).toBe(1);
  });

  test('the menus’ town has the full town view’s haze at any distance, thinner round mountains', () => {
    for (const r of [40, 120, 300, 640, 1100]) {
      expect(menuFogFrom(r)).toBeCloseTo(FOG_FROM * r);
      expect(menuFog(r, false)).toBeLessThanOrEqual(1);
      expect(menuFog(r, true)).toBeCloseTo(menuFog(r, false) * 0.35);
    }
    expect(menuFog(TOWN_FULL, false)).toBeCloseTo(TOWN_FOG);
    expect(menuFog(TOWN_MAX, false)).toBeCloseTo(townFog(TOWN_MAX));
    // Close up (the lot a new-game screen glides to) the look's own density.
    expect(menuFog(40, false)).toBe(1);
  });

  test('where the land ends ahead: the nearest of the picture’s middle and sides', () => {
    const [x0, z0, x1, z1] = [-RING, -RING, 1024 + RING, 1024 + RING];
    // Looking along -x (α 0) from the middle, the picture straight: the edge at x0, 500 m up.
    expect(landEdge(512, 500, 512, 0, 0, x0, z0, x1, z1)).toBeCloseTo(Math.hypot(512 + RING, 500));
    // The picture's sides reach the edge no further than its middle does.
    expect(landEdge(512, 500, 512, 0, 0.6, x0, z0, x1, z1)).toBeLessThanOrEqual(landEdge(512, 500, 512, 0, 0, x0, z0, x1, z1));
    // An eye off the land looking across it: where the line leaves it; looking away: nothing ends ahead.
    expect(landEdge(1024 + RING + 300, 500, 512, 0, 0, x0, z0, x1, z1)).toBeCloseTo(Math.hypot(1024 + 2 * RING + 300, 500));
    expect(landEdge(-RING - 300, 500, 512, 0, 0, x0, z0, x1, z1)).toBe(Infinity);
  });

  test('the land fades by its edge seen from above, not on a low camera’s horizon', () => {
    const r = 900;
    const from = FOG_FROM * r;
    const haze = (d: number, density: number) => 1 - Math.exp(-2.2 * (Math.max(0, d - from) * density) ** 2);
    expect(haze(2600, edgeFog(r, from, 2600, 0.8))).toBeCloseTo(FOG_EDGE);
    // An edge nearer than the start: never thicker than that haze half the radius on.
    expect(haze(from + r / 2, edgeFog(r, from, from - 100, 0.8))).toBeCloseTo(FOG_EDGE);
    expect(edgeFog(r, from, 2600, 1.32)).toBe(0);
    expect(edgeFog(r, from, Infinity, 0.8)).toBe(0);
  });

  test('the menus’ town has the full town view’s planes at any distance', () => {
    const [x0, z0, x1, z1] = [-RING, -RING, 1024 + RING, 1024 + RING];
    // The main menu's low shot, 440 m out: the far plane past the land's far corner, not the play camera's 1600 m.
    const [ex, ey, ez] = [100, 110, 300];
    expect(townFar(440, 1600, ex, ey, ez, x0, z0, x1, z1, 1)).toBeGreaterThan(Math.hypot(x1 - ex, ey, z1 - ez));
    expect(townNear(440, 110, 1)).toBeCloseTo(Math.min(440 * TOWN_NEAR_SHARE, 55));
  });

  test('the near plane moves out with the town view, never past half the eye’s height over the land', () => {
    expect(townNear(TOWN_MAX, 600)).toBeCloseTo(TOWN_MAX * TOWN_NEAR_SHARE);
    expect(townNear(TOWN_FULL, 250)).toBeCloseTo(TOWN_FULL * TOWN_NEAR_SHARE);
    // Over a hill: half the clearance, and never nearer than the play camera's plane.
    expect(townNear(TOWN_MAX, 100)).toBe(50);
    expect(townNear(TOWN_MAX, 10)).toBe(TOWN_MAX * LENS.nearShare);
    let last = townNear(TOWN_FROM, 1e4);
    for (let r = TOWN_FROM; r <= TOWN_MAX; r += 5) {
      const n = townNear(r, 1e4);
      expect(n).toBeGreaterThanOrEqual(last);
      expect(n - last).toBeLessThan(2.5);
      last = n;
    }
  });

  test('the roads keep their depth from the full town view', () => {
    // A 24-bit depth buffer tells apart z²/(near·2²⁴) m at z m: under half of the roads' 1.2 cm over the ground
    // (seen at the tilt's flattest, cos 1.0 of it lies along the view) at the far end of what the camera shows.
    for (const r of [TOWN_FULL, 650, TOWN_MAX]) {
      const height = r * Math.cos(TOWN_BETA[1]);
      const near = townNear(r, height);
      const furthest = height / Math.cos(TOWN_BETA[1] + LENS.fov / 2);
      const step = (furthest * furthest) / (near * 2 ** 24);
      expect(step).toBeLessThan(0.5 * 0.012 * Math.cos(TOWN_BETA[1]));
    }
  });

  test('the far plane reaches past the whole world from the full town view, the preset’s kept in play', () => {
    const [x0, z0, x1, z1] = [-RING, -RING, 1024 + RING, 1024 + RING];
    // The eye of a camera 900 m out over a corner of the land.
    const [ex, ey, ez] = [1024 + 600, 560, 1024 + 300];
    const far = townFar(TOWN_MAX, 1800, ex, ey, ez, x0, z0, x1, z1);
    expect(far).toBeGreaterThan(Math.hypot(ex - x0, ey, ez - z0));
    expect(townFar(TOWN_FROM, 1800, ex, ey, ez, x0, z0, x1, z1)).toBe(1800);
    // A small town's world is inside the preset's plane already: it stays.
    expect(townFar(TOWN_MAX, 1600, 60, 500, 60, -150, -150, 270, 270)).toBe(1600);
    let last = 1800;
    for (let r = TOWN_FROM; r <= TOWN_FULL; r += 5) {
      const f = townFar(r, 1800, ex, ey, ez, x0, z0, x1, z1);
      expect(f).toBeGreaterThanOrEqual(last);
      last = f;
    }
  });
});

describe('the town view: flights', () => {
  const fly = (f: Flight, dt = 1 / 60) => {
    const out = { x: 0, z: 0, radius: 0 };
    const path: { x: number; z: number; radius: number }[] = [];
    while (f.step(dt, out)) path.push({ ...out });
    return path;
  };

  test('a flight inside the play range is the one it always was', () => {
    const f = new Flight();
    f.start(10, 10, 30, 310, 10, 40);
    const path = fly(f);
    // (0.6 s and a 400th of a second a metre; up and over by a third of the way at most, 60 m.)
    expect(path.length).toBe(Math.round((0.6 + 300 / 400) * 60));
    const mid = path[Math.floor(path.length / 2)];
    expect(mid.radius).toBeGreaterThan(80);
    expect(path[path.length - 1]).toEqual({ x: 310, z: 10, radius: 40 });
  });

  test('down from the town view: the way there leads, the zoom lands on it, and the end is exact', () => {
    const f = new Flight();
    f.start(512, 512, 600, 212, 512, 24);
    const path = fly(f);
    const secs = path.length / 60;
    expect(secs).toBeGreaterThan(2.2);
    expect(secs).toBeLessThanOrEqual(3.6);
    expect(path[path.length - 1]).toEqual({ x: 212, z: 512, radius: 24 });
    let last = { x: 512, radius: 600 };
    for (const p of path) {
      // Always on towards the place and (already above cruising height) always down, by a small share a frame.
      expect(p.x).toBeLessThanOrEqual(last.x + 1e-9);
      expect(p.radius).toBeLessThanOrEqual(last.radius + 1e-9);
      expect(last.radius / p.radius).toBeLessThan(1.08);
      last = p;
    }
    // Most of the way is done while the camera is still high: the last stretch is the zoom alone.
    const arrived = path.findIndex((p) => Math.abs(p.x - 212) < 3);
    expect(path[arrived].radius).toBeGreaterThan(24 * 3);
  });

  test('up into the town view: the zoom leads', () => {
    const f = new Flight();
    f.start(100, 100, 30, 700, 700, 500);
    const path = fly(f);
    expect(path[path.length - 1]).toEqual({ x: 700, z: 700, radius: 500 });
    const half = path.findIndex((p) => p.x >= 400);
    expect(path[half].radius).toBeGreaterThan(200);
  });

  test('a long way from low down rises to cruise before it comes down', () => {
    const f = new Flight();
    f.start(100, 100, 130, 900, 100, 40);
    const path = fly(f);
    expect(Math.max(...path.map((p) => p.radius))).toBeGreaterThan(250);
    expect(path[path.length - 1]).toEqual({ x: 900, z: 100, radius: 40 });
  });

  test('a flight lands on someone who walked on meanwhile', () => {
    const f = new Flight();
    f.start(512, 512, 500, 300, 300, 24);
    const out = { x: 0, z: 0, radius: 0 };
    let x = 300;
    let lastX = 512;
    while (f.active) {
      x += 0.05;
      f.aim(x, 300);
      f.step(1 / 60, out);
      expect(Math.abs(out.x - lastX)).toBeLessThan(8);
      lastX = out.x;
    }
    expect(out.x).toBeCloseTo(x);
    expect(out.radius).toBe(24);
  });

  test('reduce motion: there at once', () => {
    const f = new Flight();
    f.start(512, 512, 700, 100, 100, 24, true);
    const out = { x: 0, z: 0, radius: 0 };
    expect(f.step(1 / 60, out)).toBe(true);
    expect(out).toEqual({ x: 100, z: 100, radius: 24 });
    expect(f.active).toBe(false);
  });

  test('where a flight to a town point ends when no radius is asked for', () => {
    // To a place: near enough to see it, the camera's own distance when that is near already.
    expect(townArrival(600, false)).toBe(40);
    expect(townArrival(30, false)).toBe(30);
    expect(townArrival(10, false)).toBe(24);
    // To a resident: the distance a game opens at, or nearer if the camera is.
    expect(townArrival(600, true)).toBe(24);
    expect(townArrival(12, true)).toBe(12);
  });
});

describe('the town view: the roaming focus', () => {
  const plot: FocusPlot = { id: 7, x: 500, z: 500, w: 20, d: 20 };
  const other: FocusPlot = { id: 9, x: 700, z: 500, w: 20, d: 20 };
  const at = (x: number, z: number) => [plot, other].find((p) => x >= p.x && x < p.x + p.w && z >= p.z && z < p.z + p.d) ?? null;
  const cam = (x: number, radius: number) => ({ x, z: 510, radius, alpha: 0, fovH: 1.1 });

  test('above TOWN_FROM no lot becomes the focus: the one there was stays', () => {
    const roam = new Roam();
    const taken: number[] = [];
    const take = (p: FocusPlot) => (taken.push(p.id), true);
    roam.update(cam(510, 30), 0, at, take);
    roam.update(cam(510, 30), SETTLE_MS + 20, at, take);
    expect(taken).toEqual([7]);
    // Up in the town view, settled over another lot for seconds.
    for (let t = 1000; t < 6000; t += 100) roam.update(cam(710, 400), t, at, take);
    expect(taken).toEqual([7]);
    expect(roam.focus).toBe(7);
    // Back down over it: it settles as ever.
    roam.update(cam(710, 60), 6100, at, take);
    roam.update(cam(710, 60), 6200 + SETTLE_MS, at, take);
    expect(taken).toEqual([7, 9]);
  });

  test('the stream of chunks stays the size it has at the play camera’s farthest', () => {
    const low = new Roam();
    const high = new Roam();
    low.update(cam(512, 180), 0, at, () => true);
    high.update(cam(512, TOWN_MAX), 0, at, () => true);
    expect(high.chunks.size).toBe(low.chunks.size);
  });
});
