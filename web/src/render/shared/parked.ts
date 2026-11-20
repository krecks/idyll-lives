/**
 * Bikes and e-scooters left out by a place their riders are visiting (sim-core `ride.rs` `park_spot`, the
 * snapshot's `ride` -2 less a stable id and `parkX`/`parkZ`/`parkYaw`): where `ParkedRides` draws each thing to ride.
 * Apart from the renderer, for its test.
 */

/** Where a bike left out by a place stands (its middle) and which way it faces (rad, a placement's turn). */
export interface Away {
  x: number;
  z: number;
  yaw: number;
}

/**
 * Where each of `parked` is drawn now (its placement's x, z and turn; `home` its own): left out by a place
 * (`away`) there, wherever it's from; else at home when it's off the viewed lot and not ridden (`hidden`); else
 * not at all. For `ParkedRides.write`, and its test.
 */
export function placements<P extends { id: number; x: number; z: number; home: boolean }>(
  parked: readonly P[],
  away: ReadonlyMap<number, Away>,
  hidden: ReadonlySet<number>,
): { p: P; at: Away | null }[] {
  const out: { p: P; at: Away | null }[] = [];
  for (const p of parked) {
    const at = away.get(p.id);
    if (at) out.push({ p, at });
    else if (p.home && !hidden.has(p.id)) out.push({ p, at: null });
  }
  return out;
}

/** The elements of a copy's matrix that say where it stands and which way (`castKey`), and a float's bits read as one. */
const KEYED = [0, 2, 12, 13, 14] as const;
const BITS_F = new Float32Array(1);
const BITS_U = new Uint32Array(BITS_F.buffer);

/**
 * The copies of a draw (their matrices, game space) that stand where `reaches` says (the sun's map), where they stand
 * and which way, as a number: the same while those are, however many copies there are elsewhere. For
 * `ParkedRides.castKey`, and its test.
 */
export function castKey(copies: readonly ArrayLike<number>[], reaches: (x: number, z: number) => boolean): number {
  let h = 0x811c9dc5;
  for (const m of copies) {
    if (!reaches(m[12], m[14])) continue;
    for (const k of KEYED) {
      BITS_F[0] = m[k];
      h = Math.imul(h ^ BITS_U[0], 0x01000193);
    }
  }
  return h >>> 0;
}
