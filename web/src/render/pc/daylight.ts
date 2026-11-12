/**
 * How much of the day it is, 0 (night) to 1, for the shaders that draw something only daylight shows
 * (`game_daylight`, one value a device): the lawn's stripes and the shade round what stands on it
 * (`shaders/grass.ts`, `LAWN.night`) and the leaves' warm light from above (`nature.ts`, `LEAVES.night`).
 * Under the moon the stripes were bars on a dark lawn, the shade a black hole, and the warm tops of a hedge
 * the one thing the night's grade left yellow-green.
 *
 * Set with the lighting (`lotContents.ts`); a device nobody has told the time is at day (the gallery, the
 * previews).
 */

import type * as pc from './engine';

const told = new WeakSet<pc.GraphicsDevice>();

/** The time of day for `device`'s shaders: `day` 0 (night) to 1. */
export function setDaylight(device: pc.GraphicsDevice, day: number): void {
  told.add(device);
  device.scope.resolve('game_daylight').setValue(Math.min(1, Math.max(0, day)));
}

/** Day, unless `device` has been told the time: for whoever makes a material that reads it. */
export function daylightByDefault(device: pc.GraphicsDevice): void {
  if (!told.has(device)) setDaylight(device, 1);
}
