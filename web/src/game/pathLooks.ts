/**
 * What a house's front path and drive are made of (`housegen.json` `paving`): each drawn by weight for the house's
 * class (its label; `default` for a class not listed), the drive now and then laid as the path is. The town writes the
 * look on each of the house's paths (`layoutTown`, `Rect.look`), so the renderer lays them in it
 * (`render/shared/paving.ts` `PATH_LOOK`) and a save keeps them. A path without one (a flat's, a place's, an
 * older town's) is the hexagonal pavers it always was.
 */

import { weighted } from './housegen';

/** The surfaces a garden path or drive is laid in. */
const PATH_LOOKS = ['hex', 'concrete', 'brick', 'flagstone', 'gravel', 'asphalt'] as const;
export type PathLook = (typeof PATH_LOOKS)[number];

export interface PavingSpec {
  /** By class label (`default`: any other), the front path's looks by weight. */
  path: Record<string, Record<string, number>>;
  /** The same for the drive. */
  drive: Record<string, Record<string, number>>;
  /** How often the drive is laid as the path is, where it may be (0..1). */
  match?: number;
}

const isLook = (k: string | null): k is PathLook => !!k && (PATH_LOOKS as readonly string[]).includes(k);

/** The front path's and the drive's looks of a house of class `cls`, from `rand` (null: the spec doesn't say). */
export function pathLooks(spec: PavingSpec | undefined, cls: string | undefined, rand: () => number): { path: PathLook; drive: PathLook } | null {
  if (!spec) return null;
  const of = (table: Record<string, Record<string, number>>) => table[cls ?? ''] ?? table.default ?? {};
  const path = weighted(rand, of(spec.path));
  const drives = of(spec.drive);
  const same = rand() < (spec.match ?? 0) && (drives[path ?? ''] ?? 0) > 0;
  const drive = same ? path : weighted(rand, drives);
  return isLook(path) && isLook(drive) ? { path, drive } : null;
}
