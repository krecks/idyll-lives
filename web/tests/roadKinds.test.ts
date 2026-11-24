/**
 * The kinds of road (`core/roadKinds.ts` `ROAD_KIND`): the kinds that were read from it as they were (`GRADE`,
 * `hasSidewalks`, `carriageway`, `streetWidth`), and what the new ones are.
 */
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { kindInfo, ROAD_KIND, ROAD_KINDS } from '../src/core/roadKinds';
import { ROAD_WIDTH } from '../src/game/countryRouter';
import { streetWidth } from '../src/game/edit/streets';
import { TRAIL_WIDTH } from '../src/game/macro';
import { LANE_WIDTH, MAIN_WIDTH, SIDEWALK } from '../src/game/roads';
import { GRADE } from '../src/render/shared/builtSurface';
import { carriageway, hasSidewalks } from '../src/render/shared/junctions';

describe('ROAD_KIND', () => {
  test('the kinds that were keep their widths, sidewalks and grades', () => {
    expect([ROAD_KIND.street.width, ROAD_KIND.lane.width, ROAD_KIND.country.width, ROAD_KIND.trail.width]).toEqual([MAIN_WIDTH, LANE_WIDTH, ROAD_WIDTH, TRAIL_WIDTH]);
    expect([ROAD_KIND.street.sidewalk, ROAD_KIND.lane.sidewalk, ROAD_KIND.country.sidewalk, ROAD_KIND.trail.sidewalk]).toEqual([SIDEWALK, SIDEWALK, 0, 0]);
    expect({ street: GRADE.street, lane: GRADE.lane, country: GRADE.country, trail: GRADE.trail }).toEqual({ street: 0.08, lane: 0.08, country: 0.12, trail: 0.22 });
    // (A drawn lane is a lane's 6 m, narrower than a street.)
    expect(ROAD_KINDS.filter((k) => !ROAD_KIND[k].separated).map((k) => [k, streetWidth(k)])).toEqual([
      ['street', 7],
      ['lane', 6],
      ['country', 7],
      ['trail', 2],
      ['avenue', 14],
    ]);
  });

  test('sidewalks and carriageways as they were', () => {
    const road = (kind: RoadPlan['kind'], width: number): RoadPlan => ({ points: [], width, ends: ['out', 'out'], kind });
    expect(hasSidewalks(undefined)).toBe(false);
    expect([road(undefined, 7), road('street', 7), road('lane', 6), road('country', 7), road('trail', 2)].map((r) => [hasSidewalks(r), carriageway(r)])).toEqual([
      [true, 7 - 2 * SIDEWALK],
      [true, 7 - 2 * SIDEWALK],
      [true, 6 - 2 * SIDEWALK],
      [false, 7],
      [false, 2],
    ]);
  });

  test('the new kinds', () => {
    const { avenue, motorway, ramp } = ROAD_KIND;
    // An avenue: two lanes each way about its median, between sidewalks.
    expect([avenue.width, avenue.median, avenue.lanesEachWay, avenue.separated]).toEqual([14, 2.6, 2, false]);
    // A motorway's carriageway: two 3.5 m lanes and a shoulder, one-way, cars only.
    expect(motorway.width).toBeGreaterThanOrEqual(2 * 3.5 + 1);
    expect([motorway.lanesEachWay, motorway.separated, motorway.sidewalk]).toEqual([2, true, 0]);
    expect([ramp.width, ramp.lanesEachWay, ramp.separated]).toEqual([5, 1, true]);
    expect(kindInfo({})).toBe(ROAD_KIND.street);
  });
});
