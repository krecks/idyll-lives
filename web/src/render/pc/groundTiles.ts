/**
 * A big world's ground tiles (`nature.ts` `terrainTiles`): each has three levels of detail and draws
 * one of them, chosen by distance (`frameTiles`). Apart, so it can be checked on the engine's null
 * device without the rest of the landscape. And the town view's colour map over them (`TownGroundMap`).
 */

import * as pc from './engine';
import { groundDepth, type RoadPlan, type WorldStructure, type WorldZone } from '../../core/protocol';
import { carriageway, hasSidewalks, planJunctions } from '../shared/junctions';
import { kindInfo } from '../../core/roadKinds';
import { lengths } from '../../game/roads';
import { junctionStretches } from '../shared/markings';
import { allBarriers, OVERPASS, without } from '../shared/motorway';
import { boxOf, clipRect, COURSE, courseCells, JOINT, LINE, PAVING_LOOK, TILE, townPaving, type TownPaving } from '../shared/townPaving';
import { PATH_LOOK, pathLook } from '../shared/paving';
import { bridgeSpans, raisedSpans, stretchOf } from '../shared/roads';
import { BRIDGE } from '../shared/street';
import type { FieldCanvas } from '../shared/farmland';

/**
 * A ground tile as `terrainTiles` builds it: its draw, its meshes by level (the finest only while it is near enough to
 * draw it: null otherwise), where its middle is and the level drawn.
 */
export interface GroundTile {
  mi: pc.MeshInstance;
  meshes: (pc.Mesh | null)[];
  x: number;
  z: number;
  lod: number;
  /**
   * Its finest grid's colours as shaded (RGBA bytes, sRGB, its ring too) and its heights (m): the levels are made from
   * them (`Landscape.tileLevel`), and a height field's brushes draw them again with the colours (`Landscape.reground`)
   * until it's shaded again.
   */
  kept?: Uint8Array;
  heights?: Float32Array;
  /** The levels (a bit each) a brush changed the heights of while another was drawn: written again when next drawn. */
  stale: number;
  /** How the roads level the land at each point of its grid (`Landscape.roadLevels`), once a brush has reached it. */
  levels?: Float32Array;
}

/**
 * Destroys the ground tiles' levels no draw holds (all but the one each draws now, which goes with its
 * entity): left to themselves, about 40 MB of a big world's vertex buffers with every new game or load.
 */
export function disposeTiles(tiles: readonly GroundTile[]): void {
  for (const t of tiles) for (const m of t.meshes) if (m && m !== t.mi.mesh) m.destroy();
}

// ---- the town view's colour map (a ground colour map at altitude) ----

/** Texels a metre at most, and the map's side at most (2048: half a metre a texel over a Big world). */
const MAP_PER_M = 2;
const MAP_MAX = 2048;
/**
 * The map's colours (sRGB): the asphalt and pavements as the street's textures and tones come out, soil, paths; and
 * the town's paving as the near ground shows it (`townPaving.ts` `PAVING_LOOK`): the centre's stone and its joints,
 * yards, a car park's asphalt and its bays' paint.
 */
const MAP = {
  asphalt: '#55544F',
  pavement: '#C4C0B6',
  soil: '#6B4E37',
  furrow: '#5A412E',
  path: PATH_LOOK.hex.map,
  paving: PAVING_LOOK.stone,
  joint: PAVING_LOOK.joint,
  yard: PAVING_LOOK.yard,
  carpark: PAVING_LOOK.asphalt,
  line: PAVING_LOOK.line,
  median: '#66783F',
  concrete: '#AFACA4',
} as const;

/** What the maps ask of the land: a big world's farmland, its box and how to paint it (`LandscapeLayout.farmland`). */
export interface TownGroundLand {
  farmland(): { box: [number, number, number, number]; paint(g: FieldCanvas): void } | null;
  /** Counts the changes of a land drawn from a height field (its cover moves the farmland): the map is made again for a new one. */
  readonly landVersion?: number;
}

/**
 * What a town's ground map is drawn from, as a hash: its roads, lots, centres, paving, bays and car parks, greens,
 * fields and zones (not how many of each: another town with a street moved, or a lot moved, has the same
 * counts). Kept per world's meta.
 */
function groundVersion(world: WorldStructure): number {
  const meta = world.meta;
  if (!meta) return 0;
  const known = groundVersions.get(meta);
  if (known?.plots === world.plots && known.parking === world.parking && known.carParks === world.carParks) return known.version;
  const text = JSON.stringify([meta.roads, world.plots.map((p) => [p.x, p.z, p.w, p.d]), meta.centre, meta.paving, world.parking, world.carParks, meta.greens, meta.world?.fields, meta.zones, meta.paths]);
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) h = Math.imul(h ^ text.charCodeAt(i), 16777619);
  const version = h >>> 0;
  groundVersions.set(meta, { plots: world.plots, parking: world.parking, carParks: world.carParks, version });
  return version;
}
const groundVersions = new WeakMap<object, { plots: WorldStructure['plots']; parking: WorldStructure['parking']; carParks: WorldStructure['carParks']; version: number }>();

/** What `paintPaving` draws with: a 2D canvas's context (or a stand-in that records what is drawn, in the tests). */
export type PavingCanvas = Pick<OffscreenCanvasRenderingContext2D, 'fillStyle' | 'strokeStyle' | 'lineWidth' | 'lineCap' | 'fillRect' | 'beginPath' | 'moveTo' | 'lineTo' | 'closePath' | 'rect' | 'fill' | 'stroke'>;

/**
 * The town's paving into the map (`g`, in town metres over a `W` × `D` ground): the tiles it lays whole
 * (`TownPaving.tiles`) a row's run of them at a time, the stone's joints over them (`COURSE` m apart, `JOINT` wide) as
 * the near ground cuts them, the tiles laid in part along a piece's edge (`TownPaving.parts`) as far as it, cut at their
 * joints as the near ground cuts them, then the pieces laid by their own outlines (a car park's apron) less the lots in
 * them, and the bays' lines.
 */
export function paintPaving(g: PavingCanvas, paving: TownPaving, W: number, D: number): void {
  const laid = paving.tiles(W, D);
  const parts = paving.parts(W, D);
  const whole = (i: number) => (parts.has(i) ? 0 : laid[i]);
  const runs: [number, number, number, number][] = [];
  for (let z = 0; z < D; z++) {
    for (let x = 0; x < W; ) {
      const c = whole(z * W + x);
      let e = x + 1;
      while (e < W && whole(z * W + e) === c) e++;
      if (c) runs.push([c, x, z, e - x]);
      x = e;
    }
  }
  for (const [c, x, z, n] of runs) {
    g.fillStyle = c === TILE.yard ? MAP.yard : MAP.paving;
    g.fillRect(x, z, n, 1);
  }
  // The joints: across a run where one runs along x through its row, and along its row where one runs along z.
  g.fillStyle = MAP.joint;
  const h = JOINT / 2;
  for (const [c, x, z, n] of runs) {
    if (c !== TILE.stone) continue;
    for (let k = Math.ceil((x - h) / COURSE); k * COURSE - h < x + n; k++) {
      const [a, b] = [Math.max(x, k * COURSE - h), Math.min(x + n, k * COURSE + h)];
      if (b > a) g.fillRect(a, z, b - a, 1);
    }
    for (let k = Math.ceil((z - h) / COURSE); k * COURSE - h < z + 1; k++) {
      const [a, b] = [Math.max(z, k * COURSE - h), Math.min(z + 1, k * COURSE + h)];
      if (b > a) g.fillRect(x, a, n, b - a);
    }
  }
  // (Each cell of a part on its own: two pieces' parts in one tile may overlap.)
  for (const [i, polys] of parts) {
    const [x, z, c] = [i % W, Math.floor(i / W), laid[i]];
    for (const { cell, joint } of courseCells(x, z, x + 1, z + 1, c === TILE.stone ? COURSE : 0)) {
      g.fillStyle = joint ? MAP.joint : c === TILE.yard ? MAP.yard : MAP.paving;
      for (const q of polys) {
        const part = clipRect(q, ...cell);
        if (part.length < 3) continue;
        g.beginPath();
        part.forEach(([px, pz], k) => (k ? g.lineTo(px, pz) : g.moveTo(px, pz)));
        g.closePath();
        g.fill();
      }
    }
  }
  for (const p of paving.pieces) {
    if (p.edge !== 'exact') continue;
    g.fillStyle = MAP[p.surface === 'asphalt' ? 'carpark' : p.surface === 'stone' ? 'paving' : 'yard'];
    g.beginPath();
    p.poly.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
    g.closePath();
    // (Its lots cut out of it: each a rectangle inside it, filled even-odd.)
    const [x0, z0, x1, z1] = boxOf(p.poly);
    for (const r of p.holes) if (r.x < x1 && r.x + r.w > x0 && r.z < z1 && r.z + r.d > z0) g.rect(r.x, r.z, r.w, r.d);
    g.fill('evenodd');
  }
  g.strokeStyle = MAP.line;
  g.lineWidth = LINE;
  g.lineCap = 'butt';
  g.beginPath();
  for (const b of paving.bays) {
    for (const [p, q] of b.lines) {
      g.moveTo(p[0], p[1]);
      g.lineTo(q[0], q[1]);
    }
  }
  g.stroke();
}

/** How far either side of a road (past its width) and round a lot (m) the farmland stops: the verge, the lot's edge. */
const FIELD_VERGE = 2.5;
const FIELD_LOT = 1;

/**
 * A big world's farmland onto a map (`g` in world metres), under whatever is drawn after it: its fields
 * (`LandscapeLayout.farmland`), then none of them on the roads and their verges or on the lots.
 */
function paintFarmland(g: OffscreenCanvasRenderingContext2D, farmland: NonNullable<ReturnType<TownGroundLand['farmland']>>, world: WorldStructure): void {
  g.save();
  farmland.paint(g);
  g.globalCompositeOperation = 'destination-out';
  g.fillStyle = '#000';
  g.strokeStyle = '#000';
  g.lineCap = 'round';
  g.lineJoin = 'round';
  for (const p of world.plots) g.fillRect(p.x - FIELD_LOT, p.z - FIELD_LOT, p.w + 2 * FIELD_LOT, p.d + 2 * FIELD_LOT);
  for (const r of world.meta?.roads ?? []) {
    g.lineWidth = r.width + 2 * FIELD_VERGE;
    g.beginPath();
    r.points.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
    g.stroke();
  }
  g.restore();
}

/** A canvas's pixels premultiplied by their alpha (in place): a map's mips stay true at an edge. */
function premultiplied(data: Uint8ClampedArray): Uint8Array {
  for (let i = 0; i < data.length; i += 4) {
    const a = data[i + 3] / 255;
    data[i] = Math.round(data[i] * a);
    data[i + 1] = Math.round(data[i + 1] * a);
    data[i + 2] = Math.round(data[i + 2] * a);
  }
  return new Uint8Array(data.buffer);
}

/** A map of no farmland (nothing in it), for the ground's materials until a landscape's own is made; the materials given it. */
const noFields = new WeakMap<pc.GraphicsDevice, pc.Texture>();
const fielded = new WeakSet<pc.StandardMaterial>();

/**
 * The ground's material takes the farmland's map (`shaders/fieldGround.ts`): its define, and no fields until a
 * landscape gives its tiles their own (`FieldMap.lay`, on each draw: two landscapes may share the material).
 */
export function fieldGround(device: pc.GraphicsDevice, material: pc.StandardMaterial): void {
  if (fielded.has(material)) return;
  fielded.add(material);
  let none = noFields.get(device);
  if (!none) noFields.set(device, (none = new pc.Texture(device, { name: 'noFields', width: 1, height: 1, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, levels: [new Uint8Array(4)] })));
  material.setDefine('GAME_FIELD_MAP', '');
  material.setParameter('game_fieldMap', none);
  material.setParameter('game_fieldArea', new Float32Array([0, 0, 0, 0]));
  material.update();
}

/**
 * A big world's farmland as the near ground draws it: a colour map over the box its fields reach (`MAP_PER_M` texels
 * a metre, at most `MAP_MAX` a side), each field's crop in its rows inside its grass margin and its hedges
 * (`farmland.ts` `paintFields`), none on the roads, their verges or the lots; laid over the ground tiles' colour
 * (`shaders/fieldGround.ts`) but in the town view, whose own map has them (`TownGroundMap`). Made again for another
 * town or land (`bake`).
 */
export class FieldMap {
  private texture: pc.Texture | null = null;
  private key = '';
  private readonly area = new Float32Array(4);

  constructor(private readonly device: pc.GraphicsDevice) {}

  /** Makes the map for `world` on `land` (nothing again for the same; none without farmland or a 2D canvas). Says whether it changed. */
  bake(world: WorldStructure, land: TownGroundLand): boolean {
    const farmland = typeof OffscreenCanvas === 'undefined' ? null : land.farmland();
    const key = farmland ? `${world.meta?.seed ?? 0}|${groundVersion(world)}|${land.landVersion ?? 0}|${farmland.box.join(',')}` : '';
    if (key === this.key) return false;
    this.key = key;
    this.texture?.destroy();
    this.texture = null;
    this.area.fill(0);
    if (!farmland) return true;
    const [x0, z0, x1, z1] = farmland.box;
    const w = Math.max(1, Math.min(MAP_MAX, Math.ceil((x1 - x0) * MAP_PER_M)));
    const h = Math.max(1, Math.min(MAP_MAX, Math.ceil((z1 - z0) * MAP_PER_M)));
    const canvas = new OffscreenCanvas(w, h);
    const g = canvas.getContext('2d', { willReadFrequently: true })!;
    const [sx, sz] = [w / (x1 - x0), h / (z1 - z0)];
    g.setTransform(sx, 0, 0, sz, -x0 * sx, -z0 * sz);
    paintFarmland(g, farmland, world);
    const data = premultiplied(g.getImageData(0, 0, w, h).data);
    this.texture = new pc.Texture(this.device, { name: 'fieldMap', width: w, height: h, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, anisotropy: 8, minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [data] });
    this.area.set([x0, z0, 1 / (x1 - x0), 1 / (z1 - z0)]);
    return true;
  }

  /** The map on these draws (the ground tiles'), or none. */
  lay(draws: Iterable<pc.MeshInstance>): void {
    const none = noFields.get(this.device);
    for (const mi of draws) {
      if (this.texture) {
        mi.setParameter('game_fieldMap', this.texture);
        mi.setParameter('game_fieldArea', this.area);
      } else if (none) {
        mi.setParameter('game_fieldMap', none);
        mi.setParameter('game_fieldArea', this.area);
      }
    }
  }

  dispose(): void {
    this.texture?.destroy();
    this.texture = null;
    this.key = '';
  }
}

/**
 * The ground's colour map at altitude: what the town lays on its land (streets and pavements, the junctions'
 * pads, an avenue's green median, a motorway's barrier, its overpasses over what they cross, its paving as the street lays it near (`townPaving`: the centre, courts and yards, car parks and their
 * bays), its farm fields and paths, a big world's farmland as its ground
 * has it) over the whole world in one texture, baked once per town (`bake`; again only for another town), and laid over the ground tiles' colour by the town
 * view's blend (`shaders/townGround.ts`). The tiles' material takes its define only while the blend is over 0
 * (`apply`): up to `TOWN_FROM` the ground draws with the shader it always had.
 */
export class TownGroundMap {
  private texture: pc.Texture | null = null;
  private key = '';
  /** Counts the maps baked: one more for each town whose ground differs from the last's (`groundVersion`). */
  version = 0;
  private area = new Float32Array(4);
  private on = false;

  constructor(private readonly device: pc.GraphicsDevice) {}

  /**
   * Bakes the map for `world` (nothing again for the same town; nothing without a 2D canvas: the tests' engine).
   * `land`: the landscape, for a big world's farmland.
   */
  bake(world: WorldStructure, land: TownGroundLand | null = null): void {
    const meta = world.meta;
    const W = world.width;
    const D = groundDepth(world);
    const key = `${meta?.seed ?? 0}|${W}x${D}|${groundVersion(world)}|${land ? 1 : 0}|${land?.landVersion ?? 0}`;
    if (key === this.key || typeof OffscreenCanvas === 'undefined' || !meta) return;
    this.key = key;
    this.version++;
    const size = Math.min(MAP_MAX, 2 ** Math.ceil(Math.log2(Math.max(W, D) * MAP_PER_M)));
    const sx = size / W;
    const sz = size / D;
    const canvas = new OffscreenCanvas(size, size);
    const g = canvas.getContext('2d', { willReadFrequently: true })!;
    g.setTransform(sx, 0, 0, sz, 0, 0);
    // A big world's farmland first, under everything: its fields, as the near ground has them (`FieldMap`).
    const farmland = land?.farmland();
    if (farmland) paintFarmland(g, farmland, world);
    g.lineCap = 'round';
    g.lineJoin = 'round';
    const poly = (p: readonly (readonly [number, number])[]) => {
      g.beginPath();
      p.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
      g.closePath();
    };
    const roads = meta.roads ?? [];
    // (No water: the water's sheet over the ground draws it, inside the world as past its edge. Painted here, a sea
    // reaching past the edge stepped from one blue to another along it.)
    // None of the farmland on the lots (their gardens stay theirs) or the greens.
    g.globalCompositeOperation = 'destination-out';
    g.fillStyle = '#000';
    for (const p of world.plots) g.fillRect(p.x, p.z, p.w, p.d);
    for (const green of meta.greens ?? []) {
      poly(green);
      g.fill();
    }
    g.globalCompositeOperation = 'source-over';
    // The town's paving (`townPaving`: the centre's stone in courses, the courts and yards, the car parks and their
    // bays), as the street lays it near.
    paintPaving(g, townPaving(world), W, D);
    g.lineCap = 'round';
    // Farm fields on farm lots: soil in furrows.
    for (const f of meta.world?.fields ?? []) {
      g.fillStyle = MAP.soil;
      g.fillRect(f.x, f.z, f.w, f.d);
      g.fillStyle = MAP.furrow;
      const along = f.w >= f.d;
      for (let k = 0.5; k < (along ? f.d : f.w); k += 1.2) (along ? g.fillRect(f.x, f.z + k, f.w, 0.45) : g.fillRect(f.x + k, f.z, 0.45, f.d));
    }
    // Paths, then the roads: their pavements (a road's width has them), then their carriageways, trails as paths.
    for (const p of meta.paths ?? []) {
      g.fillStyle = PATH_LOOK[pathLook(p.look)].map;
      g.fillRect(p.x, p.z, p.w, p.d);
    }
    const stroke = (r: RoadPlan, width: number, colour: string) => {
      g.strokeStyle = colour;
      g.lineWidth = width;
      g.beginPath();
      r.points.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
      g.stroke();
    };
    for (const r of roads) if (hasSidewalks(r)) stroke(r, r.width, MAP.pavement);
    // A bridge's stone deck, wider than its road (`street.ts` `bridge`), under its asphalt: from up here its
    // parapets are lost, the deck over the water isn't.
    // (Square at its ends: the banks are where its stone stops.)
    g.lineCap = 'butt';
    for (const r of roads) for (const [s0, s1] of bridgeSpans(r)) stroke({ ...r, points: stretchOf(r, s0, s1) }, r.width + 2 * BRIDGE.apron, MAP.pavement);
    g.lineCap = 'round';
    for (const r of roads) if (r.kind === 'trail') stroke(r, r.width, MAP.path);
    for (const r of roads) if (r.kind !== 'trail') stroke(r, carriageway(r), MAP.asphalt);
    // The junctions' pads: where the roads' ends meet, asphalt as wide as the widest carriageway there.
    g.fillStyle = MAP.asphalt;
    for (const j of world.junctions ?? []) {
      let half = 0;
      for (const r of roads) for (const [x, z] of [r.points[0], r.points[r.points.length - 1]]) if (Math.hypot(x - j.x, z - j.z) < 2) half = Math.max(half, carriageway(r) / 2);
      if (!half) continue;
      g.beginPath();
      g.arc(j.x, j.z, half + 1, 0, Math.PI * 2);
      g.fill();
    }
    // The avenues' medians, green, cut at their junctions; the motorways' barriers.
    g.lineCap = 'butt';
    const plans = planJunctions(world.junctions ?? [], roads);
    const cut = (r: RoadPlan, list: readonly [number, number][]) => list.map(([a, b]) => ({ ...r, points: stretchOf(r, a, b) }));
    for (const r of roads) {
      const m = kindInfo(r).median;
      if (!m || r.points.length < 2) continue;
      const total = lengths(r.points).at(-1)!;
      for (const piece of cut(r, without([[0, total]], junctionStretches(r, plans)))) stroke(piece, m, MAP.median);
    }
    const barriers = (only?: (r: RoadPlan, s: number) => boolean) => {
      for (const { a, runs } of allBarriers(roads)) {
        for (const run of runs) {
          g.strokeStyle = MAP.concrete;
          g.lineWidth = 0.6;
          g.beginPath();
          let on = false;
          for (const p of run) {
            if (only && !only(roads[a], p.sa)) {
              on = false;
              continue;
            }
            if (on) g.lineTo(p.x, p.z);
            else g.moveTo(p.x, p.z);
            on = true;
          }
          g.stroke();
        }
      }
    };
    barriers();
    // The overpasses last, over what they pass over: their concrete wider than their road, its asphalt, the barrier on them.
    const decks = new Map(roads.flatMap((r) => {
      const spans = raisedSpans(r);
      return spans.length ? [[r, spans] as const] : [];
    }));
    for (const [r, spans] of decks) for (const piece of cut(r, spans)) stroke(piece, r.width + 2 * OVERPASS.apron, MAP.concrete);
    for (const [r, spans] of decks) for (const piece of cut(r, spans)) stroke(piece, carriageway(r), MAP.asphalt);
    if (decks.size) barriers((r, s) => !!decks.get(r)?.some(([a, b]) => s >= a && s <= b));
    g.lineCap = 'round';
    // Premultiplied (so the mips stay true at an edge), up to the GPU with its mips.
    const data = premultiplied(g.getImageData(0, 0, size, size).data);
    this.texture?.destroy();
    const texture = new pc.Texture(this.device, { name: 'townGroundMap', width: size, height: size, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [data] });
    this.texture = texture;
    this.area.set([0, 0, 1 / W, 1 / D]);
  }

  /** Whether it is laid over the ground now. */
  get active(): boolean {
    return this.on;
  }

  /** Laid over `materials` (the ground tiles') by `blend` (0: not at all, and not in their shader either). */
  apply(materials: Iterable<pc.StandardMaterial>, blend: number): void {
    const on = blend > 0 && !!this.texture;
    for (const m of materials) {
      if (on !== this.on) {
        m.setDefine('GAME_TOWN_GROUND', on ? '' : false);
        m.update();
      }
      if (on) {
        m.setParameter('game_townMap', this.texture!);
        m.setParameter('game_townMapArea', this.area);
        m.setParameter('game_townGround', blend);
      }
    }
    this.on = on;
  }

  dispose(): void {
    this.texture?.destroy();
    this.texture = null;
    this.key = '';
  }
}
