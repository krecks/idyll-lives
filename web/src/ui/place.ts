/**
 * Where a point of the town is, in words (the place pill under the top bar, `PlacePill.svelte`):
 * the lot there, named as the town map names it, and the area (a big world's hamlet) round it.
 */

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

/** The lot's name: a public place's own, a household's ("The Smiths"), else `empty` (its address). */
export function lotTitle(p: PlacePlot, households: readonly { name: string; plot: number | null }[], empty = p.name): string {
  if (p.public) return p.name;
  const hh = households.find((h) => h.plot === p.id);
  return hh ? `The ${hh.name}s` : empty;
}

/** A resident and the bus (`SimView.bus`), in words: "Waiting for the bus", "On the bus to work" (or school); '' otherwise (home on the bus: `tripText`). */
export function busText(s: { bus?: 'wait' | 'work' | 'home' | null; job: unknown }): string {
  if (s.bus === 'wait') return 'Waiting for the bus';
  if (s.bus === 'work') return `On the bus to ${s.job ? 'work' : 'school'}`;
  return '';
}

/** `lotTitle` within a sentence: "on the way to the Smiths". */
export function lotInSentence(p: PlacePlot, households: readonly { name: string; plot: number | null }[]): string {
  const name = lotTitle(p, households);
  return p.public ? name : name.replace(/^The /, 'the ');
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
 * plot; `time` writes a minute of day as the player's clock does. `bus`: home on the bus
 * ("On the bus home · back about 17:40").
 */
export function tripText(
  trip: { to: number; at?: number },
  home: number | null,
  plots: readonly PlacePlot[],
  households: readonly { name: string; plot: number | null }[],
  time: (minute: number) => string,
  ride?: SimRide,
  bus = false,
): string {
  const when = (word: string) => (trip.at === undefined ? '' : ` · ${word} about ${time(trip.at)}`);
  if (trip.to === home) return `${ride ? rideText(ride.kind, 'home') : bus ? 'On the bus home' : 'On the way home'}${when('back')}`;
  const p = plots[trip.to];
  return `On the way to ${p ? lotInSentence(p, households) : 'somewhere'}${when('there')}`;
}

/** What a resident rides on their way (`SimView.ride`). */
export type SimRide = { kind: 'bike' | 'scooter'; to: 'work' | 'school' | 'home' };

/** On a bike or an e-scooter, in words: "Cycling to work", "Riding the e-scooter home". */
export function rideText(kind: SimRide['kind'], to: SimRide['to']): string {
  return `${kind === 'bike' ? 'Cycling' : 'Riding the e-scooter'} ${to === 'home' ? 'home' : `to ${to}`}`;
}

/**
 * A resident on their way (`SimView`'s `bus`, `trip` and `ride`), in words: the bus first (`busText`),
 * then the trip (`tripText`), then the ride (`rideText`); '' when they aren't on their way. `home`,
 * `plots`, `households` and `time` as `tripText`'s.
 */
export function wayText(
  s: { bus?: 'wait' | 'work' | 'home' | null; job: unknown; trip?: { to: number; at?: number }; ride?: SimRide },
  home: number | null,
  plots: readonly PlacePlot[],
  households: readonly { name: string; plot: number | null }[],
  time: (minute: number) => string,
): string {
  const bus = busText(s);
  if (bus) return bus;
  if (s.trip) return tripText(s.trip, home, plots, households, time, s.ride, s.bus === 'home');
  if (s.ride) return rideText(s.ride.kind, s.ride.to);
  return '';
}
