/**
 * Lamps light their own room: each lamp's light stops at the room's walls (M-lighting-08).
 *
 * The lamps are clustered lights with no shadows of their own, so a lamp beside a wall lit the
 * floor and furniture of the room behind it. Here every lit fragment knows its room, and every
 * lamp its own, from a mask of the rooms on the lot in view (a texel per tile, on every storey
 * shown); in the clustered lighting loop a lamp in another room than the fragment is skipped. A
 * fragment is classed by a point a little out along its normal, so a wall's two faces belong to
 * the rooms they face (and its outside face to the garden, which lamps indoors don't light). Tiles
 * the mask doesn't know (off the lot in view, split by a diagonal wall, storeys not shown) are lit
 * by every lamp, as before.
 *
 * The test sits in Babylon's clustered lighting function (`pbrClusteredLightingFunctions`, WGSL and
 * GLSL, patched in the shader store before anything compiles), so it costs a texel fetch per
 * fragment and one per lamp the cluster gives it. The forward lights (Low, and a GPU without
 * clustered lights) aren't masked: a few lamps light the biggest rooms there anyway.
 */

import { Constants, MaterialPluginBase, RawTexture, ShaderStore, Vector4, type AbstractEngine, type BaseTexture, type Material, type MaterialDefines, type Scene, type SubMesh, type UniformBuffer } from './core';
import '@babylonjs/core/Shaders/ShadersInclude/pbrClusteredLightingFunctions.js';
import '@babylonjs/core/ShadersWGSL/ShadersInclude/pbrClusteredLightingFunctions.js';
import { WALL_HEIGHT } from '../shared/walls';

/** The mask's size in texels: a lot is at most 256 tiles each way, and up to four storeys are shown. */
const MASK_W = 256;
const STOREYS = 4;
const MASK_H = 256 * STOREYS;
/** How far out along its normal a fragment is classed (m): off a wall's face, into the room it faces. */
const OUT = 0.1;

/** The rooms of the lot in view, a value per tile: 0 unknown (lit by every lamp), 1 outdoors, 2 and up a room. */
export interface RoomMask {
  /** The lot's corner (m) and size (tiles). */
  x0: number;
  z0: number;
  w: number;
  d: number;
  /** The lowest storey given, and how many, each `d` rows of `tiles` (from the lowest up). */
  storey: number;
  storeys: number;
  tiles: Uint16Array;
}

/** What every lit material reads to keep lamps in their rooms (`RoomLightPlugin`). */
export class RoomLightState {
  /** Masking is on (clustered lamps on WebGPU or WebGL2); a define, so switching recompiles. */
  enabled = false;
  /** The mask: two bytes per tile (low, high) in its corner, `area` (x0, z0 in metres, w, d in tiles). */
  readonly mask: RawTexture;
  readonly area = new Vector4(0, 0, 0, 0);
  /** The storeys in the mask: the lowest, how many (none: nothing masked), the storey height. */
  readonly band = new Vector4(0, 0, WALL_HEIGHT, 0);
  private readonly bytes = new Uint8Array(MASK_W * MASK_H * 2);
  private shown: RoomMask | null = null;

  constructor(scene: Scene) {
    const engine = scene.getEngine();
    // (WebGL1 has no two-channel format nor `texelFetch`: it never masks.)
    const webgl1 = !engine.isWebGPU && engine.version === 1;
    this.mask = new RawTexture(this.bytes, MASK_W, MASK_H, webgl1 ? Constants.TEXTUREFORMAT_LUMINANCE_ALPHA : Constants.TEXTUREFORMAT_RG, scene, false, false, Constants.TEXTURE_NEAREST_SAMPLINGMODE, Constants.TEXTURETYPE_UNSIGNED_BYTE);
    this.mask.wrapU = this.mask.wrapV = Constants.TEXTURE_CLAMP_ADDRESSMODE;
    this.mask.name = 'roomLightMask';
  }

  /** Shows `rooms` (null: none, every lamp lights everything). Returns whether anything changed (a WebGPU snapshot must re-record then). */
  show(rooms: RoomMask | null): boolean {
    const was = this.shown;
    if (was === rooms) return false;
    if (rooms && was && was.x0 === rooms.x0 && was.z0 === rooms.z0 && was.w === rooms.w && was.d === rooms.d && was.storey === rooms.storey && was.storeys === rooms.storeys && was.tiles.every((t, i) => t === rooms.tiles[i])) {
      this.shown = rooms;
      return false;
    }
    this.shown = rooms;
    if (!rooms) {
      this.band.w = 0;
      return true;
    }
    const w = Math.min(rooms.w, MASK_W);
    const rows = Math.min(rooms.d * rooms.storeys, MASK_H);
    this.bytes.fill(0);
    for (let z = 0; z < rows; z++) {
      for (let x = 0; x < w; x++) {
        const v = rooms.tiles[z * rooms.w + x];
        const at = (z * MASK_W + x) * 2;
        this.bytes[at] = v & 255;
        this.bytes[at + 1] = v >> 8;
      }
    }
    this.mask.update(this.bytes);
    this.area.set(rooms.x0, rooms.z0, w, rooms.d);
    this.band.set(rooms.storey, Math.min(rooms.storeys, Math.floor(MASK_H / Math.max(1, rooms.d))), WALL_HEIGHT, 1);
    return true;
  }
}

/** Binds the mask and its uniforms on a lit (PBR) material; the test itself is in the patched include. */
export class RoomLightPlugin extends MaterialPluginBase {
  constructor(
    material: Material,
    private readonly state: RoomLightState,
  ) {
    super(material, 'RoomLights', 214, { ROOMLIGHTS: false }, true, false);
    // (For `hardBindForSubMesh`: the uniforms are set on every draw, frozen material or not.)
    this.registerForExtraEvents = true;
    this._enable(true);
  }

  override getClassName(): string {
    return 'RoomLightPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override prepareDefines(defines: MaterialDefines): void {
    defines.ROOMLIGHTS = this.state.enabled;
  }

  override getUniforms(): { externalUniforms: string[] } {
    return { externalUniforms: ['roomLightArea', 'roomLightBand'] };
  }

  override getSamplers(samplers: string[]): void {
    samplers.push('roomLightMask');
  }

  override isReadyForSubMesh(): boolean {
    return this.state.mask.isReady();
  }

  // (Not in `getActiveTextures`: the mask is shared, and a material disposed with its textures mustn't take it along.)
  override hasTexture(texture: BaseTexture): boolean {
    return texture === this.state.mask;
  }

  override bindForSubMesh(ubo: UniformBuffer): void {
    if (this.state.enabled) ubo.setTexture('roomLightMask', this.state.mask);
  }

  override hardBindForSubMesh(_ubo: UniformBuffer, _scene: Scene, _engine: AbstractEngine, subMesh: SubMesh): void {
    const effect = subMesh.effect;
    if (!effect || !this.state.enabled) return;
    const { area, band } = this.state;
    effect.setFloat4('roomLightArea', area.x, area.y, area.z, area.w);
    effect.setFloat4('roomLightBand', band.x, band.y, band.z, band.w);
  }
}

/** The room a point is in, as the mask has it (0: unknown). */
const ROOM_OF_WGSL = `
#ifdef ROOMLIGHTS
uniform roomLightArea: vec4f;
uniform roomLightBand: vec4f;
var roomLightMask: texture_2d<f32>;
fn roomLightRoom{X}(p: vec3f)->f32 {if (uniforms.roomLightBand.w<0.5) {return 0.0;}
let k=floor(p.y/uniforms.roomLightBand.z)-uniforms.roomLightBand.x;let t=floor(p.xz-uniforms.roomLightArea.xy);
if (k<0.0 || k>=uniforms.roomLightBand.y || t.x<0.0 || t.y<0.0 || t.x>=uniforms.roomLightArea.z || t.y>=uniforms.roomLightArea.w) {return 0.0;}
let v=textureLoad(roomLightMask,vec2<i32>(i32(t.x),i32(k*uniforms.roomLightArea.w+t.y)),0);return floor(v.r*255.0+0.5)+floor(v.g*255.0+0.5)*256.0;}
#endif
`;

const ROOM_OF_GLSL = `
#ifdef ROOMLIGHTS
uniform vec4 roomLightArea;
uniform vec4 roomLightBand;
uniform highp sampler2D roomLightMask;
float roomLightRoom(vec3 p) {if (roomLightBand.w<0.5) {return 0.0;}
float k=floor(p.y/roomLightBand.z)-roomLightBand.x;vec2 t=floor(p.xz-roomLightArea.xy);
if (k<0.0 || k>=roomLightBand.y || t.x<0.0 || t.y<0.0 || t.x>=roomLightArea.z || t.y>=roomLightArea.w) {return 0.0;}
vec4 v=texelFetch(roomLightMask,ivec2(int(t.x),int(k*roomLightArea.w+t.y)),0);return floor(v.r*255.0+0.5)+floor(v.g*255.0+0.5)*256.0;}
#endif
`;

/** Inserts `add` into an include after (or before) `at`, once; false if `at` isn't there (a Babylon upgrade moved it). */
function insert(store: Record<string, string>, include: string, at: string, add: string, before = false): boolean {
  const text = store[include];
  if (!text || !text.includes(at) || text.includes(add)) return !!text?.includes(add);
  store[include] = text.replace(at, before ? add + at : at + add);
  return true;
}

let patched = false;

/**
 * Puts the room test into Babylon's clustered lighting functions (Babylon 9.30). If an upgrade
 * moved the code it hooks onto, the lamps light through walls again, and a warning says so.
 */
export function patchRoomLights(): void {
  if (patched) return;
  patched = true;
  const wgsl = ShaderStore.IncludesShadersStoreWGSL;
  const glsl = ShaderStore.IncludesShadersStore;
  const fn = 'pbrClusteredLightingFunctions';
  const here = (s: string) => `\n#ifdef ROOMLIGHTS\n${s}\n#endif\n`;
  const ok = [
    insert(wgsl, fn, '#if defined(CLUSTLIGHT{X}) && defined(CLUSTLIGHT_BATCH) && CLUSTLIGHT_BATCH>0\n', ROOM_OF_WGSL),
    insert(wgsl, fn, 'var result: lightingInfo;', here(`let roomHere=roomLightRoom{X}(posW+N*${OUT});`), true),
    insert(wgsl, fn, 'let light=getClusteredLight(lightDataTexture,batchOffset+maskOffset+trailing);', here('let roomThere=roomLightRoom{X}(light.vLightData.xyz);if (roomHere>0.0 && roomThere>0.0 && roomHere!=roomThere) {continue;}')),
    insert(glsl, fn, '#include<clusteredLightingFunctions>\n', ROOM_OF_GLSL),
    insert(glsl, fn, 'lightingInfo result;', here(`float roomHere=roomLightRoom(posW+N*${OUT});`), true),
    insert(glsl, fn, 'ClusteredLight light=getClusteredLight(lightDataTexture,batchOffset+maskOffset+position);', here('float roomThere=roomLightRoom(light.vLightData.xyz);if (roomHere>0.0 && roomThere>0.0 && roomHere!=roomThere) {continue;}')),
  ];
  if (ok.includes(false)) console.warn('[roomLights] Babylon\'s clustered lighting changed: lamps light through walls again');
}
