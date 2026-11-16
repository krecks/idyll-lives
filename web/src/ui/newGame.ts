/**
 * The new game's four steps (Land, Town, Household, Home) as `app` keeps them: the world's code as
 * the player has chosen it so far, the keys that tell a later step whether what it made still
 * stands, and step 1's figures for the land. Pure, so the flow's rules can be tested without a page.
 *
 * A step's draft carries the key of what it was made from. Step 1's land is keyed by its place,
 * size, hills, woods and seed, and by a fingerprint of the land as the player shaped it. Step 2's
 * town adds the town, the villages, the town's seed and where the player moved it; step 3's people
 * add the people's seed. A step whose key no longer matches is made again from the step before it.
 */

import type { Rle, WorldOptions } from '../core/protocol';
import { COVER, landHash, NONE, type Land } from '../game/land';
import { hash } from '../game/noise';
import { offered, type PeopleStreams } from '../game/people';
import type { Pt } from '../game/roads';
import type { NeighbourhoodDraft } from '../game/town';
import { defaultsOf, LAND_SIZE, LEGACY_KINDS, placeOf, sizeFor, townFits, type WorldKind } from '../game/world';
import { readCode, type WorldCode } from '../game/worldCode';

/** A world's code with every choice made: the town and the villages too (the place's own until step 2 changes them). */
export type Chosen = WorldCode & Required<Pick<WorldCode, 'town' | 'villages'>>;

/** Step 1's Size choices, smallest first, each with how wide its land is. */
export const SIZE_CHOICES: readonly { value: NonNullable<WorldOptions['size']>; label: string; metres: number }[] = (['small', 'medium', 'large'] as const).map((value) => ({
  value,
  label: value[0].toUpperCase() + value.slice(1),
  metres: LAND_SIZE[value],
}));

/** The town sizes and the villages' counts, smallest first. */
const TOWNS: readonly Chosen['town'][] = ['hamlet', 'village', 'town', 'city'];
const VILLAGES: readonly Chosen['villages'][] = ['none', 'few', 'many'];

/** A new world's code: place `kind` from `seed`, with the place's own options where `options` doesn't give one, and the land as big as its town and villages want unless it says. */
export function startCode(kind: WorldKind, seed: number, options: Partial<WorldOptions> = {}): Chosen {
  const o = { ...defaultsOf(kind), ...options };
  return fitted({ kind, size: o.size ?? sizeFor(o.town, o.villages), hills: o.hills, woods: o.woods, seed, town: o.town, villages: o.villages });
}

/**
 * `code` with a town and villages that fit its land (`townFits`): as they are if they fit, else
 * fewer villages first, then a smaller town, so the nearest choice to the player's is kept.
 */
export function fitted(code: Chosen): Chosen {
  if (townFits(code.size, code.town, code.villages)) return code;
  for (let t = TOWNS.indexOf(code.town); t >= 0; t--) {
    for (let v = t === TOWNS.indexOf(code.town) ? VILLAGES.indexOf(code.villages) : VILLAGES.length - 1; v >= 0; v--) {
      if (townFits(code.size, TOWNS[t], VILLAGES[v])) return { ...code, town: TOWNS[t], villages: VILLAGES[v] };
    }
  }
  return { ...code, town: 'hamlet', villages: 'none' };
}

/** `code` on another land seed: the town's and the people's own seeds, made for the old land, go with it. */
export function withSeed(code: Chosen, seed: number): Chosen {
  const { townSeed: _town, peopleSeed: _people, ...rest } = code;
  return { ...rest, seed };
}

/**
 * The code a player typed or pasted (`readCode`), over the one they have: a whole code is taken as
 * it is (the place's own town and villages where it names none); a bare number is a new land seed
 * for the place and options as they are. Null: not a code.
 */
export function codeFrom(text: string, current: Chosen): Chosen | null {
  const read = readCode(text);
  if (!read) return null;
  if (!('kind' in read)) return withSeed(current, read.seed);
  const place = defaultsOf(read.kind);
  return fitted({ town: place.town, villages: place.villages, ...read });
}

/**
 * The code step 1 opens with: `?code=` if the page's URL has one (a bare number: the seed), over
 * `?world=` and `?options=` (the timing tools' worlds; a kind from before places is its place at
 * its size), else the countryside's own from `seed`.
 */
export function firstCode(codeText: string | null, world: { kind: string; options: Partial<WorldOptions> } | null, seed: number): Chosen {
  const legacy = world && Object.hasOwn(LEGACY_KINDS, world.kind) ? LEGACY_KINDS[world.kind].town : undefined;
  const asked = startCode(placeOf(world?.kind ?? 'country'), seed, { ...(legacy ? { town: legacy } : {}), ...world?.options });
  return (codeText && codeFrom(codeText, asked)) || asked;
}

/** The options the land and town jobs are given for `code` (the land job takes the town and villages too: for now the land is made as their world's). */
export function optionsOf(code: Chosen): WorldOptions {
  return { size: code.size, hills: code.hills, woods: code.woods, town: code.town, villages: code.villages };
}

/** Step 1's land's key: its place, size, hills, woods and seed, and the land as shaped (none: as generated). */
export function landKey(code: WorldCode, shaped: Land | null | undefined): string {
  return `${code.kind}:${code.size}:${code.hills}:${code.woods}:${code.seed >>> 0}:${shaped?.edited ? landHash(shaped) : ''}`;
}

/** Step 2's town's key: the land's (`landKey`), its town and villages, its own seed, and where the player moved it (none: where the land has it). */
export function townKey(land: string, code: Chosen, site: Pt | null): string {
  return `${land}|${code.town}:${code.villages}:${code.townSeed ?? ''}:${site ? `${Math.round(site[0])},${Math.round(site[1])}` : ''}`;
}

/** What the people's seed is drawn from when the code has none of its own (`peopleSeedOf`). */
const PEOPLE_SALT = 0x9e0b1e;

/**
 * The seed `code`'s people are drawn from (`people.ts` `peopleStreams`): its own, once "New
 * neighbours" rolled one or a code typed in gave one; else a mix of the town's seed (the land's
 * while the town has none of its own), so a code without one still makes the same people.
 */
export function peopleSeedOf(code: WorldCode): number {
  if (code.peopleSeed !== undefined) return code.peopleSeed >>> 0;
  return Math.floor(hash(code.townSeed ?? code.seed, PEOPLE_SALT, 0x5eed) * 2 ** 32) >>> 0;
}

/** Step 3's key: step 2's town's (`townKey`) and the people's seed. The town is lived in again only when this changed. */
export function peopleKey(town: string, seed: number): string {
  return `${town}|${seed >>> 0}`;
}

/** Households left not yet offered, under which "Show others" starts the round again (or all of them, in a smaller town). */
const FRESH_LEAST = 10;

/**
 * The households step 3 offers once "Show others" has been pressed `page` times: each offer is
 * `offered` from one stream running on, among the households not offered yet, so every press shows
 * others; once fewer than ten are left the round starts again from all of them. Page 0 is
 * `offered(town, streams)` itself, so the same code offers the same households first.
 */
export function offerPage(town: NeighbourhoodDraft, streams: PeopleStreams, page: number): number[] {
  const rand = streams.offer();
  const running: PeopleStreams = { ...streams, offer: () => rand };
  const all = town.households;
  let shown = new Set<number>();
  let offer: number[] = [];
  for (let p = 0; p <= page; p++) {
    let rest = all.filter((h) => !shown.has(h.slot));
    if (rest.length < Math.min(FRESH_LEAST, all.length)) {
      shown = new Set();
      rest = all;
    }
    offer = offered({ ...town, households: rest }, running);
    for (const slot of offer) shown.add(slot);
  }
  return offer;
}

/** Step 1's figures: the land's size (m), and the share of it under water and under woods (0..1). */
export interface LandStats {
  width: number;
  depth: number;
  water: number;
  woods: number;
}

/**
 * `draft`'s land in figures: from `field` (the land as the brushes have it) when there is one,
 * else from the draft's water tiles and woods.
 */
export function landStats(draft: NeighbourhoodDraft, field?: Land | null): LandStats {
  const { width, depth } = draft;
  if (field) {
    let wet = 0;
    for (let k = 0; k < field.h.length; k++) if (field.wl[k] !== NONE && field.wl[k] > field.h[k]) wet++;
    let woods = 0;
    for (let k = 0; k < field.cover.length; k++) if (field.cover[k] === COVER.woods) woods++;
    return { width, depth, water: wet / Math.max(1, field.h.length), woods: woods / Math.max(1, field.cover.length) };
  }
  const area = Math.max(1, width * depth);
  const woods = (draft.open?.zones ?? []).filter((z) => z.kind === 'woods').reduce((sum, z) => sum + polygonArea(z.poly), 0);
  return { width, depth, water: wetTiles(draft.open?.water) / area, woods: Math.min(1, woods / area) };
}

/** The tiles a run-length mask (`Rle`) sets. */
function wetTiles(rle: Rle | undefined): number {
  let n = 0;
  for (let k = 0; rle && k + 1 < rle.length; k += 2) if (rle[k + 1]) n += rle[k];
  return n;
}

/** A polygon's area (m², either winding). */
function polygonArea(poly: readonly [number, number][]): number {
  let twice = 0;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) twice += (poly[j][0] + poly[i][0]) * (poly[j][1] - poly[i][1]);
  return Math.abs(twice) / 2;
}
