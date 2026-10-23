/**
 * Fences and gates the player builds on PlayCanvas (the port of `babylon/fences.ts`): one mesh of
 * every fence edge in its style, as `shared/fences.ts` writes it (posts on the grid corners they
 * share, a gate a framed leaf between two stout posts), in the plain finish. A gate swings open
 * for someone going through it (`swingGate`) by moving its vertices in the one mesh: no draw of
 * its own.
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
  /** The mesh, and its vertices as last uploaded (the leaves' are written into them as they swing). */
  mesh: pc.Mesh;
  positions: Float32Array;
  normals: Float32Array;
}

/** Builds the mesh of all fences and gates under `parent` (null when there are none). */
export function buildFences(app: pc.AppBase, parent: pc.Entity, fences: readonly FenceEdge[], styles: readonly FenceStyleDef[]): Fences | null {
  const plan = planFences(fences, styles);
  if (!plan) return null;
  const data = plan.geo.meshData({ colors: true });
  const mesh = meshFrom(app.graphicsDevice, data, plan.gates.length > 0);
  if (!mesh) return null;
  const entity = drawMesh(parent, 'fences', mesh, MaterialLibrary.for(app).plainFinish(), { castShadows: true });
  return { entity, gates: plan.gates, mesh, positions: data.positions, normals: data.normals };
}

/**
 * Turns a gate's leaf `open` of the way (0 shut, 1 a quarter turn) about its hinge, its latch end
 * towards the side `towards` (+1: +z of a fence along x, +x of one along z; -1 the other).
 */
export function swingGate(fences: Fences, gate: GateLeaf, open: number, towards: number): void {
  const { pos, nor } = swungGate(gate, open, towards);
  fences.positions.set(pos, gate.first * 3);
  fences.normals.set(nor, gate.first * 3);
  fences.mesh.setPositions(fences.positions);
  fences.mesh.setNormals(fences.normals);
  // (Its box follows the leaves standing open.)
  fences.mesh.update(pc.PRIMITIVE_TRIANGLES, true);
}
