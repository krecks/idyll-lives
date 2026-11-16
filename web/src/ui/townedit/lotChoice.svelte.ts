/**
 * What the Lot tool puts down, as its card (`LotCard.svelte`) sets it: a home of a size with a
 * garden, a farm, a public place, an outdoor spot or a civic place. Kept while the tool is put down
 * and taken up again.
 */

import type { GardenSize } from '../../game/housegen';
import { CIVIC_PLACES, publicChoices, type LotChoice } from '../../game/edit/lots';
import type { NeighbourhoodDraft, Templates } from '../../game/town';

export type LotGroup = 'home' | 'farm' | 'public' | 'outdoor' | 'civic';
export type HomeSize = 'small' | 'medium' | 'large';

export const lotChoice = $state({
  group: 'home' as LotGroup,
  size: 'medium' as HomeSize,
  /** A home's garden; `any`: as the world's homes have theirs. */
  garden: 'any' as GardenSize | 'any',
  /** The public place's template and the outdoor spot's (empty: the first). */
  place: '',
  outdoor: '',
  /** The civic place (an index into `CIVIC_PLACES`; -1: none chosen). */
  civic: -1,
});

/** The outdoor spots' templates. */
export const outdoorChoices = (t: Templates): string[] => t.parks.filter((p) => p.kind === 'outdoor').map((p) => p.id);

/** The civic places `draft`'s main town has not got (`CIVIC_PLACES`, by their places' kinds and names). */
export function civicLacking(draft: NeighbourhoodDraft): { civic: number; label: string }[] {
  const places = (draft.open?.places ?? []).filter((p) => p.area === 0);
  return CIVIC_PLACES.flatMap((c, i) => (places.some((p) => p.kind === c.place && c.known.test(p.name)) ? [] : [{ civic: i, label: c.label }]));
}

/**
 * What `lotChoice` puts down (`lots.ts` `LotChoice`), or null where nothing can be put down yet:
 * a civic place (placing one by hand comes with build-up, `buildup.ts` `addCivic`).
 */
export function choiceOf(t: Templates): LotChoice | null {
  const c = lotChoice;
  switch (c.group) {
    case 'home':
      return { kind: 'house', size: c.size, ...(c.garden === 'any' ? {} : { garden: c.garden }) };
    case 'farm':
      return { kind: 'farm' };
    case 'public': {
      const template = c.place || publicChoices(t)[0];
      const p = t.parks.find((q) => q.id === template);
      return p ? { kind: p.kind === 'shop' ? 'shop' : 'park', template } : null;
    }
    case 'outdoor': {
      const template = c.outdoor || outdoorChoices(t)[0];
      return template ? { kind: 'outdoor', template } : null;
    }
    case 'civic':
      return null;
  }
}

/** The size of the lot `lotChoice` puts down (m), about: for its ghost where it can't go. */
export function choiceSize(t: Templates): { w: number; d: number } {
  const c = lotChoice;
  const frame =
    c.group === 'home'
      ? t.lotSizes?.[c.size]
      : c.group === 'farm'
        ? t.lotSizes?.farm
        : c.group === 'outdoor'
          ? t.parks.find((p) => p.id === (c.outdoor || outdoorChoices(t)[0]))?.lot
          : undefined;
  const f = frame ?? t.plot;
  return { w: f.width, d: f.depth };
}
