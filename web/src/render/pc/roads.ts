/**
 * The roads that connect a town with the world outside, as PlayCanvas draws them (the port of
 * `babylon/roads.ts`): the road surfaces, their centre lines and a "Welcome to …" sign by the way
 * into town. Where they go (and the side roads of a town of several streets) is
 * `shared/countryRoads.ts`'s; the landscape (`nature.ts`) levels the ground under them and hands
 * them here (`Landscape.build`'s `buildRoads`).
 */

import * as pc from './engine';
import { GAME_FRONT_FACE, MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import { LIFT, STEP, STRAIGHT, type CountryRoads, type Point } from '../shared/countryRoads';
import { Geo, type V3 } from '../shared/geometry';

/** The road surfaces, their centre lines and the welcome signs of `roads`, under `parent`. */
export function drawRoads(app: pc.AppBase, roads: CountryRoads, parent: pc.Entity): pc.Entity[] {
  const lib = MaterialLibrary.for(app);
  const device = app.graphicsDevice;
  const surface = new Geo();
  const lines = new Geo().color('#E8E2CC');
  const out: pc.Entity[] = [];
  const quad = (g: Geo, p: V3[]) => g.poly(p, [0, 1, 0]);

  for (const r of roads.links) {
    const y = 0.012;
    quad(surface, [[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]]);
    const xc = r.x + r.w / 2;
    for (let z = r.z + 1; z + 2.4 < r.z + r.d - 1; z += 5) quad(lines, [[xc - 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z + 2.4], [xc - 0.06, y + 0.005, z + 2.4]]);
  }
  for (const road of roads.roads) {
    // Each point's sideways direction (across the road), from its neighbours.
    const side = road.map((_, i) => {
      const [ax, az] = road[Math.max(0, i - 1)];
      const [bx, bz] = road[Math.min(road.length - 1, i + 1)];
      const l = Math.hypot(bx - ax, bz - az) || 1;
      return [-(bz - az) / l, (bx - ax) / l];
    });
    const edge = (i: number, s: number, y: number, half = road[i][3] / 2): V3 => [road[i][0] + side[i][0] * half * s, road[i][2] + y, road[i][1] + side[i][1] * half * s];
    for (let i = 0; i + 1 < road.length; i++) {
      quad(surface, [edge(i, 1, LIFT), edge(i + 1, 1, LIFT), edge(i + 1, -1, LIFT), edge(i, -1, LIFT)]);
      // A dash along the middle of every stretch (2.4 m of each 4).
      const [ax, az, ah] = road[i];
      const [bx, bz, bh] = road[i + 1];
      const t = 2.4 / STEP;
      const [cx, cz, ch] = [ax + (bx - ax) * t, az + (bz - az) * t, ah + (bh - ah) * t];
      const [sx, sz] = side[i].map((v) => v * 0.06);
      const y = LIFT + 0.006;
      quad(lines, [[ax + sx, ah + y, az + sz], [cx + sx, ch + y, cz + sz], [cx - sx, ch + y, cz - sz], [ax - sx, ah + y, az - sz]]);
    }
    const sign = welcomeSign(app, roads.name, road, parent);
    if (sign) out.push(sign);
  }
  const surfaceMesh = meshFrom(device, surface.meshData());
  if (surfaceMesh) out.push(drawMesh(parent, 'countryRoads', surfaceMesh, lib.surface('material.road', { doubleSided: true })));
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
  // (A turn about y in game space, as Babylon's `rotation.y`: the world root's mirror takes it into PlayCanvas's.)
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
  const texture = new pc.Texture(app.graphicsDevice, { name: 'welcomeSignText', width: 512, height: 192, format: pc.PIXELFORMAT_SRGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
  texture.setSource(canvas);
  const mat = new pc.StandardMaterial();
  mat.name = 'welcomeSignText';
  mat.frontFace = GAME_FRONT_FACE;
  mat.diffuseMap = texture;
  mat.useMetalness = true;
  mat.metalness = 0;
  mat.gloss = 0.2;
  // (Babylon's 0.25, linear.)
  const e = Math.pow(0.25, 1 / 2.2);
  mat.emissive.set(e, e, e);
  MaterialLibrary.for(app).adopt(mat);
  byName.set(name, mat);
  return mat;
}
