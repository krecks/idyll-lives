// No country road leaves a street's end that another area's street comes up to (`macro.ts`
// `continued`): the two meet end to end, one street on through, and a country road from there ran
// on along the other street a few metres off before turning away across it, a junction knotted
// into two a few metres apart (Canalside's Queen'ston Lane, from Foundry Street's end by Union
// Street, the dense world of the play-test, meta.seed 808114082). Nor a T off one of the streets there
// (`sideT`): since the motorway went down before the villages (2026-11-24) Canalside's villages are others,
// and in Station Square (dense, seed 14) Station Square Lane came into a street where another village's
// street's end comes up to it; King's Row (dense, seed 45) has two such streets.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import { nearest } from '../src/game/roads';
import { CLASSIC } from './support/classicWorlds';

describe('country roads by streets meeting end to end', async () => {
  usePublicFetch();
  const { useTownContent } = await import('../src/game/town');
  const { generateFromSpec } = await import('../src/game/macro');
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  test(
    "big dense 45 (King's Row): the two streets meet, and no country road leaves where they do",
    () => {
      const town = generateFromSpec(content, t, CLASSIC.dense, 45);
      expect(town.name).toBe("King's Row");
      const streets = town.roads.filter((r) => r.kind !== 'country' && r.kind !== 'trail' && r.area !== undefined);
      // (Each pair of other areas' streets whose ends meet: no country road ends there.)
      const met: [number, number][] = [];
      for (const a of streets) {
        for (const b of streets) {
          if (a === b || a.area === b.area) continue;
          for (const p of [a.points[0], a.points[a.points.length - 1]]) if (nearest(b.points, p[0], p[1]).dist <= Math.max(a.width, b.width) / 2 + 1) met.push(p);
        }
      }
      expect(met.some(([x, z]) => Math.hypot(x - 281, z - 419) < 2)).toBe(true);
      const country = town.roads.filter((r) => r.kind === 'country');
      const at = country.flatMap((r) => [r.points[0], r.points[r.points.length - 1]].filter(([x, z]) => met.some(([mx, mz]) => Math.hypot(x - mx, z - mz) < 5)).map(() => r.name));
      expect(at).toEqual([]);
    },
    60_000,
  );
});
