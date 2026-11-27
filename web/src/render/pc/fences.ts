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

/** Past this many spans of gates moved by the land at once (`regroundGates`), the whole mesh is sent up in one. */
const MANY_SPANS = 64;

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

/**
 * The gates on the ground as it is now (`ground`, as they were built on; a land stroke moved it): each leaf and post
 * that moved lifted or lowered in the mesh and those spans alone sent up, as a swing's (many of them: the span from the
 * first to the last, at once), a leaf's rest with it (it swings on from there), the mesh's box grown to hold them.
 */
export function regroundGates(fences: Fences, ground: (x: number, z: number) => number): void {
  const vb = fences.mesh.vertexBuffer;
  const { e, floats } = floatsOf(vb, pc.SEMANTIC_POSITION)!;
  const box = fences.mesh.aabb;
  let [lo, hi] = [box.getMin().y, box.getMax().y];
  // (The spans moved: first vertex, count.)
  const spans: number[] = [];
  const lift = (first: number, count: number, dy: number) => {
    for (let v = first; v < first + count; v++) {
      const o = (e.offset + v * e.stride) / 4 + 1;
      floats[o] += dy;
      lo = Math.min(lo, floats[o]);
      hi = Math.max(hi, floats[o]);
    }
    spans.push(first, count);
  };
  for (const leaf of fences.gates) {
    const dy = ground(leaf.centre[0], leaf.centre[1]) - leaf.y;
    if (dy !== 0) {
      leaf.y += dy;
      for (let i = 1; i < leaf.rest.length; i += 3) leaf.rest[i] += dy;
      lift(leaf.first, leaf.rest.length / 3, dy);
    }
    // (A post two gates share moves once: its `y` is the ground's after the first.)
    for (const post of leaf.posts) {
      const dp = ground(post.x, post.z) - post.y;
      if (dp === 0) continue;
      post.y += dp;
      lift(post.first, post.count, dp);
    }
  }
  if (!spans.length) return;
  // (A town's gates' mesh is some 12 MB, its posts after all the leaves: a stroke's few gates sent up span by span, a
  // whole town's, 1 000 uploads, as one.)
  if (spans.length <= 2 * MANY_SPANS) for (let i = 0; i < spans.length; i += 2) vb.unlock(e.offset + spans[i] * e.stride, (spans[i + 1] - 1) * e.stride + 12);
  else vb.unlock();
  const min = box.getMin().clone();
  const max = box.getMax().clone();
  [min.y, max.y] = [Math.min(min.y, lo), Math.max(max.y, hi)];
  const grown = new pc.BoundingBox();
  grown.setMinMax(min, max);
  fences.mesh.aabb = grown;
}

/** A leaf's `values` (x, y, z a vertex, from vertex `first`) into `vb`'s copy of stream `semantic`, and that span alone sent up: not the whole town's gates. */
function writeLeaf(vb: pc.VertexBuffer, semantic: string, first: number, values: Float32Array): void {
  const stream = floatsOf(vb, semantic);
  if (!stream) return;
  const { e, floats } = stream;
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

/** `vb`'s stream `semantic` (null: it has none) and its copy, locked (`unlock` sends it up), as floats. */
function floatsOf(vb: pc.VertexBuffer, semantic: string) {
  const e = vb.format.elements.find((el) => el.name === semantic);
  if (!e) return null;
  const copy = vb.lock();
  return { e, floats: copy instanceof ArrayBuffer ? new Float32Array(copy) : new Float32Array(copy.buffer, copy.byteOffset, copy.byteLength >> 2) };
}
