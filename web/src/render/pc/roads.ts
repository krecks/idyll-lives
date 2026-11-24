/**
 * The roads that connect a town with the world outside, as PlayCanvas draws them: the road
 * surfaces, their centre lines and a "Welcome to …" sign by the way into town; a motorway's barrier, and its deck where
 * it goes on over the water (plainly: a slab, a parapet on its outer edge, a pier now and then). Where they go (and
 * the side roads of a town of several streets) is `shared/countryRoads.ts`'s; the landscape
 * (`nature.ts`) levels the ground under them and hands them here (`Landscape.build`'s
 * `buildRoads`).
 */

import * as pc from './engine';
import { GAME_FRONT_FACE, MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import { LIFT, STEP, STRAIGHT, type CountryRoads, type Point } from '../shared/countryRoads';
import { Geo, type V3 } from '../shared/geometry';
import { asphaltVary, carriageway, tint, STREET_LOOK } from '../shared/paving';
import { BARRIER_CONCRETE, BARRIER_FINISH, MOTORWAY } from '../shared/motorway';

/** The stretches (`STEP` m each) over which a road out of town goes from the junction's even tone to its own wear. */
const JOIN_FADE = 3;
/**
 * A motorway's deck over the water past the edge, plainly (m): its slab this deep under the road, reaching this far
 * past its outer edge; a parapet there this high and wide; a pier every this many stretches (`STEP` m each), this wide,
 * down into the bed under the water.
 */
const DECK = { slab: 1.1, apron: 0.5, parapet: 1.0, wall: 0.3, every: 6, pier: 1.2, sink: 0.5 } as const;

/** The road surfaces, their centre lines and the welcome signs of `roads`, under `parent`. */
export function drawRoads(app: pc.AppBase, roads: CountryRoads, parent: pc.Entity): pc.Entity[] {
  const lib = MaterialLibrary.for(app);
  const device = app.graphicsDevice;
  const surface = new Geo();
  const lines = new Geo().color('#E8E2CC');
  const out: pc.Entity[] = [];
  const quad = (g: Geo, p: V3[]) => g.poly(p, [0, 1, 0]);

  /**
   * A stretch of asphalt worn as the town's streets are (`shared/paving.ts`): strips across it, pale where
   * wheels run, darker between them and at its edges. `at`: the point `t` along it, `off` m from its middle.
   * `even`: how much of the wear is evened out to the one tone the streets have where they meet
   * (`STREET_LOOK.junction`), at `t` along it (0: worn; 1: even).
   */
  const worn = (at: (t: number, off: number) => V3, half: number, even: (t: number) => number = () => 0) => {
    const band = carriageway(half);
    const offs = [-half, ...band.at.filter((o) => Math.abs(o) < half - 0.05), half];
    const tone = (t: number, off: number): V3 => {
      const [x, , z] = at(t, off);
      const e = even(t);
      return tint((band.tone(off) * (1 - e) + STREET_LOOK.junction * e) * asphaltVary(x, z));
    };
    for (let o = 0; o + 1 < offs.length; o++) {
      const c: [number, number][] = [[0, offs[o + 1]], [1, offs[o + 1]], [1, offs[o]], [0, offs[o]]];
      surface.poly(c.map(([t, off]) => at(t, off)), [0, 1, 0], undefined, undefined, c.map(([t, off]) => tone(t, off)));
    }
  };

  for (const r of roads.links) {
    const y = 0.012;
    const rows = Math.max(1, Math.round(r.d / 4));
    // (One even tone, as the streets have where they meet (`STREET_LOOK.junction`): a link lies over the street it
    // joins, and worn its own way it lay there like a slab.)
    for (let k = 0; k < rows; k++) worn((t, off) => [r.x + r.w / 2 + off, y, r.z + ((k + t) / rows) * r.d], r.w / 2, () => 1);
    const xc = r.x + r.w / 2;
    for (let z = r.z + 1; z + 2.4 < r.z + r.d - 1; z += 5) quad(lines, [[xc - 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z + 2.4], [xc - 0.06, y + 0.005, z + 2.4]]);
  }
  for (const [ri, road] of roads.roads.entries()) {
    // Each point's sideways direction (across the road), from its neighbours.
    const side = road.map((_, i) => {
      const [ax, az] = road[Math.max(0, i - 1)];
      const [bx, bz] = road[Math.min(road.length - 1, i + 1)];
      const l = Math.hypot(bx - ax, bz - az) || 1;
      return [-(bz - az) / l, (bx - ax) / l];
    });
    const edge = (i: number, s: number, y: number, half = road[i][3] / 2): V3 => [road[i][0] + side[i][0] * half * s, road[i][2] + y, road[i][1] + side[i][1] * half * s];
    for (let i = 0; i + 1 < road.length; i++) {
      // (From the town's end, where it meets a link in the one even tone, it comes into its own wear over
      // `JOIN_FADE` stretches: worn right up to the link, it met it at a straight step of tone.)
      const joined = (t: number) => 1 - Math.min(1, (i + t) / JOIN_FADE) ** 2 * (3 - 2 * Math.min(1, (i + t) / JOIN_FADE));
      worn(
        (t, off) => {
          const [a, b] = [edge(i, 1, LIFT, -off), edge(i + 1, 1, LIFT, -off)];
          return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        },
        Math.min(road[i][3], road[i + 1][3]) / 2,
        joined,
      );
      // A dash along the middle of every stretch (2.4 m of each 4).
      const [ax, az, ah] = road[i];
      const [bx, bz, bh] = road[i + 1];
      const t = 2.4 / STEP;
      const [cx, cz, ch] = [ax + (bx - ax) * t, az + (bz - az) * t, ah + (bh - ah) * t];
      const [sx, sz] = side[i].map((v) => v * 0.06);
      const y = LIFT + 0.006;
      quad(lines, [[ax + sx, ah + y, az + sz], [cx + sx, ch + y, cz + sz], [cx - sx, ch + y, cz - sz], [ax - sx, ah + y, az - sz]]);
    }
    // (A motorway's carriageway: no sign of its own.)
    const sign = roads.motorways.has(ri) ? null : welcomeSign(app, roads.name, road, parent);
    if (sign) out.push(sign);
  }
  // The motorway's barrier between its carriageways, plainly: a concrete strip as high as the town's.
  const barrier = new Geo().color(BARRIER_CONCRETE, BARRIER_FINISH);
  for (const line of roads.barriers) {
    for (let i = 0; i + 1 < line.length; i++) {
      const [[ax, az, ah, w], [bx, bz, bh]] = [line[i], line[i + 1]];
      const l = Math.hypot(bx - ax, bz - az) || 1;
      const [sx, sz] = [(-(bz - az) / l) * (w / 2), ((bx - ax) / l) * (w / 2)];
      const [tx, tz] = [sx * (MOTORWAY.top / MOTORWAY.base), sz * (MOTORWAY.top / MOTORWAY.base)];
      const [ya, yb] = [ah + LIFT, bh + LIFT];
      const [ha, hb] = [ya + MOTORWAY.high, yb + MOTORWAY.high];
      for (const k of [1, -1]) quad(barrier, [[ax + sx * k, ya, az + sz * k], [bx + sx * k, yb, bz + sz * k], [bx + tx * k, hb, bz + tz * k], [ax + tx * k, ha, az + tz * k]]);
      quad(barrier, [[ax + tx, ha, az + tz], [bx + tx, hb, bz + tz], [bx - tx, hb, bz - tz], [ax - tx, ha, az - tz]]);
    }
  }
  // A motorway's deck over the water: its slab, its parapet on the outer edge, its piers (in the barrier's concrete).
  for (const [ri, { on, bed }] of roads.decks) {
    const road = roads.roads[ri];
    const other = roads.roads[roads.pairs.get(ri) ?? -1];
    for (let i = 0; i + 1 < road.length; i++) {
      if (!on[i] && !on[i + 1]) continue;
      const [[ax, az, ah, w], [bx, bz, bh]] = [road[i], road[i + 1]];
      const l = Math.hypot(bx - ax, bz - az) || 1;
      const [fx, fz] = [(bx - ax) / l, (bz - az) / l];
      // (Across it, towards its outer edge: away from its pair.)
      let [sx, sz] = [-fz, fx];
      if (other?.[i] && (other[i][0] - ax) * sx + (other[i][1] - az) * sz > 0) [sx, sz] = [-sx, -sz];
      // (A point `t` (0, 1) along the stretch, `off` m across it towards its outer edge, `y` over its road.)
      const at = (t: 0 | 1, off: number, y: number): V3 => {
        const [x, z, h] = t === 0 ? [ax, az, ah] : [bx, bz, bh];
        return [x + sx * off, h + y, z + sz * off];
      };
      const [inner, outer, top] = [-w / 2, w / 2 + DECK.apron, LIFT - 0.004];
      // The slab: its outer side, its underside (its inner side meets its pair's), its ends where the deck ends.
      barrier.poly([at(0, outer, top), at(1, outer, top), at(1, outer, -DECK.slab), at(0, outer, -DECK.slab)], [sx, 0, sz]);
      barrier.poly([at(0, outer, -DECK.slab), at(1, outer, -DECK.slab), at(1, inner, -DECK.slab), at(0, inner, -DECK.slab)], [0, -1, 0]);
      for (const [t, end, k] of [[0, i > 0 && !on[i - 1], -1], [1, i + 2 < road.length && !on[i + 2], 1]] as const) {
        if (end) barrier.poly([at(t, inner, top), at(t, outer, top), at(t, outer, -DECK.slab), at(t, inner, -DECK.slab)], [fx * k, 0, fz * k]);
      }
      // The parapet on its outer edge: its two faces and its top.
      const wall = outer - DECK.wall;
      barrier.poly([at(0, outer, top), at(1, outer, top), at(1, outer, DECK.parapet), at(0, outer, DECK.parapet)], [sx, 0, sz]);
      barrier.poly([at(0, wall, top), at(1, wall, top), at(1, wall, DECK.parapet), at(0, wall, DECK.parapet)], [-sx, 0, -sz]);
      barrier.poly([at(0, wall, DECK.parapet), at(1, wall, DECK.parapet), at(1, outer, DECK.parapet), at(0, outer, DECK.parapet)], [0, 1, 0]);
      // Now and then a pier under the slab's middle, down into the bed.
      const [y0, y1] = [bed[i] - DECK.sink, ah - DECK.slab];
      if (i % DECK.every !== DECK.every / 2 || !on[i] || y1 <= y0) continue;
      const mid = (inner + outer) / 2;
      const [cx, cz, r] = [ax + sx * mid, az + sz * mid, DECK.pier / 2];
      const c = (u: number, v: number, y: number): V3 => [cx + fx * u + sx * v, y, cz + fz * u + sz * v];
      for (const [u, v, nu, nv] of [[r, 0, 1, 0], [-r, 0, -1, 0], [0, r, 0, 1], [0, -r, 0, -1]]) {
        const [du, dv] = [nv * r, nu * r];
        barrier.poly([c(u - du, v - dv, y1), c(u + du, v + dv, y1), c(u + du, v + dv, y0), c(u - du, v - dv, y0)], [fx * nu + sx * nv, 0, fz * nu + sz * nv]);
      }
    }
  }
  const barrierMesh = meshFrom(device, barrier.meshData({ colors: true }));
  if (barrierMesh) out.push(drawMesh(parent, 'countryBarriers', barrierMesh, lib.plainFinish()));
  const surfaceMesh = meshFrom(device, surface.meshData({ colors: true }));
  if (surfaceMesh) out.push(drawMesh(parent, 'countryRoads', surfaceMesh, lib.surface('material.road', { doubleSided: true, vertexColors: true })));
  const marks = meshFrom(device, lines.meshData({ colors: true }));
  if (marks) out.push(drawMesh(parent, 'countryRoadLines', marks, lib.finish('material.finish.matte')));
  return out;
}

/** The welcome signs' lettering, per app and town name (drawn once). */
const lettering = new WeakMap<pc.AppBase, Map<string, pc.StandardMaterial>>();

/** A "Welcome to …" sign on the right of the road into town, a little way out, facing arrivals. */
function welcomeSign(app: pc.AppBase, name: string, road: Point[], parent: pc.Entity): pc.Entity | null {
  const i = Math.min(road.length - 1, STRAIGHT);
  if (!name || i < 1) return null;
  const [x, z, h, width] = road[i];
  // Arrivals drive towards the town (point i towards point i - 1); the sign stands on their right.
  const [bx, bz] = [road[i - 1][0] - x, road[i - 1][1] - z];
  const l = Math.hypot(bx, bz) || 1;
  const [fx, fz] = [bx / l, bz / l];
  const right = [-fz, fx];
  const off = width / 2 + 1.6;
  const [px, pz] = [x + right[0] * off, z + right[1] * off];

  const g = new Geo();
  g.color('#6B5236');
  g.box(-1.35, 0, -0.06, -1.2, 1.9, 0.06);
  g.box(1.2, 0, -0.06, 1.35, 1.9, 0.06);
  g.color('#2F5D3A');
  g.box(-1.5, 1.0, -0.05, 1.5, 2.1, 0.02);
  const device = app.graphicsDevice;
  const frameMesh = meshFrom(device, g.meshData({ colors: true }));
  if (!frameMesh) return null;
  const frame = new pc.Entity('welcomeSign');
  parent.addChild(frame);
  frame.setLocalPosition(px, h, pz);
  // (A turn about y in game space: the world root's mirror takes it into PlayCanvas's.)
  frame.setLocalEulerAngles(0, ((Math.atan2(fx, fz) + Math.PI) * 180) / Math.PI, 0);
  drawMesh(frame, 'welcomeSignFrame', frameMesh, MaterialLibrary.for(app).finish('material.finish.satin'));

  // The lettering on the board's front (+z), reading left to right from in front of it (the left
  // of someone facing it, looking along -z, is +x: the texture starts there; its top row is v = 0).
  const text = new Geo();
  const [w, ht, y0, zf] = [1.45, 0.52, 1.55, 0.03];
  text.poly([[w, y0 - ht, zf], [-w, y0 - ht, zf], [-w, y0 + ht, zf], [w, y0 + ht, zf]], [0, 0, 1], [[0, 1], [1, 1], [1, 0], [0, 0]]);
  const textMesh = meshFrom(device, text.meshData());
  if (textMesh) drawMesh(frame, 'welcomeSignText', textMesh, letteringOf(app, name));
  return frame;
}

/** The material of a sign's lettering for town `name`: the board's green, a cream border and the words, faintly lit from within. */
function letteringOf(app: pc.AppBase, name: string): pc.StandardMaterial {
  let byName = lettering.get(app);
  if (!byName) lettering.set(app, (byName = new Map()));
  const had = byName.get(name);
  if (had) return had;
  const canvas = document.createElement('canvas');
  canvas.width = 512;
  canvas.height = 192;
  const ctx = canvas.getContext('2d')!;
  ctx.fillStyle = '#2F5D3A';
  ctx.fillRect(0, 0, 512, 192);
  ctx.strokeStyle = '#F2EEE2';
  ctx.lineWidth = 6;
  ctx.strokeRect(10, 10, 492, 172);
  ctx.fillStyle = '#F2EEE2';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = '600 34px sans-serif';
  ctx.fillText('Welcome to', 256, 62);
  let size = 64;
  do ctx.font = `700 ${size}px sans-serif`;
  while (ctx.measureText(name).width > 460 && (size -= 4) > 24);
  ctx.fillText(name, 256, 124);
  // (A linear format: the material decodes it, `pow(c, 2.2)`, as it does every gamma-space colour texture; `materials.ts` `textureWithFallback`.)
  const texture = new pc.Texture(app.graphicsDevice, { name: 'welcomeSignText', width: 512, height: 192, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
  texture.setSource(canvas);
  const mat = new pc.StandardMaterial();
  mat.name = 'welcomeSignText';
  mat.frontFace = GAME_FRONT_FACE;
  mat.diffuseMap = texture;
  mat.useMetalness = true;
  mat.metalness = 0;
  mat.gloss = 0.2;
  // (0.25 in linear light.)
  const e = Math.pow(0.25, 1 / 2.2);
  mat.emissive.set(e, e, e);
  MaterialLibrary.for(app).adopt(mat);
  byName.set(name, mat);
  return mat;
}
