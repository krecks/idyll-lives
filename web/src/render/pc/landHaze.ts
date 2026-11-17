/**
 * The haze the menus' land fades into at its edge, from far out (the whole land's shot of the new game's steps).
 * The landscape reaches `landscapeRing` m past the land and no further; from 1,300 m up the picture took in the bare
 * sky's colour past it, a straight edge down one side. Drawing the landscape further would cost the 8 GB machines
 * its tiles, woods and grass over twice the ground; this costs one draw of a few hundred triangles.
 *
 * A veil over the landscape's last `FADE` m and on past its edge, as far as the sky's dome, in the colour the sky has
 * at the horizon and below it (the haze's: `Sky.update`), clear at its inner edge and whole by the landscape's own:
 * the land goes into the haze, and past it there is only the haze, the same colour as the dome behind it. It hangs
 * `LIFT` m over the highest land under it, so the trees on the ring's hills stay under it. It shows from
 * `SHOW_FROM` m out (`strength`): nearer, the play camera's and the area shots' haze is their own.
 *
 * Hung over the land, the veil was seen against land nearer the camera than the land under it: on the far side the
 * land's last metres showed through its clearer part, and its edge stood hard against the haze past it. So each of
 * its points is moved along the line from the eye to the land it is for (`view`), to where that line is at its
 * height: from wherever the camera is, the land is seen through the veil meant for it, and fades the same on every side.
 */

import * as pc from './engine';

/** How far in from the landscape's edge the veil begins (m), and its share of the way across where it is whole. */
const FADE = 240;
/**
 * The veil's rings, in from the landscape's edge (m; negative: past it), and how clear it is at each (its alpha):
 * along a side; round a corner each the same distance from the rectangle `FADE` m in (`set`).
 */
const RINGS: readonly [number, number][] = [
  [FADE, 0],
  [FADE * 0.6, 0.3],
  [FADE * 0.25, 0.8],
  [0, 1],
  [-200, 1],
  [-6000, 1],
];
/** How far over the highest land under it the veil hangs (m). */
const LIFT = 35;
/** The land under the veil is sampled every this far along its edge (m). */
const STEP = 40;
/** Each corner's fan of ways out: this many pieces (even: the fan split mirrored about its middle). */
const ARC = 16;
/** A side's height eases to its corner's over this far from it (m). */
const EASE = 160;
/** The camera's distance (m) the veil fades in over. */
const SHOW_FROM = 650;
const SHOW_FULL = 1050;

export class LandHaze {
  private readonly material: pc.StandardMaterial;
  private readonly entity: pc.Entity;
  private mesh: pc.Mesh | null = null;
  private key = '';
  private shown = -1;
  /** Per vertex: the point on the land it is for (x, height, z; past the edge, the edge's height), and its height. */
  private ground = new Float32Array(0);
  private height = new Float32Array(0);
  private positions = new Float32Array(0);
  private colors = new Float32Array(0);
  private indices = new Uint32Array(0);
  /** The eye the veil was last moved for (`view`). */
  private readonly eye = { x: NaN, y: NaN, z: NaN };

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
  ) {
    const mat = (this.material = new pc.StandardMaterial());
    mat.name = 'landHaze';
    mat.useLighting = false;
    mat.useSkybox = false;
    mat.useFog = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.opacityVertexColor = true;
    mat.opacityVertexColorChannel = 'a';
    mat.blendType = pc.BLEND_NORMAL;
    mat.depthWrite = false;
    mat.cull = pc.CULLFACE_NONE;
    mat.opacity = 0;
    mat.update();
    this.entity = new pc.Entity('landHaze', app);
    this.entity.enabled = false;
    parent.addChild(this.entity);
  }

  /**
   * The veil round the landscape `box` (m: from x0, z0 to x1, z1, its ring included) over the land `height` has, on
   * `layer`. Made again only for another box or another land (`key`).
   */
  set(box: { x0: number; z0: number; x1: number; z1: number }, height: (x: number, z: number) => number, layer: number, key: string): void {
    const full = `${box.x0},${box.z0},${box.x1},${box.z1}|${key}|${layer}`;
    if (full === this.key) return;
    this.key = full;
    // The veil's rings are the rectangle `FADE` m in from the landscape's edge grown outwards: straight along its sides
    // and round at its corners, so each ring is the same distance from where the veil begins all the way round, and
    // its clearness goes by ring alone. (A ring as a rectangle `d` m in from the edge was a corner's way out, the
    // diagonal, stretched by √2; and its quads split all one way, a seam down the bisector of every corner.)
    const inset = Math.max(0, Math.min(FADE, (box.x1 - box.x0) / 2, (box.z1 - box.z0) / 2));
    const corners: [number, number][] = [
      [box.x0 + inset, box.z0 + inset],
      [box.x1 - inset, box.z0 + inset],
      [box.x1 - inset, box.z1 - inset],
      [box.x0 + inset, box.z1 - inset],
    ];
    // Round the inner rectangle: at each corner a fan of `ARC` ways out, then the side from it to the next corner.
    // Each station is a point on the inner rectangle and its way out; a quad runs from one station to the next,
    // split one way in the first half of its piece (a fan or a side) and the other way in the second, so each corner
    // is drawn the same either side of its bisector.
    const stations: { x: number; z: number; nx: number; nz: number; corner: number; side: number; along: number }[] = [];
    // (`corner`: a fan's corner, else the corner nearest the side's station; `side`: -1 in a fan; `along`: m from that corner.)
    const late: boolean[] = [];
    for (let k = 0; k < 4; k++) {
      const [cx, cz] = corners[k];
      const [ex, ez] = corners[(k + 1) % 4];
      // (Side k's way out: side 0, along z0, looks towards -z; each next one a quarter turn on.)
      const out = -Math.PI / 2 + (k * Math.PI) / 2;
      for (let j = 0; j < ARC; j++) {
        const a = out - Math.PI / 2 + ((Math.PI / 2) * j) / ARC;
        stations.push({ x: cx, z: cz, nx: Math.cos(a), nz: Math.sin(a), corner: k, side: -1, along: 0 });
        late.push(j >= ARC / 2);
      }
      const len = Math.hypot(ex - cx, ez - cz);
      const n = 2 * Math.max(1, Math.ceil(len / (2 * STEP)));
      for (let i = 0; i < n; i++) {
        const t = i / n;
        stations.push({ x: cx + (ex - cx) * t, z: cz + (ez - cz) * t, nx: Math.cos(out), nz: Math.sin(out), corner: t < 0.5 ? k : (k + 1) % 4, side: k, along: Math.min(t, 1 - t) * len });
        late.push(i >= n / 2);
      }
    }
    const n = stations.length;
    // Each side station's height: over the highest land across the strip the veil fades over, and its neighbours'
    // (so nothing between two stations stands through it).
    const strip = stations.map((p) => {
      if (p.side < 0) return -Infinity;
      let top = -Infinity;
      for (let d = 0; d <= inset; d += STEP / 2) top = Math.max(top, height(p.x + p.nx * d, p.z + p.nz * d));
      return top;
    });
    const sides = strip.map((h, i) => (stations[i].side < 0 ? h : Math.max(h, strip[(i + n - 1) % n], strip[(i + 1) % n])));
    // One height for each corner's fan: over the land in the corner's square and its sides' ends.
    const cornerTop = corners.map(([cx, cz], k) => {
      const [sx, sz] = [k === 1 || k === 2 ? 1 : -1, k >= 2 ? 1 : -1];
      let top = -Infinity;
      for (let u = 0; u <= inset; u += STEP / 2) for (let v = 0; v <= inset; v += STEP / 2) top = Math.max(top, height(cx + sx * u, cz + sz * v));
      return top;
    });
    // (The side stations next to a corner raise it: the fan meets them level.)
    stations.forEach((p, i) => {
      if (p.side >= 0 && p.along <= STEP + 1e-6) cornerTop[p.corner] = Math.max(cornerTop[p.corner], sides[i]);
    });
    // Towards a corner a side eases up to its corner's height over `EASE` m, so the veil has no step where they meet.
    const lift = stations.map((p, i) => {
      const corner = cornerTop[p.corner];
      if (p.side < 0) return corner + LIFT;
      const t = Math.min(1, p.along / EASE);
      return Math.max(sides[i], corner + (sides[i] - corner) * t * t * (3 - 2 * t)) + LIFT;
    });
    const ground: number[] = [];
    const heights: number[] = [];
    const colors: number[] = [];
    const clampX = (x: number) => Math.min(box.x1, Math.max(box.x0, x));
    const clampZ = (z: number) => Math.min(box.z1, Math.max(box.z0, z));
    for (const [d, a] of RINGS) {
      const o = Math.max(0, inset - d);
      stations.forEach((p, i) => {
        const [x, z] = [p.x + p.nx * o, p.z + p.nz * o];
        ground.push(x, Math.min(lift[i], height(clampX(x), clampZ(z))), z);
        heights.push(lift[i]);
        colors.push(1, 1, 1, a);
      });
    }
    const indices: number[] = [];
    for (let r = 0; r + 1 < RINGS.length; r++) {
      for (let i = 0; i < n; i++) {
        const [a, b] = [r * n + i, r * n + ((i + 1) % n)];
        const [c, e] = [a + n, b + n];
        if (late[i]) indices.push(a, b, c, b, e, c);
        else indices.push(a, b, e, a, e, c);
      }
    }
    this.ground = new Float32Array(ground);
    this.height = new Float32Array(heights);
    this.colors = new Float32Array(colors);
    this.indices = new Uint32Array(indices);
    this.positions = new Float32Array(ground.length);
    // (Hung straight over its land until `view` has an eye: written again in place as the camera moves.)
    for (let v = 0; v < heights.length; v++) this.positions.set([ground[v * 3], heights[v], ground[v * 3 + 2]], v * 3);
    this.eye.x = NaN;
    // (The old mesh goes with its mesh instance.)
    if (this.entity.render) this.entity.removeComponent('render');
    const mesh = (this.mesh = new pc.Mesh(this.app.graphicsDevice));
    mesh.clear(true, false);
    this.write();
    const mi = new pc.MeshInstance(mesh, this.material);
    mi.cull = false;
    this.entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false, layers: [layer] });
  }

  /** The mesh written from the arrays (its positions as `view` left them). */
  private write(): void {
    const mesh = this.mesh!;
    mesh.setPositions(this.positions);
    mesh.setColors(this.colors);
    mesh.setIndices(this.indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES, false);
  }

  /**
   * The eye where the camera is (game space, as the veil is): each point of the veil moved along the line from the eye
   * to the land it is for, to where that line is at the veil's height, so that land is seen through it. Only while it
   * shows, and when the eye moved; true if it was moved (the frame is to be drawn again). Allocation-free.
   */
  view(x: number, y: number, z: number): boolean {
    if (!this.mesh || this.shown <= 0) return false;
    const e = this.eye;
    if (Math.abs(x - e.x) + Math.abs(y - e.y) + Math.abs(z - e.z) < 0.5) return false;
    e.x = x;
    e.y = y;
    e.z = z;
    const g = this.ground;
    const h = this.height;
    const out = this.positions;
    for (let v = 0; v < h.length; v++) {
      const gx = g[v * 3];
      const gy = g[v * 3 + 1];
      const gz = g[v * 3 + 2];
      // (How far from the land towards the eye the line reaches the veil's height; an eye under it: as far as it goes.)
      const t = y - gy > 1 ? Math.min(0.95, (h[v] - gy) / (y - gy)) : 0.95;
      out[v * 3] = gx + (x - gx) * t;
      out[v * 3 + 1] = h[v];
      out[v * 3 + 2] = gz + (z - gz) * t;
    }
    this.write();
    return true;
  }

  /** Its colour: the haze's (`scene.fog.color`, as PlayCanvas takes a colour: the sky's dome comes out the same). */
  setColour(c: pc.Color): void {
    this.material.emissive.set(c.r, c.g, c.b);
    this.material.update();
  }

  /** How far out the camera is (m): the veil fades in from `SHOW_FROM`. False: none (the game's view). */
  show(radius: number | false): void {
    const t = radius === false ? 0 : Math.min(1, Math.max(0, (radius - SHOW_FROM) / (SHOW_FULL - SHOW_FROM)));
    const k = t * t * (3 - 2 * t);
    if (Math.abs(k - this.shown) < 0.005) return;
    this.shown = k;
    this.material.opacity = k;
    this.material.update();
    this.entity.enabled = k > 0 && !!this.mesh;
  }

  destroy(): void {
    this.entity.destroy();
    this.material.destroy();
  }
}
