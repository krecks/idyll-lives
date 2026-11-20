/**
 * The built world as dumped for its gate (`cargo check-glb --built`): a fixed set of samples made by the game's own
 * generators (`web/src/render/shared`: `HousePlanner.build`, `silhouettes`, `farHouse` and `farLook`, the street,
 * `dressBuilding`, the town view's unit meshes, `writeFences`, `paintCar`), each as the meshes one frame draws of it.
 * No browser and no simulation: the worlds they are built from are made here, as the menus' preview makes its own
 * (`game/townPreview.ts`), from house plans written below and from the shipped `houses.json` and `housegen.json`.
 *
 * - **Houses**: each plan as the viewed lot's house (`house.<name>`: every storey, its doors shut, and the lot's
 *   garage and porch), as a neighbour's silhouette (`neighbour.<name>`), as the street's far house (`far.<name>`)
 *   and as the town view's box (`town.<name>`). The plans cover the roofs (gable, hip, flat; over a plain box, an L,
 *   a U and a ring), one and two storeys, a porch, a garage, diagonal walls, every wall covering, a floor covering
 *   of each finish, and every door and window style.
 * - **The lots' buildings closed**: a building of each use dressed (`building.<use>`, as a lot with that look) and as
 *   the town view's box (`buildingFar.<use>`).
 * - **A street** (`street.block`): two streets crossing, a lane with a bend to a dead end, a country road over a
 *   bridge, lots with houses, each fence style round a garden, parked cars.
 * - **Fences** (`fence.<style>`) and **cars** (`car.<body>`) each on their own.
 *
 * A sample's meshes are in the space PlayCanvas draws them in (metres, y up, the game's z mirrored: the world
 * hangs under a root mirrored in z), so a triangle's winding and its normals are glTF's.
 */

import { readFileSync } from 'node:fs';
import type { DoorStyleDef, FenceStyleDef, FloorCoveringDef, ObjectDef, RoofColorDef, RoofStyleDef, WallCoveringDef, WindowStyleDef } from '../../web/src/content/content';
import type { BuildingUse, DiagonalWall, LotLook, FenceEdge, ObjectPlacement, Opening, PlotInfo, RoadPlan, WallEdge, WorldStructure } from '../../web/src/core/protocol';
import type { HouseTemplate } from '../../web/src/game/town';
import type { Geo } from '../../web/src/render/shared/geometry';
import type { HousePart, HouseScheme } from '../../web/src/render/shared/house';
import type { TownBuilding } from '../../web/src/render/shared/townBuildings';
import type { DumpMesh } from './glb';

// The renderer's modules read the page's address (and its frames' timing) as they load.
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: { addEventListener: () => {}, removeEventListener: () => {} } });
const { Geo: GeoClass } = await import('../../web/src/render/shared/geometry');
const { HousePlanner, houseScheme, HOUSE_MATERIALS, WALL_HEIGHT, WALL_STUB } = await import('../../web/src/render/shared/house');
const { planStreet, TownStreet, Painter, NO_FOCUS } = await import('../../web/src/render/shared/street');
const { lotBuilding, lotLookBuilding, unitMeshes } = await import('../../web/src/render/shared/townBuildings');
const { dressBuilding } = await import('../../web/src/render/shared/buildingDress');
const { writeFences } = await import('../../web/src/render/shared/fences');
const { CARS, CAR_PAINT, paintCar } = await import('../../web/src/render/shared/cars');
const { generateHouse, objectInfos } = await import('../../web/src/game/housegen');
const { deriveJunctions } = await import('../../web/tests/fixtures/junctions');

/** What a sample is: the viewed house, its neighbour's silhouette, the street's far house, the town view's box, and so on. */
export type SampleKind = 'house' | 'neighbour' | 'far' | 'town' | 'building' | 'buildingFar' | 'street' | 'fence' | 'car';

/** One sample: its meshes and what the checks need to know of it besides. */
export interface Sample {
  /** `<kind>.<name>`: its file's name, and its key in the gate's baseline. */
  key: string;
  kind: SampleKind;
  /** A far version: the sample it is the far version of (the viewed house, the dressed building). */
  of?: string;
  /** How it was asked for, for the report (a plan's roof and coverings, a building's use). */
  about: string;
  /** Storeys of a house or a building. */
  storeys?: number;
  meshes: DumpMesh[];
}

/** The materials that are no manifest entry: what the street and the town view draw in their own. */
export const BUILT_MATERIALS = {
  /** The plain finish: the vertex colour, its roughness and metalness in the vertex alpha (`street.ts` `encodeFinish`). */
  plain: 'built.plain',
  /** Lamp glass: unlit, glowing at night. */
  glow: 'built.glow',
  /** Pools of lamplight on the ground, added. */
  pools: 'built.lightPool',
  /** Soft contact shadows, blended by vertex alpha. */
  shadow: 'built.shadow',
  /** Road markings, drawn over the road. */
  paint: 'built.roadPaint',
} as const;

const root = new URL('../../web/public/content/', import.meta.url);
const json = <T>(name: string): T => JSON.parse(readFileSync(new URL(name, root), 'utf8')) as T;
interface ContentFile {
  objects?: ObjectDef[];
  include?: string[];
  wallCoverings: WallCoveringDef[];
  floorCoverings: FloorCoveringDef[];
  doorStyles: DoorStyleDef[];
  windowStyles: WindowStyleDef[];
  roofStyles: RoofStyleDef[];
  roofColors: RoofColorDef[];
  fenceStyles: FenceStyleDef[];
  roomRules?: { lampTiles?: number; windowTiles?: number };
}
const base = json<ContentFile>('base.json');
const objects = new Map<string, ObjectDef>();
for (const f of [base, ...(base.include ?? []).map((name) => json<ContentFile>(name))]) for (const o of f.objects ?? []) if (!objects.has(o.id)) objects.set(o.id, o);
const shipped = json<{ plot: { width: number; depth: number }; lotSizes: Record<string, { width: number; depth: number }>; houses: HouseTemplate[] }>('houses.json');

// ---- meshes ---------------------------------------------------------------------------------------

/** Where a part's geometry goes as it is dumped: `dy` m up and `dz` m along z (a storey on its rows), then turned `turn` about y round `pivot` (a door leaf shut). */
interface Placed {
  dy?: number;
  dz?: number;
  turn?: number;
  pivot?: readonly [number, number];
}

/** A geometry as a dumped mesh: its vertices placed, then mirrored in z as the world root mirrors them. */
function meshOf(name: string, material: string, geo: Geo, extras: DumpMesh['extras'] = {}, cut = false, at: Placed = {}): DumpMesh {
  const n = geo.vertexCount;
  const positions = new Float32Array(n * 3);
  const normals = new Float32Array(n * 3);
  const [c, s] = [Math.cos(at.turn ?? 0), Math.sin(at.turn ?? 0)];
  const [px, pz] = at.pivot ?? [0, 0];
  const { pos, nor } = geo;
  for (let i = 0; i < n * 3; i += 3) {
    // (A turn about y as PlayCanvas makes one on an entity under the mirrored root: x' = x cos + z sin, z' = z cos − x sin.)
    const [x, z] = [pos[i] - px, pos[i + 2] - pz];
    positions[i] = px + x * c + z * s;
    positions[i + 1] = pos[i + 1] + (at.dy ?? 0);
    positions[i + 2] = -(pz + z * c - x * s + (at.dz ?? 0));
    normals[i] = nor[i] * c + nor[i + 2] * s;
    normals[i + 1] = nor[i + 1];
    normals[i + 2] = -(nor[i + 2] * c - nor[i] * s);
  }
  return { name, material, positions, normals, uvs: new Float32Array(geo.uv), colors: geo.col.slice(), ...(cut ? { cut: geo.cut.slice() } : {}), indices: geo.idx.slice(), extras };
}

/** What a house part is drawn with (`HousePart.surface`), as a material's name. */
function materialOf(part: HousePart): string {
  const s = part.surface;
  if (s.kind === 'surface') return s.key;
  if (s.kind === 'glass') return HOUSE_MATERIALS.clearGlass;
  if (s.kind === 'neighbourGlass') return HOUSE_MATERIALS.glass;
  return BUILT_MATERIALS.shadow;
}

/** A house part as a dumped mesh, with what the checks read of it: whether it is a roof (hidden with the walls down), only in the shadow map, a pattern over its texture. */
function partMesh(part: HousePart, extras: DumpMesh['extras'] = {}, at: Placed = {}): DumpMesh {
  const s = part.surface;
  return meshOf(
    part.name,
    materialOf(part),
    part.geo,
    { ...extras, ...(part.roof ? { roof: true } : {}), ...(part.shadowOnly ? { shadowOnly: true } : {}), ...(part.cast ? { cast: true } : {}), ...(s.kind === 'surface' && s.pattern ? { pattern: s.pattern } : {}) },
    part.cut,
    at,
  );
}

/** Meshes of the same name, material and extras as one, as the street draws its chunks' shares of a layer (`pc/street.ts`). */
function merged(meshes: DumpMesh[]): DumpMesh[] {
  const by = new Map<string, DumpMesh[]>();
  for (const m of meshes) {
    if (!m.indices.length) continue;
    const key = JSON.stringify([m.name, m.material, m.extras, !!m.cut]);
    by.set(key, [...(by.get(key) ?? []), m]);
  }
  return [...by.values()].map((list) => {
    if (list.length === 1) return list[0];
    const total = (of: (m: DumpMesh) => number) => list.reduce((n, m) => n + of(m), 0);
    const out: DumpMesh = {
      ...list[0],
      positions: new Float32Array(total((m) => m.positions.length)),
      normals: new Float32Array(total((m) => m.normals.length)),
      uvs: new Float32Array(total((m) => m.uvs.length)),
      colors: new Uint8Array(total((m) => m.colors.length)),
      ...(list[0].cut ? { cut: new Uint8Array(total((m) => m.cut!.length)) } : {}),
      indices: new Uint32Array(total((m) => m.indices.length)),
    };
    let [v, i] = [0, 0];
    for (const m of list) {
      out.positions.set(m.positions, v * 3);
      out.normals.set(m.normals, v * 3);
      out.uvs.set(m.uvs, v * 2);
      out.colors.set(m.colors, v * 4);
      out.cut?.set(m.cut!, v * 4);
      for (let k = 0; k < m.indices.length; k++) out.indices[i + k] = m.indices[k] + v;
      v += m.positions.length / 3;
      i += m.indices.length;
    }
    return out;
  });
}

// ---- house plans ------------------------------------------------------------------------------------

type Seg = [number, number, number, number];
/** A door or a window in a wall: its edge (`x`: along x on grid line `z`; `z`: along z on grid line `x`), as `houses.json` has them. */
interface Gap {
  x: number;
  z: number;
  axis: 'x' | 'z';
}
/** A storey of a plan: walls as segments along the grid, doors and windows in them, and the things that say what its rooms are for. */
interface Floor {
  walls: Seg[];
  doors: Gap[];
  windows?: Gap[];
  objects?: { def: string; x: number; z: number; rot: number }[];
}
/** A house plan on its lot (tiles, the street along the lot's -z side, as `houses.json` has its houses). */
interface Plan extends Floor {
  name: string;
  about: string;
  lot: { width: number; depth: number };
  upper?: Floor;
  /** Diagonal walls across tiles (a wall, a door or a window), each with the half of its tile that is indoors. */
  diagonals?: { axis: 'dp' | 'dn'; x: number; z: number; kind: DiagonalWall['kind'] }[];
  /** The look its lot's place gives it (`houseScheme`): the lot is put where it is this. */
  look: (s: HouseScheme) => boolean;
  /** The roof the player chose: a roof style's and a roof colour's id. */
  roof?: [string, string];
  /** Every wall covering on its inside faces in turn, a floor covering a room (every third: each finish comes up), every door and window style in turn. */
  each?: boolean;
  /** The wall covering (its id) the player painted its outside faces with. */
  painted?: string;
  /** A porch over its front door (the first door, in the wall along the street), a garage beside it. */
  porch?: boolean;
  garage?: boolean;
}

const box = (x0: number, z0: number, x1: number, z1: number): Seg[] => [[x0, z0, x1, z0], [x0, z1, x1, z1], [x0, z0, x0, z1], [x1, z0, x1, z1]];
const along = (axis: 'x' | 'z', at: number, ...places: number[]): Gap[] => places.map((p) => (axis === 'x' ? { x: p, z: at, axis } : { x: at, z: p, axis }));
const thing = (def: string, x: number, z: number, rot = 0) => ({ def, x, z, rot });

/** The generator's house for a lot size (`housegen.json`, as a new town's lots get theirs): the first of its seeds (as `tools/housegen/sample.ts` counts them) whose house has as many storeys as asked. */
function generated(name: string, about: string, size: string, storeys: 1 | 2, look: Plan['look']): Plan {
  const cfg = json<Parameters<typeof generateHouse>[0]>('housegen.json');
  const info = objectInfos({ object: (id) => objects.get(id), radius: (d) => Math.max(0.02, Math.min(0.45, d?.radius ?? 0.12)) });
  for (let seed = 7919; seed < 7919 * 200; seed += 7919) {
    const h = generateHouse(cfg, info, shipped.lotSizes[size], shipped.plot, size, seed, base.roomRules?.lampTiles, base.roomRules?.windowTiles, undefined, 'roomy');
    if (!h || !!h.upper !== (storeys === 2)) continue;
    const floor = (f: Floor): Floor => ({ walls: f.walls, doors: f.doors, windows: f.windows, objects: f.objects });
    return { name, about: `${about} (seed ${seed}: ${h.name})`, lot: h.lot ?? shipped.lotSizes[size], ...floor(h as Floor), ...(h.upper ? { upper: floor(h.upper as Floor) } : {}), look, porch: true };
  }
  throw new Error(`no house of ${storeys} storeys generated for a ${size} lot`);
}

/** The plans, in the order they are dumped. */
function plans(): Plan[] {
  const maple = shipped.houses.find((h) => h.id === 'maple')!;
  return [
    {
      name: 'cottage',
      about: 'houses.json\'s Maple Cottage: one storey, a plain box, siding under a gable roof of its own look, a porch',
      lot: shipped.plot,
      walls: maple.walls as Seg[],
      doors: maple.doors as Gap[],
      windows: maple.windows as Gap[],
      objects: maple.objects,
      look: (s) => !s.brick && !s.hip && s.shutters,
      porch: true,
    },
    {
      name: 'brick',
      about: 'one storey, a plain box in brick under a hip roof of its own look, a garage beside it',
      lot: { width: 22, depth: 18 },
      walls: [...box(3, 4, 12, 11), [8, 4, 8, 11], [3, 8, 8, 8]],
      doors: [...along('x', 4, 5), ...along('z', 8, 5, 9), ...along('x', 8, 6)],
      windows: [...along('x', 4, 3, 9, 10), ...along('x', 11, 4, 6, 9, 10), ...along('z', 3, 5, 9), ...along('z', 12, 5, 6, 9)],
      objects: [thing('sofa', 4, 9), thing('fridge', 9, 5), thing('bed', 4, 5), thing('toilet', 10, 9)],
      look: (s) => s.brick && s.hip && s.shutters,
      garage: true,
    },
    {
      name: 'painted',
      about: 'one storey, a plain box whose outside the player painted (brick.red) under the roof they chose (flat, navy)',
      lot: { width: 18, depth: 16 },
      walls: [...box(3, 4, 11, 10), [7, 4, 7, 10]],
      doors: [...along('x', 4, 5), ...along('z', 7, 6)],
      windows: [...along('x', 4, 3, 8, 9), ...along('x', 10, 4, 5, 8, 9), ...along('z', 3, 5, 8), ...along('z', 11, 5, 8)],
      objects: [thing('sofa', 4, 5), thing('bed', 8, 5)],
      look: (s) => !s.brick && !s.hip,
      roof: ['flat', 'navy'],
      painted: 'brick.red',
      porch: true,
    },
    {
      name: 'finishes',
      about: 'one storey, an L with diagonal walls: every wall covering inside, a floor of each finish, every door and window style; the roof chosen (steep gable, terracotta)',
      lot: { width: 26, depth: 22 },
      walls: [
        ...([[3, 3, 14, 3], [3, 3, 3, 15], [3, 15, 9, 15], [9, 15, 9, 9], [9, 9, 14, 9], [15, 4, 15, 8]] as Seg[]),
        [3, 6, 9, 6], [3, 9, 9, 9], [3, 12, 9, 12], [6, 3, 6, 6], [9, 3, 9, 9], [12, 3, 12, 9], [6, 9, 6, 12],
      ],
      doors: [...along('x', 3, 4), ...along('x', 12, 4, 7), ...along('x', 9, 4, 7), ...along('x', 6, 4, 7), ...along('z', 9, 4, 7), ...along('z', 12, 5), ...along('x', 15, 5)],
      windows: [...along('x', 3, 7, 10, 13), ...along('z', 3, 4, 7, 10, 13), ...along('z', 15, 5, 7), ...along('x', 9, 10, 12), ...along('x', 15, 7), ...along('z', 9, 13)],
      // (Two corners of the wing cut off, one with a window in it; a door across a corner of a room.)
      diagonals: [
        { axis: 'dp', x: 14, z: 3, kind: 'window' },
        { axis: 'dn', x: 14, z: 8, kind: 'wall' },
        { axis: 'dp', x: 10, z: 6, kind: 'door' },
      ],
      objects: [thing('toilet', 4, 4), thing('fridge', 7, 4), thing('sofa', 10, 4), thing('bed', 13, 5), thing('bed', 4, 7), thing('sofa', 7, 7), thing('shower', 4, 10), thing('stove', 7, 10), thing('bed', 4, 13)],
      look: (s) => !s.brick && s.shutters,
      roof: ['steepGable', 'terracotta'],
      each: true,
    },
    {
      name: 'two-storey',
      about: 'two storeys, a plain box under the roof chosen (hip, slate)',
      lot: { width: 18, depth: 16 },
      walls: [...box(3, 3, 11, 10), [7, 3, 7, 10]],
      doors: [...along('x', 3, 5), ...along('z', 7, 6)],
      windows: [...along('x', 3, 4, 8, 9), ...along('x', 10, 4, 5, 8, 9), ...along('z', 3, 5, 8), ...along('z', 11, 5, 8)],
      objects: [thing('sofa', 4, 5), thing('fridge', 8, 5)],
      upper: {
        walls: [...box(3, 3, 11, 10), [7, 3, 7, 10]],
        doors: along('z', 7, 6),
        windows: [...along('x', 3, 4, 5, 8, 9), ...along('x', 10, 4, 5, 8, 9), ...along('z', 3, 5, 8), ...along('z', 11, 5, 8)],
        objects: [thing('bed', 4, 5), thing('toilet', 8, 5)],
      },
      look: (s) => !s.brick,
      roof: ['hip', 'slate'],
      porch: true,
    },
    {
      name: 'ring',
      about: 'two storeys, the upper on the middle of the lower: the roof below a ring round it, of its own look',
      lot: { width: 20, depth: 18 },
      walls: [...box(3, 3, 13, 12), [8, 3, 8, 12]],
      doors: [...along('x', 3, 5), ...along('z', 8, 7)],
      windows: [...along('x', 3, 4, 9, 11), ...along('x', 12, 4, 6, 9, 11), ...along('z', 3, 5, 9), ...along('z', 13, 5, 9)],
      objects: [thing('sofa', 4, 5), thing('fridge', 9, 5)],
      upper: { walls: box(5, 5, 11, 10), doors: [], windows: [...along('x', 5, 6, 9), ...along('x', 10, 6, 9), ...along('z', 5, 7), ...along('z', 11, 7)], objects: [thing('bed', 6, 6)] },
      look: (s) => !s.brick && s.hip,
    },
    {
      name: 'wing',
      about: 'two storeys, the upper along the back of the lower and in from its ends: the roof below a U round it; the roof chosen (low hip, moss)',
      lot: { width: 20, depth: 18 },
      walls: [...box(3, 3, 13, 12), [8, 3, 8, 12]],
      doors: [...along('x', 3, 5), ...along('z', 8, 7)],
      windows: [...along('x', 3, 4, 9, 11), ...along('x', 12, 4, 6, 9, 11), ...along('z', 3, 5, 9), ...along('z', 13, 5, 9)],
      objects: [thing('sofa', 4, 5), thing('fridge', 9, 5)],
      upper: { walls: box(5, 7, 11, 12), doors: [], windows: [...along('x', 7, 6, 9), ...along('x', 12, 6, 9), ...along('z', 5, 9), ...along('z', 11, 9)], objects: [thing('bed', 6, 8)] },
      look: (s) => s.brick,
      roof: ['lowHip', 'moss'],
    },
    {
      name: 'corner',
      about: 'two storeys, the upper on a corner of the lower: the roof below an L round it; the roof chosen (gable, charcoal)',
      lot: { width: 20, depth: 18 },
      walls: [...box(3, 3, 13, 12), [8, 3, 8, 12]],
      doors: [...along('x', 3, 5), ...along('z', 8, 7)],
      windows: [...along('x', 3, 4, 9, 11), ...along('x', 12, 4, 6, 9, 11), ...along('z', 3, 5, 9), ...along('z', 13, 5, 9)],
      objects: [thing('sofa', 4, 5), thing('fridge', 9, 5)],
      upper: { walls: box(3, 3, 8, 8), doors: [], windows: [...along('x', 3, 4, 6), ...along('x', 8, 4, 6), ...along('z', 3, 5), ...along('z', 8, 5)], objects: [thing('bed', 4, 4)] },
      look: (s) => !s.brick,
      roof: ['gable', 'charcoal'],
    },
    generated('generated-small', 'housegen.json\'s house of one storey for a small lot, of its own look', 'small', 1, (s) => !s.brick),
    generated('generated-tall', 'housegen.json\'s house of two storeys for a medium lot, of its own look', 'medium', 2, () => true),
  ];
}

// ---- a plan as a world ------------------------------------------------------------------------------

/** The street in front of a lot: its width, and the rows of a storey before the lot's (m). */
const STREET = 10;
/** The lot beside the house's, empty: the one in view when the house is a neighbour's. */
const NEXT = 12;

const coverings = base.wallCoverings;
const floorCoverings = base.floorCoverings;
const indexOf = (list: readonly { id: string }[], id: string): number => {
  const i = list.findIndex((e) => e.id === id);
  if (i < 0) throw new Error(`no ${id} in the content`);
  return i;
};

/** A plan as the world the renderer is given: its lot at (`ox`, `STREET`) with the street along its -z side and an empty lot beside it, every storey on its rows. */
function worldOf(plan: Plan, ox: number): WorldStructure {
  const { width: w, depth: d } = plan.lot;
  const W = ox + w + NEXT;
  const SD = d + STREET;
  const floors = [plan as Floor, ...(plan.upper ? [plan.upper] : [])];
  const D = SD * floors.length;
  /** The row of the world a plan's row 0 is on, on storey `k`. */
  const rows = (k: number) => STREET + k * SD;
  // Wall edges: 0 none, 1 a wall, 2 a door, 3 a window.
  const h = new Uint8Array(W * (D + 1));
  const v = new Uint8Array((W + 1) * D);
  floors.forEach((f, k) => {
    for (const [ax, az, bx, bz] of f.walls) {
      if (az === bz) for (let x = Math.min(ax, bx); x < Math.max(ax, bx); x++) h[(az + rows(k)) * W + ox + x] = 1;
      else if (ax === bx) for (let z = Math.min(az, bz); z < Math.max(az, bz); z++) v[(z + rows(k)) * (W + 1) + ox + ax] = 1;
      else throw new Error(`${plan.name}: a wall segment runs along the grid`);
    }
    for (const [list, kind] of [[f.doors, 2], [f.windows ?? [], 3]] as const) {
      for (const g of list) {
        const at = g.axis === 'x' ? (g.z + rows(k)) * W + ox + g.x : (g.z + rows(k)) * (W + 1) + ox + g.x;
        const edges = g.axis === 'x' ? h : v;
        if (edges[at] !== 1) throw new Error(`${plan.name}: a ${kind === 2 ? 'door' : 'window'} at ${g.x}, ${g.z} (${g.axis}) on storey ${k} is in no wall, or in another's place`);
        edges[at] = kind;
      }
    }
  });
  // Diagonal walls by tile; a tile one splits has two halves: half 0 touches its -z side (and its +x side for
  // `dp`, its -x side for `dn`), half 1 the two others.
  const diagonalAt = new Map<number, 'dp' | 'dn'>();
  for (const g of plan.diagonals ?? []) diagonalAt.set((g.z + rows(0)) * W + ox + g.x, g.axis);
  /** The half of tile `t` that touches its side `s` (0: -z, 1: +x, 2: +z, 3: -x). */
  const halfOn = (t: number, s: number): 0 | 1 => {
    const axis = diagonalAt.get(t);
    if (!axis) return 0;
    return (axis === 'dp' ? [0, 0, 1, 1] : [0, 1, 1, 0])[s] as 0 | 1;
  };
  // Rooms: what can't be reached from the edge of a storey's rows without crossing a wall (doors and windows are walls).
  const label = new Int32Array(W * D * 2).fill(-1);
  const flood = (t: number, half: 0 | 1, id: number) => {
    const stack = [t * 2 + half];
    while (stack.length) {
      const node = stack.pop()!;
      const [tile, hf] = [node >> 1, node & 1];
      if (label[node] >= 0) continue;
      label[node] = id;
      if (!diagonalAt.has(tile)) label[node ^ 1] = id;
      const [x, z] = [tile % W, Math.floor(tile / W)];
      const first = Math.floor(z / SD) * SD;
      for (let s = 0; s < 4; s++) {
        if (diagonalAt.has(tile) && halfOn(tile, s) !== hf) continue;
        const [nx, nz] = [x + [0, 1, 0, -1][s], z + [-1, 0, 1, 0][s]];
        if (nx < 0 || nx >= W || nz < first || nz >= first + SD) continue;
        const wall = s === 0 ? h[z * W + x] : s === 2 ? h[(z + 1) * W + x] : s === 1 ? v[z * (W + 1) + x + 1] : v[z * (W + 1) + x];
        if (wall) continue;
        const next = nz * W + nx;
        stack.push(next * 2 + halfOn(next, (s + 2) % 4));
      }
    }
  };
  for (let k = 0; k < floors.length; k++) {
    for (let x = 0; x < W; x++) for (const z of [k * SD, k * SD + SD - 1]) for (const half of [0, 1] as const) flood(z * W + x, half, 0);
    for (let z = k * SD; z < (k + 1) * SD; z++) for (const x of [0, W - 1]) for (const half of [0, 1] as const) flood(z * W + x, half, 0);
  }
  let next = 1;
  for (let node = 0; node < label.length; node++) if (label[node] < 0) flood(node >> 1, (node & 1) as 0 | 1, next++);
  const rooms = new Uint16Array(W * D);
  for (let t = 0; t < W * D; t++) rooms[t] = Math.max(label[t * 2], label[t * 2 + 1]);
  const room = (x: number, z: number) => (x >= 0 && z >= 0 && x < W && z < D ? rooms[z * W + x] : 0);

  // The walls as the simulation lists them, with their coverings: a plan with `each` has every wall covering in turn
  // on its inside faces; one `painted`, that covering on its outside faces.
  const walls: WallEdge[] = [];
  const openings: Opening[] = [];
  const painted = plan.painted ? indexOf(coverings, plan.painted) + 1 : 0;
  let [faces, doors, windows] = [0, 0, 0];
  const cover = (inside: number): number => (inside ? (plan.each ? (faces++ % coverings.length) + 1 : 0) : painted);
  const add = (axis: 'h' | 'v', x: number, z: number, e: number) => {
    if (!e) return;
    // (Face 0 looks towards -z or -x: the tile before the edge sees it.)
    const [before, after] = axis === 'h' ? [room(x, z - 1), room(x, z)] : [room(x - 1, z), room(x, z)];
    const look: [number, number] = [cover(before), cover(after)];
    const wall: WallEdge = { axis, x, z, ...(look[0] || look[1] ? { faces: look } : {}) };
    walls.push(wall);
    if (e === 2) openings.push({ ...wall, kind: 'door', ...(plan.each ? { style: doors++ % base.doorStyles.length } : {}) });
    if (e === 3) openings.push({ ...wall, kind: 'window', ...(plan.each ? { style: windows++ % base.windowStyles.length } : {}) });
  };
  for (let z = 0; z <= D; z++) for (let x = 0; x < W; x++) add('h', x, z, h[z * W + x]);
  for (let z = 0; z < D; z++) for (let x = 0; x <= W; x++) add('v', x, z, v[z * (W + 1) + x]);
  const diagonals: DiagonalWall[] = (plan.diagonals ?? []).map((g, i) => {
    const t = (g.z + rows(0)) * W + ox + g.x;
    const of: [number, number] = [label[t * 2], label[t * 2 + 1]];
    const look: [number, number] = [cover(of[0]), cover(of[1])];
    return { axis: g.axis, x: ox + g.x, z: g.z + rows(0), kind: g.kind, rooms: of, ...(look[0] || look[1] ? { faces: look } : {}), ...(plan.each && g.kind !== 'wall' ? { style: (i + 3) % (g.kind === 'door' ? base.doorStyles.length : base.windowStyles.length) } : {}) };
  });
  // A floor covering a room on a plan with `each`: every third of them, which takes in each finish.
  const floorOf = new Map<number, number>();
  const laid: [number, number, number][] = [];
  if (plan.each) {
    for (let z = 0; z < D; z++) {
      for (let x = 0; x < W; x++) {
        const r = room(x, z);
        if (!r) continue;
        if (!floorOf.has(r)) floorOf.set(r, (floorOf.size * 3) % floorCoverings.length);
        laid.push([x, z, floorOf.get(r)! + 1]);
      }
    }
  }
  const placed: ObjectPlacement[] = floors.flatMap((f, k) =>
    (f.objects ?? []).map((o, i): ObjectPlacement => {
      const [fw, fd] = objects.get(o.def)?.footprint ?? [1, 1];
      const [pw, pd] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
      return { id: k * 1000 + i, def: o.def, x: ox + o.x, z: o.z + rows(k), rot: o.rot, w: pw, d: pd, quality: 0, sellValue: null };
    }),
  );
  // The house's box of walls (the ground storey's), its front door (the plan's first) and the path from it to the street.
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [ax, az, bx, bz] of plan.walls) [x0, z0, x1, z1] = [Math.min(x0, ax, bx), Math.min(z0, az, bz), Math.max(x1, ax, bx), Math.max(z1, az, bz)];
  const front = plan.doors[0];
  if (!front || front.axis !== 'x' || front.z !== z0) throw new Error(`${plan.name}: its first door is its front door, in the wall along the street`);
  const roof = plan.roof ? ([indexOf(base.roofStyles, plan.roof[0]), indexOf(base.roofColors, plan.roof[1])] as [number, number]) : undefined;
  const plots: PlotInfo[] = [
    { id: 0, name: plan.name, x: ox, z: STREET, w, d, public: false, entry: [ox + front.x + 0.5, STREET + 1], house: [ox + x0, STREET + z0, ox + x1, STREET + z1], ...(roof ? { roof } : {}), ...(plan.upper ? { storeys: 2 } : {}) },
    { id: 1, name: 'next door', x: ox + w, z: STREET, w: NEXT, d, public: false, entry: null, house: null },
  ];
  const road: RoadPlan = { points: [[-40, STREET / 2], [W + 40, STREET / 2]], width: STREET, ends: ['out', 'out'] };
  return {
    version: 0,
    lotVersion: 0,
    mode: 'living',
    width: W,
    depth: D,
    ...(plan.upper ? { storeys: 2, storeyDepth: SD } : {}),
    objects: placed,
    sims: [],
    households: [],
    plots,
    exits: [],
    rooms,
    walls,
    openings,
    diagonals,
    fences: [],
    ...(laid.length ? { floors: laid } : {}),
    meta: { kind: 'town', name: 'Sample', roads: [road], paths: [{ x: ox + front.x, z: STREET, w: 1, d: z0 }], facing: [0, 0], seed: 1, gardenFences: true },
    meshes: { walls: NO_MESH, wallsLow: NO_MESH, floors: NO_MESH },
    garages: plan.garage ? [{ plot: 0, x0: ox + x1 + 2, z0: STREET + z0, x1: ox + x1 + 6, z1: STREET + z0 + 6 }] : [],
    porches: plan.porch ? [{ plot: 0, axis: 'h', x: ox + front.x, z: STREET + z0, out: -1 }] : [],
    cars: [],
  };
}
const NO_MESH = { positions: new Float32Array(0), normals: new Float32Array(0), uvs: new Float32Array(0), indices: new Uint32Array(0) };

/** A planner with the content's looks, as the renderer gives them. */
function plannerOf(): InstanceType<typeof HousePlanner> {
  const planner = new HousePlanner();
  planner.setLooks({ coverings: base.wallCoverings, floors: base.floorCoverings, doors: base.doorStyles, windows: base.windowStyles, roofs: base.roofStyles, roofColors: base.roofColors });
  return planner;
}

const STREET_OPTIONS = { has: () => true, entry: () => undefined, lived: false };

/** A plan's four samples: the viewed house, the neighbour's silhouette, the street's far house, the town view's box. */
async function houseSamples(plan: Plan): Promise<Sample[]> {
  const planner = plannerOf();
  // The lot's place: the first along x whose look is the plan's.
  // (Not on the world's edge: a row house's wall stands on its lot's.)
  let ox = 2;
  for (; ; ox += 1) {
    if (ox > 4000) throw new Error(`${plan.name}: no lot has its look`);
    if (plan.look(houseScheme(ox, STREET))) break;
  }
  const world = worldOf(plan, ox);
  const [home, next] = world.plots;
  const count = world.storeys ?? 1;
  const SD = world.storeyDepth ?? world.depth;
  const W = world.width;
  const about = `${plan.about} (lot at x ${ox})`;

  // The viewed house: every storey as the renderer builds it (`PlayCanvasRenderer.buildLot`), lifted to its height
  // and back over the ground's rows, its door leaves shut; then the lot's own share of the street, its garage and porch.
  const near: DumpMesh[] = [];
  for (let k = 0; k < count; k++) {
    const covered = (x: number, z: number) => k + 1 < count && (world.rooms[(z + SD) * W + x] ?? 0) !== 0;
    const view = { x: home.x, z: home.z + k * SD, w: home.w, d: home.d };
    const built = planner.build(world, view, k === 0 ? { covered, holes: new Set(), landings: new Set() } : { upper: true, covered, holes: new Set(), landings: new Set() });
    if (!built) throw new Error(`${plan.name}: no house on storey ${k}`);
    const at: Placed = { dy: k * WALL_HEIGHT, dz: -k * SD };
    const leaves = new Map(built.doors.flatMap((door) => door.parts.map((p) => [p, door] as const)));
    for (const part of built.parts) {
      const door = leaves.get(part);
      near.push(partMesh(part, { storey: k, ...(door ? { door: true } : {}) }, door ? { ...at, turn: door.close, pivot: door.pivot } : at));
    }
  }
  const viewed = await planStreet(planner, world, { x: home.x, z: home.z, w: home.w, d: home.d }, STREET_OPTIONS);
  for (const part of viewed.lot?.silhouettes.parts ?? []) near.push(partMesh(part, { storey: 0, lot: true }));

  // From next door: the street's silhouette of it.
  const seen = await planStreet(planner, world, { x: next.x, z: next.z, w: next.w, d: next.d }, STREET_OPTIONS);
  const neighbour = seen.silhouettes.parts.map((part) => partMesh(part));

  // Far off: the street's far house, and the town view's box (`pc/street.ts`: `farLook` on the lot's pad, here 0).
  const plot = { x: home.x, z: home.z, house: home.house!, roof: home.roof, storeys: home.storeys, cover: planner.exteriorCover(world, home.house!, home.storeys ?? 1) };
  const far = new GeoClass();
  planner.farHouse(far, plot);
  const town = lotBuilding({ id: home.id, chunk: 0, storeys: home.storeys ?? 1, pad: 0, ...planner.farLook(plot) }, () => 0);

  const key = `house.${plan.name}`;
  return [
    // (Each part of the viewed house is a draw of its own: a room's glass, a door's leaf.)
    { key, kind: 'house', about, storeys: count, meshes: near.filter((m) => m.indices.length > 0) },
    { key: `neighbour.${plan.name}`, kind: 'neighbour', of: key, about, storeys: count, meshes: merged(neighbour) },
    { key: `far.${plan.name}`, kind: 'far', of: key, about, storeys: count, meshes: [meshOf('farHouses', BUILT_MATERIALS.plain, far)] },
    { key: `town.${plan.name}`, kind: 'town', of: key, about, storeys: count, meshes: unitMeshesOf(town) },
  ];
}

/** A building as the town view draws it (`pc/townView.ts`): the unit walls and its roof's unit mesh, each scaled, turned and put in place as its copy's matrix does, in its copy's colours (a unit mesh's red says wall or roof, its green the tone). */
function unitMeshesOf(b: TownBuilding): DumpMesh[] {
  const units = unitMeshes();
  const copy = (name: string, unit: Geo, up: number, y: number): DumpMesh => {
    const geo = new GeoClass();
    const { pos, nor, uv, col, idx } = unit;
    for (let i = 0; i < unit.vertexCount; i++) {
      const [x, h, z] = [pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]];
      // (A normal goes through the matrix's inverse transpose: each axis by the other two's scale.)
      const [nx, ny, nz] = [nor[i * 3] / b.w, nor[i * 3 + 1] / up, nor[i * 3 + 2] / b.d];
      const [wx, wz] = [nx * b.ux - nz * b.uz, nx * b.uz + nz * b.ux];
      const l = Math.hypot(wx, ny, wz) || 1;
      const of = col[i * 4] > 127 ? b.wall : b.roofColor;
      const tone = col[i * 4 + 1] / 255;
      geo.r = of[0] * tone;
      geo.g = of[1] * tone;
      geo.b = of[2] * tone;
      geo.a = col[i * 4 + 3] / 255;
      geo.point([b.cx + x * b.ux * b.w - z * b.uz * b.d, y + h * up, b.cz + x * b.uz * b.w + z * b.ux * b.d], [wx / l, ny / l, wz / l], uv[i * 2], uv[i * 2 + 1]);
    }
    for (let i = 0; i < idx.length; i += 3) geo.face(idx[i], idx[i + 1], idx[i + 2]);
    return meshOf(name, BUILT_MATERIALS.plain, geo);
  };
  return [copy('town.walls', units.walls, b.top - b.base, b.base), copy(`town.${b.roof}`, units[b.roof], b.rise, b.top)];
}

// ---- the lots' buildings closed ---------------------------------------------------------------------

/**
 * A building of each use a lot's look has: its size (m along its ridge and across), its roof, its storeys (at most the
 * world's three), as the town's generator makes them about, and its look (the palette's).
 */
const BUILDINGS: [BuildingUse, number, number, LotLook['roof'], number, number][] = [
  ['flats', 16, 11, 'flat', 3, 2],
  ['shop', 10, 8, 'flat', 2, 3],
  ['cafe', 12, 9, 'hip', 2, 4],
  ['restaurant', 14, 10, 'mansard', 2, 5],
  ['office', 18, 12, 'flat', 3, 6],
  ['school', 24, 12, 'hip', 2, 7],
  ['church', 18, 10, 'gable', 3, 8],
  ['hall', 16, 11, 'mansard', 2, 9],
  ['workshop', 12, 9, 'gable', 1, 10],
  ['farm', 13, 9, 'hip', 2, 11],
];

/** A lot's building of each use closed, dressed as the street draws it near, and as the town view's box. */
function buildingSamples([use, w, d, roof, storeys, look]: (typeof BUILDINGS)[number]): Sample[] {
  // (Its road to the south.)
  const [cx, cz] = [100, 50];
  const t = lotLookBuilding([cx - w / 2, cz - d / 2, cx + w / 2, cz + d / 2], { use, storeys, roof, look }, storeys, 0, () => 0);
  const dress = dressBuilding(t, use, [cx, cz + 30]);
  const about = `a ${use}, ${w} m by ${d} m, ${storeys} storey${storeys === 1 ? '' : 's'} under a ${roof} roof, look ${look}`;
  const near = [...dress.parts.map((p) => partMesh(p)), ...dress.panes.map((g, k) => meshOf('buildingPanes', HOUSE_MATERIALS.glass, g, { set: k }))];
  const key = `building.${use}`;
  return [
    { key, kind: 'building', about, storeys, meshes: merged(near) },
    { key: `buildingFar.${use}`, kind: 'buildingFar', of: key, about, storeys, meshes: unitMeshesOf(t) },
  ];
}

// ---- a street ---------------------------------------------------------------------------------------

const fenceStyles = base.fenceStyles;

/** A fence of `style` round the rectangle (x0, z0)..(x1, z1), a gate in the middle of its -z side. */
function fenceRound(x0: number, z0: number, x1: number, z1: number, style: number): FenceEdge[] {
  const out: FenceEdge[] = [];
  const gate = Math.floor((x0 + x1) / 2);
  for (let x = x0; x < x1; x++) out.push({ axis: 'h', x, z: z0, kind: x === gate ? 'gate' : 'fence', style }, { axis: 'h', x, z: z1, kind: 'fence', style });
  for (let z = z0; z < z1; z++) out.push({ axis: 'v', x: x0, z, kind: 'fence', style }, { axis: 'v', x: x1, z, kind: 'fence', style });
  return out;
}

/**
 * A made-up block: a street along x crossed by another (four arms: lit), a lane off it that bends and ends in a
 * turning circle, a country road off the cross street over a bridge; six lots along the street, a box house on
 * each, each garden in another fence style, a car parked on each.
 */
function streetWorld(): WorldStructure {
  const [W, D] = [240, 200];
  const line = (x0: number, z0: number, x1: number, z1: number, n: number): [number, number][] => Array.from({ length: n + 1 }, (_, i) => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
  const bend: [number, number][] = [...line(60, 105, 60, 150, 9), ...Array.from({ length: 8 }, (_, i): [number, number] => [60 + 20 * (1 - Math.cos(((i + 1) * Math.PI) / 16)), 150 + 20 * Math.sin(((i + 1) * Math.PI) / 16)]), ...line(80, 170, 120, 170, 8).slice(1)];
  const roads: RoadPlan[] = [
    { points: line(0, 100, W, 100, 24), width: 10, ends: ['out', 'out'], name: 'High Street' },
    { points: line(160, 0, 160, D, 20), width: 10, ends: ['out', 'out'], name: 'Cross Street' },
    { points: bend, width: 8, ends: ['join', 'dead'], kind: 'lane', name: 'Bend Lane' },
    { points: line(165, 40, 240, 40, 15), width: 7, ends: ['join', 'out'], kind: 'country', bridge: [30, 60] },
  ];
  const walls: WallEdge[] = [];
  const openings: Opening[] = [];
  const plots: PlotInfo[] = [];
  const paths: { x: number; z: number; w: number; d: number }[] = [];
  const fences: FenceEdge[] = [];
  const rooms = new Uint16Array(W * D);
  for (let i = 0; i < fenceStyles.length; i++) {
    // Lots 22 m wide north of the street (z 73..95), the house's front door in its +z wall.
    const [x, z, w, d] = [8 + i * 24, 73, 22, 22];
    const [x0, z0, x1, z1] = [x + 5, z + 5, x + 15, z + 13];
    for (let u = x0; u < x1; u++) walls.push({ axis: 'h', x: u, z: z0 }, { axis: 'h', x: u, z: z1 });
    for (let u = z0; u < z1; u++) walls.push({ axis: 'v', x: x0, z: u }, { axis: 'v', x: x1, z: u });
    for (let u = z0; u < z1; u++) for (let t = x0; t < x1; t++) rooms[u * W + t] = i + 1;
    openings.push({ axis: 'h', x: x0 + 4, z: z1, kind: 'door' }, { axis: 'h', x: x0 + 1, z: z1, kind: 'window' }, { axis: 'h', x: x0 + 7, z: z1, kind: 'window' }, { axis: 'v', x: x0, z: z0 + 3, kind: 'window' });
    plots.push({ id: i, name: `${i}`, x, z, w, d, public: false, entry: [x0 + 4.5, z + d - 1], house: [x0, z0, x1, z1], ...(i % 2 ? { storeys: 2 } : {}) });
    paths.push({ x: x0 + 4, z: z1, w: 1, d: z + d - z1 });
    fences.push(...fenceRound(x + 1, z + 1, x + w - 1, z + d - 1, i));
  }
  const world: WorldStructure = {
    version: 0,
    lotVersion: 0,
    mode: 'living',
    width: W,
    depth: D,
    objects: [],
    sims: [],
    households: [],
    plots,
    exits: [],
    rooms,
    walls,
    openings,
    fences,
    meta: { kind: 'town', name: 'Sample', roads, paths, facing: plots.map(() => 2), seed: 1, gardenFences: true },
    meshes: { walls: NO_MESH, wallsLow: NO_MESH, floors: NO_MESH },
    porches: plots.map((p, i) => ({ plot: i, axis: 'h' as const, x: p.house![0] + 4, z: p.house![3], out: 1 })),
    cars: plots.map((p, i) => ({ plot: i, x: p.x + 18.5, z: p.z + 16, yaw: 0, body: i % CARS.length, paint: i % CAR_PAINT.length })),
    junctions: deriveJunctions(roads),
  };
  return world;
}

/** The street's block as the game draws it with no lot in view: every chunk's share, each layer as one mesh. */
function streetSample(): Sample {
  const world = streetWorld();
  const street = new TownStreet(plannerOf(), world, { ...STREET_OPTIONS, lived: true, ground: true, fenceStyles }, 'game');
  const meshes: DumpMesh[] = [];
  for (const id of street.chunkIds()) {
    const c = street.chunk(id, NO_FOCUS);
    for (const part of c.silhouettes.parts) meshes.push(partMesh(part));
    for (const home of c.silhouettes.homes) meshes.push(meshOf('silhouetteWindowsLit', HOUSE_MATERIALS.glass, home.panes));
    meshes.push(meshOf('sidewalks', 'material.sidewalk', c.walk), meshOf('props', BUILT_MATERIALS.plain, c.props), meshOf('glow', BUILT_MATERIALS.glow, c.glow));
    meshes.push(meshOf('lightPools', BUILT_MATERIALS.pools, c.pools), meshOf('contactShadows', BUILT_MATERIALS.shadow, c.shadows));
    for (const g of c.ground) meshes.push(meshOf(g.name, g.paint ? BUILT_MATERIALS.paint : g.material, g.geo, g.toned ? { toned: true } : {}));
  }
  return { key: 'street.block', kind: 'street', about: 'two streets crossing, a lane that bends to a dead end, a country road over a bridge, six lots with a house, a fenced garden and a parked car each', meshes: merged(meshes) };
}

/** A fence style round a garden 6 m by 4 m, with its gate. */
function fenceSample(style: number): Sample | null {
  const def = fenceStyles[style];
  // (A hedge is the hedge model's copies, not built here; its gate is a wooden one.)
  const p = new Painter(new GeoClass());
  writeFences(p, fenceRound(2, 2, 8, 6, style), fenceStyles, { gates: null, hedges: [] });
  if (p.geo.empty) return null;
  return { key: `fence.${def.id}`, kind: 'fence', about: `${def.label}: round 6 m by 4 m, a gate in it`, meshes: [meshOf('fences', BUILT_MATERIALS.plain, p.geo)] };
}

/** A car body parked, in the street's paint: the family cars used, the others' ages by turns (used, new, old). */
function carSample(body: number): Sample {
  const p = new Painter(new GeoClass());
  const age = (body < 4 ? 1 : body % 3) as 0 | 1 | 2;
  paintCar(p.at(0, 0), CARS[body], CAR_PAINT[body], false, true, age);
  const name = CARS[body].id;
  return { key: `car.${name}`, kind: 'car', about: `the ${name} parked, paint ${CAR_PAINT[body]}, ${['old', 'used', 'new'][age]}`, meshes: [meshOf('car', BUILT_MATERIALS.plain, p.geo)] };
}

/** Every sample, in the order it is dumped. */
export async function samples(): Promise<Sample[]> {
  const out: Sample[] = [];
  for (const plan of plans()) out.push(...(await houseSamples(plan)));
  for (const b of BUILDINGS) out.push(...buildingSamples(b));
  out.push(streetSample());
  for (let i = 0; i < fenceStyles.length; i++) {
    const s = fenceSample(i);
    if (s) out.push(s);
  }
  for (let i = 0; i < CARS.length; i++) out.push(carSample(i));
  return out;
}

/** What the gate reads beside the meshes (`built.json`): the samples, and the two measures of the cutaway it applies. */
export function index(list: readonly Sample[]): string {
  return `${JSON.stringify(
    {
      about: 'The built world\'s samples (tools/built/samples.ts): metres, y up, the game\'s z mirrored as PlayCanvas draws it.',
      storey: WALL_HEIGHT,
      stub: WALL_STUB,
      samples: list.map((s) => ({ key: s.key, kind: s.kind, ...(s.of ? { of: s.of } : {}), ...(s.storeys ? { storeys: s.storeys } : {}), about: s.about })),
    },
    null,
    2,
  )}\n`;
}
