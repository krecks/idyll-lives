// game/pmath.ts: the generator's math, the same to the last bit on every machine and engine (its values pinned
// here, from macOS's Bun, held on Linux's in CI), and within an ulp or so of the platform's own `Math`.
import { expect, test } from 'bun:test';
import * as p from '../src/game/pmath';

test('the same values to the last bit everywhere', () => {
  const xs = [0.1, 0.7853981633974483, 1, 2.5, -3.7, 100.25, 12345.678];
  expect(xs.map(p.sin)).toEqual([0.09983341664682815,0.7071067811865475,0.8414709848078965,0.5984721441039564,0.5298361409084934,-0.27728285645485135,-0.7040813137533816]);
  expect(xs.map(p.cos)).toEqual([0.9950041652780258,0.7071067811865476,0.5403023058681398,-0.8011436155469337,-0.848100031710408,0.9607883312760612,0.7101193587160628]);
  expect(xs.map((x) => p.atan2(x, 1.3 - x))).toEqual([0.08314123188844123,0.9907662699975697,1.2793395323170293,2.0183163019520665,-0.6370703292756835,2.34966847842132,2.3561418374159073]);
  expect([0.1, -0.6, 0.9].map(p.acos)).toEqual([1.4706289056333368,2.214297435588181,0.45102681179626236]);
  expect([0.1, -0.6, 0.98].map(p.asin)).toEqual([0.1001674211615598,-0.6435011087932844,1.3704614844717768]);
  expect([0.1, -2.5, 7.25].map(p.exp)).toEqual([1.1051709180756477,0.0820849986238988,1408.1048482046956]);
  expect([0.1, 2.5, 1e6].map(p.log)).toEqual([-2.3025850929940455,0.9162907318741551,13.815510557964274]);
  expect(([[0.3, 1.4], [2.5, 3], [0.7, 1.5]] as const).map(([a, b]) => p.pow(a, b))).toEqual([0.18534025517022357,15.625,0.5856620185738528]);
  expect([27, -2.5, 1e-3].map(p.cbrt)).toEqual([3,-1.3572088082974534,0.1]);
  expect([p.hypot(3, 4), p.hypot(0.1, 0.2)]).toEqual([5,0.223606797749979]);
});

test("close to the platform's Math, and its edge cases", () => {
  let seed = 12345;
  const rnd = () => (seed = (Math.imul(seed, 1103515245) + 12345) >>> 0) / 4294967296;
  const ulps = (a: number, b: number) => (Object.is(a, b) ? 0 : Math.abs(a - b) / Math.max(Number.EPSILON * Math.abs(b), Number.MIN_VALUE));
  let worst = 0;
  let worstPow = 0;
  for (let i = 0; i < 20000; i++) {
    const x = (rnd() * 2 - 1) * [1, 10, 1000, 1e6][i % 4];
    const y = (rnd() * 2 - 1) * 100;
    const u = rnd() * 2 - 1;
    worst = Math.max(
      worst,
      ulps(p.sin(x), Math.sin(x)),
      ulps(p.cos(x), Math.cos(x)),
      ulps(p.atan(x), Math.atan(x)),
      ulps(p.atan2(y, x), Math.atan2(y, x)),
      ulps(p.hypot(x, y), Math.hypot(x, y)),
      ulps(p.acos(u), Math.acos(u)),
      ulps(p.asin(u), Math.asin(u)),
      ulps(p.exp(y), Math.exp(y)),
      ulps(p.log(Math.abs(x)), Math.log(Math.abs(x))),
      ulps(p.cbrt(x), Math.cbrt(x)),
    );
    const b = rnd();
    worstPow = Math.max(worstPow, ulps(p.pow(b, 1.4), Math.pow(b, 1.4)));
  }
  expect(worst).toBeLessThan(2.5);
  for (const [a, b] of [[0, -1], [-0, -1], [1, 0], [0, 0], [-0, 0], [-1, 0], [Infinity, Infinity], [-Infinity, -Infinity], [1, -Infinity]]) expect(p.atan2(a, b)).toBe(Math.atan2(a, b));
  expect([p.acos(1), p.acos(-1), p.asin(1), p.exp(0), p.log(1), p.pow(0.5, 0), p.cbrt(-8), p.sin(0), p.cos(0)]).toEqual([0, Math.PI, Math.PI / 2, 1, 0, 1, -2, 0, 1]);
  expect([p.sin(Infinity), p.cos(NaN), p.acos(1.5), p.log(-1), p.pow(-2, 0.5)].every(Number.isNaN)).toBe(true);
  expect(worstPow).toBeLessThan(16);
});
