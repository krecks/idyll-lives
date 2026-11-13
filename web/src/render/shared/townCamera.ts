/**
 * The town view's camera: how the play orbit changes as it zooms out
 * past `TOWN_FROM` to `TOWN_MAX` (`lens.ts`). There is no switch: everything here is a function of the orbit's
 * radius that is the play camera's own value up to `TOWN_FROM` and eases with `townBlend` from there, so nothing
 * jumps on the way out or back in.
 *
 * - The tilt's limits ease into `TOWN_BETA` (a bird's-eye view, neither straight down nor along the ground).
 * - The target is held over land: the pan bounds are drawn in from the land's edge as the view widens (`townInset`).
 * - The wheel and the zoom keys go faster (`townWheel`, `townKeyZoom`): the keys are metres a frame, which is quick
 *   at 20 m and would take a minute from 120 m to 900 m.
 * - The haze thins to `TOWN_FOG` of its density by `TOWN_FULL` and on from there (`townFog`), and it begins further
 *   out, `FOG_FROM` times the camera's distance in the full town view (`townFogFrom`): the town is as clear as the
 *   play camera's lot and only the land well past it fades into the horizon.
 * - The camera's planes move out (`townNear`, `townFar`) so the whole world is drawn and the roads, a centimetre
 *   over the ground, keep their depth.
 *
 * The menus' town (the overview behind the main menu and the new-game screens) is a town camera too, always in the
 * full town view whatever its distance: the same planes and the same haze (`menuFog`, `menuFogFrom`).
 *
 * Engine-neutral, no allocations per frame.
 */

import { tuned } from '../tune';
import { LENS, RADIUS_LIMITS, TOWN_FROM, TOWN_FULL, TOWN_MAX, townBlend } from './lens';
import type { OrbitShape } from './orbit';

/**
 * The haze from over the town (`?tune=haze.from:2` on the dev server).
 * - `town`: the share of the look's density left in the full town view (`townFog`).
 * - `from`: where the haze begins there, a multiple of the camera's distance (`shaders/fog.ts` `game_fogFrom`). The
 *   town the camera looks at is about its distance away, its far side at most half as far again, so 1.5 leaves it
 *   clear.
 * - `ramp`: how much sooner than the rest of the town view the start moves out (`townFogFrom`): halfway there the
 *   start is all the way out, or the far half of the picture is still the play camera's haze, a grey wall.
 * - `edge`: how much haze at least where the land ends in the picture (`landEdge`, `edgeFog`): with the haze begun
 *   so far out, the land past the town was nearly as clear as the town, and its edge a hard line on the sky's dome.
 */
const HAZE = tuned('haze', { town: 0.25, from: 1.5, ramp: 2, edge: 0.6 });

/** The tilt's limits in the full town view (rad from straight down). */
export const TOWN_BETA = [0.6, 1.0] as const;
/** The share of the haze's density left in the full town view. */
export const TOWN_FOG = HAZE.town;
/** Where the haze begins in the full town view and the menus' town: this many times the camera's distance from its target. */
export const FOG_FROM = HAZE.from;
/** The least haze where the land ends in the picture, in the town view and the menus' town (0 none, 1 all). */
export const FOG_EDGE = HAZE.edge;
/** The orbit's radius limits with the town view: the play camera's nearest, the town view's farthest. */
export const TOWN_RADIUS_LIMITS = [RADIUS_LIMITS[0], TOWN_MAX] as const;
/** The near plane's share of the camera's distance in the full town view (the play camera's: `LENS.nearShare`). */
export const TOWN_NEAR_SHARE = 1 / 8;
/**
 * How far past its target (m, a share of the radius) a town camera sees land near the top of the picture at a
 * middling tilt: what `townInset` keeps on the land and its surroundings.
 */
const REACH = 0.6;
/** The least the target can roam either way in the full town view (m): the pan bounds are never drawn in further. */
const ROAM_LEAST = 96;

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

/** The tilt's limits for a camera `radius` m out, into `out`: `limits` (the play camera's) eased into `TOWN_BETA`. */
export function townBeta(radius: number, limits: readonly [number, number], out: [number, number]): void {
  const b = townBlend(radius);
  out[0] = lerp(limits[0], Math.max(limits[0], TOWN_BETA[0]), b);
  out[1] = lerp(limits[1], Math.min(limits[1], TOWN_BETA[1]), b);
}

/**
 * How far (m) the pan bounds are drawn in from each edge for a camera `radius` m out, over land `width` × `depth` m
 * with `margin` m of the bounds round it and `ring` m of landscape past its edge: the further out the camera, the
 * more of what lies past its target it sees, so the target stays that much further from the edge. Never so far that
 * less than `ROAM_LEAST` m is left to roam either way.
 */
export function townInset(radius: number, width: number, depth: number, margin: number, ring: number): number {
  const b = townBlend(radius);
  if (b <= 0) return 0;
  const most = Math.max(0, Math.min(width, depth) / 2 + margin - ROAM_LEAST);
  return b * Math.min(most, Math.max(margin, margin + REACH * radius - ring));
}

/** The wheel's zoom, as a multiple of the play camera's share of the radius: up to twice as fast in the full town view. */
export function townWheel(radius: number): number {
  return 1 + townBlend(radius);
}

/**
 * The zoom keys' speed, as a multiple of the play camera's (which is metres a frame, the same at any distance): past
 * `TOWN_FROM` it grows with the radius, and up to three times that in the full town view.
 */
export function townKeyZoom(radius: number): number {
  return radius <= TOWN_FROM ? 1 : (radius / TOWN_FROM) * (1 + 2 * townBlend(radius));
}

/**
 * The haze's density for a camera `radius` m out, as a share of the look's: down to `TOWN_FOG` by `TOWN_FULL`,
 * and on the way there never more than keeps the middle of the picture as clear as it
 * is from `TOWN_FROM` (`TOWN_FROM` over the radius: at the look's own density the town 250 m off would be half haze
 * before it clears again). Past `TOWN_FULL` it thins on the same way (a ninth at `TOWN_MAX`): at a flat quarter the
 * town 900 m off is four tenths haze and everything behind it is gone.
 */
export function townFog(radius: number): number {
  if (radius <= TOWN_FROM) return 1;
  if (radius >= TOWN_FULL) return (TOWN_FOG * TOWN_FULL) / radius;
  return Math.min(1 - (1 - TOWN_FOG) * townBlend(radius), TOWN_FROM / radius);
}

/**
 * Where the haze begins (m from the eye) for a camera `radius` m out: at the eye up to `TOWN_FROM` (the play camera's
 * haze), eased out to `FOG_FROM` times the radius halfway into the town view (`HAZE.ramp`) and on from there. Before
 * this the land a little past the town was half haze from 250 m, and from `TOWN_MAX` everything past the town went
 * into a grey-blue wall.
 */
export function townFogFrom(radius: number): number {
  return FOG_FROM * radius * Math.min(1, HAZE.ramp * townBlend(radius));
}

/**
 * How far (m) from an eye at (`ex`, `ey`, `ez`), looking along `alpha` (the orbit's azimuth: it looks towards
 * -(cos α, sin α)), the land `x0`..`x1` × `z0`..`z1` ends: along the middle of the picture and along its sides
 * (`spread` rad either way, half the lens's width), the nearest of the three. Past it there is only the sky's dome.
 * Infinity when none of the three crosses the land ahead.
 */
export function landEdge(ex: number, ey: number, ez: number, alpha: number, spread: number, x0: number, z0: number, x1: number, z1: number): number {
  let nearest = Infinity;
  for (let i = -1; i <= 1; i++) {
    const a = alpha + i * spread;
    const fx = -Math.cos(a);
    const fz = -Math.sin(a);
    // (The line along it is between each pair of sides from where it enters to where it leaves; it leaves the
    // land at the first of the two exits, if it was in it by then.)
    const out = Math.min(slab(ex, fx, x0, x1, false), slab(ez, fz, z0, z1, false));
    if (out > 0 && out >= Math.max(slab(ex, fx, x0, x1, true), slab(ez, fz, z0, z1, true))) nearest = Math.min(nearest, out);
  }
  return Math.hypot(nearest, ey);
}

/** Where a line from `e` going `f` a metre enters (`enter`) or leaves the band `lo`..`hi` (m along it; ±Infinity along it). */
function slab(e: number, f: number, lo: number, hi: number, enter: boolean): number {
  if (Math.abs(f) < 1e-9) return e >= lo && e <= hi ? (enter ? -Infinity : Infinity) : enter ? Infinity : -Infinity;
  const a = (lo - e) / f;
  const b = (hi - e) / f;
  return enter ? Math.min(a, b) : Math.max(a, b);
}

/**
 * The least haze density (per m) for a camera `radius` m out whose haze begins `from` m out and whose land ends
 * `edge` m off (`landEdge`), tilted `beta` from straight down: what lies at the edge `FOG_EDGE` haze (in
 * `shaders/fog.ts`'s terms: the exponential-squared factor taken to the 2.2th power), so the land fades out before
 * it ends on the sky's dome. Only from above: a low camera (the main menu's) sees the land end as the horizon, hills
 * against the sky, and a haze that hid them took the far half of the picture with it. (Never thicker than that haze
 * half the radius past the start: a camera near the land's edge looking out keeps its town.)
 */
export function edgeFog(radius: number, from: number, edge: number, beta: number): number {
  const above = 1 - Math.min(1, Math.max(0, (beta - 1.05) / 0.2));
  if (!Number.isFinite(edge) || above <= 0) return 0;
  return (above * Math.sqrt(-Math.log(1 - FOG_EDGE) / 2.2)) / Math.max(radius / 2, edge - from, 1);
}

/**
 * The menus' town's haze density for a camera `radius` m out, as a share of the look's: the full town view's
 * (`townFog` past `TOWN_FULL`, the look's own nearer than `TOWN_FOG · TOWN_FULL`), a third of that round a mountain
 * town, whose peaks all round are what the picture is for.
 */
export function menuFog(radius: number, mountainous: boolean): number {
  return Math.min(1, (TOWN_FOG * TOWN_FULL) / Math.max(1, radius)) * (mountainous ? 0.35 : 1);
}

/** Where the menus' town's haze begins (m from the eye) for a camera `radius` m out: the full town view's, at any distance. */
export function menuFogFrom(radius: number): number {
  return FOG_FROM * radius;
}

/**
 * The near plane (m) for a camera `radius` m out whose eye is `clearance` m over the land under it: the play
 * camera's (`LENS.nearShare` of the radius, never under `LENS.near`) eased out to `TOWN_NEAR_SHARE` of the radius,
 * and never more than half the eye's height over the land (a hill under a tilted camera stays whole). `b`: how far
 * into the town view (`townBlend`; the menus' town is always fully in it).
 */
export function townNear(radius: number, clearance: number, b = townBlend(radius)): number {
  const play = Math.max(LENS.near, radius * LENS.nearShare);
  if (b <= 0) return play;
  return Math.max(play, Math.min(radius * lerp(LENS.nearShare, TOWN_NEAR_SHARE, b), clearance / 2));
}

/**
 * The far plane (m) for a camera `radius` m out whose eye is at (`ex`, `ey`, `ez`): `far` (the preset's) eased out
 * to past the furthest corner of the rectangle `x0`..`x1` × `z0`..`z1` (the land and the landscape round it), so the
 * whole world is drawn from the full town view. Never nearer than `far`. `b`: as `townNear`'s.
 */
export function townFar(radius: number, far: number, ex: number, ey: number, ez: number, x0: number, z0: number, x1: number, z1: number, b = townBlend(radius)): number {
  if (b <= 0) return far;
  const dx = Math.max(Math.abs(ex - x0), Math.abs(ex - x1));
  const dz = Math.max(Math.abs(ez - z0), Math.abs(ez - z1));
  // (Some way past the corner: hills rise there, and the sky's dome stands just inside the plane.)
  const whole = Math.hypot(dx, ey, dz) * 1.05 + 60;
  return Math.max(far, lerp(far, whole, b));
}

/**
 * The orbit's shape for a land `width` × `depth` m whose pan bounds lie `margin` m round it, `ring` m of landscape
 * past its edge (`OrbitControls.shape`).
 */
export function townOrbit(width: number, depth: number, margin: number, ring: number): OrbitShape {
  return {
    beta: (radius, limits, out) => townBeta(radius, limits, out),
    inset: (radius) => townInset(radius, width, depth, margin, ring),
    wheel: townWheel,
    keyZoom: townKeyZoom,
  };
}

/** The radius a flight to a town point ends at when none is asked for: near enough to see who is there, no nearer than the camera is. */
export function townArrival(radius: number, toResident: boolean): number {
  return toResident ? Math.min(radius, 24) : Math.min(Math.max(radius, 24), 40);
}
