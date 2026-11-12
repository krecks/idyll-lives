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

import type { GrassQuality } from './shared/grass';

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
   * How many times a second the sun's map is drawn again for the residents' poses (a breathing Sim
   * would draw it every frame: 181 casters, 0.9 ms of CPU and 0.6 ms of the GPU on the M3). Everything
   * else that changes it (the sun's step, the walls, a thing placed or hidden) draws it at once; while
   * the game is paused nothing does. 0: every change (about 22 fps fewer on the village orbit, Medium,
   * WebGPU). `?shadowrate=<Hz>` overrides it.
   */
  shadowRate: number;
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
   * A reflection's resolution, as a share of the screen's along each side, rounded to the nearest
   * power of two (at 1080p 1024×512 for 0.4 and 0.5, 1024×1024 for 0.75);
   * multisampled as `msaaSamples` says. Memory while one reflects: Low +7 MB, Medium +25 MB, High +50 MB.
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
   * Share of the lawn's flowers drawn (always the same ones; the placement is otherwise unchanged).
   * Under 1, none of the patches of daisies, buttercups and violets in their drifts either
   * (`shared/nature.ts` `lawns`): 0.08 ms of GPU a frame on Low/WebGL2, about 0.07 on Medium/WebGPU.
   */
  lawnDensity: number;
  /**
   * The lawn's grass (`pc/grass.ts`): drawn within `radius` m of the eye (the ground carries the
   * lawn past it), `density` of its blades (fewer drawn wider, covering as much), and from the
   * draw `nearest` (`GRASS.lods`; 1 leaves out the curved 3-segment blades).
   */
  grass: GrassQuality;
  /**
   * The side (m) of the squares the game draws the woods in, each left out when out of view; 0: eight
   * sectors round the town, each reaching to the horizon (`shared/nature.ts`). The same trees drawn
   * either way, but 100 m squares leave out most of those off screen: Low on WebGL2, the wide view,
   * 1.79 → 1.21 M triangles and −0.3 ms of GPU time; the orbit 1.30 → 1.04 M and −0.1 ms. The cost:
   * 446 more meshes to cull on the village, +0.2 ms of main-thread time (performance.md, 0.5.0).
   */
  woodsCell: number;
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
  /**
   * The lot's objects cast the sun's shadows from one merged mesh of their bodies (baked again as
   * one moves or hides) rather than from every model's thin instances: the furniture costs one
   * shadow draw, not a hundred. See-through things and moving parts cast from their own meshes.
   */
  mergedObjectShadows: boolean;
  /** 1: the street's models' far versions, where they have one (fewer triangles); 0: full detail. */
  streetLod: 0 | 1;
  /**
   * The menus' town, standing still, drawn once and shown again until something in it changes,
   * instead of drawn anew every frame: while it holds, nothing is drawn and the canvas keeps its
   * last picture (`render/pc/heldFrames.ts`). The game itself draws live. `?snapshot=0` turns it off.
   */
  holdStillFrames: boolean;
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
  /**
   * Garden birds (render/shared/birds.ts): at most `count` of them, living in the trees, bushes and
   * hedges within `range` m of the viewed lot. A draw per species and part (five parts, up to five
   * species: up to 25 draws, fewer with fewer species about), no shadows; only the birds whose pose
   * changed are written each frame, only the parts that changed (wings only in flight), and only
   * those parts' buffers uploaded (Medium at 8:00: about 12 of 24 birds and 14 of 25 buffers a frame,
   * most of them swaying with their branches; at night about 3 and 6). The birds' own main-thread time a frame, M3 Pro headless, the village
   * at 8:00: Low 0.03 ms, Medium 0.05 ms, High 0.06 ms (CPU throttled 4×: 0.11 / 0.24 / 0.25 ms);
   * the frame's CPU within noise at 1× (±0.5 ms), +1.2–1.4 ms at 4× on Medium and High.
   */
  birds: { count: number; range: number };
  /**
   * Residents outdoors on any lot are drawn within this many metres of the camera's target, and in
   * the view (`docs/design/open-world.md`, life on every lot); those indoors only on the viewed lot.
   * Past `FAR_POSE_DISTANCE` they're posed less often.
   */
  residentRadius: number;
  /**
   * A big world's distances (`docs/design/open-world.md`): what is drawn round the roaming camera.
   * Medium and High are the open world as it was tuned; Low draws less of it (on WebGL2 the woods and
   * far houses past the haze cost an older laptop's GPU what it can't spare).
   */
  openWorld: OpenWorldQuality;
  /**
   * Low's leaner picture chain (`pc/post.ts`): the scene in an 11/11/10-bit float target where the GPU
   * can draw into one (RGBA16F otherwise), and the FXAA's input 8 bits a channel, the image-processed
   * colour clamped there. Half the bytes of the two half-float full-size targets. Medium and High
   * keep the half-float scene (and have no FXAA).
   */
  leanTargets: boolean;
  /**
   * Contact shading (`pc/post.ts`): the sky's light, and a third of the lamps' and the sun's, is shaded where
   * things meet (a sofa against the wall, a chair's legs on the floor, a pot on the sill), from the
   * scene's depth (PlayCanvas's screen-space ambient occlusion at half the picture's size, applied in
   * the lighting). It draws the scene's depth once more before the scene (a depth-only pass over every
   * solid thing), so it is off on Low. `?diag=noao` leaves it out.
   */
  contactShading: boolean;
  /**
   * Cloud shadows by day (`pc/cloudShadows.ts`): one more texture read wherever the sun's light is worked out.
   * Medium on WebGPU: within the runs' noise. Low on WebGL2: 2.58 → 2.65 ms of GPU a frame in the village
   * orbit (M3 Pro, medians of 5, 2026-11-12), so Low has none. `?diag=noclouds` leaves them out.
   */
  cloudShadows: boolean;
}

/** A big world's distances on a preset (`QualitySettings.openWorld`). */
export interface OpenWorldQuality {
  /** The stream radius's share (`shared/focus.ts` `streamRadius`): the chunks streamed in round the camera, and with them the residents and things the sim keeps in full. */
  stream: number;
  /** The far LOD's houses past this (m, a draw group's middle from the camera's target) aren't drawn (`pc/street.ts`). */
  farCull: number;
  /** The woods' squares further than this (m, middle to the camera's target) aren't drawn (`pc/nature.ts`): the haze has them. */
  woodsFar: number;
  /** The side (m) of the woods' squares (`shared/nature.ts`): smaller ones leave out more trees off screen, at a draw each. */
  woodsCell: number;
  /** The lawn flowers' squares further than this (m) aren't drawn: a flower is a few pixels by then. */
  flowersFar: number;
  /** The ground tiles' finest and middle levels within these (m, a tile's middle to the camera's target; past them the coarsest). */
  tiles: readonly [number, number];
  /**
   * The camera's far plane (m). The haze (`look.ts` `fog`) has everything past about 600 m (the fog
   * factor under 0.01 % there), so a nearer plane leaves out only the far ground tiles behind it.
   */
  far: number;
}

/** What the renderer is built with: a preset's knobs, and which preset (the F3 overlay shows it). */
export interface RenderQuality extends QualitySettings {
  preset: GraphicsPreset;
}

/** The open world as it was tuned (Medium and High). */
const OPEN_WORLD: OpenWorldQuality = { stream: 1, farCull: 650, woodsFar: 560, woodsCell: 256, flowersFar: 170, tiles: [200, 460], far: 2600 };

export const QUALITY: Record<GraphicsPreset, QualitySettings> = {
  low: { shadowMapSize: 1024, shadowRate: 15, msaaSamples: 1, anisotropy: 4, clusteredLamps: false, mirrors: 1, mirrorScale: 0.4, mirrorRate: 10, lightingStep: 10, lawnDensity: 0.5, grass: { radius: 45, density: 0.5, nearest: 1 }, woodsCell: 100, landscapeShadows: false, streetShadows: false, characterShadows: false, mergedObjectShadows: true, streetLod: 1, holdStillFrames: true, ambientRange: 0, birds: { count: 6, range: 18 }, residentRadius: 90, openWorld: { stream: 0.7, farCull: 450, woodsFar: 400, woodsCell: 128, flowersFar: 120, tiles: [150, 350], far: 1800 }, leanTargets: true, contactShading: false, cloudShadows: false },
  medium: { shadowMapSize: 2048, shadowRate: 30, msaaSamples: 4, anisotropy: 8, clusteredLamps: true, mirrors: 1, mirrorScale: 0.5, mirrorRate: 10, lightingStep: 0.5, lawnDensity: 1, grass: { radius: 60, density: 1, nearest: 0 }, woodsCell: 0, landscapeShadows: true, streetShadows: true, characterShadows: true, mergedObjectShadows: false, streetLod: 0, holdStillFrames: true, ambientRange: 40, birds: { count: 24, range: 35 }, residentRadius: 140, openWorld: OPEN_WORLD, leanTargets: false, contactShading: true, cloudShadows: true },
  high: { shadowMapSize: 4096, shadowRate: 30, msaaSamples: 4, anisotropy: 8, clusteredLamps: true, mirrors: 2, mirrorScale: 0.75, mirrorRate: 0, lightingStep: 0.5, lawnDensity: 1, grass: { radius: 90, density: 1, nearest: 0 }, woodsCell: 0, landscapeShadows: true, streetShadows: true, characterShadows: true, mergedObjectShadows: false, streetLod: 0, holdStillFrames: true, ambientRange: 60, birds: { count: 40, range: 50 }, residentRadius: 200, openWorld: OPEN_WORLD, leanTargets: false, contactShading: true, cloudShadows: true },
};

export const GRAPHICS_PRESETS = Object.keys(QUALITY) as GraphicsPreset[];

export function isGraphicsPreset(value: unknown): value is GraphicsPreset {
  return typeof value === 'string' && Object.hasOwn(QUALITY, value);
}
