/**
 * The town's vendors and shopkeepers (sim-core `staff.rs`): staff who come in to work at the public
 * places, not residents. The simulation says where each stands and what they do (snapshot rows after
 * the residents', `header.staffCount`); here they get a look, made from their seed as a newcomer's is,
 * and become residents for the renderer only (rows `sims.length + k`, never listed, picked or saved).
 *
 * Also what the pie menu needs of them: whether someone serves at a thing (a post), and whether it's
 * open at a time of day (its content `staff.hours`).
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content, ObjectDef } from '../content/content';
import type { SimInfo, WorldStructure } from '../core/protocol';
import { lookAtStage, lookFromSeed, palette } from './household';

/** The servers of `world` as residents drawn after its own (ids from `sims.length` on). */
export function staffResidents(content: Content, assets: AssetRegistry, world: Pick<WorldStructure, 'sims' | 'staff'>): SimInfo[] {
  const stages = content.adultStages.filter((s) => s.id !== 'elder');
  const genders = content.genders.length ? content.genders.map((g) => g.id) : ['female', 'male'];
  return (world.staff ?? []).map((s, k) => {
    const seed = Math.imul(s.seed + 1, 0x9e3779b1) >>> 0;
    const gender = genders[seed % genders.length];
    const stage = stages[(seed >>> 8) % Math.max(1, stages.length)];
    const appearance = lookAtStage(lookFromSeed(assets, gender, seed), stage, palette(assets, 'palette.hair'));
    return { id: world.sims.length + k, name: `staff ${s.seed}`, household: -1, gender, attractedTo: [], appearance, traits: [], perks: [], stage: stage?.id };
  });
}

/** Whether someone serves at the object with stable id `object` (a post with a server). */
export function staffed(world: Pick<WorldStructure, 'staff'> | null | undefined, object: number): boolean {
  return !!world?.staff?.some((s) => s.posts.includes(object));
}

/** Whether a post of kind `def` is open at `minute` of the day (its `staff.hours`, past midnight when `from` is later). */
export function openAt(def: ObjectDef, minute: number): boolean {
  const hours = def.staff?.hours;
  if (!hours) return true;
  const [from, to] = hours;
  const h = minute / 60;
  return from <= to ? h >= from && h < to : h >= from || h < to;
}

/** An hour of the day as the clock shows it: `10:00`. */
export function clockTime(hour: number): string {
  const m = Math.round(hour * 60) % 1440;
  return `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}`;
}
