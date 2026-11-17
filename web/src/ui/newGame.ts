/**
 * The new game's five steps (Land, Streets & zones, Build, Household, Home) as `app` keeps them: the
 * world's code as the player has chosen it so far, the keys that tell a later step whether what it
 * made still stands, and step 1's figures for the land. Pure, so the flow's rules can be tested
 * without a page.
 *
 * A step's draft carries the key of what it was made from. Step 1's land is keyed by its place,
 * size, hills, woods and seed, and by a fingerprint of the land as the player shaped it. Step 2's
 * layout (`townKey`) adds the town, the villages, the town's seed and where the player moved it.
 * Step 3's built town (`buildKey`) adds the layout as the player left it and the build's own seed;
 * step 4's people add the people's seed (`peopleKey`). A step whose key no longer matches is made
 * again from the step before it.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import type { GameKind, Lifespan, Rle, WorldOptions } from '../core/protocol';
import type { RoadPlan } from '../core/protocol';
import type { TownHistory } from '../game/edit/history';
import type { HouseholdDraft } from '../game/household';
import { COVER, landHash, NONE, type Land } from '../game/land';
import { hash } from '../game/noise';
import { fillPlan, fillTown, offered, peopleStreams, type FillTier, type PeopleStreams } from '../game/people';
import type { Pt } from '../game/roads';
import type { StartRequest } from '../game/session';
import { EMPTY_LOT, withLot, type NeighbourhoodDraft, type Templates } from '../game/town';
import { defaultsOf, LAND_SIZE, LEGACY_KINDS, placeOf, sizeFor, townFits, type WorldKind } from '../game/world';
import { readCode, type WorldCode } from '../game/worldCode';
import { layoutHash } from '../game/uses';
import type { Proposal } from '../game/townJobs';

/**
 * The new game's five steps, in order, as the screens name them: each one's screen (`app.svelte.ts` `Screen`), its
 * name, and its button on to the next (the household and home steps word theirs as the choice made has it).
 */
export const STEPS: readonly { id: 'land' | 'streets' | 'town' | 'household' | 'home'; label: string; next?: string }[] = [
  { id: 'land', label: 'Land', next: 'Lay out the streets →' },
  { id: 'streets', label: 'Streets & zones', next: 'Build the town →' },
  { id: 'town', label: 'Build', next: 'Choose a household →' },
  { id: 'household', label: 'Household' },
  { id: 'home', label: 'Home' },
];

/** Step `n`'s eyebrow (from 1): "Step 2 of 5 · Streets & zones". */
export function stepLabel(n: number): string {
  return `Step ${n} of ${STEPS.length} · ${STEPS[n - 1]?.label ?? ''}`;
}

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

/** Step 2's layout's key (what its proposal is made from): the land's (`landKey`), its town and villages, its own seed, and where the player moved it (none: where the land has it). */
export function townKey(land: string, code: Chosen, site: Pt | null): string {
  return `${land}|${code.town}:${code.villages}:${code.townSeed ?? ''}:${site ? `${Math.round(site[0])},${Math.round(site[1])}` : ''}`;
}

/** A layout's fingerprint (`uses.ts`): what Build makes the town from. */
export { layoutHash };

/**
 * The built town's key: the layout's (`layoutKey`, what its proposal was made from: the town's key, `townKey`), the
 * layout as the player left it (`layoutHash` of its plan) and the build's own seed ("Build again" rolls another).
 */
export function buildKey(layoutKey: string, layoutHash: string, buildSeed: number): string {
  return `${layoutKey}|${layoutHash}:${buildSeed >>> 0}`;
}

/** What `app.svelte.ts` keeps of step 2's layout that the rules below read. */
export interface LayoutLike {
  key: string;
  proposal: Pick<Proposal, 'plan'>;
  plan?: NeighbourhoodDraft;
  history?: Pick<TownHistory, 'draft'>;
}

/** The layout as the player left it (what Build makes the town from): the edits' plan, else the proposal's. */
export function layoutPlan(layout: LayoutLike): NeighbourhoodDraft {
  return layout.history?.draft ?? layout.plan ?? layout.proposal.plan;
}

/** Step 3's key for `layout` built with `buildSeed` (`buildKey`). */
export function worldKeyOf(layout: LayoutLike, buildSeed: number): string {
  return buildKey(layout.key, layoutHash(layoutPlan(layout)), buildSeed);
}

/**
 * What a change asks of the steps after it, as the flow has it: a change to the land, the town, the villages, the
 * town's seed or its site proposes the layout again (`propose`); a change to the layout or the build's seed builds
 * the town again (`build`); a change to the built town makes its households again (`people`). Each asks first when
 * it throws away the player's own work: `ask` names what goes (the layout's edits, the built town's tweaks).
 */
export function whatGoes(change: 'land' | 'propose' | 'layout' | 'build', state: { layoutEdited: boolean; townTweaked: boolean }): { remake: ('layout' | 'town' | 'people')[]; ask: ('layout' | 'town')[] } {
  const remake: ('layout' | 'town' | 'people')[] = change === 'land' || change === 'propose' ? ['layout', 'town', 'people'] : ['town', 'people'];
  const ask: ('layout' | 'town')[] = [];
  if (remake.includes('layout') && state.layoutEdited) ask.push('layout');
  if (state.townTweaked) ask.push('town');
  return { remake, ask };
}

/** What `whatGoes` asks about, in words for the question: "your streets and zones, and your changes to the built town". */
export function goesWords(ask: readonly ('layout' | 'town')[]): string {
  const words = ask.map((a) => (a === 'layout' ? 'your streets and zones' : 'your changes to the built town'));
  return words.join(', and ');
}

/** The player's changes to a town, in words for a question: "your change", "your 3 changes" ("your changes" when not counted). */
export function changesWord(n: number): string {
  return n === 1 ? 'your change' : n > 1 ? `your ${n} changes` : 'your changes';
}

/** A road as `addedStreets` tells roads apart: its kind, width and line (its name and area left out, which Build gives again). */
const roadSig = (r: RoadPlan) => `${r.kind ?? 'street'}|${r.width}|${r.points.map(([x, z]) => `${x.toFixed(1)},${z.toFixed(1)}`).join(';')}`;

/**
 * Step 3's streets put into the layout too, so they don't make the town be built again: the roads `after` (the
 * built town's plan after an edit) has that `before` hadn't, added to `layout`'s roads (each once), and those
 * `before` had that `after` hasn't (a road split or bent by the new one's junction) taken out where the layout has
 * them. The layout itself when no road changed.
 */
export function addedStreets(layout: NeighbourhoodDraft, before: NeighbourhoodDraft, after: NeighbourhoodDraft): NeighbourhoodDraft {
  if (before.roads === after.roads) return layout;
  const was = new Set(before.roads.map(roadSig));
  const now = new Set(after.roads.map(roadSig));
  const fresh = after.roads.filter((r) => !was.has(roadSig(r)));
  // (Only an edit that drew a road changes the layout: one that only took roads away leaves it.)
  if (!fresh.length) return layout;
  const goneRoads = before.roads.filter((r) => !now.has(roadSig(r)));
  const had = new Set(layout.roads.map(roadSig));
  // (A piece of a road split by the new one's junction goes in only where the layout had that road, in its place.)
  const inLayout = goneRoads.filter((g) => had.has(roadSig(g)));
  const pieceOf = (r: RoadPlan) => goneRoads.find((g) => r.points.every((p) => onLine(p, g.points)));
  const added = fresh.filter((r) => {
    const whole = pieceOf(r);
    return !had.has(roadSig(r)) && (!whole || inLayout.includes(whole));
  });
  const out = new Set(inLayout.map(roadSig));
  return { ...layout, roads: [...layout.roads.filter((r) => !out.has(roadSig(r))), ...added] };
}

/** Whether point `p` is on the line through `pts` (within half a metre). */
function onLine(p: readonly [number, number], pts: readonly (readonly [number, number])[]): boolean {
  for (let k = 1; k < pts.length; k++) {
    const [a, b] = [pts[k - 1], pts[k]];
    const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
    const len2 = dx * dx + dz * dz;
    const u = len2 ? Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / len2)) : 0;
    if (Math.hypot(a[0] + u * dx - p[0], a[1] + u * dz - p[1]) <= 0.5) return true;
  }
  return false;
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

/** Step 4's key: step 3's built town's (`buildKey`) and the people's seed. The town is lived in again only when this changed. */
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

/** Step 4's choice: the household, the home chosen for it, and the game's choices. */
export interface MoveIn {
  /** Step 3's town with its neighbours (the first wave). */
  town: NeighbourhoodDraft;
  /** The home chosen. */
  slot: number;
  household: HouseholdDraft;
  /** The home of the town's household chosen in step 3 (null: a household made in Creation). */
  from: number | null;
  /** Start on an empty lot, the house there cleared away (not for a household staying in its own home). */
  emptyLot: boolean;
  mode: GameKind;
  lifespan: Lifespan;
  /** `peopleSeedOf` the world's code (none: the newcomers are drawn from a seed of their own). */
  peopleSeed?: number;
}

/**
 * The new game Move in starts. A household of the town's that stays in its home is taken over as
 * it is (`existing`). One that moves leaves its old home empty, for sale and for the fill like any
 * other, and moves into the new one as a household made in Creation does. Either way the homes
 * still for sale are filled on the way in (`fill`, `townOnMoveIn`).
 */
export function moveInRequest(m: MoveIn): Extract<StartRequest, { kind: 'new' }> {
  const common = { mode: m.mode, lifespan: m.lifespan, fill: true, ...(m.peopleSeed === undefined ? {} : { peopleSeed: m.peopleSeed }) };
  if (m.from === m.slot) return { kind: 'new', town: m.town, household: m.household, slot: m.slot, existing: true, ...common };
  let town = m.from === null ? m.town : { ...m.town, households: m.town.households.filter((h) => h.slot !== m.from) };
  if (m.emptyLot) town = withLot(town, m.slot, EMPTY_LOT);
  return { kind: 'new', town, household: m.household, slot: m.slot, ...common };
}

/**
 * The town a new game starts in (`session.ts`, before its town file is put together). With `fill`,
 * the second wave of neighbours (`people.ts` `fillTown`): every home still for sale lived in but for
 * the few cheapest kept, up to the neighbours' cap, nearest the player's home first, none of them
 * with the player's family name, at `tier` (`people.ts` `tierFor` the machine's graphics preset; full
 * by default). Drawn from the request's people seed (none: one from `random`).
 */
export function townOnMoveIn(
  content: Content,
  assets: AssetRegistry,
  t: Templates,
  request: Extract<StartRequest, { kind: 'new' }>,
  random: () => number = Math.random,
  tier: FillTier = 'full',
): NeighbourhoodDraft {
  if (!request.fill) return request.town;
  const streams = peopleStreams(request.peopleSeed ?? (random() * 2 ** 32) >>> 0);
  return fillTown(content, assets, t, request.town, request.slot, streams, { tier, avoid: [request.household.name] });
}

/**
 * The households whose houses step 4 has the town worker furnish ahead (`townJobs.ts`
 * `furnishAhead`), before the player has chosen a home: the town's own (`town`: step 3's) and those
 * Move in would move into the homes still for sale (`people.ts` `fillPlan` with no home chosen),
 * drawn as `townOnMoveIn` draws them (`household`: the player's, whose family name none takes). Once
 * a home is chosen a few may come out otherwise; Move in furnishes those.
 */
export function householdsAhead(
  content: Content,
  assets: AssetRegistry,
  t: Templates,
  town: NeighbourhoodDraft,
  peopleSeed: number,
  household: HouseholdDraft,
  tier: FillTier = 'full',
): { slot: number; household: HouseholdDraft }[] {
  return [...town.households, ...fillPlan(content, assets, t, town, undefined, peopleStreams(peopleSeed), { tier, avoid: [household.name] })];
}
