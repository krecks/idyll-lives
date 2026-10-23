/**
 * The house builder on Babylon: the viewed lot's house and the street's houses as the
 * engine-neutral planner writes them (`shared/house.ts`: walls, floors, roofs, doors and windows,
 * the cutaway's data), uploaded into one mesh per part with the material library's surfaces, and
 * the viewed house's clear glass glowing by room at night.
 */

import { Color3, Vector3, type Material, type Mesh, type PBRMaterial, type Scene } from './core';
import { toMesh } from './meshes';
import { LAYER_SHADOW_ONLY } from '../shared/layers';
import { MaterialLibrary } from './materials';
import type { WorldStructure, Opening, StoreyLayout } from '../../core/protocol';
import type { ViewRect } from '../types';
import type { Geo } from '../shared/geometry';
import {
  HOUSE_MATERIALS,
  HousePlanner,
  type DoorLeaf as PlannedLeaf,
  type HouseLooks,
  type HousePart,
  type HousePlan,
  type SilhouetteSpec,
  type SilhouettePlan,
  type StoreyBuild,
  type SurfacePattern,
} from '../shared/house';

export {
  HALF_WALL_HEIGHT,
  HOUSE_MATERIALS,
  houseObjectKey,
  houseScheme,
  placeLights,
  WALL_HALF,
  WALL_HEIGHT,
  WALL_STUB,
  type HouseLooks,
  type HouseScheme,
  type HouseSurfaces,
  type LampLight,
  type RoofShape,
  type RoomLight,
  type RoomTiles,
  type SilhouetteSpec,
  type StoreyBuild,
} from '../shared/house';

/** A door leaf that swings, with its meshes (see `shared/house.ts` `DoorLeaf`). */
export type DoorLeaf = PlannedLeaf<Mesh>;

export interface HouseBuild extends Omit<HousePlan, 'parts' | 'doors'> {
  meshes: Mesh[];
  /** Shadow casters. */
  casters: Mesh[];
  /** Roofs and gables: drawn only with walls up. */
  roofs: Mesh[];
  /** Door leaves that swing shut (front doors, bathroom doors); the rest stand open. */
  doors: DoorLeaf[];
}

export interface SilhouetteBuild extends Omit<SilhouettePlan, 'parts'> {
  meshes: Mesh[];
  casters: Mesh[];
}

export class HouseBuilder {
  private readonly lib: MaterialLibrary;
  /** The engine-neutral geometry (`shared/house.ts`), also the street's (`shared/street.ts`). */
  readonly planner = new HousePlanner();

  constructor(private readonly scene: Scene) {
    this.lib = MaterialLibrary.for(scene);
  }

  /** The content's wall coverings, door and window styles (structures refer to them by index). */
  setLooks(looks: HouseLooks): void {
    this.planner.setLooks(looks);
  }

  /** Materials (cached by the library) so the renderer can update window glow etc. */
  material(key: string, cut = true, pattern?: SurfacePattern): Material {
    return this.lib.surface(key, { vertexColors: true, cutaway: cut, pattern });
  }

  /**
   * Build mode's Paint tool: wall faces (from `HouseBuild.surfaces`) as wall covering `cover`
   * (1-based) would dress them, in its own material. Null when there's nothing to show.
   */
  coverPreview(parts: readonly Geo[], cover: number): Mesh | null {
    const laid = this.planner.coverPreview(parts, cover);
    return laid && this.laid(laid);
  }

  /** Build mode's Floor tool: floor tiles (from `HouseBuild.surfaces`) laid with floor covering `floor` (1-based). */
  floorPreview(parts: readonly Geo[], floor: number): Mesh | null {
    const laid = this.planner.floorPreview(parts, floor);
    return laid && this.laid(laid);
  }

  private laid(part: HousePart & { surface: { kind: 'surface'; key: string; pattern?: SurfacePattern } }): Mesh | null {
    const mesh = toMesh(part.geo, part.name, this.scene, { colors: true, cut: part.cut });
    if (!mesh) return null;
    mesh.material = this.material(part.surface.key, part.cut, part.surface.pattern);
    mesh.receiveShadows = true;
    mesh.isPickable = false;
    return mesh;
  }

  /**
   * Night look of the viewed house's clear glass: a faint warm sheen once the lamps are on
   * (the lit rooms themselves show through it). `lamps` is the lighting key's lamp level.
   */
  setNight(lamps: number, glow: Color3): void {
    const mat = this.material(HOUSE_MATERIALS.clearGlass) as PBRMaterial;
    const g = Math.min(1, Math.max(0, (lamps - 0.35) / 0.5));
    glow.scaleToRef(g * g * 0.35, this.glassGlow);
    this.lib.setEmissive(mat, this.glassGlow);
    this.setWindowGlow(this.windowGlow);
  }
  private readonly glassGlow = new Color3();

  /**
   * How much each room's windows glow (0 out, 1 full; rooms not listed glow): the windows of a
   * room where everyone is asleep go dark with its lights. Kept for the glass of later builds.
   */
  setWindowGlow(levels: ReadonlyMap<number, number>): void {
    this.windowGlow = levels;
    for (const [room, mat] of this.roomGlass) this.lib.setEmissive(mat, this.glassGlow.scaleToRef(levels.get(room) ?? 1, this.roomGlow));
  }
  private windowGlow: ReadonlyMap<number, number> = new Map();
  /** The viewed house's clear glass for each room it lets light out of (its own glow). */
  private readonly roomGlass = new Map<number, PBRMaterial>();
  private readonly roomGlow = new Color3();

  /** Clear glass for windows and glazed doors of `room` (0: outdoors, shared; previews share one too). */
  private glassMaterial(room: number, cut: boolean): Material {
    if (!room || !cut) return this.material(HOUSE_MATERIALS.clearGlass, cut);
    let mat = this.roomGlass.get(room);
    if (!mat) {
      mat = this.lib.surface(HOUSE_MATERIALS.clearGlass, { vertexColors: true, cutaway: true, variant: `room${room}` });
      this.roomGlass.set(room, mat);
      this.lib.setEmissive(mat, this.glassGlow.scaleToRef(this.windowGlow.get(room) ?? 1, this.roomGlow));
    }
    return mat;
  }

  /** What a part is drawn with. */
  private materialOf(part: HousePart): Material {
    const s = part.surface;
    switch (s.kind) {
      case 'surface':
        return this.material(s.key, part.cut, s.pattern);
      case 'glass':
        return this.glassMaterial(s.room, part.cut);
      case 'contactShadow':
        return this.lib.contactShadow();
      case 'neighbourGlass':
        return this.neighbourGlass(s.lit);
    }
  }

  /** A part's mesh (null when it's empty). */
  private meshOf(part: HousePart): Mesh | null {
    const mesh = toMesh(part.geo, part.name, this.scene, { colors: true, cut: part.cut });
    if (!mesh) return null;
    mesh.material = this.materialOf(part);
    mesh.receiveShadows = !part.shadowOnly;
    if (part.shadowOnly) mesh.layerMask = LAYER_SHADOW_ONLY;
    mesh.isPickable = false;
    // Glass is see-through: drawn after the other blended surfaces (contact shadows).
    if (part.glass) mesh.alphaIndex = 10;
    return mesh;
  }

  build(world: WorldStructure, view: ViewRect | null, storey: StoreyBuild = {}): HouseBuild | null {
    const plan = this.planner.build(world, view, storey);
    if (!plan) return null;
    const meshes: Mesh[] = [];
    const casters: Mesh[] = [];
    const roofs: Mesh[] = [];
    const of = new Map<HousePart, Mesh>();
    for (const part of plan.parts) {
      const mesh = this.meshOf(part);
      if (!mesh) continue;
      of.set(part, mesh);
      meshes.push(mesh);
      if (part.cast) casters.push(mesh);
      if (part.roof) roofs.push(mesh);
    }
    const doors = plan.doors.map(({ parts, pivot, ...leaf }): DoorLeaf => {
      const own = parts.map((p) => of.get(p)).filter((m) => m !== undefined);
      for (const mesh of own) mesh.setPivotPoint(new Vector3(pivot[0], 0, pivot[1]));
      return { ...leaf, meshes: own };
    });
    return { meshes, casters, roofs, rooms: plan.rooms, shrubs: plan.shrubs, trees: plan.trees, surfaces: plan.surfaces, doors };
  }

  /**
   * The top of a neighbouring house's roof (m), as `silhouettes` builds it on its box of walls (the
   * plot's `house`, its roof style and storeys): for the birds, who fly over it.
   */
  roofTop(plot: { x: number; z: number; house: [number, number, number, number]; roof?: [number, number]; storeys?: number }): number {
    return this.planner.roofTop(plot);
  }

  /**
   * Neighbouring houses as dressed boxes (`HousePlanner.silhouettes`): the meshes (one per material
   * for the whole street), the ones that cast shadows, and the porch / garage lamps for the street's
   * night lighting. `layout`: where each storey's rows of the lot are.
   */
  silhouettes(houses: readonly SilhouetteSpec[], openings: readonly Opening[], layout: StoreyLayout | null = null): SilhouetteBuild {
    return this.silhouetteMeshes(this.planner.silhouettes(houses, openings, layout));
  }

  /** The meshes of neighbouring houses as planned (`HousePlanner.silhouettes`). */
  silhouetteMeshes({ parts, homes, lamps }: SilhouettePlan): SilhouetteBuild {
    const out: SilhouetteBuild = { meshes: [], casters: [], homes, lamps };
    for (const part of parts) {
      const mesh = this.meshOf(part);
      if (!mesh) continue;
      out.meshes.push(mesh);
      if (part.cast) out.casters.push(mesh);
    }
    return out;
  }

  /**
   * Neighbours' window glass (`SilhouetteBuild.homes`' panes) as one mesh: glowing at night as the
   * street's (`lit`), or dark (everyone in asleep, or nobody home). Null when there's none.
   */
  neighbourWindows(panes: readonly Geo[], lit: boolean): Mesh | null {
    return this.meshOf(this.planner.neighbourPanes(panes, lit));
  }

  /** Neighbours' window glass: the street's, glowing at night (`lit`), or a variant that doesn't. */
  neighbourGlass(lit: boolean): Material {
    return lit ? this.material(HOUSE_MATERIALS.glass) : this.lib.surface(HOUSE_MATERIALS.glass, { vertexColors: true, cutaway: true, variant: 'dark' });
  }
}
