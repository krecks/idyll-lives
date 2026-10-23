/**
 * Fences and gates the player builds (Build mode's Fence and Gate tools): one baked mesh of
 * every fence edge in its style (picket, ranch rails, slats, wrought iron, low stone wall).
 * Fences are drawn per 1 m grid edge with posts on the grid corners they share, so runs and
 * corners join up; a gate is a framed leaf in the fence's style between two stout posts, which
 * swings open for someone going through it (`Fences.gates`, `swingGate`) by moving its vertices
 * in the one mesh: no draw of its own. The geometry is engine-neutral (`shared/fences.ts`).
 */

import { VertexBuffer, type Mesh, type Scene } from './core';
import type { FenceStyleDef } from '../../content/content';
import type { FenceEdge } from '../../core/protocol';
import { toMesh } from './meshes';
import { MaterialLibrary } from './materials';
import { planFences, swungGate, type GateLeaf } from '../shared/fences';

export type { GateLeaf } from '../shared/fences';

export interface Fences {
  mesh: Mesh;
  gates: GateLeaf[];
}

/** Builds the mesh of all fences and gates (null when there are none). */
export function buildFences(scene: Scene, fences: readonly FenceEdge[], styles: readonly FenceStyleDef[], layerMask?: number): Fences | null {
  const plan = planFences(fences, styles);
  if (!plan) return null;
  const { geo, gates } = plan;
  const mesh = toMesh(geo, 'fences', scene, { colors: true, cut: false, updatable: gates.length > 0 });
  if (!mesh) return null;
  if (gates.length) {
    // Room in its bounds for the leaves standing open.
    const bounds = mesh.getBoundingInfo();
    const [min, max] = [bounds.minimum.clone(), bounds.maximum.clone()];
    for (const { hinge: [x, z] } of gates) {
      min.set(Math.min(min.x, x - 1), min.y, Math.min(min.z, z - 1));
      max.set(Math.max(max.x, x + 1), max.y, Math.max(max.z, z + 1));
    }
    bounds.reConstruct(min, max);
  }
  mesh.hasVertexAlpha = false;
  mesh.material = MaterialLibrary.for(scene).plainFinish();
  mesh.receiveShadows = true;
  mesh.isPickable = false;
  if (layerMask !== undefined) mesh.layerMask = layerMask;
  return { mesh, gates };
}

/**
 * Turns a gate's leaf `open` of the way (0 shut, 1 a quarter turn) about its hinge, its latch end
 * towards the side `towards` (+1: +z of a fence along x, +x of one along z; -1 the other).
 */
export function swingGate(mesh: Mesh, gate: GateLeaf, open: number, towards: number): void {
  const { pos, nor } = swungGate(gate, open, towards);
  mesh.getVertexBuffer(VertexBuffer.PositionKind)?.updateDirectly(pos, gate.first * 3);
  mesh.getVertexBuffer(VertexBuffer.NormalKind)?.updateDirectly(nor, gate.first * 3);
}
