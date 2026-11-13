/**
 * Where a resident at work is shown: at the place whose `work` lists their
 * career's category; when several do, one picked by the resident's id, the same every time; when none do, nowhere
 * ("at work"). The sim doesn't say where anyone works: this is only where the tags put them.
 */

import type { TownPlace } from '../../core/protocol';

/** A small integer hash (so neighbours' ids don't all land on the same place). */
function spread(id: number): number {
  let h = Math.imul(id + 1, 0x9e3779b1) >>> 0;
  h ^= h >>> 15;
  return Math.imul(h, 0x85ebca6b) >>> 0;
}

/** The index (into `places`) of where a worker of `category` shows, picked stably by `simId`; null: none takes them. */
export function workplaceOf(simId: number, category: string | null | undefined, places: readonly Pick<TownPlace, 'work'>[]): number | null {
  if (!category) return null;
  const takers: number[] = [];
  places.forEach((p, i) => {
    if (p.work?.includes(category)) takers.push(i);
  });
  if (!takers.length) return null;
  return takers[spread(simId) % takers.length];
}

/** A pupil's school: one of the places of kind `school`, picked stably by `simId`; null: the town has none. */
export function schoolOf(simId: number, places: readonly Pick<TownPlace, 'kind'>[]): number | null {
  const schools: number[] = [];
  places.forEach((p, i) => {
    if (p.kind === 'school') schools.push(i);
  });
  return schools.length ? schools[spread(simId) % schools.length] : null;
}
