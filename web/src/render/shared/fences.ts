/**
 * Fences and gates the player builds (Build mode's Fence and Gate tools): one baked mesh of
 * every fence edge in its style (picket, ranch rails, slats, wrought iron, low stone wall).
 * Fences are drawn per 1 m grid edge with posts on the grid corners they share, so runs and
 * corners join up; a gate is a framed leaf in the fence's style between two stout posts, which
 * swings open for someone going through it (`Fences.gates`, `swingGate`) by moving its vertices
 * in the one mesh: no draw of its own.
 *
 * Engine-neutral: `planFences` writes the geometry and the gates' leaves, `swungGate` where a
 * leaf's vertices go as it swings; `babylon/fences.ts` and `pc/fences.ts` draw them.
 */

import type { FenceStyleDef } from '../../content/content';
import type { FenceEdge } from '../../core/protocol';
import { Geo } from './geometry';
import { Painter } from './street';

type Kind = FenceStyleDef['kind'];

const FALLBACK: FenceStyleDef = { id: 'picket', label: 'Picket', kind: 'picket', color: '#F2F0EA', height: 1 };
/** Colour of iron gate leaves in stone walls. */
const IRON = '#2B2D30';

/** How far out from its post a gate leaf's hinge stands, along the fence (the leaf starts at 0.09). */
const GATE_HINGE = 0.08;

/** A gate's leaf in the fences' mesh: its vertices, and where it turns. */
export interface GateLeaf {
  axis: 'h' | 'v';
  /** The middle of its gateway, and the hinge it turns about (x, z). */
  centre: [number, number];
  hinge: [number, number];
  /** Along the fence from the hinge to the latch (unit x, z). */
  along: [number, number];
  /** Its vertices in the mesh (from `first`, as many as `rest` holds), shut: positions and normals. */
  first: number;
  rest: Float32Array;
  restNormals: Float32Array;
}

/** The fences' geometry (plain finish, by vertex colour) and their gates' leaves in it. */
export interface FencePlan {
  geo: Geo;
  gates: GateLeaf[];
}

/** Writes all fences and gates (null when there are none). */
export function planFences(fences: readonly FenceEdge[], styles: readonly FenceStyleDef[]): FencePlan | null {
  if (!fences.length) return null;
  const p = new Painter(new Geo());
  const styleOf = (f: FenceEdge) => styles[f.style ?? 0] ?? styles[0] ?? FALLBACK;
  // Posts where fences meet or end: once per grid corner, in the tallest style there.
  const posts = new Map<string, { x: number; z: number; style: FenceStyleDef; gate: boolean }>();
  for (const f of fences) {
    const style = styleOf(f);
    const ends: [number, number][] = f.axis === 'h' ? [[f.x, f.z], [f.x + 1, f.z]] : [[f.x, f.z], [f.x, f.z + 1]];
    for (const [x, z] of ends) {
      const key = `${x}:${z}`;
      const had = posts.get(key);
      if (!had || (style.height ?? 1) > (had.style.height ?? 1)) posts.set(key, { x, z, style, gate: f.kind === 'gate' || !!had?.gate });
      else if (f.kind === 'gate') had.gate = true;
    }
  }
  const gates: GateLeaf[] = [];
  for (const f of fences) {
    const style = styleOf(f);
    // Local frame: +x along the edge (0..1), +z across it.
    p.at(f.x, f.z, f.axis === 'h' ? 0 : -Math.PI / 2);
    if (f.kind !== 'gate') {
      panel(p, style);
      continue;
    }
    const first = p.geo.pos.length / 3;
    gate(p, style);
    const [hx, , hz] = p.p(GATE_HINGE, 0, 0);
    gates.push({
      axis: f.axis,
      centre: f.axis === 'h' ? [f.x + 0.5, f.z] : [f.x, f.z + 0.5],
      hinge: [hx, hz],
      along: f.axis === 'h' ? [1, 0] : [0, 1],
      first,
      rest: new Float32Array(p.geo.pos.slice(first * 3)),
      restNormals: new Float32Array(p.geo.nor.slice(first * 3)),
    });
  }
  for (const post of posts.values()) {
    if (post.style.kind === 'stone' && !post.gate) continue;
    p.at(post.x, post.z, 0);
    postAt(p, post.style, post.gate);
  }
  return { geo: p.geo, gates };
}

const scratch = { pos: new Float32Array(0), nor: new Float32Array(0) };

/**
 * A gate's leaf `open` of the way (0 shut, 1 a quarter turn) about its hinge, its latch end
 * towards the side `towards` (+1: +z of a fence along x, +x of one along z; -1 the other): its
 * vertices' positions and normals (from `GateLeaf.first` in the mesh; scratch arrays, reused).
 */
export function swungGate(gate: GateLeaf, open: number, towards: number): { pos: Float32Array; nor: Float32Array } {
  const { rest, restNormals, hinge, along } = gate;
  // A turn of +90 degrees takes `along` to (z, -x).
  const across = gate.axis === 'h' ? [0, towards] : [towards, 0];
  const turn = (along[1] * across[0] - along[0] * across[1] >= 0 ? 1 : -1) * (Math.PI / 2) * open;
  const [c, s] = [Math.cos(turn), Math.sin(turn)];
  if (scratch.pos.length < rest.length) scratch.pos = new Float32Array(rest.length);
  if (scratch.nor.length < rest.length) scratch.nor = new Float32Array(rest.length);
  const pos = scratch.pos.subarray(0, rest.length);
  const nor = scratch.nor.subarray(0, rest.length);
  for (let i = 0; i < rest.length; i += 3) {
    const dx = rest[i] - hinge[0];
    const dz = rest[i + 2] - hinge[1];
    pos[i] = hinge[0] + dx * c + dz * s;
    pos[i + 1] = rest[i + 1];
    pos[i + 2] = hinge[1] - dx * s + dz * c;
    const [nx, nz] = [restNormals[i], restNormals[i + 2]];
    nor[i] = nx * c + nz * s;
    nor[i + 1] = restNormals[i + 1];
    nor[i + 2] = -nx * s + nz * c;
  }
  return { pos, nor };
}

const height = (s: FenceStyleDef) => s.height ?? 1;
const rough: Record<Kind, number> = { picket: 0.6, rails: 0.8, slats: 0.55, iron: 0.45, stone: 0.95 };

/** A grid-corner post (stouter beside a gate). */
function postAt(p: Painter, s: FenceStyleDef, gate: boolean): void {
  const h = height(s) + (gate ? 0.08 : 0.04);
  const r = s.kind === 'iron' ? (gate ? 0.06 : 0.035) : s.kind === 'stone' ? 0.2 : gate ? 0.07 : 0.05;
  const colour = s.kind === 'stone' ? s.color : s.color;
  p.paint(colour, rough[s.kind], s.kind === 'iron' ? 0.4 : 0);
  p.box(-r, 0, -r, r, h, r, 'ny');
  if (s.kind === 'picket' || s.kind === 'rails') p.paint(shade(colour, 0.92), rough[s.kind], 0).box(-r - 0.015, h, -r - 0.015, r + 0.015, h + 0.035, r + 0.015, 'ny');
  if (s.kind === 'iron') p.box(-0.03, h, -0.03, 0.03, h + 0.07, 0.03, 'ny');
}

/** One metre of fence between two posts. */
function panel(p: Painter, s: FenceStyleDef): void {
  const h = height(s);
  p.paint(s.color, rough[s.kind], s.kind === 'iron' ? 0.4 : 0);
  switch (s.kind) {
    case 'picket':
      for (const y of [0.25 * h, 0.7 * h]) p.box(0, y, -0.03, 1, y + 0.06, -0.01, 'ny');
      for (let x = 0.12; x < 0.95; x += 0.15) {
        for (const [z, n] of [[0.01, 1], [-0.01, -1]] as const) {
          p.poly([[x - 0.036, 0.05, z], [x + 0.036, 0.05, z], [x + 0.036, h * 0.86, z], [x, h * 0.94, z], [x - 0.036, h * 0.86, z]], [0, 0, n]);
        }
      }
      return;
    case 'rails':
      for (const y of [0.22, 0.52, 0.82]) p.box(0, y * h, -0.025, 1, y * h + 0.09, 0.025, 'ny');
      return;
    case 'slats':
      for (let y = 0.06; y + 0.09 <= h; y += 0.12) p.box(0, y, -0.02, 1, y + 0.09, 0.02);
      return;
    case 'iron':
      p.box(0, 0.1, -0.012, 1, 0.13, 0.012);
      p.box(0, h - 0.12, -0.012, 1, h - 0.09, 0.012);
      for (let x = 0.1; x < 0.95; x += 0.1) {
        p.box(x - 0.01, 0.02, -0.01, x + 0.01, h - 0.02, 0.01, 'ny');
        p.box(x - 0.02, h - 0.02, -0.02, x + 0.02, h + 0.02, 0.02, 'ny');
      }
      return;
    case 'stone':
      // A dry-stone wall, a little overlapping its neighbours so corners close, with a cap.
      p.box(-0.16, 0, -0.16, 1.16, h - 0.06, 0.16, 'ny');
      p.paint(shade(s.color, 1.12), 0.9, 0).box(-0.19, h - 0.06, -0.19, 1.19, h, 0.19, 'ny');
      return;
  }
}

/** A gate leaf: a frame with a brace, filled in the fence's style (iron in a stone wall). */
function gate(p: Painter, s: FenceStyleDef): void {
  const stone = s.kind === 'stone';
  const h = stone ? 1.0 : height(s);
  const colour = stone ? IRON : s.color;
  const [x0, x1, y0, y1] = [0.09, 0.91, 0.06, h * 0.92];
  const t = s.kind === 'iron' || stone ? 0.012 : 0.03;
  p.paint(shade(colour, 0.9), rough[stone ? 'iron' : s.kind], stone || s.kind === 'iron' ? 0.4 : 0);
  p.box(x0, y0, -t, x1, y0 + 0.07, t);
  p.box(x0, y1 - 0.07, -t, x1, y1, t);
  p.box(x0, y0, -t, x0 + 0.06, y1, t);
  p.box(x1 - 0.06, y0, -t, x1, y1, t);
  p.paint(colour, rough[stone ? 'iron' : s.kind], stone || s.kind === 'iron' ? 0.4 : 0);
  if (s.kind === 'iron' || stone) {
    for (let x = x0 + 0.12; x < x1 - 0.06; x += 0.1) p.box(x - 0.01, y0, -0.01, x + 0.01, y1, 0.01);
    return;
  }
  // Diagonal brace from the hinge foot to the latch head.
  const [bx0, by0, bx1, by1] = [x0 + 0.06, y0 + 0.07, x1 - 0.06, y1 - 0.07];
  const w = 0.04;
  for (const [z, n] of [[t + 0.002, 1], [-t - 0.002, -1]] as const) {
    p.poly([[bx0, by0, z], [bx0 + w, by0, z], [bx1, by1 - w, z], [bx1, by1, z], [bx1 - w, by1, z], [bx0, by0 + w, z]], [0, 0, n]);
  }
  if (s.kind === 'picket') {
    for (let x = x0 + 0.11; x < x1 - 0.06; x += 0.13) p.box(x - 0.03, y0, -0.01, x + 0.03, y1 + 0.04, 0.01);
  } else if (s.kind === 'slats') {
    for (let y = y0 + 0.1; y + 0.08 < y1 - 0.06; y += 0.12) p.box(x0 + 0.06, y, -0.015, x1 - 0.06, y + 0.08, 0.015);
  } else {
    p.box(x0, (y0 + y1) / 2 - 0.04, -t, x1, (y0 + y1) / 2 + 0.04, t);
  }
}

/** A colour lightened (>1) or darkened (<1). */
function shade(hex: string, k: number): string {
  const n = parseInt(hex.slice(1), 16);
  const c = [n >> 16, (n >> 8) & 255, n & 255].map((v) => Math.max(0, Math.min(255, Math.round(v * k))));
  return `#${c.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}
