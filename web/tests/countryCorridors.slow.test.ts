// Country roads that follow the same way share it (`game/macro.ts` `alongExisting`): one laid
// along a road there already is laid only up to it and on from it, joining it in a T, rather than
// the two running side by side a metre or a few apart, where cars on them meet nose to nose with
// the other on their right and, each pulling over to its right, lock for good (`roads.ts`
// `sideBySide`). From the big worlds the traffic checks locked up in: Oakview (seed base 0, CI's:
// roads 48 and 53 for 240 m, 1.5–6 m apart) and Elder Green (seed base 12: two roads off the
// same port's T for 160 m, 0.2–2 m apart, then a jam there). And from the big worlds the world
// sampler found laid side by side (seed bases 16, 17, 21 and 22): Eaglescar (a road off another
// from a port, out along that port's high street for 74 m: the street was left out as the port's
// own), Heronmere and Barleyham (a port right by a road that the way out of it runs along a cell
// or more on: the stub out of the port and back onto it wound, and the road was laid whole beside
// it), Rowanhurst (a road off another that looped round and back along it) and Willow End (leaving
// a road in a T facing the side the way crosses it from, not the side it goes on to: a hairpin).
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { RoadPlan } from '../src/core/protocol';
import { sideBySide } from '../src/game/roads';
import { CLASSIC } from './support/classicWorlds';

/** How long a country road may run alongside another, a little apart, at most (m): where one joins another at a slant, or leaves a port beside a street's end. */
const ALONGSIDE = 40;

describe('country roads', async () => {
  usePublicFetch();
  const { useTownContent } = await import('../src/game/town');
  const { generateFromSpec } = await import('../src/game/macro');
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  /** The stretches longer than `ALONGSIDE` where a country road runs alongside another road a little apart. */
  const alongside = (roads: readonly RoadPlan[]) =>
    sideBySide(roads, ALONGSIDE, 'country').map(({ a, b, length, at }) => `${roads[a].name} and ${roads[b].name}: ${length} m by ${at.map(Math.round).join(', ')}`);

  for (const [kind, seed, name] of [
    ['medium', 104735, 'Oakview'],
    ['small', 199762, 'Elder Green'],
    ['mountain', 231441, 'Eaglescar'],
    ['lakeside', 239360, 'Heronmere'],
    ['farm', 271032, 'Barleyham'],
    ['small', 278952, 'Willow End'],
    ['forest', 278953, 'Rowanhurst'],
  ] as const) {
    test(
      `big ${kind} ${seed} (${name}): no country road runs alongside another a few metres off`,
      () => {
        const town = generateFromSpec(content, t, CLASSIC[kind], seed);
        expect(town.name).toBe(name);
        expect(alongside(town.roads)).toEqual([]);
      },
      60_000,
    );
  }

  test('two roads side by side a few metres apart are found; on one middle line, or apart, not', () => {
    const line = (z: number, x0 = 0, x1 = 200): RoadPlan => ({ points: [[x0, z], [x1, z]], width: 7, ends: ['join', 'join'], kind: 'country' });
    expect(sideBySide([line(0), line(3)], 0).map((s) => s.length)).toEqual([198]);
    expect(sideBySide([line(0), line(0, 50, 150)], 0)).toEqual([]);
    expect(sideBySide([line(0), line(9)], 0)).toEqual([]);
    // (Square across: a junction, not alongside.)
    const across: RoadPlan = { points: [[100, -3], [100, 50]], width: 7, ends: ['join', 'join'], kind: 'country' };
    expect(sideBySide([line(0), across], 6)).toEqual([]);
  });
});
