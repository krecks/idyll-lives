/**
 * Posts are drawn by the part of the street whose fences end at them: the lot in view's own fences
 * are the game's (hidden with the ground over the basement), so their posts are too, not the street's.
 */
import { expect, test } from 'bun:test';
import type { FenceEdge } from '../src/core/protocol';
import { writeFences } from '../src/render/shared/fences';
import { Geo } from '../src/render/shared/geometry';
import { Painter } from '../src/render/shared/street';

const fence = (x: number, z: number): FenceEdge => ({ axis: 'h', x, z, kind: 'fence' }) as FenceEdge;

function vertices(drawn: FenceEdge[], round: FenceEdge[]): number {
  const p = new Painter(new Geo());
  writeFences(p, drawn, [], { gates: null, hedges: [], postsFrom: round, postHere: () => true });
  return p.geo.pos.length;
}

test("the lot in view's fences leave no posts in the street's share", () => {
  const street = [fence(0, 0), fence(1, 0)];
  const lot = [fence(6, 0), fence(7, 0)];
  expect(vertices(street, [...street, ...lot])).toBe(vertices(street, street));
});

test('a corner shared with a fence drawn here keeps its post', () => {
  const street = [fence(0, 0)];
  const lot = [fence(1, 0)];
  // (The street's fence ends at x = 1, where the lot's begins: that post is drawn here.)
  expect(vertices(street, [...street, ...lot])).toBe(vertices(street, street));
  expect(vertices(street, street)).toBeGreaterThan(0);
});
