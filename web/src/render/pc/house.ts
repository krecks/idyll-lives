/**
 * The house builder on PlayCanvas: the viewed lot's house and the street's houses as the
 * engine-neutral planner writes them (`shared/house.ts`: walls, floors, roofs, doors and windows,
 * the cutaway's data), each part one entity drawing one mesh in the material library's surfaces,
 * and the viewed house's clear glass glowing by room at night.
 *
 * Everything is in game space under the entity it is given (a storey's, lifted to its height,
 * under the world root). A door leaf that swings hangs on an entity of its own at its pivot,
 * which the renderer turns (and slides) to open and shut it. The lid over the rooms that keeps
 * the sun out of them (`ceilingShadow`) is on the shadow-only layer (`sunShadows.ts`). The walls'
 * cutaway is the materials' (`shaders/cutaway.ts`): every part that follows it carries the
 * cutaway attribute.
 */

import * as pc from './engine';
import { MaterialLibrary, type SurfacePattern } from './materials';
import { drawMesh, meshFrom } from './meshes';
import { shadowOnlyLayer } from './sunShadows';
import type { Opening, StoreyLayout, WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import type { RGB } from '../shared/environment';
import type { Geo } from '../shared/geometry';
import { HOUSE_MATERIALS, HousePlanner, type DoorLeaf, type HouseLooks, type HousePart, type HousePlan, type LaidPreview, type SilhouettePlan, type SilhouetteSpec, type StoreyBuild } from '../shared/house';

/** A part as drawn: its entity (and the part). */
export interface HouseMesh {
  entity: pc.Entity;
  part: HousePart;
}

/**
 * A door leaf that swings: its parts hang on `pivot` (an entity at the point they turn about, under
 * the storey's), which the renderer turns about y (`close`, in game space) and moves by `slide`
 * from `base` (its place, as built).
 */
export type HouseDoor = Omit<DoorLeaf<never>, 'meshes'> & { pivot: pc.Entity; base: pc.Vec3 };

export interface HouseBuild extends Omit<HousePlan, 'parts' | 'doors'> {
  /** Every part drawn (the empty ones aren't), in the planner's order. */
  meshes: HouseMesh[];
  /** Roofs and gables: drawn only with walls up. */
  roofs: HouseMesh[];
  doors: HouseDoor[];
}

export interface SilhouetteBuild extends Omit<SilhouettePlan, 'parts'> {
  meshes: HouseMesh[];
}

export class HouseBuilder {
  private readonly lib: MaterialLibrary;
  /** The engine-neutral geometry (`shared/house.ts`), also the street's (`shared/street.ts`). */
  readonly planner = new HousePlanner();
  private readonly device: pc.GraphicsDevice;
  private readonly shadowOnly: number;

  constructor(app: pc.AppBase) {
    this.lib = MaterialLibrary.for(app);
    this.device = app.graphicsDevice;
    this.shadowOnly = shadowOnlyLayer(app).id;
  }

  /** The content's wall coverings, door and window styles (structures refer to them by index). */
  setLooks(looks: HouseLooks): void {
    this.planner.setLooks(looks);
  }

  /** A house surface (cached by the library), vertex-coloured, with the cutaway or without. */
  material(key: string, cut = true, pattern?: SurfacePattern): pc.StandardMaterial {
    return this.lib.surface(key, { vertexColors: true, cutaway: cut, pattern });
  }

  /**
   * Build mode's Paint tool: wall faces (from `HouseBuild.surfaces`) as wall covering `cover`
   * (1-based) would dress them, in its own material, under `parent`. Null when there's nothing to show.
   */
  coverPreview(parts: readonly Geo[], cover: number, parent: pc.Entity): pc.Entity | null {
    const laid = this.planner.coverPreview(parts, cover);
    return laid && this.laid(laid, parent);
  }

  /** Build mode's Floor tool: floor tiles (from `HouseBuild.surfaces`) laid with floor covering `floor` (1-based). */
  floorPreview(parts: readonly Geo[], floor: number, parent: pc.Entity): pc.Entity | null {
    const laid = this.planner.floorPreview(parts, floor);
    return laid && this.laid(laid, parent);
  }

  private laid(part: LaidPreview, parent: pc.Entity): pc.Entity | null {
    const mesh = meshFrom(this.device, part.geo.meshData({ colors: true, cut: part.cut }));
    return mesh && drawMesh(parent, part.name, mesh, this.material(part.surface.key, part.cut, part.surface.pattern));
  }

  /**
   * Night look of the viewed house's clear glass: a faint warm sheen once the lamps are on
   * (the lit rooms themselves show through it). `lamps` is the lighting key's lamp level, `glow`
   * the lamps' colour (linear).
   */
  setNight(lamps: number, glow: RGB): void {
    const g = Math.min(1, Math.max(0, (lamps - 0.35) / 0.5));
    const k = g * g * 0.35;
    this.glassGlow.set(glow.r * k, glow.g * k, glow.b * k);
    this.lib.setEmissive(this.material(HOUSE_MATERIALS.clearGlass), this.glassGlow);
    this.setWindowGlow(this.windowGlow);
  }
  private readonly glassGlow = new pc.Color(0, 0, 0);

  /**
   * How much each room's windows glow (0 out, 1 full; rooms not listed glow): the windows of a
   * room where everyone is asleep go dark with its lights. Kept for the glass of later builds.
   */
  setWindowGlow(levels: ReadonlyMap<number, number>): void {
    this.windowGlow = levels;
    for (const [room, mat] of this.roomGlass) this.lib.setEmissive(mat, this.roomGlowOf(room));
  }
  private windowGlow: ReadonlyMap<number, number> = new Map();
  /** The viewed house's clear glass for each room it lets light out of (its own glow). */
  private readonly roomGlass = new Map<number, pc.StandardMaterial>();
  private readonly roomGlow = new pc.Color(0, 0, 0);

  private roomGlowOf(room: number): pc.Color {
    const k = this.windowGlow.get(room) ?? 1;
    const g = this.glassGlow;
    return this.roomGlow.set(g.r * k, g.g * k, g.b * k);
  }

  /** Clear glass for windows and glazed doors of `room` (0: outdoors, shared; previews share one too). */
  private glassMaterial(room: number, cut: boolean): pc.StandardMaterial {
    if (!room || !cut) return this.material(HOUSE_MATERIALS.clearGlass, cut);
    let mat = this.roomGlass.get(room);
    if (!mat) {
      mat = this.lib.surface(HOUSE_MATERIALS.clearGlass, { vertexColors: true, cutaway: true, variant: `room${room}` });
      this.roomGlass.set(room, mat);
      this.lib.setEmissive(mat, this.roomGlowOf(room));
    }
    return mat;
  }

  /** Neighbours' window glass: the street's, glowing at night (`lit`), or a variant that doesn't. */
  neighbourGlass(lit: boolean): pc.StandardMaterial {
    return lit ? this.material(HOUSE_MATERIALS.glass) : this.lib.surface(HOUSE_MATERIALS.glass, { vertexColors: true, cutaway: true, variant: 'dark' });
  }

  /** What a part is drawn with. */
  private materialOf(part: HousePart): pc.Material {
    const s = part.surface;
    switch (s.kind) {
      case 'surface':
        return this.material(s.key, part.cut, s.pattern);
      case 'glass':
        return this.glassMaterial(s.room, part.cut);
      case 'contactShadow':
        return this.lib.contactShadow(part.cut);
      case 'neighbourGlass':
        return this.neighbourGlass(s.lit);
    }
  }

  /** A part's entity under `parent` (null when it's empty). */
  private draw(part: HousePart, parent: pc.Entity): HouseMesh | null {
    const mesh = meshFrom(this.device, part.geo.meshData({ colors: true, cut: part.cut }));
    if (!mesh) return null;
    const entity = drawMesh(parent, part.name, mesh, this.materialOf(part), {
      castShadows: part.cast,
      receiveShadows: !part.shadowOnly,
      ...(part.shadowOnly ? { layers: [this.shadowOnly] } : {}),
      // Glass is see-through: drawn after the other blended surfaces (contact shadows).
      alphaIndex: part.glass ? 10 : 0,
    });
    return { entity, part };
  }

  /** The viewed lot's house on one storey (`storey`), under `parent` (lifted to the storey's height); null without a wall layout. */
  build(world: WorldStructure, view: ViewRect | null, storey: StoreyBuild, parent: pc.Entity): HouseBuild | null {
    const plan = this.planner.build(world, view, storey);
    if (!plan) return null;
    const meshes: HouseMesh[] = [];
    const roofs: HouseMesh[] = [];
    const doorParts = new Map<HousePart, HouseDoor>();
    const doors = plan.doors.map(({ parts, pivot, ...leaf }): HouseDoor => {
      const hinge = new pc.Entity('doorPivot');
      hinge.setLocalPosition(pivot[0], 0, pivot[1]);
      parent.addChild(hinge);
      const door: HouseDoor = { ...leaf, pivot: hinge, base: new pc.Vec3(pivot[0], 0, pivot[1]) };
      for (const p of parts) doorParts.set(p, door);
      return door;
    });
    for (const part of plan.parts) {
      const door = doorParts.get(part);
      const drawn = this.draw(part, door ? door.pivot : parent);
      if (!drawn) continue;
      // A leaf's parts in its pivot's frame.
      if (door) drawn.entity.setLocalPosition(-door.base.x, 0, -door.base.z);
      meshes.push(drawn);
      if (part.roof) roofs.push(drawn);
    }
    return { meshes, roofs, rooms: plan.rooms, shrubs: plan.shrubs, trees: plan.trees, surfaces: plan.surfaces, doors };
  }

  /**
   * The top of a neighbouring house's roof (m), as `silhouettes` builds it on its box of walls (the
   * plot's `house`, its roof style and storeys): for the birds, who fly over it.
   */
  roofTop(plot: { x: number; z: number; house: [number, number, number, number]; roof?: [number, number]; storeys?: number }): number {
    return this.planner.roofTop(plot);
  }

  /** Neighbouring houses as dressed boxes (`HousePlanner.silhouettes`) under `parent`. `layout`: where each storey's rows of the lot are. */
  silhouettes(houses: readonly SilhouetteSpec[], openings: readonly Opening[], layout: StoreyLayout | null, parent: pc.Entity): SilhouetteBuild {
    return this.silhouetteMeshes(this.planner.silhouettes(houses, openings, layout), parent);
  }

  /** The meshes of neighbouring houses as planned (`HousePlanner.silhouettes`). */
  silhouetteMeshes({ parts, homes, lamps }: SilhouettePlan, parent: pc.Entity): SilhouetteBuild {
    const meshes = parts.map((p) => this.draw(p, parent)).filter((m) => m !== null);
    return { meshes, homes, lamps };
  }

  /**
   * Neighbours' window glass (`SilhouetteBuild.homes`' panes) as one mesh: glowing at night as the
   * street's (`lit`), or dark (everyone in asleep, or nobody home). Null when there's none.
   */
  neighbourWindows(panes: readonly Geo[], lit: boolean, parent: pc.Entity): HouseMesh | null {
    return this.draw(this.planner.neighbourPanes(panes, lit), parent);
  }
}
