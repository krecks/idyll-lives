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
 */

import * as pc from './engine';

/** How far in from the landscape's edge the veil begins (m), and its share of the way across where it is whole. */
const FADE = 240;
/** The veil's rings, in from the landscape's edge (m; negative: past it), and how clear it is at each (its alpha). */
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
/** The camera's distance (m) the veil fades in over. */
const SHOW_FROM = 650;
const SHOW_FULL = 1050;

export class LandHaze {
  private readonly material: pc.StandardMaterial;
  private readonly entity: pc.Entity;
  private mesh: pc.Mesh | null = null;
  private key = '';
  private shown = -1;

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
    // Each ring a rectangle, its points the same way round each (a side's count the outer edge's), so they join up.
    const sides: [number, number, number, number][] = [
      [box.x0, box.z0, box.x1, box.z0],
      [box.x1, box.z0, box.x1, box.z1],
      [box.x1, box.z1, box.x0, box.z1],
      [box.x0, box.z1, box.x0, box.z0],
    ];
    // (Each side's way in: down, left, up, right; a corner's both of its sides'.)
    const ins: [number, number][] = [
      [0, 1],
      [-1, 0],
      [0, -1],
      [1, 0],
    ];
    /** Edge points, each with the way in from it (a rectangle `d` m in has its points at `d` times it). */
    const edge: [number, number, number, number][] = [];
    sides.forEach(([ax, az, bx, bz], k) => {
      const n = Math.max(1, Math.ceil(Math.hypot(bx - ax, bz - az) / STEP));
      const [prev, own] = [ins[(k + 3) % 4], ins[k]];
      for (let i = 0; i < n; i++) edge.push([ax + ((bx - ax) * i) / n, az + ((bz - az) * i) / n, own[0] + (i ? 0 : prev[0]), own[1] + (i ? 0 : prev[1])]);
    });
    const at = ([x, z, nx, nz]: readonly number[], d: number): [number, number] => [x + nx * d, z + nz * d];
    // Each column's height: over the highest land along it, from where the veil begins to the edge.
    const lift = edge.map((p) => {
      let top = -Infinity;
      for (let d = 0; d <= FADE; d += STEP / 2) top = Math.max(top, height(...at(p, d)));
      return top + LIFT;
    });
    const positions: number[] = [];
    const colors: number[] = [];
    const n = edge.length;
    for (const [d, a] of RINGS) {
      edge.forEach((p, i) => {
        const [px, pz] = at(p, d);
        positions.push(px, lift[i], pz);
        colors.push(1, 1, 1, a);
      });
    }
    const indices: number[] = [];
    for (let r = 0; r + 1 < RINGS.length; r++) {
      for (let i = 0; i < n; i++) {
        const [a, b] = [r * n + i, r * n + ((i + 1) % n)];
        const [c, e] = [a + n, b + n];
        indices.push(a, b, e, a, e, c);
      }
    }
    // (The old mesh goes with its mesh instance.)
    if (this.entity.render) this.entity.removeComponent('render');
    const mesh = (this.mesh = new pc.Mesh(this.app.graphicsDevice));
    mesh.setPositions(positions);
    mesh.setColors(colors);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const mi = new pc.MeshInstance(mesh, this.material);
    mi.cull = false;
    this.entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false, layers: [layer] });
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
