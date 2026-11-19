/**
 * The built surface the renderer draws a world on (engine-neutral): a world whose land its town
 * follows (`meta.land` seeded `'follow'`) has its roads, lots and the land round them worked out
 * once (`builtSurface.ts`, `builtSurfaceOf`), shared by everything that draws on it (the terrain,
 * the street, the movers); any other world (a classic one, an older save, a land drawn flat under its
 * town) has none, and everything on it stands at 0 as it always has.
 *
 * `Levels` answers the heights the street's meshes take, by road (the road objects of
 * `meta.roads`, whichever list of them a caller has) and by junction.
 */

import type { JunctionInfo, LandFile, RoadPlan, WorldStructure } from '../../core/protocol';
import { decodeLand, type Land } from '../../game/land';
import { builtSurfaceOf, type BuiltSurface, type SurfaceWorld } from './builtSurface';

/** Each world's surface by its land (`meta.land`, the same object in every world message of a game), with the junctions it was solved with. */
const made = new WeakMap<LandFile, { junctions: string; surface: BuiltSurface }>();

/** Whether a world's land is one its town follows (it has a surface). */
export function follows(world: { meta?: { land?: LandFile } | null } | null | undefined): boolean {
  return world?.meta?.land?.seeded === 'follow';
}

/** The junctions' identity for the cache (a world message may bring the same ones as a new array). */
export const junctionKey = (junctions: readonly JunctionInfo[] | undefined) => (junctions?.length ? JSON.stringify(junctions.map((j) => [j.x, j.z, j.arms.map((a) => [a.road, a.walkTo])])) : '');

/**
 * The surface of `world`'s built land, made once per land (null: not a land its town follows).
 * `land`: the land to solve it on, if not the world's own (`LandscapeLayout.setLand`: a land the
 * brushes change in place): then a surface of its own, not shared.
 */
export function surfaceOf(world: (SurfaceWorld & { meta?: { land?: LandFile } | null }) | null | undefined, land?: Land): BuiltSurface | null {
  if (!world) return null;
  if (land) return land.seeded === 'follow' ? builtSurfaceOf(world, land) : null;
  const file = world.meta?.land;
  if (!file || file.seeded !== 'follow') return null;
  const key = junctionKey(world.junctions);
  const had = made.get(file);
  if (had && had.junctions === key) return had.surface;
  const surface = builtSurfaceOf(world, decodeLand(file));
  made.set(file, { junctions: key, surface });
  return surface;
}

/**
 * `surface`, solved for `world` on its own land elsewhere (a landscape's, `LandscapeLayout.takeRoads`), is the one
 * `surfaceOf(world)` gives from now on: one surface for the land and what stands on it, not a second one solved.
 */
export function shareSurface(world: { junctions?: JunctionInfo[]; meta?: { land?: LandFile } | null }, surface: BuiltSurface): void {
  const file = world.meta?.land;
  if (file?.seeded === 'follow') made.set(file, { junctions: junctionKey(world.junctions), surface });
}

/** A junction as far as its height goes: where it is. */
interface Spot {
  x: number;
  z: number;
}

/** A junction the surface solved is the one drawn within this (m). */
const JUNCTION_MATCH = 6;

/**
 * The heights the street's meshes take on a surface: a road's at `s` m along it (any list of the
 * world's road objects: they are found in `meta.roads` by identity), a junction's, and the surface
 * anywhere (`at`). Null where the world has no surface (`levelsOf`).
 */
export class Levels {
  private readonly index = new Map<RoadPlan, number>();

  constructor(
    readonly surface: BuiltSurface,
    roads: readonly RoadPlan[],
  ) {
    roads.forEach((r, i) => this.index.set(r, i));
  }

  /** Road `road`'s surface (m) `s` m along it; a road not among the world's: the surface under its first point. */
  road(road: RoadPlan, s: number): number {
    const i = this.index.get(road);
    if (i === undefined) {
      const p = road.points[0];
      return p ? this.surface.at(p[0], p[1]) : 0;
    }
    return this.surface.roadY(i, s);
  }

  /** A junction's one height (m): the surface's junction there, else the road under its middle. */
  junction(j: Spot): number {
    let best: { y: number; d: number } | null = null;
    for (const h of this.junctions()) {
      const d = Math.hypot(h.x - j.x, h.z - j.z);
      if (d <= JUNCTION_MATCH && (!best || d < best.d)) best = { y: h.y, d };
    }
    return best ? best.y : this.surface.at(j.x, j.z);
  }
  private junctionList: { x: number; z: number; y: number }[] | null = null;
  private junctions(): { x: number; z: number; y: number }[] {
    return (this.junctionList ??= this.surface.junctionHeights);
  }

  /** The surface (m) at (`x`, `z`): a lot's pad, a road's height, else the graded land (`BuiltSurface.at`). */
  at(x: number, z: number): number {
    return this.surface.at(x, z);
  }
}

/** The levels of `world`'s surface over its roads (null: none, the world stands at 0). */
export function levelsOf(world: WorldStructure | null | undefined, roads: readonly RoadPlan[] = world?.meta?.roads ?? []): Levels | null {
  const surface = surfaceOf(world);
  return surface ? new Levels(surface, roads) : null;
}
