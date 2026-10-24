/**
 * Time-of-day lighting: interpolates sun, sky and exposure between the look's keyframes
 * (`render/look.ts`). Pure math; the renderer applies the result.
 *
 * Rough physical ratios behind the keys: on a clear day direct sun is ~4-5x the sky's diffuse
 * light on the ground; at golden hour they are about equal; at night the "camera" compensates
 * the faint moonlight with a higher exposure and a blue tint.
 *
 * Engine-neutral: it writes only the fields of its colours (`r`, `g`, `b`, sRGB 0..1) and vectors
 * (`x`, `y`, `z`, game space), so a renderer can keep its engine's own types in them
 * (`createLighting(() => new Color3(), () => new Vector3())`).
 */

import type { Look } from '../look';

const SUNRISE = 6;
const SUNSET = 20;
/** Hours around sunrise/sunset over which the direct light fades out and back in (sun <-> moon). */
const HANDOVER = 0.4;

export interface RGB {
  r: number;
  g: number;
  b: number;
}

export interface XYZ {
  x: number;
  y: number;
  z: number;
}

export interface Lighting<C extends RGB = RGB, V extends XYZ = XYZ> {
  sunDirection: V;
  sunIntensity: number;
  sunColor: C;
  skyIntensity: number;
  skyColor: C;
  groundColor: C;
  clearColor: C;
  zenithColor: C;
  exposure: number;
  /** Scale for image-based lighting (scene.environmentIntensity). */
  envIntensity: number;
  /** Interior lamps and lit windows, 0..1. */
  lamps: number;
  /** 0 by day .. 1 at night (dusk and dawn in between): drives the moonlight colour grade. */
  night: number;
  /** Sun elevation factor 0 (horizon or night) .. 1 (noon), for sky effects. */
  sunUp: number;
}

/** Writes lighting for `minuteOfDay` into `out` (no allocations after the first call). */
export function lightingAt<L extends Lighting>(minuteOfDay: number, look: Look, out: L): L {
  const keys = look.keys;
  const hour = (minuteOfDay / 60) % 24;
  let i = 0;
  while (i < keys.length - 2 && keys[i + 1].hour <= hour) i++;
  const a = keys[i];
  const b = keys[i + 1];
  const t = (hour - a.hour) / (b.hour - a.hour);

  out.sunIntensity = lerp(a.sun, b.sun, t);
  out.skyIntensity = lerp(a.sky, b.sky, t);
  out.exposure = lerp(a.exposure, b.exposure, t);
  out.envIntensity = lerp(a.env, b.env, t);
  out.lamps = lerp(a.lamps, b.lamps, t);
  const n = Math.min(1, Math.max(0, (out.lamps - 0.3) / 0.7));
  out.night = n * n * (3 - 2 * n);
  lerpHex(a.sunColor, b.sunColor, t, out.sunColor);
  lerpHex(a.skyColor, b.skyColor, t, out.skyColor);
  lerpHex(a.groundColor, b.groundColor, t, out.groundColor);
  lerpHex(a.clear, b.clear, t, out.clearColor);
  lerpHex(a.zenith, b.zenith, t, out.zenithColor);

  // Sun arcs east -> west during the day (low and long-shadowed); at night a fixed high "moon"
  // keeps soft shadows.
  if (hour >= SUNRISE && hour <= SUNSET) {
    const arc = ((hour - SUNRISE) / (SUNSET - SUNRISE)) * Math.PI;
    out.sunUp = Math.sin(arc);
    setNormalized(out.sunDirection, -Math.cos(arc), -(0.22 + out.sunUp * 0.9) * look.sunHeight, 0.45);
  } else {
    out.sunUp = 0;
    setNormalized(out.sunDirection, 0.35, -1, 0.25);
  }
  // The light switches between the sun's arc and the fixed moon at sunrise/sunset: fade it to
  // nothing across the switch so shadows never jump (the sky fill carries the scene meanwhile).
  const edge = Math.min(Math.abs(hour - SUNRISE), Math.abs(hour - SUNSET));
  if (edge < HANDOVER) {
    const t = edge / HANDOVER;
    out.sunIntensity *= t * t * (3 - 2 * t);
  }
  return out;
}

/** A lighting to write into, its colours and vectors made by `color` and `vector` (plain objects by default). */
export function createLighting<C extends RGB = RGB, V extends XYZ = XYZ>(
  color: () => C = () => ({ r: 0, g: 0, b: 0 }) as C,
  vector: () => V = () => ({ x: 0, y: 0, z: 0 }) as V,
): Lighting<C, V> {
  const sunDirection = vector();
  setNormalized(sunDirection, 0, -1, 0);
  return {
    sunDirection,
    sunIntensity: 1,
    sunColor: color(),
    skyIntensity: 1,
    skyColor: color(),
    groundColor: color(),
    clearColor: color(),
    zenithColor: color(),
    exposure: 1,
    envIntensity: 1,
    lamps: 0,
    night: 0,
    sunUp: 0,
  };
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function setNormalized(v: XYZ, x: number, y: number, z: number): void {
  const len = Math.hypot(x, y, z) || 1;
  v.x = x / len;
  v.y = y / len;
  v.z = z / len;
}

/** `#rrggbb` as 0..1 channels (each byte / 255), parsed once each. */
const hexCache = new Map<string, readonly [number, number, number]>();
function hex(value: string): readonly [number, number, number] {
  let c = hexCache.get(value);
  if (!c) {
    const v = parseInt(value.slice(1, 7), 16);
    hexCache.set(value, (c = [((v >> 16) & 255) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255]));
  }
  return c;
}

function lerpHex(a: string, b: string, t: number, out: RGB): void {
  const x = hex(a);
  const y = hex(b);
  out.r = x[0] + (y[0] - x[0]) * t;
  out.g = x[1] + (y[1] - x[1]) * t;
  out.b = x[2] + (y[2] - x[2]) * t;
}
