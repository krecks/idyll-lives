/**
 * `?diag=albedo` and `?diag=normals` (`debug/benchFlags.ts`): every lit surface drawn as its base
 * colour or its world normal instead of its shading, for screenshots that compare geometry and
 * colour with the lighting left out (the PlayCanvas port is checked against these).
 *
 * A material plugin replaces the material's final colour after fog and image processing (the
 * renderer also turns the post-processing's image processing and the night grade off), so the
 * picture is the colour itself: the base colour in gamma space, the normal as `n * 0.5 + 0.5` in
 * game space. It runs last (priority 1000), after the other plugins have set the colour (the
 * residents' painted clothes, wallpaper, blankets). Unlit materials and the sky keep their look.
 */

import { MaterialPluginBase, ShaderLanguage, type Material, type Nullable } from './core';
import { benchDiag } from '../../debug/benchFlags';

/** What the picture shows instead of the shading, if a diag view is asked for (read once). */
const VIEW: 'albedo' | 'normals' | null = (() => {
  const diag = benchDiag();
  return diag.has('albedo') ? 'albedo' : diag.has('normals') ? 'normals' : null;
})();

/** Whether a diag view replaces the shading (the renderer then leaves the image processing out). */
export const diagViewActive = VIEW !== null;

class DiagViewPlugin extends MaterialPluginBase {
  constructor(material: Material) {
    super(material, 'DiagView', 1000, { DIAGVIEW: true }, true, true);
  }

  override getClassName(): string {
    return 'DiagViewPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    if (shaderType !== 'fragment') return null;
    const webgpu = shaderLanguage === ShaderLanguage.WGSL;
    const v3 = webgpu ? 'vec3f' : 'vec3';
    const decl = webgpu ? 'var' : 'vec3';
    // The base colour as the soft look takes it: metals in their own colour, not PBR's black diffuse.
    const shown =
      VIEW === 'normals'
        ? `normalW * 0.5 + ${v3}(0.5)`
        : `pow(max(diagBase, ${v3}(0.0)), ${v3}(1.0 / 2.2))`;
    return {
      CUSTOM_FRAGMENT_BEFORE_FRAGCOLOR: `
#if defined(DIAGVIEW) && !defined(UNLIT)
${decl} diagBase${webgpu ? ': vec3f' : ''} = surfaceAlbedo;
#ifdef METALLICWORKFLOW
diagBase = surfaceAlbedo + albedoOpacityOut.surfaceAlbedo * reflectivityOut.metallic;
#endif
finalColor = ${webgpu ? 'vec4f' : 'vec4'}(${shown}, finalColor.a);
#endif
`,
    };
  }
}

/** Materials already given the view (a material can be adopted more than once). */
const viewed = new WeakSet<Material>();

/** Draws `material` (a PBR material) as the diag view asks, if one does; once per material. */
export function applyDiagView(material: Material): void {
  if (!VIEW || viewed.has(material)) return;
  viewed.add(material);
  new DiagViewPlugin(material);
}
