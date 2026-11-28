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

/** What a draw keyed by its copies answers (`Instances.castKey`: `castKeyOf` of its copies, kept while they are). */
interface CastKeyed {
  castKey(reaches: (x: number, z: number) => boolean, ground: number): number;
}

/**
 * The draws keyed by their copies (`InstancesOptions.castKeyed`: the parked cars' and bikes'), by their mesh instance:
 * what the sun's map asks of a still caster (`SunShadows.castKey`).
 */
export const keyedDraws = new WeakMap<object, CastKeyed>();

/** The elements of a copy's matrix that say where it stands and which way (`castKeyOf`), and a float's bits read as one. */
const KEYED = [0, 2, 12, 13, 14] as const;
const BITS_F = new Float32Array(1);
const BITS_U = new Uint32Array(BITS_F.buffer);

/**
 * The copies of a draw that stand where `reaches` says (the sun's map), where they stand and which way, as a number:
 * the same while those are, however many copies there are elsewhere; 0 when none is there. The `count` copies are laid
 * out `stride` floats apart in `data`, each starting with its matrix (game space; an instanced draw's buffer:
 * `Instances.castKey`). A copy hidden (an all-zero matrix) stands nowhere.
 */
export function castKeyOf(data: ArrayLike<number>, count: number, stride: number, reaches: (x: number, z: number) => boolean): number {
  let h = 0x811c9dc5;
  let any = false;
  for (let i = 0; i < count; i++) {
    const o = i * stride;
    if (data[o + 15] === 0 || !reaches(data[o + 12], data[o + 14])) continue;
    any = true;
    for (const k of KEYED) {
      BITS_F[0] = data[o + k];
      h = Math.imul(h ^ BITS_U[0], 0x01000193);
    }
  }
  // (0 is kept for none there; a hash that came out 0 is 1.)
  return any ? h >>> 0 || 1 : 0;
}
