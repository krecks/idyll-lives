/**
 * Step 2's edits over the town before it is built again with them (`Renderer.setTownGhosts`): streets as ribbons
 * (a soft fill between two bright edges; red where they may not go; a bridge stretch pale, with its deck's planks
 * across it), lots as outlines over a faint fill (new, red where one may not go; one to be removed grey, with its
 * building dimmed under a dark veil), and villages' boxes as dashed bands. Everything is draped over the land (and
 * over water, where a bridge would be) `LIFT` m up, unlit and clear of the fog, in one mesh written again in place
 * on every call: a few thousand vertices, well under 2 ms. An entity under the world root (game space), on the
 * layer the camera draws the town with.
 *
 * From far out (the whole world's shot, a metre or so a pixel) a street's ghost was under a pixel or two across and its
 * edges and a lot's outline less: they are drawn no thinner on screen than `MIN_HALF_PX` and `MIN_LINE_PX`, by the
 * scale the camera gives (`setScale`), in steps so a zoom draws them again only now and then.
 */

import * as pc from './engine';
import { GAME_FRONT_FACE } from './materials';
import { MeshBuffers } from './meshBuffers';
import type { GhostRoadTone, TownGhosts } from '../types';
import type { WorldStructure } from '../../core/protocol';

/** How high over the ground the ghosts hang (m). */
const LIFT = 0.18;
/** The longest piece (m) a ghost's edge is drawn in, so it follows the land. */
const STEP = 2;
/** The largest cell (m) a ghost's fill is drawn in. */
const CELL = 4;
/** A village's box is filled in cells this large (m): it can be a few hundred metres across. */
const BOX_CELL = 8;
/** A street's edges: this wide (m), inside its width. */
const EDGE = 0.45;
/** A lot's outline: this far in from its edges (m), and this wide. */
const LOT_INSET = 0.35;
const LOT_LINE = 0.5;
/** A village's box: its band's width (m), and its dashes' and gaps' length. */
const BOX_LINE = 1.4;
const DASH = 6;
/** A bridge's planks: one every this far (m) along it, this wide. */
const PLANK_EVERY = 3;
const PLANK = 0.5;
/** The fewest CSS px a street's ghost is across either side of its middle, and an edge, outline or band is wide. */
const MIN_HALF_PX = 5;
const MIN_LINE_PX = 1.5;
/** The scale (m a CSS px) goes in steps of this ratio, rounded up, and under `SCALE_FROM` is none (nothing near that thin). */
const SCALE_STEP = Math.SQRT2;
const SCALE_FROM = 0.1;
/** A storey's height (m), and a roof's over the top storey, for the veil over a building to be removed. */
const STOREY = 3;
const ROOF = 3.2;

/**
 * The tones (sRGB) and their strengths: a fill's alpha and an edge's (a fill of 0 is left out). `laid`: a new street
 * drawn as the road it will be, near asphalt; `pick`: the road picked, its outline only; `join`: the road a street
 * being drawn will join.
 */
const TONES = {
  new: { rgb: [143, 211, 255], fill: 0.28, edge: 0.95 },
  bad: { rgb: [239, 83, 80], fill: 0.34, edge: 1 },
  bridge: { rgb: [242, 226, 184], fill: 0.4, edge: 1 },
  gone: { rgb: [150, 156, 166], fill: 0.42, edge: 0.85 },
  laid: { rgb: [78, 82, 88], fill: 0.88, edge: 0.95 },
  pick: { rgb: [143, 211, 255], fill: 0, edge: 1 },
  join: { rgb: [93, 187, 99], fill: 0.3, edge: 1 },
} as const satisfies Record<GhostRoadTone | TownGhosts['lots'][number]['tone'], unknown>;
type Tone = keyof typeof TONES;
/** The veil over a building to be removed: dark, half see-through. */
const VEIL = { rgb: [24, 27, 32], alpha: 0.55 } as const;

/** Where the ghosts stand: the ground (m) at a town point (its water's top where it is wet). */
export type Drape = (x: number, z: number) => number;
/**
 * The surface of the town's road along a ghost's line, if the town has that road (`shared/roads.ts` `roadHeightsOf`):
 * its height `s` m along it. A road's ghost lies on it where it is higher than the ground (on a deck); else on the ground.
 */
export type RoadHeights = (points: readonly (readonly [number, number])[]) => ((s: number) => number) | null;

/** A polyline cut into pieces no longer than `step` (m); `closed`: back to its first point. */
function resample(points: readonly (readonly [number, number])[], step: number, closed = false): [number, number][] {
  const out: [number, number][] = [];
  const n = points.length;
  if (!n) return out;
  const last = closed ? n : n - 1;
  for (let k = 0; k < last; k++) {
    const [ax, az] = points[k];
    const [bx, bz] = points[(k + 1) % n];
    const pieces = Math.max(1, Math.ceil(Math.hypot(bx - ax, bz - az) / step));
    for (let i = 0; i < pieces; i++) out.push([ax + ((bx - ax) * i) / pieces, az + ((bz - az) * i) / pieces]);
  }
  if (!closed) out.push([points[n - 1][0], points[n - 1][1]]);
  return out;
}

/** Each point's way across (a unit normal, its left, scaled at a bend so a band keeps its width; closed: round the corner). */
function normals(path: readonly [number, number][], closed: boolean): [number, number][] {
  const n = path.length;
  const seg = (i: number): [number, number] => {
    const [a, b] = [path[i], path[(i + 1) % n]];
    const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1;
    return [-(b[1] - a[1]) / l, (b[0] - a[0]) / l];
  };
  return path.map((_, i) => {
    const before = i > 0 ? seg(i - 1) : closed ? seg(n - 1) : seg(0);
    const after = i < n - 1 ? seg(i) : closed ? seg(i) : seg(n - 2);
    let [nx, nz] = [before[0] + after[0], before[1] + after[1]];
    const l = Math.hypot(nx, nz);
    if (l < 1e-6) return before;
    [nx, nz] = [nx / l, nz / l];
    // (A mitre: the band as wide at the bend as along it, but not past twice that at a sharp one.)
    const k = Math.min(2, 1 / Math.max(0.5, nx * after[0] + nz * after[1]));
    return [nx * k, nz * k];
  });
}

export class TownGhostDraw {
  private readonly entity: pc.Entity;
  private readonly mesh: pc.Mesh;
  private readonly material: pc.StandardMaterial;
  private readonly mi: pc.MeshInstance;
  private readonly buf = new MeshBuffers(4096);
  /** Its buffers are made for 32-bit indices. */
  private wide = false;
  /** M a CSS px where the camera looks, in `SCALE_STEP` steps (0: near enough that no ghost is drawn wider for it). */
  private scale = 0;

  constructor(app: pc.AppBase, parent: pc.Entity) {
    const mesh = (this.mesh = new pc.Mesh(app.graphicsDevice));
    // (Written again on every call, in place while it fits.)
    mesh.clear(true, true, 4096, 6 * 4096);
    // (Its buffers made now, with their colours: a mesh instance reads what its mesh's vertices carry when it is made.)
    mesh.setPositions(new Float32Array(3), 3, 1);
    mesh.setColors32(new Uint8Array(4), 1);
    mesh.setIndices(new Uint32Array(3), 3);
    mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const mat = (this.material = new pc.StandardMaterial());
    mat.name = 'townGhosts';
    mat.frontFace = GAME_FRONT_FACE;
    mat.useLighting = false;
    mat.useFog = false;
    mat.useSkybox = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.emissiveVertexColor = true;
    // (The tones are sRGB: linearised.)
    (mat as pc.StandardMaterial & { vertexColorGamma: boolean }).vertexColorGamma = true;
    mat.opacityVertexColor = true;
    mat.opacityVertexColorChannel = 'a';
    mat.blendType = pc.BLEND_NORMAL;
    mat.depthWrite = false;
    mat.cull = pc.CULLFACE_NONE;
    // (In front of the ground and the road it lies on.)
    mat.slopeDepthBias = -4;
    mat.depthBias = -2;
    mat.update();
    this.mi = new pc.MeshInstance(mesh, mat);
    // (Its box is made again as it changes; never left out by an old one.)
    this.mi.cull = false;
    this.entity = new pc.Entity('townGhosts', app);
    this.entity.addComponent('render', { meshInstances: [this.mi], castShadows: false, receiveShadows: false });
    this.entity.enabled = false;
    parent.addChild(this.entity);
  }

  /**
   * How many m a CSS px is where the camera looks (from its distance, lens and the canvas's height). True if that
   * crossed a step: the ghosts are to be drawn again (`set`) for their least widths on screen.
   */
  setScale(metresPerPx: number): boolean {
    const scale = metresPerPx < SCALE_FROM ? 0 : SCALE_STEP ** Math.ceil(Math.log(metresPerPx) / Math.log(SCALE_STEP) - 1e-9);
    if (scale === this.scale) return false;
    this.scale = scale;
    return true;
  }

  /** A width (m) no less than `px` CSS px on screen. */
  private least(m: number, px: number): number {
    return Math.max(m, px * this.scale);
  }

  /**
   * Draws `ghosts` (null: none) draped over `drape`, on `layer`. `world`: the town shown, whose building on a lot to be
   * removed is dimmed. `heights`: its roads' surfaces, a road's ghost lying on its deck where it has one (the motorway
   * picked: its outline was on the ground under its overpass, showing through it). Cheap: every pointer move.
   */
  set(ghosts: TownGhosts | null, drape: Drape, layer: number, world: WorldStructure | null, heights?: RoadHeights | null): void {
    const empty = !ghosts || (!ghosts.roads.length && !ghosts.lots.length && !ghosts.boxes.length);
    if (empty) {
      this.entity.enabled = false;
      return;
    }
    const buf = this.buf;
    buf.reset();
    for (const box of ghosts.boxes) this.box(box, box.tone, drape);
    for (const lot of ghosts.lots) {
      this.lot(lot, lot.tone, drape);
      if (lot.tone === 'gone' && world) this.veil(lot, drape, world);
    }
    for (const road of ghosts.roads) this.road(road.points, road.width, road.tone, drape, heights?.(road.points) ?? null);
    const mesh = this.mesh;
    // (Past 65,535 vertices its indices need 32 bits: its buffers made again, larger, once.)
    if (buf.nv > 65535 && !this.wide) {
      this.wide = true;
      mesh.clear(true, true, buf.nv * 2, buf.ni * 2);
    }
    mesh.setPositions(buf.pos, 3, buf.nv);
    mesh.setColors32(buf.col, buf.nv);
    mesh.setIndices(buf.idx, buf.ni);
    mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const render = this.entity.render!;
    if (render.layers.length !== 1 || render.layers[0] !== layer) render.layers = [layer];
    this.entity.enabled = buf.ni > 0;
  }

  destroy(): void {
    this.entity.destroy();
    this.mesh.destroy();
    this.material.destroy();
  }

  // ---- what each ghost is made of ----------------------------------------------------------------

  /**
   * A band along `path` (resampled, with its `normals`) from `a` to `b` m across it (its left positive), `alpha`
   * strong; `skip(i)`: leave out the piece from point `i` to the next (a dash's gap). `ys`: at least this high (m) at
   * each point, across it (a road's surface along it).
   */
  private band(path: readonly [number, number][], ns: readonly [number, number][], a: number, b: number, rgb: readonly number[], alpha: number, drape: Drape, closed: boolean, skip?: (i: number) => boolean, ys?: readonly number[]): void {
    const buf = this.buf;
    const n = path.length;
    buf.reserve(n * 2 + 2, n * 6);
    const first = buf.nv;
    for (let i = 0; i < n; i++) {
      const [x, z] = path[i];
      const [nx, nz] = ns[i];
      for (const off of [a, b]) {
        const [px, pz] = [x + nx * off, z + nz * off];
        buf.vertex(px, Math.max(drape(px, pz), ys?.[i] ?? -Infinity) + LIFT, pz, rgb, alpha);
      }
    }
    const pieces = closed ? n : n - 1;
    for (let i = 0; i < pieces; i++) {
      if (skip?.(i)) continue;
      const j = (i + 1) % n;
      buf.quad(first + i * 2, first + j * 2, first + j * 2 + 1, first + i * 2 + 1);
    }
  }

  /** A rectangle's fill: a grid of cells no larger than `cell` m, draped. */
  private fill(r: { x: number; z: number; w: number; d: number }, cell: number, rgb: readonly number[], alpha: number, drape: Drape): void {
    const buf = this.buf;
    const [nx, nz] = [Math.max(1, Math.ceil(r.w / cell)), Math.max(1, Math.ceil(r.d / cell))];
    buf.reserve((nx + 1) * (nz + 1), nx * nz * 6);
    const first = buf.nv;
    for (let j = 0; j <= nz; j++) {
      for (let i = 0; i <= nx; i++) {
        const [x, z] = [r.x + (r.w * i) / nx, r.z + (r.d * j) / nz];
        buf.vertex(x, drape(x, z) + LIFT, z, rgb, alpha);
      }
    }
    for (let j = 0; j < nz; j++) {
      for (let i = 0; i < nx; i++) {
        const a = first + j * (nx + 1) + i;
        buf.quad(a, a + 1, a + nx + 2, a + nx + 1);
      }
    }
  }

  /** A street: its fill between two bright edges; a bridge's with planks across it too. `surfaceAt`: the town's road's surface `s` m along it, if it is one. */
  private road(points: readonly (readonly [number, number])[], width: number, tone: GhostRoadTone, drape: Drape, surfaceAt: ((s: number) => number) | null): void {
    if (points.length < 2) return;
    const t = TONES[tone];
    const path = resample(points, STEP);
    const ns = normals(path, false);
    const half = this.least(width / 2, MIN_HALF_PX);
    const edge = Math.min(this.least(EDGE, MIN_LINE_PX), half / 2);
    // (On the road's surface where it is up off the ground: `resample` keeps the line's corners, so its length along it is the road's.)
    let s = 0;
    const ys = surfaceAt ? path.map((p, i) => surfaceAt((s += i ? Math.hypot(p[0] - path[i - 1][0], p[1] - path[i - 1][1]) : 0))) : undefined;
    if (t.fill > 0) this.band(path, ns, -half + edge, half - edge, t.rgb, t.fill, drape, false, undefined, ys);
    this.band(path, ns, -half, -half + edge, t.rgb, t.edge, drape, false, undefined, ys);
    this.band(path, ns, half - edge, half, t.rgb, t.edge, drape, false, undefined, ys);
    if (tone !== 'bridge') return;
    // (Planks across the deck, one every `PLANK_EVERY` m along it.)
    const buf = this.buf;
    let along = 0;
    let next = PLANK_EVERY / 2;
    for (let i = 0; i + 1 < path.length; i++) {
      const [[ax, az], [bx, bz]] = [path[i], path[i + 1]];
      const l = Math.hypot(bx - ax, bz - az);
      while (next <= along + l && l > 0) {
        const f = (next - along) / l;
        const [x, z] = [ax + (bx - ax) * f, az + (bz - az) * f];
        const [ux, uz] = [(bx - ax) / l, (bz - az) / l];
        const [nx, nz] = [-uz, ux];
        buf.reserve(4, 6);
        const corners = [
          [-PLANK / 2, -half + edge],
          [PLANK / 2, -half + edge],
          [PLANK / 2, half - edge],
          [-PLANK / 2, half - edge],
        ].map(([u, v]) => {
          const [px, pz] = [x + ux * u + nx * v, z + uz * u + nz * v];
          return buf.vertex(px, drape(px, pz) + LIFT, pz, t.rgb, t.edge * 0.8);
        });
        buf.quad(corners[0], corners[1], corners[2], corners[3]);
        next += PLANK_EVERY;
      }
      along += l;
    }
  }

  /** A lot: its outline, a little in from its edges, over a faint fill. */
  private lot(r: { x: number; z: number; w: number; d: number }, tone: Tone, drape: Drape): void {
    const t = TONES[tone];
    this.fill(r, CELL, t.rgb, t.fill, drape);
    const [x0, z0, x1, z1] = [r.x + LOT_INSET, r.z + LOT_INSET, r.x + r.w - LOT_INSET, r.z + r.d - LOT_INSET];
    if (x1 <= x0 || z1 <= z0) return;
    const path = resample(
      [
        [x0, z0],
        [x1, z0],
        [x1, z1],
        [x0, z1],
      ],
      STEP,
      true,
    );
    const line = this.least(LOT_LINE, MIN_LINE_PX);
    this.band(path, normals(path, true), -line / 2, line / 2, t.rgb, t.edge, drape, true);
  }

  /** A village's box: a dashed band round it over the faintest fill. */
  private box(r: { x: number; z: number; w: number; d: number }, tone: 'new' | 'bad', drape: Drape): void {
    const t = TONES[tone];
    this.fill(r, BOX_CELL, t.rgb, t.fill * 0.4, drape);
    const path = resample(
      [
        [r.x, r.z],
        [r.x + r.w, r.z],
        [r.x + r.w, r.z + r.d],
        [r.x, r.z + r.d],
      ],
      STEP,
      true,
    );
    // (Dashes: every piece is `STEP` m or less, so a dash is a run of pieces.)
    const lengthsAt: number[] = [0];
    for (let i = 1; i < path.length; i++) lengthsAt.push(lengthsAt[i - 1] + Math.hypot(path[i][0] - path[i - 1][0], path[i][1] - path[i - 1][1]));
    const gap = (i: number) => Math.floor(lengthsAt[i] / DASH) % 2 === 1;
    const line = this.least(BOX_LINE, MIN_LINE_PX);
    this.band(path, normals(path, true), -line / 2, line / 2, t.rgb, t.edge, drape, true, gap);
  }

  /** The building on a lot to be removed, under a dark veil: a box round it up to its roof. */
  private veil(r: { x: number; z: number; w: number; d: number }, drape: Drape, world: WorldStructure): void {
    const [cx, cz] = [r.x + r.w / 2, r.z + r.d / 2];
    const plot = world.plots.find((p) => cx >= p.x && cx < p.x + p.w && cz >= p.z && cz < p.z + p.d);
    if (!plot?.house) return;
    const [x0, z0, x1, z1] = plot.house;
    const top = (plot.storeys ?? 1) * STOREY + ROOF;
    // (A little bigger than the building, so its walls are inside the veil, not through it.)
    const m = 0.3;
    const [a0, b0, a1, b1] = [x0 - m, z0 - m, x1 + m, z1 + m];
    const y0 = Math.min(drape(a0, b0), drape(a1, b0), drape(a1, b1), drape(a0, b1)) - 0.2;
    const y1 = drape((a0 + a1) / 2, (b0 + b1) / 2) + top + m;
    const buf = this.buf;
    buf.reserve(20, 30);
    const v = (x: number, y: number, z: number) => buf.vertex(x, y, z, VEIL.rgb, VEIL.alpha);
    // The four sides and the top.
    for (const [ax, az, bx, bz] of [
      [a0, b0, a1, b0],
      [a1, b0, a1, b1],
      [a1, b1, a0, b1],
      [a0, b1, a0, b0],
    ]) {
      buf.quad(v(ax, y0, az), v(bx, y0, bz), v(bx, y1, bz), v(ax, y1, az));
    }
    buf.quad(v(a0, y1, b0), v(a1, y1, b0), v(a1, y1, b1), v(a0, y1, b1));
  }
}
