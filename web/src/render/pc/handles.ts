/**
 * Build and Buy mode's pointers: the hover tile (`HoverTile`) and the resize handles (`ResizeHandles`).
 *
 * Resize handles (the port of `babylon/handles.ts`): blue arrows around the object picked in Buy
 * mode (or a window or door in Build mode), one per side it can grow towards and one pointing up
 * for its height. Arrows on the floor lie flat, arrows on a wall stand in its plane, the height
 * arrow turns to face the camera. They draw over everything (Babylon's rendering group 1: here
 * without the depth test, after every other blended draw) and keep about the same size on screen.
 * The game says where they go (`ResizeHandle`); picking them is by distance on screen.
 *
 * In game space under the world root (`space.ts`), unlit, in Babylon's colours as its unlit
 * standard material showed them (the sRGB hex linearised before the image processing). No
 * allocations per frame or per `at`.
 */

import * as pc from './engine';
import { drawAfter } from './meshes';
import { toLinear } from '../shared/math';
import type { ResizeHandle } from '../types';

/** Arrow length (m) at the reference distance, and the distance (m) at which arrows are that size. */
const LENGTH = 0.42;
const REFERENCE = 9;
/** A press this close (CSS px) to an arrow's middle takes it. */
const REACH = 26;

const BLUE = linearHex('#4FA7E8');
/** The hover tile's colour (the interface's accent) and how much of it shows. */
const ACCENT = linearHex('#5B7CFA');
const TILE_ALPHA = 0.35;
/** The hover tile's side (m) and height over the floor (m). */
const TILE_SIZE = 0.92;
const TILE_Y = 0.025;
const HOT = linearHex('#9AD4FF');

type Shape = 'xz' | 'xy' | 'up' | 'down';

/** One arrow shown: its entity and draw, where it is (game space), and whether it turns to the camera. */
interface Arrow {
  entity: pc.Entity;
  draw: pc.MeshInstance;
  at: pc.Vec3;
  facing: boolean;
}

export class ResizeHandles {
  private readonly shapes: Record<Shape, pc.Mesh>;
  private readonly material: pc.StandardMaterial;
  private arrows: Arrow[] = [];
  private handles: readonly ResizeHandle[] = [];
  private readonly screen = { x: 0, y: 0 };

  constructor(
    device: pc.GraphicsDevice,
    private readonly parent: pc.Entity,
  ) {
    const m = (this.material = new pc.StandardMaterial());
    m.name = 'resizeHandle';
    m.useLighting = false;
    m.useSkybox = false;
    m.useFog = false;
    m.diffuse.set(0, 0, 0);
    // (Its colour is each arrow's `material_emissive`, linear.)
    m.emissive.set(1, 1, 1);
    m.cull = pc.CULLFACE_NONE;
    // Over everything: blended (so after every opaque draw), last of those, no depth.
    m.blendType = pc.BLEND_NORMAL;
    m.depthTest = false;
    m.depthWrite = false;
    m.update();
    this.shapes = { xz: arrowMesh(device, 'xz'), xy: arrowMesh(device, 'xy'), up: arrowMesh(device, 'up'), down: arrowMesh(device, 'down') };
    // (Kept: the arrows' draws share them.)
    for (const mesh of Object.values(this.shapes)) mesh.incRefCount();
  }

  /** Shows `handles` at the game-space spots `place` gives (render rows), `hot` brighter; null hides them. */
  set(handles: readonly ResizeHandle[] | null, place: (h: ResizeHandle, out: pc.Vec3) => void, hot: string | null): void {
    const list = handles ?? [];
    const was = this.handles;
    const same = list.length === was.length && list.every((h, i) => h.id === was[i].id && h.dir.every((v, k) => v === was[i].dir[k]) && (h.wall ?? false) === (was[i].wall ?? false));
    if (!same) {
      for (const a of this.arrows) a.entity.destroy();
      this.arrows = list.map((h) => this.arrowFor(h));
    }
    this.handles = list;
    this.arrows.forEach((a, i) => {
      place(list[i], a.at);
      a.entity.setLocalPosition(a.at);
      a.draw.setParameter('material_emissive', list[i].id === hot ? HOT : BLUE);
    });
  }

  /** Per frame: keeps the arrows about the same size on screen, the height arrows turned to the camera (`eye`: game space). */
  frame(eye: { x: number; y: number; z: number }): void {
    for (const a of this.arrows) {
      const p = a.at;
      const s = Math.min(2.6, Math.max(0.55, Math.hypot(eye.x - p.x, eye.y - p.y, eye.z - p.z) / REFERENCE));
      a.entity.setLocalScale(s, s, s);
      // About the vertical, its face to the camera (Babylon's `BILLBOARDMODE_Y`).
      if (a.facing) a.entity.setLocalEulerAngles(0, Math.atan2(eye.x - p.x, eye.z - p.z) * pc.math.RAD_TO_DEG, 0);
    }
  }

  /** The handle within reach of screen point (`sx`, `sy`), given a projection of game-space points to CSS px. */
  at(sx: number, sy: number, project: (x: number, y: number, z: number, out: { x: number; y: number }) => boolean): string | null {
    let best: string | null = null;
    let bestD = REACH;
    const out = this.screen;
    for (let i = 0; i < this.arrows.length; i++) {
      const p = this.arrows[i].at;
      if (!project(p.x, p.y, p.z, out)) continue;
      const d = Math.hypot(out.x - sx, out.y - sy);
      if (d < bestD) {
        bestD = d;
        best = this.handles[i].id;
      }
    }
    return best;
  }

  /** An arrow pointing along the handle's direction: flat on the floor, standing on a wall, or upright facing the camera. */
  private arrowFor(h: ResizeHandle): Arrow {
    const up = h.dir[1] !== 0;
    const mesh = this.shapes[up ? (h.dir[1] > 0 ? 'up' : 'down') : h.wall ? 'xy' : 'xz'];
    const entity = new pc.Entity(`resize:${h.id}`);
    const draw = new pc.MeshInstance(mesh, this.material);
    drawAfter(draw, 127);
    entity.addComponent('render', { meshInstances: [draw], castShadows: false, receiveShadows: false });
    // +x turned onto the direction (yaw clockwise from +x seen from above, as Babylon's y rotation).
    if (!up) entity.setLocalEulerAngles(0, Math.atan2(-h.dir[2], h.dir[0]) * pc.math.RAD_TO_DEG, 0);
    this.parent.addChild(entity);
    return { entity, draw, at: new pc.Vec3(), facing: up };
  }

  destroy(): void {
    for (const a of this.arrows) a.entity.destroy();
    this.arrows = [];
    this.handles = [];
    for (const mesh of Object.values(this.shapes)) {
      mesh.decRefCount();
      if (mesh.refCount < 1) mesh.destroy();
    }
    this.material.destroy();
  }
}

/**
 * The tile under the pointer (Babylon's `cursor`): a flat square a little smaller than the tile, in
 * the accent colour, see-through. In lot rows under the storey layer, which lifts it to the storey
 * in view.
 */
export class HoverTile {
  private readonly entity: pc.Entity;
  private readonly material: pc.StandardMaterial;

  constructor(device: pc.GraphicsDevice, storeyLayer: pc.Entity) {
    const m = (this.material = new pc.StandardMaterial());
    m.name = 'cursor';
    m.useLighting = false;
    m.useSkybox = false;
    m.diffuse.set(0, 0, 0);
    m.emissive.set(1, 1, 1);
    m.opacity = TILE_ALPHA;
    m.blendType = pc.BLEND_NORMAL;
    m.cull = pc.CULLFACE_NONE;
    m.update();
    const h = TILE_SIZE / 2;
    const mesh = new pc.Mesh(device);
    mesh.setPositions([-h, 0, -h, h, 0, -h, h, 0, h, -h, 0, h]);
    mesh.setNormals([0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
    mesh.setIndices([0, 1, 2, 0, 2, 3]);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const draw = new pc.MeshInstance(mesh, m);
    draw.setParameter('material_emissive', ACCENT);
    this.entity = new pc.Entity('cursor');
    this.entity.addComponent('render', { meshInstances: [draw], castShadows: false, receiveShadows: false });
    this.entity.enabled = false;
    storeyLayer.addChild(this.entity);
  }

  /** Over tile (`x`, `z`) (lot rows of the storey in view), or hidden. */
  set(tile: { x: number; z: number } | null): void {
    if (tile) this.entity.setLocalPosition(tile.x + 0.5, TILE_Y, tile.z + 0.5);
    this.entity.enabled = !!tile;
  }

  destroy(): void {
    this.entity.destroy();
    this.material.destroy();
  }
}

/** An sRGB hex colour in linear light (what Babylon's unlit standard material passed on to the image processing). */
function linearHex(hex: string): Float32Array {
  const v = parseInt(hex.slice(1), 16);
  return Float32Array.of(toLinear(((v >> 16) & 255) / 255), toLinear(((v >> 8) & 255) / 255), toLinear((v & 255) / 255));
}

/** A chunky arrow centred on its middle: pointing +x in the xz plane or the xy plane, or pointing +y or -y (`up`, `down`: in the xy plane). */
function arrowMesh(device: pc.GraphicsDevice, plane: Shape): pc.Mesh {
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
  const normals = outline.flatMap(() => (plane === 'xz' ? [0, 1, 0] : [0, 0, -1]));
  const mesh = new pc.Mesh(device);
  mesh.setPositions(positions);
  mesh.setNormals(normals);
  // Shaft quad and head triangle.
  mesh.setIndices([0, 1, 5, 0, 5, 6, 2, 3, 4]);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  return mesh;
}
