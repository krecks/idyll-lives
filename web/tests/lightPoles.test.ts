/**
 * Traffic light poles (`WorldStructure.lightPoles`, sim-core `traffic::SignalArm::pole`): the kerbside furniture
 * (streetlights, mailboxes, hydrants, bins, benches, signs) keeps clear of them along the kerb, `POLE_CLEAR` either
 * side; without them it stands where it always did.
 */
import { describe, expect, test } from 'bun:test';
import { streetOptions as options, streetTown } from './support/streetWorld';
import type { WorldStructure } from '../src/core/protocol';
import type { StreetPlan } from '../src/render/shared/street';

/** The street: along x, its middle at z = MID, 7 m wide. */
const MID = 23;
const WIDTH = 7;
/** Where its kerbs are from its middle (m: 4.3 m between them), and its pavements out to its edges. */
const KERB = 2.15;

/** Two houses each side of the street; `poles`: the lights' poles on its kerbs. */
const town = (poles: [number, number, number][]): WorldStructure => streetTown(MID, WIDTH, poles.length ? { lightPoles: poles } : {});

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
    const poles: [number, number, number][] = [north, south].filter((s) => s.length).map((s) => [s[0][0], s[0][1] < MID ? MID - KERB - 0.4 : MID + KERB + 0.4, 0]);
    const lit = streetlights(await planStreet(new HousePlanner(), town(poles), null, options));
    expect(lit.length).toBeGreaterThan(0);
    for (const [px, pz] of poles) {
      const side = Math.sign(pz - MID);
      const near = lit.filter(([x, z]) => Math.sign(z - MID) === side && Math.abs(x - px) < POLE_CLEAR);
      expect(near).toEqual([]);
    }
});
});
