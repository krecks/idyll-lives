/**
 * The graphics presets: Low, Medium and High (Settings › Performance › Graphics; `?quality=low|
 * medium|high` overrides the setting for testing). Medium is everyone's default and the game's
 * intended look. High is Medium with sharper sun shadows and live mirrors. Low is Medium with
 * modest trims for older laptops (integrated Intel graphics).
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
   * How many mirrors (the nearest the camera) reflect the room live; 0: none. Each draws the world
   * once more, about 50 MB of memory traffic a frame (P1-09); the live cost wasn't measured.
   */
  mirrors: number;
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
  /** WebGPU only: record draw commands once and replay them each frame (the menus' town). */
  snapshotRendering: boolean;
  /**
   * Metres from the camera within which things' ambient clips play (a ceiling fan, clock hands, a
   * fountain's water: model `motions.idle`); 0: none, all at rest. Each moving thing rewrites its
   * model's part buffers every frame (one upload per model, not per thing); shadows aren't redrawn
   * for them. Not measured yet.
   */
  ambientRange: number;
}

/** What the renderer is built with: a preset's knobs, and which preset (the F3 overlay shows it). */
export interface RenderQuality extends QualitySettings {
  preset: GraphicsPreset;
}

export const QUALITY: Record<GraphicsPreset, QualitySettings> = {
  low: { shadowMapSize: 1024, msaaSamples: 1, anisotropy: 4, clusteredLamps: false, mirrors: 0, lightingStep: 10, lawnDensity: 0.5, landscapeShadows: false, snapshotRendering: true, ambientRange: 0 },
  medium: { shadowMapSize: 2048, msaaSamples: 4, anisotropy: 8, clusteredLamps: true, mirrors: 0, lightingStep: 0.5, lawnDensity: 1, landscapeShadows: true, snapshotRendering: true, ambientRange: 40 },
  high: { shadowMapSize: 4096, msaaSamples: 4, anisotropy: 8, clusteredLamps: true, mirrors: 2, lightingStep: 0.5, lawnDensity: 1, landscapeShadows: true, snapshotRendering: true, ambientRange: 60 },
};

export const GRAPHICS_PRESETS = Object.keys(QUALITY) as GraphicsPreset[];

export function isGraphicsPreset(value: unknown): value is GraphicsPreset {
  return typeof value === 'string' && Object.hasOwn(QUALITY, value);
}
