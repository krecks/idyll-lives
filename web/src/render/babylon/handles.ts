/**
 * Resize handles: blue arrows around the object picked in Buy mode (or a window or door in
 * Build mode), one per side it can grow towards and one pointing up for its height. Arrows on
 * the floor lie flat, arrows on a wall stand in its plane, the height arrow turns to face the
 * camera. They draw over everything (their own rendering group) and keep about the same size on
 * screen. The game says where they go (`ResizeHandle`); picking them is by distance on screen.
 */

import { Color3, Mesh, StandardMaterial, Vector3, VertexData, type Scene } from './core';
import type { ResizeHandle } from '../types';

/** Arrow length (m) at the reference distance, and the distance (m) at which arrows are that size. */
const LENGTH = 0.42;
const REFERENCE = 9;
/** A press this close (CSS px) to an arrow's middle takes it. */
const REACH = 26;

const BLUE = Color3.FromHexString('#4FA7E8');
const HOT = Color3.FromHexString('#9AD4FF');

export class ResizeHandles {
  private readonly flat: Mesh;
  private readonly standing: Mesh;
  private readonly upright: Mesh;
  private readonly downright: Mesh;
  private readonly material: StandardMaterial;
  private readonly hotMaterial: StandardMaterial;
  private meshes: Mesh[] = [];
  private handles: readonly ResizeHandle[] = [];
  /** World positions of the handles (render space), for sizing and picking. */
  private spots: Vector3[] = [];

  constructor(
    scene: Scene,
    private readonly layerMask: number,
  ) {
    this.material = arrowMaterial(scene, 'resizeHandle', BLUE);
    this.hotMaterial = arrowMaterial(scene, 'resizeHandleHot', HOT);
    this.flat = arrowMesh(scene, 'resizeArrowFlat', 'xz');
    this.standing = arrowMesh(scene, 'resizeArrowStanding', 'xy');
    this.upright = arrowMesh(scene, 'resizeArrowUp', 'up');
    this.downright = arrowMesh(scene, 'resizeArrowDown', 'down');
  }

  /** Shows `handles` (render-space positions from `place`), `hot` brighter; null hides them. */
  set(handles: readonly ResizeHandle[] | null, place: (h: ResizeHandle) => Vector3, hot: string | null): void {
    const list = handles ?? [];
    const same = list.length === this.handles.length && list.every((h, i) => h.id === this.handles[i].id && h.dir.every((v, k) => v === this.handles[i].dir[k]) && (h.wall ?? false) === (this.handles[i].wall ?? false));
    if (!same) {
      for (const m of this.meshes) m.dispose();
      this.meshes = list.map((h) => this.arrowFor(h));
    }
    this.handles = list;
    this.spots = list.map(place);
    this.meshes.forEach((m, i) => {
      m.position.copyFrom(this.spots[i]);
      m.material = list[i].id === hot ? this.hotMaterial : this.material;
    });
  }

  /** Per frame: keeps the arrows about the same size on screen. */
  update(eye: Vector3): void {
    this.meshes.forEach((m, i) => m.scaling.setAll(Math.min(2.6, Math.max(0.55, Vector3.Distance(eye, this.spots[i]) / REFERENCE))));
  }

  /** The handle within reach of screen point (`sx`, `sy`), given a projection to CSS px. */
  at(sx: number, sy: number, project: (p: Vector3, out: { x: number; y: number }) => boolean): string | null {
    let best: string | null = null;
    let bestD = REACH;
    const out = { x: 0, y: 0 };
    this.spots.forEach((p, i) => {
      if (!project(p, out)) return;
      const d = Math.hypot(out.x - sx, out.y - sy);
      if (d < bestD) {
        bestD = d;
        best = this.handles[i].id;
      }
    });
    return best;
  }

  /** An arrow pointing along the handle's direction: flat on the floor, standing on a wall, or upright facing the camera. */
  private arrowFor(h: ResizeHandle): Mesh {
    const up = h.dir[1] !== 0;
    const mesh = (up ? (h.dir[1] > 0 ? this.upright : this.downright) : h.wall ? this.standing : this.flat).clone(`resize:${h.id}`);
    mesh.setEnabled(true);
    mesh.isPickable = false;
    mesh.layerMask = this.layerMask;
    mesh.renderingGroupId = 1;
    if (up) {
      // Pointing up (or down), turned to the camera about the vertical.
      mesh.billboardMode = Mesh.BILLBOARDMODE_Y;
    } else {
      // +x turned onto the direction (yaw is clockwise from +x seen from above, as Babylon's y rotation).
      mesh.rotation.y = Math.atan2(-h.dir[2], h.dir[0]);
    }
    return mesh;
  }

  dispose(): void {
    for (const m of this.meshes) m.dispose();
    this.flat.dispose();
    this.standing.dispose();
    this.upright.dispose();
    this.downright.dispose();
    this.material.dispose();
    this.hotMaterial.dispose();
  }
}

function arrowMaterial(scene: Scene, name: string, color: Color3): StandardMaterial {
  const m = new StandardMaterial(name, scene);
  m.disableLighting = true;
  m.emissiveColor = color;
  m.backFaceCulling = false;
  m.fogEnabled = false;
  return m;
}

/** A chunky arrow centred on its middle: pointing +x in the xz plane or the xy plane, or pointing +y or -y (`up`, `down`: in the xy plane). */
function arrowMesh(scene: Scene, name: string, plane: 'xz' | 'xy' | 'up' | 'down'): Mesh {
  const L = LENGTH;
  // Shaft from the tail to the head's base, then the head (outline in arrow space: along, across).
  const outline: [number, number][] = [
    [-L / 2, -0.06],
    [L * 0.05, -0.06],
    [L * 0.05, -0.15],
    [L / 2, 0],
    [L * 0.05, 0.15],
    [L * 0.05, 0.06],
    [-L / 2, 0.06],
  ];
  const positions = outline.flatMap(([a, b]) => (plane === 'xz' ? [a, 0, b] : plane === 'xy' ? [a, b, 0] : plane === 'up' ? [b, a, 0] : [b, -a, 0]));
  // Shaft quad and head triangle.
  const indices = [0, 1, 5, 0, 5, 6, 2, 3, 4];
  const normals = outline.flatMap(() => (plane === 'xz' ? [0, 1, 0] : [0, 0, -1]));
  const data = new VertexData();
  data.positions = positions;
  data.indices = indices;
  data.normals = normals;
  const mesh = new Mesh(name, scene);
  data.applyToMesh(mesh);
  mesh.setEnabled(false);
  return mesh;
}
