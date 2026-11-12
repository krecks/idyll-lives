/**
 * Where a fence or gate meets a house wall, the wall holds it: no post of its own stands in the wall
 * (its faces flickered in the wall's). A hedge beside a gate stops short of the gate's post instead of
 * burying it in its rounded end.
 */
import { expect, test } from 'bun:test';
import type { FenceEdge } from '../src/core/protocol';
import type { FenceStyleDef } from '../src/content/content';
import { fenceHedges, wallCorners, writeFences } from '../src/render/shared/fences';
import { Geo } from '../src/render/shared/geometry';
import { Painter, type Inst } from '../src/render/shared/street';

const STYLES = [
  { id: 'picket', label: 'Picket', kind: 'picket', color: '#ffffff' },
  { id: 'hedge', label: 'Hedge', kind: 'hedge', color: '#2f5a2a' },
] as FenceStyleDef[];
const gate = (x: number, z: number, style = 0): FenceEdge => ({ axis: 'h', x, z, kind: 'gate', style });
const hedge = (x: number, z: number): FenceEdge => ({ axis: 'h', x, z, kind: 'fence', style: 1 });

function vertices(fences: FenceEdge[], wallAt?: (x: number, z: number) => boolean): number {
  const p = new Painter(new Geo());
  writeFences(p, fences, STYLES, { gates: null, hedges: [], wallAt });
  return p.geo.pos.length;
}

test('the corners walls end or turn at', () => {
  const corners = wallCorners([
    { axis: 'h', x: 4, z: 2 },
    { axis: 'v', x: 4, z: 2 },
  ]);
  expect([...corners].sort()).toEqual(['4:2', '4:3', '5:2']);
});

test('a gate against a house wall has no post standing in the wall', () => {
  // A gate from (3, 2) to (4, 2); the house's wall turns at (4, 2).
  const walls = wallCorners([{ axis: 'v', x: 4, z: 2 }]);
  const free = vertices([gate(3, 2)]);
  const againstWall = vertices([gate(3, 2)], (x, z) => walls.has(`${x}:${z}`));
  // One of the gate's two posts is left out, the rest (its leaf, the other post) as it was.
  expect(againstWall).toBeLessThan(free);
  expect(againstWall).toBeGreaterThan(0);
  expect(vertices([gate(3, 2)], () => false)).toBe(free);
});

const along = (h: Inst) => h[2] * 1.05;

test('a hedge beside a gate stands clear of its post, at its size as planted', () => {
  // Hedge on (1, 0)-(2, 0), a gate on (2, 0)-(3, 0), hedge on (3, 0)-(4, 0); and a lone hedge further on.
  const [left, right, lone] = fenceHedges([hedge(1, 0), gate(2, 0, 1), hedge(3, 0), hedge(8, 0)], STYLES);
  // The lone stretch reaches a little past both ends of its edge (so stretches close up).
  expect(lone[0]).toBeCloseTo(8.5);
  expect(along(lone)).toBeGreaterThan(1);
  // Beside the gate: its end 0.12 m short of the post at x = 2 (and x = 3), moved back, not shrunk.
  expect(left[0] + along(left) / 2).toBeCloseTo(2 - 0.12, 5);
  expect(right[0] - along(right) / 2).toBeCloseTo(3 + 0.12, 5);
  expect(along(left)).toBeGreaterThan(1);
  expect(along(right)).toBeGreaterThan(1);
});

test('a hedge between two gates is shortened to fit between their posts', () => {
  const [between] = fenceHedges([gate(0, 0, 1), hedge(1, 0), gate(2, 0, 1)], STYLES);
  expect(between[0]).toBeCloseTo(1.5, 5);
  expect(between[0] - along(between) / 2).toBeCloseTo(1.12, 5);
  expect(between[0] + along(between) / 2).toBeCloseTo(1.88, 5);
});
