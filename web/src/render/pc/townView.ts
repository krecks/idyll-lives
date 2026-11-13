/**
 * The town view's drawing on PlayCanvas (docs/design/living-town.md, "Town-scale drawing"): what the
 * camera shows of the town once it zooms out past `TOWN_FROM` (`townView.blend > 0`), and nothing
 * before (the play camera draws as it always has).
 *
 * - **Buildings** (`TownBuildings`): every backdrop building of the world (`WorldMeta.backdrop`) and
 *   every far lot's house (the street's far LOD, `Street.townHouses`: the lots outside the chunks
 *   streamed in) as copies of five unit meshes: a box of walls, and a gable, hip, flat or mansard
 *   roof. One instanced draw per mesh for the whole world, in one material
 *   (`shaders/townBuilding.ts`: the walls' and the roof's colours per copy, the windows drawn by the
 *   fragment shader in bands a storey apart, a share of them lit at night: a shop's while it is open, a home's by
 *   the hour, a lot's house's while someone is up in it). A backdrop building's
 *   colours come from the backdrop palette (`BACKDROP_PALETTE`, by its `look`, varied a little per
 *   building), warm town tones: red and orange tile roofs over cream, sand and brick walls, grey
 *   slate here and there; its houses' gardens have hedges, low boxes in the same draws (`gardenHedges`);
 *   a lot's house keeps its own (`HousePlanner.farLook`, as the street's far
 *   LOD has it). A footprint that isn't a rectangle is drawn as the rectangle round it, along its
 *   longest side (or its ridge).
 * - **Movers** (`TownMovers`): the cars, taxis, bikes and residents the near drawing leaves out (past
 *   the chunks streamed in, past the residents' radius), as tiny instanced shapes in their own
 *   colours, where the snapshot has them (`Cars.forEachFar`, `Residents.spot`); larger the further out
 *   the camera is, so they read as the toy town's traffic.
 *
 * Both fade in with the blend (a dither: nothing pops in as the camera passes `TOWN_FROM`). Culling
 * is the frustum's (each draw's box over its copies); nothing here is left out by distance.
 */

import * as pc from './engine';
import { Instances, INSTANCE_COLOR, type InstanceStream } from './instancing';
import { MaterialLibrary } from './materials';
import { meshFrom } from './meshes';
import { withoutContact } from './post';
import { TOWN_INFO_SEMANTIC, TOWN_ROOF_SEMANTIC, TOWN_WALL_SEMANTIC, townBuildingMaterial } from './shaders/townBuilding';
// (The ground's colour map's shader feature, `groundTiles.ts` `TownGroundMap`: registered with the rest before any material is made.)
import './shaders/townGround';
import { Geo } from '../shared/geometry';
import { FINISH, litAt, ROOFS, unitMeshes, type RoofKind, type TownBuilding } from '../shared/townBuildings';

export { backdropBuilding, backdropGround, gardenHedges, lotBuilding, type TownLotHouse } from '../shared/townBuildings';

/** How bright a lit window glows at night (times the street's window glow). */
const GLOW = 1.4;

// ---- the buildings ------------------------------------------------------------------------

const TOWN_WALL: InstanceStream = { semantic: TOWN_WALL_SEMANTIC, components: 4 };
const TOWN_ROOF: InstanceStream = { semantic: TOWN_ROOF_SEMANTIC, components: 4 };
const TOWN_INFO: InstanceStream = { semantic: TOWN_INFO_SEMANTIC, components: 4 };

/** The town view's buildings: the walls' draw and a draw per roof, over every building given. */
export class TownBuildings {
  /** Its material and draws, made the first time it is shown (`made`): nothing of it exists for a game that never zooms out. */
  private material: pc.StandardMaterial | null = null;
  private readonly lib: MaterialLibrary;
  private readonly draws = new Map<'walls' | RoofKind, Instances>();
  private readonly meshes: pc.Mesh[] = [];
  private backdrop: TownBuilding[] = [];
  private lots: TownBuilding[] = [];
  /** The buildings given since the draws were last written. */
  private due = false;
  private readonly m = new Float32Array(16);
  private readonly v = new Float32Array(4);
  private shown = false;
  private readonly glow = new Float32Array(3);

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
  ) {
    this.lib = MaterialLibrary.for(app);
  }

  /**
   * Its material and draws, made once. Not before the town view first comes in: a material and draws made with the
   * game's own (if never drawn) put the game's draws in another order, and surfaces at the same depth (a decal on the
   * street) came out a few pixels differently in the play view.
   */
  private made(): pc.StandardMaterial {
    if (this.material) return this.material;
    const { app, parent, lib } = this;
    const mat = lib.plainFinish().clone();
    mat.name = 'townBuildings';
    lib.adopt(mat);
    townBuildingMaterial(mat);
    // No contact shading this far out, nor in the depth it is made from (`post.ts`).
    withoutContact(mat);
    mat.setParameter('game_townGlow', this.glow);
    this.material = mat;
    const units = unitMeshes();
    for (const key of ['walls', ...ROOFS] as const) {
      const mesh = meshFrom(app.graphicsDevice, units[key].meshData({ colors: true }))!;
      mesh.incRefCount();
      this.meshes.push(mesh);
      const draw = new Instances(app.graphicsDevice, parent, mesh, mat, { name: `town.${key}`, streams: [TOWN_WALL, TOWN_ROOF, TOWN_INFO], castShadows: false, receiveShadows: true, capacity: 256 });
      draw.meshInstance.setParameter('game_townPart', key === 'walls' ? 0 : 1);
      lib.excludeFromLights([], [draw.meshInstance]);
      draw.enabled = false;
      this.draws.set(key, draw);
    }
    this.due = true;
    return mat;
  }

  /** The backdrop's buildings (a new town's). */
  setBackdrop(list: TownBuilding[]): void {
    this.backdrop = list;
    this.due = true;
    if (this.shown) this.upload();
  }

  /**
   * The hour's lights: each backdrop building's share of lit windows at `minute` of the day on `weekday` (`litAt`: a
   * place's while it is open, a home's by the hour). The copies are written again only when a share changed.
   */
  setTime(minute: number, weekday: number): void {
    let changed = false;
    for (const b of this.backdrop) {
      const now = litAt(b, minute, weekday);
      if (now === b.now) continue;
      b.now = now;
      changed = true;
    }
    if (!changed) return;
    this.due = true;
    if (this.shown) this.upload();
  }

  /** The far lots' houses (as the chunks streamed in change, and as their people get up and go to bed). */
  setLots(list: TownBuilding[]): void {
    this.lots = list;
    this.due = true;
    if (this.shown) this.upload();
  }

  get count(): number {
    return this.backdrop.length + this.lots.length;
  }

  /** Shown (`fade` > 0) or not, fading in by `fade` (0..1). */
  show(fade: number): void {
    const on = fade > 0 && this.count > 0;
    if (on !== this.shown) {
      this.shown = on;
      if (on) this.made();
      if (on && this.due) this.upload();
      for (const d of this.draws.values()) d.enabled = on && d.count > 0;
    }
    if (on) this.material!.setParameter('game_townFade', fade);
  }

  /** The windows' glow at night: the lamps' level (0..1, `Lighting.lamps`) and the glow's colour (linear, the street's windows'). */
  setNight(level: number, color: { r: number; g: number; b: number }): void {
    const k = Math.max(0, Math.min(1, (level - 0.3) / 0.45)) * GLOW;
    this.glow[0] = color.r * k;
    this.glow[1] = color.g * k;
    this.glow[2] = color.b * k;
    this.material?.setParameter('game_townGlow', this.glow);
  }

  private upload(): void {
    this.made();
    this.due = false;
    const all = this.backdrop.length ? (this.lots.length ? [...this.backdrop, ...this.lots] : this.backdrop) : this.lots;
    const walls = this.draws.get('walls')!;
    walls.setCount(all.length);
    const counts = new Map<RoofKind, number>(ROOFS.map((r) => [r, 0]));
    for (const b of all) counts.set(b.roof, counts.get(b.roof)! + 1);
    for (const r of ROOFS) this.draws.get(r)!.setCount(counts.get(r)!);
    const at = new Map<RoofKind, number>(ROOFS.map((r) => [r, 0]));
    const m = this.m;
    all.forEach((b, i) => {
      // Walls: x along the rectangle, up to the eaves, z across.
      m.fill(0);
      m[0] = b.ux * b.w;
      m[2] = b.uz * b.w;
      m[5] = b.top - b.base;
      m[8] = -b.uz * b.d;
      m[10] = b.ux * b.d;
      m[12] = b.cx;
      m[13] = b.base;
      m[14] = b.cz;
      m[15] = 1;
      this.write(walls, i, b);
      // The roof over them.
      const roof = this.draws.get(b.roof)!;
      const k = at.get(b.roof)!;
      at.set(b.roof, k + 1);
      m[5] = b.rise;
      m[13] = b.top;
      this.write(roof, k, b);
    });
    for (const d of this.draws.values()) {
      d.update();
      d.enabled = this.shown && d.count > 0;
    }
  }

  private write(draw: Instances, i: number, b: TownBuilding): void {
    const v = this.v;
    draw.setMatrix(i, this.m);
    v.set([b.wall[0], b.wall[1], b.wall[2], b.floor - b.base]);
    draw.set(TOWN_WALL, i, v);
    v.set([b.roofColor[0], b.roofColor[1], b.roofColor[2], b.style]);
    draw.set(TOWN_ROOF, i, v);
    v.set([b.seed, b.top - b.floor, b.accent, b.now ?? b.lit]);
    draw.set(TOWN_INFO, i, v);
  }

  /** Draws and triangles (for the report). */
  stats(): { draws: number; copies: number; triangles: number } {
    let draws = 0;
    let triangles = 0;
    for (const d of this.draws.values()) {
      if (!d.count) continue;
      draws++;
      triangles += (d.mesh.primitive[0].count / 3) * d.count;
    }
    return { draws, copies: this.count, triangles };
  }

  destroy(): void {
    for (const d of this.draws.values()) d.destroy();
    this.draws.clear();
    for (const mesh of this.meshes) {
      mesh.decRefCount();
      if (mesh.refCount === 0) mesh.destroy();
    }
    if (this.material) {
      this.lib.release(this.material);
      this.material.destroy();
    }
  }
}

// ---- the movers ---------------------------------------------------------------------------

/** What moves, as the town view draws it. */
export type MoverKind = 'car' | 'taxi' | 'bike' | 'person';

/** The movers' shapes (m, as made; scaled up with the blend: `MOVER_GROW`). */
function moverMesh(kind: 'car' | 'bike' | 'person'): Geo {
  const g = new Geo().color('#FFFFFF', FINISH);
  if (kind === 'car') {
    // A body and a cabin, along +z (as the cars face: yaw 0 along +z).
    g.box(-0.85, 0.25, -2.1, 0.85, 1.0, 2.1, 'ny');
    g.color('#C8CCD0', FINISH).box(-0.75, 1.0, -1.1, 0.75, 1.5, 0.9, 'ny');
  } else if (kind === 'bike') {
    g.color('#3A3A3A', FINISH).box(-0.08, 0.25, -0.85, 0.08, 0.95, 0.85, 'ny');
  } else {
    // A resident: legs, a body in their colour, a head.
    g.color('#5A5450', FINISH).box(-0.17, 0, -0.12, 0.17, 0.85, 0.12, 'ny');
    g.color('#FFFFFF', FINISH).box(-0.24, 0.85, -0.15, 0.24, 1.45, 0.15, 'ny');
    g.color('#E0B8A0', FINISH).box(-0.12, 1.45, -0.12, 0.12, 1.75, 0.12, 'ny');
  }
  return g;
}

/** How much larger the movers are drawn at the full town view (1 + this × the blend): they read as its traffic. */
export const MOVER_GROW = 1.4;

/** One mover this frame: where it is (its ground height too), which way it faces (rad about +y, 0 along +z) and its colour (sRGB). */
export interface Mover {
  kind: MoverKind;
  x: number;
  y: number;
  z: number;
  yaw: number;
  color: readonly [number, number, number];
}

/** The movers past the near drawing: an instanced draw per shape, written again each frame they're shown. */
export class TownMovers {
  /** Its draws, made the first movers it is given (as `TownBuildings`' material: nothing of it before the town view). */
  private draws: Record<'car' | 'bike' | 'person', Instances> | null = null;
  private readonly meshes: pc.Mesh[] = [];
  private readonly m = new Float32Array(16);
  private readonly c = new Float32Array(4);

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
  ) {}

  private made(): Record<'car' | 'bike' | 'person', Instances> {
    if (this.draws) return this.draws;
    const { app, parent } = this;
    const lib = MaterialLibrary.for(app);
    const mat = lib.instanceColored(lib.plainFinish());
    const make = (kind: 'car' | 'bike' | 'person') => {
      const mesh = meshFrom(app.graphicsDevice, moverMesh(kind).meshData({ colors: true }))!;
      mesh.incRefCount();
      this.meshes.push(mesh);
      const d = new Instances(app.graphicsDevice, parent, mesh, mat, { name: `town.${kind}s`, streams: [INSTANCE_COLOR], castShadows: false, receiveShadows: true, capacity: 64 });
      lib.excludeFromLights([], [d.meshInstance]);
      d.enabled = false;
      return d;
    };
    return (this.draws = { car: make('car'), bike: make('bike'), person: make('person') });
  }

  /** This frame's movers (none: hidden), drawn `grow` times as large. */
  set(movers: readonly Mover[], grow: number): void {
    const draws = this.made();
    const counts = { car: 0, bike: 0, person: 0 };
    for (const mv of movers) counts[mv.kind === 'taxi' ? 'car' : mv.kind]++;
    for (const k of ['car', 'bike', 'person'] as const) draws[k].setCount(counts[k]);
    const at = { car: 0, bike: 0, person: 0 };
    const { m, c } = this;
    for (const mv of movers) {
      const k = mv.kind === 'taxi' ? 'car' : mv.kind;
      const cs = Math.cos(mv.yaw) * grow;
      const sn = Math.sin(mv.yaw) * grow;
      // A turn about +y (yaw 0: facing +z), scaled.
      m.fill(0);
      m[0] = cs;
      m[2] = -sn;
      m[5] = grow;
      m[8] = sn;
      m[10] = cs;
      m[12] = mv.x;
      m[13] = mv.y;
      m[14] = mv.z;
      m[15] = 1;
      const i = at[k]++;
      draws[k].setMatrix(i, m);
      c[0] = mv.color[0];
      c[1] = mv.color[1];
      c[2] = mv.color[2];
      c[3] = 1;
      draws[k].set(INSTANCE_COLOR, i, c);
    }
    for (const k of ['car', 'bike', 'person'] as const) {
      const d = draws[k];
      d.update();
      d.enabled = d.count > 0;
    }
  }

  hide(): void {
    if (this.draws) for (const d of Object.values(this.draws)) d.enabled = false;
  }

  stats(): { draws: number; copies: number } {
    let draws = 0;
    let copies = 0;
    for (const d of Object.values(this.draws ?? {})) {
      if (!d.enabled || !d.count) continue;
      draws++;
      copies += d.count;
    }
    return { draws, copies };
  }

  destroy(): void {
    for (const d of Object.values(this.draws ?? {})) d.destroy();
    for (const mesh of this.meshes) {
      mesh.decRefCount();
      if (mesh.refCount === 0) mesh.destroy();
    }
  }
}
