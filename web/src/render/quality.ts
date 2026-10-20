/**
 * The graphics presets: Low, Medium and High (Settings › Performance › Graphics; `?quality=low|
 * medium|high` overrides the setting for testing). Medium is everyone's default and the game's
 * intended look. High is Medium with sharper sun shadows and two mirrors drawn every frame. Low is
 * Medium with heavier trims for older laptops (integrated Intel graphics): fewer draws and triangles.
 *
 * Every knob is in this one table. A new knob gets a field below, with what it does and what it
 * costs, and a value in each preset.
 *
 * Costs are what leaving a feature out saved on Medium: GPU time on an M3 Pro at 1920×1080, WebGPU,
 * no vsync, median of 3 runs (the rendering review P1, `?bench=village`). "Intel" figures are
 * P1's bandwidth model for an HD 620, not measurements.
 */

export type GraphicsPreset = 'low' | 'medium' | 'high';

export interface QualitySettings {
  /**
   * Size of the sun's shadow map (texels a side). Shadows cost 1.2 ms by day and 1.5 ms at night at
   * 2048 (WebGL2: 1.5 / 2.3 ms of the frame), measured before 0.2.0 culled the casters to the map and
   * redrew it only when something in it moves (now about a third of the frame's draw calls in play,
   * none while nothing moves). Each step up is four times the texels: 1024 is blurrier, 4096 sharper.
   */
  shadowMapSize: number;
  /**
   * 4: 4× multisampling of the HDR scene (smooth edges): 3.0 ms of Medium's 10.6 ms by day; about
   * 100 MB of memory traffic a frame at 1080p, about 4.5 ms on Intel. 1: FXAA instead (a cheap
   * filter pass; edges a little softer).
   */
  msaaSamples: number;
  /**
   * Anisotropic filtering of the floor, wall, road and ground textures (sharp at a slant).
   * Not measured apart; texture bandwidth, small on the M3.
   */
  anisotropy: number;
  /**
   * Every lamp lights (clustered lighting, up to 32) rather than the four nearest the biggest
   * rooms. At night 4.3 ms on WebGPU and 1.6 ms on WebGL2; by day about nothing, since 0.2.0 leaves
   * dark lamps out of the cluster (before: 3.1 / 0.8 ms with every lamp dark).
   */
  clusteredLamps: boolean;
  /**
   * How many mirrors (the nearest the camera, on screen) reflect the room; 0: none. Each draws again
   * only what can be seen through its glass, and only the residents in it: the village's bathroom
   * mirror, 67 draws a reflection (was 437 with the whole world and every resident). Its render target
   * exists only while a mirror reflects, and 10 s after.
   */
  mirrors: number;
  /**
   * A reflection's resolution, as a share of the screen's along each side, rounded by Babylon to
   * powers of two (at 1080p 1024×512 for 0.4 and 0.5, 1024×1024 for 0.75); multisampled as
   * `msaaSamples` says. Memory while one reflects: Low +7 MB, Medium +25 MB, High +50 MB.
   */
  mirrorScale: number;
  /**
   * At most this many draws of a reflection a second, each only when something in it changed (the
   * camera, the mirror, a resident in it, the light, things placed); held in between, looked up
   * through the camera it was drawn for. 0: every frame. Medium with the camera swinging before a
   * mirror (the worst case, as a resident moving in front of it): −6 % fps, +1.0 ms CPU, +0.1 ms
   * GPU (WebGPU, M3 Pro); camera still, or the orbit: within noise. High's two every frame: −20 %.
   */
  mirrorRate: number;
  /**
   * Game minutes between lighting updates at speed 3 and up (slower speeds: every half minute).
   * Each update re-colours the sun, sky and lamps and thaws the materials for a few frames: at
   * speed 5, 20 updates a second, CPU p95 4.7–6 ms against 0.3–0.4 ms (P1-11).
   */
  lightingStep: number;
  /**
   * Share of the lawn's grass tufts and flowers drawn (always the same ones; the placement is
   * otherwise unchanged): 0.5 leaves out 20,410 of the landscape's 44,655 plants.
   */
  lawnDensity: number;
  /**
   * The woods round the town cast shadows (the lot's own garden trees, the street and the house
   * always do). Low's half lawn and no woods' shadows together: −0.29 M triangles, −6 shadow draws
   * and −0.5 ms GPU in the orbit view by day (Low, WebGPU).
   */
  landscapeShadows: boolean;
  /**
   * The street outside the viewed lot casts the sun's shadows: neighbours' houses and gardens,
   * trees, parked cars, kerb furniture and public-place things. The lot's own house, garden and
   * furniture always do. Low leaves the street out: fewer shadow-map draws (the street is mostly
   * seen beyond the lot) at the cost of softer, flatter neighbours.
   */
  streetShadows: boolean;
  /**
   * The residents cast the sun's shadows. Low leaves them to their soft contact shadows: 38 fewer
   * shadow-map draws in the village orbit, at the cost of flat-looking people in direct sun.
   */
  characterShadows: boolean;
  /** WebGPU only: record draw commands once and replay them each frame (the menus' town). */
  snapshotRendering: boolean;
  /**
   * Metres from the camera within which things' ambient clips play (a ceiling fan, clock hands, a
   * fountain's water: model `motions.idle`); 0: none, all at rest. Each moving thing rewrites its
   * model's part buffers every frame (one upload per model, not per thing); shadows aren't redrawn
   * for them, and no draw calls are added (moving parts have their meshes anyway). 94 at once on a
   * lot, M3 Pro headless: 0.09 ms of CPU for the clips and +0.15–0.3 ms for the frame; with the CPU
   * throttled 4× (a stand-in for an old laptop's): 0.2–0.35 ms and +0.4–0.6 ms. Not measured on
   * Intel graphics hardware.
   */
  ambientRange: number;
}

/** What the renderer is built with: a preset's knobs, and which preset (the F3 overlay shows it). */
export interface RenderQuality extends QualitySettings {
  preset: GraphicsPreset;
}

export const QUALITY: Record<GraphicsPreset, QualitySettings> = {
  low: { shadowMapSize: 1024, msaaSamples: 1, anisotropy: 4, clusteredLamps: false, mirrors: 1, mirrorScale: 0.4, mirrorRate: 10, lightingStep: 10, lawnDensity: 0.5, landscapeShadows: false, streetShadows: false, characterShadows: false, snapshotRendering: true, ambientRange: 0 },
  medium: { shadowMapSize: 2048, msaaSamples: 4, anisotropy: 8, clusteredLamps: true, mirrors: 1, mirrorScale: 0.5, mirrorRate: 30, lightingStep: 0.5, lawnDensity: 1, landscapeShadows: true, streetShadows: true, characterShadows: true, snapshotRendering: true, ambientRange: 40 },
  high: { shadowMapSize: 4096, msaaSamples: 4, anisotropy: 8, clusteredLamps: true, mirrors: 2, mirrorScale: 0.75, mirrorRate: 0, lightingStep: 0.5, lawnDensity: 1, landscapeShadows: true, streetShadows: true, characterShadows: true, snapshotRendering: true, ambientRange: 60 },
};

export const GRAPHICS_PRESETS = Object.keys(QUALITY) as GraphicsPreset[];

export function isGraphicsPreset(value: unknown): value is GraphicsPreset {
  return typeof value === 'string' && Object.hasOwn(QUALITY, value);
}
