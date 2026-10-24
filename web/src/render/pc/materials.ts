/**
 * Physically based materials and image-based lighting on PlayCanvas (the port of
 * `babylon/materials.ts`).
 *
 * - Lot surfaces (walls, floors, roads, terrain) and the "finishes" of primitive parts (fabric, wood,
 *   ceramic, metal, ...) are `StandardMaterial`s in the metalness workflow, built from manifest
 *   `material` entries: albedo, normal and packed AO/roughness/metal maps tiled in metres.
 * - Manifest colours are sRGB, as PlayCanvas takes them (it shades in linear space). Vertex and
 *   per-instance colours (primitive parts, Sim tints) are converted on the GPU (`shaders/surface.ts`).
 * - The environment (`.pcenv`, converted from the Babylon `.env` by `cargo envconv`, `env.ts`) lights
 *   everything with sky ambient (spherical harmonics) and reflections (the environment atlas); its
 *   intensity follows the time of day (`setEnvironmentIntensity`). The sky's hemispheric fill
 *   (`setSkyFill`) is the game's own term, Babylon's hemispheric light.
 *
 * - House surfaces can carry the cutaway (`cutaway`): walls drop to stubs in the vertex shader when
 *   the cutaway state says so (driven by a per-vertex attribute, see `shared/geometry.ts`), at no
 *   CPU cost. Every mesh drawn with such a material needs that attribute (`WALL_CUT_SEMANTIC`).
 *
 * Every lit material made or adopted here is shaded in the game's soft look (`shaders/softLook.ts`):
 * the light PBR computed, redrawn as a wide soft ramp in the light's own colour. The Settings'
 * Shading option (`classic`) draws the PBR shading instead; it applies live (`setShading`), and
 * the diag views (`?diag=albedo|normals`, `shaders/diagView.ts`) apply to all of them too.
 *
 * The shader features are chunk overrides shared by every material (`shaders/chunks.ts`), switched
 * per material by defines. The state they read (the cutaway, the dug-out ground, the basements, the
 * sky's fill) is the library's, handed to the shaders as global uniforms before every frame: the
 * materials never need touching for it. Materials aren't frozen as Babylon's were; one is updated
 * (`material.update()`) only when something of its own changes.
 */

import * as pc from './engine';
import { skyLighting, type SkyLighting } from './env';
import { INSTANCE_COLOR_SEMANTIC, installGameChunks, patchEngineChunks, WALL_CUT_SEMANTIC } from './shaders/chunks';
import { DIAG_VIEW_DEFINE } from './shaders/diagView';
import { softLookWanted } from './shaders/softLook';
import { withBuild } from '../../assets/build';
import { optimizedUrl, type AssetRegistry } from '../../assets/registry';
import type { MaterialEntry } from '../../assets/types';
import type { RGB } from '../shared/environment';
import { WALL_HEIGHT } from '../shared/walls';
import type { Shading } from '../types';

export { INSTANCE_COLOR_SEMANTIC, WALL_CUT_SEMANTIC } from './shaders/chunks';

/** Default finish for primitive parts without a `material`. */
export const DEFAULT_FINISH = 'material.finish.matte';

const FALLBACK_FINISH: MaterialEntry = { type: 'material', color: '#FFFFFF', roughness: 0.75 };

/**
 * A pattern over a surface's texture, in the fragment shader (no texture or draw of its own):
 * - `stripes`: wallpaper's faint stripes, 12.5 cm apart along the wall, fading out where they'd be
 *   finer than a pixel or two.
 * - `whitewash`: wood limed pale, its grain still showing (the tint multiplies a mid-tone wood).
 */
export type SurfacePattern = 'stripes' | 'whitewash';

/**
 * The light mask of the sun (the renderer's directional light): every mesh's dynamic bit, and one
 * of its own that meshes left out of the lamps keep (`excludeFromLights`).
 */
export const SUN_LIGHT_BIT = 8;
export const SUN_LIGHT_MASK = pc.MASK_AFFECT_DYNAMIC | SUN_LIGHT_BIT;

/**
 * The front face of the game's meshes, for materials that cull: game-space meshes keep Babylon's
 * winding (counter-clockwise on screen, as drawn by Babylon), and the world root's mirror makes
 * PlayCanvas turn every material's front face round (`worldScaleSign`). Clockwise, turned round,
 * is the winding they have. (Materials adopted from glTF keep their own: the models decide, by how
 * they bake.)
 */
export const GAME_FRONT_FACE = pc.FRONTFACE_CW;

/** Cutaway state shared by every wall material of a scene (set by the renderer). */
export class WallCutState {
  /** 1 where the camera looks along +x, -x, +z, -z (matches `LOOK` bits in geometry.ts). */
  readonly look = new pc.Vec4(0, 0, 0, 0);
  /** x: all walls down (0/1); y: stub height in metres. */
  readonly params = new pc.Vec4(0, 0.35, 0, 0);
  /**
   * A stretch of wall hidden while Build mode's preview stands in for it: everything within
   * `holeSpan.x` of the segment `hole` (x0, z0, x1, z1), between heights `holeSpan.y` and `.z`.
   * A zero width hides nothing.
   */
  readonly hole = new pc.Vec4(0, 0, 0, 0);
  readonly holeSpan = new pc.Vec4(0, 0, 0, 0);
}

/**
 * The bits of a tile in the windowless rooms' mask (`DigState.darken`): dark whole, or the halves
 * of a tile split by a diagonal wall that are, and which way the wall runs. In tile-local
 * u = x - x0, v = z - z0, a 'dp' wall's half 0 is v < u (half 1 v > u), a 'dn' wall's u + v < 1
 * (half 1 u + v > 1), as slanted.ts `halfTriangle` has them.
 */
export const WINDOWLESS_BITS = { tile: 1, half0: 2, half1: 4, dn: 8 } as const;

/** Tiles kept out of daylight (`DigState.darken`): `w` by `d` of them from (x0, z0), heights y0 to y1. */
export interface DarkTiles {
  x0: number;
  z0: number;
  w: number;
  d: number;
  y0: number;
  y1: number;
  /** A byte per tile (`WINDOWLESS_BITS`), row by row; not changed once given to `darken`. */
  tiles: Uint8Array;
}

/** The mask's side in tiles: a storey of a lot is at most 256 tiles each way (sim-core `lot.rs`). */
const MASK_SIZE = 256;

/**
 * The ground dug away over the lot while its basement is in view: `rect` (x0, z0, x1, z1, in
 * metres) of the ground's materials isn't drawn above `below` metres while `on`.
 */
export class DigState {
  readonly rect = new pc.Vec4(0, 0, 0, 0);
  on = false;
  below = -0.5;
  /**
   * The landscape's draws, the only ones dug away (`MaterialLibrary.digInto`): their finishes are
   * shared (a rock's plain finish is every model's), and the stairs down rise out of the basement
   * through `rect`.
   */
  readonly meshes = new WeakSet<pc.MeshInstance>();
  /**
   * The basement itself is in view: everything lit inside `rect` is underground then, out of the
   * sun (`shaders/underground.ts`), with `lit` the share of the sun, the sky fill and the
   * environment (sky ambient and reflections) that still reaches it.
   */
  under = false;
  readonly lit = new pc.Vec4(0, 0.1, 0.1, 0);
  /**
   * Rooms without windows on the storey in view, as dark as the basement by day: `mask` has a
   * byte per tile of `area` (x0, z0 in metres, then w, d in tiles; `WINDOWLESS_BITS`) in its
   * corner, `band` the heights it reaches (y0, y1; nowhere while both are 0). One texture of a
   * fixed size, never swapped.
   */
  readonly mask: pc.Texture;
  readonly area = new pc.Vec4(0, 0, 0, 0);
  readonly band = new pc.Vec4(0, 0, 0, 0);
  /** What `mask` holds (uploaded only when that changes), and its bytes as uploaded. */
  private shown: DarkTiles | null = null;
  private readonly bytes = new Uint8Array(MASK_SIZE * MASK_SIZE);

  constructor(device: pc.GraphicsDevice) {
    // One byte per tile, read whole (no filtering between tiles, no mipmaps).
    this.mask = new pc.Texture(device, {
      name: 'windowlessMask',
      width: MASK_SIZE,
      height: MASK_SIZE,
      format: pc.PIXELFORMAT_R8,
      mipmaps: false,
      minFilter: pc.FILTER_NEAREST,
      magFilter: pc.FILTER_NEAREST,
      addressU: pc.ADDRESS_CLAMP_TO_EDGE,
      addressV: pc.ADDRESS_CLAMP_TO_EDGE,
      levels: [this.bytes],
    });
  }

  /**
   * Keeps daylight out of the tiles `dark` marks, or out of nowhere (null). Returns whether that
   * changed anything.
   */
  darken(dark: DarkTiles | null): boolean {
    const [y0, y1] = dark ? [dark.y0, dark.y1] : [0, 0];
    let changed = this.band.x !== y0 || this.band.y !== y1;
    this.band.set(y0, y1, 0, 0);
    if (!dark) return changed;
    const was = this.shown;
    if (was !== dark && (!was || was.w !== dark.w || was.d !== dark.d || dark.tiles.some((t, i) => t !== was.tiles[i]))) {
      this.bytes.fill(0);
      for (let z = 0; z < Math.min(dark.d, MASK_SIZE); z++) this.bytes.set(dark.tiles.subarray(z * dark.w, z * dark.w + Math.min(dark.w, MASK_SIZE)), z * MASK_SIZE);
      const level = this.mask.lock();
      (level as Uint8Array).set(this.bytes);
      this.mask.unlock();
      this.shown = dark;
      changed = true;
    }
    if (this.area.x !== dark.x0 || this.area.y !== dark.z0 || this.area.z !== dark.w || this.area.w !== dark.d) {
      this.area.set(dark.x0, dark.z0, dark.w, dark.d);
      changed = true;
    }
    return changed;
  }
}

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

/** The room mask's size in texels: a lot is at most 256 tiles each way, and up to four storeys are shown. */
const ROOM_MASK_W = 256;
const ROOM_MASK_H = 256 * 4;

/**
 * Which room each tile of the lot in view is in, so lamps light only their own (the port of
 * `babylon/roomLights.ts`; the test is in the clustered lights' loop, `shaders/roomLights.ts`):
 * `mask` has two bytes per tile (low, high) in its corner, `area` the lot (x0, z0 in metres, then w,
 * d in tiles), `band` its storeys (the lowest, how many, the storey height, and 1 while masking:
 * nothing is masked at 0). Uniforms only: masking costs no shader of its own.
 */
export class RoomLightState {
  /** Masking is on (every lamp lights, clustered: `lamps.ts`; the low preset's four don't keep to their rooms). */
  enabled = false;
  readonly mask: pc.Texture;
  readonly area = new pc.Vec4(0, 0, 0, 0);
  readonly band = new pc.Vec4(0, 0, WALL_HEIGHT, 0);
  private readonly bytes = new Uint8Array(ROOM_MASK_W * ROOM_MASK_H * 2);
  private shown: RoomMask | null = null;

  constructor(device: pc.GraphicsDevice) {
    this.mask = new pc.Texture(device, {
      name: 'roomLightMask',
      width: ROOM_MASK_W,
      height: ROOM_MASK_H,
      format: pc.PIXELFORMAT_RG8,
      mipmaps: false,
      minFilter: pc.FILTER_NEAREST,
      magFilter: pc.FILTER_NEAREST,
      addressU: pc.ADDRESS_CLAMP_TO_EDGE,
      addressV: pc.ADDRESS_CLAMP_TO_EDGE,
      levels: [this.bytes],
    });
  }

  /** Shows `rooms` (null: none, every lamp lights everything). Returns whether anything changed. */
  show(rooms: RoomMask | null): boolean {
    if (!this.enabled) rooms = null;
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
    const w = Math.min(rooms.w, ROOM_MASK_W);
    const rows = Math.min(rooms.d * rooms.storeys, ROOM_MASK_H);
    const bytes = this.bytes;
    bytes.fill(0);
    for (let z = 0; z < rows; z++) {
      for (let x = 0; x < w; x++) {
        const v = rooms.tiles[z * rooms.w + x];
        const at = (z * ROOM_MASK_W + x) * 2;
        bytes[at] = v & 255;
        bytes[at + 1] = v >> 8;
      }
    }
    const level = this.mask.lock();
    (level as Uint8Array).set(bytes);
    this.mask.unlock();
    this.area.set(rooms.x0, rooms.z0, w, rooms.d);
    this.band.set(rooms.storey, Math.min(rooms.storeys, Math.floor(ROOM_MASK_H / Math.max(1, rooms.d))), WALL_HEIGHT, 1);
    return true;
  }
}

/**
 * Smallest change (share of the value, and absolute) passed on to a material's emissive glow: lamps
 * and windows glow by the time of day, in steps mostly far smaller than that.
 */
const LIGHT_CHANGE = 0.02;
const LIGHT_CHANGE_MIN = 0.004;

/** The sky's hemispheric fill comes from here (game space, as Babylon's `HemisphericLight`). */
const SKY_FILL_DIRECTION = { x: 0.2, y: 1, z: -0.3 };

export interface SurfaceOptions {
  /** Multiply albedo by (sRGB) vertex / instance colours. */
  vertexColors?: boolean;
  /** Render both faces (generated lot geometry does not guarantee winding). */
  doubleSided?: boolean;
  /** Derive UVs from the XZ position (metres) instead of the mesh's own UVs. */
  planarUV?: boolean;
  /** Follow the wall cutaway state (meshes need the `wallCut` attribute). */
  cutaway?: boolean;
  /** A separate copy of the surface under this name (e.g. one per room, for its own emissive glow). */
  variant?: string;
  /** A pattern drawn over its texture. */
  pattern?: SurfacePattern;
}

/** Standard material properties PlayCanvas's typings leave out. */
type Standard = pc.StandardMaterial & { ambientSH: Float32Array | null; vertexColorGamma: boolean };

/** A texture the library loads: a stand-in of the same kind until the file is there. */
interface TextureSlot {
  texture: pc.Texture;
  loaded: boolean;
  /** The materials (and their map) waiting for it. */
  users: { material: pc.StandardMaterial; map: 'diffuseMap' | 'normalMap' | 'aoMap' | 'glossMap' | 'metalnessMap' }[];
}

export class MaterialLibrary {
  private static readonly byApp = new WeakMap<pc.AppBase, MaterialLibrary>();

  /** The library for `app`; created on first use (without a registry, finishes fall back to matte). */
  static for(app: pc.AppBase): MaterialLibrary {
    let lib = MaterialLibrary.byApp.get(app);
    if (!lib) MaterialLibrary.byApp.set(app, (lib = new MaterialLibrary(app)));
    return lib;
  }

  private assets: AssetRegistry | null = null;
  private readonly cache = new Map<string, pc.StandardMaterial>();
  private readonly textures = new Map<string, TextureSlot>();
  private readonly pending = new Set<Promise<unknown>>();
  /** Every lit material (made or adopted): the look, the diag view, basements and the sky's ambient. */
  private readonly lit = new Set<pc.StandardMaterial>();
  private readonly dug = new WeakSet<pc.Material>();
  private readonly instanced = new WeakMap<pc.StandardMaterial, pc.StandardMaterial>();
  /** Emissive glows as last set (linear), to pass on only changes one could see. */
  private readonly glows = new WeakMap<pc.StandardMaterial, pc.Color>();
  private shadowMaterials: [pc.StandardMaterial | null, pc.StandardMaterial | null] = [null, null];
  private sky: SkyLighting | null = null;
  private envTimer: ReturnType<typeof setTimeout> | null = null;
  private soft = softLookWanted();
  private readonly device: pc.GraphicsDevice;
  /** Anisotropic filtering of the surfaces' textures (the graphics preset's `anisotropy`; set before the first surface). */
  anisotropy = 8;
  /** Wall cutaway state read by every `cutaway` material. */
  readonly wallCut = new WallCutState();
  /** Told of every material made from a manifest entry, before its first update (`onCreate`). */
  private readonly creators: ((key: string, material: pc.StandardMaterial) => void)[] = [];
  /** The ground dug away over a basement in view (see `digInto`), and the basements and windowless rooms kept dark. */
  readonly dig: DigState;
  /** Which room each tile of the lot in view is in, so lamps light only their own. */
  readonly rooms: RoomLightState;
  /** The sky's hemispheric fill (linear, scaled by its intensity; `setSkyFill`). */
  private readonly skySky = new Float32Array(3);
  private readonly skyGround = new Float32Array(3);
  /** Where it comes from (PlayCanvas space, unit length). */
  private readonly skyDir = new Float32Array(3);
  /** The shader features' state as global uniforms (`pushUniforms`). */
  private readonly uniforms: { id: pc.ScopeId; value: Float32Array; fill: (out: Float32Array) => void }[];
  private readonly mask: pc.ScopeId;
  private readonly roomMask: pc.ScopeId;

  private constructor(readonly app: pc.AppBase) {
    const device = (this.device = app.graphicsDevice);
    patchEngineChunks(device);
    this.dig = new DigState(device);
    this.rooms = new RoomLightState(device);
    const scope = device.scope;
    this.setSkyDirection(SKY_FILL_DIRECTION);
    const dig = this.dig;
    const cut = this.wallCut;
    const vec4 = (v: pc.Vec4) => (o: Float32Array) => {
      o[0] = v.x;
      o[1] = v.y;
      o[2] = v.z;
      o[3] = v.w;
    };
    const table: [string, number, (out: Float32Array) => void][] = [
      ['game_hemiSky', 3, (o) => o.set(this.skySky)],
      ['game_hemiGround', 3, (o) => o.set(this.skyGround)],
      ['game_hemiDir', 3, (o) => o.set(this.skyDir)],
      ['game_wallCutLook', 4, vec4(cut.look)],
      ['game_wallCutParams', 4, vec4(cut.params)],
      ['game_wallHole', 4, vec4(cut.hole)],
      ['game_wallHoleSpan', 4, vec4(cut.holeSpan)],
      ['game_digRect', 4, vec4(dig.rect)],
      [
        'game_digSpan',
        4,
        (o) => {
          o[0] = dig.on ? 1 : 0;
          o[1] = dig.below;
        },
      ],
      ['game_undergroundRect', 4, vec4(dig.rect)],
      [
        'game_undergroundLit',
        4,
        (o) => {
          o[0] = dig.lit.x;
          o[1] = dig.lit.y;
          o[2] = dig.lit.z;
          o[3] = dig.under ? 1 : 0;
        },
      ],
      ['game_windowlessBand', 4, vec4(dig.band)],
      ['game_windowlessArea', 4, vec4(dig.area)],
      ['game_roomLightArea', 4, vec4(this.rooms.area)],
      ['game_roomLightBand', 4, vec4(this.rooms.band)],
    ];
    this.uniforms = table.map(([name, size, fill]) => ({ id: scope.resolve(name), value: new Float32Array(size), fill }));
    this.mask = scope.resolve('game_windowlessMask');
    this.roomMask = scope.resolve('game_roomLightMask');
    this.pushUniforms();
    // The state as it is now, before every frame (a few uniforms: cheaper than tracking changes, no allocations).
    app.on('prerender', this.pushUniforms, this);
  }

  private pushUniforms(): void {
    for (const u of this.uniforms) {
      u.fill(u.value);
      u.id.setValue(u.value);
    }
    this.mask.setValue(this.dig.mask);
    this.roomMask.setValue(this.rooms.mask);
  }

  /**
   * Calls `fn` with each material made from now on from a manifest entry (its key, and the material
   * before its first update): the shader features of a finish that aren't the library's own (the
   * plants' wind, `nature.ts`). Copies (`instanceColored`) keep what `fn` set.
   */
  onCreate(fn: (key: string, material: pc.StandardMaterial) => void): void {
    this.creators.push(fn);
  }

  setAssets(assets: AssetRegistry): this {
    this.assets = assets;
    return this;
  }

  /** A textured surface from a manifest `material` entry (walls, floors, roads, terrain). */
  surface(key: string, options: SurfaceOptions = {}): pc.StandardMaterial {
    const id = `${key}|${options.vertexColors ? 'vc' : ''}|${options.doubleSided ? '2s' : ''}|${options.planarUV ? 'xz' : ''}|${options.cutaway ? 'cut' : ''}|${options.variant ?? ''}|${options.pattern ?? ''}`;
    let mat = this.cache.get(id);
    if (!mat) {
      const entry = this.assets?.get(key, 'material') ?? { type: 'material', color: '#DDDDDD' };
      mat = this.create(key, entry, { ...options, planarUV: options.planarUV || entry.projection === 'xz' });
      this.cache.set(id, mat);
    }
    return mat;
  }

  /** A vertex-coloured finish for primitive parts (`material.finish.*`). */
  finish(key: string = DEFAULT_FINISH): pc.StandardMaterial {
    const id = `finish|${key}`;
    let mat = this.cache.get(id);
    if (!mat) {
      const entry = (this.assets && this.assets.get(key, 'material')) || FALLBACK_FINISH;
      mat = this.create(key, entry, { vertexColors: true });
      this.cache.set(id, mat);
    }
    return mat;
  }

  /** Whether the manifests have finish `key` (`material.finish.*`). */
  hasFinish(key: string): boolean {
    return !!this.assets?.has(key, 'material');
  }

  /** Whether a finish is untextured and opaque (it can share the plain material), with its roughness and metalness. */
  finishInfo(key: string): { plain: boolean; roughness: number; metallic: number } {
    const entry = (this.assets && this.assets.has(key, 'material') && this.assets.get(key, 'material')) || FALLBACK_FINISH;
    return {
      plain: !entry.texture && (entry.alpha ?? 1) >= 1,
      roughness: entry.roughness ?? 0.8,
      metallic: entry.metallic ?? 0,
    };
  }

  /** The shared material for plain finishes (vertex colour alpha = encoded roughness/metalness, `encodeFinish`). */
  plainFinish(): pc.StandardMaterial {
    const id = 'finish|plain';
    let mat = this.cache.get(id);
    if (!mat) {
      mat = this.create('material.finish.plain', { type: 'material', color: '#FFFFFF', roughness: 1, metallic: 0 }, { vertexColors: true });
      mat.setDefine('GAME_PLAIN_FINISH', '');
      mat.update();
      this.cache.set(id, mat);
    }
    return mat;
  }

  /**
   * `material` for instanced draws with a per-instance colour (`INSTANCE_COLOR_SEMANTIC`, see
   * `instancing.ts`): a copy, kept per material, that multiplies its albedo by the instance's colour
   * (converted from sRGB where the material converts vertex colours).
   */
  instanceColored(material: pc.StandardMaterial): pc.StandardMaterial {
    let copy = this.instanced.get(material);
    if (!copy) {
      copy = material.clone();
      copy.name = `${material.name}|instanceColor`;
      copy.setAttribute('game_instanceColor', INSTANCE_COLOR_SEMANTIC);
      copy.setDefine('GAME_INSTANCE_COLOR', '');
      copy.update();
      this.instanced.set(material, copy);
      if (this.lit.has(material)) this.lit.add(copy);
      this.reuseTextures(material, copy);
    }
    return copy;
  }

  /** Adopts a material created elsewhere (the glTF loader's) into the look, the sky's ambient, basements and lamps. */
  adopt(material: pc.Material | null): void {
    if (!(material instanceof pc.StandardMaterial) || this.lit.has(material)) return;
    installGameChunks(material, this.device);
    this.light(material);
    material.update();
  }

  /**
   * Unlit black with per-vertex alpha: soft contact shadows / ambient occlusion decals. Follows the
   * wall cutaway like the walls it darkens (`cutaway`, the default: its meshes need the cutaway
   * attribute; `false` for those without, such as the street's).
   */
  contactShadow(cutaway = true): pc.StandardMaterial {
    const slot = cutaway ? 1 : 0;
    const have = this.shadowMaterials[slot];
    if (have) return have;
    const mat = new pc.StandardMaterial();
    mat.name = cutaway ? 'contactShadow' : 'contactShadowPlain';
    mat.frontFace = GAME_FRONT_FACE;
    mat.useLighting = false;
    mat.useSkybox = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(0, 0, 0);
    mat.opacityVertexColor = true;
    mat.opacityVertexColorChannel = 'a';
    mat.blendType = pc.BLEND_NORMAL;
    mat.depthWrite = false;
    mat.slopeDepthBias = -2;
    installGameChunks(mat, this.device);
    // (Unlit: none of the look.)
    mat.setDefine('GAME_LOOK', false);
    if (cutaway) this.cutaway(mat);
    mat.update();
    this.shadowMaterials[slot] = mat;
    return mat;
  }

  /** A material's emissive colour (linear), updated only for a change one could see. */
  setEmissive(material: pc.StandardMaterial, color: pc.Color): void {
    let was = this.glows.get(material);
    if (!was) this.glows.set(material, (was = new pc.Color(-1, -1, -1)));
    if (!visibleChange(was.r, color.r) && !visibleChange(was.g, color.g) && !visibleChange(was.b, color.b)) return;
    was.copy(color);
    // (PlayCanvas takes its colours in sRGB.)
    const g = (v: number) => Math.pow(Math.max(0, v), 1 / 2.2);
    material.emissive.set(g(color.r), g(color.g), g(color.b));
    material.update();
  }

  /** Uploads a material's changed properties. */
  touch(material: pc.Material): void {
    material.update();
  }

  /** Makes `meshInstance` (the landscape's ground, lawn, woods and rocks) follow `dig`. */
  digInto(meshInstance: pc.MeshInstance): void {
    this.dig.meshes.add(meshInstance);
    meshInstance.setParameter('game_digMesh', 1);
    const material = meshInstance.material;
    if (!(material instanceof pc.StandardMaterial) || this.dug.has(material)) return;
    this.dug.add(material);
    // (One the library made or adopted already has its chunks.)
    this.adopt(material);
    // Not dug unless the draw says so (the material is shared with everything else of it).
    material.setParameter('game_digMesh', 0);
    material.setDefine('GAME_DIG', '');
    material.update();
  }

  /**
   * Leaves `meshes` out of the lamps (`lights`, which become dynamic-only lights): the meshes keep
   * the sun (`SUN_LIGHT_MASK`, the renderer's sun light must have it) and drop the dynamic bit the
   * lamps light (clustered or not).
   */
  excludeFromLights(lights: readonly pc.LightComponent[], meshes: Iterable<pc.MeshInstance>): void {
    for (const light of lights) light.mask = pc.MASK_AFFECT_DYNAMIC;
    for (const mi of meshes) mi.mask = (mi.mask & ~pc.MASK_AFFECT_DYNAMIC) | SUN_LIGHT_BIT;
  }

  /** Draws every lit material in `shading` (`classic`: PBR; `soft`: the soft look). Their shaders compile again. */
  setShading(shading: Shading): void {
    const soft = shading === 'soft';
    if (soft === this.soft) return;
    this.soft = soft;
    for (const m of this.lit) {
      m.setDefine('GAME_SOFT', soft ? '' : false);
      m.update();
    }
  }

  /**
   * The sky's hemispheric fill (Babylon's `HemisphericLight`): `sky` from above, `ground` from below,
   * linear (the look's colours as Babylon used them), scaled by `intensity`.
   */
  setSkyFill(sky: RGB, ground: RGB, intensity: number): void {
    const s = this.skySky;
    const g = this.skyGround;
    s[0] = sky.r * intensity;
    s[1] = sky.g * intensity;
    s[2] = sky.b * intensity;
    g[0] = ground.r * intensity;
    g[1] = ground.g * intensity;
    g[2] = ground.b * intensity;
  }

  /** Where the sky's fill comes from (game space; the game's is `SKY_FILL_DIRECTION`, the dev pages' straight up). */
  setSkyDirection(d: { x: number; y: number; z: number }): void {
    const len = Math.hypot(d.x, d.y, d.z);
    // (PlayCanvas space: the world is mirrored in z.)
    this.skyDir.set([d.x / len, d.y / len, -d.z / len]);
  }

  /**
   * Loads the sky's lighting (`url`: the manifest's `environment.sky`, whose `.pcenv` copy is read,
   * `env.ts`); resolves (without throwing) even when it fails, and at the latest after 15 s (the sky
   * still lights the scene once it's there).
   */
  loadEnvironment(url: string): Promise<boolean> {
    return new Promise((resolve) => {
      let done = false;
      const finish = (ok: boolean) => {
        if (done) return;
        done = true;
        if (this.envTimer) clearTimeout(this.envTimer);
        this.envTimer = null;
        resolve(ok);
      };
      this.envTimer = setTimeout(() => finish(false), 15000);
      void skyLighting(this.device, url).then((sky) => {
        if (sky && !this.app.graphicsDevice) return finish(false);
        if (sky) {
          this.sky = sky;
          this.app.scene.envAtlas = sky.atlas;
          for (const m of this.lit) {
            (m as Standard).ambientSH = sky.sh;
            m.update();
          }
        }
        finish(!!sky);
      });
    });
  }

  /** Scales image-based lighting (sky ambient + reflections), e.g. for the time of day (a uniform). */
  setEnvironmentIntensity(value: number): void {
    this.app.scene.skyboxIntensity = value;
  }

  /** Resolves once every texture asked for so far has loaded (or failed): for pictures of the scene. */
  async texturesReady(): Promise<void> {
    while (this.pending.size) await Promise.all([...this.pending]);
  }

  private create(key: string, entry: MaterialEntry, options: SurfaceOptions): pc.StandardMaterial {
    if (new URLSearchParams(location.search).has('profile')) console.log(`[material] create ${key} ${options.variant ?? ''}`);
    const mat = new pc.StandardMaterial();
    mat.name = options.variant ? `${key}|${options.variant}` : key;
    mat.frontFace = GAME_FRONT_FACE;
    mat.useMetalness = true;
    mat.diffuse.fromString(entry.color);
    // Babylon's tiling: uv · scale from the origin (PlayCanvas's tiling is from the top, `1 - scale` makes it the same).
    const scale = entry.uvScale ?? 1;
    const tiling = new pc.Vec2(scale, scale);
    const offset = new pc.Vec2(0, 1 - scale);
    const base = entry.texture;
    if (base) {
      this.useTexture(mat, 'diffuseMap', base, true);
      mat.diffuseMapTiling = tiling;
      mat.diffuseMapOffset = offset;
      if (entry.normal) {
        this.useTexture(mat, 'normalMap', new URL(entry.normal, base).href, false);
        mat.normalMapTiling = tiling;
        mat.normalMapOffset = offset;
        mat.bumpiness = entry.normalStrength ?? 1;
      }
      if (entry.arm) {
        const arm = new URL(entry.arm, base).href;
        this.useTexture(mat, 'aoMap', arm, false);
        mat.aoMapChannel = 'r';
        this.useTexture(mat, 'glossMap', arm, false);
        mat.glossMapChannel = 'g';
        this.useTexture(mat, 'metalnessMap', arm, false);
        mat.metalnessMapChannel = 'b';
        mat.aoMapTiling = mat.glossMapTiling = mat.metalnessMapTiling = tiling;
        mat.aoMapOffset = mat.glossMapOffset = mat.metalnessMapOffset = offset;
      }
    }
    // Roughness, as PlayCanvas's inverted gloss; with a packed map the scalars multiply its channels.
    mat.glossInvert = true;
    mat.gloss = entry.roughness ?? (entry.arm ? 1 : 0.8);
    mat.metalness = entry.metallic ?? (entry.arm ? 1 : 0);
    if (entry.alpha !== undefined && entry.alpha < 1) {
      mat.opacity = entry.alpha;
      mat.blendType = pc.BLEND_NORMAL;
      // Keep glass reflections strong while the body is see-through.
      mat.opacityFadesSpecular = false;
    }
    if (options.doubleSided) {
      mat.cull = pc.CULLFACE_NONE;
      mat.twoSidedLighting = true;
    }
    if (options.vertexColors) {
      mat.diffuseVertexColor = true;
      (mat as Standard).vertexColorGamma = true;
    }
    installGameChunks(mat, this.device);
    if (options.vertexColors) mat.setDefine('GAME_SRGB_COLORS', '');
    if (options.planarUV) mat.setDefine('GAME_PLANAR_UV', '');
    if (options.cutaway) this.cutaway(mat);
    if (options.pattern === 'stripes') mat.setDefine('GAME_PATTERN_STRIPES', '');
    if (options.pattern === 'whitewash') mat.setDefine('GAME_PATTERN_WHITEWASH', '');
    for (const fn of this.creators) fn(key, mat);
    this.light(mat);
    mat.update();
    return mat;
  }

  /** The cutaway on `mat` (its meshes carry the `wallCut` stream). */
  private cutaway(mat: pc.StandardMaterial): void {
    mat.setAttribute('game_wallCut', WALL_CUT_SEMANTIC);
    mat.setDefine('GAME_WALLCUT', '');
  }

  /** The lit material's part of the look: basements, the soft look, the diag view, the sky's ambient. */
  private light(mat: pc.StandardMaterial): void {
    mat.setDefine('GAME_UNDERGROUND', '');
    mat.setDefine('GAME_SOFT', this.soft ? '' : false);
    if (DIAG_VIEW_DEFINE) mat.setDefine(DIAG_VIEW_DEFINE, '');
    if (this.sky) (mat as Standard).ambientSH = this.sky.sh;
    this.lit.add(mat);
  }

  /** Gives `mat` the texture at `url` (a stand-in of the same kind until it has loaded). */
  private useTexture(mat: pc.StandardMaterial, map: TextureSlot['users'][number]['map'], url: string, srgb: boolean): void {
    const slot = this.textureSlot(url, srgb, map === 'normalMap' ? 'normal' : map === 'diffuseMap' ? 'white' : 'arm');
    mat[map] = slot.texture;
    if (!slot.loaded) slot.users.push({ material: mat, map });
  }

  /** A copy keeps getting its originals' textures as they load. */
  private reuseTextures(from: pc.StandardMaterial, to: pc.StandardMaterial): void {
    for (const slot of this.textures.values()) {
      if (slot.loaded) continue;
      for (const u of slot.users.slice()) if (u.material === from) slot.users.push({ material: to, map: u.map });
    }
  }

  /** (`srgb`: colour, in gamma space; loaded as it is all the same and decoded where it is read, see `textureWithFallback`.) */
  private textureSlot(url: string, srgb: boolean, kind: 'white' | 'normal' | 'arm'): TextureSlot {
    const id = `${url}|${srgb ? 'srgb' : 'linear'}`;
    let slot = this.textures.get(id);
    if (slot) return slot;
    const made: TextureSlot = { texture: standIn(this.device, kind), loaded: false, users: [] };
    this.textures.set(id, made);
    const job = textureWithFallback(this.app, url, this.anisotropy).then((texture) => {
      made.loaded = true;
      if (!texture) return;
      made.texture = texture;
      for (const { material, map } of made.users) {
        material[map] = texture;
        material.update();
      }
      made.users = [];
    });
    this.pending.add(job);
    void job.finally(() => this.pending.delete(job));
    return made;
  }
}

/** 1 × 1 textures standing in while a surface's own load (same kind and format, so the shader stays the same). */
const standIns = new WeakMap<pc.GraphicsDevice, Map<string, pc.Texture>>();
function standIn(device: pc.GraphicsDevice, kind: 'white' | 'normal' | 'arm'): pc.Texture {
  let byKind = standIns.get(device);
  if (!byKind) standIns.set(device, (byKind = new Map()));
  const id = kind;
  let tex = byKind.get(id);
  if (!tex) {
    // White albedo; a flat normal; AO 1, roughness 1, not metal.
    const texel = kind === 'normal' ? [128, 128, 255, 255] : kind === 'arm' ? [255, 255, 0, 255] : [255, 255, 255, 255];
    tex = new pc.Texture(device, { name: `standIn-${id}`, width: 1, height: 1, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, levels: [new Uint8Array(texel)] });
    byKind.set(id, tex);
  }
  return tex;
}

/**
 * A texture from the build's compressed copy (KTX2) when there is one, else from the file; a copy
 * that fails to load falls back to the file. Resolves to null when neither loads.
 *
 * Always in a linear (UNORM) format, colour maps too, as Babylon loaded the game's textures (its
 * `gammaSpace` textures, without sRGB buffers): a base or emissive colour map is then decoded where
 * the material reads it, `pow(c, 2.2)` (PlayCanvas does so for an 8-bit or block format that isn't
 * sRGB, `Texture.encoding`), and its mipmaps are averaged in gamma space (the GPU's, or the KTX2
 * file's), as Babylon's were. A hardware sRGB format decodes with the exact sRGB curve and
 * averages in linear light: 3–10 % brighter on the game's textured surfaces. Normal and packed
 * maps are read as they are either way.
 */
export function textureWithFallback(app: pc.AppBase, source: string, anisotropy?: number): Promise<pc.Texture | null> {
  // Normal and ARM maps sit next to the albedo (`new URL(entry.normal, base)`), without the build's query.
  const url = withBuild(source);
  const copy = optimizedUrl(url);
  const load = (file: string) =>
    new Promise<pc.Texture>((resolve, reject) => {
      const asset = new pc.Asset(file, 'texture', { url: file }, { srgb: false });
      asset.once('load', () => resolve(asset.resource as pc.Texture));
      asset.once('error', (err: unknown) => reject(new Error(String(err))));
      app.assets.add(asset);
      app.assets.load(asset);
    });
  return load(copy)
    .catch((err: unknown) => {
      if (copy === url) throw err;
      console.warn(`[render] compressed ${copy} failed, loading ${url}`);
      return load(url);
    })
    .then((texture) => {
      if (anisotropy !== undefined) texture.anisotropy = anisotropy;
      return texture;
    })
    .catch((err: unknown) => {
      console.warn(`[render] texture ${url} unavailable`, err);
      return null;
    });
}

/** Vertex-colour alpha for the plain finish: [0, 0.5) dielectric, [0.5, 1] metal; roughness scaled within. */
export function encodeFinish(roughness: number, metallic: number): number {
  const r = Math.min(0.999, Math.max(0, roughness)) * 0.5;
  return metallic >= 0.5 ? 0.5 + r : r;
}

/** Whether a lighting value moved enough to pass on (see `LIGHT_CHANGE`). */
function visibleChange(from: number, to: number): boolean {
  // Going off (or coming on) always counts.
  if (from === 0 || to === 0) return from !== to;
  return Math.abs(to - from) >= Math.max(LIGHT_CHANGE_MIN, Math.max(Math.abs(from), Math.abs(to)) * LIGHT_CHANGE);
}
