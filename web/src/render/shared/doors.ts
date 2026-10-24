/**
 * Doors and gates that swing for residents going through them, engine-neutral (the renderers
 * turn the leaves): whether a resident's way crosses a doorway.
 */

/**
 * Seconds a door or a gate takes to swing open or shut at speed 1 (quicker as the game runs
 * faster, up to 10x, as walkers' turns are). It opens for someone whose way crosses its doorway
 * within `DOOR_AHEAD` metres (sim-core `snapshot::AHEAD`: about a second's walk), or who stands
 * less than `DOORWAY` metres either side of it; open, it waits to shut until nobody walking is
 * within `DOOR_CLEAR` of it (a leaf is 0.83 m long).
 */
export const DOOR_SWING = 0.45;
const DOOR_AHEAD = 1.5;
const DOORWAY = 0.35;
const DOOR_CLEAR = 1.1;
/** Floats per resident in `movers`: where they are, the waypoint they walk to, where their way is `DOOR_AHEAD` on. */
export const MOVER = 6;
/** Smoothstep: eases 0..1 in and out. */
export function ease(t: number): number {
  return t * t * (3 - 2 * t);
}

/**
 * Whether someone in `movers` (`MOVER` floats each) goes through the 1 m doorway whose middle is
 * `centre`, in a wall or fence along x (`axis` 'h') or z: their way crosses it before `DOOR_AHEAD`
 * metres on, or they're in it. Walking past, or up to it and away, isn't. One that stands `open`
 * also stays so while anyone on the move is still within `DOOR_CLEAR` of it (the leaf would
 * sweep through them shutting). Returns the way they go across (+1 towards +z of an 'h' edge or
 * +x of a 'v' one, -1 the other), 0 for nobody.
 */
export function goingThrough(movers: readonly number[], axis: 'h' | 'v', centre: readonly [number, number], open: boolean): number {
  // In the doorway's own terms: `a` along the edge from its middle, `n` across it.
  const h = axis === 'h';
  const [ca, cn] = h ? centre : [centre[1], centre[0]];
  const [ia, iN] = h ? [0, 1] : [1, 0];
  for (let i = 0; i < movers.length; i += MOVER) {
    const pa = movers[i + ia] - ca;
    const pn = movers[i + iN] - cn;
    let wa = movers[i + 2 + ia] - ca;
    let wn = movers[i + 2 + iN] - cn;
    const qa = movers[i + 4 + ia] - ca;
    const qn = movers[i + 4 + iN] - cn;
    if ((Math.abs(pa) <= 0.5 && Math.abs(pn) < DOORWAY) || (open && pa * pa + pn * pn < DOOR_CLEAR * DOOR_CLEAR)) return qn >= pn ? 1 : -1;
    // A waypoint further than the look ahead: where it looks to is on the way there.
    if (Math.hypot(wa - pa, wn - pn) >= DOOR_AHEAD) {
      wa = qa;
      wn = qn;
    }
    const way = crosses(pa, pn, wa, wn) || crosses(wa, wn, qa, qn);
    if (way) return way;
  }
  return 0;
}

/** Whether a step from `s` to `e` (along, across) crosses a 1 m doorway at the origin: its way across (±1), or 0. */
function crosses(sa: number, sn: number, ea: number, en: number): number {
  if (sn * en > 0 || sn === en) return 0;
  return Math.abs(sa + ((ea - sa) * sn) / (sn - en)) <= 0.5 ? Math.sign(en - sn) : 0;
}
