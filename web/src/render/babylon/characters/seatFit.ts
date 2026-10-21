/**
 * Seated legs fitted to the body and the seat (characters.ts `perch` and `seatLegs`, checked by
 * tools/characters/check_clips.ts): the seated clips are made for a 0.45 m seat on the reference body, so a
 * shorter body's feet would hang above the floor, a lower seat's go through it, and short thighs on a deep
 * seat end inside its cushion. The runtime moves the hips forward on a deep seat (`perch`), as far as the
 * knees need to clear its front (furniture.json `seats` `front`), and turns each leg so the ankle comes
 * down to the floor (`feetDown`, `legTurn`): the thigh sloping down off the seat's front by `SEAT_FIT.slope`
 * at most, then the feet hang (a child on a grown-up's chair). Body space, scale 1, in the y-z plane (+z
 * forward). Pure maths, shared with the checker.
 */

/**
 * A leg in a clip (`measureLegs`; body space, scale 1, in the y-z plane): the hip's height and depth,
 * the thigh's and the calf's lengths and their directions (rad from straight down towards the front).
 */
export interface LegSide {
  hy: number;
  hz: number;
  l1: number;
  l2: number;
  t0: number;
  c0: number;
}

/**
 * - `slope`: how far (rad) the thigh may slope down from the clip's (about level) to reach the floor: sitting
 *   on the front of the seat; any lower and the feet hang.
 * - `clear`: how far (m, for a body of the reference height) the knee joint comes past a seat's front, the calf
 *   behind it clear of the cushion.
 * - `rim`: the hips stay this far (m, likewise) behind a seat's front, the buttocks on the seat.
 * - `stretch`: relaxing (`relax`, `nap`, `watch`), the feet this much further forward (m, likewise).
 */
export const SEAT_FIT = { slope: 0.3, clear: 0.08, rim: 0.16, stretch: 0.12 };
/** The reference body's height (m), which the lengths above are for. */
const REF_HEIGHT = 1.77;

/**
 * Turns a leg (`fit`) so its ankle reaches (`y`, `z`) (the knee bent forward; out of reach, straightened
 * towards it): the thigh's and the calf's rotations about x into `out` (the calf's relative to the turned
 * thigh), added to the clip's (characters.ts `addRotation`: a positive x rotation swings a limb from +z
 * towards -y).
 */
export function legTurn(fit: LegSide, y: number, z: number, out: Float32Array | number[]): Float32Array | number[] {
  const { hy, hz, l1, l2 } = fit;
  const r = clamp(Math.hypot(y - hy, z - hz), Math.abs(l1 - l2) + 0.01, l1 + l2 - 0.002);
  // Direction angles from straight down (-y) towards the front (+z).
  const d = Math.atan2(z - hz, hy - y);
  const thigh = d + Math.acos(clamp((l1 * l1 + r * r - l2 * l2) / (2 * l1 * r), -1, 1));
  const calf = d - Math.acos(clamp((l2 * l2 + r * r - l1 * l1) / (2 * l2 * r), -1, 1));
  out[0] = fit.t0 - thigh;
  out[1] = fit.c0 - calf - out[0];
  return out;
}

/**
 * How far forward (m, body scale 1) the hips move on a seat whose top ends `depth` m (body scale 1) ahead of
 * the seated pelvis joint (at `pelvisZ`): until the knee joint is `SEAT_FIT.clear` past it, keeping the hips
 * `SEAT_FIT.rim` behind it. `size`: the body's height over the reference's.
 */
export function perch(fit: LegSide, pelvisZ: number, depth: number, size: number): number {
  // (The knee as far back as it comes with the thigh sloping down to the floor: `feetDown`.)
  const knee = fit.hz + fit.l1 * Math.sin(fit.t0 - SEAT_FIT.slope) - pelvisZ;
  return clamp(depth + SEAT_FIT.clear * size - knee, 0, Math.max(0, depth - SEAT_FIT.rim * size));
}

/**
 * Where the ankle goes (body space, scale 1) for the foot to stand on the floor, the ankle `floor` m up
 * (body space: its standing height less the root's height over the floor): below the clip's ankle, `stretch`
 * m further forward. Out of reach with the thigh sloping down `SEAT_FIT.slope` more than the clip's, the
 * shin hangs straight down from the knee there instead.
 */
export function feetDown(fit: LegSide, floor: number, stretch: number): [number, number] {
  const z = fit.hz + fit.l1 * Math.sin(fit.t0) + fit.l2 * Math.sin(fit.c0) + stretch;
  const t = fit.t0 - SEAT_FIT.slope;
  const knee: [number, number] = [fit.hy - fit.l1 * Math.cos(t), fit.hz + fit.l1 * Math.sin(t)];
  const lowest = knee[0] - fit.l2;
  return floor >= lowest ? [floor, z] : [lowest, Math.min(z, knee[1] + 0.02)];
}

/** A body's height over the reference's (`SEAT_FIT`'s lengths are the reference's). */
export function seatSize(height: number): number {
  return height / REF_HEIGHT;
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}
