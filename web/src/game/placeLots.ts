/**
 * The town's named places as lots. The building of a civic place (a town hall, a church, a school, a
 * fire station: `places.ts` `CIVIC`) or of a place of another kind (a shop, a café, an office, a
 * workshop, a farm shop: `ZONE`) is a public lot of the sim's (`PlotSlot.kind` `shop`), built from a
 * layout of its own (`houses.json` `parks` of kind `place`, on a lot of that layout's size) and looking
 * from outside as a building of its kind (`PlotSlot.dress`, the town file's `meta.lotLooks`), as tall as
 * its real storeys.
 *
 * - `placeTemplate`: the layout a named place's lot is built from, and `placeOfTemplate` the other way.
 * - `placeUnit`: a named place's lot before it is placed (`LotPlacer.place`).
 * - `lotName`: the name it gets once placed, as its place is called.
 * - `slotPlace`: the place (`TownPlace`) any public lot is: its kind, name, tag, hours and the careers
 *   working there. A civic place keeps its kind's hours (`HOURS`) widened to its careers' shifts; the
 *   sim's own shops and the park keep theirs (`LOT_HOURS`, none).
 *
 * (`CIVIC` is only read when these are called: `places.ts` reaches this module's imports again, and
 * may not have made it yet as this module loads.)
 */

import type { Content } from '../content/content';
import type { LotLook, TownPlace, TownSize } from '../core/protocol';
import { CIVIC, HOURS, hoursFor, KIND_WORK, LOT_HOURS, LOT_WORK, placeName, ZONE, type Civic, type Kind, type Namer, type ZoneKind } from './places';
import { plotRect, type NeighbourhoodDraft, type PlotSlot, type PublicTemplate, type Templates, type Unplaced } from './town';

type Rand = () => number;

/** What a named place's lot is for: a civic place (its index into `CIVIC`), or a place of another kind. */
export type PlaceWhat = { civic: number } | { zone: ZoneKind };

const pick = <T>(rand: Rand, list: readonly T[]): T => list[Math.floor(rand() * list.length)];
const r2 = (v: number) => Math.round(v * 100) / 100;

/** The town sizes, smallest first (a civic place's layout for a size it names none for: the nearest smaller one's, else larger). */
const SIZES: readonly TownSize[] = ['hamlet', 'village', 'town', 'city'];

/**
 * The layout (a `parks` id of kind `place`) the lot of civic place `what` (a `CIVIC` entry) or of a
 * place of kind `what` is built from, in a place of `size`: a civic place's own for that size (a
 * hamlet's or a village's hall is the parish hall, a town's the town hall), one of the kind's
 * drawn from `rand` (a kind with one layout draws nothing); `compact`: one of its compact layouts (`ZONE`), if it has any.
 */
export function placeTemplate(what: Civic | ZoneKind, size: TownSize, rand: Rand, compact = false): string {
  if (typeof what === 'string') {
    const list = compact && ZONE[what].compact.length ? ZONE[what].compact : ZONE[what].templates;
    return list.length === 1 ? list[0] : pick(rand, list);
  }
  // (Its own for the size, else the nearest smaller size's, else the nearest larger's.)
  const at = SIZES.indexOf(size);
  const near = [...SIZES.slice(0, at + 1).reverse(), ...SIZES.slice(at + 1)].find((s) => what.templates[s]);
  return near ? (what.templates[near] ?? '') : '';
}

/** What a lot built from layout `template` is for (`placeTemplate` the other way round), or null: it is no named place's (the park, the sim's own shops, an outdoor spot). */
export function placeOfTemplate(template: string): PlaceWhat | null {
  const civic = CIVIC.findIndex((c) => Object.values(c.templates).includes(template));
  if (civic >= 0) return { civic };
  const zone = (Object.keys(ZONE) as ZoneKind[]).find((k) => ZONE[k].templates.includes(template) || ZONE[k].compact.includes(template));
  return zone ? { zone } : null;
}

/**
 * How a lot of `kind` built from layout `p` looks from outside: its use and roof as the kind's (`Kind`), its
 * colours, from `rand`; as many storeys as it really has (its ground floor, and the storey above if its layout
 * has one). What stands over it (`PlotSlot.upper`: flats over a shop, an office's floors) raises it with them.
 */
function dressOf(kind: Kind, p: PublicTemplate, rand: Rand): LotLook {
  return { use: kind.use, storeys: p.upper ? 2 : 1, roof: pick(rand, kind.roofs), look: Math.floor(rand() * 1024) };
}

/**
 * A named place's lot before it is placed (`LotPlacer.place`): civic place `what` (a `CIVIC` entry)
 * or a place of kind `what`, in a place of `size`. Its layout (`placeTemplate`; `compact`: one of its kind's
 * compact ones), its lot the layout's own (`w`, `d`), the layout's name until it is named (`lotName`), and how
 * it looks from outside (`dress`), drawn from `rand` in that order. A `shop` lot, as the sim's own shops.
 */
export function placeUnit(t: Templates, what: Civic | ZoneKind, size: TownSize, rand: Rand, compact = false): Unplaced {
  const template = placeTemplate(what, size, rand, compact);
  const p = t.parks.find((q) => q.id === template);
  if (!p || p.kind !== 'place') throw new Error(`no place layout "${template}"`);
  const frame = p.lot ?? t.plot;
  const kind = typeof what === 'string' ? ZONE[what].kind : (what.kinds?.[size] ?? what.kind);
  return { kind: 'shop', template, name: p.name, lot: 'medium', w: frame.width, d: frame.depth, dress: dressOf(kind, p, rand) };
}

/**
 * The name a named place's lot gets (its `PlotSlot.name`, and its place's): a civic place's own
 * (`Civic.name`: "Ashby School"), else one for a place of its kind (`placeName`; a
 * farm shop the farmer's), none twice (`namer.unique`: `namer` is its area's). Null: the lot is no
 * named place's.
 */
export function lotName(s: Pick<PlotSlot, 'template'>, namer: Namer): string | null {
  const what = placeOfTemplate(s.template);
  if (!what) return null;
  if ('civic' in what) return namer.unique(() => CIVIC[what.civic].name(namer));
  if (what.zone === 'farm') return namer.unique(() => `${namer.last()}'s Farm Shop`);
  return namer.unique(() => placeName(what.zone, namer));
}

/** The kind of place public lot `s` is: a named place's its own (`placeOfTemplate`); the market is a shop, the café a café, any other shop a shop, a park a park. */
export function lotPlaceKind(s: Pick<PlotSlot, 'kind' | 'template'>): TownPlace['kind'] {
  const what = placeOfTemplate(s.template);
  if (what) return 'civic' in what ? CIVIC[what.civic].place : what.zone;
  if (s.kind === 'outdoor') return 'outdoor';
  if (s.kind === 'park') return s.template === 'market' ? 'shop' : 'park';
  return s.template === 'cafe' ? 'cafe' : 'shop';
}

/**
 * The place (`TownPlace`) public lot `slot` of `draft` is: its kind (`lotPlaceKind`), its name, its tag
 * (at its door, the middle of its front; a park's and an outdoor spot's in its middle), its plot,
 * its opening hours (a named place's: its kind's, `HOURS`; the sim's own shops: `LOT_HOURS`; a park
 * and an outdoor spot none), each widened to the shifts of the careers working there (a civic place's
 * own, `Civic.work`; another kind's, `KIND_WORK`; the sim's own shops', `LOT_WORK`), and its area.
 *
 * Its name is the lot's. With `namer` (its area's, whose names in use it keeps) it is made one no other
 * place has (`namer.unique`): the lot's, else with the area's name before it (as a hamlet's café is
 * called by its hamlet's), else numbered.
 */
export function slotPlace(draft: Pick<NeighbourhoodDraft, 'name'> & { open?: { areas: readonly { name: string }[] } }, t: Templates, content: Content, slot: PlotSlot, namer?: Namer): TownPlace {
  const kind = lotPlaceKind(slot);
  const what = placeOfTemplate(slot.template);
  const area = slot.area ?? 0;
  let name = slot.name;
  if (namer) {
    const areaName = draft.open?.areas[area]?.name ?? draft.name;
    let tries = 0;
    name = namer.unique(() => (tries++ === 0 ? slot.name : `${areaName} ${slot.name.replace(/^The /, '')}`));
  }
  const r = plotRect(t, slot);
  const open = kind === 'park' || kind === 'outdoor';
  const front = ([[0.5, 0], [0, 0.5], [0.5, 1], [1, 0.5]] as const)[slot.facing];
  const [x, z] = open ? [r.x + r.w / 2, r.z + r.d / 2] : [r.x + r.w * front[0], r.z + r.d * front[1]];
  const work = what ? ('civic' in what ? CIVIC[what.civic].work : (KIND_WORK[kind] ?? [])) : (LOT_WORK[slot.template] ?? []);
  const base = what ? HOURS[kind as keyof typeof HOURS] : LOT_HOURS;
  return {
    name,
    kind,
    x: r2(x),
    z: r2(z),
    plot: slot.index,
    ...(open ? {} : { hours: hoursFor(base, work, content.careerCategories) }),
    ...(work.length ? { work: [...work] } : {}),
    area,
  };
}
