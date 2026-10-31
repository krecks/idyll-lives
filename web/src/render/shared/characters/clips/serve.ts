/**
 * Buying from a vendor and serving (sim-core `staff.rs`): `buy`, the customer at the front of a cart, a
 * stall or a counter, and `sell`, the vendor behind it, played together from the moment the vendor
 * starts serving (both rows change in the same snapshot) and as long as each other (`SERVE_SECONDS`, a
 * served use's five game minutes at speed 1). In turn (`SERVE`, shares of the clip): the customer
 * orders, pointing at what they want, while the vendor listens and nods; the vendor turns to the
 * goods and reaches down into them (the freezer, the hot plate, the case's back door: the object's
 * clip opens then); holds it out over the top and hands it over; the customer takes it in the right
 * hand and pays with the left, the vendor takes the money and puts it away.
 *
 * What changes hands is drawn by the residents' code (`HANDED`): the snapshot gives both rows the
 * item (`held`), the vendor holds it from picking it up to handing it over, the customer from then on.
 */

import { clip, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { hangArms, stand } from './idle';
import { armIK, wrist, type ArmOptions } from './ik';

/** Seconds of `buy` and `sell` (a served use's minutes at speed 1). */
export const SERVE_SECONDS = 5;
/** Where the clips are, as shares of them: the order taken, the goods reached for, picked up, handed over, paid for. */
export const SERVE = { ordered: 0.24, reach: 0.3, picked: 0.42, handed: 0.62, paid: 0.84 } as const;
/** Who holds what changes hands, by clip and share of it (`residents.ts`): the vendor between picking it up and handing it over, the customer from then. */
export const HANDED: Record<string, readonly [number, number]> = { sell: [SERVE.picked, SERVE.handed], buy: [SERVE.handed, 1] };

type Hand = ArmOptions & { at: Vec };

function hands(body: PoseSpec, l: Hand | null, r: Hand | null): PoseSpec {
  return { ...body, ...(l ? armIK('l', l.at, body, { pole: [-0.5, -1, -0.3], ...l }) : {}), ...(r ? armIK('r', r.at, body, { pole: [0.5, -1, -0.3], ...r }) : {}) };
}

// The vendor: behind the counter (its top about 0.9-1 m up, half a metre ahead).
const behind = (o: Parameters<typeof stand>[0] = {}): PoseSpec => stand({ lean: 4, fwd: 0.02, ...o });
/** Listening, the hands on the counter's edge. */
const listen = (nod: number): PoseSpec => hands(behind({ head: [6 + 8 * nod, 0, 0] }), { at: [-0.2, 0.98, 0.34], pronate: 80, hand: wrist('l', 10, 0) }, { at: [0.2, 0.98, 0.34], pronate: 80, hand: wrist('r', 10, 0) });
/** Down into the goods (turned a little to the right, where they are). */
const into = (deep: number): PoseSpec =>
  hands(behind({ lean: 16 + 10 * deep, sink: 0.02 + 0.02 * deep, twist: -12, head: [16 + 8 * deep, -10, 0] }), { at: [-0.2, 0.98, 0.32], pronate: 80, hand: wrist('l', 10, 0) }, { at: [0.16, 0.92 - 0.1 * deep, 0.48], pronate: 70, hand: wrist('r', -10, 0) });
/** Holding it out over the top. */
const holdOut = (far: number): PoseSpec => hands(behind({ lean: 8, fwd: 0.03, head: [4, 0, 0] }), { at: [-0.2, 0.98, 0.32], pronate: 80, hand: wrist('l', 10, 0) }, { at: [0.12, 1.14, 0.38 + 0.2 * far], pronate: 10, hand: wrist('r', 15, 0) });
/** The hand out, palm up, for the money; then into the till or the tin at the side. */
const palmUp = (): PoseSpec => hands(behind({ lean: 6, head: [8, 0, 0] }), { at: [-0.2, 0.98, 0.32], pronate: 80, hand: wrist('l', 10, 0) }, { at: [0.1, 1.08, 0.46], pronate: -80, hand: wrist('r', 20, 0) });
const till = (): PoseSpec => hands(behind({ lean: 10, twist: -18, head: [14, -18, 0] }), { at: [-0.2, 0.98, 0.32], pronate: 80, hand: wrist('l', 10, 0) }, { at: [0.3, 0.96, 0.3], pronate: 60, hand: wrist('r', 0, 0) });
const rest = (): PoseSpec => ({ ...behind(), ...hangArms(6, 7, 18) });

// The customer: at the front, facing it.
const front = (o: Parameters<typeof stand>[0] = {}): PoseSpec => stand({ lean: 2, ...o });
/** Pointing at what they'd like, a nod. */
const order = (beat: number): PoseSpec => hands(front({ head: [8 + 6 * beat, 4, 0], twist: 4 }), null, { at: [0.12, 1.16, 0.4], pronate: 20, hand: wrist('r', -10, 0) });
const waitFor = (): PoseSpec => ({ ...front({ shift: 0.3, head: [6, 0, 0] }), ...hangArms(4, 7, 16) });
/** Reaching out for it, then holding it in front. */
const take = (): PoseSpec => hands(front({ lean: 6, fwd: 0.03, head: [6, 0, 0] }), null, { at: [0.1, 1.12, 0.48], pronate: 10, hand: wrist('r', 15, 0) });
const holding = (): PoseSpec => hands(front({ head: [10, 0, 0] }), null, { at: [0.16, 1.02, 0.28], pronate: 0, hand: wrist('r', 20, 0) });
/** Paying with the other hand while holding it. */
const pay = (out: number): PoseSpec =>
  hands(front({ lean: 4 + 3 * out, head: [8, 0, 0] }), { at: [-0.1, 1.05 + 0.08 * out, 0.26 + 0.22 * out], pronate: -60 * out, hand: wrist('l', 10, 0) }, { at: [0.16, 1.02, 0.28], pronate: 0, hand: wrist('r', 20, 0) });

export function serveClips(): ClipDef[] {
  const s = SERVE;
  return [
    clip('sell', { seconds: SERVE_SECONDS, loop: false }, [
      [0, () => listen(0)],
      [0.08, () => listen(1), 'smooth'],
      [0.14, () => listen(0), 'smooth'],
      [s.ordered, () => listen(0.3), 'smooth'],
      [s.reach, () => into(0.4), 'smooth'],
      [s.picked - 0.04, () => into(1), 'smooth'],
      [s.picked + 0.02, () => into(0.5), 'smooth'],
      [s.handed - 0.08, () => holdOut(0.6), 'smooth'],
      [s.handed, () => holdOut(1), 'smooth'],
      [s.handed + 0.06, () => palmUp(), 'smooth'],
      [s.paid, () => palmUp(), 'smooth'],
      [s.paid + 0.08, () => till(), 'smooth'],
      [1, () => rest(), 'smooth'],
    ]),
    clip('buy', { seconds: SERVE_SECONDS, loop: false }, [
      [0, () => order(0)],
      [0.08, () => order(1), 'smooth'],
      [0.16, () => order(0), 'smooth'],
      [s.ordered, () => order(0.5), 'smooth'],
      [s.ordered + 0.08, () => waitFor(), 'smooth'],
      [s.handed - 0.08, () => waitFor(), 'smooth'],
      [s.handed, () => take(), 'smooth'],
      [s.handed + 0.06, () => holding(), 'smooth'],
      [s.handed + 0.12, () => pay(1), 'smooth'],
      [s.paid, () => pay(1), 'smooth'],
      [s.paid + 0.08, () => pay(0), 'smooth'],
      [1, () => holding(), 'smooth'],
    ]),
  ];
}
