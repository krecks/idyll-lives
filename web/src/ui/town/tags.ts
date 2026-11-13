/**
 * The town tags' contents: who and what gets a tag, what it says, where it
 * stands and what a click on it does. Pure: from the game's views (`SimView`, the relationships, the `PlotInfo`s,
 * the town's places); which of them are up for the screen at a zoom (`placesInView`, `nearestFew`) and how they rank
 * there (`tagRank`). `TownTags.svelte` places them on screen each frame (`game/tagLayout.ts`, as the menus do).
 */

import type { HouseholdInfo, PlotInfo, RelationshipView, SimInfo, SimView, TownArea, TownPlace } from '../../core/protocol';
import { kinLabel, friendLabel, romanceLabel } from '../relationship';
import { lotInSentence, wayText } from '../place';
import type { TagKind } from './kinds';
import { openState, openText, type HomeForSale, type NameLabel } from './places';
import { schoolOf, workplaceOf } from './workplace';

/** Where a tag stands: over a resident (where they are drawn, each frame), or at a town point. */
export type TagAnchor = { sim: number } | { x: number; z: number };

/** What a click on a tag does. */
export type TagAction =
  | { kind: 'select'; sim: number }
  | { kind: 'home' }
  | { kind: 'person'; sim: number; plot: number | null }
  | { kind: 'place'; place: number }
  | { kind: 'sale'; plot: number };

export interface TownTag {
  key: string;
  kind: TagKind;
  title: string;
  detail: string;
  anchor: TagAnchor;
  /** Metres over the ground its stem points to. */
  lift: number;
  /** Its look: your household, your people, a place (open or closed), a home for sale, a street or an area. */
  tone: 'mine' | 'person' | 'place' | 'closed' | 'sale' | 'street' | 'area';
  action: TagAction | null;
  /** Shown only with the orbit's radius in this range (m): street names close, area names far. */
  radius?: [number, number];
  /** A place's: the area (`WorldMeta.areas`) it is in, 0 the main town. */
  area?: number;
  /** A place's: one of yours (the household or your people) is there. */
  ours?: boolean;
}

/** Where a resident is, as the tags tell it: on a lot, at a place (their work or school, by `workplace.ts`), or on their way or out. */
export interface Whereabouts {
  /** The place (`places` index) they count at, or null. */
  place: number | null;
  /** The status in words ("at work · Bakery · home 17:10"). */
  text: string;
  /** Away at work or school, at no place the town has: no spot to stand on. */
  unplaced: boolean;
}

export interface TagWorld {
  sims: readonly SimView[];
  roster: readonly Pick<SimInfo, 'id' | 'gender' | 'gone'>[];
  households: readonly HouseholdInfo[];
  plots: readonly PlotInfo[];
  relationships: readonly RelationshipView[];
  places: readonly TownPlace[];
  /** Places' index by their public lot. */
  placeOfPlot: ReadonlyMap<number, number>;
  minute: number;
  weekday: number;
  /** Writes a minute of the day as the player's clock does. */
  time: (minute: number) => string;
  days: readonly string[];
  /** Whether someone at life stage `stage` goes to school. */
  pupil: (stage: string | null | undefined) => boolean;
  /** The town's areas (`WorldMeta.areas`: a big world's hamlets), for a hamlet's place whose name another place has too. */
  areas?: readonly Pick<TownArea, 'name' | 'kind'>[];
}

/** Where `s` is, in words, and the place they count at. */
export function whereabouts(s: SimView, w: TagWorld): Whereabouts {
  const home = w.households[s.household]?.plot ?? null;
  const way = wayText(s, home, w.plots, w.households, w.time);
  if (way) return { place: null, text: way.charAt(0).toLowerCase() + way.slice(1), unplaced: false };
  if (s.awayUntil !== null) {
    const school = !s.job && w.pupil(s.stage);
    const place = school ? schoolOf(s.id, w.places) : workplaceOf(s.id, s.job?.category, w.places);
    const parts = [school ? 'at school' : 'at work'];
    if (place !== null) parts.push(w.places[place].name);
    parts.push(`home ${w.time(s.awayUntil)}`);
    return { place, text: parts.join(' · '), unplaced: place === null };
  }
  const at = s.visiting ?? s.plot;
  const place = at === null ? null : (w.placeOfPlot.get(at) ?? null);
  if (s.visiting !== null) {
    const p = w.plots[s.visiting];
    return { place, text: `visiting ${p ? lotInSentence(p, w.households) : 'friends'}`, unplaced: false };
  }
  if (s.plot === null) return { place: null, text: 'out', unplaced: false };
  if (s.plot === home) return { place: null, text: 'at home', unplaced: false };
  const p = w.plots[s.plot];
  return { place, text: p ? `at ${lotInSentence(p, w.households)}` : 'out', unplaced: false };
}

/** The anchor of a resident's tag: themselves, or the place they're away at; null when away at no place. */
function anchorOf(s: SimView, where: Whereabouts, places: readonly TownPlace[]): TagAnchor | null {
  if (s.awayUntil !== null && !(s.trip || s.taxi || s.ride)) {
    if (where.place === null) return null;
    const p = places[where.place];
    return { x: p.x, z: p.z };
  }
  return { sim: s.id };
}

/** How close a bond is (partners and family first), for picking and ordering your people. */
function closeness(r: RelationshipView): number {
  return r.friendship + Math.max(0, r.romance) * 0.5 + (r.partners ? 200 : 0) + (r.kin ? 100 : 0);
}

/** One of your people: who they are to the household (the closest bond any member has with them). */
export interface Person {
  sim: number;
  label: string;
  closeness: number;
}

/**
 * Your people: family, partners and friends (`friendLabel` "Friend" and up, both ways or not) of the household's
 * members, outside the household and still in town; the closest `max` of them.
 */
export function yourPeople(members: ReadonlySet<number>, w: Pick<TagWorld, 'relationships' | 'roster'>, max = 16): Person[] {
  const roster = new Map(w.roster.map((s) => [s.id, s]));
  const theirs = new Map<number, number>();
  for (const r of w.relationships) if (members.has(r.b)) theirs.set(r.a * 65536 + r.b, r.friendship);
  const best = new Map<number, RelationshipView>();
  for (const r of w.relationships) {
    if (!members.has(r.a) || members.has(r.b)) continue;
    const who = roster.get(r.b);
    if (!who || who.gone) continue;
    if (!r.kin && !r.partners && r.romance < 35 && r.friendship < 35) continue;
    const had = best.get(r.b);
    if (!had || closeness(r) > closeness(had)) best.set(r.b, r);
  }
  const out: Person[] = [];
  for (const [sim, r] of best) {
    const label = kinLabel(r.kin, roster.get(sim)?.gender) ?? romanceLabel(r) ?? friendLabel(r.friendship, theirs.get(r.b * 65536 + r.a) ?? 0);
    out.push({ sim, label, closeness: closeness(r) });
  }
  return out.sort((a, b) => b.closeness - a.closeness || a.sim - b.sim).slice(0, max);
}

/** The household's members' tags: name, what they're doing and where. Those away at no place go in `away`. */
export function householdTags(members: readonly SimView[], w: TagWorld): { tags: TownTag[]; away: { sim: SimView; text: string }[] } {
  const tags: TownTag[] = [];
  const away: { sim: SimView; text: string }[] = [];
  for (const s of members) {
    const where = whereabouts(s, w);
    const anchor = anchorOf(s, where, w.places);
    if (!anchor) {
      away.push({ sim: s, text: where.text });
      continue;
    }
    tags.push({ key: `hh:${s.id}`, kind: 'household', title: s.name, detail: where.text, anchor, lift: 2.2, tone: 'mine', action: { kind: 'select', sim: s.id } });
  }
  return { tags, away };
}

/**
 * The household's home's own tag, while none of the household's tags stands there (nobody is in): the home is marked
 * from anywhere in the town view. Over its house (its lot, without one). A click glides down to it.
 */
export function homeTag(home: PlotInfo | null | undefined, members: readonly SimView[], homeId: number | null): TownTag | null {
  if (!home || members.some((s) => s.plot === homeId && s.awayUntil === null)) return null;
  const [x0, z0, x1, z1] = home.house ?? [home.x, home.z, home.x + home.w, home.z + home.d];
  return { key: 'hh:home', kind: 'household', title: 'Home', detail: 'nobody in', anchor: { x: (x0 + x1) / 2, z: (z0 + z1) / 2 }, lift: 3 + 3 * (home.storeys ?? 1), tone: 'mine', action: { kind: 'home' } };
}

/** Your people's tags, where they are (not those at your home: they're in the play view, with you). */
export function peopleTags(people: readonly Person[], w: TagWorld, homePlot: number | null): TownTag[] {
  const out: TownTag[] = [];
  for (const p of people) {
    const s = w.sims.find((x) => x.id === p.sim);
    if (!s || (homePlot !== null && s.plot === homePlot)) continue;
    const where = whereabouts(s, w);
    const anchor = anchorOf(s, where, w.places);
    if (!anchor) continue;
    const plot = s.plot ?? w.households[s.household]?.plot ?? null;
    out.push({ key: `p:${s.id}`, kind: 'people', title: s.name, detail: `${p.label} · ${where.text}`, anchor, lift: 2.2, tone: 'person', action: { kind: 'person', sim: s.id, plot } });
  }
  return out;
}

/**
 * The places' tags: name, open or closed (and till when), which of `yours` (household and people: id → name) are
 * there, and how many are inside (on its lot, or working there). A hamlet's place whose name another place has too
 * ("Bakery" in two hamlets) says which hamlet first ("Millbrook · open till 18:00"); the main town's needs no saying.
 */
export function placeTags(w: TagWorld, yours: ReadonlyMap<number, string>): TownTag[] {
  const times = new Map<string, number>();
  for (const p of w.places) times.set(p.name, (times.get(p.name) ?? 0) + 1);
  const inside = new Array<number>(w.places.length).fill(0);
  const there: string[][] = w.places.map(() => []);
  for (const s of w.sims) {
    const where = whereabouts(s, w);
    if (where.place === null) continue;
    inside[where.place]++;
    const name = yours.get(s.id);
    if (name) there[where.place].push(name);
  }
  return w.places.map((p, i) => {
    const state = openState(p.hours, w.minute, w.weekday);
    const hamlet = p.area > 0 && (times.get(p.name) ?? 0) > 1 && w.areas?.[p.area]?.kind === 'hamlet' ? w.areas[p.area].name : '';
    const parts = [hamlet, openText(state, w.weekday, w.time, w.days)];
    if (there[i].length) parts.push(there[i].slice(0, 3).join(', ') + (there[i].length > 3 ? ` +${there[i].length - 3}` : ''));
    if (inside[i]) parts.push(`${inside[i]} inside`);
    return {
      key: `pl:${i}`,
      kind: 'places' as const,
      title: p.name,
      detail: parts.filter(Boolean).join(' · '),
      anchor: { x: p.x, z: p.z },
      lift: 7,
      tone: state.open ? ('place' as const) : ('closed' as const),
      action: { kind: 'place' as const, place: i },
      area: p.area,
      ours: there[i].length > 0,
    };
  });
}

/**
 * The kinds' ranks on screen (`game/tagLayout.ts`: the lower takes the room first; the household's own come before
 * all of them, and are shown however crowded it is). Your people; the places one of yours is at; the areas' names
 * (the town's and its hamlets', a few, which say where the rest are); the other places; the homes for sale; the
 * streets' names.
 */
export const TAG_RANK = { people: 1, ours: 2, area: 3, places: 4, sale: 5, street: 6 } as const;

/** `t`'s rank on screen (`TAG_RANK`); the household's own rank first. */
export function tagRank(t: Pick<TownTag, 'kind' | 'tone' | 'ours'>): number {
  switch (t.kind) {
    case 'household':
      return 0;
    case 'people':
      return TAG_RANK.people;
    case 'places':
      return t.ours ? TAG_RANK.ours : TAG_RANK.places;
    case 'sale':
      return TAG_RANK.sale;
    case 'names':
      return t.tone === 'area' ? TAG_RANK.area : TAG_RANK.street;
  }
}

/**
 * How many places' tags are up at once with the camera `radius` m out: a street's worth close in (27 at 200 m), a
 * dozen or so over the whole main town (14 at 400 m), six from the farthest out. A Big town has 70 places or more:
 * with them all up, 400 m out, a pile of 30 tags hid the middle of the town.
 */
export function mostPlaces(radius: number): number {
  return Math.max(6, Math.round(5400 / Math.max(1, radius)));
}

/** How far (m) round a hamlet the camera may look and still be looking into it (its places come up). */
export const HAMLET_REACH = 40;

/**
 * The places whose tags are up with the camera looking at `look` from `radius` m out (their keys): every place one
 * of yours is at; of the rest, the `mostPlaces(radius)` nearest the middle of the view, the main town's (area 0)
 * and those of a hamlet the camera looks into (its area, `HAMLET_REACH` round it). The other hamlets' places come
 * with the pointer on them: a big world's hamlets each have their shops, and from over the town they read as
 * repeats of its own.
 */
export function placesInView(
  tags: readonly Pick<TownTag, 'key' | 'kind' | 'anchor' | 'area' | 'ours'>[],
  areas: readonly Pick<TownArea, 'x' | 'z' | 'w' | 'd'>[],
  look: { x: number; z: number },
  radius: number,
): Set<string> {
  const into = (i: number) => {
    const a = areas[i];
    return !!a && look.x >= a.x - HAMLET_REACH && look.x <= a.x + a.w + HAMLET_REACH && look.z >= a.z - HAMLET_REACH && look.z <= a.z + a.d + HAMLET_REACH;
  };
  const out = new Set<string>();
  const rest: { key: string; far: number }[] = [];
  for (const t of tags) {
    if (t.kind !== 'places' || !('x' in t.anchor)) continue;
    if (t.ours) out.add(t.key);
    else if (!t.area || into(t.area)) rest.push({ key: t.key, far: Math.hypot(t.anchor.x - look.x, t.anchor.z - look.z) });
  }
  for (const key of nearestFew(rest, mostPlaces(radius))) out.add(key);
  return out;
}

/** Homes for sale show up to this far out (m of orbit radius): past it, a big town's dozens of them would bury the rest. */
export const SALE_REACH = 480;

/**
 * How many homes for sale show at once: the ones nearest the middle of the view (`nearestFew`). A Big town has
 * a couple of hundred, and with them all on, three tags in four were prices: the household's own tags and the
 * places were lost among them.
 */
export const SALE_MOST = 12;

/** How many homes for sale show at once with the camera `radius` m out: `SALE_MOST` close in, fewer farther (6 at 400 m), as the places thin. */
export function mostSales(radius: number): number {
  return Math.min(SALE_MOST, Math.round(2400 / Math.max(1, radius)));
}

/** The `most` keys nearest (least `far`), the first by key among equals: which of a crowded kind's tags show. */
export function nearestFew(tags: readonly { key: string; far: number }[], most: number): Set<string> {
  if (tags.length <= most) return new Set(tags.map((t) => t.key));
  const order = [...tags].sort((a, b) => a.far - b.far || (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
  return new Set(order.slice(0, Math.max(0, most)).map((t) => t.key));
}

/** The homes for sale: their price and size. */
export function saleTags(homes: readonly HomeForSale[], plots: readonly PlotInfo[], money: (n: number) => string): TownTag[] {
  return homes.map((h) => {
    const p = plots[h.plot];
    const size = h.floor ? `${h.storeys > 1 ? `${h.storeys} floors · ` : ''}${Math.round(h.floor)} m²` : `empty lot · ${h.w} × ${h.d} m`;
    return {
      key: `s:${h.plot}`,
      kind: 'sale' as const,
      title: h.price ? `${money(h.price)}+` : 'For sale',
      detail: size,
      anchor: { x: p.x + p.w / 2, z: p.z + p.d / 2 },
      lift: 6,
      tone: 'sale' as const,
      action: { kind: 'sale' as const, plot: h.plot },
      radius: [0, SALE_REACH] as [number, number],
    };
  });
}

/** Street names close in (up to 520 m out), area names farther out (from 200 m). */
export function nameTags(names: readonly NameLabel[]): TownTag[] {
  return names.map((n) => ({
    key: n.key,
    kind: 'names' as const,
    title: n.text,
    detail: '',
    anchor: { x: n.x, z: n.z },
    lift: 0.5,
    tone: n.area ? ('area' as const) : ('street' as const),
    action: null,
    radius: n.area ? ([200, Infinity] as [number, number]) : ([0, 520] as [number, number]),
  }));
}
