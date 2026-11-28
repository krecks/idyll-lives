// `game/walk.ts`: WoW keys in game x/z (z is the engine's -z): W ahead, S back, Q/E step left/right, A/D turn.
import { expect, test } from 'bun:test';
import type { SimBridge } from '../src/core/bridge';
import type { Command } from '../src/core/protocol';
import { inFront, moveDir, wallOnLine, WalkMode, type WalkKey } from '../src/game/walk';
import type { Renderer } from '../src/render/types';

const dir = (keys: WalkKey[], heading = 0) => moveDir(new Set(keys), heading);
const near = (v: [number, number] | null, x: number, z: number) => {
  expect(v![0]).toBeCloseTo(x);
  expect(v![1]).toBeCloseTo(z);
};

test('W/S walk along the heading, Q/E across it', () => {
  near(dir(['w']).dir, 1, 0);
  near(dir(['s']).dir, -1, 0);
  near(dir(['e']).dir, 0, -1);
  near(dir(['q']).dir, 0, 1);
  near(dir(['w'], Math.PI / 2).dir, 0, 1);
  near(dir(['w', 'e']).dir, Math.SQRT1_2, -Math.SQRT1_2);
});

test('nothing held, or opposites, stands; A/D only turn', () => {
  expect(dir([]).dir).toBeNull();
  expect(dir(['w', 's']).dir).toBeNull();
  expect(dir(['a'])).toEqual({ dir: null, turn: -1 });
  expect(dir(['d']).turn).toBe(1);
});

test('F reaches whoever is in front, near enough', () => {
  const me = { x: 0, z: 0 };
  expect(inFront(me, 0, 1.5, 0.3, 2.5)).toBe(true);
  expect(inFront(me, 0, 1.5, 1.5, 2.5)).toBe(false); // 45° off
  expect(inFront(me, 0, -1, 0, 2.5)).toBe(false); // behind
  expect(inFront(me, 0, 3, 0, 2.5)).toBe(false); // out of reach
  expect(inFront(me, Math.PI / 2, 0, 2, 2.5)).toBe(true);
});

test('a wall across the way is found, a diagonal only when both ways round are walled', () => {
  // (A wall between x = 1 and x = 2 on row 0.)
  const wall = (ax: number, az: number, bx: number, bz: number) => az === 0 && bz === 0 && Math.min(ax, bx) === 1 && Math.max(ax, bx) === 2;
  expect(wallOnLine(0.5, 0.5, 2.5, 0.5, wall)).toBe(true);
  expect(wallOnLine(2.5, 0.5, 0.5, 0.5, wall)).toBe(true);
  expect(wallOnLine(0.5, 1.5, 2.5, 1.5, wall)).toBe(false);
  expect(wallOnLine(1.5, 0.5, 1.9, 0.9, wall)).toBe(false);
  // (From (1, 0) to (2, 1): round by (1, 1) is open.)
  expect(wallOnLine(1.9, 0.9, 2.1, 1.1, wall)).toBe(false);
});

/** A walk mode over a fake bridge (one resident, id 7, at `pos`; `pos` null: away) with the keys typed at `window`. */
function harness() {
  const g = globalThis as Record<string, unknown>;
  g.window ??= new EventTarget();
  g.requestAnimationFrame = () => 0;
  g.cancelAnimationFrame = () => {};
  for (const n of ['HTMLInputElement', 'HTMLTextAreaElement', 'HTMLSelectElement', 'HTMLElement']) g[n] ??= class {};
  const layout = { header: { simCount: 0 }, headerLen: 1, simStride: 4, sim: { id: 0, x: 1, z: 2, away: 3 } };
  const state = { pos: { x: 5, z: 5 } as { x: number; z: number } | null, here: true, busy: false, sent: [] as Command[], away: [] as boolean[], lost: 0 };
  const bridge = {
    layout,
    latest: () => new Float32Array([1, 7, state.pos?.x ?? 0, state.pos?.z ?? 0, state.pos ? 0 : 1]),
    send: (c: Command) => state.sent.push(c),
  } as unknown as SimBridge;
  const renderer = { setWalkView: () => 0, setWalkLook: () => {} } as unknown as Renderer;
  const walk = new WalkMode({
    bridge,
    renderer,
    useAt: () => {},
    lost: () => state.lost++,
    gone: () => !state.here,
    away: (on) => state.away.push(on),
    busy: () => state.busy,
    fits: () => true,
  });
  walk.enter(7, new EventTarget() as unknown as HTMLElement);
  let now = 1000;
  const tick = () => (walk as unknown as { tick(now: number): void }).tick((now += 200));
  const key = (type: 'keydown' | 'keyup', k: string, shiftKey = false) => window.dispatchEvent(Object.assign(new Event(type), { key: k, shiftKey }));
  const steers = () => state.sent.filter((c) => c.type === 'steer');
  return { walk, state, tick, key, steers };
}

test('a key held while a menu is open waits, and walks once it closes', () => {
  const h = harness();
  h.key('keydown', 'w');
  h.state.busy = true;
  h.tick();
  expect(h.steers()).toHaveLength(0);
  h.state.busy = false;
  h.tick();
  expect(h.steers()).toHaveLength(1);
  h.walk.exit();
});

test('Shift runs', () => {
  const h = harness();
  h.key('keydown', 'w');
  h.tick();
  h.key('keydown', 'Shift', true);
  h.tick();
  expect(h.steers().map((c) => (c as { run?: boolean }).run)).toEqual([false, true]);
  h.walk.exit();
});

test('a refused steer is not sent again until the keys change', () => {
  const h = harness();
  h.key('keydown', 'w');
  h.tick();
  h.walk.refused();
  h.tick();
  h.tick();
  expect(h.steers()).toHaveLength(1);
  h.key('keydown', 'q');
  h.tick();
  expect(h.steers()).toHaveLength(2);
  h.walk.exit();
});

test('away (in a car, at work), walk mode waits and takes up again; gone for good, it ends', () => {
  const h = harness();
  h.state.pos = null;
  h.tick();
  h.tick();
  expect(h.state.away).toEqual([true]);
  expect(h.state.lost).toBe(0);
  h.state.pos = { x: 6, z: 5 };
  h.tick();
  expect(h.state.away).toEqual([true, false]);
  expect(h.state.sent.filter((c) => c.type === 'hold' && c.on)).toHaveLength(2);
  h.state.pos = null;
  h.state.here = false;
  h.tick();
  expect(h.state.lost).toBe(1);
  h.walk.exit();
});
