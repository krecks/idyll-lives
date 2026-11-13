// Footsteps (`audio/steps.ts`, `audio/footsteps.ts`, `audio/surfaces.ts`): a step is the same every time, each
// surface its own (carpet dull, tiles bright), bare feet duller than shoes; a walker's steps start as its heels
// strike, one a strike, however the looks at its stride fall; and what is underfoot is read from the world as the
// house and the street draw it.
import { describe, expect, test } from 'bun:test';
import { STEPS } from '../src/audio/catalog';
import { nextStrike, stepDue, type Walker } from '../src/audio/footsteps';
import { FOOTWEAR, stepSamples, STEP_VERSIONS, SURFACES, type Footwear, type Surface } from '../src/audio/steps';
import { Ground, type GroundWorld } from '../src/audio/surfaces';

const RATE = 48000;

/** How bright a sound is: the root mean square of its sample-to-sample change over that of its samples, as a frequency (Hz). */
function brightness(d: Float32Array): number {
  let change = 0;
  let level = 0;
  for (let i = 1; i < d.length; i++) {
    change += (d[i] - d[i - 1]) ** 2;
    level += d[i] ** 2;
  }
  return (Math.sqrt(change / level) * RATE) / (2 * Math.PI);
}

const bright = (surface: Surface, wear: Footwear) => {
  let sum = 0;
  for (let v = 0; v < STEP_VERSIONS; v++) sum += brightness(stepSamples(surface, wear, v, RATE));
  return sum / STEP_VERSIONS;
};

describe('a step', () => {
  test('is whole, short, starts and ends at nothing, and is the same every time', () => {
    for (const surface of SURFACES) {
      for (const wear of FOOTWEAR) {
        for (let v = 0; v < STEP_VERSIONS; v++) {
          const d = stepSamples(surface, wear, v, RATE);
          expect(stepSamples(surface, wear, v, RATE)).toEqual(d);
          expect(d.length / RATE).toBeLessThanOrEqual(0.3);
          let peak = 0;
          let sum = 0;
          for (const x of d) {
            peak = Math.max(peak, Math.abs(x));
            sum += x * x;
          }
          expect(Number.isFinite(sum)).toBe(true);
          expect(peak).toBeCloseTo(1, 5);
          expect(Math.abs(d[0])).toBeLessThan(0.01);
          expect(Math.abs(d[d.length - 1])).toBeLessThan(0.001);
          // Most of it is over in the first 0.2 s.
          let late = 0;
          for (let i = Math.round(0.2 * RATE); i < d.length; i++) late += d[i] * d[i];
          expect(late / sum).toBeLessThan(0.02);
        }
        // The versions differ.
        expect(stepSamples(surface, wear, 0, RATE)).not.toEqual(stepSamples(surface, wear, 1, RATE));
      }
    }
  });

  test('each surface sounds its own: carpet dull, a board a knock, paving gritty, tiles a click, grass a rustle', () => {
    const shoe = Object.fromEntries(SURFACES.map((s) => [s, bright(s, 'shoe')])) as Record<Surface, number>;
    expect(shoe.carpet).toBeLessThan(400);
    expect(shoe.wood).toBeGreaterThan(shoe.carpet * 2);
    expect(shoe.paving).toBeGreaterThan(shoe.wood * 2);
    expect(shoe.tile).toBeGreaterThan(shoe.paving);
    expect(shoe.grass).toBeGreaterThan(3000);
    // Bare feet have no sole to click: duller on every hard floor.
    for (const s of ['wood', 'tile', 'paving'] as const) expect(bright(s, 'bare')).toBeLessThan(shoe[s] * 0.6);
  });

  test('levels: tiles in shoes the loudest, bare feet under shoes everywhere', () => {
    for (const s of SURFACES) {
      expect(STEPS.level[s].bare).toBeLessThan(STEPS.level[s].shoe);
      expect(STEPS.level[s].shoe).toBeLessThanOrEqual(STEPS.level.tile.shoe);
    }
  });
});

describe('a walker\'s steps', () => {
  test('the next heel strike: the right at half the stride, the left at its end', () => {
    expect(nextStrike(0.4, 2, 0.18)).toEqual({ in: expect.closeTo(0.05, 9), foot: 1 });
    expect(nextStrike(0.9, 2, 0.18)).toEqual({ in: expect.closeTo(0.05, 9), foot: 0 });
    expect(nextStrike(0.1, 2, 0.18)).toBe(null);
    expect(nextStrike(0.4, 0, 0.18)).toBe(null);
    expect(nextStrike(NaN, 2, 0.18)).toBe(null);
  });

  // A stride of `seconds` a cycle looked at every 100 to 135 ms (as frames fall) for `span` s: the steps started.
  const walk = (seconds: number, span: number, start = 0.13) => {
    const steps: { when: number; foot: number }[] = [];
    let seed = 7;
    const r = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
    const phase = (t: number) => (start + t / seconds) % 1;
    const w: Walker = { stepped: -1, seen: 0 };
    for (let t = 0.1; t < span; t += 0.1 + 0.035 * r()) {
      const due = stepDue(w, phase(t), 1 / seconds, t);
      if (due) steps.push(due);
    }
    return steps;
  };

  test('at a walk: one step a heel strike, each at the moment the heel lands, left and right in turn', () => {
    const seconds = 1.1;
    const steps = walk(seconds, 11);
    // Two steps a cycle.
    expect(steps.length).toBeGreaterThanOrEqual(19);
    expect(steps.length).toBeLessThanOrEqual(20);
    for (const [i, s] of steps.entries()) {
      // The stride's phase at the step's time is 0 (left) or 0.5 (right), to a millisecond.
      const at = (0.13 + s.when / seconds) % 1;
      const off = Math.min(Math.abs(at - 0.5), at, 1 - at) * seconds;
      expect(off).toBeLessThan(0.001);
      expect(s.foot).toBe(Math.abs(at - 0.5) < 0.01 ? 1 : 0);
      if (i) {
        expect(s.when - steps[i - 1].when).toBeCloseTo(seconds / 2, 3);
        expect(s.foot).not.toBe(steps[i - 1].foot);
      }
    }
  });

  test('at a run: every strike still, and never closer than a rattle', () => {
    const steps = walk(0.5, 6);
    expect(steps.length).toBeGreaterThanOrEqual(21);
    for (let i = 1; i < steps.length; i++) expect(steps[i].when - steps[i - 1].when).toBeGreaterThanOrEqual(STEPS.apart);
    // A sprint in a sped-up game (five cycles a second): its steps are thinned to that spacing, not played ten a second.
    const sprint = walk(0.2, 4);
    for (let i = 1; i < sprint.length; i++) expect(sprint[i].when - sprint[i - 1].when).toBeGreaterThanOrEqual(STEPS.apart);
    expect(sprint.length).toBeLessThanOrEqual(Math.ceil(4 / STEPS.apart));
  });
});

describe('what is underfoot', () => {
  // A town 10 wide, the ground 6 rows and a storey above: a bathroom (1), a bedroom (2), a hall (3), a room upstairs (4);
  // a path along row 5, a patio tile, and a road off the map's left edge.
  const W = 10;
  const rooms = new Uint16Array(W * 12);
  const fill = (id: number, x0: number, z0: number, x1: number, z1: number) => {
    for (let z = z0; z < z1; z++) for (let x = x0; x < x1; x++) rooms[z * W + x] = id;
  };
  fill(1, 0, 0, 2, 2);
  fill(2, 2, 0, 5, 3);
  fill(3, 5, 0, 7, 3);
  fill(4, 0, 6, 5, 9);
  const thing = (def: string, x: number, z: number) => ({ id: x * 100 + z, def, x, z, w: 1, d: 1, rot: 0 });
  const world = {
    width: W,
    depth: 12,
    storeyDepth: 6,
    rooms,
    floors: [[6, 1, 9], [8, 4, 13], [2, 7, 5]] as [number, number, number][],
    objects: [thing('toilet', 0, 0), thing('bed', 3, 1)],
    meta: { paths: [{ x: 0, z: 5, w: 10, d: 1 }], roads: [{ points: [[20, -30], [20, 40]], width: 6, ends: ['out', 'out'] }] },
  } as unknown as GroundWorld;
  // The content's floor coverings, as `base.json` orders them (a tile's covering is an index + 1).
  const finishes = ['wood', 'wood', 'wood', 'whitewash', 'tile', 'tile', 'tile', 'tile', 'carpet', 'carpet', 'carpet', 'carpet', 'stone'];

  test('a room\'s own floor, a covering laid over it, a path, a patio, the lawn', () => {
    const g = new Ground(finishes);
    expect(g.at(1, 1)).toBe('grass');
    g.setWorld(world);
    expect(g.at(0.5, 1.5)).toBe('tile'); // the bathroom
    expect(g.at(3, 2)).toBe('carpet'); // the bedroom
    expect(g.at(5.5, 1)).toBe('wood'); // the hall: nothing says what it is for
    expect(g.at(6.5, 1.5)).toBe('carpet'); // a carpet laid in the hall
    expect(g.at(2.5, 7.5)).toBe('tile'); // tiles laid upstairs
    expect(g.at(1, 7)).toBe('wood'); // upstairs otherwise
    expect(g.at(8, 10)).toBe('wood'); // outdoors upstairs: a deck
    expect(g.at(8.5, 4.5)).toBe('paving'); // a stone patio
    expect(g.at(3, 5.5)).toBe('paving'); // the path
    expect(g.at(8, 2)).toBe('grass');
    expect(g.at(-3, 2)).toBe('grass');
    expect(g.at(3, 400)).toBe('grass');
  });

  test('it follows the world: a bed moved out, the room is a wooden floor again', () => {
    const g = new Ground(finishes);
    g.setWorld(world);
    expect(g.at(3, 2)).toBe('carpet');
    g.setWorld({ ...world, objects: [thing('toilet', 0, 0)] } as GroundWorld);
    expect(g.at(3, 2)).toBe('wood');
    expect(g.at(0.5, 1.5)).toBe('tile');
  });
});
