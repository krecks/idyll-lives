/**
 * The soft look: PBR's light, drawn as a wide, soft ramp in the light's own colour.
 *
 * PBR shades every pixel with the light that reaches it. The soft look keeps the amount and the
 * colour of that light but redraws it as a wide ramp, and keeps the environment around it (the
 * sky, with its occlusion) at a fraction, so shadows stay cool and readable instead of dark. The
 * sun and the lamps both go through the ramp, so golden hour stays golden, lamps stay warm and
 * night stays dark; a soft rim, a little more saturation and no outlines finish it.
 *
 * A material plugin replaces the material's colour just before fog and image processing, so ACES,
 * the exposure and the night grade still apply. It runs after the material's other plugins
 * (priority 300), so the wall cutaway, basements and plain finishes have done their part. Metals
 * are shaded in their base colour (PBR gives them no diffuse colour); unlit materials keep their
 * own look. GLSL (WebGL2) and WGSL (WebGPU).
 *
 * `applySoftLook` is called by `MaterialLibrary` for every PBR material it makes or adopts, which
 * is every lit surface in the game and its tools. Water, glass panes that keep their reflections
 * and the sky are left out on purpose (see `docs/design/look.md`).
 *
 * It is the game's look (Settings, Performance, Shading: Soft). `classic` draws the PBR shading it
 * replaced, and switching is live: the plugin sets its define from the setting every time a
 * material's shaders are prepared, so `refreshSoftLook` is all it takes to redraw every scene's
 * materials the other way (the game's and the previews'; pictures already drawn are cached by
 * shading). `?look=soft|classic` on the dev server overrides the setting.
 */

import { MaterialPluginBase, ShaderLanguage, type Material, type MaterialDefines, type Nullable, type Scene } from './core';
import { settings } from '../../settings/settings.svelte';
import type { Shading } from '../types';

/** The plugin's name (`material.pluginManager.getPlugin`); also its define's, lowercased. */
const PLUGIN = 'SoftLook';

const GLSL = `
// The PBR result, split into the light that falls on the surface (the sun and the lamps, with
// their cast shadows) and the environment around it (the sky, with its occlusion). The stylised
// ramp replaces the first; the second keeps the scene's ambient shading. The light's own colour
// comes through, so golden hour and night lamps keep their warmth.
vec3 pbr = max(finalColor.rgb - finalEmissive, vec3(0.0));
vec3 ambient = max(pbr - finalDiffuse, vec3(0.0));
vec3 albedoLum = vec3(max(dot(base, vec3(0.2126, 0.7152, 0.0722)), 0.06));
vec3 light = finalDiffuse / albedoLum;
float lightAmt = dot(light, vec3(0.2126, 0.7152, 0.0722));
vec3 lightCol = light / max(lightAmt, 1e-3);
float lit = smoothstep(0.0, 1.3, lightAmt);
vec3 shade = mix(vec3(0.72, 0.8, 1.15), vec3(2.3, 2.15, 1.85), lit);
vec3 soft = ambient * 0.6 + base * shade * lightCol * clamp(lightAmt * 0.42, 0.0, 1.0);
float fres = pow(1.0 - max(dot(normalW, viewDirectionW), 0.0), 3.0);
soft *= 1.0 + 0.35 * fres;
soft = mix(vec3(dot(soft, vec3(0.2126, 0.7152, 0.0722))), soft, 1.15);
finalColor.rgb = max(soft, vec3(0.0)) + finalEmissive;
`;

const WGSL = `
// See the GLSL block: the PBR light is split from the environment, and the ramp replaces the
// first (in the light's own colour) while the second keeps the ambient shading.
let pbr = max(finalColor.rgb - finalEmissive, vec3f(0.0));
let ambient = max(pbr - finalDiffuse, vec3f(0.0));
let albedoLum = vec3f(max(dot(base, vec3f(0.2126, 0.7152, 0.0722)), 0.06));
let light = finalDiffuse / albedoLum;
let lightAmt = dot(light, vec3f(0.2126, 0.7152, 0.0722));
let lightCol = light / max(lightAmt, 1e-3);
let lit = smoothstep(0.0, 1.3, lightAmt);
let shade = mix(vec3f(0.72, 0.8, 1.15), vec3f(2.3, 2.15, 1.85), lit);
var soft = ambient * 0.6 + base * shade * lightCol * clamp(lightAmt * 0.42, 0.0, 1.0);
let fres = pow(1.0 - max(dot(normalW, viewDirectionW), 0.0), 3.0);
soft *= 1.0 + 0.35 * fres;
soft = mix(vec3f(dot(soft, vec3f(0.2126, 0.7152, 0.0722))), soft, 1.15);
finalColor = vec4f(max(soft, vec3f(0.0)) + finalEmissive, finalColor.a);
`;

class SoftLookPlugin extends MaterialPluginBase {
  constructor(material: Material) {
    super(material, PLUGIN, 300, { SOFTLOOK: false }, true, false);
    this._enable(true);
  }

  override getClassName(): string {
    return 'SoftLookPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override prepareDefines(defines: MaterialDefines): void {
    // Read every time: the shaders follow the setting (`refreshSoftLook` re-prepares them).
    defines.SOFTLOOK = softLookWanted();
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType !== 'fragment') return null;
    const webgpu = shaderLanguage === ShaderLanguage.WGSL;
    return {
      CUSTOM_FRAGMENT_BEFORE_FOG: `
#if defined(SOFTLOOK) && !defined(UNLIT)
#ifdef METALLICWORKFLOW
  ${webgpu ? 'let' : 'vec3'} base = surfaceAlbedo + albedoOpacityOut.surfaceAlbedo * reflectivityOut.metallic;
#else
  ${webgpu ? 'let' : 'vec3'} base = surfaceAlbedo;
#endif
${webgpu ? WGSL : GLSL}
#endif
`,
    };
  }
}

/** Materials already shaded (a material can be adopted more than once). */
const shaded = new WeakSet<Material>();
/** Scenes with shaded materials, for `refreshSoftLook` (the game's, the previews' studios). */
const scenes = new Set<Scene>();

/**
 * Shades `material` (a PBR material) in the soft look, unless the setting says otherwise. Called
 * before its first draw, and once per material: a second plugin would shade the result twice.
 */
export function applySoftLook(material: Material): void {
  if (shaded.has(material)) return;
  shaded.add(material);
  new SoftLookPlugin(material);
  const scene = material.getScene();
  if (scenes.has(scene)) return;
  scenes.add(scene);
  scene.onDisposeObservable.addOnce(() => scenes.delete(scene));
}

/**
 * Prepares every shaded material's shaders again, so it is drawn the way the setting says now (the
 * plugin sets its define while they are prepared). `markDirty(true)`, not just dirty defines: a
 * frozen material skips preparing while its meshes were ready before, and only `markDirty` clears that.
 */
export function refreshSoftLook(): void {
  for (const scene of scenes) for (const material of scene.materials) if (material.pluginManager?.getPlugin(PLUGIN)) material.markDirty(true);
}

/** Whether the soft look is drawn: the setting (`settings.shading`), or `?look=` on the dev server. */
export function softLookWanted(): boolean {
  return (softLookFromURL() ?? settings.shading) === 'soft';
}

/** The shading the setting asks for (what `LiveRenderOptions.shading` carries). */
export function shadingFromSettings(): Shading {
  return softLookWanted() ? 'soft' : 'classic';
}

/** `?look=soft|classic|pbr` on the dev server: the look to draw whatever the setting says (read once). */
const LOOKS: Record<string, Shading> = { soft: 'soft', classic: 'classic', pbr: 'classic' };
let askedLook: Shading | null | undefined;
function softLookFromURL(): Shading | null {
  if (!import.meta.env.DEV) return null;
  if (askedLook === undefined) askedLook = LOOKS[new URLSearchParams(location.search).get('look') ?? ''] ?? null;
  return askedLook;
}
