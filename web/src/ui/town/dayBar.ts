/**
 * The top bar's day bar (docs/design/living-town.md, The top bar): how far the day is from sunrise to sunset, the
 * sun's own hours (`render/shared/environment.ts`, the light's arc).
 */

import { SUNRISE, SUNSET } from '../../render/shared/environment';

export interface DayBar {
  /** 0 at sunrise, 1 at sunset; held at 0 before sunrise and 1 after sunset. */
  share: number;
  /** The sun is up. */
  day: boolean;
  sunrise: number;
  sunset: number;
}

/** The day bar at `minute` of the day. */
export function dayBar(minute: number): DayBar {
  const sunrise = SUNRISE * 60;
  const sunset = SUNSET * 60;
  const m = ((minute % 1440) + 1440) % 1440;
  return { share: Math.min(1, Math.max(0, (m - sunrise) / (sunset - sunrise))), day: m >= sunrise && m < sunset, sunrise, sunset };
}
