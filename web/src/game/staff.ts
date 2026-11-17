/**
 * The town's vendors and shopkeepers (sim-core `staff.rs`): staff who come in to work at the public
 * places, not residents. The simulation says where each stands and what they do (snapshot rows after
 * the residents', `header.staffCount`); here they get a look, made from their seed as a newcomer's is,
 * in the uniform of the post they serve at, and become residents for the renderer only (rows
 * `sims.length + k`, never listed, picked or saved).
 *
 * Also what the pie menu needs of them: whether someone serves at a thing (a post), and whether it's
 * open at a time of day (its content `staff.hours`).
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content, ObjectDef } from '../content/content';
import type { SimInfo, WorldStructure } from '../core/protocol';
import { lookAtStage, lookFromSeed, palette } from './household';

/** What someone wears at work in place of their own clothes: the garments (outfit.ts ids) and their colours (`body`: the top's). */
interface Clothes {
  top: string;
  body: string;
  bottom: string;
  bottomColor: string;
  shoes: string;
  shoesColor: string;
}
/** A uniform: its clothes, and what those of a gender wear instead (a garment that body hasn't got: no suit for a woman's). */
type Uniform = Clothes & { by?: Record<string, Partial<Clothes>> };

/**
 * The uniforms, by the name a post's content `staff.uniform` gives (absent or unknown: `apron`). Their face, hair and
 * skin are their own.
 * - `apron`: a shop's, café's or stall's: a white shirt under a green apron (`top.apron`: the shirt painted on the
 *   body in white, outfit.ts `SHIRT.apron`; the apron a shell over the front of the trousers in the top's colour),
 *   dark trousers and dark shoes.
 * - `bar`: the same in black, behind a bar.
 * - `office`: a front desk's: a pale blue shirt (a blouse for a woman), dark trousers, dress shoes.
 * - `suit`: a hotel's or a bank's: a navy suit (a navy jacket and skirt for a woman).
 * - `service`: the duty desk of a police, fire or army station: navy shirt and trousers, black boots.
 * - `teacher`: a brown jacket over a shirt, grey trousers.
 * - `clergy`: black from top to toe.
 */
export const UNIFORMS: Readonly<Record<string, Uniform>> = {
  apron: { top: 'top.apron', body: '#3F6B52', bottom: 'bottom.trousers', bottomColor: '#2B2D33', shoes: 'shoes.sneakers', shoesColor: '#2B2D33' },
  bar: { top: 'top.apron', body: '#26272B', bottom: 'bottom.trousers', bottomColor: '#26272B', shoes: 'shoes.dress', shoesColor: '#1E1E22' },
  office: { top: 'top.shirt', body: '#D5E1EE', bottom: 'bottom.trousers', bottomColor: '#2B2D33', shoes: 'shoes.dress', shoesColor: '#1E1E22', by: { female: { top: 'top.blouse' } } },
  suit: { top: 'top.suit', body: '#2B3445', bottom: 'bottom.suit', bottomColor: '#2B3445', shoes: 'shoes.dress', shoesColor: '#1E1E22', by: { female: { top: 'top.jacket', bottom: 'bottom.skirt' } } },
  service: { top: 'top.shirt', body: '#26324F', bottom: 'bottom.trousers', bottomColor: '#1D2333', shoes: 'shoes.boots', shoesColor: '#1E1E22' },
  teacher: { top: 'top.jacket', body: '#6E5A45', bottom: 'bottom.trousers', bottomColor: '#4A5568', shoes: 'shoes.dress', shoesColor: '#3A2E26' },
  clergy: { top: 'top.long', body: '#1E1E22', bottom: 'bottom.trousers', bottomColor: '#1E1E22', shoes: 'shoes.dress', shoesColor: '#1E1E22' },
};

/** What someone of `gender` wears serving at a post of kind `def` (its `staff.uniform`, else the apron). */
export function uniformFor(def: ObjectDef | undefined, gender: string): Clothes {
  const { by, ...clothes } = UNIFORMS[def?.staff?.uniform ?? 'apron'] ?? UNIFORMS.apron;
  return { ...clothes, ...by?.[gender] };
}

/**
 * The post each server of `world` dresses for, by its kind: the first of their posts (`world.objects`, by stable id)
 * with a `staff.uniform`, else their first; undefined where it isn't among the things.
 */
export function staffPosts(content: Pick<Content, 'object'>, world: Pick<WorldStructure, 'staff'> & Partial<Pick<WorldStructure, 'objects'>>): (ObjectDef | undefined)[] {
  // (The kinds of the posts someone serves at, found in one pass over the things.)
  const posts = new Set((world.staff ?? []).flatMap((s) => s.posts));
  const kinds = new Map<number, ObjectDef | undefined>();
  for (const o of world.objects ?? []) if (posts.has(o.id)) kinds.set(o.id, content.object(o.def));
  return (world.staff ?? []).map((s) => s.posts.map((id) => kinds.get(id)).find((d) => d?.staff?.uniform) ?? kinds.get(s.posts[0]));
}

/** The servers of `world` as residents drawn after its own (ids from `sims.length` on), each in their post's uniform (`staffPosts`). */
export function staffResidents(content: Content, assets: AssetRegistry, world: Pick<WorldStructure, 'sims' | 'staff'> & Partial<Pick<WorldStructure, 'objects'>>): SimInfo[] {
  const stages = content.adultStages.filter((s) => s.id !== 'elder');
  const genders = content.genders.length ? content.genders.map((g) => g.id) : ['female', 'male'];
  const posts = staffPosts(content, world);
  return (world.staff ?? []).map((s, k) => {
    const seed = Math.imul(s.seed + 1, 0x9e3779b1) >>> 0;
    const gender = genders[seed % genders.length];
    const stage = stages[(seed >>> 8) % Math.max(1, stages.length)];
    const look = lookAtStage(lookFromSeed(assets, gender, seed), stage, palette(assets, 'palette.hair'));
    // (A garment the body hasn't got is passed over for one it has, `outfit.ts` `chosen`.)
    const appearance = { ...look, ...uniformFor(posts[k], gender) };
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
