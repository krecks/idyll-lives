/**
 * Traffic light poles (`WorldStructure.lightPoles`, sim-core `traffic::SignalArm::pole`): the kerbside furniture
 * (streetlights, mailboxes, hydrants, bins, benches, signs) keeps clear of them along the kerb, `POLE_CLEAR` either
 * side; without them it stands where it always did.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, WorldStructure } from '../src/core/protocol';
import type { StreetPlan } from '../src/render/shared/street';

/** The street: along x, its middle at z = MID, 7 m wide. */
const MID = 23;
const WIDTH = 7;
/** Where its kerbs are from its middle (m: 4.3 m between them), and its pavements out to its edges. */
const KERB = 2.15;

/** Two houses each side of the street; `poles`: the lights' poles on its kerbs. */
function town(poles: [number, number][]): WorldStructure {
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
    ...(poles.length ? { lightPoles: poles } : {}),
  } as unknown as WorldStructure;
}

const options = { has: () => true, entry: () => undefined, lived: false };

/** Where the plan's streetlights stand on the street's pavements (their glow; not the lots' gardens past them). */
function streetlights(plan: StreetPlan): [number, number][] {
  const out: [number, number][] = [];
  for (let i = 0; i < plan.glow.pos.length; i += 3) {
    const [x, z] = [plan.glow.pos[i], plan.glow.pos[i + 2]];
    const off = Math.abs(z - MID);
    if (x >= 0.5 && x <= 37.5 && off > KERB && off < WIDTH / 2) out.push([x, z]);
  }
  return out;
}

describe('kerbside furniture by traffic light poles', async () => {
  // The renderer's modules read the page's address (and its frames' timing) as they load.
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { planStreet, POLE_CLEAR } = await import('../src/render/shared/street');
  test('stands clear of the poles along the kerb', async () => {
    const plain = streetlights(await planStreet(new HousePlanner(), town([]), null, options));
    expect(plain.length).toBeGreaterThan(0);
    // (A pole where a streetlight stood, on each side's kerb: it stands somewhere else.)
    const north = plain.filter(([, z]) => z < MID);
    const south = plain.filter(([, z]) => z > MID);
    const poles: [number, number][] = [north, south].filter((s) => s.length).map((s) => [s[0][0], s[0][1] < MID ? MID - KERB - 0.4 : MID + KERB + 0.4]);
    const lit = streetlights(await planStreet(new HousePlanner(), town(poles), null, options));
    expect(lit.length).toBeGreaterThan(0);
    for (const [px, pz] of poles) {
      const side = Math.sign(pz - MID);
      const near = lit.filter(([x, z]) => Math.sign(z - MID) === side && Math.abs(x - px) < POLE_CLEAR);
      expect(near).toEqual([]);
    }
});
});
