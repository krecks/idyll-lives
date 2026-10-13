/**
 * Dev-only look test for the character bench (`/characters.html?style=cel|soft`), the model
 * gallery (`/gallery.html?keys=model.fridge&style=cel|soft`) and the game on the dev server
 * (`/?bench=village&style=cel|soft`, see `toonGame`): residents, items and houses shaded in other
 * art styles, to see how they would look. Not part of the build.
 *
 * - `cel`: Dispatch's look (cel shading): two flat tones split hard at the terminator and at the
 *   edges of cast shadows, a cool shadow tone, a thin rim light on the lit side and ink outlines
 *   (inverted hulls). Residents' faces get jagged patches: the hard split follows the generated
 *   heads' normals.
 * - `soft`: Screenbound's look (stylised, hand-painted): a wide, wrapped light ramp from cool
 *   shadows to warm light, a soft rim, a little more saturation and no outlines.
 *
 * A material plugin replaces the PBR result just before fog and image processing, so ACES and the
 * exposure still apply. Both keep the sun's cast shadows (only the sun casts them) and the
 * emissive (the eyes' catchlights) and leave lamps out, so the test is for daylight. Metals are
 * shaded in their base colour (PBR gives them no diffuse colour); unlit materials keep their own
 * look. GLSL (the character bench, WebGL2) and WGSL (the gallery and the game, WebGPU).
 */

import { MaterialPluginBase, Mesh, ShaderLanguage, type AbstractEngine, type Material, type MaterialDefines, type Nullable, type Scene, type SubMesh, type UniformBuffer, type Vector3 } from '../render/babylon/core';
import { RegisterMaterialPlugin } from '@babylonjs/core/Materials/materialPluginManager';
import '@babylonjs/core/Rendering/outlineRenderer';

export type ToonStyle = 'cel' | 'soft';

const GLSL: Record<ToonStyle, string> = {
  cel: `
float aa = max(fwidth(nl), 1e-4);
float lit = smoothstep(0.05 - aa, 0.05 + aa, nl) * step(0.5, sunShadow);
vec3 shade = mix(vec3(1.0, 1.05, 1.35), vec3(2.1, 2.0, 1.85), lit);
float fres = 1.0 - max(dot(normalW, viewDirectionW), 0.0);
float rimAA = max(fwidth(fres), 1e-4);
float rim = smoothstep(0.78 - rimAA, 0.78 + rimAA, fres) * lit;
vec3 toon = base * shade * (1.0 + 0.4 * rim);
`,
  soft: `
float ramp = smoothstep(-0.35, 0.75, nl) * mix(0.45, 1.0, sunShadow);
vec3 shade = mix(vec3(1.0, 1.08, 1.45), vec3(2.1, 2.0, 1.85), ramp) * (0.92 + 0.12 * normalW.y);
float fres = pow(1.0 - max(dot(normalW, viewDirectionW), 0.0), 3.0);
vec3 toon = base * shade * (1.0 + 0.35 * fres);
toon = mix(vec3(dot(toon, vec3(0.2126, 0.7152, 0.0722))), toon, 1.1);
`,
};

const WGSL: Record<ToonStyle, string> = {
  cel: `
let aa = max(fwidth(nl), 1e-4);
let lit = smoothstep(0.05 - aa, 0.05 + aa, nl) * step(0.5, sunShadow);
let shade = mix(vec3f(1.0, 1.05, 1.35), vec3f(2.1, 2.0, 1.85), lit);
let fres = 1.0 - max(dot(normalW, viewDirectionW), 0.0);
let rimAA = max(fwidth(fres), 1e-4);
let rim = smoothstep(0.78 - rimAA, 0.78 + rimAA, fres) * lit;
var toon = base * shade * (1.0 + 0.4 * rim);
`,
  soft: `
let ramp = smoothstep(-0.35, 0.75, nl) * mix(0.45, 1.0, sunShadow);
let shade = mix(vec3f(1.0, 1.08, 1.45), vec3f(2.1, 2.0, 1.85), ramp) * (0.92 + 0.12 * normalW.y);
let fres = pow(1.0 - max(dot(normalW, viewDirectionW), 0.0), 3.0);
var toon = base * shade * (1.0 + 0.35 * fres);
toon = mix(vec3f(dot(toon, vec3f(0.2126, 0.7152, 0.0722))), toon, 1.1);
`,
};

/**
 * Shades every PBR material created from now on that `applies` in `style`, lit by a sun that
 * shines towards `sunDirection()` (read at every draw, so it follows the time of day). Call it
 * before the materials are made.
 */
export function useToonLook(style: ToonStyle, sunDirection: () => Vector3, applies: (material: Material) => boolean): void {
  // Only the sun casts shadows: every other light adds 1 to aggShadow.
  const glsl = `#if defined(SIMTOON) && !defined(UNLIT)
{
  float nl = dot(normalW, -normalize(toonSun));
  float sunShadow = clamp(aggShadow * numLights - (numLights - 1.0), 0.0, 1.0);
#ifdef METALLICWORKFLOW
  vec3 base = surfaceAlbedo + albedoOpacityOut.surfaceAlbedo * reflectivityOut.metallic;
#else
  vec3 base = surfaceAlbedo;
#endif
${GLSL[style]}
  finalColor.rgb = max(toon, vec3(0.0)) + finalEmissive;
}
#endif
`;
  const wgsl = `#if defined(SIMTOON) && !defined(UNLIT)
{
  let nl = dot(normalW, -normalize(uniforms.toonSun));
  let sunShadow = clamp(aggShadow * numLights - (numLights - 1.0), 0.0, 1.0);
#ifdef METALLICWORKFLOW
  let base = surfaceAlbedo + albedoOpacityOut.surfaceAlbedo * reflectivityOut.metallic;
#else
  let base = surfaceAlbedo;
#endif
${WGSL[style]}
  finalColor = vec4f(max(toon, vec3f(0.0)) + finalEmissive, finalColor.a);
}
#endif
`;

  class ToonPlugin extends MaterialPluginBase {
    constructor(material: Material) {
      super(material, 'SimToon', 300, { SIMTOON: false }, true, false);
      this.registerForExtraEvents = true;
      this._enable(true);
    }

    override getClassName(): string {
      return 'SimToonPlugin';
    }

    override isCompatible(): boolean {
      return true;
    }

    override prepareDefines(defines: MaterialDefines): void {
      defines.SIMTOON = true;
    }

    override getUniforms(): { externalUniforms: string[] } {
      return { externalUniforms: ['toonSun'] };
    }

    override hardBindForSubMesh(_ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, subMesh: SubMesh): void {
      const d = sunDirection();
      subMesh.effect?.setFloat3('toonSun', d.x, d.y, d.z);
    }

    override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
      if (shaderType !== 'fragment') return null;
      const webgpu = shaderLanguage === ShaderLanguage.WGSL;
      return {
        CUSTOM_FRAGMENT_DEFINITIONS: webgpu ? 'uniform toonSun: vec3f;' : 'uniform vec3 toonSun;',
        CUSTOM_FRAGMENT_BEFORE_FOG: webgpu ? wgsl : glsl,
      };
    }
  }

  RegisterMaterialPlugin('SimToon', (material) => (material.getClassName() === 'PBRMaterial' && applies(material) ? new ToonPlugin(material) : null));
}

/**
 * Ink outlines round `meshes`: Babylon's outline renderer draws each mesh grown along its normals
 * by `width` metres, before the mesh, and writes the grown mesh's depth after it (so what is drawn
 * later, like the floors, doesn't cover the ink). Changed here into an inverted hull: only the
 * grown mesh's back faces are drawn. The renderer would draw its front faces, which cover
 * neighbouring parts of the same model drawn before it, and a resident's eyes, brows and mouth
 * drawn after the body.
 */
export function addInkOutlines(scene: Scene, meshes: Mesh[], width: number): void {
  for (const mesh of meshes) {
    mesh.renderOutline = true;
    mesh.outlineWidth = width;
    mesh.outlineColor.set(0.13, 0.09, 0.08);
  }
  const outlines = scene.getOutlineRenderer() as unknown as { render: OutlineRender & { inverted?: true } };
  if (outlines.render.inverted) return;
  const render = outlines.render.bind(outlines);
  const engine = scene.getEngine();
  outlines.render = Object.assign((...args: Parameters<OutlineRender>) => {
    // Culling on, the winding reversed: the front faces are culled.
    engine.setState(true, 0, false, true);
    render(...args);
  }, { inverted: true as const });
}

type OutlineRender = (subMesh: SubMesh, batch: unknown, useOverlay?: boolean, renderPassId?: number) => void;

/**
 * The look test in the game (`BabylonRenderer.init` calls it on `?style=`): every lit material, and
 * with `cel` (unless `?outline=0`) ink round residents, furniture and, while `wallsUp()`, the house,
 * given as meshes appear. Nature (trees, bushes, grass, flowers, rocks) gets none. The house's
 * outlines don't follow the wall cutaway, which lowers walls in the vertex shader, so they show
 * only with the walls up.
 */
export function toonGame(scene: Scene, style: ToonStyle, sunDirection: () => Vector3, wallsUp: () => boolean): void {
  useToonLook(style, sunDirection, () => true);
  if (style !== 'cel' || new URLSearchParams(location.search).get('outline') === '0') return;
  const seen = new WeakSet<Mesh>();
  const house: Mesh[] = [];
  scene.onBeforeRenderObservable.add(() => {
    for (const mesh of scene.meshes) {
      if (!(mesh instanceof Mesh) || !mesh.material || seen.has(mesh)) continue;
      seen.add(mesh);
      const name = mesh.name;
      if (/^sim:(body|cloth|hair)$/.test(mesh.material.name)) addInkOutlines(scene, [mesh], 0.012);
      else if (name.startsWith('model.') && !/^model\.(tree|pine|spruce|bush|grassTuft|flowers|rock|hedge)/.test(name)) addInkOutlines(scene, [mesh], 0.018);
      else if (/^(walls|plinth|trim|roof)/.test(name)) {
        addInkOutlines(scene, [mesh], 0.03);
        house.push(mesh);
      }
    }
    const up = wallsUp();
    for (const mesh of house) mesh.renderOutline = up;
  });
}
