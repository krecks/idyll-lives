/**
 * Bus streets (`WorldStructure.busStreets`, sim-core `bus::kerbside`): on a street a bus drives where
 * two buses pass pulled over onto the pavement, the kerbside furniture (streetlights, mailboxes,
 * hydrants, bins, benches, signs) keeps back from the road, no further in than the street's `keep`
 * from its edges: the strip the buses may pull into is clear of it. Elsewhere it stands where it
 * always did, nearer the kerb.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, WorldStructure } from '../src/core/protocol';
import type { StreetPlan } from '../src/render/shared/street';

/** The street: along x, its middle at z = MID, WIDTH wide (a town's 7 m main road). */
const MID = 23;
const WIDTH = 7;
/** How far in from its edges the furniture may reach on it, as the sim has it for 7 m (`bus::kerbside`). */
const KEEP = 1.05;

/** Two houses each side of the street, `bus`: a bus drives it. */
function town(bus: boolean): WorldStructure {
  const [north, south] = [MID - WIDTH / 2, MID + WIDTH / 2];
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
      roads: [{ points: [[0, MID], [38, MID]], width: WIDTH, ends: ['out', 'out'] }],
      paths,
      facing: lots.map((l) => l.facing),
      seed: 1,
    },
    ...(bus ? { busStreets: [{ road: 0, keep: KEEP }] } : {}),
  } as unknown as WorldStructure;
}

const options = { has: () => true, entry: () => undefined, lived: false };

/** How far in from the street's nearer edge (m) each point of the plan's furniture on the street stands, at most. */
function reach(plan: StreetPlan): number {
  const [north, south] = [MID - WIDTH / 2, MID + WIDTH / 2];
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

describe('kerbside furniture on a bus street', async () => {
  // The renderer's modules read the page's address (and its frames' timing) as they load.
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { planStreet } = await import('../src/render/shared/street');
  test('keeps out of the strip the buses pull into', async () => {
    const [plain, bus] = [await planStreet(new HousePlanner(), town(false), null, options), await planStreet(new HousePlanner(), town(true), null, options)];
    // (Furniture there at all, and nearer the road without the bus: the test sees what moved.)
    expect(reach(plain)).toBeGreaterThan(KEEP);
    expect(reach(bus)).toBeGreaterThan(0);
    expect(reach(bus)).toBeLessThanOrEqual(KEEP + 1e-3);
  });
});
