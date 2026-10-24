/**
 * The picture after the light (the port of the game's Babylon `DefaultRenderingPipeline`,
 * `setupPostProcessing` and `applyLook` in `babylon/BabylonRenderer.ts`, and its night grade):
 *
 * - The scene is drawn in linear light into a half-float target (Babylon's HDR pipeline texture),
 *   multisampled as the preset says (`QualitySettings.msaaSamples`: Babylon's `samples` on its first
 *   post-process), by PlayCanvas's `CameraFrame`, which then composes it onto the canvas.
 * - The compose runs Babylon's image processing (`shaders/post.ts`: exposure, vignette, ACES,
 *   gamma, contrast, colour curves) and then the moonlight grade (`nightGrade.ts`).
 * - FXAA where Babylon had it, on Low (no multisampling): the compose draws into a target of its
 *   own and Babylon's FXAA pass draws that onto the canvas, the grade at its end (Babylon's
 *   order: image processing, FXAA, grade).
 * - `?diag=albedo|normals` and `?diag=linear` leave the image processing and the grade out (the
 *   colour itself, encoded for the screen); as on Babylon, without them only Low's FXAA is left,
 *   so Medium and High draw straight onto the canvas there. `?diag=noaa`: no multisampling, no
 *   FXAA; `?diag=nonight`: no grade.
 *
 * The drawing size is the canvas's (`shared/resolution.ts`, `applyResolution`), all passes at
 * full size, as Babylon's post-processes were. Live changes (the exposure each lighting step, the
 * night's amount, the look) are uniform values: no shader compiles, nothing allocated.
 *
 * Light units: the scene's linear colour is what Babylon's materials gave its image processing,
 * and the exposure is Babylon's (`l.exposure * LOOK.exposure`), so the light's strengths can
 * be Babylon's. Nothing else may tone-map: `scene.exposure` and the camera's tone mapping are
 * unused while the chain runs. The clear colour is drawn into the linear scene, so it is linear
 * too (`sceneLinear`).
 */

import * as pc from './engine';
import { NIGHT_TINT, nightAmount } from './nightGrade';
import { composeChunks, FXAA, PAGE_TONE_MAPPING, pageToneMapping } from './shaders/post';
import { curveUniforms, NEUTRAL_CURVES, vignetteUniforms, type ColorCurveSettings, type VignetteSettings } from '../shared/imageProcessing';

export interface PostOptions {
  /** Multisampling of the scene (1: none). */
  samples: number;
  /** Babylon's FXAA after the image processing. */
  fxaa: boolean;
  /** Babylon's image processing (off: the colour itself, the diag views). */
  imageProcessing: boolean;
  /** The moonlight grade (`nightGrade.ts`). */
  night: boolean;
}

/** What the image processing does (Babylon's `ImageProcessingConfiguration`, ACES always). */
export interface PostLook {
  contrast: number;
  /** Weight 0: no vignette. */
  vignette: VignetteSettings;
  curves: ColorCurveSettings;
}

/** The game's options for `quality` and the bench's `?diag=` flags, as `setupPostProcessing` chose them on Babylon. */
export function gamePostOptions(samples: number, diag: ReadonlySet<string>, colourItself: boolean): PostOptions {
  const aa = !diag.has('noaa');
  const imageProcessing = !colourItself && !diag.has('linear');
  return {
    // (Without the image processing Babylon's pipeline had no pass to multisample but the FXAA, on Low.)
    samples: aa && imageProcessing ? samples : 1,
    fxaa: aa && samples <= 1,
    imageProcessing,
    night: imageProcessing && !diag.has('nonight'),
  };
}

/** The game's look (`look.ts` through `applyLook` and `setupPostProcessing`). */
export function gameLook(look: { contrast: number; vignette: number; saturation: number; highlightsHue: number; highlightsDensity: number; shadowsHue: number; shadowsDensity: number }): PostLook {
  return {
    contrast: look.contrast,
    vignette: { weight: look.vignette, stretch: 0.5, color: [0.02, 0.02, 0.04] },
    curves: { ...NEUTRAL_CURVES, globalSaturation: look.saturation, highlightsHue: look.highlightsHue, highlightsDensity: look.highlightsDensity, shadowsHue: look.shadowsHue, shadowsDensity: look.shadowsDensity },
  };
}

/** No contrast, vignette or curves: exposure and ACES only (the dev pages). */
export const PLAIN_LOOK: PostLook = { contrast: 1, vignette: { weight: 0, stretch: 0, color: [0, 0, 0] }, curves: NEUTRAL_CURVES };

const SCENE_FORMATS = [pc.PIXELFORMAT_RGBA16F, pc.PIXELFORMAT_RGBA32F];

/** Babylon's FXAA pass: `source` onto the canvas (or `target`). */
class FxaaPass extends pc.RenderPassShaderQuad {
  private readonly sourceId: pc.ScopeId;
  private readonly texelId: pc.ScopeId;
  private readonly texel = new Float32Array(2);

  constructor(
    device: pc.GraphicsDevice,
    private readonly source: pc.Texture,
    night: boolean,
    target: pc.RenderTarget | null = null,
  ) {
    super(device);
    const defines = new Map<string, string>();
    if (night) defines.set('GAME_NIGHT', '');
    if ((target ?? device.backBuffer)?.isColorBufferSrgb(0)) defines.set('GAME_SRGB_TARGET', '');
    if (/mali/i.test(driverInfo(device))) defines.set('MALI', '');
    this.shader = pc.ShaderUtils.createShader(device, {
      uniqueName: `GameFxaa-${[...defines.keys()].join('-')}`,
      attributes: { aPosition: pc.SEMANTIC_POSITION },
      vertexChunk: 'quadVS',
      fragmentGLSL: FXAA.glsl,
      fragmentWGSL: FXAA.wgsl,
      fragmentDefines: defines,
    });
    this.sourceId = device.scope.resolve('gameFxaaSource');
    this.texelId = device.scope.resolve('gameFxaaTexel');
  }

  override execute(): void {
    this.texel[0] = 1 / this.source.width;
    this.texel[1] = 1 / this.source.height;
    this.sourceId.setValue(this.source);
    this.texelId.setValue(this.texel);
    super.execute();
  }
}

/** The GPU's name, where Babylon read it for its FXAA's `MALI` (`extractDriverInfo`). */
function driverInfo(device: pc.GraphicsDevice): string {
  const d = device as unknown as { unmaskedRenderer?: string; gpuAdapter?: { info?: { vendor?: string; architecture?: string; description?: string } } };
  const info = d.gpuAdapter?.info;
  return [d.unmaskedRenderer, info?.vendor, info?.architecture, info?.description].filter(Boolean).join(' ');
}

/** Babylon's half-float FXAA input, `width` × `height`: the compose draws into it, the FXAA pass from it. */
function fxaaSource(device: pc.GraphicsDevice, width: number, height: number): pc.RenderTarget {
  const texture = new pc.Texture(device, {
    name: 'gameFxaaSource',
    width: Math.max(1, width),
    height: Math.max(1, height),
    format: device.getRenderableHdrFormat([pc.PIXELFORMAT_RGBA16F], true) ?? pc.PIXELFORMAT_RGBA8,
    mipmaps: false,
    minFilter: pc.FILTER_LINEAR,
    magFilter: pc.FILTER_LINEAR,
    addressU: pc.ADDRESS_CLAMP_TO_EDGE,
    addressV: pc.ADDRESS_CLAMP_TO_EDGE,
  });
  return new pc.RenderTarget({ name: 'gameFxaaSource', colorBuffer: texture, depth: false });
}

/** The game's chain on another camera (`PostProcessing.attach`). */
export interface PostAttachment {
  /** The target was resized to `width` × `height`. */
  resize(width: number, height: number): void;
  destroy(): void;
}

export class PostProcessing {
  /** The chain on the camera (null: the camera draws straight onto the canvas). */
  readonly frame: pc.CameraFrame | null = null;
  /** Whether the scene is drawn into the chain, in linear light: then its clear colour is linear too. */
  readonly sceneLinear: boolean;
  private fxaa: FxaaPass | null = null;
  private fxaaTarget: pc.RenderTarget | null = null;
  private readonly device: pc.GraphicsDevice;
  private readonly ids: Record<'exposure' | 'contrast' | 'vignette1' | 'vignette2' | 'neutral' | 'positive' | 'negative' | 'night' | 'nightTint', pc.ScopeId>;
  private readonly vignette1 = new Float32Array(4);
  private readonly vignette2 = new Float32Array(4);
  private readonly curves = { neutral: new Float32Array(4), positive: new Float32Array(4), negative: new Float32Array(4) };
  private readonly tint = new Float32Array(NIGHT_TINT);
  private look: PostLook = PLAIN_LOOK;
  private exposureNow = 1;
  private nightNow = 0;
  private readonly onResize = (width: number, height: number) => this.resized(width, height);

  constructor(
    app: pc.AppBase,
    private readonly camera: pc.CameraComponent,
    readonly options: Readonly<PostOptions>,
  ) {
    const device = (this.device = app.graphicsDevice);
    const scope = device.scope;
    this.ids = {
      exposure: scope.resolve('gameExposure'),
      contrast: scope.resolve('gameContrast'),
      vignette1: scope.resolve('gameVignette1'),
      vignette2: scope.resolve('gameVignette2'),
      neutral: scope.resolve('gameCurveNeutral'),
      positive: scope.resolve('gameCurvePositive'),
      negative: scope.resolve('gameCurveNegative'),
      night: scope.resolve('gameNightAmount'),
      nightTint: scope.resolve('gameNightTint'),
    };
    const chained = options.imageProcessing || options.fxaa;
    this.sceneLinear = chained;
    if (!chained) {
      // Straight onto the canvas: the colour, encoded for the screen.
      camera.toneMapping = pc.TONEMAP_LINEAR;
      camera.gammaCorrection = pc.GAMMA_SRGB;
      app.scene.exposure = 1;
      return;
    }
    // The compose's chunks (one set per device: the game has one camera with a chain).
    for (const lang of ['glsl', 'wgsl'] as const) {
      const chunks = pc.ShaderChunks.get(device, lang === 'glsl' ? pc.SHADERLANGUAGE_GLSL : pc.SHADERLANGUAGE_WGSL);
      const code = composeChunks(lang, { imageProcessing: options.imageProcessing, night: options.night && !options.fxaa, toFxaa: options.fxaa });
      chunks.set('composeDeclarationsPS', code.declarations);
      chunks.set('composeMainEndPS', code.mainEnd);
    }
    if (options.fxaa) {
      // The compose draws into this (Babylon's half-float FXAA input), the FXAA pass onto the canvas.
      this.fxaaTarget = fxaaSource(device, device.width, device.height);
      // (Before the frame: its passes take the camera's target, and size the scene by it.)
      camera.renderTarget = this.fxaaTarget;
    }
    this.frame = this.cameraFrame(app, camera);
    if (this.fxaaTarget) {
      const fxaa = (this.fxaa = new FxaaPass(device, this.fxaaTarget.colorBuffer, options.night));
      fxaa.init(null);
      camera.framePasses = [...camera.framePasses, fxaa];
    }
    this.setLook(PLAIN_LOOK);
    this.exposure = 1;
    this.night = 0;
    this.ids.nightTint.setValue(this.tint);
    device.on('resizecanvas', this.onResize);
    this.resized(device.width, device.height);
  }

  /** The chain's frame on `camera`: the scene in linear light, multisampled, composed by the image processing. */
  private cameraFrame(app: pc.AppBase, camera: pc.CameraComponent): pc.CameraFrame {
    const frame = new pc.CameraFrame(app, camera);
    frame.rendering.renderFormats = SCENE_FORMATS;
    frame.rendering.samples = Math.max(1, this.options.samples);
    frame.rendering.toneMapping = pc.TONEMAP_NONE;
    frame.rendering.sharpness = 0;
    frame.update();
    return frame;
  }

  /**
   * The same picture from another camera, into `target` (the save's picture, `thumbnail.ts`): the
   * game camera's passes at the target's size, with the same uniforms (the exposure, the look, the
   * night are the device's, set once for both). The vignette keeps the canvas's proportions, as
   * Babylon's did in its picture (its post-processes were the canvas's size).
   */
  attach(app: pc.AppBase, camera: pc.CameraComponent, target: pc.RenderTarget): PostAttachment {
    if (!this.frame) {
      // Straight into the target: the colour, encoded as the game camera's.
      camera.toneMapping = this.camera.toneMapping;
      camera.gammaCorrection = this.camera.gammaCorrection;
      camera.renderTarget = target;
      return { resize: () => {}, destroy: () => {} };
    }
    const source = this.options.fxaa ? fxaaSource(this.device, target.width, target.height) : null;
    camera.renderTarget = source ?? target;
    const frame = this.cameraFrame(app, camera);
    let fxaa: FxaaPass | null = null;
    if (source) {
      fxaa = new FxaaPass(this.device, source.colorBuffer, this.options.night, target);
      fxaa.init(target);
      camera.framePasses = [...camera.framePasses, fxaa];
    }
    return {
      resize: (width, height) => source?.resize(width, height),
      destroy: () => {
        frame.destroy();
        fxaa?.destroy();
        source?.destroyTextureBuffers();
        source?.destroy();
      },
    };
  }

  /** Babylon's `ImageProcessingConfiguration.exposure` (the scene's colour times this, before tone mapping). */
  set exposure(value: number) {
    this.exposureNow = value;
    this.ids.exposure.setValue(value);
  }

  get exposure(): number {
    return this.exposureNow;
  }

  /** The moonlight grade's amount, 0 (none, by day) .. 1 (`nightGrade.ts`). */
  set night(amount: number) {
    this.nightNow = this.options.night ? nightAmount(amount) : 0;
    this.ids.night.setValue(this.nightNow);
  }

  get night(): number {
    return this.nightNow;
  }

  /** Contrast, vignette and colour curves (live: uniforms only). */
  setLook(look: PostLook): void {
    this.look = look;
    this.ids.contrast.setValue(look.contrast);
    curveUniforms(look.curves, this.curves);
    this.ids.neutral.setValue(this.curves.neutral);
    this.ids.positive.setValue(this.curves.positive);
    this.ids.negative.setValue(this.curves.negative);
    this.resized(this.device.width, this.device.height);
  }

  /** Bench reports: what Babylon's `benchInfo` had as `postSamples` and `fxaa`. */
  get samples(): number {
    return this.frame ? Math.max(1, this.options.samples) : 1;
  }

  private resized(width: number, height: number): void {
    // The vignette's shape follows the picture's proportions.
    vignetteUniforms(width, height, this.look.vignette, this.vignette1, this.vignette2);
    this.ids.vignette1.setValue(this.vignette1);
    this.ids.vignette2.setValue(this.vignette2);
    if (this.fxaaTarget && (this.fxaaTarget.width !== width || this.fxaaTarget.height !== height)) this.fxaaTarget.resize(Math.max(1, width), Math.max(1, height));
  }

  destroy(): void {
    this.device.off('resizecanvas', this.onResize);
    this.frame?.destroy();
    this.fxaa?.destroy();
    if (this.fxaaTarget) {
      // (Back onto the canvas: null, the default, which PlayCanvas's types leave out.)
      if (this.camera.renderTarget === this.fxaaTarget) this.camera.renderTarget = null as unknown as pc.RenderTarget;
      this.fxaaTarget.destroyTextureBuffers();
      this.fxaaTarget.destroy();
    }
  }
}

/**
 * The dev pages' look (`debug/pcStage.ts`): Babylon's pages did the image processing in each
 * material (exposure and ACES; the clear colour and the diag views' colour as they are, the
 * canvas multisampled), so here it is the camera's tone mapping, made Babylon's on the page's
 * device (`PAGE_TONE_MAPPING`; with `contrast`, Babylon's contrast after it, as the previews'
 * scenes set it). Before the first material compiles.
 */
export function pageLook(app: pc.AppBase, camera: pc.CameraComponent, exposure: number, contrast = 1): void {
  const device = app.graphicsDevice;
  // (With Babylon's contrast after it: the previews' scenes, `pageToneMapping`.)
  const chunk = contrast === 1 ? PAGE_TONE_MAPPING : pageToneMapping(contrast);
  pc.ShaderChunks.get(device, pc.SHADERLANGUAGE_GLSL).set('tonemappingAces2PS', chunk.glsl);
  pc.ShaderChunks.get(device, pc.SHADERLANGUAGE_WGSL).set('tonemappingAces2PS', chunk.wgsl);
  camera.toneMapping = pc.TONEMAP_ACES2;
  camera.gammaCorrection = pc.GAMMA_SRGB;
  app.scene.exposure = exposure;
}
