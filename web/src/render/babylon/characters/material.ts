/**
 * Character materials: PBR (no textures) plus a plugin that reads each Sim's appearance from the
 * shared pose texture (the same texture that holds the skinning matrices; one row per Sim) and the
 * per-vertex `simField` and `simTone` attributes (docs/characters-format.md):
 *
 * - body: the Sim's skin colour, with the tone's blush (lips, cheeks) and cavity darkening; the
 *   clothes are painted on it: where a garment's cut covers the body's fields the skin takes the
 *   top, bottom or shoe colour (anti-aliased edges; a top cut `over` the bottom wins over it, else
 *   the bottom wins), rougher than skin; under hair, the scalp (head skin whose crew field is
 *   -1 - scalp weight) takes the hair colour;
 * - cloth: shells (jackets, skirts, shoes) in the flat top / bottom / shoe colour, soles darker;
 * - hair: the hair colour, with the tone's clump bands;
 * - brow: brows in the hair colour (darker); lash: lashes, near black;
 * - mouth: teeth, and the mouth plate darkened by its tone;
 * - eye: the eyeballs drawn procedurally: sclera, the iris in the Sim's eye colour with a darker
 *   limbal ring, the pupil (fields: offset from the iris centre, iris radius, pupil radius).
 *
 * Face morphs (expressions, blinks, speech; see `MorphSet` in data.ts) are added to the vertex
 * position and normal before skinning, on meshes with a `simMorph` attribute: each Sim's row holds
 * up to `MORPH_PAIRS` (channel, weight) pairs after the appearance texels.
 *
 * Row layout (RGBA32F texels): bones * 4 matrix texels, then `APPEARANCE_TEXELS` texels:
 *   0: skin colour rgb (linear), top: sleeve cut
 *   1: top colour rgb, top: hem height
 *   2: bottom colour rgb, top: crew neckline cut
 *   3: hair colour rgb, top: scoop neckline cut
 *   4: r = 1 if the top is painted over the bottom, g = age (deeper cavities: elders 1), bottom: waist height
 *   5: skin reference colour rgb (the skin is tinted relative to it; 1), bottom: leg cut
 *   6: shoe colour rgb, shoes: top height
 *   7: eye colour rgb, 1 if the Sim wears hair (the scalp takes the hair colour)
 * Disabled cuts are -10 (or +10 for "below" cuts) so their predicate always / never holds.
 * Then `MORPH_TEXELS` texels of (channel, weight, channel, weight); weight 0 = unused.
 */

import {
  MaterialPluginBase,
  ShaderLanguage,
  type AbstractEngine,
  type AbstractMesh,
  type BaseTexture,
  type Material,
  type MaterialDefines,
  type Nullable,
  type Scene,
  type SubMesh,
  type UniformBuffer,
} from '../core';

export const APPEARANCE_TEXELS = 8;
/** Face morph weights per Sim: (channel, weight) pairs, two per texel. */
export const MORPH_TEXELS = 12;
export const MORPH_PAIRS = MORPH_TEXELS * 2;

export type CharacterSurface = 'body' | 'cloth' | 'hair' | 'brow' | 'lash' | 'mouth' | 'eye';

/** The set's morph texture and its layout (texels per row, header texels per part block). */
export interface MorphBinding {
  texture: BaseTexture;
  width: number;
  header: number;
}

let pendingKind: CharacterSurface = 'eye';
let pendingOffset = 0;
let pendingMorph: MorphBinding | null = null;

export class CharacterPlugin extends MaterialPluginBase {
  private readonly kind: CharacterSurface;
  /** First appearance texel in a row (bones * 4). */
  private readonly offset: number;

  constructor(
    material: Material,
    kind: CharacterSurface,
    offset: number,
    private readonly data: BaseTexture,
    private readonly morph: MorphBinding | null = null,
  ) {
    // The base constructor asks for the shader code before the fields exist.
    pendingKind = kind;
    pendingOffset = offset;
    pendingMorph = morph;
    super(
      material,
      'SimCharacter',
      230,
      {
        SIMCHAR: false,
        SIMBODY: false,
        SIMCLOTH: false,
        SIMHAIR: false,
        SIMBROW: false,
        SIMLASH: false,
        SIMEYE: false,
        SIMMOUTH: false,
        SIMMORPH: false,
      },
      true,
      true,
    );
    this.kind = kind;
    this.offset = offset;
  }

  private get k(): CharacterSurface {
    return this.kind ?? pendingKind;
  }

  override getClassName(): string {
    return 'SimCharacterPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  private get m(): MorphBinding | null {
    return this.morph === undefined ? pendingMorph : this.morph;
  }

  override prepareDefines(defines: MaterialDefines, _scene: Scene, mesh: AbstractMesh): void {
    const kind = this.k;
    defines.SIMCHAR = true;
    defines.SIMBODY = kind === 'body';
    defines.SIMCLOTH = kind === 'cloth';
    defines.SIMHAIR = kind === 'hair';
    defines.SIMBROW = kind === 'brow';
    defines.SIMLASH = kind === 'lash';
    defines.SIMEYE = kind === 'eye';
    defines.SIMMOUTH = kind === 'mouth';
    defines.SIMMORPH = !!this.m && mesh.isVerticesDataPresent('simMorph');
  }

  override getAttributes(attributes: string[], _scene: Scene, mesh: AbstractMesh): void {
    attributes.push('simField', 'simTone');
    if (this.m && mesh.isVerticesDataPresent('simMorph')) attributes.push('simMorph');
  }

  override isReadyForSubMesh(): boolean {
    return this.data.isReady() && (!this.morph || this.morph.texture.isReady());
  }

  override getSamplers(samplers: string[]): void {
    samplers.push('simDataTexture');
    if (this.m) samplers.push('simMorphTexture');
  }

  override getActiveTextures(textures: BaseTexture[]): void {
    if (this.morph) textures.push(this.morph.texture);
  }

  override hasTexture(texture: BaseTexture): boolean {
    return texture === this.data || texture === this.morph?.texture;
  }

  override bindForSubMesh(ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, _subMesh: SubMesh): void {
    ubo.setTexture('simDataTexture', this.data);
    if (this.morph) ubo.setTexture('simMorphTexture', this.morph.texture);
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    const off = this.offset ?? pendingOffset;
    const morph = this.m;
    return shaderLanguage === ShaderLanguage.WGSL ? wgsl(shaderType, off, morph) : glsl(shaderType, off, morph);
  }
}

const GUARD = '#if defined(SIMCHAR) && defined(INSTANCES) && defined(BAKED_VERTEX_ANIMATION_TEXTURE)';
/** Microsurface (1 - roughness) of painted clothes; skin keeps the material's. */
const CLOTH_MICRO = '0.15';
/** Hair is matte: no more glossy than this (the sheen gives it its soft highlight). */
const HAIR_MICRO = '0.38';

function wgsl(shaderType: string, off: number, morph: MorphBinding | null): { [pointName: string]: string } {
  if (shaderType === 'vertex') {
    const W = morph?.width ?? 1;
    const H = morph?.header ?? 0;
    const pairs = off + APPEARANCE_TEXELS;
    return {
      CUSTOM_VERTEX_DEFINITIONS: `${GUARD}
attribute simField: vec4f;
attribute simTone: vec4f;
varying vSimSlot: f32;
varying vSimField: vec4f;
varying vSimTone: vec4f;
#ifdef SIMMORPH
attribute simMorph: vec4f;
var simMorphTexture: texture_2d<f32>;
fn simMorphTexel(i: i32) -> vec4f { return textureLoad(simMorphTexture, vec2<i32>(i % ${W}, i / ${W}), 0); }
#endif
#endif
`,
      CUSTOM_VERTEX_UPDATE_POSITION: `${GUARD}
#ifdef SIMMORPH
let mRow = i32(vertexInputs.bakedVertexAnimationSettingsInstanced.x + 0.5);
// A first channel of -1: no morphs on this face this frame (too small on screen).
if (vertexInputs.simMorph.x >= 0.0 && textureLoad(bakedVertexAnimationTexture, vec2<i32>(${pairs}, mRow), 0).x > -0.5) {
  let mBase = i32(vertexInputs.simMorph.x + 0.5);
  let mSlot = i32(vertexInputs.simMorph.y + 0.5);
  let mCount = i32(vertexInputs.simMorph.z + 0.5);
  for (var k = 0; k < ${MORPH_TEXELS}; k++) {
    let pw = textureLoad(bakedVertexAnimationTexture, vec2<i32>(${pairs} + k, mRow), 0);
    for (var j = 0; j < 2; j++) {
      let w = select(pw.y, pw.w, j == 1);
      if (abs(w) < 0.0001) { continue; }
      let ch = i32(select(pw.x, pw.z, j == 1) + 0.5);
      let lf = simMorphTexel(mBase + ch / 4)[ch % 4];
      if (lf < -0.5) { continue; }
      let t = mBase + ${H} + (i32(lf + 0.5) * mCount + mSlot) * 2;
      positionUpdated += simMorphTexel(t).xyz * w;
      normalUpdated += simMorphTexel(t + 1).xyz * w;
    }
  }
}
#endif
#endif
`,
      CUSTOM_VERTEX_MAIN_END: `${GUARD}\nvertexOutputs.vSimSlot = vertexInputs.bakedVertexAnimationSettingsInstanced.x;\nvertexOutputs.vSimField = vertexInputs.simField;\nvertexOutputs.vSimTone = vertexInputs.simTone;\n#endif\n`,
    };
  }
  return {
    CUSTOM_FRAGMENT_DEFINITIONS: `${GUARD}
varying vSimSlot: f32;
varying vSimField: vec4f;
varying vSimTone: vec4f;
var simDataTexture: texture_2d<f32>;
#endif
`,
    CUSTOM_FRAGMENT_MAIN_BEGIN: `${GUARD}
let simRow = i32(fragmentInputs.vSimSlot + 0.5);
let simF = fragmentInputs.vSimField;
let simT = fragmentInputs.vSimTone;
let simA = textureLoad(simDataTexture, vec2<i32>(${off}, simRow), 0);
#ifdef SIMBODY
// Painted clothes: coverage of the top, bottom and shoes (m, > 0 covered), edges anti-aliased.
let c1 = textureLoad(simDataTexture, vec2<i32>(${off + 1}, simRow), 0).w;
let c2 = textureLoad(simDataTexture, vec2<i32>(${off + 2}, simRow), 0).w;
let c3 = textureLoad(simDataTexture, vec2<i32>(${off + 3}, simRow), 0).w;
let simT4 = textureLoad(simDataTexture, vec2<i32>(${off + 4}, simRow), 0);
let c5 = textureLoad(simDataTexture, vec2<i32>(${off + 5}, simRow), 0).w;
let c6 = textureLoad(simDataTexture, vec2<i32>(${off + 6}, simRow), 0).w;
let gTop = min(min((simA.w - simF.x) * 0.55, simF.y - c1), min(simF.z - c2, simF.w - c3));
let gBottom = min(min(simT4.w - simF.y, simF.y - c5), (-0.5 - simF.x) * 0.55);
let gShoe = c6 - simF.y;
let simTop = smoothstep(-1.0, 1.0, gTop / max(fwidth(gTop), 1e-5));
let simBottom = smoothstep(-1.0, 1.0, gBottom / max(fwidth(gBottom), 1e-5));
let simShoe = smoothstep(-1.0, 1.0, gShoe / max(fwidth(gShoe), 1e-5));
let simCloth = max(max(simTop, simBottom), simShoe);
#endif
#ifdef SIMEYE
let simR = length(simF.xy);
let simAA = max(fwidth(simR), 1e-5);
// A catchlight up and to the viewer's left of the pupil (the eye's own frame: it moves with the gaze).
let simC = length(simF.xy - vec2f(0.38, 0.42) * simF.z);
let simCatch = 1.0 - smoothstep(0.2 * simF.z - simAA, 0.2 * simF.z + simAA, simC);
#endif
#endif
`,
    CUSTOM_FRAGMENT_UPDATE_ALPHA: `${GUARD}
#ifdef SIMBODY
{
  let simRef = textureLoad(simDataTexture, vec2<i32>(${off + 5}, simRow), 0);
  // Skin: the Sim's colour (relative to the reference), blushed and darkened in cavities by the tone.
  var skin = simA.rgb * clamp(surfaceAlbedo / max(simRef.rgb, vec3f(0.01)), vec3f(0.0), vec3f(2.0));
  skin = skin * mix(vec3f(1.0), vec3f(0.98, 0.62, 0.6), clamp(simT.x, 0.0, 1.0)) * (1.0 - (0.45 + 0.2 * simT4.y) * clamp(simT.y, 0.0, 1.0));
  let scalp = clamp(-simF.z - 1.0, 0.0, 1.0) * textureLoad(simDataTexture, vec2<i32>(${off + 7}, simRow), 0).w;
  skin = mix(skin, textureLoad(simDataTexture, vec2<i32>(${off + 3}, simRow), 0).rgb * 0.7, scalp);
  let cTop = textureLoad(simDataTexture, vec2<i32>(${off + 1}, simRow), 0).rgb;
  let cBottom = textureLoad(simDataTexture, vec2<i32>(${off + 2}, simRow), 0).rgb;
  let cShoe = textureLoad(simDataTexture, vec2<i32>(${off + 6}, simRow), 0).rgb;
  let topOver = mix(mix(skin, cBottom, simBottom), cTop, simTop);
  let topUnder = mix(mix(skin, cTop, simTop), cBottom, simBottom);
  surfaceAlbedo = mix(select(topUnder, topOver, simT4.x > 0.5), cShoe, simShoe);
}
#endif
#ifdef SIMCLOTH
{
  // Shells: the flat top / bottom / shoe colour (fields.z = category); soles darker.
  let cTop = textureLoad(simDataTexture, vec2<i32>(${off + 1}, simRow), 0).rgb;
  let cBottom = textureLoad(simDataTexture, vec2<i32>(${off + 2}, simRow), 0).rgb;
  let cShoe = textureLoad(simDataTexture, vec2<i32>(${off + 6}, simRow), 0).rgb;
  let c = select(select(cShoe, cBottom, simF.z < 1.5), cTop, simF.z < 0.5);
  surfaceAlbedo = surfaceAlbedo * c * select(1.0, 0.35, simF.z > 1.5 && abs(simF.y - 1.0) < 0.5);
}
#endif
#ifdef SIMHAIR
// The hair colour in clumps (tone.z), darker deep in (tone.y).
surfaceAlbedo = textureLoad(simDataTexture, vec2<i32>(${off + 3}, simRow), 0).rgb * (0.8 + 0.5 * (simT.z - 0.5)) * (1.0 - 0.5 * clamp(simT.y, 0.0, 1.0));
#endif
#ifdef SIMBROW
surfaceAlbedo = textureLoad(simDataTexture, vec2<i32>(${off + 3}, simRow), 0).rgb * 0.45;
#endif
#ifdef SIMLASH
surfaceAlbedo = vec3f(0.03, 0.022, 0.02);
#endif
#ifdef SIMEYE
{
  // Sclera, the iris (a darker limbal ring at its edge) and the pupil.
  var eyeColour = textureLoad(simDataTexture, vec2<i32>(${off + 7}, simRow), 0).rgb;
  eyeColour = select(vec3f(0.24, 0.14, 0.07), eyeColour, dot(eyeColour, vec3f(1.0)) > 0.001);
  let iris = 1.0 - smoothstep(simF.z - simAA, simF.z + simAA, simR);
  let pupil = 1.0 - smoothstep(simF.w - simAA, simF.w + simAA, simR);
  let ring = smoothstep(simF.z * 0.62, simF.z, simR);
  let irisColour = eyeColour * (1.0 - 0.55 * ring) * (0.85 + 0.3 * smoothstep(simF.w, simF.z * 0.7, simR));
  surfaceAlbedo = mix(mix(vec3f(0.93, 0.91, 0.88), irisColour, iris), vec3f(0.02), pupil);
}
#endif
#ifdef SIMMOUTH
{
  // Inside the mouth: the tongue red (tone.x), shaded (tone.y); the mouth plate (tone.y = 1) dark red.
  let shade = clamp(simT.y, 0.0, 1.0);
  let c = mix(surfaceAlbedo, vec3f(0.6, 0.17, 0.17), clamp(simT.x, 0.0, 1.0)) * (1.0 - 0.6 * shade);
  surfaceAlbedo = mix(c, vec3f(0.09, 0.018, 0.022), smoothstep(0.7, 1.0, shade));
}
#endif
#endif
`,
    CUSTOM_FRAGMENT_BEFORE_FINALCOLORCOMPOSITION: `${GUARD}
#ifdef SIMEYE
finalEmissive += vec3f(0.85) * simCatch;
#endif
#endif
`,
    // Painted clothes are rougher than skin.
    '!var microSurface: f32=reflectivityOut\\.microSurface;var roughness: f32=reflectivityOut\\.roughness;': `var microSurface: f32=reflectivityOut.microSurface;var roughness: f32=reflectivityOut.roughness;
${GUARD}
#ifdef SIMBODY
microSurface = mix(microSurface, ${CLOTH_MICRO}, simCloth);
roughness = 1.0 - microSurface;
#endif
#ifdef SIMHAIR
microSurface = min(microSurface, ${HAIR_MICRO});
roughness = 1.0 - microSurface;
#endif
#endif
`,
  };
}

function glsl(shaderType: string, off: number, morph: MorphBinding | null): { [pointName: string]: string } {
  if (shaderType === 'vertex') {
    const W = morph?.width ?? 1;
    const H = morph?.header ?? 0;
    const pairs = off + APPEARANCE_TEXELS;
    return {
      CUSTOM_VERTEX_DEFINITIONS: `${GUARD}
attribute vec4 simField;
attribute vec4 simTone;
varying float vSimSlot;
varying vec4 vSimField;
varying vec4 vSimTone;
#ifdef SIMMORPH
attribute vec4 simMorph;
uniform highp sampler2D simMorphTexture;
vec4 simMorphTexel(int i) { return texelFetch(simMorphTexture, ivec2(i % ${W}, i / ${W}), 0); }
#endif
#endif
`,
      CUSTOM_VERTEX_UPDATE_POSITION: `${GUARD}
#ifdef SIMMORPH
int mRow = int(bakedVertexAnimationSettingsInstanced.x + 0.5);
if (simMorph.x >= 0.0 && texelFetch(bakedVertexAnimationTexture, ivec2(${pairs}, mRow), 0).x > -0.5) {
  int mBase = int(simMorph.x + 0.5);
  int mSlot = int(simMorph.y + 0.5);
  int mCount = int(simMorph.z + 0.5);
  for (int k = 0; k < ${MORPH_TEXELS}; k++) {
    vec4 pw = texelFetch(bakedVertexAnimationTexture, ivec2(${pairs} + k, mRow), 0);
    for (int j = 0; j < 2; j++) {
      float w = j == 1 ? pw.w : pw.y;
      if (abs(w) < 0.0001) continue;
      int ch = int((j == 1 ? pw.z : pw.x) + 0.5);
      float lf = simMorphTexel(mBase + ch / 4)[ch % 4];
      if (lf < -0.5) continue;
      int t = mBase + ${H} + (int(lf + 0.5) * mCount + mSlot) * 2;
      positionUpdated += simMorphTexel(t).xyz * w;
      normalUpdated += simMorphTexel(t + 1).xyz * w;
    }
  }
}
#endif
#endif
`,
      CUSTOM_VERTEX_MAIN_END: `${GUARD}\nvSimSlot = bakedVertexAnimationSettingsInstanced.x;\nvSimField = simField;\nvSimTone = simTone;\n#endif\n`,
    };
  }
  return {
    CUSTOM_FRAGMENT_DEFINITIONS: `${GUARD}
varying float vSimSlot;
varying vec4 vSimField;
varying vec4 vSimTone;
uniform highp sampler2D simDataTexture;
#endif
`,
    CUSTOM_FRAGMENT_MAIN_BEGIN: `${GUARD}
int simRow = int(vSimSlot + 0.5);
vec4 simF = vSimField;
vec4 simT = vSimTone;
vec4 simA = texelFetch(simDataTexture, ivec2(${off}, simRow), 0);
#ifdef SIMBODY
float c1 = texelFetch(simDataTexture, ivec2(${off + 1}, simRow), 0).w;
float c2 = texelFetch(simDataTexture, ivec2(${off + 2}, simRow), 0).w;
float c3 = texelFetch(simDataTexture, ivec2(${off + 3}, simRow), 0).w;
vec4 simT4 = texelFetch(simDataTexture, ivec2(${off + 4}, simRow), 0);
float c5 = texelFetch(simDataTexture, ivec2(${off + 5}, simRow), 0).w;
float c6 = texelFetch(simDataTexture, ivec2(${off + 6}, simRow), 0).w;
float gTop = min(min((simA.w - simF.x) * 0.55, simF.y - c1), min(simF.z - c2, simF.w - c3));
float gBottom = min(min(simT4.w - simF.y, simF.y - c5), (-0.5 - simF.x) * 0.55);
float gShoe = c6 - simF.y;
float simTop = smoothstep(-1.0, 1.0, gTop / max(fwidth(gTop), 1e-5));
float simBottom = smoothstep(-1.0, 1.0, gBottom / max(fwidth(gBottom), 1e-5));
float simShoe = smoothstep(-1.0, 1.0, gShoe / max(fwidth(gShoe), 1e-5));
float simCloth = max(max(simTop, simBottom), simShoe);
#endif
#ifdef SIMEYE
float simR = length(simF.xy);
float simAA = max(fwidth(simR), 1e-5);
float simC = length(simF.xy - vec2(0.38, 0.42) * simF.z);
float simCatch = 1.0 - smoothstep(0.2 * simF.z - simAA, 0.2 * simF.z + simAA, simC);
#endif
#endif
`,
    CUSTOM_FRAGMENT_UPDATE_ALPHA: `${GUARD}
#ifdef SIMBODY
{
  vec4 simRef = texelFetch(simDataTexture, ivec2(${off + 5}, simRow), 0);
  vec3 skin = simA.rgb * clamp(surfaceAlbedo / max(simRef.rgb, vec3(0.01)), vec3(0.0), vec3(2.0));
  skin = skin * mix(vec3(1.0), vec3(0.98, 0.62, 0.6), clamp(simT.x, 0.0, 1.0)) * (1.0 - (0.45 + 0.2 * simT4.y) * clamp(simT.y, 0.0, 1.0));
  float scalp = clamp(-simF.z - 1.0, 0.0, 1.0) * texelFetch(simDataTexture, ivec2(${off + 7}, simRow), 0).w;
  skin = mix(skin, texelFetch(simDataTexture, ivec2(${off + 3}, simRow), 0).rgb * 0.7, scalp);
  vec3 cTop = texelFetch(simDataTexture, ivec2(${off + 1}, simRow), 0).rgb;
  vec3 cBottom = texelFetch(simDataTexture, ivec2(${off + 2}, simRow), 0).rgb;
  vec3 cShoe = texelFetch(simDataTexture, ivec2(${off + 6}, simRow), 0).rgb;
  vec3 topOver = mix(mix(skin, cBottom, simBottom), cTop, simTop);
  vec3 topUnder = mix(mix(skin, cTop, simTop), cBottom, simBottom);
  surfaceAlbedo = mix(simT4.x > 0.5 ? topOver : topUnder, cShoe, simShoe);
}
#endif
#ifdef SIMCLOTH
{
  vec3 cTop = texelFetch(simDataTexture, ivec2(${off + 1}, simRow), 0).rgb;
  vec3 cBottom = texelFetch(simDataTexture, ivec2(${off + 2}, simRow), 0).rgb;
  vec3 cShoe = texelFetch(simDataTexture, ivec2(${off + 6}, simRow), 0).rgb;
  vec3 c = simF.z < 0.5 ? cTop : simF.z < 1.5 ? cBottom : cShoe;
  surfaceAlbedo = surfaceAlbedo * c * (simF.z > 1.5 && abs(simF.y - 1.0) < 0.5 ? 0.35 : 1.0);
}
#endif
#ifdef SIMHAIR
surfaceAlbedo = texelFetch(simDataTexture, ivec2(${off + 3}, simRow), 0).rgb * (0.8 + 0.5 * (simT.z - 0.5)) * (1.0 - 0.5 * clamp(simT.y, 0.0, 1.0));
#endif
#ifdef SIMBROW
surfaceAlbedo = texelFetch(simDataTexture, ivec2(${off + 3}, simRow), 0).rgb * 0.45;
#endif
#ifdef SIMLASH
surfaceAlbedo = vec3(0.03, 0.022, 0.02);
#endif
#ifdef SIMEYE
{
  vec3 eyeColour = texelFetch(simDataTexture, ivec2(${off + 7}, simRow), 0).rgb;
  if (dot(eyeColour, vec3(1.0)) <= 0.001) eyeColour = vec3(0.24, 0.14, 0.07);
  float iris = 1.0 - smoothstep(simF.z - simAA, simF.z + simAA, simR);
  float pupil = 1.0 - smoothstep(simF.w - simAA, simF.w + simAA, simR);
  float ring = smoothstep(simF.z * 0.62, simF.z, simR);
  vec3 irisColour = eyeColour * (1.0 - 0.55 * ring) * (0.85 + 0.3 * smoothstep(simF.w, simF.z * 0.7, simR));
  surfaceAlbedo = mix(mix(vec3(0.93, 0.91, 0.88), irisColour, iris), vec3(0.02), pupil);
}
#endif
#ifdef SIMMOUTH
{
  float shade = clamp(simT.y, 0.0, 1.0);
  vec3 c = mix(surfaceAlbedo, vec3(0.6, 0.17, 0.17), clamp(simT.x, 0.0, 1.0)) * (1.0 - 0.6 * shade);
  surfaceAlbedo = mix(c, vec3(0.09, 0.018, 0.022), smoothstep(0.7, 1.0, shade));
}
#endif
#endif
`,
    CUSTOM_FRAGMENT_BEFORE_FINALCOLORCOMPOSITION: `${GUARD}
#ifdef SIMEYE
finalEmissive += vec3(0.85) * simCatch;
#endif
#endif
`,
    '!float microSurface=reflectivityOut\\.microSurface;float roughness=reflectivityOut\\.roughness;': `float microSurface=reflectivityOut.microSurface;float roughness=reflectivityOut.roughness;
${GUARD}
#ifdef SIMBODY
microSurface = mix(microSurface, ${CLOTH_MICRO}, simCloth);
roughness = 1.0 - microSurface;
#endif
#ifdef SIMHAIR
microSurface = min(microSurface, ${HAIR_MICRO});
roughness = 1.0 - microSurface;
#endif
#endif
`,
  };
}
