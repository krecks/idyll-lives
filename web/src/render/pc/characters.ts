/**
 * The residents on PlayCanvas: the engine-neutral residents (`shared/characters/residents.ts`: clips,
 * walking, sitting, faces, props, quilts and every number drawn) drawn through `PlayCanvasResidentView`,
 * the PlayCanvas side of `ResidentView` (`shared/characters/view.ts`).
 *
 * - The pose texture (`poseTexture.ts`): RGBA32F, a row per resident, the band of rows that changed sent
 *   each frame; the face morphs' texture beside it. Both handed to every resident material once
 *   (`setResidentTextures`, `characterMaterial.ts`).
 * - One mesh per body and part (`residentMesh`: the part's fields, morph slots and tone interleaved),
 *   drawn instanced (`instancing.ts`): an identity matrix and the resident's row per instance
 *   (`RESIDENT_ROW`), skinned on the GPU from that row; never culled (the bones carry the placement).
 *   The residents' code rewrites the rows of the parts drawn by distance and face detail per frame;
 *   the instance buffer is uploaded only when they changed.
 * - Shadow-only copies (the low-detail body and clothes, where the preset has residents cast shadows):
 *   the same meshes on the shadow-only layer (`layers.ts`), cast into the sun's map, never drawn by a
 *   camera. The full meshes cast only for a set without low-detail parts (`casters`).
 * - Quilts (one mesh per body, `blanketMaterial`), the selection ring and the props held
 *   (`residentProps.ts`) as instanced draws whose data the residents' code rewrites in place.
 *
 * Game space: everything hangs under the world root (`space.ts`); the camera's position goes back
 * to game space for the faces' detail. Per frame: no allocations.
 */

import * as pc from './engine';
import { BLANKET_EXT, BLANKET_TINT, blanketMaterial, characterMaterial, createMorphTexture, RESIDENT_ROW, residentMesh, setResidentTextures } from './characterMaterial';
import { Instances } from './instancing';
import { shadowOnlyLayer } from './layers';
import { contactBlob } from './lotContents';
import type { MaterialLibrary } from './materials';
import { fileColour, onModelFileArrived } from './models';
import { PoseTexture } from './poseTexture';
import { ResidentProps, type Prop } from './residentProps';
import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry } from '../../assets/types';
import type { Content } from '../../content/content';
import type { QuiltGeometry } from '../shared/characters/blanket';
import type { BodyData, CharacterSet } from '../shared/characters/data';
import { Characters as SharedCharacters } from '../shared/characters/residents';
import { MAX_CHARACTERS } from '../shared/characters/rig';
import type { ResidentView, RowChange } from '../shared/characters/view';
import { computeNormals } from '../shared/modelParts';
import type { TemplateCache } from '../shared/templateCache';

export { EXPRESSIONS, FRIENDLY, STANCE, type StageDirection } from '../shared/characters/expressions';
export type { Face } from '../shared/characters/face';
export { MAX_CHARACTERS } from '../shared/characters/rig';
export { GAME_RATE, QUIET_SPEED } from '../shared/gameSpeed';

/** The selection ring's glow (sRGB). */
const RING_COLOR = '#7BE38A';

const IDENTITY = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);

/** A part mesh as drawn (`ResidentView`'s `M`): its instanced draw, switched on, and how many it draws. */
export interface PartDraw {
  readonly draw: Instances;
  readonly shadow: boolean;
  enabled: boolean;
  count: number;
  /** The rows it draws (4 numbers each, the resident's row first; `count` of them), as last written. */
  rows: Float32Array | null;
}

/** A body's quilt as drawn (`ResidentView`'s `Q`). */
export interface QuiltDraw {
  readonly draw: Instances;
  readonly matrices: Float32Array;
  readonly ext: Float32Array;
  readonly tint: Float32Array;
  enabled: boolean;
  count: number;
}

export interface ResidentViewOptions {
  /** The camera the residents are seen by (the faces' detail, the eyes' glances); default: the app's first enabled camera. */
  camera?: () => pc.CameraComponent | null;
}

export class PlayCanvasResidentView implements ResidentView<PartDraw, QuiltDraw, Prop> {
  private readonly device: pc.GraphicsDevice;
  private poses: PoseTexture | null = null;
  private morphs: pc.Texture | null = null;
  private readonly shadowLayer: pc.Layer;
  private readonly parts: PartDraw[] = [];
  /** The part meshes by their draw's mesh instance (`partOf`). */
  private readonly byInstance = new WeakMap<pc.MeshInstance, PartDraw>();
  private readonly quilts: QuiltDraw[] = [];
  private quiltMat: pc.Material | null = null;
  private ring: Instances | null = null;
  private ringMatrix: Float32Array | null = null;
  private ringMaterial: pc.StandardMaterial | null = null;
  private readonly propDraws: ResidentProps;
  readonly props: TemplateCache<Prop | null>;

  constructor(
    private readonly app: pc.AppBase,
    /** Game space (the world root, or a dev page's). */
    private readonly root: pc.Entity,
    assets: AssetRegistry,
    private readonly options: ResidentViewOptions = {},
  ) {
    this.device = app.graphicsDevice;
    this.shadowLayer = shadowOnlyLayer(app);
    this.propDraws = new ResidentProps(app, root, assets, MAX_CHARACTERS);
    this.props = this.propDraws.cache;
  }

  // --- the pose texture ------------------------------------------------------------------

  init(set: CharacterSet, data: Float32Array, width: number, rows: number): void {
    this.poses = new PoseTexture(this.device, width, rows, data);
    const m = set.morphs;
    this.morphs = m ? createMorphTexture(this.device, m) : null;
    setResidentTextures(this.app, { poses: this.poses.texture, bones: set.bones.length, morphs: this.morphs, morphWidth: m?.width ?? 0, morphHeader: m?.headerTexels ?? 0 });
  }

  uploadPoses(): void {
    this.poses?.uploadAll();
  }

  uploadPoseRows(from: number, count: number): void {
    this.poses?.uploadRows(from, from + count - 1);
  }

  // --- part meshes -----------------------------------------------------------------------

  createPart(body: BodyData, part: string, shadow: boolean): PartDraw {
    const data = body.parts.get(part)!;
    const material = characterMaterial(this.app, part, { shadow, morph: !shadow && !!data.morph });
    const draw = new Instances(this.device, this.root, residentMesh(this.device, data), material, {
      name: `${shadow ? 'simShadow' : 'sim'}:${body.name}:${part}`,
      streams: [RESIDENT_ROW],
      // (Cast from the shadow-only copies; the full meshes only for a set without them: `castFrom`.)
      castShadows: shadow,
      receiveShadows: !shadow,
      cull: false,
      ...(shadow ? { layers: [this.shadowLayer.id] } : {}),
    });
    // The mesh is this draw's alone (a draw holds it; destroying the draw lets go of it).
    draw.mesh.decRefCount();
    // Enabled, drawing nothing until its rows come (`setPartRows`).
    draw.enabled = false;
    const pd: PartDraw = { draw, shadow, enabled: true, count: 0, rows: null };
    this.parts.push(pd);
    this.byInstance.set(draw.meshInstance, pd);
    return pd;
  }

  /** The part mesh `mi` draws, if it draws one (the mirrors draw only the residents they show: `Characters.rowsOf`). */
  partOf(mi: pc.MeshInstance): PartDraw | null {
    return this.byInstance.get(mi) ?? null;
  }

  partEnabled(mesh: PartDraw, on: boolean): void {
    mesh.enabled = on;
    this.show(mesh);
  }

  setPartRows(mesh: PartDraw, rows: Float32Array, _perFrame: boolean): void {
    this.writeRows(mesh, rows, rows.length >> 2);
  }

  partRows(mesh: PartDraw, rows: Float32Array, count: number, change: RowChange): void {
    if (change === 'count') {
      // (The rows already uploaded: only how many of them are drawn.)
      mesh.count = mesh.draw.count = count;
      mesh.rows = rows;
      mesh.draw.meshInstance.instancingCount = count;
      this.show(mesh);
      return;
    }
    this.writeRows(mesh, rows, count);
  }

  /** `mesh` draws the first `count` rows of `rows` (4 numbers each, the row first), uploaded. */
  private writeRows(mesh: PartDraw, rows: Float32Array, count: number): void {
    const d = mesh.draw;
    d.setCount(count);
    for (let j = 0; j < count; j++) {
      d.setMatrix(j, IDENTITY);
      d.set(RESIDENT_ROW, j, rows, j * 4);
    }
    d.update(false);
    mesh.count = count;
    mesh.rows = rows;
    this.show(mesh);
  }

  /** Drawn while enabled and drawing any (PlayCanvas would draw an instanced mesh with none once, uninstanced). */
  private show(mesh: PartDraw): void {
    const on = mesh.enabled && mesh.count > 0;
    if (mesh.draw.enabled !== on) mesh.draw.enabled = on;
  }

  /** The full meshes in `casters` cast the sun's shadow, the others don't (the shadow-only copies always do). */
  castFrom(casters: readonly PartDraw[]): void {
    for (const p of this.parts) if (!p.shadow) p.draw.meshInstance.castShadow = casters.includes(p);
  }

  // --- quilts ----------------------------------------------------------------------------

  quiltMaterial(toeZ: number, footZ: number): void {
    this.quiltMat = blanketMaterial(this.app, toeZ, footZ);
  }

  createQuilt(name: string, geometry: QuiltGeometry, matrices: Float32Array, ext: Float32Array, tint: Float32Array): QuiltDraw {
    const mesh = new pc.Mesh(this.device);
    mesh.setPositions(geometry.positions);
    // (Smooth, as Babylon's `VertexData.ComputeNormals` made them.)
    const normals = new Float32Array(geometry.positions.length);
    computeNormals(geometry.positions, geometry.indices, normals);
    mesh.setNormals(normals);
    mesh.setUvs(0, geometry.uvs);
    mesh.setIndices(geometry.indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const draw = new Instances(this.device, this.root, mesh, this.quiltMat!, { name, streams: [BLANKET_EXT, BLANKET_TINT], castShadows: false, receiveShadows: true, cull: false, capacity: MAX_CHARACTERS });
    draw.mesh.decRefCount();
    draw.enabled = false;
    const q: QuiltDraw = { draw, matrices, ext, tint, enabled: false, count: 0 };
    this.quilts.push(q);
    return q;
  }

  quiltRows(quilt: QuiltDraw, count: number, upload: boolean): void {
    const d = quilt.draw;
    d.setCount(count);
    if (upload) {
      for (let j = 0; j < count; j++) {
        d.setMatrix(j, quilt.matrices, j * 16);
        d.set(BLANKET_EXT, j, quilt.ext, j * 2);
        d.set(BLANKET_TINT, j, quilt.tint, j * 3);
      }
      d.update(false);
    } else d.meshInstance.instancingCount = count;
    quilt.count = count;
    this.showQuilt(quilt);
  }

  quiltEnabled(quilt: QuiltDraw, on: boolean): void {
    quilt.enabled = on;
    this.showQuilt(quilt);
  }

  private showQuilt(quilt: QuiltDraw): void {
    const on = quilt.enabled && quilt.count > 0;
    if (quilt.draw.enabled !== on) quilt.draw.enabled = on;
  }

  // --- the selection ring ----------------------------------------------------------------

  createRing(matrix: Float32Array): void {
    const size = 256;
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    const ctx = canvas.getContext('2d')!;
    const g = ctx.createRadialGradient(size / 2, size / 2, size * 0.3, size / 2, size / 2, size / 2);
    g.addColorStop(0, 'rgba(255,255,255,0)');
    g.addColorStop(0.55, 'rgba(255,255,255,0.0)');
    g.addColorStop(0.72, 'rgba(255,255,255,0.85)');
    g.addColorStop(0.8, 'rgba(255,255,255,0.55)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, size, size);
    const texture = new pc.Texture(this.device, { name: 'simRing', width: size, height: size, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
    texture.setSource(canvas);
    const m = (this.ringMaterial = new pc.StandardMaterial());
    m.name = 'simRing';
    m.useLighting = false;
    m.useSkybox = false;
    m.diffuse.set(0, 0, 0);
    m.emissive = new pc.Color().fromString(RING_COLOR);
    m.opacityMap = texture;
    m.opacityMapChannel = 'a';
    m.blendType = pc.BLEND_NORMAL;
    m.depthWrite = false;
    // (Babylon's `zOffset` -4.)
    m.slopeDepthBias = -4;
    m.cull = pc.CULLFACE_NONE;
    m.update();
    // A 1 m square on the floor (the contact shadows' quad).
    this.ring = new Instances(this.device, this.root, contactBlob(this.app).mesh, m, { name: 'simRing', castShadows: false, receiveShadows: false, cull: false });
    this.ring.enabled = false;
    this.ringMatrix = matrix;
  }

  ringMoved(): void {
    const ring = this.ring;
    const m = this.ringMatrix;
    if (!ring || !m) return;
    // (All zeros: nobody selected, or not drawn.)
    const on = m[15] !== 0;
    if (ring.enabled !== on) ring.enabled = on;
    if (!on) return;
    ring.setCount(1);
    ring.setMatrix(0, m);
    ring.update(false);
  }

  // --- props -----------------------------------------------------------------------------

  buildProp(key: string): Promise<Prop | null> {
    return this.propDraws.build(key);
  }

  showProp(prop: Prop, count: number): void {
    this.propDraws.show(prop, count);
  }

  onModelFile(listener: (url: string) => void): () => void {
    return onModelFileArrived(this.app, listener);
  }

  fileColour(entry: ModelEntry | undefined, name: string): Promise<string | undefined> {
    return fileColour(this.app, entry, name);
  }

  // --- the camera ------------------------------------------------------------------------

  camera(out: Float32Array): boolean {
    const cam = this.options.camera ? this.options.camera() : firstCamera(this.app);
    if (!cam) return false;
    const p = cam.entity.getPosition();
    // (Game space: z turned round, `space.ts`.)
    out[0] = p.x;
    out[1] = p.y;
    out[2] = -p.z;
    out[3] = this.device.height;
    out[4] = cam.projection === pc.PROJECTION_PERSPECTIVE ? (cam.fov * Math.PI) / 180 : NaN;
    return true;
  }

  onDispose(fn: () => void): void {
    this.app.once('destroy', fn);
  }

  /** Lets go of everything drawn (the renderer or the page goes). */
  destroy(): void {
    for (const p of this.parts) p.draw.destroy();
    for (const q of this.quilts) q.draw.destroy();
    this.parts.length = 0;
    this.quilts.length = 0;
    this.ring?.destroy();
    this.ring = null;
    this.ringMaterial?.opacityMap?.destroy();
    this.ringMaterial?.destroy();
    this.propDraws.destroy();
    this.poses?.destroy();
    this.morphs?.destroy();
    this.poses = this.morphs = null;
  }
}

/** The app's first enabled camera (its main view). */
function firstCamera(app: pc.AppBase): pc.CameraComponent | null {
  for (const c of app.systems.camera!.cameras) if (c.enabled && c.entity.enabled) return c;
  return null;
}

export interface CharactersOptions extends ResidentViewOptions {
  /** Builds again the props drawn without a model file that came (the game's). */
  refreshProps?: boolean;
  /** Residents cast sun shadows (the shadow-only copies are built; `QualitySettings.characterShadows`). */
  castsShadows?: boolean;
}

/** The residents drawn by PlayCanvas under `root` (game space). */
export class Characters extends SharedCharacters<PartDraw, QuiltDraw, Prop> {
  private readonly pcView: PlayCanvasResidentView;

  constructor(app: pc.AppBase, root: pc.Entity, assets: AssetRegistry, _lib: MaterialLibrary, content?: Content, options: CharactersOptions = {}) {
    const view = new PlayCanvasResidentView(app, root, assets, { camera: options.camera });
    super(view, assets, content, { refreshProps: options.refreshProps, castsShadows: options.castsShadows });
    this.pcView = view;
  }

  /**
   * Whether the draw `mi` draws any of the resident rows set in `bits` (16 a component, as
   * `residentRowFilter`), null for any other draw (the mirrors: a part none of whose residents a
   * glass shows isn't drawn there). No allocations.
   */
  drawsRows(mi: pc.MeshInstance, bits: Float32Array): boolean | null {
    const part = this.pcView.partOf(mi);
    if (!part) return null;
    const rows = part.rows;
    if (!rows) return false;
    for (let j = 0; j < part.count; j++) {
      const row = rows[j * 4] | 0;
      if (Math.floor(bits[row >> 4] / 2 ** (row & 15)) % 2 === 1) return true;
    }
    return false;
  }

  /** The resident rows the draw `mi` draws now (`rowsOf`), null for any other draw (the mirrors). */
  rowsOfInstance(mi: pc.MeshInstance): { rows: Float32Array; count: number } | null {
    const part = this.pcView.partOf(mi);
    return part ? this.rowsOf(part) : null;
  }

  override async build(...args: Parameters<SharedCharacters<PartDraw, QuiltDraw, Prop>['build']>): Promise<void> {
    await super.build(...args);
    this.pcView.castFrom(this.casters());
  }

  override stress(n: number, x: number, z: number): void {
    super.stress(n, x, z);
    this.pcView.castFrom(this.casters());
  }

  override dispose(): void {
    super.dispose();
    this.pcView.destroy();
  }
}
