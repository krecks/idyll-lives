/**
 * Where a point of the town is, in words (the place pill under the top bar, `PlacePill.svelte`):
 * the lot there, named as the town map names it, and the area (a big world's hamlet) round it. And
 * a plot by its id as the views have it, a flat's too (`plotInfo`, `flatsIn`).
 */

import { flatOf, type FlatInfo, type PlotInfo } from '../core/protocol';
import { distanceToRect } from '../render/shared/chunks';

/** A lot as `placeAt` needs it. */
export interface PlacePlot {
  id: number;
  name: string;
  x: number;
  z: number;
  w: number;
  d: number;
  public: boolean;
  /** What the lot is (`PlotInfo.kind`): a building of flats is named by its own name, nobody's home itself. */
  kind?: PlotInfo['kind'];
}

/**
 * The town's ground lots and its flats, as the game's views hold them (`game`, a `TagWorld`, the world's
 * structure): a household's or a resident's plot may be a flat (an id past the ground lots, `flatOf`).
 */
export interface Lots<P extends PlacePlot = PlotInfo> {
  plots: readonly P[];
  flats?: readonly FlatInfo[];
}

/**
 * Plot `id` as a lot: a ground lot's own, or a flat's as a home of its own (its name, its rectangle on the
 * ground's rows, its walls' box that rectangle, one storey, never public; visitors arrive at its building's
 * street door). Undefined for none.
 */
export function plotInfo<P extends PlacePlot>(w: Lots<P>, id: number | null | undefined): P | PlotInfo | undefined {
  if (id == null) return undefined;
  // (`flatOf` reads no more of the ground lots than how many there are.)
  const f = flatOf(w as unknown as Lots, id);
  if (!f) return w.plots[id];
  const lot = w.plots[f.of] as Partial<PlotInfo> | undefined;
  return { id: f.id, name: f.name, x: f.x, z: f.z, w: f.w, d: f.d, public: false, entry: lot?.entry ?? null, house: [f.x, f.z, f.x + f.w, f.z + f.d], storeys: 1 };
}

/** The flats of ground lot `lot` (a building of flats', those over a place), by storey, then as the town file lists them. */
export function flatsIn(w: Pick<Lots<PlacePlot>, 'flats'>, lot: number): FlatInfo[] {
  return (w.flats ?? []).filter((f) => f.of === lot).sort((a, b) => a.storey - b.storey || a.id - b.id);
}

/** An area (a hamlet) as `placeAt` needs it. */
export interface PlaceArea {
  name: string;
  x: number;
  z: number;
  w: number;
  d: number;
}

/** A lot this near (m) still names the place when the point is just off it (the roaming camera's `FOCUS_REACH`). */
const REACH = 12;

/** How far (m) a point is from a rectangle (0 inside). */
const distance = (x: number, z: number, r: PlaceArea) => distanceToRect(x, z, r.x, r.z, r.w, r.d);

/** The lot's name: a public place's own (a building of flats' too), a household's ("The Smiths"), else `empty` (its address). */
export function lotTitle(p: PlacePlot, households: readonly { name: string; plot: number | null }[], empty = p.name): string {
  if (p.public || p.kind === 'flats') return p.name;
  const hh = households.find((h) => h.plot === p.id);
  return hh ? `The ${hh.name}s` : empty;
}

/** `lotTitle` within a sentence: "on the way to the Smiths". */
export function lotInSentence(p: PlacePlot, households: readonly { name: string; plot: number | null }[]): string {
  const name = lotTitle(p, households);
  return p.public || p.kind === 'flats' ? name : name.replace(/^The /, 'the ');
}

/** "The Smiths · Millbrook": the lot at (or near) `x`, `z` and the area round it; '' when it's neither. */
export function placeAt(x: number, z: number, plots: readonly PlacePlot[], households: readonly { name: string; plot: number | null }[], areas: readonly PlaceArea[]): string {
  let lot: PlacePlot | null = null;
  let best = REACH;
  for (const p of plots) {
    const d = distance(x, z, p);
    if (d >= best) continue;
    best = d;
    lot = p;
    if (d === 0) break;
  }
  const parts: string[] = [];
  if (lot) parts.push(lotTitle(lot, households));
  const area = areas.find((a) => distance(x, z, a) === 0);
  if (area && area.name !== parts[0]) parts.push(area.name);
  return parts.join(' · ');
}

/**
 * A resident on their way somewhere (`SimView.trip`), in words: "On the way home · back about
 * 17:40", "On the way to the Smiths · there about 14:20"; without the time when it isn't known
 * (the sim works it out only for residents whose details are shown). `home`: their household's
 * plot; `lots` the town's lots and flats (a trip may go to a flat); `time` writes a minute of day as
 * the player's clock does.
 */
export function tripText(
  trip: { to: number; at?: number },
  home: number | null,
  lots: Lots<PlacePlot>,
  households: readonly { name: string; plot: number | null }[],
  time: (minute: number) => string,
  ride?: SimRide,
): string {
  const when = (word: string) => (trip.at === undefined ? '' : ` · ${word} about ${time(trip.at)}`);
  if (trip.to === home) return `${ride ? rideText(ride.kind, 'home') : 'On the way home'}${when('back')}`;
  const p = plotInfo(lots, trip.to);
  const going = ride?.to === 'out' ? (ride.kind === 'bike' ? 'Cycling' : 'Riding the e-scooter') : 'On the way';
  return `${going} to ${p ? lotInSentence(p, households) : 'somewhere'}${when('there')}`;
}

/** What a resident rides on their way (`SimView.ride`). */
export type SimRide = { kind: 'bike' | 'scooter'; to: 'work' | 'school' | 'out' | 'home' };

/** On a bike or an e-scooter, in words: "Cycling to work", "Riding the e-scooter home", "Cycling out" (a visit or an outing, its place unknown here). */
export function rideText(kind: SimRide['kind'], to: SimRide['to']): string {
  return `${kind === 'bike' ? 'Cycling' : 'Riding the e-scooter'} ${to === 'home' || to === 'out' ? to : `to ${to}`}`;
}

/** The work taxi a resident waits for or rides (`SimView.taxi`, sim-core `taxi.rs`). */
export type SimTaxi = { state: 'wait' | 'work' | 'home'; at?: number };

/**
 * The work taxi, in words: "Waiting for the taxi", "Taxi to work · there about 08:40" (to school
 * for a pupil, `job` absent), "Taxi home". `time` writes a minute of day as the player's clock does.
 */
export function taxiText(taxi: SimTaxi, job: unknown, time: (minute: number) => string): string {
  if (taxi.state === 'wait') return 'Waiting for the taxi';
  if (taxi.state === 'home') return 'Taxi home';
  const when = taxi.at === undefined ? '' : ` · there about ${time(taxi.at)}`;
  return `Taxi to ${job ? 'work' : 'school'}${when}`;
}

/**
 * A resident on their way (`SimView`'s `taxi`, `trip` and `ride`), in words: the work taxi
 * (`taxiText`), else the trip (`tripText`), else the ride (`rideText`); '' when they aren't on
 * their way. `home`, `lots`, `households` and `time` as `tripText`'s.
 */
export function wayText(
  s: { trip?: { to: number; at?: number }; ride?: SimRide; taxi?: SimTaxi; job?: unknown },
  home: number | null,
  lots: Lots<PlacePlot>,
  households: readonly { name: string; plot: number | null }[],
  time: (minute: number) => string,
): string {
  if (s.taxi) return taxiText(s.taxi, s.job, time);
  if (s.trip) return tripText(s.trip, home, lots, households, time, s.ride);
  if (s.ride) return rideText(s.ride.kind, s.ride.to);
  return '';
}
