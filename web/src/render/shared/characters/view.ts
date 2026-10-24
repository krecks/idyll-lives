/**
 * What the residents' engine-neutral code (`residents.ts`: clips, walking, sitting, faces, props,
 * quilts, the pose texture's data) asks of the engine that draws them: one `ResidentView` per
 * renderer or preview, created before the `Characters` it serves (`babylon/characters.ts`,
 * `pc/characters.ts`).
 *
 * The residents' code owns every number that is drawn: the pose texture's data (one row per
 * resident, `layout.ts`), each part mesh's rows, the props' and quilts' instance data and the
 * selection ring's matrix. The engine owns the meshes, materials and textures, and is told when
 * those numbers change. Matrices are 16 floats, translation at 12, 13, 14 (Babylon's matrices,
 * PlayCanvas's `Mat4.data`), in game space (`shared/math.ts`). Nothing here is called per frame
 * with new arrays: every array is given once and rewritten in place.
 *
 * Calls come in the order a renderer builds its scene in (texture, quilts, ring at `init`; parts
 * as residents first wear them), so an engine that draws in creation order draws as Babylon did.
 */

import type { ModelEntry } from '../../../assets/types';
import type { TemplateCache } from '../templateCache';
import type { QuiltGeometry } from './blanket';
import type { BodyData, CharacterSet } from './data';

/** A thing residents carry or hold (content `items`, a station's tool, an object's held part), as the engine built it. */
export interface ResidentProp {
  /** One matrix per resident row (`MAX_CHARACTERS` of them), drawn as instances; a row all zeros draws nothing. */
  readonly matrices: Float32Array;
  /** Its top: the highest point of a part (its own frame, as built: a mop's handle, a guitar's head); [0, 0, 0] for a whole model. */
  readonly top: readonly [number, number, number];
  /** The files it was built without (a small block then, `ModelTemplate.failed`). */
  readonly failed?: string[];
}

/**
 * How a part's rows changed (`ResidentView.partRows`): only how many are drawn (`count`), the array last given
 * rewritten in place (`rows`: the parts drawn by distance and detail, per frame), or a new array in its place
 * that stays (`new`: the shells' shadow copies, when someone's clothes change).
 */
export type RowChange = 'count' | 'rows' | 'new';

/**
 * The engine's side of the residents. `M`: a part mesh (one per body and part, drawn instanced, one instance per
 * resident row wearing it); `Q`: a quilt mesh (one per body); `P`: a prop as built.
 */
export interface ResidentView<M extends object, Q, P extends ResidentProp> {
  /**
   * Once, as the set has loaded: the pose texture, `width` x `rows` RGBA32F texels from `data` (nearest, no mips,
   * rewritten in place and uploaded by `uploadPoses` / `uploadPoseRows`); then, for a set with face morphs
   * (`set.morphs`), the morph texture (half floats, as `morphs.data`).
   */
  init(set: CharacterSet, data: Float32Array, width: number, rows: number): void;
  /** Uploads the whole pose texture (`build`). */
  uploadPoses(): void;
  /** Uploads rows `from` .. `from + count - 1` of the pose texture (per frame: the band of rows that changed). */
  uploadPoseRows(from: number, count: number): void;

  /**
   * The mesh of `body`'s part `part` (`BodyData.parts`; its vertex fields, morph slots and tone, skinned on the GPU
   * from the pose texture by the row of each instance), enabled, its rows given by `setPartRows` before the next
   * frame. Its material by the part's surface (`parts.ts` `surfaceOf`; `babylon/characters/material.ts`).
   * `shadow`: its shadow-only copy, drawn into the sun's shadow map only (never by a camera), receiving no shadows.
   */
  createPart(body: BodyData, part: string, shadow: boolean): M;
  /** Enables or disables a part mesh (a disabled one draws nothing and casts nothing). */
  partEnabled(mesh: M, on: boolean): void;
  /**
   * The rows `mesh` draws from now on, all of them: `rows`, 4 numbers per instance (its resident's row twice, then
   * two zeros: Babylon's baked-vertex-animation settings). `perFrame`: `partRows(mesh, n, 'rows')` rewrites the
   * array and its count per frame; otherwise it stays as it is (until `setPartRows` or `partRows(…, 'new')`).
   */
  setPartRows(mesh: M, rows: Float32Array, perFrame: boolean): void;
  /** `mesh` draws the first `count` instances of its rows (none: not drawn at all), changed as `change` says; `rows` is the array it draws from now. */
  partRows(mesh: M, rows: Float32Array, count: number, change: RowChange): void;

  /** Once, before the first quilt: the quilts' material, the cover stretched past the toes from local z `toeZ` to `footZ`. */
  quiltMaterial(toeZ: number, footZ: number): void;
  /**
   * A body's quilt mesh (`geometry`, drawn instanced from `matrices`, 16 floats each, `ext`, 2 each: how far
   * past the toes the quilt reaches and where it is folded down, and `tint`, 3 each: its linear colour over the
   * cover's grey), disabled and drawing nothing.
   */
  createQuilt(name: string, geometry: QuiltGeometry, matrices: Float32Array, ext: Float32Array, tint: Float32Array): Q;
  /** The quilt draws its first `count` instances (none: not drawn at all); `upload`: their data was rewritten (per frame), else only the count changed (`build`). */
  quiltRows(quilt: Q, count: number, upload: boolean): void;
  /** Enables or disables a quilt mesh. */
  quiltEnabled(quilt: Q, on: boolean): void;

  /** Once, at `init`: the selection ring, a 1 m glowing ring on the floor drawn with one instance from `matrix` (all zeros: hidden). */
  createRing(matrix: Float32Array): void;
  /** `matrix` was rewritten (per frame). */
  ringMoved(): void;

  /** The props' templates (built by `buildProp`, set aside and disposed by the engine's hooks, `shared/templateCache.ts`). */
  readonly props: TemplateCache<P | null>;
  /**
   * Builds prop `key`: a model (`<model key>`, an item's or a tool's: the whole model, its top [0, 0, 0]), or a moving
   * part of an object's model on its own (`<model key>#<part>`, then `@<colorsKey>` for the colours the player gave
   * it), in the part's own frame (its node's origin and axes at rest), with its top; null if the model has no such
   * part. Without the model's file, a small block (`failed`). Its `matrices` (`MAX_CHARACTERS` x 16, all zeros) are
   * its instances' from then on, nothing drawn until `showProp`.
   */
  buildProp(key: string): Promise<P | null>;
  /** Per frame, for every built prop: draws its first `count` instances (their matrices rewritten), or, 0, none. */
  showProp(prop: P, count: number): void;
  /** Calls `listener` with each model file that comes after failing (`onModelFileArrived`); returns how to stop. */
  onModelFile(listener: (url: string) => void): () => void;
  /** The colour (sRGB hex) material `name` of the model `entry` has in its file (`fileColour`; undefined: none). */
  fileColour(entry: ModelEntry | undefined, name: string): Promise<string | undefined>;

  /**
   * The camera the residents are drawn by now, into `out`: its position (x, y, z, game space), the render height
   * (px) and its vertical field of view (rad; NaN for an orthographic camera). False without one. For the faces'
   * detail by their size on screen and for the eyes that glance at the camera.
   */
  camera(out: Float32Array): boolean;
  /** Calls `fn` once when the scene or app goes (`?debug` hooks). */
  onDispose(fn: () => void): void;
}
