/**
 * The roads that connect a town with the world outside, as Babylon draws them: the road surfaces,
 * their centre lines and a "Welcome to …" sign by the way into town. Where they go (and the side
 * roads of a town of several streets) is `shared/countryRoads.ts`'s.
 *
 * Visual only and outside the town (residents leave town at the ends of its streets): the
 * landscape (`nature.ts`) levels the ground under the roads, keeps trees off them and draws them.
 */

import { Color3, DynamicTexture, Mesh, MeshBuilder, StandardMaterial, type Scene } from './core';
import { CountryRoads as RoadLayout, LIFT, STEP, STRAIGHT, type Point } from '../shared/countryRoads';
import { Geo, type V3 } from '../shared/geometry';
import { toMesh } from './meshes';
import { MaterialLibrary } from './materials';

export { ROAD_VERGE, type NearRoad, type RoadLand } from '../shared/countryRoads';

/** The country roads as Babylon draws them (their layout: `shared/countryRoads.ts`). */
export class CountryRoads extends RoadLayout {
  /** The road surfaces, their centre lines and the welcome signs. */
  build(scene: Scene): Mesh[] {
    const lib = MaterialLibrary.for(scene);
    const surface = new Geo();
    const lines = new Geo().color('#E8E2CC');
    const out: Mesh[] = [];
    const quad = (g: Geo, p: V3[]) => g.poly(p, [0, 1, 0]);

    for (const r of this.links) {
      const y = 0.012;
      quad(surface, [[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]]);
      const xc = r.x + r.w / 2;
      for (let z = r.z + 1; z + 2.4 < r.z + r.d - 1; z += 5) quad(lines, [[xc - 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z], [xc + 0.06, y + 0.005, z + 2.4], [xc - 0.06, y + 0.005, z + 2.4]]);
    }
    for (const road of this.roads) {
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
      const sign = this.sign(scene, road);
      if (sign) out.push(...sign);
    }
    const roads = toMesh(surface, 'countryRoads', scene, { colors: false });
    if (roads) {
      roads.material = lib.surface('material.road', { doubleSided: true });
      roads.receiveShadows = true;
      out.push(roads);
    }
    const marks = toMesh(lines, 'countryRoadLines', scene, { colors: true });
    if (marks) {
      marks.material = lib.finish('material.finish.matte');
      marks.receiveShadows = true;
      out.push(marks);
    }
    for (const m of out) m.isPickable = false;
    return out;
  }

  /** A "Welcome to …" sign on the right of the road into town, a little way out, facing arrivals. */
  private sign(scene: Scene, road: Point[]): Mesh[] | null {
    const i = Math.min(road.length - 1, STRAIGHT);
    if (!this.name || i < 1) return null;
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
    const frame = toMesh(g, 'welcomeSign', scene, { colors: true });
    if (!frame) return null;
    frame.material = MaterialLibrary.for(scene).finish('material.finish.satin');

    const tex = new DynamicTexture('welcomeSignText', { width: 512, height: 192 }, scene, true);
    const ctx = tex.getContext() as CanvasRenderingContext2D;
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
    while (ctx.measureText(this.name).width > 460 && (size -= 4) > 24);
    ctx.fillText(this.name, 256, 124);
    tex.update();
    const mat = new StandardMaterial('welcomeSignText', scene);
    mat.diffuseTexture = tex;
    mat.specularColor = Color3.Black();
    mat.emissiveColor = new Color3(0.25, 0.25, 0.25);
    const text = MeshBuilder.CreatePlane('welcomeSignText', { width: 2.9, height: 1.04 }, scene);
    text.material = mat;
    text.parent = frame;
    // The plane faces -z; the board's front is +z.
    text.rotation.y = Math.PI;
    text.position.set(0, 1.55, 0.03);

    frame.position.set(px, h, pz);
    frame.rotation.y = Math.atan2(fx, fz) + Math.PI;
    frame.receiveShadows = true;
    text.isPickable = false;
    return [frame, text];
  }
}
