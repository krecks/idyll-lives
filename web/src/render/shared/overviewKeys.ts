/**
 * What the menus' town (`pc/overview.ts`) is built from, as keys: the overview builds again only when one
 * changes, and keeps the draws whose own key is the same. Engine-neutral, so the tests can check them.
 */

import type { WorldStructure } from '../../core/protocol';
import { hashString } from './chunks';
import { surfaceOf } from './landSurface';
import { townRoads } from './roads';

/** What the town's road mesh is made from (`townRoadMesh`): its roads and junctions, and on a land the town follows their heights. */
export function roadKey(world: WorldStructure): string {
  const surface = surfaceOf(world);
  return `${hashString(JSON.stringify([townRoads(world), world.junctions ?? null]))}|${surface ? surface.roadsSig() : '-'}`;
}

/**
 * Identity of a town layout: the overview builds again only when it changes. Its roads are in it (`roadKey`): a road
 * drawn with no lot along it (a country road out of town, a street with nothing along it) changes nothing else, and
 * the town would keep its old road mesh while the land under it was graded for the new one. `roads`: its `roadKey`,
 * if made already.
 */
export function townKey(world: WorldStructure, roads = roadKey(world)): string {
  return [
    world.meta?.seed ?? 0,
    world.width,
    world.depth,
    world.plots.map((p) => `${p.house?.join(',') ?? '-'}`).join(';'),
    world.openings.length,
    world.objects.length,
    world.objects.reduce((a, o) => a + o.x * 31 + o.z * 7 + o.rot, 0),
    (world.meta?.paths ?? []).map((r) => `${r.x},${r.z},${r.w}`).join(';'),
    JSON.stringify(world.meta?.world ?? null),
    world.meta?.home ?? '-',
    roads,
  ].join('|');
}

/** What a town's backdrop buildings are drawn from (`townBackdrop`): its buildings, its places' hours, and on a land it follows the surface's roads (as the road mesh is keyed: `roadKey`). */
export function backdropKey(world: WorldStructure): string {
  const surface = surfaceOf(world);
  return `${hashString(JSON.stringify([world.meta?.backdrop ?? null, (world.meta?.places ?? []).map((p) => p.hours ?? null)]))}|${surface ? surface.roadsSig() : '-'}`;
}
