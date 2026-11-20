/**
 * What the Lot tool puts down, as its card (`LotCard.svelte`) sets it: a home (a house of a size with a
 * garden, or a building of flats), a farm, a public place, an outdoor spot or a civic place. Kept while the tool is put down
 * and taken up again.
 */

import type { GardenSize } from '../../game/housegen';
import { civicOffered } from '../../game/edit/buildup';
import { draftAreaSize } from '../../game/places';
import { CIVIC_PLACES, publicChoices, type LotChoice } from '../../game/edit/lots';
import { placeTemplate } from '../../game/placeLots';
import type { NeighbourhoodDraft, Templates } from '../../game/town';

export type LotGroup = 'home' | 'farm' | 'public' | 'outdoor' | 'civic';
/** A home is a house on its lot, or a building of flats. */
export type HomeKind = 'house' | 'flats';
export type HomeSize = 'small' | 'medium' | 'large';

export const lotChoice = $state({
  group: 'home' as LotGroup,
  home: 'house' as HomeKind,
  size: 'medium' as HomeSize,
  /** A home's garden; `any`: as the world's homes have theirs. */
  garden: 'any' as GardenSize | 'any',
  /** The public place's template and the outdoor spot's (empty: the first). */
  place: '',
  outdoor: '',
  /** The civic place (an index into `CIVIC_PLACES`; -1: none chosen, the first offered), and the area the pointer was last over (its choices: `civicLacking`). */
  civic: -1,
  civicArea: 0,
});

/** About how big a building of flats' lot is (m), for its ghost where it can't go: each is drawn where it goes. */
const FLATS_LOT = { width: 20, depth: 24 };

/** The outdoor spots' templates. */
export const outdoorChoices = (t: Templates): string[] => t.parks.filter((p) => p.kind === 'outdoor').map((p) => p.id);

/** The civic places area `area` of `draft` has not got (`buildup.ts` `civicOffered`): what the Lot tool may put down there. */
export function civicLacking(draft: NeighbourhoodDraft, area = lotChoice.civicArea): { civic: number; label: string }[] {
  return civicOffered(draft, area);
}

/** The civic place the Lot tool puts down in area `area`: the one chosen if the area lacks it, else the first it lacks; -1: none. */
export function civicChosen(draft: NeighbourhoodDraft, area = lotChoice.civicArea): number {
  const offered = civicLacking(draft, area);
  return offered.some((c) => c.civic === lotChoice.civic) ? lotChoice.civic : (offered[0]?.civic ?? -1);
}

/** What `lotChoice` puts down as a lot (`lots.ts` `LotChoice`), or null: a civic place, put down by `buildup.ts` `addCivic` (which of them the area under the pointer lacks: `civicChosen`). */
export function choiceOf(t: Templates): LotChoice | null {
  const c = lotChoice;
  switch (c.group) {
    case 'home':
      return c.home === 'flats' ? { kind: 'flats' } : { kind: 'house', size: c.size, ...(c.garden === 'any' ? {} : { garden: c.garden }) };
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

/** The size of the lot `lotChoice` puts down (m), about: for its ghost where it can't go (a civic place's its layout's lot in `draft`'s area under the pointer). */
export function choiceSize(t: Templates, draft?: NeighbourhoodDraft): { w: number; d: number } {
  const c = lotChoice;
  const frame =
    c.group === 'home'
      ? c.home === 'flats'
        ? FLATS_LOT
        : t.lotSizes?.[c.size]
      : c.group === 'farm'
        ? t.lotSizes?.farm
        : c.group === 'outdoor'
          ? t.parks.find((p) => p.id === (c.outdoor || outdoorChoices(t)[0]))?.lot
          : undefined;
  if (c.group === 'civic' && draft) {
    const what = CIVIC_PLACES[civicChosen(draft)];
    const lot = what && t.parks.find((p) => p.id === placeTemplate(what, draftAreaSize(draft, c.civicArea), () => 0))?.lot;
    if (lot) return { w: lot.width, d: lot.depth };
  }
  const f = frame ?? t.plot;
  return { w: f.width, d: f.depth };
}
