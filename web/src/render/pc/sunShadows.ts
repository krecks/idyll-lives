// The fade at the map's edge follows Babylon.js 9.30's shadow `frustumEdgeFalloff`. Changed for
// Idyll Lives: as a patch of PlayCanvas's `lightFunctionShadowPS`.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The sun's shadow map in the game: it covers a fixed square round the viewed lot, so its texels
 * keep their size and place while the camera turns and the sun steps, and it is drawn again only
 * when something in it changed (`moved`): the sun's step, a resident's pose (throttled to the
 * preset's `shadowRate`), a thing placed, the walls; a car or a resident moving only when it is in
 * the square or near it (`reaches`). Casters outside the square aren't drawn. The menus' town is fitted round the camera's
 * view instead, and drawn every frame.
 *
 * The still casters' picture is kept apart (`still`, `stillPasses`): a depth texture the map's size holding all but
 * what moves on its own (the residents' draws, the lot's moving parts: `movers`). A redraw for what moves alone (a
 * resident's pose, a door a use opens: `posed`) copies that picture into the map and draws only what moves over it
 * (about 40 of the village's 220 draws); the picture is drawn again first when something in it changed (`moved`, the
 * light's view, a still caster that came, went or moved). Depths are depths whatever order they are drawn in, so the
 * map is the same as drawn at once (pixel for pixel on both backends). `?diag=wholeshadows` draws everything every time.
 *
 * PlayCanvas fits a directional light's map to the camera's view. For a light with a square, the
 * fit is handed the square instead: the directional shadow renderer's `cull` asks the camera for
 * the corners of its view, and gets eight points of a sphere round the square's middle (its
 * radius the square's half side), seen from the camera. Everything else (the light's camera
 * turned with the sun, snapped to whole texels, its depth fitted to the casters, the casters culled
 * to it) is PlayCanvas's own.
 *
 * Where the map ends, the sun shines (an engine patch of `lightFunctionShadowPS`, `patchShadowEdge`):
 * outside the map it is lit (PlayCanvas would read the map's edge texels there, smeared over the
 * land beyond, with the casters' depths: shadow cut off along a diagonal), and the map's last
 * stretch fades to lit in a circle (`frustumEdgeFalloff`, see the header; `SHADOW_EDGE_FADE`), so
 * the shadows end softly round the lot rather than at a square's edge. The menus' town has no fade.
 *
 * The map's size, filtering and bias are the preset's (`QualitySettings.shadowMapSize`, see
 * `SunShadows.constructor`).
 *
 * What only the shadow map draws (the house's lid over its rooms, which keeps the sun out of them
 * but through windows and doorways; the woods drawn whole; Low's merged object shadows) is on a
 * layer of its own that the sun lights and no camera draws (`layers.ts` `shadowOnlyLayer`).
 */

import * as pc from './engine';
import { directionalShadows } from './internals';
import { shadowOnlyLayer } from './layers';
import { toEngine } from './space';
import type { Language } from './shaders/chunks';

export { shadowOnlyLayer } from './layers';

/**
 * The share of the map's half side, from its edge in, over which the game's shadows fade out (as
 * radius² in the map's clip space, as `frustumEdgeFalloff` has it).
 */
const SHADOW_EDGE_FADE = 0.15;

/**
 * The receivers' offset along their normal before the map is read, in texels of the map (PlayCanvas's
 * `normalOffsetBias`, metres: worked out from the square and the map's size). Up to 0.3.2 the
 * casters were pushed 2 cm in along theirs (at 2048 texels over ~60 m: about 0.7 of a texel); a
 * texel's worth keeps the acne off the lawn at Low's 1024 texels without the shadows letting go of
 * their casters at High's 4096.
 */
const NORMAL_OFFSET_TEXELS = 0.75;

/**
 * The depth bias (PlayCanvas's `shadowBias`: a polygon offset of `-1000 ×` it, constant and per
 * slope). Up to 0.3.2 every caster had 0.15 % of the depth range (about 7 cm) added; PlayCanvas fits
 * the range to the casters and offsets by slope, which keeps the steep faces (roofs, the hills)
 * clean at a much smaller number.
 */
const SHADOW_BIAS = 0.02;

/**
 * How far (m) past the square's edge a moving caster can still throw its shadow into the map (a
 * resident or a car with the sun low): one moving further out leaves the map as it is (`reaches`).
 */
export const CASTER_REACH = 8;

/**
 * How far the light's view × projection may differ, element by element, from the one the still casters were drawn
 * with for their picture to be used again (`SunShadows.drawPass`): PlayCanvas fits it again for every redraw, and its
 * float noise as the camera turns (±2.4e-7 measured) moves the map by a ten-thousandth of a texel. 1e-6: under 0.03 of
 * a texel 30 m from the middle.
 */
const VIEW_NOISE = 1e-6;

/** The sun's pass as PlayCanvas 2.23 makes it (`RenderPassShadowDirectional`; `check:chunks`). */
interface ShadowPass {
  light: SceneLight;
  camera: pc.Camera;
  renderTarget: pc.RenderTarget | null;
  shadowRenderer: { renderFace(light: SceneLight, camera: pc.Camera, face: number, clear: boolean): void };
  depthStencilOps: { clearDepth: boolean };
  before(): void;
  execute(): void;
}

/** PlayCanvas 2.23's scene light, as far as the sun's pass reads it (`check:chunks`). */
interface SceneLight {
  numCascades: number;
  _shadowMap: object | null;
  _shadowCascadesInvalidated: boolean;
  shadowUpdateOverrides: number[] | null;
  getRenderData(camera: pc.Camera, face: number): { shadowCamera: pc.Camera; visibleCasters: pc.MeshInstance[] };
}

/** The game's sun shadows by PlayCanvas's light (`stillPasses`). */
const owners = new WeakMap<object, SunShadows>();
const stillPatched = new WeakSet<object>();

/**
 * The sun's pass draws as `SunShadows` says (`drawStill`, `drawPass`): before it starts, the still casters' picture
 * into the map (drawn again first when they changed); in it, without clearing, what moves.
 */
function stillPasses(app: pc.AppBase): void {
  // SAFETY: PlayCanvas 2.23's `ShadowRendererDirectional.getLightRenderPass` makes the pass and hands it to
  // `this.shadowRenderer.setupRenderPass(renderPass, shadowCamera, clear)`, asked of the instance; a pass calls its
  // `before` outside the GPU's pass and its `execute` in it, which draws each cascade with
  // `shadowRenderer.renderFace` (`tools/render/chunks.json`).
  const sr = directionalShadows(app) as unknown as { shadowRenderer?: { setupRenderPass(pass: ShadowPass, camera: pc.Camera, clear: boolean): void } } | undefined;
  const shadows = sr?.shadowRenderer;
  if (!shadows || stillPatched.has(shadows)) return;
  stillPatched.add(shadows);
  const setup = shadows.setupRenderPass.bind(shadows);
  shadows.setupRenderPass = (pass, camera, clear) => {
    setup(pass, camera, clear);
    const owner = pass.light ? owners.get(pass.light) : undefined;
    if (!owner?.splits(pass.light)) return;
    pass.depthStencilOps.clearDepth = false;
    pass.before = () => owner.drawStill(pass);
    pass.execute = () => owner.drawPass(pass);
  };
}

interface Square {
  /** The middle (PlayCanvas space) and the half side (m). */
  centre: pc.Vec3;
  radius: number;
}

/** The squares of the lights that have one (by PlayCanvas's scene light). */
const squares = new WeakMap<object, Square>();
const patched = new WeakSet<object>();

/** Hands the directional shadow renderer of `app` the squares of lights that have one. */
function patch(app: pc.AppBase): void {
  const sr = directionalShadows(app);
  if (!sr || patched.has(sr)) return;
  patched.add(sr);
  const cull = sr.cull.bind(sr);
  const corners = Array.from({ length: 8 }, () => new pc.Vec3());
  const inverse = new pc.Mat4();
  const offsets: [number, number, number][] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1], [0, 0, 0], [0, 0, 0]];
  sr.cull = (light, comp, camera, casters, mask) => {
    const square = squares.get(light);
    if (!square) return cull(light, comp, camera, casters, mask);
    const own = camera.getFrustumCorners;
    camera.getFrustumCorners = () => {
      // Eight points whose middle is the square's and whose farthest is its half side away, in the camera's space (`cull` turns them back into the world).
      inverse.copy(camera.node.getWorldTransform()).invert();
      const { centre: c, radius: r } = square;
      for (let i = 0; i < 8; i++) {
        const [x, y, z] = offsets[i];
        inverse.transformPoint(corners[i].set(c.x + x * r, c.y + y * r, c.z + z * r), corners[i]);
      }
      return corners;
    };
    try {
      return cull(light, comp, camera, casters, mask);
    } finally {
      camera.getFrustumCorners = own;
    }
  };
}

/** Anchors of the edge patch in `lightFunctionShadowPS` (also in `tools/render/chunks.json`). */
export const SHADOW_EDGE_ANCHORS = {
  glsl: {
    coord: 'vec3 shadowCoord = getShadowSampleCoord{i}(shadowMatrix, light{i}_shadowParams, vPositionW, vec3(0.0), lightDirW, dLightDirNormW, dVertexNormalW);',
    reads: ['getShadowPCF1x1', 'getShadowPCF3x3', 'getShadowPCF5x5'].map((f) => `return ${f}(SHADOWMAP_PASS(light{i}_shadowMap), shadowCoord, light{i}_shadowParams);`),
  },
  wgsl: {
    coord: 'var shadowCoord: vec3f = getShadowSampleCoord{i}(shadowMatrix, uniform.light{i}_shadowParams, vPositionW, vec3f(0.0), &lightDirArg, dLightDirNormW, dVertexNormalW);',
    reads: ['getShadowPCF1x1', 'getShadowPCF3x3', 'getShadowPCF5x5'].map((f) => `return ${f}(light{i}_shadowMap, light{i}_shadowMapSampler, shadowCoord, uniform.light{i}_shadowParams);`),
  },
} as const;

const EDGE_CODE: Record<Language, { decl: string; fade: string }> = {
  glsl: {
    decl: `
#ifndef GAME_SUN_EDGE
#define GAME_SUN_EDGE
	uniform float game_sunShadowFade;
#endif
`,
    fade: `
				vec2 gameSunClip = shadowCoord.xy * 2.0 - 1.0;
				float gameSunFade = smoothstep(1.0 - game_sunShadowFade, 1.00000012, clamp(dot(gameSunClip, gameSunClip), 0.0, 1.0));
				if (gameSunFade >= 1.0) { return 1.0; }`,
  },
  wgsl: {
    decl: `
#ifndef GAME_SUN_EDGE
#define GAME_SUN_EDGE
	uniform game_sunShadowFade: f32;
#endif
`,
    fade: `
				let gameSunClip = shadowCoord.xy * 2.0 - 1.0;
				let gameSunFade = smoothstep(1.0 - uniform.game_sunShadowFade, 1.00000012, clamp(dot(gameSunClip, gameSunClip), 0.0, 1.0));
				if (gameSunFade >= 1.0) { return 1.0; }`,
  },
};

const PATCHED = '// (idyll-lives: shadow edge)';
const edged = new WeakSet<pc.GraphicsDevice>();

/** `code` with `anchor` replaced (exactly once: a missing anchor is an upgrade to look at). */
function replaceOnce(code: string, anchor: string, replacement: string): string {
  const at = code.indexOf(anchor);
  if (at < 0 || code.indexOf(anchor, at + anchor.length) >= 0) throw new Error(`[render] shader chunk lightFunctionShadowPS: anchor not found once: ${anchor}`);
  return code.slice(0, at) + replacement + code.slice(at + anchor.length);
}

/**
 * Patches `device`'s directional shadow lookup (once, before the first shader is built): lit outside
 * the map, fading to lit over its last stretch (`game_sunShadowFade`, see the file's header).
 */
function patchShadowEdge(device: pc.GraphicsDevice): void {
  if (edged.has(device)) return;
  edged.add(device);
  for (const lang of ['glsl', 'wgsl'] as const) {
    const chunks = pc.ShaderChunks.get(device, lang === 'glsl' ? pc.SHADERLANGUAGE_GLSL : pc.SHADERLANGUAGE_WGSL);
    let code = chunks.get('lightFunctionShadowPS');
    if (typeof code !== 'string') throw new Error('[render] PlayCanvas has no shader chunk lightFunctionShadowPS');
    if (code.includes(PATCHED)) continue;
    const a = SHADOW_EDGE_ANCHORS[lang];
    code = replaceOnce(code, a.coord, `${a.coord}${EDGE_CODE[lang].fade}`);
    for (const read of a.reads) code = replaceOnce(code, read, `${read.replace(/^return (.*);$/, 'return mix($1, 1.0, gameSunFade);')}`);
    chunks.set('lightFunctionShadowPS', `${PATCHED}\n${EDGE_CODE[lang].decl}${code}`);
  }
}

export interface SunShadowOptions {
  /** Texels a side (`QualitySettings.shadowMapSize`). */
  size: number;
  /** Pose-driven redraws a second at most (`QualitySettings.shadowRate`; 0: every change). */
  rate: number;
  /** The fade to lit at the map's edge (default `SHADOW_EDGE_FADE`, the game's; 0: none). */
  fade?: number;
  /**
   * The still casters' picture kept apart, so a redraw for what moves on its own alone draws only that over a copy of
   * it (`posed`; default true; `?diag=wholeshadows` off). A second depth texture the map's size.
   */
  still?: boolean;
}

export class SunShadows {
  private readonly light: pc.LightComponent;
  /** Milliseconds between pose-driven redraws (0: every change), and when the last one was made. */
  private readonly interval: number;
  private lastDraw = -Infinity;
  private pending = false;
  private overview = false;
  private readonly size: number;
  private readonly square: Square = { centre: new pc.Vec3(), radius: 30 };
  /** The square on the ground (game space, m): from (`x0`, `z0`) to (`x1`, `z1`). */
  private readonly ground = { x0: -30, z0: -30, x1: 30, z1: 30 };
  private readonly fadeId: pc.ScopeId;
  private readonly fade: number;
  /**
   * Something in the map changed (`moved`; `urgent`: not a resident's pose; `casters`: something that casts, not
   * only the sun's way), for what else holds a picture of the world (the mirrors, the lamps' shadows).
   */
  onMoved: ((urgent: boolean, casters: boolean) => void) | null = null;
  /**
   * What moves on its own and casts (the residents' draws, the lot's moving parts), drawn at every redraw over the
   * still casters' picture (`still`); null: every redraw draws everything.
   */
  movers: ((out: pc.MeshInstance[]) => void) | null = null;
  /** Whether the still casters' picture is kept (`SunShadowOptions.still`). */
  readonly still: boolean;
  /** Redraws since the page opened: of the still casters too, and of only what moves (the bench's counts). */
  readonly draws = { whole: 0, moving: 0 };
  /** The still casters' picture is drawn again at the next redraw: something in it changed (`moved`). */
  private whole = true;
  /** The still casters' picture (a depth texture the map's size) and what it was drawn of. */
  private stillTarget: pc.RenderTarget | null = null;
  private readonly drawn = { viewProj: new Float32Array(16), map: null as object | null, casters: [] as pc.MeshInstance[], boxes: new Float64Array(0), counts: new Float64Array(0) };
  /** This redraw's casters: those that move (`movers`), the still ones, and all of them as PlayCanvas culled them. */
  private readonly moving = new Set<pc.MeshInstance>();
  private readonly movingList: pc.MeshInstance[] = [];
  private readonly stillCasters: pc.MeshInstance[] = [];
  private readonly allCasters: pc.MeshInstance[] = [];
  private readonly viewProj = new pc.Mat4();
  private readonly view = new pc.Mat4();
  /** A pass drawing the still casters into their picture (`drawStill`). */
  private stillPass: pc.RenderPass | null = null;
  private readonly device: pc.GraphicsDevice;

  /**
   * Shadows from `sun`'s light: a map of `options.size` texels a side, filtered by 3×3 PCF on every
   * preset, the receivers offset by a share of a texel.
   */
  constructor(app: pc.AppBase, sun: pc.Entity, options: SunShadowOptions) {
    patch(app);
    patchShadowEdge(app.graphicsDevice);
    this.device = app.graphicsDevice;
    this.still = options.still ?? true;
    if (this.still) stillPasses(app);
    // (A lost context loses the picture too.)
    this.device.on('devicerestored', this.restored);
    this.fade = options.fade ?? SHADOW_EDGE_FADE;
    this.fadeId = app.graphicsDevice.scope.resolve('game_sunShadowFade');
    this.fadeId.setValue(this.fade);
    this.light = sun.light!;
    this.size = options.size;
    this.interval = options.rate > 0 ? 1000 / Math.min(240, options.rate) : 0;
    const l = this.light;
    l.shadowResolution = options.size;
    l.shadowType = pc.SHADOW_PCF3_32F;
    l.numCascades = 1;
    l.shadowBias = SHADOW_BIAS;
    l.shadowUpdateMode = pc.SHADOWUPDATE_NONE;
    squares.set(l.light, this.square);
    owners.set(l.light, this);
    shadowOnlyLayer(app, sun);
    this.applyBias();
  }

  /** The receivers' normal offset for the square's texels (`NORMAL_OFFSET_TEXELS`). */
  private applyBias(): void {
    this.light.normalOffsetBias = (NORMAL_OFFSET_TEXELS * 2 * this.square.radius) / this.size;
  }

  /** The square round the viewed lot: its middle (game space, on the ground) and half side (m). */
  setSquare(x: number, z: number, radius: number): void {
    toEngine(x, 0, z, this.square.centre);
    this.square.radius = radius;
    const g = this.ground;
    g.x0 = x - radius;
    g.z0 = z - radius;
    g.x1 = x + radius;
    g.z1 = z + radius;
    this.applyBias();
    this.moved();
  }

  /**
   * Whether a caster moving at (x, z) (game space) could change the map: it is within `CASTER_REACH` m
   * of the square (always, in the menus' town, whose map follows the view). What the cars and
   * residents ask of each that moved: one moving across town leaves the map as it is.
   */
  reaches(x: number, z: number): boolean {
    const g = this.ground;
    const r = CASTER_REACH;
    return this.overview || (x >= g.x0 - r && x <= g.x1 + r && z >= g.z0 - r && z <= g.z1 + r);
  }

  /** The menus' town (fitted round the view, drawn every frame, no fade at its edge) or the game. */
  setOverview(on: boolean): void {
    this.overview = on;
    if (on) squares.delete(this.light.light);
    else squares.set(this.light.light, this.square);
    this.fadeId.setValue(on ? 0 : this.fade);
    this.light.shadowUpdateMode = on ? pc.SHADOWUPDATE_REALTIME : pc.SHADOWUPDATE_NONE;
    this.moved();
  }

  /**
   * Something in the map changed: it is drawn again (whole) in the next frame. `urgent` changes (the sun,
   * the walls, a thing placed) draw at once; what moves on its own (a car, a door indoors, a moving part, a chair
   * pulled in) may wait for the shadow rate. `casters`: false when only the sun's way changed, or what casts no
   * shadow but the sun's (a door's leaf).
   */
  moved(urgent = true, casters = true): void {
    this.whole = true;
    this.redraw(urgent, casters);
  }

  /**
   * What moves on its own moved where it casts into the map (a resident's pose, a moving part a use moves): drawn
   * again at the shadow rate, over the still casters' picture unless something else changed too (`moved`).
   */
  posed(): void {
    if (!this.still || !this.movers) this.whole = true;
    this.redraw(false, true);
  }

  private redraw(urgent: boolean, casters: boolean): void {
    this.onMoved?.(urgent, casters);
    if (this.overview) return;
    const now = performance.now();
    if (this.interval <= 0 || urgent || now - this.lastDraw >= this.interval) {
      this.pending = false;
      this.lastDraw = now;
      this.light.shadowUpdateMode = pc.SHADOWUPDATE_THISFRAME;
    } else {
      this.pending = true;
    }
  }

  /** Each frame: a pose redraw held back draws once the interval has passed. */
  frame(): void {
    if (!this.pending || performance.now() - this.lastDraw < this.interval) return;
    this.pending = false;
    this.lastDraw = performance.now();
    this.light.shadowUpdateMode = pc.SHADOWUPDATE_THISFRAME;
  }

  /** Whether the sun's pass for `light` is drawn as two (`stillPasses`): the still casters' picture, then what moves. */
  splits(light: SceneLight): boolean {
    return this.still && !!this.movers && !this.overview && light.numCascades === 1;
  }

  /**
   * Before the sun's pass (outside the GPU's pass): the still casters' picture drawn again if they changed (whole: as
   * PlayCanvas would draw the map, without what moves), then copied into the map. The light's view must be the one
   * the picture was drawn with (within `VIEW_NOISE`), the map the same, and the still casters the same ones in the same
   * places (each one's box and copies); else it is drawn again.
   */
  drawStill(pass: ShadowPass): void {
    const { light, camera } = pass;
    const map = pass.renderTarget;
    const data = light.getRenderData(camera, 0);
    const casters = data.visibleCasters;
    // This redraw's casters, the moving and the still.
    const moving = this.moving;
    const list = this.movingList;
    moving.clear();
    list.length = 0;
    this.movers?.(list);
    for (const mi of list) moving.add(mi);
    const still = this.stillCasters;
    still.length = 0;
    for (const mi of casters) if (!moving.has(mi)) still.push(mi);
    this.lightViewProj(data.shadowCamera);
    if (!map?.depthBuffer) return;
    const redraw = this.whole || !this.sameStill(light, still);
    if (redraw && !this.drawStillCasters(pass, map, still)) return;
    // SAFETY: both of PlayCanvas 2.23's devices copy a render target's depth with `copyRenderTarget(source, dest,
    // color, depth)` (WebGPU `copyTextureToTexture`, WebGL2 `blitFramebuffer`), outside a pass (`check:chunks`).
    (this.device as unknown as { copyRenderTarget(source: pc.RenderTarget, dest: pc.RenderTarget, color: boolean, depth: boolean): boolean }).copyRenderTarget(this.stillTarget!, map, false, true);
    if (redraw) this.draws.whole++;
    else this.draws.moving++;
    this.whole = false;
  }

  /** In the sun's pass: what moves, over the still casters' picture (or everything, if that couldn't be made). */
  drawPass(pass: ShadowPass): void {
    const { light, camera, shadowRenderer } = pass;
    const data = light.getRenderData(camera, 0);
    const casters = data.visibleCasters;
    const all = this.allCasters;
    all.length = 0;
    for (const mi of casters) all.push(mi);
    const copied = this.stillTarget !== null && !this.whole;
    if (copied) {
      casters.length = 0;
      for (const mi of all) if (this.moving.has(mi)) casters.push(mi);
    }
    try {
      // (Without the picture: cleared here, every caster drawn, as PlayCanvas would.)
      shadowRenderer.renderFace(light, camera, 0, !copied);
    } finally {
      casters.length = 0;
      for (const mi of all) casters.push(mi);
      all.length = 0;
    }
    // (What PlayCanvas's own `execute` does after it.)
    if (light.shadowUpdateOverrides?.[0] === pc.SHADOWUPDATE_THISFRAME) light.shadowUpdateOverrides[0] = pc.SHADOWUPDATE_NONE;
    light._shadowCascadesInvalidated = false;
  }

  /** Whether the still casters are those the picture was drawn of, as they were, seen the same way. */
  private sameStill(light: SceneLight, still: readonly pc.MeshInstance[]): boolean {
    const d = this.drawn;
    if (!this.stillTarget || light._shadowMap !== d.map || d.casters.length !== still.length) return false;
    const vp = this.viewProj.data;
    // (The fit's float noise as the camera moves isn't a change: `VIEW_NOISE`.)
    for (let k = 0; k < 16; k++) if (Math.abs(vp[k] - d.viewProj[k]) > VIEW_NOISE) return false;
    for (let i = 0; i < still.length; i++) {
      const mi = still[i];
      if (mi !== d.casters[i] || mi.instancingCount !== d.counts[i]) return false;
      const b = mi.aabb;
      const o = i * 6;
      const c = b.center;
      const h = b.halfExtents;
      if (c.x !== d.boxes[o] || c.y !== d.boxes[o + 1] || c.z !== d.boxes[o + 2] || h.x !== d.boxes[o + 3] || h.y !== d.boxes[o + 4] || h.z !== d.boxes[o + 5]) return false;
    }
    return true;
  }

  /** The still casters drawn into their picture (made, the map's size and format, the first time), and remembered. */
  private drawStillCasters(pass: ShadowPass, map: pc.RenderTarget, still: readonly pc.MeshInstance[]): boolean {
    const { light, camera, shadowRenderer } = pass;
    const depth = map.depthBuffer!;
    let target = this.stillTarget;
    if (target && (target.width !== map.width || target.height !== map.height || target.depthBuffer?.format !== depth.format)) {
      this.dropStill();
      target = null;
    }
    if (!target) {
      const texture = new pc.Texture(this.device, { name: 'sunStillCasters', format: depth.format, width: map.width, height: map.height, mipmaps: false, minFilter: pc.FILTER_NEAREST, magFilter: pc.FILTER_NEAREST });
      target = this.stillTarget = new pc.RenderTarget({ name: 'sunStillCasters', depthBuffer: texture });
      const p = (this.stillPass = new pc.RenderPass(this.device));
      p.init(target);
      p.depthStencilOps.clearDepth = true;
      p.depthStencilOps.clearDepthValue = 1;
      p.depthStencilOps.storeDepth = true;
    }
    const data = light.getRenderData(camera, 0);
    const casters = data.visibleCasters;
    const cam = data.shadowCamera;
    const all = this.allCasters;
    all.length = 0;
    for (const mi of casters) all.push(mi);
    casters.length = 0;
    for (const mi of still) casters.push(mi);
    const p = this.stillPass!;
    // (The light camera draws into the picture for the while: `renderFace` sizes its viewport by it.)
    const own = cam.renderTarget;
    cam.renderTarget = target;
    p.execute = () => shadowRenderer.renderFace(light, camera, 0, false);
    try {
      p.render();
    } finally {
      cam.renderTarget = own;
      casters.length = 0;
      for (const mi of all) casters.push(mi);
      all.length = 0;
    }
    this.remember(light, still);
    return true;
  }

  /** The light camera's view × projection (`viewProj`), as PlayCanvas's `setCameraUniforms` makes it (x and y). */
  private lightViewProj(cam: pc.Camera): void {
    const node = cam.node;
    this.view.setTRS(node.getPosition(), node.getRotation(), pc.Vec3.ONE).invert();
    this.viewProj.mul2(cam.projectionMatrix, this.view);
  }

  /** What the still casters' picture was drawn of: the light's view, the map, the casters with their boxes and copies. */
  private remember(light: SceneLight, still: readonly pc.MeshInstance[]): void {
    const d = this.drawn;
    d.viewProj.set(this.viewProj.data);
    d.map = light._shadowMap;
    d.casters.length = 0;
    if (d.boxes.length !== still.length * 6) d.boxes = new Float64Array(still.length * 6);
    if (d.counts.length !== still.length) d.counts = new Float64Array(still.length);
    for (let i = 0; i < still.length; i++) {
      const mi = still[i];
      d.casters.push(mi);
      d.counts[i] = mi.instancingCount;
      const b = mi.aabb;
      const o = i * 6;
      d.boxes[o] = b.center.x;
      d.boxes[o + 1] = b.center.y;
      d.boxes[o + 2] = b.center.z;
      d.boxes[o + 3] = b.halfExtents.x;
      d.boxes[o + 4] = b.halfExtents.y;
      d.boxes[o + 5] = b.halfExtents.z;
    }
  }

  private dropStill(): void {
    this.stillPass?.destroy();
    this.stillPass = null;
    this.stillTarget?.destroyTextureBuffers();
    this.stillTarget?.destroy();
    this.stillTarget = null;
    this.drawn.map = null;
  }

  private readonly restored = (): void => {
    this.whole = true;
  };

  /** Lets go of the still casters' picture. */
  destroy(): void {
    this.device.off('devicerestored', this.restored);
    this.dropStill();
  }
}
