/**
 * The cutaway hides a part of a wall that drops (`Cut.Hide`: window frames and panes, the wall
 * above and beside an opening, door leaves) by sending every vertex of it to one point
 * (`shaders/cutaway.ts`), so its triangles have no area. That hides it on every storey, as lowering
 * it below the floor did only on the ground: an upper storey's came down into the rooms below.
 * It holds only while each of the house's triangles has one cut (its three vertices the same kind
 * and masks), on the ground and upstairs alike: else a corner would be pulled to that point.
 */
import { expect, test } from 'bun:test';
import type { DiagonalWall, Opening, WallEdge, WorldStructure } from '../src/core/protocol';
import { Cut } from '../src/render/shared/geometry';
import { CUTAWAY } from '../src/render/pc/shaders/cutaway';
import { HousePlanner, type HousePlan } from '../src/render/shared/house';

/** Rows a storey. */
const DEPTH = 12;
const W = 12;

/** A box house 6 by 5 tiles from (2, 2) on both storeys: windows front and back, the front door, a room split upstairs by a wall with a door, a diagonal window and doorway. */
function twoStoreys(): WorldStructure {
  const walls: WallEdge[] = [];
  const openings: Opening[] = [];
  const diagonals: DiagonalWall[] = [];
  const rooms = new Array(W * DEPTH * 2).fill(0);
  for (let k = 0; k < 2; k++) {
    const z0 = 2 + k * DEPTH;
    const edge = (axis: 'h' | 'v', x: number, z: number, kind?: 'door' | 'window') => {
      walls.push({ axis, x, z });
      if (kind) openings.push({ axis, x, z, kind });
    };
    for (let x = 2; x < 8; x++) {
      edge('h', x, z0, x === 3 || x === 6 ? 'window' : undefined);
      edge('h', x, z0 + 5, x === 4 || x === 5 ? 'window' : x === 6 && k === 0 ? 'door' : undefined);
    }
    for (let z = z0; z < z0 + 5; z++) {
      edge('v', 2, z, z === z0 + 2 ? 'window' : undefined);
      edge('v', 8, z, z === z0 + 2 ? 'window' : undefined);
      if (k === 1) edge('v', 5, z, z === z0 + 3 ? 'door' : undefined);
    }
    for (let z = z0; z < z0 + 5; z++) for (let x = 2; x < 8; x++) rooms[z * W + x] = k === 1 && x >= 5 ? 3 : k + 1;
    // A diagonal window and a diagonal doorway across two tiles.
    diagonals.push({ axis: 'dp', x: 3, z: z0 + 3, kind: 'window', rooms: [k + 1, k + 1] }, { axis: 'dn', x: 4, z: z0 + 3, kind: 'door', rooms: [k + 1, k + 1] });
  }
  return { width: W, depth: DEPTH * 2, storeyDepth: DEPTH, storeys: 2, walls, openings, diagonals, rooms, objects: [], plots: [], households: [], sims: [], exits: [] } as unknown as WorldStructure;
}

function storey(k: number): HousePlan {
  const world = twoStoreys();
  const plan = new HousePlanner().build(world, { x: 0, z: k * DEPTH, w: W, d: DEPTH }, k === 0 ? {} : { upper: true, covered: () => false });
  if (!plan) throw new Error('no house');
  return plan;
}

/** The triangles of the cut parts (the swinging door leaves' too) whose vertices differ in their cut, and how many triangles are hidden. */
function cuts(plan: HousePlan): { mixed: string[]; hidden: number } {
  const mixed: string[] = [];
  let hidden = 0;
  for (const part of [...plan.parts, ...plan.doors.flatMap((d) => d.parts)]) {
    if (!part.cut) continue;
    const { cut, idx } = part.geo;
    for (let t = 0; t < idx.length; t += 3) {
      const of = (v: number) => cut.slice(v * 4, v * 4 + 3).join(',');
      const [a, b, c] = [of(idx[t]), of(idx[t + 1]), of(idx[t + 2])];
      if (a !== b || a !== c) mixed.push(`${part.name}: ${a} | ${b} | ${c}`);
      else if (cut[idx[t] * 4 + 1] === Cut.Hide) hidden++;
    }
  }
  return { mixed, hidden };
}

test.each([
  ['the ground', 0],
  ['upstairs', 1],
])("every triangle of %s's walls, windows and doors has one cut", (_, k) => {
  const { mixed, hidden } = cuts(storey(k));
  expect(mixed.slice(0, 5)).toEqual([]);
  // (Windows and doors there to hide: the check sees some.)
  expect(hidden).toBeGreaterThan(100);
});

test('the shader sends a hidden vertex to one point, in GLSL and WGSL alike (not only lower: upstairs that is the room below)', () => {
  for (const lang of [CUTAWAY.glsl, CUTAWAY.wgsl]) {
    const hide = lang.vsLocal?.match(/game_wallCut\.y > 1\.5\) \{([^}]*)\}/)?.[1] ?? '';
    expect(hide).toMatch(/^\s*localPos = vec3f?\(0\.0, -2\.0, 0\.0\);\s*$/);
  }
});
