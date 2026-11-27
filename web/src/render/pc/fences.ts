/**
 * Fences and gates the player builds on PlayCanvas: one mesh of every fence edge in its style, as
 * `shared/fences.ts` writes it (posts on the grid corners they share, a gate a framed leaf between
 * two stout posts), in the plain finish. A gate swings open for someone going through it
 * (`swingGate`) by moving its vertices in the one mesh: no draw of its own, only its leaf's vertices
 * sent up again, the mesh's box made once round every leaf swung open.
 */

import * as pc from './engine';
import { MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import type { FenceStyleDef } from '../../content/content';
import type { FenceEdge } from '../../core/protocol';
import { planFences, swungGate, type GateLeaf } from '../shared/fences';

export type { GateLeaf } from '../shared/fences';

export interface Fences {
  entity: pc.Entity;
  gates: GateLeaf[];
  /** The mesh (its vertex buffer's copy on the CPU: the leaves' vertices are written into it as they swing). */
  mesh: pc.Mesh;
}

/** How far a leaf's corner swinging may stand past its box shut and open (m): a leaf is thin, its arc hardly bulges. */
const SWING_PAD = 0.05;

/**
 * Builds the mesh of all fences and gates under `parent` (null when there are none). Hedges are the
 * garden's (`fenceHedges`), but for `hedgeBoxes` (Build mode's previews: plain boxes). `wallAt`: no post
 * where a fence meets a wall (`writeFences`).
 */
export function buildFences(
  app: pc.AppBase,
  parent: pc.Entity,
  fences: readonly FenceEdge[],
  styles: readonly FenceStyleDef[],
  { hedgeBoxes = false, wallAt, ground }: { hedgeBoxes?: boolean; wallAt?: (x: number, z: number) => boolean; ground?: (x: number, z: number) => number } = {},
): Fences | null {
  const plan = planFences(fences, styles, { hedgeBoxes, wallAt, ground });
  if (!plan) return null;
  const data = plan.geo.meshData({ colors: true });
  const mesh = meshFrom(app.graphicsDevice, data, plan.gates.length > 0 ? 'dynamic' : 'static');
  if (!mesh) return null;
  // (Its box round the leaves standing open either way, made once: a swing then leaves it be.)
  if (plan.gates.length) {
    const [min, max] = [mesh.aabb.getMin().clone(), mesh.aabb.getMax().clone()];
    for (const gate of plan.gates) {
      for (const towards of [1, -1]) {
        const { pos } = swungGate(gate, 1, towards);
        for (let i = 0; i < pos.length; i += 3) {
          min.set(Math.min(min.x, pos[i]), Math.min(min.y, pos[i + 1]), Math.min(min.z, pos[i + 2]));
          max.set(Math.max(max.x, pos[i]), Math.max(max.y, pos[i + 1]), Math.max(max.z, pos[i + 2]));
        }
      }
    }
    const pad = new pc.Vec3(SWING_PAD, SWING_PAD, SWING_PAD);
    const box = new pc.BoundingBox();
    box.setMinMax(min.sub(pad), max.add(pad));
    mesh.aabb = box;
  }
  const entity = drawMesh(parent, 'fences', mesh, MaterialLibrary.for(app).plainFinish(), { castShadows: true });
  return { entity, gates: plan.gates, mesh };
}

/**
 * Turns a gate's leaf `open` of the way (0 shut, 1 a quarter turn) about its hinge, its latch end
 * towards the side `towards` (+1: +z of a fence along x, +x of one along z; -1 the other).
 */
export function swingGate(fences: Fences, gate: GateLeaf, open: number, towards: number): void {
  const { pos, nor } = swungGate(gate, open, towards);
  writeLeaf(fences.mesh.vertexBuffer, pc.SEMANTIC_POSITION, gate.first, pos);
  writeLeaf(fences.mesh.vertexBuffer, pc.SEMANTIC_NORMAL, gate.first, nor);
}

/** A leaf's `values` (x, y, z a vertex, from vertex `first`) into `vb`'s copy of stream `semantic`, and that span alone sent up: not the whole town's gates. */
function writeLeaf(vb: pc.VertexBuffer, semantic: string, first: number, values: Float32Array): void {
  const e = vb.format.elements.find((el) => el.name === semantic);
  if (!e) return;
  const copy = vb.lock();
  const floats = copy instanceof ArrayBuffer ? new Float32Array(copy) : new Float32Array(copy.buffer, copy.byteOffset, copy.byteLength >> 2);
  const start = e.offset + first * e.stride;
  const n = values.length / 3;
  for (let v = 0; v < n; v++) {
    const o = (start + v * e.stride) / 4;
    floats[o] = values[v * 3];
    floats[o + 1] = values[v * 3 + 1];
    floats[o + 2] = values[v * 3 + 2];
  }
  vb.unlock(start, (n - 1) * e.stride + 12);
}
