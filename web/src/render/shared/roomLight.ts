/**
 * A room's own light by night, from how much lamplight it has (`pc/lamps.ts` sums it, `RoomLightState.light` holds it
 * per room, `pc/shaders/chunks.ts`'s lighting patch applies it): the lamps' light off its walls and floor, and how
 * much of the sky's and the moon's light is left in it.
 *
 * Why: the lamps are lights without shadows or bounces, and the sky's fill and the moon lit a room as they light the
 * garden. Whatever no lamp faced in a lamplit room (the side of a fridge, a bed's far edge, the walls between the
 * lamps' pools) took the night's blue instead, and the night grade, which turns what is dark and not warm blue-grey,
 * made it bluer: a lit room was half warm, half lavender.
 */

import { tuned } from '../tune';

/**
 * A room is lamplit by how much lamplight it has for its size: wholly with `full` lamps of power 1 on in a room of up
 * to `area` tiles (a larger room needs as many more), a room's dim fill (a room without a lamp) counting as `fill` of
 * one. Low draws four lamps in all, so a big room there is partly lamplit, and keeps that much of the sky's light.
 * - `bounce`: the lamps' light off the walls and the floor, lighting from every side what no lamp faces, as a share of
 *   a lamp's brightness, in a room wholly lamplit.
 * - `sky`: the share of the sky's fill and ambient, and of the moon's light, left by night in a room wholly lamplit.
 *   Outdoors and in a room with its lights out it is 1: the moon lights a dark room through its windows, which is how
 *   a bedroom with everyone asleep is still seen; with the lamps on the eye is set to them, and the blue goes. It
 *   comes in over `night` of the time of day's lamp level (dusk to night): while the sky outside is still light, a
 *   room with its lamps on is as light as the day leaves it.
 * - `dayAmbient`, `dayFill`: by day, the share a room has of the sky's ambient (which comes from every side alike) and
 *   of its hemispheric fill (which comes from above: a table's top takes more of it than its side). Both 1: a room is
 *   lit as the garden is. Less ambient and more fill, and what faces up in a room stands off from what faces
 *   sideways. `dayTint`: the colour both take in a room (white: the sky's own, a blue under which cream walls are
 *   grey; the light in a room has come off its floor and walls). All three go back to the garden's over `night`, as
 *   the lamps come on (`roomDay`).
 */
export const ROOM = tuned('room', { sky: 0.15, bounce: 0.08, full: 1, fill: 0.3, area: 16, night: [0.4, 0.85], dayAmbient: 1, dayFill: 1, dayTint: '#FFFFFF' });

/**
 * A room's shares of the sky's ambient and of its fill at the time of day's lamp level `lamps` (0 by day, 1 at night),
 * and how much of `dayTint` they take (1 by day, 0 at night).
 */
export function roomDay(lamps: number, room: typeof ROOM = ROOM): { ambient: number; fill: number; tint: number } {
  const night = Math.min(1, Math.max(0, (lamps - room.night[0]) / (room.night[1] - room.night[0])));
  return { ambient: room.dayAmbient + (1 - room.dayAmbient) * night, fill: room.dayFill + (1 - room.dayFill) * night, tint: 1 - night };
}

/**
 * The light of a room of `area` tiles whose lamps' brightness sums to `flux` (a lamp of power 1 full on is 1), at the
 * time of day's lamp level `lamps` (0 by day, 1 at night): how far its bounce is up, and the sky's share (both 0..1).
 */
export function roomLight(flux: number, area: number, lamps: number, room: typeof ROOM = ROOM): { bounce: number; sky: number } {
  const lamplit = Math.min(1, Math.max(0, flux) / Math.max(1, area / room.area) / room.full);
  const night = Math.min(1, Math.max(0, (lamps - room.night[0]) / (room.night[1] - room.night[0])));
  return { bounce: lamplit, sky: 1 + (room.sky - 1) * lamplit * night };
}
