/**
 * The game camera's lens, its zoom and tilt limits and the pose it starts in (`pc/PlayCanvasRenderer.ts`), apart from
 * the renderer so what is tuned to where the camera looks from (the residents' detail: `characters/parts.ts`
 * `LOD_FAR_PX`) and its tests read the same numbers.
 */

/** The game camera's zoom and tilt limits. */
export const RADIUS_LIMITS = [6, 95] as const;
export const BETA_LIMITS = [0.3, 1.35] as const;
/** Its lens: vertical field of view (a slightly long lens flattens the perspective: the dollhouse look), near and far (m). */
// (The near plane a fiftieth of the camera's distance, never under 0.3 m: from the town overview's 200–300 m at 0.3 m
// the roads, 1.2 cm over the ground, fought it in green patches. Close up nothing changes: 0.3 m to 15 m out.)
export const LENS = { fov: 0.7, near: 0.3, far: 1600, nearShare: 1 / 50 } as const;
/** The game camera's pose when a game opens (`OrbitControls`: azimuth, tilt and distance, m). */
export const DEFAULT_ORBIT = { alpha: -Math.PI * 0.62, beta: 0.86, radius: 24 } as const;
