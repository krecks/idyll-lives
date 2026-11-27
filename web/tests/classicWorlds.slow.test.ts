// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15), re-pinned when the town's named
// places became lots of their own (`macro.ts` `namedLots`, 2026-11-17), when a village's hall
// got a look of its own (`Civic.kinds`, 2026-11-18), and for the real town (2026-11-20: every building a
// lot, laid by the dense pass, with buildings of flats, car parks and bays; the country road's corner rule;
// 679 family names), and when the generator's math became the same on every machine and engine (2026-11-21:
// `game/pmath.ts` for `Math.sin`, `Math.hypot` and the like, whose last bits Bun takes from the system's libm, so
// Linux's made other towns than macOS's), and when a road whose name and its lane's were taken got the street's other
// words before a number (2026-11-24: "Elmfield Way", not "Elmfield Lane 21"). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences), and when a lane became a lane's 6 m (2026-11-23: every area road but its first, the
// lots along it half a metre nearer its middle), and for the motorway (2026-11-23: one in every world where it fits,
// edge to edge past the main town, the country roads crossing under it at its gates; in the others, the note why
// not in their diagnostics), for traffic on the right as the game shows it (2026-11-24: each motorway's
// carriageways on the other sides of its middle line, its ramps with them), and when the motorway went down before
// the villages (2026-11-24: its course chosen with the main town alone, none costing a village; the villages kept
// clear of it, its gates chosen by them; no side T where another area's street comes up end to end), and when 70 careers'
// hours moved a quarter or half an hour (2026-11-27: the jobs' starts spread across the morning). A change that
// moves one on purpose re-pins it here.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import { createHash } from 'node:crypto';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, 'ad7e1aeae3b5d44d'],
  ['small', 22, '5e9d475fbf39a8d9'],
  ['medium', 11, 'caa30a5ab9c106fb'],
  ['medium', 22, 'e9a365b546449ecf'],
  ['dense', 11, '2f40d45a08b1c156'],
  ['dense', 22, '536eaf8bdad6b315'],
  ['farm', 11, '98b8b6e2cfbb8134'],
  ['farm', 22, 'ee22b1dc3b0dec69'],
  ['seaside', 11, '4b53c8bc5ad81378'],
  ['seaside', 22, 'd63bdc3650fa89a2'],
  ['mountain', 11, '7be313904fff3377'],
  ['mountain', 22, '6e1ec82c2cd0b591'],
  ['lakeside', 11, '02414342ce8855bf'],
  ['lakeside', 22, '158fec3671ad17d8'],
  ['forest', 11, '202ec567588e8385'],
  ['forest', 22, '683716bec9dfac10'],
  ['dense', 808114082, '6f15a39acd379243'],
  ['dense', 45, 'b07f18985ff04e7d'],
  ['small', 199762, '53d1404115b4fca6'],
  ['forest', 223520, '8a0082da846d38ad'],
  ['medium', 104735, '5112db7531757ede'],
  ['mountain', 231441, '440da34c5376b012'],
  ['lakeside', 239360, '195159c7796d6c94'],
  ['farm', 271032, '57b1fbcf4a337ef6'],
  ['small', 278952, '42c9fbdd83d24ac0'],
  ['forest', 278953, 'efbeefdcf0cf3485'],
  ['forest', 425059, 'ba5acf0db8a2ca53'],
  ['lakeside', 215603, '3c8671f2f9082669'],
  ['forest', 320330, '5dd92e45a38195b0'],
  ['dense', 320329, '51d0d0ed6b4267d4'],
  ['seaside', 215602, '41c5be8cacda85f9'],
  ['forest', 104735, '0b5dbb8f2623fa4c'],
  ['forest', 112654, 'fceda88fd52e89da'],
  ['lakeside', 225304, '766ed383611cfb30'],
  ['small', 314192, 'c402c860ae43f8cd'],
];

describe('classic worlds', async () => {
  usePublicFetch();
  const { useTownContent } = await import('../src/game/town');
  const { generateFromSpec } = await import('../src/game/macro');
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);
  const hash = (text: string) => createHash('sha256').update(text).digest('hex').slice(0, 16);

  for (const [kind, seed, want] of GOLDEN) {
    test(
      `${kind} ${seed}`,
      () => {
        expect(hash(JSON.stringify(generateFromSpec(content, t, CLASSIC[kind], seed)))).toBe(want);
      },
      60_000,
    );
  }
});
