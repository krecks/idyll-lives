// The fade at the map's edge follows Babylon.js 9.30's shadow `frustumEdgeFalloff`. Changed for
// Idyll Lives: as a patch of PlayCanvas's `lightFunctionShadowPS`.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The sun's shadow map in the game: it covers a fixed square round the viewed lot, so its texels
 * keep their size and place while the camera turns and the sun steps, and it is drawn again only
 * when something in it changed (`moved`): the sun's step, a resident's pose (throttled to the
 * preset's `shadowRate`), a thing placed, the walls. Casters outside the square aren't drawn. The menus' town is fitted round the camera's
 * view instead, and drawn every frame.
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

/** Anchors of the edge patch in `lightFunctionShadowPS` (also in `tools/render/check_chunks.ts`). */
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
  private readonly fadeId: pc.ScopeId;
  private readonly fade: number;
  /** Something in the map changed (`moved`; `urgent`: not a resident's pose), for what else holds a picture of the world (the mirrors). */
  onMoved: ((urgent: boolean) => void) | null = null;

  /**
   * Shadows from `sun`'s light: a map of `options.size` texels a side, filtered by 3×3 PCF on every
   * preset, the receivers offset by a share of a texel.
   */
  constructor(app: pc.AppBase, sun: pc.Entity, options: SunShadowOptions) {
    patch(app);
    patchShadowEdge(app.graphicsDevice);
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
    this.applyBias();
    this.moved();
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
   * Something in the map changed: it is drawn again in the next frame. `urgent` changes (the sun,
   * the walls, a thing placed) draw at once; a resident's pose may wait for the shadow rate.
   */
  moved(urgent = true): void {
    this.onMoved?.(urgent);
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
}
