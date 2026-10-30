/**
 * Doors and gates that swing for residents going through them, engine-neutral (the renderers
 * turn the leaves and the resident puts a hand to the handle): whether a resident's way crosses a
 * doorway, how near they are to it, and which rooms' doors can be kept shut.
 */
import type { ObjectDef, RoomKindDef } from '../../content/content';

/**
 * Seconds a door or a gate takes to swing open or shut at speed 1 (quicker as the game runs
 * faster, up to 10x, as walkers' turns are). It starts when the walker's hand could be on it, and
 * it is timed so the leaf is open by the time they are: the lead is `PACE` × the game's rate ×
 * `DOOR_SWING`, or `DOOR_HAND` where that is less (a slow walk, where the hand is nearer than the
 * swing needs). At speed 1 that is a hand's reach, and the leaf is open as the walker arrives; the
 * quickest walkers (1.38x: traits and perks) push it the rest of the way through, as a hand on a
 * swinging door does. Open, it waits to shut until nobody walking is within `DOOR_CLEAR` of it (a
 * leaf is 0.83 m long).
 */
export const DOOR_SWING = 0.45;
/** How fast residents walk at 1× (m/s; sim-core `WALK_SPEED`, 1.5 m a second). */
const PACE = 1.5;
/** The nearest a leaf starts for: a walker's hand reaching it (m). */
const DOOR_HAND = 0.75;
/** The reach the hand comes out over, once the leaf starts (m of the walker's way). */
const HAND_FADE = 0.5;
const DOORWAY = 0.35;
const DOOR_CLEAR = 1.1;
/**
 * Floats per resident in `movers`: where they are, the waypoint they walk to, where their way is
 * `lead` on, and which resident they are (their row, for `hand`; only walkers are in it).
 */
export const MOVER = 7;
/** Smoothstep: eases 0..1 in and out. */
export function ease(t: number): number {
  return t * t * (3 - 2 * t);
}

/**
 * How far ahead of a doorway a leaf starts to swing, for a game running `rate` times as fast: far
 * enough that the swing (`DOOR_SWING` of game time) finishes as the walker arrives, and never less
 * than a hand's reach, so a slow walk opens the door at the hand.
 */
export function doorLead(rate: number): number {
  return Math.max(DOOR_HAND, PACE * Math.max(rate, 1) * DOOR_SWING);
}

/**
 * Whether someone in `movers` (`MOVER` floats each) goes through the 1 m doorway whose middle is
 * `centre`, in a wall or fence along x (`axis` 'h') or z: their way crosses it before `lead`
 * metres on, or they're in it. Walking past, or up to it and away, isn't. One that stands `open`
 * also stays so while anyone on the move is still within `DOOR_CLEAR` of it (the leaf would sweep
 * through them shutting). Returns the way they go across (+1 towards +z of an 'h' edge or +x of a
 * 'v' one, -1 the other), 0 for nobody. `hand`: one float a resident, filled with how far out the
 * hand reaches for the handle, 1 as they come to it and 0 further off (the leaf's swing is timed
 * to it; the furthest any doorway they're going through gets).
 */
export function goingThrough(
  movers: readonly number[],
  axis: 'h' | 'v',
  centre: readonly [number, number],
  open: boolean,
  lead: number,
  hand?: Float32Array,
): number {
  // In the doorway's own terms: `a` along the edge from its middle, `n` across it.
  const h = axis === 'h';
  const [ca, cn] = h ? centre : [centre[1], centre[0]];
  const [ia, iN] = h ? [0, 1] : [1, 0];
  let way = 0;
  for (let i = 0; i < movers.length; i += MOVER) {
    const pa = movers[i + ia] - ca;
    const pn = movers[i + iN] - cn;
    // Where their way is `lead` m on: along the direction the snapshot gives (its ahead point, which
    // follows the path round corners), never past it.
    let wa = movers[i + 4 + ia] - ca;
    let wn = movers[i + 4 + iN] - cn;
    const far = Math.hypot(wa - pa, wn - pn);
    if (far > 0) {
      const f = Math.min(1, lead / far);
      wa = pa + (wa - pa) * f;
      wn = pn + (wn - pn) * f;
    }
    const here = Math.abs(pa) <= 0.5 && Math.abs(pn) < DOORWAY;
    if (here || (open && pa * pa + pn * pn < DOOR_CLEAR * DOOR_CLEAR)) {
      const across = wn >= pn ? 1 : -1;
      if (!way) way = across;
      if (hand) {
        const w = handAt(Math.abs(pn), lead);
        const who = movers[i + 6] | 0;
        if (w > hand[who]) hand[who] = w;
      }
      continue;
    }
    const across = crosses(pa, pn, wa, wn);
    if (!across) continue;
    if (!way) way = across;
    if (hand) {
      const w = handAt(Math.abs(pn), lead);
      const who = movers[i + 6] | 0;
      if (w > hand[who]) hand[who] = w;
    }
  }
  return way;
}

/** How far out the hand reaches `dist` m short of the doorway: 0 at `lead`, 1 within `HAND_FADE` of it. */
function handAt(dist: number, lead: number): number {
  const fade = Math.min(HAND_FADE, lead);
  return Math.max(0, Math.min(1, (lead - dist) / fade));
}

/** Whether a step from `s` to `e` (along, across) crosses a 1 m doorway at the origin: its way across (±1), or 0. */
function crosses(sa: number, sn: number, ea: number, en: number): number {
  if (sn * en > 0 || sn === en) return 0;
  return Math.abs(sa + ((ea - sa) * sn) / (sn - en)) <= 0.5 ? Math.sign(en - sn) : 0;
}

/**
 * Object kinds whose room the simulation may keep shut (sim-core `privacy.rs`): what makes a room
 * of a private kind (a toilet's `bathroom` tag: the bathroom), and a use done behind a shut door
 * (a double bed's *Try for a baby*). Their rooms' doors swing (`HouseLooks.shuts`).
 */
export function shutDefs(content: { readonly objectList: readonly ObjectDef[]; readonly roomKinds: readonly RoomKindDef[] }): Set<string> {
  const tags = new Set(content.roomKinds.filter((k) => k.private).flatMap((k) => k.tags));
  const shuts = (def: ObjectDef) => def.interactions.some((i) => i.private || i.conceive || (i.tags ?? []).some((t) => tags.has(t)));
  return new Set(content.objectList.filter(shuts).map((d) => d.id));
}
