/**
 * Build mode's previews and overlays on PlayCanvas (`setEdgePreview`, `setPaintPreview`,
 * `setFloorPreview`, `setRoomOverlay` and `setBuildGrid`), all on the storey layer (`storeyLayer`: they work in lot rows, and the layer lifts them to the storey
 * in view):
 *
 * - The wall tools' edges: where they can go, walls, doors and windows as the house would build them
 *   (`HouseBuilder.build` with `preview`, standing while walls are down, the stretch of wall they go
 *   into hidden by the cutaway's hole, `wallHole.ts`) and fences and gates in their style; removals, and whatever
 *   can't go, as tinted films (unit boxes scaled to each edge, one instanced draw per kind).
 * - The Paint and Floor tools: the faces or tiles they would cover in the covering's real look
 *   (`HouseBuilder.coverPreview`, `floorPreview`, laid on the house's `HouseSurfaces`), or, for the
 *   automatic look, a white film over each.
 * - The room-score overlay: a film over each tile in its room's colour.
 * - The tile grid: white lines on a transparent texture repeated once per tile, with fainter half-
 *   and quarter-tile lines that fade out with distance (its mipmaps).
 *
 * The films and the grid are unlit `StandardMaterial`s: their colour, taken from gamma to linear
 * (`toLinear`, `hexInto`), is a per-draw `material_emissive`, blended at a fixed alpha, in the fog,
 * writing no depth (as none of the game's blended draws do; the grid is also pulled towards the
 * camera by a depth bias). Updates rebuild only what changed, and nothing is
 * allocated while a preview stays as it is.
 */

import * as pc from './engine';
import { buildFences } from './fences';
import type { HouseBuilder } from './house';
import { Instances } from './instancing';
import { GAME_FRONT_FACE, MaterialLibrary } from './materials';
import { setWallHole, wallHoleFor, type WallHole } from './wallHole';
import type { Content } from '../../content/content';
import type { DiagonalWall, Opening, WallEdge, WorldStructure } from '../../core/protocol';
import type { EdgePreview, PaintPreviewFace, RoomOverlayTile, ViewRect, WallMode } from '../types';
import { HALF_WALL_HEIGHT, WALL_HEIGHT, type HouseSurfaces } from '../shared/house';
import { toLinear } from '../shared/math';

/** Fences and gates previewed as films: as tall as this (m). */
const FENCE_PREVIEW_HEIGHT = 1.0;
/** Copies each edge film draw has room for at first (it grows). */
const PREVIEW_CAPACITY = 128;

type FilmKind = 'wall' | 'door' | 'window' | 'open';
const FILM_KINDS: readonly FilmKind[] = ['wall', 'door', 'window', 'open'];
/** Each edge film's colour and alpha (removals are always red); what can't go is tinted `INVALID`. */
const FILM_LOOK: Record<FilmKind, { color: string; alpha: number }> = {
  wall: { color: '#9FC4FF', alpha: 0.45 },
  door: { color: '#B07A4A', alpha: 0.6 },
  window: { color: '#7FE0F2', alpha: 0.55 },
  open: { color: '#E5484D', alpha: 0.5 },
};
const INVALID = '#E5484D';

/** A box of size 1: its corners, normals and triangles, face by face. */
const BOX_POSITIONS = [
  1, -1, 1, -1, -1, 1, -1, 1, 1, 1, 1, 1, 1, 1, -1, -1, 1, -1, -1, -1, -1, 1, -1, -1, 1, 1, -1, 1, -1, -1, 1, -1, 1, 1, 1, 1, -1, 1, 1, -1, -1, 1, -1, -1, -1, -1, 1, -1, -1, 1, 1, -1, 1, -1, 1, 1, -1, 1, 1, 1, 1, -1, 1, 1, -1, -1, -1, -1, -1, -1, -1,
  1,
].map((v) => v / 2);
const BOX_NORMALS = [0, 0, 1, 0, 0, -1, 1, 0, 0, -1, 0, 0, 0, 1, 0, 0, -1, 0].flatMap((_, i, n) => (i % 3 === 0 ? [0, 1, 2, 3].flatMap(() => [n[i], n[i + 1], n[i + 2]]) : []));
const BOX_INDICES = [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23];
/** A 1 × 1 ground quad: corners (x, z), uvs, triangles. */
const GROUND_CORNERS = [-0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5];
const GROUND_UVS = [0, 1, 1, 1, 0, 0, 1, 0];
const GROUND_INDICES = [3, 1, 0, 2, 3, 0];

/** What the previews need of the renderer (as of the call). */
export interface BuildPreviewHost {
  readonly content: Content;
  /** The world last asked for (the walls, openings and fences a preview goes into). */
  world(): WorldStructure | null;
  /** The lot being shown (null: everything). */
  view(): ViewRect | null;
  /** The storey in view, and the first lot row of storey `k`. */
  storey(): number;
  rowOf(k: number): number;
  wallMode(): WallMode;
}

/** Every storey's surfaces in one (their keys are lot rows: a storey's own). */
export function mergeSurfaces(list: readonly HouseSurfaces[]): HouseSurfaces {
  return { faces: new Map(list.flatMap((s) => [...s.faces])), floors: new Map(list.flatMap((s) => [...s.floors])) };
}

/**
 * A hex colour as a film emits it (into `out`, linear): the films' colours are gamma-space values,
 * taken to linear (`pow(c, 2.2)`) for the image processing that follows.
 */
function hexInto(hex: string, out: Float32Array): Float32Array {
  out[0] = toLinear(parseInt(hex.slice(1, 3), 16) / 255);
  out[1] = toLinear(parseInt(hex.slice(3, 5), 16) / 255);
  out[2] = toLinear(parseInt(hex.slice(5, 7), 16) / 255);
  return out;
}

/** An unlit blended film: a `StandardMaterial` with lighting off, its emissive colour per draw. */
function filmMaterial(name: string, alpha: number, cull: number): pc.StandardMaterial {
  const m = new pc.StandardMaterial();
  m.name = name;
  m.useLighting = false;
  m.useSkybox = false;
  m.diffuse.set(0, 0, 0);
  m.emissive.set(1, 1, 1);
  m.opacity = alpha;
  m.blendType = pc.BLEND_NORMAL;
  // (The game's blended draws don't write depth. A box's faces all show.)
  m.depthWrite = false;
  m.frontFace = GAME_FRONT_FACE;
  m.cull = cull;
  m.update();
  return m;
}

function staticMesh(device: pc.GraphicsDevice, positions: number[], normals: number[], indices: number[], uvs?: number[]): pc.Mesh {
  const mesh = new pc.Mesh(device);
  mesh.setPositions(positions);
  mesh.setNormals(normals);
  if (uvs) mesh.setUvs(0, uvs);
  mesh.setIndices(indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  return mesh;
}

/** The Paint or Floor tool's preview in the covering's real look: what it shows (`shown`), and the drawn part. */
interface LaidSlot {
  keys: string[];
  cover: number | null;
  shown: string;
  entity: pc.Entity | null;
}

/** Instanced films (a material and a draw), with a colour of their own. */
interface Film {
  draw: Instances;
  material: pc.StandardMaterial;
  color: Float32Array;
}

/** The room overlay's draw: a quad per tile, coloured per vertex (its arrays grow, kept). */
interface OverlayDraw {
  entity: pc.Entity;
  mesh: pc.Mesh;
  positions: Float32Array;
  normals: Float32Array;
  colors: Float32Array;
  indices: Uint32Array;
  capacity: number;
}

/** The grid's draw: a quad over the rectangle (its corners and uvs kept). */
interface GridDraw {
  entity: pc.Entity;
  mesh: pc.Mesh;
  texture: pc.Texture;
  material: pc.StandardMaterial;
  positions: Float32Array;
  uvs: Float32Array;
}

export class BuildPreviews {
  private readonly device: pc.GraphicsDevice;
  private readonly lib: MaterialLibrary;
  private readonly box: pc.Mesh;
  private readonly ground: pc.Mesh;
  private readonly m = new Float32Array(16);

  // The wall tools.
  private readonly films: Record<FilmKind, Film>;
  private readonly counts: Record<FilmKind, number> = { wall: 0, door: 0, window: 0, open: 0 };
  private valid = true;
  /** Walls, doors, windows, fences and gates shown as built (under one entity), and what they show. */
  private built: pc.Entity | null = null;
  private builtKey = '';

  // The Paint and Floor tools.
  private surfaces: HouseSurfaces | null = null;
  private readonly laid: Record<'faces' | 'floors', LaidSlot> = {
    faces: { keys: [], cover: null, shown: '', entity: null },
    floors: { keys: [], cover: null, shown: '', entity: null },
  };
  private readonly paint: Film;
  private readonly floor: Film;

  // The room overlay, and the tile grid.
  private overlay: OverlayDraw | null = null;
  private readonly grid: GridDraw;
  private gridRect = '';

  constructor(
    private readonly app: pc.AppBase,
    private readonly layer: pc.Entity,
    private readonly house: HouseBuilder,
    private readonly host: BuildPreviewHost,
  ) {
    this.device = app.graphicsDevice;
    this.lib = MaterialLibrary.for(app);
    this.box = staticMesh(this.device, BOX_POSITIONS, BOX_NORMALS, BOX_INDICES);
    const groundPositions = [0, 1, 2, 3].flatMap((i) => [GROUND_CORNERS[i * 2], 0, GROUND_CORNERS[i * 2 + 1]]);
    this.ground = staticMesh(this.device, groundPositions, [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0], GROUND_INDICES, GROUND_UVS);

    // Edge films: both faces of a box (seen through), each kind its own draw.
    const film = (name: string, mesh: pc.Mesh, alpha: number, cull: number, capacity: number, color: string): Film => {
      const material = filmMaterial(name, alpha, cull);
      const draw = new Instances(this.device, layer, mesh, material, { name, castShadows: false, receiveShadows: false, capacity, cull: false });
      const c = hexInto(color, new Float32Array(3));
      draw.meshInstance.setParameter('material_emissive', c);
      draw.enabled = false;
      return { draw, material, color: c };
    };
    this.films = {
      wall: film('preview-wall', this.box, FILM_LOOK.wall.alpha, pc.CULLFACE_NONE, PREVIEW_CAPACITY, FILM_LOOK.wall.color),
      door: film('preview-door', this.box, FILM_LOOK.door.alpha, pc.CULLFACE_NONE, PREVIEW_CAPACITY, FILM_LOOK.door.color),
      window: film('preview-window', this.box, FILM_LOOK.window.alpha, pc.CULLFACE_NONE, PREVIEW_CAPACITY, FILM_LOOK.window.color),
      open: film('preview-open', this.box, FILM_LOOK.open.alpha, pc.CULLFACE_NONE, PREVIEW_CAPACITY, FILM_LOOK.open.color),
    };
    // Paint tool: one film per face; Floor tool: one per tile; tinted with the covering.
    this.paint = film('preview-paint', this.box, 0.6, pc.CULLFACE_BACK, 64, '#FFFFFF');
    this.floor = film('preview-floor', this.ground, 0.6, pc.CULLFACE_BACK, 64, '#FFFFFF');
    this.grid = this.makeGrid();
  }

  /** The viewed house's surfaces (`HouseBuild.surfaces`, every storey's; null: no house), for the Paint and Floor tools. */
  setSurfaces(surfaces: HouseSurfaces | null): void {
    this.surfaces = surfaces;
    // Lay the shown previews on the new surfaces.
    for (const which of ['faces', 'floors'] as const) {
      const slot = this.laid[which];
      if (!slot.shown) continue;
      slot.shown = '';
      this.showLaid(which, slot.cover, slot.keys);
    }
  }

  setEdgePreview(edges: readonly EdgePreview[], valid: boolean): void {
    // Where they can go, walls, doors, windows, fences and gates show as built; removals (and what can't go) as films.
    const built = valid ? edges.filter((e) => e.kind !== 'open') : [];
    this.showBuilt(built);
    const counts = this.counts;
    for (const kind of FILM_KINDS) counts[kind] = 0;
    for (const e of edges) {
      if (built.length && e.kind !== 'open') continue;
      // Fences show as low walls, gates as low doors.
      const kind: FilmKind = e.kind === 'fence' ? 'wall' : e.kind === 'gate' ? 'door' : e.kind;
      const n = counts[kind]++;
      // Unit slab (1 x 1 x 1 centred box) scaled to the edge: walls full height, doors a
      // door-sized slab, windows a glassy pane between sill and head, removals a red sleeve
      // just around the existing wall.
      const h = e.kind === 'fence' || e.kind === 'gate' ? FENCE_PREVIEW_HEIGHT : e.kind === 'door' ? 2.1 : e.kind === 'window' ? 1.2 : e.kind === 'open' ? WALL_HEIGHT + 0.06 : e.form ? HALF_WALL_HEIGHT : WALL_HEIGHT;
      const y0 = e.kind === 'window' ? 0.9 : 0;
      const t = e.kind === 'open' ? 0.22 : e.kind === 'window' ? 0.2 : e.kind === 'fence' || e.kind === 'gate' ? 0.08 : 0.16;
      const m = this.m;
      m.fill(0);
      const diagonal = e.axis === 'dp' || e.axis === 'dn';
      if (diagonal) {
        // Diagonal across the tile: a √2-long slab turned ±45° about its centre (scale, then yaw).
        const c = Math.SQRT1_2;
        const sn = e.axis === 'dp' ? -c : c;
        m[0] = Math.SQRT2 * c;
        m[2] = -Math.SQRT2 * sn;
        m[8] = t * sn;
        m[10] = t * c;
      } else {
        m[0] = e.axis === 'h' ? 1 : t;
        m[10] = e.axis === 'h' ? t : 1;
      }
      m[5] = h;
      m[12] = diagonal || e.axis === 'h' ? e.x + 0.5 : e.x;
      m[13] = y0 + h / 2;
      m[14] = diagonal ? e.z + 0.5 : e.axis === 'h' ? e.z : e.z + 0.5;
      m[15] = 1;
      const draw = this.films[kind].draw;
      draw.reserve(n + 1);
      draw.setMatrix(n, m);
    }
    for (const kind of FILM_KINDS) {
      const draw = this.films[kind].draw;
      draw.setCount(counts[kind]);
      draw.update(false);
      // Empty previews are disabled so they cost no draw call.
      draw.enabled = counts[kind] > 0;
    }
    if (valid !== this.valid) {
      this.valid = valid;
      for (const kind of ['wall', 'door', 'window'] as const) hexInto(valid ? FILM_LOOK[kind].color : INVALID, this.films[kind].color);
    }
  }

  setPaintPreview(faces: readonly PaintPreviewFace[], cover: number | null): void {
    const real = this.showLaid('faces', faces.length ? cover : null, faces.map((f) => `${f.axis}:${f.x}:${f.z}:${f.side}`));
    // The automatic look (0) depends on the room, so a white film stands in for it.
    const color = real || cover === null ? null : ((cover ? this.host.content.wallCoverings[cover - 1]?.color : undefined) ?? '#FFFFFF');
    this.paintFilm(faces, color);
  }

  setFloorPreview(tiles: readonly { x: number; z: number }[], floor: number | null): void {
    const real = this.showLaid('floors', tiles.length ? floor : null, tiles.map((t) => `${t.x}:${t.z}`));
    const color = real || floor === null ? null : ((floor ? this.host.content.floorCoverings[floor - 1]?.color : undefined) ?? '#FFFFFF');
    this.floorFilm(tiles, color);
  }

  setRoomOverlay(tiles: readonly RoomOverlayTile[]): void {
    const n = tiles.length;
    if (!this.overlay && n === 0) return;
    const o = (this.overlay ??= this.makeOverlay());
    if (n > o.capacity) {
      const cap = Math.max(n, 64) * 2;
      o.positions = new Float32Array(cap * 12);
      o.normals = new Float32Array(cap * 12);
      o.colors = new Float32Array(cap * 16);
      o.indices = new Uint32Array(cap * 6);
      for (let i = 0; i < cap; i++) {
        for (let k = 0; k < 4; k++) o.normals[i * 12 + k * 3 + 1] = 1;
        for (let k = 0; k < 6; k++) o.indices[i * 6 + k] = i * 4 + GROUND_INDICES[k];
      }
      o.capacity = cap;
    }
    // One film per tile, tinted per tile; just under the floor tool's film.
    for (let i = 0; i < n; i++) {
      const t = tiles[i];
      for (let k = 0; k < 4; k++) {
        const p = i * 12 + k * 3;
        o.positions[p] = t.x + 0.5 + GROUND_CORNERS[k * 2];
        o.positions[p + 1] = 0.025;
        o.positions[p + 2] = t.z + 0.5 + GROUND_CORNERS[k * 2 + 1];
        const c = i * 16 + k * 4;
        o.colors[c] = toLinear(t.rgb[0]);
        o.colors[c + 1] = toLinear(t.rgb[1]);
        o.colors[c + 2] = toLinear(t.rgb[2]);
        o.colors[c + 3] = 1;
      }
    }
    if (n > 0) {
      o.mesh.setPositions(o.positions, 3, n * 4);
      o.mesh.setNormals(o.normals, 3, n * 4);
      o.mesh.setColors(o.colors, 4, n * 4);
      o.mesh.setIndices(o.indices, n * 6);
      o.mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    }
    o.entity.enabled = n > 0;
  }

  setBuildGrid(rect: ViewRect | null): void {
    const g = this.grid;
    if (rect) {
      const key = `${rect.x},${rect.z},${rect.w},${rect.d}`;
      if (key !== this.gridRect) {
        this.gridRect = key;
        // A 1 m ground scaled to the rectangle, its texture repeated once per tile.
        for (let k = 0; k < 4; k++) {
          g.positions[k * 3] = rect.x + rect.w * (GROUND_CORNERS[k * 2] + 0.5);
          g.positions[k * 3 + 1] = 0.03;
          g.positions[k * 3 + 2] = rect.z + rect.d * (GROUND_CORNERS[k * 2 + 1] + 0.5);
          g.uvs[k * 2] = GROUND_UVS[k * 2] * rect.w;
          g.uvs[k * 2 + 1] = GROUND_UVS[k * 2 + 1] * rect.d;
        }
        g.mesh.setPositions(g.positions);
        g.mesh.setUvs(0, g.uvs);
        g.mesh.update(pc.PRIMITIVE_TRIANGLES, true);
      }
    }
    g.entity.enabled = rect !== null;
  }

  /** Everything hidden (the lot is going: `Renderer.clear`). */
  clear(): void {
    this.setEdgePreview([], true);
    this.setPaintPreview([], null);
    this.setFloorPreview([], null);
    this.setRoomOverlay([]);
    this.setBuildGrid(null);
    this.surfaces = null;
  }

  destroy(): void {
    this.dropBuilt();
    for (const slot of Object.values(this.laid)) slot.entity?.destroy();
    for (const f of [...Object.values(this.films), this.paint, this.floor]) {
      f.draw.destroy();
      f.material.destroy();
    }
    this.overlay?.entity.destroy();
    this.grid.entity.destroy();
    this.grid.material.destroy();
    this.grid.texture.destroy();
    this.box.destroy();
    this.ground.destroy();
  }

  /**
   * Fences and gates in their style; walls, doors and windows as the house would build them
   * (standing while walls are down), the stretch of wall they go into hidden meanwhile.
   */
  private showBuilt(edges: readonly EdgePreview[]): void {
    const key = edges.length ? JSON.stringify([this.host.storey(), this.host.wallMode(), edges]) : '';
    if (key === this.builtKey) return;
    this.builtKey = key;
    const world = this.host.world();
    if (!edges.length || !world) {
      this.dropBuilt();
      setWallHole(this.lib.wallCut, null);
      return;
    }
    const storey = this.host.storey();
    const next = new pc.Entity('preview-built');
    let hole: WallHole | null = null;
    const fences = edges.flatMap((e) => ((e.kind === 'fence' || e.kind === 'gate') && (e.axis === 'h' || e.axis === 'v') ? [{ axis: e.axis, x: e.x, z: e.z, kind: e.kind, style: e.style }] : []));
    buildFences(this.app, next, fences, this.host.content.fenceStyles);
    // A lot with just these walls (in the rooms as they are), the walls they go into keeping their looks.
    const W = world.width;
    const walls: WallEdge[] = [];
    const openings: Opening[] = [];
    const diagonals: DiagonalWall[] = [];
    const standing: EdgePreview[] = [];
    const previewed = new Set(edges.map((e) => `${e.axis}:${e.x}:${e.z}`));
    const opened = new Map((world.openings ?? []).map((o) => [`${o.axis}:${o.x}:${o.z}`, o]));
    const walled = new Map((world.walls ?? []).map((w) => [`${w.axis}:${w.x}:${w.z}`, w]));
    const slanted = new Map((world.diagonals ?? []).map((d) => [`${d.axis}:${d.x}:${d.z}`, d]));
    // With walls cut down, a door or window put into a wall raises the whole of it (the straight
    // run either side, as built) so it shows in the wall it goes into.
    const run: EdgePreview[] = [];
    if (this.host.wallMode() !== 'up') {
      for (const e of edges) {
        if (e.kind !== 'door' && e.kind !== 'window') continue;
        const at = `${e.axis}:${e.x}:${e.z}`;
        if (!walled.has(at) && !slanted.has(at)) continue;
        const [dx, dz] = e.axis === 'h' ? [1, 0] : e.axis === 'v' ? [0, 1] : e.axis === 'dp' ? [1, 1] : [1, -1];
        for (const step of [-1, 1]) {
          for (let k = step; ; k += step) {
            const [x, z] = [e.x + dx * k, e.z + dz * k];
            const along = `${e.axis}:${x}:${z}`;
            if (previewed.has(along)) continue;
            const had = slanted.get(along) ?? opened.get(along) ?? walled.get(along);
            if (!had) break;
            previewed.add(along);
            run.push({ axis: e.axis, x, z, kind: 'kind' in had ? had.kind : 'wall', style: 'style' in had ? had.style : undefined });
          }
        }
      }
    }
    for (const e of [...edges, ...run]) {
      if (e.kind !== 'wall' && e.kind !== 'door' && e.kind !== 'window') continue;
      const faces: [number, number] | undefined = e.cover ? [e.cover, e.cover] : undefined;
      if (e.axis === 'dp' || e.axis === 'dn') {
        const had = slanted.get(`${e.axis}:${e.x}:${e.z}`);
        const r = world.rooms[e.z * W + e.x] ?? 0;
        if (had) standing.push(e);
        diagonals.push({ axis: e.axis, x: e.x, z: e.z, kind: e.kind, rooms: had?.rooms ?? [r, r], faces: faces ?? had?.faces, form: had ? had.form : e.form, style: e.style });
      } else {
        const had = walled.get(`${e.axis}:${e.x}:${e.z}`);
        if (had) standing.push(e);
        walls.push({ axis: e.axis, x: e.x, z: e.z, faces: faces ?? had?.faces, form: e.kind === 'wall' ? (had ? had.form : e.form) : undefined });
        if (e.kind !== 'wall') openings.push({ axis: e.axis, x: e.x, z: e.z, kind: e.kind, style: e.style, sill: e.sill, head: e.head });
      }
    }
    if (walls.length || diagonals.length) {
      const v = this.host.view();
      const view = v ? { ...v, z: v.z + this.host.rowOf(storey) } : null;
      this.house.build({ ...world, walls, openings, diagonals }, view, { upper: storey !== 0, preview: true, standing: world }, next);
      hole = wallHoleFor(standing, storey, this.host.rowOf(storey));
    }
    // (A preview casts no shadow.)
    for (const r of next.findComponents('render') as pc.RenderComponent[]) r.castShadows = false;
    // The shown preview goes as the next one comes (no gap moving from tile to tile).
    this.dropBuilt();
    this.layer.addChild(next);
    this.built = next;
    setWallHole(this.lib.wallCut, hole);
  }

  private dropBuilt(): void {
    this.built?.destroy();
    this.built = null;
  }

  /**
   * Shows `keys` (wall faces or floor tiles in the house's surfaces) as covering `cover` lays them, in
   * its real material. False where it can't (no covering, the automatic look, no house surfaces).
   */
  private showLaid(which: 'faces' | 'floors', cover: number | null, keys: string[]): boolean {
    const slot = this.laid[which];
    slot.keys = keys;
    slot.cover = cover;
    const surfaces = this.surfaces?.[which];
    const shown = cover && surfaces ? `${cover}|${keys.join(',')}` : '';
    if (shown !== slot.shown) {
      slot.shown = shown;
      slot.entity?.destroy();
      slot.entity = null;
      if (cover && surfaces) {
        const parts = keys.flatMap((k) => surfaces.get(k) ?? []);
        slot.entity = which === 'faces' ? this.house.coverPreview(parts, cover, this.layer) : this.house.floorPreview(parts, cover, this.layer);
      }
    }
    return shown !== '';
  }

  /** The Paint tool's stand-in: a film of `color` over each face (null: none). */
  private paintFilm(faces: readonly PaintPreviewFace[], color: string | null): void {
    const n = color ? faces.length : 0;
    const draw = this.paint.draw;
    draw.setCount(n);
    // A thin film just off each face (walls are 0.14 m thick), as tall as the wall.
    const off = 0.078;
    const t = 0.012;
    const m = this.m;
    for (let i = 0; i < n; i++) {
      const f = faces[i];
      const h = f.half ? HALF_WALL_HEIGHT : WALL_HEIGHT;
      const s = f.side ? 1 : -1;
      m.fill(0);
      if (f.axis === 'h' || f.axis === 'v') {
        m[0] = f.axis === 'h' ? 1 : t;
        m[10] = f.axis === 'h' ? t : 1;
        m[12] = f.axis === 'h' ? f.x + 0.5 : f.x + s * off;
        m[14] = f.axis === 'h' ? f.z + s * off : f.z + 0.5;
      } else {
        // As the edge films: a √2-long film turned ±45°, offset towards half 1 (`s` > 0).
        const c = Math.SQRT1_2;
        const sn = f.axis === 'dp' ? -c : c;
        m[0] = Math.SQRT2 * c;
        m[2] = -Math.SQRT2 * sn;
        m[8] = t * sn;
        m[10] = t * c;
        m[12] = f.x + 0.5 + s * off * sn;
        m[14] = f.z + 0.5 + s * off * c;
      }
      m[5] = h;
      m[13] = h / 2;
      m[15] = 1;
      draw.setMatrix(i, m);
    }
    draw.update(false);
    if (color) hexInto(color, this.paint.color);
    draw.enabled = n > 0;
  }

  /** The Floor tool's stand-in: a film of `color` over each tile (null: none). */
  private floorFilm(tiles: readonly { x: number; z: number }[], color: string | null): void {
    const n = color ? tiles.length : 0;
    const draw = this.floor.draw;
    draw.setCount(n);
    // A film just above the floor (floors lie at 0.01 m), a hair inside each tile so tiles read.
    const m = this.m;
    m.fill(0);
    m[0] = 0.96;
    m[5] = 1;
    m[10] = 0.96;
    m[13] = 0.03;
    m[15] = 1;
    for (let i = 0; i < n; i++) {
      m[12] = tiles[i].x + 0.5;
      m[14] = tiles[i].z + 0.5;
      draw.setMatrix(i, m);
    }
    draw.update(false);
    if (color) hexInto(color, this.floor.color);
    draw.enabled = n > 0;
  }

  private makeOverlay(): OverlayDraw {
    const material = filmMaterial('room-overlay', 0.45, pc.CULLFACE_BACK);
    // (Its colours as the films', see `hexInto`.)
    material.emissiveVertexColor = true;
    material.update();
    const mesh = new pc.Mesh(this.device);
    mesh.clear(true, true);
    // (Its streams exist before its draw does: a mesh instance takes its shader's vertex colours from the mesh it's made with.)
    mesh.setPositions(new Float32Array(12));
    mesh.setNormals(new Float32Array(12));
    mesh.setColors(new Float32Array(16));
    mesh.setIndices(GROUND_INDICES);
    mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const entity = new pc.Entity('room-overlay');
    const mi = new pc.MeshInstance(mesh, material);
    mi.cull = false;
    // Sorted among the blended draws by the 1 m tile at the layer's origin. The Floor tool's film
    // sorts there too and goes first: a hair higher, this one is nearer a camera looking down, so drawn
    // after it.
    mi.setCustomAabb(new pc.BoundingBox(new pc.Vec3(0, 0.01, 0), new pc.Vec3(0.5, 0, 0.5)));
    entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false });
    this.layer.addChild(entity);
    return { entity, mesh, positions: new Float32Array(0), normals: new Float32Array(0), colors: new Float32Array(0), indices: new Uint32Array(0), capacity: 0 };
  }

  /**
   * The tile grid (a canvas texture of 128 px: white lines on transparent, the tile's edge
   * 3 px wide at full strength, the half-tile line 2 px at 0.55, the quarters 2 px at 0.3), repeated
   * once per tile with mipmaps, 4× anisotropic; drawn at 0.45 in a pale blue-white.
   */
  private makeGrid(): GridDraw {
    const size = 128;
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    const ctx = canvas.getContext('2d', { willReadFrequently: true })!;
    ctx.clearRect(0, 0, size, size);
    ctx.fillStyle = 'rgba(255,255,255,0.3)';
    for (const p of [32, 96]) {
      ctx.fillRect(0, p, size, 2);
      ctx.fillRect(p, 0, 2, size);
    }
    ctx.fillStyle = 'rgba(255,255,255,0.55)';
    ctx.fillRect(0, 64, size, 2);
    ctx.fillRect(64, 0, 2, size);
    ctx.fillStyle = 'rgba(255,255,255,1)';
    ctx.fillRect(0, 0, size, 3);
    ctx.fillRect(0, 0, 3, size);
    const pixels = ctx.getImageData(0, 0, size, size).data;
    const texture = new pc.Texture(this.device, {
      name: 'buildGrid',
      width: size,
      height: size,
      format: pc.PIXELFORMAT_RGBA8,
      mipmaps: true,
      anisotropy: 4,
      minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR,
      magFilter: pc.FILTER_LINEAR,
      addressU: pc.ADDRESS_REPEAT,
      addressV: pc.ADDRESS_REPEAT,
    });
    // Rows bottom up: the canvas's top row is at v = 1.
    const data = texture.lock() as Uint8Array;
    const row = size * 4;
    for (let y = 0; y < size; y++) data.set(pixels.subarray((size - 1 - y) * row, (size - y) * row), y * row);
    texture.unlock();

    const material = new pc.StandardMaterial();
    material.name = 'buildGrid';
    material.useLighting = false;
    material.useSkybox = false;
    material.diffuse.set(0, 0, 0);
    material.emissive.set(1, 1, 1);
    material.emissiveMap = texture;
    material.opacityMap = texture;
    material.opacityMapChannel = 'a';
    material.opacity = 0.45;
    material.blendType = pc.BLEND_NORMAL;
    material.depthWrite = false;
    // A polygon offset factor of -4: drawn over the floors and the lawn it lies on.
    material.slopeDepthBias = -4;
    material.frontFace = GAME_FRONT_FACE;
    material.update();

    const positions = new Float32Array(12);
    const uvs = new Float32Array(8);
    const mesh = new pc.Mesh(this.device);
    mesh.clear(true, false);
    mesh.setPositions(positions);
    mesh.setNormals([0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
    mesh.setUvs(0, uvs);
    mesh.setIndices(GROUND_INDICES);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const entity = new pc.Entity('buildGrid');
    const mi = new pc.MeshInstance(mesh, material);
    // (Its colour map is decoded where it's read: an 8-bit texture without sRGB is taken as gamma, and so is its colour.)
    mi.setParameter('material_emissive', new Float32Array([toLinear(0.9), toLinear(0.95), 1]));
    entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false });
    entity.enabled = false;
    this.layer.addChild(entity);
    return { entity, mesh, texture, material, positions, uvs };
  }
}
