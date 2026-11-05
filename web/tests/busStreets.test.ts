/**
 * Pull-over streets (`WorldStructure.busStreets`, sim-core `bus::kerbside`, `cars::kerbside`): on a
 * street a bus drives where two buses pass pulled over onto the pavement (7 m), and on every lane
 * too narrow for two cars to pass in their lanes (6 m: they pull over to pass, and a car and a bus
 * where it's straight), the kerbside furniture (streetlights, mailboxes, hydrants, bins, benches,
 * signs) keeps back from the road, no further in than the street's `keep` from its edges: the strip
 * they may pull into is clear of it. Elsewhere it stands where it always did, nearer the kerb.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, WorldStructure } from '../src/core/protocol';
import type { StreetPlan } from '../src/render/shared/street';

/** The street: along x, its middle at z = MID, a town's 7 m main road or a 6 m lane. */
const MID = 23;
/** How far in from its edges the furniture may reach, as the sim has it: a 7 m bus street (`bus::kerbside`), a 6 m lane (`cars::LANE_KERBSIDE`). */
const KEEP: Record<number, number> = { 7: 1.05, 6: 0.5 };

/** Two houses each side of a street `width` m wide; `pull`: the sim lists it (`busStreets`). */
function town(width: number, pull: boolean): WorldStructure {
  const [north, south] = [MID - width / 2, MID + width / 2];
  const lots = [
    { x: 0, w: 20, z: 0, d: north, door: 8, facing: 2 },
    { x: 20, w: 18, z: 0, d: north, door: 28, facing: 2 },
    { x: 0, w: 20, z: south, d: 46 - south, door: 9, facing: 0 },
    { x: 20, w: 18, z: south, d: 46 - south, door: 27, facing: 0 },
  ];
  const plots: PlotInfo[] = lots.map((l, id) => ({
    id,
    name: `${id}`,
    x: l.x,
    z: l.z,
    w: l.w,
    d: l.d,
    public: false,
    entry: [l.door + 0.5, l.facing === 2 ? north - 1 : south + 1],
    house: l.facing === 2 ? [l.x + 3, 5, l.x + 14, 13] : [l.x + 4, south + 5, l.x + 15, south + 13],
  }));
  const paths = lots.map((l) => (l.facing === 2 ? { x: l.door, z: 13, w: 1, d: north - 13 } : { x: l.door, z: south, w: 1, d: 5 }));
  return {
    width: 38,
    depth: 46,
    objects: [],
    plots,
    rooms: new Array(38 * 46).fill(0),
    openings: [],
    walls: [],
    households: [],
    sims: [],
    exits: [],
    meta: {
      kind: 'town',
      name: 'Test',
      streets: [],
      roads: [{ points: [[0, MID], [38, MID]], width, ends: ['out', 'out'] }],
      paths,
      facing: lots.map((l) => l.facing),
      seed: 1,
    },
    ...(pull ? { busStreets: [{ road: 0, keep: KEEP[width] }] } : {}),
  } as unknown as WorldStructure;
}

const options = { has: () => true, entry: () => undefined, lived: false };

/** How far in from the street's nearer edge (m) each point of the plan's furniture on the street stands, at most. */
function reach(plan: StreetPlan, width: number): number {
  const [north, south] = [MID - width / 2, MID + width / 2];
  let most = 0;
  const at = (x: number, z: number) => {
    // (On the street, kerb to kerb with its pavements; not the lots' gardens either side.)
    if (x < 1 || x > 37 || z <= north || z >= south) return;
    most = Math.max(most, Math.min(z - north, south - z));
  };
  for (const geo of [plan.props, plan.glow]) for (let i = 0; i < geo.pos.length; i += 3) at(geo.pos[i], geo.pos[i + 2]);
  for (const [, items] of plan.batches) for (const [x, z] of items) at(x, z);
  return most;
}

describe('kerbside furniture where buses and cars pull over', async () => {
  // The renderer's modules read the page's address (and its frames' timing) as they load.
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { planStreet } = await import('../src/render/shared/street');
  for (const [width, what] of [
    [7, 'a bus street: out of the strip the buses pull into'],
    [6, 'a 6 m lane: out of the strip the cars pull into to pass'],
  ] as const) {
    test(what, async () => {
      const keep = KEEP[width];
      const [plain, pulled] = [await planStreet(new HousePlanner(), town(width, false), null, options), await planStreet(new HousePlanner(), town(width, true), null, options)];
      // (Furniture there at all, and nearer the road without the list: the test sees what moved.)
      expect(reach(plain, width)).toBeGreaterThan(keep);
      expect(reach(pulled, width)).toBeGreaterThan(0);
      expect(reach(pulled, width)).toBeLessThanOrEqual(keep + 1e-3);
    });
  }
});
